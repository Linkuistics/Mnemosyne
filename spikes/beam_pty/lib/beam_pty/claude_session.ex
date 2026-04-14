defmodule BeamPty.ClaudeSession do
  @moduledoc """
  Thin wrapper around `:exec.run/2` that spawns Claude Code in
  `-p --input-format stream-json --output-format stream-json` mode and
  drives it over bidirectional pipes (not a PTY). The spike deliberately
  inverted its original PTY hypothesis: claude's stream-json protocol is
  pure NDJSON over stdio, and sub-C explicitly disavows interactive
  control, so a PTY is not only unnecessary but actively harmful —
  erlexec's `:pty + :stdin` combination does not wire the caller-facing
  pipe to the child's actual stdin fd, so the child sees no input.

  Output events arrive as `{:stdout, ospid, binary}` messages tagged with
  the OS pid. The caller reads events via `receive_chunks/2`, which runs
  each chunk through a sliding-buffer sentinel detector looking for the
  phase-prompt sentinel emitted by the LLM when it considers the task
  done.

  Termination is cooperative: SIGTERM with `kill_group` enabled, a
  configurable grace period, then SIGKILL if the child is still alive.
  """

  alias BeamPty.Sentinel

  @type option ::
          {:disallowed_tools, [String.t()]}
          | {:system_prompt, String.t()}
          | {:model, String.t()}
          | {:cwd, String.t()}
          | {:executable, String.t()}

  @default_executable "claude"

  @spec spawn(keyword()) :: {:ok, pid(), non_neg_integer()} | {:error, term()}
  def spawn(opts \\ []) do
    argv = build_argv(opts)

    exec_opts = [
      :monitor,
      :stdin,
      {:stdout, self()},
      {:stderr, self()},
      :kill_group,
      {:kill_timeout, 1}
    ]

    case :exec.run(argv, exec_opts) do
      {:ok, pid, ospid} -> {:ok, pid, ospid}
      other -> {:error, other}
    end
  end

  @spec send_user_message(non_neg_integer(), String.t()) :: :ok | {:error, term()}
  def send_user_message(ospid, text) when is_binary(text) do
    payload =
      Jason.encode!(%{
        "type" => "user",
        "message" => %{
          "role" => "user",
          "content" => [%{"type" => "text", "text" => text}]
        }
      })

    :exec.send(ospid, payload <> "\n")
  end

  @doc """
  Receive stdout/stderr chunks for up to `timeout_ms` milliseconds, feeding
  each chunk through a sentinel matcher. Returns when the sentinel matches,
  the child exits, or the deadline passes.

  Return shape: `{:match, chunks, stderr}`, `{:exit, status, chunks, stderr}`,
  or `{:timeout, chunks, stderr}`.
  """
  @spec receive_chunks(non_neg_integer(), keyword()) ::
          {:match, [String.t()], [String.t()]}
          | {:exit, integer(), [String.t()], [String.t()]}
          | {:timeout, [String.t()], [String.t()]}
  def receive_chunks(ospid, opts) do
    sentinel = Keyword.fetch!(opts, :sentinel)
    timeout = Keyword.get(opts, :timeout_ms, 60_000)
    deadline = System.monotonic_time(:millisecond) + timeout

    loop(ospid, Sentinel.new(sentinel), deadline, [], [])
  end

  defp loop(ospid, detector, deadline, stdout_acc, stderr_acc) do
    now = System.monotonic_time(:millisecond)
    remaining = max(0, deadline - now)

    receive do
      {stream, ^ospid, data} when stream in [:stdout, :stderr] ->
        chunk = to_string(data)
        bucket = if stream == :stdout, do: :stdout, else: :stderr

        case Sentinel.feed(detector, extract_assistant_text(chunk)) do
          {:match, _} ->
            {out, err} = push(bucket, chunk, stdout_acc, stderr_acc)
            {:match, Enum.reverse(out), Enum.reverse(err)}

          {:nomatch, det} ->
            {out, err} = push(bucket, chunk, stdout_acc, stderr_acc)
            loop(ospid, det, deadline, out, err)
        end

      {:DOWN, _ref, :process, _pid, {:exit_status, status}} ->
        {:exit, status, Enum.reverse(stdout_acc), Enum.reverse(stderr_acc)}

      {:DOWN, _ref, :process, _pid, :normal} ->
        {:exit, 0, Enum.reverse(stdout_acc), Enum.reverse(stderr_acc)}

      {:DOWN, _ref, :process, _pid, reason} ->
        {:exit, {:abnormal, reason}, Enum.reverse(stdout_acc), Enum.reverse(stderr_acc)}
    after
      remaining ->
        {:timeout, Enum.reverse(stdout_acc), Enum.reverse(stderr_acc)}
    end
  end

  defp push(:stdout, chunk, out, err), do: {[chunk | out], err}
  defp push(:stderr, chunk, out, err), do: {out, [chunk | err]}

  @doc """
  Extract assistant_text-equivalent content from a stream-json chunk. The
  spike is permissive: any text that looks like it could contain the
  sentinel is treated as assistant text. Real sub-C will decode each NDJSON
  line and filter to `type: "assistant"` events with `content[].text`.
  """
  @spec extract_assistant_text(String.t()) :: String.t()
  def extract_assistant_text(chunk) do
    chunk
    |> String.split("\n")
    |> Enum.flat_map(&decode_line/1)
    |> Enum.join("\n")
  end

  defp decode_line(""), do: []

  defp decode_line(line) do
    case Jason.decode(line) do
      {:ok, %{"type" => "assistant", "message" => %{"content" => content}}}
      when is_list(content) ->
        content
        |> Enum.flat_map(fn
          %{"type" => "text", "text" => text} -> [text]
          _ -> []
        end)

      _ ->
        []
    end
  end

  @spec terminate(non_neg_integer(), keyword()) :: :ok
  def terminate(ospid, opts \\ []) do
    grace_ms = Keyword.get(opts, :grace_ms, 500)
    :exec.kill(ospid, 15)
    wait_exit(ospid, grace_ms)
  end

  defp wait_exit(ospid, grace_ms) do
    receive do
      {:DOWN, _ref, :process, _pid, _reason} -> :ok
    after
      grace_ms ->
        _ = :exec.kill(ospid, 9)

        receive do
          {:DOWN, _ref, :process, _pid, _reason} -> :ok
        after
          2_000 -> :ok
        end
    end
  end

  defp build_argv(opts) do
    exe = Keyword.get(opts, :executable, @default_executable)
    resolved = resolve_executable!(exe)

    base = [
      resolved,
      "-p",
      "--input-format",
      "stream-json",
      "--output-format",
      "stream-json",
      "--verbose",
      "--setting-sources",
      "project,local",
      "--no-session-persistence"
    ]

    base
    |> add_model(Keyword.get(opts, :model))
    |> add_system_prompt(Keyword.get(opts, :system_prompt))
    |> add_disallowed_tools(Keyword.get(opts, :disallowed_tools))
  end

  defp resolve_executable!(exe) do
    cond do
      String.starts_with?(exe, "/") -> exe
      path = System.find_executable(exe) -> path
      true -> raise "executable #{inspect(exe)} not found on PATH"
    end
  end

  defp add_model(argv, nil), do: argv
  defp add_model(argv, model), do: argv ++ ["--model", model]

  defp add_system_prompt(argv, nil), do: argv
  defp add_system_prompt(argv, prompt), do: argv ++ ["--system-prompt", prompt]

  defp add_disallowed_tools(argv, nil), do: argv
  defp add_disallowed_tools(argv, []), do: argv

  defp add_disallowed_tools(argv, tools) when is_list(tools) do
    argv ++ ["--disallowed-tools" | tools]
  end
end
