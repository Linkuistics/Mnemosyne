# Memory — Sub-project C: Harness Adapter Layer

This plan implements sub-project C of the Mnemosyne orchestrator merge. The
design is already fully specified; this plan is the implementation work.

## Primary reference

**`{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-C-adapters-design.md`**
(committed at `71fd307`, amended at `b1a8cea`) is the authoritative design
document. Every task in this plan's backlog derives from that spec. If any
implementation question arises that the spec does not answer, the answer
goes into this memory file (and possibly back into the spec) rather than
being invented ad hoc.

## Parent plan

The orchestrator-level plan lives at
`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/` (currently — will be at
`{{PROJECT}}/mnemosyne/plans/mnemosyne-orchestrator/` after sub-project G's
migration). It coordinates this sub-plan with its siblings (A, B, D, E, F,
G, H, I, K, L, and the newly-surfaced M observability sub-project). The
parent plan's `memory.md` holds cross-sub-project architectural state.
This file holds only sub-project-C-specific implementation state.

## Sub-C unblocks sub-B's v1 dogfood acceptance test

C's primary critical-path role: B's `LlmHarnessExecutor` currently holds
a stub adapter; the C-1 dogfood acceptance test (the final task in this
plan's backlog) is the moment B's stub gets swapped for the real
`ClaudeCodeAdapter`. Until that swap lands, B can run end-to-end against
`FixtureReplayAdapter` only. The sequence is therefore:

1. C ships the trait declaration (early task in this backlog).
2. C ships `FixtureReplayAdapter` (mid backlog) — B can already use this.
3. C ships `ClaudeCodeAdapter` (mid-late backlog).
4. C runs the dogfood acceptance test (final backlog task) — this is the
   v1 acceptance for both C and (jointly) B.

## Key architectural anchors (quick reference; spec is canonical)

These are the decisions most load-bearing for implementation. Consult the
design doc for full context before acting on any of them.

### Bidirectional stream-json process model (Q1 — settled)

V1 spawns `claude --print --verbose --input-format stream-json --output-format stream-json`
as a normal child process via `std::process::Command`. **No PTY**, no
terminal escape parsing, no `portable-pty` dep. User input flows from
Mnemosyne's TUI through B's executor's input-forwarder thread into the
session's stdin as stream-json user-message envelopes. Output flows from
the child's stdout through a `BufReader::lines()` JSON parser into typed
`OutputChunk`s. See §4.1 of the spec.

### Module layout: `src/harness/` under the existing single binary crate (Q2 — settled)

Not a workspace crate. Module structure under `src/harness/` matches the
existing flat-module convention (`src/commands/`, `src/knowledge/`, etc.).
See §2.2 of the spec for the full layout.

### Actor-style `ClaudeCodeSession` with single-owner-per-state discipline

Three threads per live session: actor thread (owns child + stdin + pgid),
stdout-reader thread (parses stream-json), stderr-reader thread (collects
stderr lines). All mutable session state lives inside the actor thread.
The session handle exposed to B's executor holds only `crossbeam_channel`
endpoints + an atomic flag + one `Mutex<Option<JoinHandle>>`. The trait
methods are `&self` (not `&mut self`) so the session can be
`Arc`-shared between B's output-drainer and input-forwarder threads.
See §4.2 of the spec.

### Process-group termination from v1 (not deferred)

`std::os::unix::process::CommandExt::process_group(0)` at spawn time
makes the child its own pgrp leader. `nix::sys::signal::killpg` at
terminate time targets the whole group. Two-phase escalation: SIGTERM
first, then SIGKILL after a 500ms grace if the child has not exited.
The `child_alive_flag` `Arc<AtomicBool>` is reset by the stdout-reader
thread on EOF, so cleanly-exiting processes skip the SIGKILL. See §4.4
of the spec. Not a v1.5 deferral — explicit user direction during the
brainstorm.

### crossbeam-channel for the actor inbox and session channels

`crossbeam_channel::Sender<T>` and `Receiver<T>` are `Sync`, eliminating
`Mutex` wrappers from the session struct. `select!` is available for
future actor extensions. See §2.3 of the spec for the dep justification.

### `SessionLifecycle` chunks for protocol-level harness state observation

A new `OutputChunkKind::SessionLifecycle` variant (fourth amendment to
B's draft trait) surfaces protocol-level harness state transitions —
`"ready"` (system/init), `"turn_complete:<subtype>"` (result event),
`"exited:<status>"` (stdout EOF) — as structured chunks in the existing
output stream. **Distinct from task-level "the LLM has finished the
work" signals**, which are owned by B's executor via prompt-driven
sentinel detection. See §4.3.2, §4.3.3, and §11.1 of the spec.

### "No callback channel" means no *control*, not no *observation*

The architectural rule from `mnemosyne-orchestrator/memory.md` ("no
slash commands inside the harness") and B's contract #8 ("no callback
channel from harness to Mnemosyne") forbid the harness *calling into*
Mnemosyne. They do NOT forbid Mnemosyne *observing* harness state and
reacting on its own side. Mnemosyne reads the structured output stream
and parses it into rich typed events; the harness has no idea Mnemosyne
exists. Both `SessionLifecycle` chunks and sentinel detection are
observation channels, not control channels. See §3.3 of the spec.

### Tool profile enforcement: spawn-time flags + stream-side defence-in-depth

`IngestionMinimal` → `--allowed-tools "" --permission-mode default`.
`ResearchBroad` → no `--allowed-tools` flag, `--permission-mode acceptEdits`.
The actor's event handler also runs a defence-in-depth check on every
`tool_use` block — if an `IngestionMinimal` session somehow sees a
`ToolUse` chunk, it terminates with `UnsupportedToolProfile`. See §6 of
the spec.

### Cold-spawn only in v1; warm-pool work gated by C-1 acceptance criterion

V1 of `ClaudeCodeAdapter` ships with no warm-pool — every spawn is a
fresh `Command::spawn`. The C-1 acceptance gate (§7.3 of the spec) is
**p95 < 5 seconds across N≥10 dogfood cycles**. If the gate trips,
the §7.4 warm-pool reset spike unblocks and warm-pool implementation
moves into v1; if it doesn't, warm-pool is deferred to v1.5 with the
§7.5 sketch as the implementation starting point.

### Always-on latency instrumentation

`SpawnLatencyReport` is emitted as an `InternalMessage` chunk after
the first real chunk and also written to
`<staging>/spawn-latency.json`. Always-on, no debug flag. The dogfood
acceptance test parses these files to compute the C-1 gate decision.
This instrumentation is a **tactical seed**, not the start of a metrics
framework — proposed Sub-M (Observability) owns the broader story.

## Implementation strategy

### Phase ordering

1. **Setup** (Cargo.toml deps, module skeleton, `trait_def.rs` types only).
2. **Day-1 verification** of Claude Code's CLI surface (the §10 IOUs).
   Capture canonical stream-json samples to fixtures; lock parser field
   names and flag spellings against the pinned `claude` version.
3. **`FixtureReplayAdapter` first** (no claude dep needed; lets B start
   running its executor against a working adapter immediately).
4. **`ClaudeCodeAdapter` wire layer** (spawn, stream-json parser,
   user-message input).
5. **Actor architecture** (the actor loop, threading, defence-in-depth,
   `SessionLifecycle` emission).
6. **Lifecycle and termination** (process-group teardown, `CrashedBeforeReady`
   detection).
7. **Latency instrumentation** + dev recording subcommand.
8. **Test layers 1-3** (parser units → fixture integration → gated live).
9. **Dogfood acceptance gate** (the swap-in moment for B's stub).
10. **Conditional warm-pool work** if and only if the gate trips.

### TDD

Every type and rule in §3.1, §4.2-§4.5, §5, §6 of the spec has a
dedicated unit or integration test written first. The three test layers
(§8 of the spec) are the cheapest insurance against silent regressions.
Threading bugs in particular only surface under the actor-architecture
parity that `FixtureReplayAdapter` and `ClaudeCodeAdapter` share — that
parity is *the* reason `FixtureReplayAdapter` mirrors the live actor
architecture.

### No premature optimisation

Cold-spawn pooling is gated by the C-1 acceptance test. Do not pre-emptively
add pool machinery. The latency instrumentation is the measurement
mechanism that makes the gate concrete.

## Verified Claude Code CLI surface

This section accumulates verified facts about the `claude` binary as
day-1 verification tasks resolve. Each entry records the source command,
the pinned `claude` version, and the verification timestamp.

*(Empty until Task 3 lands.)*

## Dependencies on sibling sub-projects

- **Sub-project B (phase cycle)** — *consumes* C's trait and adapter
  implementations. Five amendments to B's draft surface are recorded as
  cross-sub-project requirements in §11.1 of the spec:
  - **Trait amendment 1**: `HarnessSession::send_user_message(&self, text)` — new method.
  - **Trait amendment 2**: `HarnessSession` methods change from `&mut self` to `&self` with `Send + Sync` bound.
  - **Trait amendment 3**: `LlmHarnessExecutor` storage changes from `Box<dyn HarnessSession>` to `Arc<dyn HarnessSession>`; gains `user_input_sender()` method; spawns two threads (output-drainer + input-forwarder).
  - **Trait amendment 4**: `OutputChunkKind` gains a `SessionLifecycle` variant.
  - **Executor requirement (5)**: `LlmHarnessExecutor` runs `Stdout` chunks through a configurable completion-sentinel matcher (sliding-buffer based) for task-level completion detection; sentinels live in phase prompt files.

  These amendments are absorbed by B's implementation phase, not by a
  B re-brainstorm. C ships the trait declaration; B's executor consumes
  the trait via `use crate::harness::*`.

- **Sub-project E (ingestion)** — uses `ToolProfile::IngestionMinimal`
  for Stage 3/4 reasoning sessions. Already specified in E's design.
  No new requirements on C from E.

- **Sub-project D (concurrency)** — no new requirements. C does not
  touch the locking model. D's per-plan and store-level locks are
  independent of C's adapter layer.

- **Sub-project G (migration)** — note that the existing
  `<project>/adapters/claude-code/` directory in the Mnemosyne repo is
  the **legacy v0.1.0 Claude Code plugin** (markdown skills consumed by
  Claude Code itself), not a Rust adapter. C does not touch it; G's
  migration plan owns retiring or renaming it.

- **Sub-project M (observability)** — proposed new sub-project surfaced
  during this brainstorm. C's `SpawnLatencyReport` is a tactical seed,
  not the start of a metrics framework. When M lands, the latency
  report should migrate onto whatever structured event bus M designs.
  C deliberately does **not** commit to a structured logging crate
  (`tracing`, `slog`, `log`, etc.) so M has full freedom.

## Open questions (implementation-level)

These are the §10 verification IOUs from the spec. Each is a day-1
implementation task with an explicit resolution method.

1. **Exact spelling of empty-tool-list value for `--allowed-tools`.**
   Candidates: `""`, `none`, omitting the flag and using
   `--disallowed-tools "*"`. Resolution: behavioural test against the
   pinned `claude` version. Resolves Task 3.
2. **Right `--permission-mode` for fully-headless operation.** Candidates:
   `acceptEdits`, `bypassPermissions`, `dangerouslySkipPermissions`.
   Security implications differ. Resolution: behavioural test against
   each candidate touching a tmpdir file. Resolves Task 3.
3. **Whether `--print "<prompt>"` accepts a prompt arg alongside
   `--input-format stream-json`.** If not, the initial prompt is
   delivered as the first stdin envelope. Resolution: behavioural test
   with both shapes. Resolves Task 3.
4. **Exact stream-json field names** (`type`, `subtype`, `content`,
   `tool_use`, `is_error`). Resolution: capture real session JSON,
   commit as fixtures, lock parser shapes. Resolves Task 3.
5. **Warm-pool reset mechanism** (structured envelope vs `/clear` text
   vs degraded single-use). Resolution: §7.4 three-check spike protocol.
   **Conditional** — only fires if the C-1 gate trips.

## Risk watch list

Ranked by impact × likelihood from §9 of the spec. Each has a mitigation
path; flag here if implementation reveals the risk materialising.

1. **Claude Code stream-json schema drift across versions** *(MEDIUM × MEDIUM)*.
   Mitigated by version pinning + `serde(flatten)` rest-fields + Layer 3
   integration tests. Watch: the `claude --version` check at startup is
   the canary.
2. **Cold-spawn latency exceeds the 5s p95 gate** *(MEDIUM × MEDIUM)*.
   Mitigated by §7.4 spike + §7.5 sketch. Watch: the dogfood acceptance
   test is the explicit measurement gate.
3. **`nix` dep behaving differently on macOS vs Linux** *(LOW × LOW)*.
   Mitigated by Layer 3 process-group cleanup tests on both platforms.
   Watch: any `pgrep` after-state failure on either platform.
4. **`--input-format stream-json` does not accept the initial prompt as
   a CLI arg** *(LOW × LOW)*. Mitigated by day-1 verification picking
   the working path.
5. **v1 ships with diagnostic-poor failure modes** *(LOW × MEDIUM,
   accepted)*. Not mitigated in v1; the proposed Sub-M (Observability)
   is the future home for structured logging. Recorded as accepted v1
   limitation, not a bug.
