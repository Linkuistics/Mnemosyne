defmodule BeamPty.SentinelTest do
  use ExUnit.Case, async: true

  alias BeamPty.Sentinel

  @sentinel "READY FOR THE NEXT PHASE"

  test "detects a sentinel arriving in a single chunk" do
    det = Sentinel.new(@sentinel)
    assert {:match, _} = Sentinel.feed(det, "some text READY FOR THE NEXT PHASE trailing")
  end

  test "detects a sentinel split across two chunks" do
    det = Sentinel.new(@sentinel)
    {:nomatch, det} = Sentinel.feed(det, "prefix READY FOR THE")
    assert {:match, _} = Sentinel.feed(det, " NEXT PHASE continuing")
  end

  test "detects a sentinel split one character at a time" do
    det = Sentinel.new(@sentinel)

    final_state =
      @sentinel
      |> String.graphemes()
      |> Enum.reduce_while({:nomatch, det}, fn ch, {:nomatch, d} ->
        case Sentinel.feed(d, ch) do
          {:match, _} = m -> {:halt, m}
          {:nomatch, _} = nm -> {:cont, nm}
        end
      end)

    assert match?({:match, _}, final_state)
  end

  test "retained window is bounded to sentinel-size - 1 bytes" do
    det = Sentinel.new(@sentinel)
    {:nomatch, det} = Sentinel.feed(det, String.duplicate("x", 10_000))
    assert byte_size(det.buffer) == byte_size(@sentinel) - 1
  end

  test "does not false-match a prefix of the sentinel alone" do
    det = Sentinel.new(@sentinel)
    assert {:nomatch, _} = Sentinel.feed(det, "READY FOR THE NEXT PHAS")
  end

  test "does not match a distinct but overlapping phrase" do
    det = Sentinel.new(@sentinel)
    assert {:nomatch, _} = Sentinel.feed(det, "READY FOR THE NEXT STEP")
  end
end
