defmodule BeamPty.ClaudeSessionTest do
  @moduledoc """
  The live Claude Code PTY probes. These tests are tagged `:live` and
  invoke the real `claude` CLI over a PTY allocated via erlexec. They are
  skipped by default; run them with:

      mix test --only live

  The tests require `claude` on PATH and network access.
  """

  use ExUnit.Case, async: false

  alias BeamPty.ClaudeSession

  @moduletag :live
  @moduletag timeout: 180_000

  @sentinel "READY FOR THE NEXT PHASE"

  setup do
    exe = real_claude_executable()

    unless exe do
      flunk("no native claude executable found — skip this test module with --exclude live")
    end

    {:ok, claude_exe: exe}
  end

  defp real_claude_executable do
    candidates = [
      "/Users/antony/.local/bin/claude",
      "/opt/homebrew/bin/claude",
      "/usr/local/bin/claude"
    ]

    Enum.find(candidates, fn path ->
      File.exists?(path) and
        (case File.stat(path) do
           {:ok, %File.Stat{type: :regular}} ->
             {kind, 0} = System.cmd("file", ["-b", path])
             String.contains?(kind, "Mach-O") or String.contains?(kind, "ELF")

           _ ->
             false
         end)
    end)
  end

  test "PTY spawn + stream-json: claude replies and the sentinel fires", %{claude_exe: exe} do
    instruction =
      "Reply with exactly one short sentence, then on a new line write: " <>
        @sentinel

    {:ok, _pid, ospid} =
      ClaudeSession.spawn(
        executable: exe,
        model: "haiku",
        disallowed_tools: ["Bash", "Edit", "Write", "Read"],
        system_prompt:
          "You are a test probe. Reply briefly. End every response with exactly the literal string: " <>
            @sentinel
      )

    :ok = ClaudeSession.send_user_message(ospid, instruction)

    result = ClaudeSession.receive_chunks(ospid, sentinel: @sentinel, timeout_ms: 120_000)

    case result do
      {:match, _chunks, _stderr} ->
        :ok

      other ->
        flunk(
          "expected {:match, _, _} from sentinel detection but got:\n" <>
            inspect(other, pretty: true, limit: :infinity)
        )
    end

    # Clean up if still running.
    _ = ClaudeSession.terminate(ospid)
  end

  test "process-group termination kills a grandchild sleep" do
    # Spawn a shell that forks a background sleep and prints the sleep's pid,
    # then waits. When we SIGTERM the shell, the sleep must also die because
    # erlexec put them in the same process group.
    script =
      "sleep 60 & " <>
        "child=$!; " <>
        "echo CHILD_PID=$child; " <>
        "wait"

    {:ok, _pid, ospid} =
      :exec.run(
        ["/bin/sh", "-c", script],
        [:monitor, {:stdout, self()}, {:stderr, self()}, :kill_group, {:kill_timeout, 1}]
      )

    child_pid = await_child_pid(ospid, 5_000)
    assert is_integer(child_pid) and child_pid > 0, "did not capture grandchild pid"

    assert alive?(child_pid), "grandchild #{child_pid} should be alive before termination"

    :ok = ClaudeSession.terminate(ospid, grace_ms: 500)

    # Give the OS a moment to reap.
    :timer.sleep(200)
    refute alive?(child_pid), "grandchild #{child_pid} still alive after process-group kill"
  end

  defp await_child_pid(ospid, timeout_ms) do
    deadline = System.monotonic_time(:millisecond) + timeout_ms
    await_child_pid_loop(ospid, deadline, "")
  end

  defp await_child_pid_loop(ospid, deadline, buffer) do
    remaining = max(0, deadline - System.monotonic_time(:millisecond))

    receive do
      {stream, ^ospid, data} when stream in [:stdout, :stderr] ->
        new_buf = buffer <> to_string(data)

        case Regex.run(~r/CHILD_PID=(\d+)/, new_buf) do
          [_, pid_str] -> String.to_integer(pid_str)
          nil -> await_child_pid_loop(ospid, deadline, new_buf)
        end
    after
      remaining -> nil
    end
  end

  defp alive?(pid) when is_integer(pid) do
    case System.cmd("kill", ["-0", Integer.to_string(pid)], stderr_to_stdout: true) do
      {_, 0} -> true
      _ -> false
    end
  end
end
