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

## BEAM / Elixir pivot (Session 9 amendment)

The orchestrator's Session 9 committed Mnemosyne to a persistent BEAM
daemon (Elixir/OTP). Sub-C's entire design was brainstormed assuming Rust
with actor threading. The orchestrator now tracks a "Sub-C amendment —
Elixir implementation and multi-adapter reservation" task with the
following consequences:

- **Implementation language**: Elixir, not Rust.
- **Process spawning**: `erlexec` replaces `std::process::Command`.
- **Supervision**: OTP GenServer supervision replaces the hand-rolled
  actor-style threading (crossbeam-channel inbox, three threads per
  session, etc.).
- **Trait surface**: `HarnessSpawner` becomes an Elixir behaviour, not a
  Rust trait.
- **Tool-call boundary**: in-session Queries use a tool-call boundary
  (new architectural element).
- **Multi-adapter support**: reserved for sub-O; sub-C ships
  single-adapter only.

**BEAM PTY spike resolved (Session 10, 2026-04-15).** The spike validated
that pipes-only `erlexec` works perfectly for driving Claude Code's
stream-json protocol. Key findings:

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

**Impact on existing backlog**: every task in `backlog.md` is written
against the Rust implementation assumption (Cargo.toml, `src/harness/`,
crossbeam, nix, etc.). The backlog must be rewritten for Elixir/OTP
before implementation begins. The task *sequence* (types → fixture replay
→ live adapter → tests → dogfood gate) and the *design intent* (tool
profiles, process-group termination, cold-spawn latency gate, fixture
replay parity) remain valid; only the implementation technology changes.

**Design doc rewritten for the pivot (Session 11, 2026-04-15).** Rather
than layering a supersede-amendment on top of the Rust brainstorm, the
spec was **rewritten inline across §1–§11** with fresh Elixir/OTP
content. The original Session 6 decisions survive only in Appendix A
(as Q1–Q5, with spike corrections), and new Appendix A entries Q6–Q8
record the BEAM pivot, the BEAM PTY spike findings, and the tool-call
boundary contract. The rewrite covers: Elixir-native scope (§1), the
GenServer + DynamicSupervisor architecture with module layout and
supervision tree placement (§2), the `@behaviour Mnemosyne.HarnessAdapter`
surface and the `Mnemosyne.Event.*` sealed typed-struct set (§3), the
full ClaudeCode adapter with the normative `:exec.run/2` spawn path,
cmux mitigation flags, session GenServer state and message set,
stream-json parser locked against spike samples, two-phase SIGTERM→SIGKILL
termination, **tool-call boundary for in-session Queries** (new §4.5:
injected tool set `ask_expert`/`dispatch_to_plan`/`read_vault_catalog`,
three candidate injection mechanisms, intercept flow via
`Mnemosyne.Router.handle_tool_call/4`, why-not-control-channel), and
the `CrashedBeforeReady` heuristic and error-reason table (§4.6). §5
re-casts FixtureReplay as a GenServer walking JSON-Lines records via
`Process.send_after/3`. §6-§8 preserve tool-profile enforcement,
cold-spawn / C-1 gate, and a three-layer ExUnit testing strategy. §9
Risks adds exec-port loss and tool-call-boundary injection brittleness;
§10 marks Q3/Q4 **resolved by the spike**, keeps Q1/Q2/Q5, and adds Q6
(tool-call boundary mechanism — day-1 spike) and Q7 (exec-port
supervision — resolved as design decision). §11 drops Rust-specific
amendments 1-3 back to B (no BEAM analogue) and keeps the consumed
`%SessionLifecycle{}` event plus the sentinel-matcher executor
requirement. Appendix B replaces the Cargo.toml diff with an `mix.exs`
projection — one new Hex dep (`erlexec`). **The rewritten spec is the
starting point for the rewritten task list in `backlog.md`.**

**Pattern note**: the user explicitly prefers pivot-induced inline
rewrites over supersede-layer amendments. Recorded as durable feedback
memory. Applies to all remaining amendment tasks in the orchestrator
backlog (sub-A, sub-B, sub-D, sub-E, sub-M, sub-G, sub-H, sub-I).

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
   `--input-format stream-json`.** **RESOLVED by the BEAM PTY spike
   (Session 10).** With `--input-format stream-json`, the initial prompt
   must be delivered as the first stdin envelope via `:exec.send/2`. The
   CLI-arg form errors with `Input must be provided either through stdin
   or as a prompt argument when using --print`. Lock: every prompt —
   including the initial one — ships as an NDJSON user envelope on stdin.
4. **Exact stream-json field names** (`type`, `subtype`, `content`,
   `tool_use`, `is_error`). **RESOLVED by the BEAM PTY spike (Session 10).**
   Canonical samples captured in `spikes/beam_pty/results/full-run.log`
   covering `system/init`, `rate_limit_event`, `assistant/thinking`,
   `assistant/text`, and `result/success`. Copy into
   `test/fixtures/harness/captured-stream-json/` during implementation
   Task 3 and lock the Elixir parser module against them.
5. **Warm-pool reset mechanism** (structured envelope vs `/clear` text
   vs degraded single-use). Resolution: §7.4 three-check spike protocol.
   **Conditional** — only fires if the C-1 gate trips.
6. **Tool-call boundary mechanism for in-session Queries** (new, from
   design doc §12.5). How are Mnemosyne-injected tools (`ask_expert`,
   `dispatch_to_plan`, `read_vault_catalog`) exposed to a running claude
   session? Candidates: (a) MCP server on Unix socket with
   `--mcp-config`; (b) stdin-side tool-definition preamble; (c)
   plugin-shipped tool shims. Resolution: focused spike at the start of
   C's implementation phase before GenServer wiring is finalised.
7. **`exec-port` supervision** (new). `erlexec` runs `exec-port` as a
   separate OS process. What happens if `exec-port` crashes while live
   sessions exist? Resolution: document expected behaviour — sessions
   are not persistent by design, so a restart drops them and plans
   restart their current phase on daemon restart. Capture the call
   chain in C's implementation phase memory.

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
