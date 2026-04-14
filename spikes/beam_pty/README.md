# BEAM PTY spike (sub-C unblock)

Validates that an Elixir daemon using
[`erlexec`](https://hex.pm/packages/erlexec) can drive the real `claude`
CLI under the sub-C harness contract:

1. bidirectional stream-json I/O
2. sentinel-string detection on the assistant-text stream
3. process-group termination (SIGTERM → 500 ms grace → SIGKILL)
4. configurable tool profiles per spawn
5. backpressure-friendly streaming output

## Result — PASS, with one corrected premise

**The PTY premise was wrong.** Claude Code's
`-p --input-format stream-json --output-format stream-json` mode is
pure NDJSON over stdio. It does not need a pseudo-terminal. When you
combine erlexec's `:pty` and `:stdin` options the caller-facing pipe is
not wired to the child's real stdin and the child reads nothing —
claude errors out with `Input must be provided either through stdin or
as a prompt argument when using --print`.

**Pipes work perfectly.** Spawning claude with

```elixir
:exec.run(argv, [:monitor, :stdin, {:stdout, self()}, {:stderr, self()},
                 :kill_group, {:kill_timeout, 1}])
```

gives:

- one `:exec.send(ospid, json_line)` per user message,
- `{:stdout, ospid, binary}` messages for each NDJSON line (one line
  per event: `system/init`, `rate_limit_event`, `assistant/thinking`,
  `assistant/text`, `result/success`),
- `{:DOWN, ospid, :process, pid, reason}` on exit.

The full sub-C adapter should use pipes, not a PTY. A PTY is only
required if sub-C ever wants to drive claude's interactive TUI
(slash commands, arrow-key selection, ANSI redraws) — which the memory
invariant "no slash commands in the harness — control forbidden,
observation required" explicitly rules out.

## What each probe verifies

### `test/sentinel_test.exs` (6 unit tests, run always)

- sentinel in a single chunk,
- sentinel split across two chunks,
- sentinel split one grapheme at a time,
- sliding window bounded to `sentinel_size - 1` bytes (no unbounded
  growth, important for long-lived sessions),
- no false positive on a prefix of the sentinel,
- no false positive on a distinct but overlapping phrase.

### `test/claude_session_test.exs` (2 live tests, tagged `:live`)

Run with `mix test --only live` (requires `claude` on PATH and network).

- **PTY spawn + stream-json**: spawns haiku, sends a user message
  instructing claude to end every response with the literal string
  `READY FOR THE NEXT PHASE`, drains stream-json output, feeds each
  `{"type":"assistant","message":{"content":[{"type":"text","text":…}]}}`
  event through the sentinel detector, asserts the sentinel fires.
- **process-group termination**: spawns `/bin/sh -c "sleep 60 & wait"`,
  captures the grandchild pid from the shell's stdout, SIGTERMs the
  group (`:exec.kill_group`), asserts the grandchild is dead after a
  500 ms grace period.

Both probes use pipes, not PTY. `:kill_group` is what makes the
termination probe pass — without it, only the direct shell child dies.

### Implicit: configurable tool profiles per spawn

`ClaudeSession.spawn/1` accepts `:disallowed_tools` which passes
through to `claude --disallowed-tools Bash Edit Write Read`. Claude
honours this at the CLI flag level; the spike doesn't need a separate
assertion — the tool-list visible in the `system/init` event reflects
whatever was passed.

### Implicit: backpressure-friendly streaming

erlexec delivers output chunks as BEAM messages to the caller process.
BEAM mailboxes are unbounded and the scheduler does its own
back-pressure via process-level scheduling. Draining with a `receive`
loop that processes chunks before accepting the next one is sufficient;
the spike's `loop/5` does exactly that.

## Known noise

- **cmux SessionStart hooks pollute output.** The user's global
  `settings.json` registers `SessionStart` / `Stop` / `Notification`
  hooks for cmux. They fire on any claude invocation and emit ~10 KB
  of JSON before the first assistant event. The spike sidesteps this
  with `--setting-sources project,local --no-session-persistence`,
  which excludes user-level settings.

- **`cat` test exited instantly without `:stdin`.** erlexec defaults
  stdin to `:null` when you pass `:pty` without `:stdin`. Passing
  `:stdin` (bare atom) creates a pipe the caller can write to via
  `:exec.send/2`.

## Running

```sh
brew install erlang elixir    # one-time, if not already installed
mix deps.get
mix test                      # sentinel unit tests (6 tests)
mix test --only live          # live claude + grandchild probes (2)
mix test --include live       # everything (8 tests)
```

Result with Erlang/OTP 28 + Elixir 1.19.5:
`Finished in 4.5 seconds. 8 tests, 0 failures.`

## Takeaway for sub-C

The BEAM harness story is unblocked. The sub-C amendment should:

1. Drop "PTY" from the stream-json path. Use `erlexec` with
   `[:monitor, :stdin, {:stdout, self()}, {:stderr, self()}, :kill_group]`.
2. Wrap the session in a `GenServer` per live session (one per
   PlanActor consultation or F-level reasoning task).
3. Parse each NDJSON line, dispatch to the sub-M telemetry boundary,
   and route tool-use events to the in-session query handler.
4. Detect the `{"type":"result"}` event as the protocol-level
   "turn over" signal, orthogonal to the phase-prompt sentinel (which
   signals task-level "done with the work").
5. Terminate with SIGTERM + kill_group, grace period 500 ms, fall back
   to SIGKILL. `{:kill_timeout, 1}` in erlexec options gives second-level
   granularity; for finer grain, send signals explicitly from the
   GenServer and wait on DOWN yourself.
