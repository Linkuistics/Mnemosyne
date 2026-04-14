### Session 10 (2026-04-14T23:03:06Z) — BEAM PTY spike PASS (with premise inversion)

- **Attempted**: validate that `erlexec` can drive the real `claude` CLI over
  a PTY with bidirectional stream-json, sentinel detection, process-group
  termination, configurable tool profiles, and backpressure-friendly output.
  Installed Erlang/OTP 28 + Elixir 1.19.5 via Homebrew. Scaffolded an
  Elixir mix project at `spikes/beam_pty/` with erlexec 2.2.3 and jason.
  Wrote a `BeamPty.Sentinel` sliding-buffer matcher (6 unit tests) and a
  `BeamPty.ClaudeSession` thin wrapper over `:exec.run/2` (2 live probes
  tagged `:live`).

- **Worked**: 8/8 tests pass. Pipes-only erlexec configuration
  (`[:monitor, :stdin, {:stdout, self()}, {:stderr, self()}, :kill_group,
  {:kill_timeout, 1}]`) cleanly drives claude end-to-end: `:exec.send/2`
  delivers NDJSON to stdin, `{:stdout, ospid, binary}` messages carry each
  stream-json event (init / rate_limit / assistant thinking / assistant
  text / result), `{:DOWN, ospid, :process, pid, reason}` fires on exit.
  Sentinel detector correctly finds `READY FOR THE NEXT PHASE` in claude's
  assistant text. Process-group termination (`:kill_group` + SIGTERM +
  500ms grace + SIGKILL) kills the grandchild of a `/bin/sh -c "sleep 60 &
  wait"` spawn. `--disallowed-tools` passes through at the CLI flag level
  and is visible in claude's `system/init` event.

- **Didn't work initially**: (1) `claude` on PATH resolves to a cmux
  wrapper script that injects `--session-id` and `--settings` flags
  incompatible with the probe — fixed by pointing at the real binary at
  `/Users/antony/.local/bin/claude`. (2) erlexec's `:pty + :stdin`
  combination: stdin is NOT wired to the child's real fd 0, so claude
  reads nothing and errors with "Input must be provided either through
  stdin or as a prompt argument when using --print". (3) DOWN message
  format from erlexec is custom: `{:DOWN, OsPid, :process, Pid, Reason}`
  — first element is the integer OS pid, not a monitor ref. (4) With
  PTY, all child output arrives tagged `:stderr` rather than `:stdout`
  because PTY slave merges both. (5) Sending `:eof` immediately after
  the user message closed stdin before claude could read it.

- **Suggests trying next**: absorb sub-C's amendment task (P1.3)
  immediately now that the approach is validated. The amendment should:
  (a) drop "PTY" from the stream-json path entirely; (b) specify
  pipes-only `erlexec` opts; (c) wrap the session in a `GenServer` with
  the NDJSON line parser and the sub-M telemetry boundary; (d) detect
  `{"type":"result"}` as the protocol-level "turn over" signal,
  orthogonal to the phase-prompt sentinel (task-level "done with the
  work"); (e) describe cmux-hook noise mitigation via
  `--setting-sources project,local --no-session-persistence`. Once
  amended, P3.1 (sub-F sibling plan scaffolding) is also unblocked.

- **Key learnings / discoveries**:
  - **The PTY premise was wrong.** Sub-C's stream-json path is stdio
    NDJSON, not a terminal. A PTY is only needed if sub-C ever wants to
    drive claude's interactive TUI (slash commands, arrow keys, ANSI
    redraws), which the memory invariant "no slash commands in the
    harness — control forbidden, observation required" explicitly rules
    out. This is a meaningful simplification for sub-C.
  - **erlexec uses a C++ port program**, not a NIF. `exec-port` is
    spawned as a separate OS process that handles PTY allocation,
    signals, and process groups without blocking BEAM schedulers. This
    is why erlexec can safely use `ptrace`, `setreuid`, and process
    groups.
  - **`:stdin` bare atom is required** if you want `:exec.send/2` to
    work. erlexec defaults stdin to `:null` (cat echo test exited with
    status 0 immediately otherwise).
  - **cmux SessionStart hooks pollute stream-json output**. Any claude
    invocation triggers ~10KB of hook JSON from user-global settings.
    `--setting-sources project,local` silences them cleanly.
  - **Sentinel sliding-buffer invariant**: window retained between
    feeds is exactly `sentinel_size - 1` bytes. Verified empirically
    after feeding 10KB of non-matching data (buffer stays at 23 bytes
    for the 24-byte sentinel). This is load-bearing for long-running
    phase sessions that may emit MB of assistant text.
  - **`{"type":"result"}`** is the protocol-level turn-over marker —
    complementary to the phase-prompt sentinel for task-level done.
    Sub-C should detect both.
