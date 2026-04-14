defmodule BeamPty.Sentinel do
  @moduledoc """
  Sliding-buffer sentinel matcher. Feeds assistant-text chunks of arbitrary
  length into a buffer that retains only the last `sentinel_size - 1` bytes,
  so a sentinel straddling two or more chunks is still detected.

  The buffer never grows unboundedly: after each feed, everything older than
  the sentinel window is discarded. This is load-bearing for Mnemosyne's
  phase-cycle: assistant sessions can emit megabytes of text, and the
  detector must run the whole lifetime of the session without retaining
  the stream.
  """

  defstruct [:sentinel, :buffer]

  @type t :: %__MODULE__{sentinel: String.t(), buffer: String.t()}

  @spec new(String.t()) :: t()
  def new(sentinel) when is_binary(sentinel) and byte_size(sentinel) > 0 do
    %__MODULE__{sentinel: sentinel, buffer: ""}
  end

  @doc """
  Feed a new chunk. Returns `{:match, detector}` if the sentinel is now
  present in the buffer, otherwise `{:nomatch, detector}` with the window
  trimmed to retain only the bytes that could still form a prefix of the
  sentinel on the next feed.
  """
  @spec feed(t(), String.t()) :: {:match | :nomatch, t()}
  def feed(%__MODULE__{sentinel: sentinel, buffer: buffer} = det, chunk)
      when is_binary(chunk) do
    combined = buffer <> chunk

    if String.contains?(combined, sentinel) do
      {:match, %{det | buffer: combined}}
    else
      keep = max(0, byte_size(combined) - (byte_size(sentinel) - 1))
      trimmed = binary_part(combined, keep, byte_size(combined) - keep)
      {:nomatch, %{det | buffer: trimmed}}
    end
  end
end
