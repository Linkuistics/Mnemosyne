# Backlog — Sub-project C: Harness Adapter Layer

Implementation backlog for sub-project C. All tasks derive from the design
doc at
`{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-C-adapters-design.md`
(committed at `71fd307`, amended at `b1a8cea`, **BEAM pivot amendment
§12 landed 2026-04-15**). Consult the spec before starting any task —
**§12 is the authoritative Elixir/BEAM projection** and supersedes every
Rust-specific implementation detail in §2–§8 wherever they conflict.

Tasks are listed in approximately recommended order, following the
implementation strategy in `{{PLAN}}/memory.md`: setup → day-1 verification
→ FixtureReplay → ClaudeCode wire layer → GenServer wiring → lifecycle →
instrumentation → tests → dogfood acceptance gate → conditional warm-pool.
The work phase picks the best next task with input from the user.

## ✅ BEAM PTY spike resolved + design doc §12 amendment landed — backlog rewrite required

The BEAM PTY spike (Session 10, 2026-04-15) validated that pipes-only
`erlexec` works perfectly for driving Claude Code's stream-json protocol.
The design doc §12 amendment (Session 11, 2026-04-15) projects every
Rust-specific element onto Elixir/OTP and is the starting point for the
rewritten backlog below. **The spike is no longer blocking, and the design
doc now matches the committed runtime.** Key findings from the spike:

1. **PTY premise was wrong** — `claude -p --input-format stream-json
   --output-format stream-json` is pure NDJSON over stdio; no
   pseudo-terminal needed.
2. **erlexec pipes-only works** — use `[:monitor, :stdin, {:stdout,
   self()}, {:stderr, self()}, :kill_group]`; the `:pty + :stdin`
   combination does NOT wire the caller's pipe to the child's real stdin.
3. Sentinel sliding-buffer detection validated.
4. Process-group termination validated.
5. cmux SessionStart hooks inject ~10KB of spurious JSON — silenced with
   `--setting-sources project,local --no-session-persistence`.
6. erlexec is a C++ port program (not a NIF), so no BEAM scheduler risks.
7. `{"type":"result"}` is protocol-level "turn over", orthogonal to
   task-level sentinel.
8. The Rust PTY-wrapper fallback is unnecessary.

Spike code at `spikes/beam_pty/`.

**All tasks below are still written against the Rust implementation
assumption** (Cargo.toml, `src/harness/`, crossbeam-channel, nix,
`std::process::Command`). They must be rewritten for Elixir/OTP before
implementation begins. The task *sequence* and *design intent* remain
valid; only the implementation technology changes.

See `memory.md` § "BEAM / Elixir pivot (Session 9 amendment)" for full
context.

## Task Backlog

### Task 1 — Cargo.toml deps + module skeleton `[setup]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Add the three new C-side deps to `Cargo.toml`:
  `which = "7"`, `nix = { version = "0.29", default-features = false, features = ["signal"] }`,
  `crossbeam-channel = "0.5"`. Confirm `serde_json`, `serde`, `chrono`,
  `thiserror` are present (they may be added by sub-B's earlier work).
  Create the empty module skeleton under `src/harness/`:
  `mod.rs`, `trait_def.rs`, `claude_code/{mod,session,actor,stream_json,spawn,input}.rs`,
  `fixture_replay/{mod,session,actor,format}.rs`. Each file should
  contain only a module-level doc comment and `pub use`s as needed for
  later tasks. Wire `pub mod harness;` into `src/lib.rs`. Run `cargo build`
  to confirm the skeleton compiles. No tests yet.
- **Results:** _pending_

### Task 2 — `trait_def.rs`: §3.1 trait surface (types only) `[types]`
- **Status:** not_started
- **Dependencies:** Task 1
- **Description:** Implement the §3.1 trait surface verbatim per the spec.
  Types: `HarnessAdapter` trait, `HarnessSession` trait (with `&self`
  signatures, `Send + Sync` bound, including the `send_user_message`
  method — the four trait amendments are baked in from day one),
  `HarnessKind`, `ToolProfile`, `OutputChunk`, `OutputChunkKind` (with
  `SessionLifecycle` variant), `SessionExitStatus`, `AdapterError`. No
  logic; just shapes with `#[derive(...)]` for `Debug`, `Clone`, `Copy`,
  `PartialEq`, `Eq` where appropriate. Pure compile-time correctness.
  Unit tests cover serde round-trips for any type that crosses the
  fixture-format boundary later (specifically `OutputChunkKind` and
  `SessionExitStatus`).
- **Results:** _pending_

### Task 3 — Day-1 verification of Claude Code CLI surface `[verification]`
- **Status:** not_started
- **Dependencies:** Task 1 (deps), Task 2 (types — for the chunk format
  reference)
- **Description:** Resolve §10 verification IOUs Q1-Q4 against the pinned
  `claude` version. Concretely:
  - Q1 (empty-tool-list spelling): run
    `claude --print --output-format stream-json --allowed-tools "" -p "use the Read tool to read /etc/hostname"`
    and verify denial in the captured output. Try the `--disallowed-tools "*"`
    fallback if `""` is interpreted as default. Lock the chosen flag
    spelling into `tool_profile_to_args`.
  - Q2 (`--permission-mode` choice): run a `--print` session that touches
    a tmpdir file under each candidate mode (`acceptEdits`,
    `bypassPermissions`, `dangerouslySkipPermissions`) and document the
    observed behaviour. Pick the least-surprising mode for v1.
  - Q3 (`--print "<prompt>"` + `--input-format stream-json`): run both
    "prompt as arg" and "prompt as first stdin envelope" shapes; pick
    whichever the pinned binary accepts.
  - Q4 (stream-json field names): capture full JSON output from a multi-
    turn session (text response + tool use + tool result + result event)
    via `claude --output-format stream-json --verbose --print "<prompt>"
    > capture.jsonl`. Commit the captured JSON to
    `tests/fixtures/harness/captured-stream-json/` as canonical samples.
    Update `stream_json.rs`'s draft serde structs to match observed
    field names exactly.
  Pin the verified `claude` version in
  `{{PROJECT}}/README.md` (or a new `tests/HARNESS_VERSION.md`) and add
  a `claude --version` check helper that warns if the running binary
  differs from the pinned version. Update `{{PLAN}}/memory.md`'s
  "Verified Claude Code CLI surface" section with all observed facts.
- **Results:** _pending_

### Task 4 — `tool_profile_to_args` helper `[tool-profile]`
- **Status:** not_started
- **Dependencies:** Task 3 (needs verified flag spellings)
- **Description:** Implement the `ToolProfile → Vec<String>` mapping in
  `src/harness/claude_code/spawn.rs` per §6.1 of the spec. Two profiles:
  `IngestionMinimal` and `ResearchBroad`, mapping to the verified flag
  spellings from Task 3. Unit tests assert exact arg vector contents for
  each profile.
- **Results:** _pending_

### Task 5 — `FixtureRecord` types + `format.rs` + serde round-trip tests `[fixture-replay]`
- **Status:** not_started
- **Dependencies:** Task 2 (uses `OutputChunkKind` + `SessionExitStatus`)
- **Description:** Implement `FixtureRecord` enum, `OutputChunkOnDisk`,
  `SessionExitStatusOnDisk` per §5.2 of the spec, with `Serialize` /
  `Deserialize` derives and `From` / `Into` conversions to/from the
  runtime types in `trait_def.rs`. Unit tests:
  - JSON Lines round-trip for every `FixtureRecord` variant.
  - `OutputChunkOnDisk ↔ OutputChunk` round-trip identity.
  - `SessionExitStatusOnDisk ↔ SessionExitStatus` round-trip identity.
  These are the lockstep tests that catch drift between the runtime and
  disk type pairs.
- **Results:** _pending_

### Task 6 — `FixtureReplaySession` actor + replay actor loop `[fixture-replay]`
- **Status:** not_started
- **Dependencies:** Task 5
- **Description:** Implement `FixtureReplaySession` per §5.4 and the
  replay actor loop per §5.5. Mirrors the `ClaudeCodeSession` actor
  architecture exactly: `cmd_tx` + `chunk_rx` + `terminated` + `actor_handle`,
  with a `FixtureCmd` inbox enum and a single actor thread that walks
  the fixture record list. Honours `Delay` (interruptible by `Terminate`),
  `ExpectUserInput` (blocks on inbox), `Output` (forwards), `Exit`
  (terminal). Uses `crossbeam_channel::select!` for the interruptible
  delay. No process spawn; no real harness.
- **Results:** _pending_

### Task 7 — `FixtureReplayAdapter` `[fixture-replay]`
- **Status:** not_started
- **Dependencies:** Task 6
- **Description:** Implement `FixtureReplayAdapter` per §5 — a struct
  that takes a fixture file path on construction and returns
  `FixtureReplaySession` handles from `spawn()`. The `prompt` and
  `tool_profile` arguments to `spawn()` are recorded for verification
  but otherwise ignored (the fixture is the source of truth for what
  the session emits). `kind()` returns `HarnessKind::FixtureReplay`.
  Unit test: spawn → drain `next_chunk` → assert all fixture chunks
  arrive in order.
- **Results:** _pending_

### Task 8 — Layer 2 fixture files + integration tests `[testing]`
- **Status:** not_started
- **Dependencies:** Task 7
- **Description:** Create the §8.2 fixture files under
  `tests/fixtures/harness/`:
  - `replay_clean_linear.jsonl` — output → output → exit
  - `replay_multi_turn.jsonl` — output → expect_user_input → output → exit
  - `replay_terminated.jsonl` — output → delay → (terminate during delay) → exit
  - `replay_tool_violation.jsonl` — assistant `tool_use` block under `IngestionMinimal`
  - `replay_crashed_before_ready.jsonl` — exit with no chunks within 100ms
  Plus integration tests in `tests/`:
  - Clean linear drain
  - Multi-turn with user interjection (driver thread sends `send_user_message`)
  - Mid-stream termination → next `next_chunk()` returns `Ok(None)`
  - Tool profile violation surfaces `UnsupportedToolProfile`
  - `CrashedBeforeReady` returned by `wait()`
  - Programmatic concurrent stress: two threads hammering `send_user_message`
    and `next_chunk` against a long-running fixture; assert no deadlocks,
    no panics, all chunks delivered in order.
  These tests exercise the actor architecture end-to-end without a real
  `claude` process. They will also drive sub-B's executor tests once B's
  implementation phase begins.
- **Results:** _pending_

### Task 9 — `stream_json.rs` event types + parser + `OutputChunk` mapping `[claude-code]`
- **Status:** not_started
- **Dependencies:** Task 3 (canonical JSON samples), Task 2 (`OutputChunk`
  type)
- **Description:** Implement `stream_json.rs` per §4.3.1 and §4.3.2.
  Define the `StreamJsonEvent`, `AssistantMessage`, `UserMessage`,
  `ContentBlock` serde structs against the canonical JSON samples
  captured in Task 3. Implement `event_to_chunks(event) -> Vec<OutputChunk>`
  per the §4.3.2 mapping table:
  - `system` (init) → `SessionLifecycle` with text `"ready"`
  - `system` (other subtype) → `InternalMessage` with JSON-serialised event
  - `assistant` `Text` block → `Stdout` chunk
  - `assistant` `ToolUse` block → `ToolUse` chunk with formatted name+input
  - `assistant` `ToolResult` block → `InternalMessage` chunk
  - `user` (echo) → `InternalMessage` chunk
  - `result` (turn boundary) → `SessionLifecycle` with text
    `"turn_complete:<subtype>"` AND record into `state.last_result`
  Unit tests use the canonical JSON samples from `tests/fixtures/harness/captured-stream-json/`
  and assert the expected chunk sequences.
- **Results:** _pending_

### Task 10 — `spawn.rs` (binary discovery + Command builder + process_group) `[claude-code]`
- **Status:** not_started
- **Dependencies:** Task 4 (`tool_profile_to_args`)
- **Description:** Implement `src/harness/claude_code/spawn.rs` per §4.1.1
  and §4.4.1 of the spec. Functions:
  - `find_claude_binary() -> Result<PathBuf, AdapterError>` using
    `which::which("claude")` → `AdapterError::HarnessNotFound` on miss.
  - `build_command(claude_path, prompt, working_dir, tool_profile, session_name) -> Command`
    that constructs the spawn args per §4.1.1 (with `--print --verbose
    --input-format stream-json --output-format stream-json --session-id
    <name>` plus the `tool_profile_to_args` flags), sets `current_dir`,
    pipes stdin/stdout/stderr, and calls `process_group(0)`.
  - `extract_pgid(child: &Child) -> nix::unistd::Pid` that converts
    `child.id()` to `Pid` for later `killpg` calls.
  Unit tests verify the constructed `Command`'s args exactly via
  `Command::get_args()`.
- **Results:** _pending_

### Task 11 — `input.rs` (user-message envelope serialisation) `[claude-code]`
- **Status:** not_started
- **Dependencies:** Task 1 (skeleton)
- **Description:** Implement `src/harness/claude_code/input.rs` per
  §4.1.2. A `serialise_user_message(text: &str) -> Result<String, serde_json::Error>`
  function that produces the stream-json user-message envelope as a
  newline-terminated JSON Lines string, ready to write to stdin. Unit
  tests assert the exact JSON shape against captured fixtures (a real
  user-message envelope from a captured session in Task 3).
- **Results:** _pending_

### Task 12 — `actor.rs` (the actor loop, ActorState, ActorCmd, ActorInbox) `[claude-code]`
- **Status:** not_started
- **Dependencies:** Task 9 (stream_json), Task 10 (spawn), Task 11 (input)
- **Description:** Implement `src/harness/claude_code/actor.rs` per §4.2.4.
  Define `ActorCmd`, `ActorInbox`, `ChunkOrEnd`, `ActorState`, and the
  `actor_loop` function. The actor loop dispatches:
  - `ActorInbox::Cmd(SendUserMessage)` → call `state.write_user_message`
  - `ActorInbox::Cmd(Terminate)` → call `state.terminate` (process-group
    kill — see Task 14)
  - `ActorInbox::Cmd(Wait(reply))` → call `state.wait_for_exit`, send
    reply, break
  - `ActorInbox::StdoutEvent(event)` → defence-in-depth tool check,
    `event_to_chunks`, forward to `chunk_tx`
  - `ActorInbox::StdoutEof` → emit `SessionLifecycle` `"exited:<status>"`
    chunk, then `ChunkOrEnd::End(Ok)`
  - `ActorInbox::StderrLine(line)` → record + forward as `Stderr` chunk
  - `ActorInbox::StreamParseError(err)` → emit `End(Err(...))`, terminate
  Unit tests use a fake stdout-event injector to drive the actor through
  every state transition without a real child process.
- **Results:** _pending_

### Task 13 — `session.rs` (`ClaudeCodeSession` handle + impl `HarnessSession`) `[claude-code]`
- **Status:** not_started
- **Dependencies:** Task 12 (actor)
- **Description:** Implement `src/harness/claude_code/session.rs` per
  §4.2.5. Define `ClaudeCodeSession` with its `crossbeam_channel`
  endpoints + `terminated` AtomicBool + `actor_handle` Mutex. Implement
  the `HarnessSession` trait methods:
  - `next_chunk` → `chunk_rx.recv()` mapped to `Ok(Some)` / `Ok(None)` / `Err`
  - `send_user_message` → check `terminated`, send `ActorCmd::SendUserMessage`
  - `terminate` → `swap` on `terminated` for idempotency, send `ActorCmd::Terminate`
  - `wait` → ad-hoc oneshot via `crossbeam_channel::bounded(1)`, send
    `ActorCmd::Wait(reply_tx)`, recv reply, join actor thread
  - `session_id` → return stored string
  Construction (`ClaudeCodeSession::new`) takes the actor handle, the
  channels, and the session ID; intended to be called only by the
  adapter's `spawn()`.
- **Results:** _pending_

### Task 14 — Process-group termination (killpg + 500ms kill-timer) `[lifecycle]`
- **Status:** not_started
- **Dependencies:** Task 12 (actor calls into this), Task 10 (pgid stored)
- **Description:** Implement the `state.terminate()` method on `ActorState`
  per §4.4.2. Phase 1: `killpg(pgid, SIGTERM)`. Drop stdin to signal EOF.
  Phase 2: spawn a fire-and-forget thread that sleeps 500ms then checks
  the `child_alive_flag` `Arc<AtomicBool>`; if still alive, `killpg(pgid,
  SIGKILL)`. The `child_alive_flag` is set to `false` by the stdout-reader
  thread when it observes EOF. Idempotent via the actor's local `terminated`
  bool. Tests:
  - Unit test: terminate sets the local flag and issues the SIGTERM call
    (mocked via a syscall-injecting test fixture, or via a dedicated
    signal-receiver child process).
  - Integration test (in Layer 3 task 21): `pgrep -f claude` post-terminate
    shows zero residual processes.
- **Results:** _pending_

### Task 15 — `CrashedBeforeReady` detection in `wait_for_exit` `[lifecycle]`
- **Status:** not_started
- **Dependencies:** Task 13 (session)
- **Description:** Implement the `CrashedBeforeReady` heuristic per
  §4.5.1: if the child exited within 2 seconds of spawn AND emitted
  zero output chunks AND exited unsuccessfully, return
  `SessionExitStatus::CrashedBeforeReady`. Otherwise classify via
  exit status / signal. Unit test uses `FixtureReplayAdapter` (which
  honours the same heuristic) with `replay_crashed_before_ready.jsonl`.
- **Results:** _pending_

### Task 16 — `claude_code/mod.rs` (`ClaudeCodeAdapter` + impl `HarnessAdapter`) `[claude-code]`
- **Status:** not_started
- **Dependencies:** Task 13 (session), Task 14 (terminate), Task 15 (crashed-before-ready)
- **Description:** Implement `ClaudeCodeAdapter` struct (holds the binary
  path + any startup-time config) and `impl HarnessAdapter`. The `spawn`
  method orchestrates the full setup: build `Command` (Task 10) → spawn
  child → extract stdin/stdout/stderr → spawn stdout-reader thread
  (which parses stream-json events and pushes to actor inbox) → spawn
  stderr-reader thread → spawn actor thread → return `Box<ClaudeCodeSession>`.
  The adapter's constructor (`ClaudeCodeAdapter::new()`) calls
  `find_claude_binary` and the `claude --version` pin check.
  Unit tests are integration-flavoured and live in Layer 3 (Task 21).
- **Results:** _pending_

### Task 17 — `SpawnLatencyReport` + always-on emission `[instrumentation]`
- **Status:** not_started
- **Dependencies:** Task 16 (adapter)
- **Description:** Implement `SpawnLatencyReport` per §7.2: a struct
  capturing `spawned_at`, `first_chunk_at`, `init_event_at`, plus the
  three derived metrics. The actor records timestamps at the appropriate
  points (when `Command::spawn` returns, when the first JSON event
  arrives, when the `system/init` event is observed) and emits the
  report as an `InternalMessage` chunk immediately after the first real
  chunk. The actor also writes the report to
  `<staging>/spawn-latency.json` — the `<staging>` path is passed in at
  spawn time as part of the working directory, so the actor knows where
  to write. Unit tests use `FixtureReplayAdapter` with timed fixtures
  to assert the report fields are populated correctly.
- **Results:** _pending_

### Task 18 — `mnemosyne dev record-fixture` subcommand `[dev-tools]`
- **Status:** not_started
- **Dependencies:** Task 16 (adapter), Task 5 (fixture format)
- **Description:** Implement the `dev record-fixture` subcommand per
  §5.6 in `src/commands/dev_record_fixture.rs`. CLI:
  `mnemosyne dev record-fixture --output <path> [--prompt <prompt>]
  [--profile research-broad|ingestion-minimal] [--max-delay-ms <ms>]
  [--interactive]`. Spawns a real `ClaudeCodeAdapter`, drives it with
  the prompt + profile, captures every `OutputChunk` into a `FixtureRecord::Output`
  and every wall-clock gap into a `FixtureRecord::Delay { ms }` (capped
  at `--max-delay-ms`, default 500). On `--interactive`, surfaces output
  to the user's terminal and accepts user input on stdin, recording
  both directions. Adds the `dev` subcommand namespace to `src/commands/mod.rs`
  with a "dev-only — not for end users" tag. Smoke test: record a
  trivial session and verify the produced JSON Lines parses back
  cleanly via `FixtureRecord::Deserialize`.
- **Results:** _pending_

### Task 19 — Layer 1 unit tests (parser + serde round-trips) `[testing]`
- **Status:** not_started
- **Dependencies:** Task 9 (stream_json), Task 5 (fixture format) — but
  this task tracks any unit tests not already written in earlier tasks
- **Description:** Catch-all for Layer 1 unit tests not yet covered by
  per-feature tasks. Particularly: edge cases in `stream_json::event_to_chunks`
  (empty content blocks, mixed text/tool_use blocks, missing optional
  fields, multi-block assistant messages with all-shared timestamps).
  Run with `cargo test --lib`.
- **Results:** _pending_

### Task 20 — Layer 2 concurrent stress test (deadlock canary) `[testing]`
- **Status:** not_started
- **Dependencies:** Task 13 (session), Task 7 (fixture adapter)
- **Description:** A programmatic test that spawns two threads against
  a single `Arc<dyn HarnessSession>` (one calling `send_user_message`
  in a loop, one calling `next_chunk` in a loop) over a long-running
  fixture (e.g., 10,000 chunks with `expect_user_input` interjected
  every 100 chunks). Asserts: no deadlocks (test completes within a
  generous timeout), no panics, all chunks delivered in order, all
  user-message commands processed. This is the regression test for the
  actor architecture's threading correctness.
- **Results:** _pending_

### Task 21 — Layer 3 integration tests against real `claude` binary `[testing]`
- **Status:** not_started
- **Dependencies:** Task 16 (adapter), Task 17 (instrumentation), Task 14 (process-group)
- **Description:** Implement the §8.3 Layer 3 test suite, all gated
  behind `MNEMOSYNE_TEST_LIVE_HARNESS=1`:
  - Smoke test: spawn + first chunk
  - Tool profile enforcement (`IngestionMinimal` denies tool use)
  - Tool profile enforcement (`ResearchBroad` allows tool use)
  - Multi-turn live (`send_user_message` after first turn → second turn
    references first-turn content)
  - Cold-spawn latency baseline: 10 spawn cycles, parse the captured
    `SpawnLatencyReport`s, assert p95 < 10s (test gate, looser than
    the C-1 acceptance gate)
  - Process group cleanup: spawn → terminate → assert `pgrep -f
    "claude.*<session-id>"` returns nothing within 1 second
  - `CrashedBeforeReady`: set bogus API key in env, spawn, assert
    `wait()` returns `CrashedBeforeReady` within 5s
  These tests are not run in normal CI; run locally during development
  and as part of the dogfood acceptance test (Task 22).
- **Results:** _pending_

### Task 22 — C-1 dogfood acceptance gate (the swap-in moment for B's stub) `[acceptance]`
- **Status:** not_started
- **Dependencies:** Task 21 (Layer 3 passes), and B's executor must be
  far enough along to accept the swap (coordinated with sub-B's plan)
- **Description:** The v1 acceptance gate for sub-project C and the
  joint v1 acceptance gate with sub-project B. Per §7.3 of the spec:
  - Swap B's `LlmHarnessExecutor` stub adapter for the real
    `ClaudeCodeAdapter`.
  - Run the Mnemosyne orchestrator plan's full work → reflect → triage
    cycle, **N≥10 times**, against the real adapter on the user's
    primary dev machine.
  - Walk every staging directory created during the runs and parse
    every `<staging>/spawn-latency.json` file.
  - Compute the cold-spawn latency p95 across all sessions in all
    cycles.
  - **Decision**: if p95 < 5 seconds, C-1 PASSES. Warm-pool work is
    deferred to v1.5. Document the decision in
    `{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md` with the
    measured p50/p95/p99 numbers and the date.
  - **Decision (alternate)**: if p95 ≥ 5 seconds, C-1 FAILS. Unblock
    Task 23 (warm-pool reset spike) and Task 24 (warm-pool implementation),
    add them to this plan as v1 work. Document the failure with
    measurements in the orchestrator parent's memory.md and proceed to
    Task 23.
  This task is the swap-in moment: the real `ClaudeCodeAdapter` becomes
  B's adapter of record. Until this passes, B keeps using fixture replay
  for end-to-end testing.
- **Results:** _pending_

### Task 23 — (CONDITIONAL) Warm-pool reset mechanism spike `[spike]`
- **Status:** not_started
- **Dependencies:** Task 22 = FAIL
- **Description:** Conditional on Task 22 tripping the C-1 gate. Execute
  the §7.4 three-check spike protocol against the pinned `claude` version:
  - Check 1 — Structured stream-json reset envelope. Inspect `claude --help`
    and recorded session output for any control envelope. If found,
    write a 30-line test that verifies fresh-context after reset.
  - Check 2 — `/clear` injected as user-message text. If Check 1 finds
    nothing, send `/clear` as the text content of a user-message envelope
    and verify fresh-context after reset.
  - Check 3 — Pre-spawned single-use degradation. If neither Check 1
    nor Check 2 works, fall back to the single-use pool model.
  Output: a markdown report at
  `tests/fixtures/sub-C-warm-pool-spike/results/spike-report.md`
  documenting which check passed and the chosen reset mechanism.
  ½ day estimated; runs against the real `claude` binary in a tmpdir.
- **Results:** _pending_

### Task 24 — (CONDITIONAL) v1 warm-pool implementation `[claude-code]`
- **Status:** not_started
- **Dependencies:** Task 23
- **Description:** Conditional on Task 22 tripping and Task 23 returning
  a usable reset mechanism. Implement `WarmPoolClaudeCodeAdapter` per
  the §7.5 sketch using the spike-validated reset path (Check 1 / Check 2 /
  Check 3 fallback). Per-profile pools (`ResearchBroad` depth 1-2,
  `IngestionMinimal` depth 3-4, configurable). Background spawner
  thread maintains target depth. `spawn()` pulls from pool → resets via
  validated mechanism → sends prompt as user-message envelope; falls
  back to fresh `Command::spawn` if pool empty. Re-run Task 22's dogfood
  cycles to verify p95 now passes. Update memory.md with the new
  measurements.
- **Results:** _pending_

### Task 25 — Adopt sub-M observability framework — actor instrumentation + parallel-emit migration of `SpawnLatencyReport` `[m-adoption]`
- **Status:** not_started
- **Dependencies:** sub-M Task 12 (`ObservabilityHarness`), sub-M Task 9
  (`MetricsRecorderLayer`), this plan's Task 18 (where
  `SpawnLatencyReport` is currently emitted)
- **Description:** Landed by sub-project M's brainstorm
  (2026-04-13, Session 7 of the mnemosyne-orchestrator plan). M's
  design doc at
  `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-M-observability-design.md`
  §9 specifies the staged migration of C's `SpawnLatencyReport` onto M's
  metrics framework via a parallel-emit window. This task is C's share.

  **Phase 1 — parallel emit (this task lands).** Add `tracing::instrument`
  annotations to `actor_loop` and the per-message handlers in C's actor.
  Span name `harness_session`, fields `session_id`, `tool_profile`,
  `harness_kind`. Span context propagates to all `mnemosyne_event!`
  calls inside the actor.

  Add parallel `metrics::histogram!` calls alongside C's existing
  `SpawnLatencyReport` writer at the three latency-measurement points:

  ```rust
  use crate::observability::metric_names::{
      HARNESS_COLD_SPAWN_LATENCY_MS,
      HARNESS_FIRST_CHUNK_LATENCY_MS,
      HARNESS_FIRST_OUTPUT_LATENCY_MS,
  };

  // alongside the existing report.write_to(<staging>/spawn-latency.json)
  metrics::histogram!(HARNESS_COLD_SPAWN_LATENCY_MS).record(cold_spawn_ms);
  metrics::histogram!(HARNESS_FIRST_CHUNK_LATENCY_MS).record(first_chunk_ms);
  metrics::histogram!(HARNESS_FIRST_OUTPUT_LATENCY_MS).record(first_output_ms);
  ```

  Also emit:
  - `metrics::counter!(HARNESS_SPAWNED).increment(1)` at successful spawn
  - `metrics::counter!(HARNESS_EXITED_CLEAN).increment(1)` at clean exit
  - `metrics::counter!(HARNESS_EXITED_ERROR).increment(1)` at error exit
  - `metrics::gauge!(HARNESS_LIVE_SESSIONS).set(...)` on spawn / terminate

  Wire the Risk 5 dump path: in C's actor error-handling branches
  (every place an `AdapterError` propagates out), call
  `observability::dump_event_tail(harness, session_id, plan_id, "harness", 1000)`
  before returning the error. The dump path is defined by sub-M Task 13.

  **DO NOT delete C's existing `SpawnLatencyReport` writer** in this
  task. Both paths run in parallel during the verification window.
  The verification test (sub-M Task 15) confirms the metric values
  match the JSON file within ±10ms. Only after ≥10 consecutive CI
  builds pass the verification does the v1.1 cleanup task delete C's
  tactical writer (created automatically by sub-M's first triage
  after the verification window closes).

  **Phase 2 — cleanup (separate future task).** After the verification
  window closes, a new task lands in this backlog from sub-M's triage
  that:
  1. Deletes C's `SpawnLatencyReport` struct definition.
  2. Deletes the `<staging>/spawn-latency.json` writer call site.
  3. Deletes the `InternalMessage` chunk delivery path for the
     latency report.
  4. Updates C's spec doc evolution log noting the migration is complete.
  Sub-G's migration plan owns the staging schema cleanup
  (`<staging>/spawn-latency.json` removed from the documented schema).

  TDD: write tests that drive a fixture `actor_loop` cycle and assert
  the expected metric updates appear in the registry snapshot. Layer 3
  integration test exists in sub-M Task 15; this task only needs unit
  tests on C's side.
- **Results:** _pending_
