# Backlog — Sub-project B: Phase Cycle Reimplementation in Rust

Implementation backlog for sub-project B. All tasks derive from the design
doc at
`{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md`.
Consult the spec before starting any task.

Tasks are listed in approximately recommended order. **Task 0 (the Obsidian +
symlinks validation spike) is a hard pre-implementation blocker** — no other
task starts until it passes on both macOS and Linux. After task 0, the
ordering follows the dependency chain: core types → plan-state → staging →
executors → runner → drivers → runtime → dogfood. The work phase picks the
best next task with input from the user.

## Task Backlog

### Task 0 — Obsidian + symlinks validation spike (cross-platform) `[spike]`
- **Status:** not_started
- **Dependencies:** none (blocks everything else)
- **Description:** Pre-implementation blocker. Validate that Obsidian on
  macOS and Linux correctly renders, indexes, and navigates a vault whose
  `projects/` subdirectory contains symlinks into a sibling project's
  `mnemosyne/` directory. Use GUIVisionVMDriver's golden images
  (`guivision-golden-macos-tahoe` and `guivision-golden-linux-24.04`) per
  the workflow in §5.2 of the spec. Consult
  `{{DEV_ROOT}}/GUIVisionVMDriver/instructions-for-llms-using-this-as-a-tool.md`
  for VM commands. Validation checks (same list for both platforms):
  (1) file tree rendering, (2) file opening, (3) graph view follows
  symlinks, (4) wikilink resolution across symlinks, (5) Dataview queries
  resolve symlinked frontmatter, (6) file-watcher picks up external edits
  via the symlinked path, (7) backlinks across symlink boundaries.
  Capture evidence via `guivision agent snapshot --mode interact` and
  `guivision screenshot` for every check. Commit captured outputs to
  `tests/fixtures/obsidian-validation/`. Acceptance: all seven checks
  pass on both platforms. If any check fails on Linux (a v1 target),
  stop and open a brainstorm discussion for the hard-copy + two-way-sync
  fallback layout.
- **Results:** _pending_

### Define core abstractions and types `[types]`
- **Status:** not_started
- **Dependencies:** Task 0
- **Description:** Define the Rust types from §2.2 of the spec:
  `PlanContext`, `PlanState`, `LastExit`, `Phase`, `ResolvedPaths`,
  `PhaseRunner`, `PhaseOutcome`, `PhaseEvent`, `PhaseError`,
  `PhaseExecutor` (trait), `ExecutorKind`, `PhaseContext`,
  `ExecutorError`, `StagingDirectory`, `CopyBackReport`, `StagingError`,
  `InteractionDriver` (trait), `InteractionKind`, `DriverError`,
  `TuiAction`, `ExecutorMode`, `HarnessAdapter` (trait from §4.1),
  `HarnessSession` (trait), `HarnessKind`, `ToolProfile`, `OutputChunk`,
  `OutputChunkKind`, `SessionExitStatus`, `AdapterError`,
  `ReflectExitHook` (trait from §4.2), `ReflectExitContext`,
  `IngestionEventPayload`. No logic; just shapes. This establishes the
  contract every downstream task is written against. Unit tests cover
  serialisation round-trips for any type that crosses the
  `InteractionDriver` boundary.
- **Results:** _pending_

### Implement `plan-state.md` parse/serialise `[plan-state]`
- **Status:** not_started
- **Dependencies:** Define core abstractions and types
- **Description:** Implement `PlanState::load(plan_dir)` and
  `PlanState::persist(&self, plan_dir)` using `serde_yaml` and
  `gray_matter`. Reads/writes `plan-state.md` (markdown with YAML
  frontmatter per §2.2.2 of the spec). `persist` uses
  write-to-temp-then-rename for atomicity. Unit tests cover: round-trip
  of every field, `schema-version: 99` rejection (hard error),
  malformed frontmatter rejection, `ingestion-fired` flag semantics,
  atomic-write survival of concurrent readers, Dataview-friendly
  kebab-case naming of all fields. No filesystem mocks — use tempdir
  fixtures.
- **Results:** _pending_

### Implement embedded prompts module `[prompts]`
- **Status:** not_started
- **Dependencies:** Define core abstractions and types
- **Description:** Create `src/prompts.rs` with `include_str!` macros
  loading the four vendored prompt-reference files from
  `{{PROJECT}}/prompts/`. Expose `BACKLOG_PLAN_SPEC`,
  `CREATE_MULTI_SESSION_PLAN`, `CODING_STYLE`, `CODING_STYLE_RUST` as
  `&'static str` constants. Implement `materialise_into(prompts_dir:
  &Path) -> Result<()>` that writes the four files into a target
  directory. Copy the actual content from
  `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md`,
  `create-a-multi-session-plan.md`, `coding-style.md`, and
  `coding-style-rust.md` into `{{PROJECT}}/prompts/` as part of this
  task. Unit tests verify: materialise produces the expected files,
  binary-level byte-equal content with the vendored sources.
- **Results:** _pending_

### Implement placeholder substitution algorithm `[substitution]`
- **Status:** not_started
- **Dependencies:** Define core abstractions and types
- **Description:** Implement forward and reverse substitution as pure
  functions against the four placeholders (`{{DEV_ROOT}}`,
  `{{PROJECT}}`, `{{PLAN}}`, `{{PROMPTS}}`). Forward: walk a file's text
  and replace each placeholder with its resolved absolute path.
  Reverse: walk a file's text and replace each absolute path with its
  placeholder, using longest-match-first ordering (PLAN before PROJECT
  before DEV_ROOT) to avoid prefix collisions. Use `regex` with a
  static precompiled pattern. Unit tests cover: all four placeholders
  substituted correctly, longest-match-first ordering against prefix
  collision inputs, round-trip (forward then reverse) returns the
  original, edge cases (empty file, file with no placeholders, file
  with multiple occurrences of the same placeholder).
- **Results:** _pending_

### Implement `StagingDirectory::render` `[staging]`
- **Status:** not_started
- **Dependencies:** Implement embedded prompts module, Implement
  placeholder substitution algorithm
- **Description:** Implement the staging render per §2.2.5 of the spec.
  Walks the plan directory, copies every `.md` file to the staging
  root with forward substitution applied, materialises embedded prompt
  content into `<staging>/prompts/`, stages the phase prompt file with
  substitution applied, stages the narrow project allowlist resolved
  from the phase prompt's `{{PROJECT}}/...` tokens. Enforces the
  critical invariants: (a) never follow symlinks into project repos
  (hard error), (b) refuse to descend into subdirectories containing
  `plan-state.md` (hard error to even try — this enforces "one plan
  per Mnemosyne process" against nested hierarchy), (c) write only
  inside the staging root. Unit tests cover: correct forward
  substitution, embedded prompts materialisation, allowlist
  resolution, symlink rejection, child-plan-descent rejection, binary
  file passthrough (don't substitute non-text files).
- **Results:** _pending_

### Implement `StagingDirectory::copy_back` `[staging]`
- **Status:** not_started
- **Dependencies:** Implement `StagingDirectory::render`
- **Description:** Implement copy-back per §2.2.5 and §4.5 of the
  spec. Walks the staging root after executor exit, compares each file
  against the canonical plan directory via content hash, applies
  reverse substitution on `.md` files only, writes through changed
  files atomically. Produces a `CopyBackReport` with files_updated,
  files_unchanged, files_added, files_rejected. Rejection happens when
  a staged file's relative path is outside the plan directory or the
  explicitly-staged project subtree — this is a hard error that aborts
  the phase outcome and preserves the staging dir for forensics. Unit
  tests cover: clean write-through, unchanged-file detection, new-file
  propagation, reverse substitution correctness, rejection rule
  enforcement, atomic rename semantics.
- **Results:** _pending_

### Implement `StagingDirectory::preserve_on_interrupt` and `cleanup` `[staging]`
- **Status:** not_started
- **Dependencies:** Implement `StagingDirectory::render`
- **Description:** `preserve_on_interrupt` moves the staging directory
  to `<vault>/runtime/interrupted/<plan-id>/<phase>-<timestamp>/` for
  forensics. `cleanup` deletes the staging directory on clean phase
  exit. Both operations are atomic (rename-based move for preserve,
  recursive delete for cleanup). Unit tests cover both paths and the
  idempotency of cleanup.
- **Results:** _pending_

### Implement `FixtureReplayExecutor` and `FixtureReplayAdapter` `[executor]`
- **Status:** not_started
- **Dependencies:** Define core abstractions and types
- **Description:** Implement the simplest executor first: reads a
  captured output stream from a JSON fixture file, pushes chunks to
  `output_tx` with simulated pacing, returns. No process spawn, no
  `$EDITOR`, no harness adapter. Also implement `FixtureReplayAdapter`
  — a `HarnessAdapter` implementation that reads a captured Claude Code
  session transcript from JSON and emits `OutputChunk`s through the
  `HarnessSession` trait. Both are used by downstream integration
  tests to drive `PhaseRunner` end-to-end without a live harness.
  Unit tests verify: fixture parsing, chunk pacing, clean exit
  semantics, the `interrupt()` no-op semantics for replay.
- **Results:** _pending_

### Implement `PhaseRunner::run_phase` `[runner]`
- **Status:** not_started
- **Dependencies:** Implement `plan-state.md` parse/serialise, Implement
  `StagingDirectory::render`, Implement `StagingDirectory::copy_back`,
  Implement `FixtureReplayExecutor` and `FixtureReplayAdapter`
- **Description:** Wire the 13-step `run_phase` flow from §2.2.3 of the
  spec: validate transition, check lock, render staging, update state,
  emit `PhaseStarted`, invoke executor, copy_back on clean exit,
  compute next phase, update `last_exit`, persist, emit `PhaseExited`,
  fire `ReflectExitHook` if phase was reflect, cleanup staging.
  Integration tests use `FixtureReplayExecutor` and a fixture plan
  directory. Test coverage: full happy path (work → reflect → triage
  → work round-trip), illegal transition rejection, copy-back rejection
  aborts cleanly, hook fires exactly once on reflect exit, hook does
  NOT fire on work or triage exit, state persistence at every
  transition boundary, event emission order.
- **Results:** _pending_

### Implement `PhaseRunner::interrupt` and the Interrupted flow `[runner]`
- **Status:** not_started
- **Dependencies:** Implement `PhaseRunner::run_phase`, Implement
  `StagingDirectory::preserve_on_interrupt` and `cleanup`
- **Description:** Implement the interrupt flow from §3.6 of the spec:
  signal executor interrupt, capture partial output, preserve staging
  dir under `<vault>/runtime/interrupted/`, update `plan-state.md`
  with `interrupted: true`, emit `PhaseInterrupted`, do not transition
  `current-phase`. Also implement the pending-interrupt flag: an
  interrupt during CopyingBack is recorded in a pending flag,
  copy-back completes, then the runner honours the flag. Unit tests
  cover: interrupt during ExecutorRunning (immediate), interrupt during
  CopyingBack (deferred until copy-back completes), idempotency of
  double-interrupt, correct forensic log path, `interrupted` flag
  persistence.
- **Results:** _pending_

### Implement `ManualEditorExecutor` `[executor]`
- **Status:** not_started
- **Dependencies:** Implement `PhaseRunner::run_phase`
- **Description:** Implement the human-driven executor per §2.2.4 of
  the spec. Writes `<staging>/phase-prompt.readonly.md` with the
  substituted phase prompt text. Determines target file(s) for the
  phase (`work` → backlog.md + session-log.md; `reflect` → memory.md +
  session-log.md; `triage` → backlog.md). Spawns `$EDITOR` (resolved
  from the `EDITOR` environment variable; fall back to hard error if
  unset) blocked on the target files. Waits for editor exit. Returns
  `Ok(())` on clean exit, `ExecutorError::EditorFailed` on non-zero.
  `interrupt()` is a no-op. Integration tests use a scripted editor
  stub (e.g., a shell script that edits a file then exits) to
  exercise the full flow through `PhaseRunner`.
- **Results:** _pending_

### Implement `LlmHarnessExecutor` with Claude Code stub adapter `[executor]`
- **Status:** not_started
- **Dependencies:** Implement `PhaseRunner::run_phase`, Define core
  abstractions and types
- **Description:** Implement the LLM executor per §2.2.4 of the spec.
  Holds a `Box<dyn HarnessAdapter>`, reads the phase prompt from
  `<staging>/phase-prompt.md`, invokes `adapter.spawn()` with the
  appropriate tool profile, streams `OutputChunk`s to `output_tx`,
  waits for child exit. `interrupt()` calls `adapter.terminate()`.
  Also implement a minimal `ClaudeCodeAdapter` stub that spawns
  `claude` as a subprocess, passes the prompt via `-n` and stdin,
  captures stdout as `OutputChunk` events. This is a temporary stub
  until sub-project C's real adapter lands. Integration tests use
  `FixtureReplayAdapter` (from the earlier task), not the real stub,
  so tests stay deterministic. Document the ClaudeCodeAdapter stub's
  limitations clearly — it does not enforce tool profiles, does not
  support warm-pool reuse, and lacks the full error handling the
  real sub-project C adapter will provide.
- **Results:** _pending_

### Implement `ReflectExitHook` wiring for sub-project E `[integration]`
- **Status:** not_started
- **Dependencies:** Implement `PhaseRunner::run_phase`
- **Description:** Expose the `ReflectExitHook` trait from §4.2 of the
  spec and wire its invocation into step 11 of `run_phase`. The hook
  is called non-blockingly on a dedicated tokio task; errors from the
  hook are not observed by B. Until sub-project E's real
  implementation lands, ship a `NullReflectExitHook` stub that logs
  the call and returns. When E lands, swap in E's concrete
  implementation. Unit tests verify: hook fires exactly once per
  clean reflect exit, hook receives the correct `ReflectExitContext`,
  hook does not fire when `last-exit.ingestion-fired` was already
  true, hook is non-blocking (the runner returns before the hook
  completes).
- **Results:** _pending_

### Implement event channel and `PhaseEvent` forwarding `[events]`
- **Status:** not_started
- **Dependencies:** Define core abstractions and types
- **Description:** Implement the shared event channel (tokio mpsc) that
  carries `PhaseEvent` variants from `PhaseRunner` to the
  `InteractionDriver`. Also wire forwarding of sub-E's
  `IngestionEventPayload` into the same channel as the
  `PhaseEvent::IngestionEvent` variant. Unit tests cover: event
  ordering preserved, backpressure semantics (what happens when the
  driver is slow), clean channel shutdown on `Quit`.
- **Results:** _pending_

### Implement `HeadlessDriver` `[driver]`
- **Status:** not_started
- **Dependencies:** Implement `PhaseRunner::run_phase`, Implement event
  channel and `PhaseEvent` forwarding
- **Description:** Implement the test-driven interaction driver per
  §2.2.6 of the spec. Reads `TuiAction`s from a fixture script (or
  stdin, for interactive test debugging) and writes `PhaseEvent`s to
  stdout as JSON lines. Owns the `PhaseRunner` for the session and
  invokes `runner.run_phase` in response to `AdvancePhase` actions.
  This is the primary end-to-end test driver — every integration
  test uses it. Unit tests cover: fixture script parsing, action
  dispatch, event serialisation, clean shutdown on `Quit`,
  error propagation.
- **Results:** _pending_

### Implement `IpcDriver` — compile-time boundary hardening `[driver]`
- **Status:** not_started
- **Dependencies:** Implement `HeadlessDriver`
- **Description:** Implement the JSON IPC driver per §2.2.6 and §4.3
  of the spec. Reads user actions from stdin as JSON lines, writes
  events to stdout as JSON lines, every message carries
  `protocol-version: 1`. This is v1's compile-time enforcement
  mechanism for the serialisable `InteractionDriver` boundary: if
  any new method added to `InteractionDriver` can't be implemented
  by `IpcDriver`, it fails to compile. Ship concrete JSON message
  shapes as they are introduced and commit example sequences to
  `tests/fixtures/ipc-protocol/`. Unit tests cover: protocol-version
  tagging, unknown-version rejection, message shape round-trips.
  `IpcDriver` has no client in v1 — its purpose is boundary
  discipline, not user-facing UX.
- **Results:** _pending_

### Implement `RatatuiDriver` — the v1 primary UI `[driver]`
- **Status:** not_started
- **Dependencies:** Implement `HeadlessDriver`, Implement event
  channel and `PhaseEvent` forwarding
- **Description:** Implement the ratatui TUI per §2.2.6 and §3.4 of
  the spec. Three panes: backlog summary (top-left, from backlog.md),
  executor output (right, streaming harness/editor output),
  notification feed (bottom-left, phase and ingestion events). Status
  bar at the bottom showing plan id, phase, mode, pid. Main event
  loop: render current state, await either a TUI input event or an
  inbound `PhaseEvent` (via tokio `select!`), dispatch to
  `PhaseRunner::run_phase` on `AdvancePhase`, to `runner.interrupt()`
  on Ctrl-C, etc. Keybindings per §3.4 (arrow keys primary, Vim
  alternates, configurable later). Takeover prompt rendered as a
  non-modal notification with `[y / n / retry-llm]` option display.
  Unit tests use ratatui's snapshot testing for pane layout; full
  interaction flows are tested via `HeadlessDriver`.
- **Results:** _pending_

### Implement symlink rescan and vault bootstrap `[vault]`
- **Status:** not_started
- **Dependencies:** Task 0 (symlinks validation spike must pass first)
- **Description:** Implement the symlink rescan lifecycle per §2.4 of
  the spec. Walks the dev root looking for project repos (directories
  containing `.git`), checks for `<project>/mnemosyne/`, creates
  relative symlinks under `<vault>/projects/<project-name>`,
  garbage-collects dangling links, detects cycles via `same-file`,
  handles the `mnemosyne rescan` CLI subcommand as an explicit trigger
  plus automatic invocation on startup. Also implement vault bootstrap
  per §2.3: create the vault directory if missing, create
  `knowledge/`, `runtime/`, `projects/` subdirectories, initialise
  `.git` and `.gitignore`, write a minimal `.obsidian/` stub and a
  vault `README.md`. Unit tests use tempdir-based fake dev roots with
  fixture project layouts.
- **Results:** _pending_

### Implement crash recovery `[lifecycle]`
- **Status:** not_started
- **Dependencies:** Implement `plan-state.md` parse/serialise, Implement
  `ReflectExitHook` wiring for sub-project E
- **Description:** Implement the three crash recovery scenarios from
  §3.3 of the spec. Scenario A (interrupted): surface the prior
  interrupt to the TUI notification feed on first render, offer
  "retry this phase" as a TUI action. Scenario B
  (ingestion-didn't-fire): fire the ingestion pipeline on the prior
  reflect's outputs as the first post-Idle action, with the hook
  flipping the flag at Stage 5 start. Scenario C (clean state): no-op.
  All three run at the Starting → Idle transition. Idempotency: running
  recovery twice on the same state produces the same result. Unit
  tests cover all three scenarios, crash-during-recovery survival, and
  the "last reflect was already fired" no-op path.
- **Results:** _pending_

### Implement the full startup sequence `[lifecycle]`
- **Status:** not_started
- **Dependencies:** Implement symlink rescan and vault bootstrap,
  Implement crash recovery, Implement `PhaseRunner::run_phase`,
  Implement `RatatuiDriver`
- **Description:** Wire the 10-step startup sequence from §3.2 of the
  spec: parse CLI, locate plan, locate host project, locate dev root,
  locate or bootstrap vault, run symlink rescan, acquire per-plan
  advisory lock, load `plan-state.md`, run crash recovery, construct
  `PhaseRunner` + event channel + `InteractionDriver`, transition to
  Idle. Every step is a hard-error boundary. Also implement the CLI
  surface: `mnemosyne`, `mnemosyne <plan-dir>`, `mnemosyne --replay
  <fixture-dir>`, `mnemosyne --headless --script <path>`, `mnemosyne
  --ipc`, `mnemosyne rescan`, `--version`, `--help`. Integration tests
  exercise the full startup against tempdir vault fixtures.
- **Results:** _pending_

### Implement per-plan advisory lock stub `[concurrency]`
- **Status:** not_started
- **Dependencies:** Implement symlink rescan and vault bootstrap
- **Description:** Implement a stub for sub-project D's advisory lock
  primitive: `PlanLockGuard::acquire(path)` using `fs2::FileExt`'s
  `lock_exclusive` (or equivalent) against
  `<vault>/runtime/locks/<plan-id>.lock`. Held for the session
  lifetime, released on drop. Acquisition failure is a hard error
  with a diagnostic naming the holding PID (read from the lock file's
  contents, which Mnemosyne writes on acquire). Sub-project D's real
  implementation replaces this stub when it lands; the replacement
  should be a drop-in swap. Unit tests cover: acquire-release cycle,
  contention detection, PID diagnostic.
- **Results:** _pending_

### Implement shutdown sequence `[lifecycle]`
- **Status:** not_started
- **Dependencies:** Implement the full startup sequence, Implement
  `PhaseRunner::interrupt` and the Interrupted flow
- **Description:** Wire the 7-step shutdown sequence from §3.7 of the
  spec: stop accepting input, force-interrupt executor if running,
  persist final `plan-state.md`, release lock, cleanup or preserve
  staging, restore terminal, exit with appropriate status code. Both
  clean quit and hard-error shutdown flow through this sequence.
  Integration tests cover: clean quit from Idle, hard-error from
  PhaseRunning, forced interrupt during CopyingBack.
- **Results:** _pending_

### End-to-end integration test corpus `[testing]`
- **Status:** not_started
- **Dependencies:** Implement `HeadlessDriver`, Implement the full
  startup sequence, Implement `PhaseRunner::run_phase`
- **Description:** Build a corpus of fixture plans under
  `tests/fixtures/plans/` and corresponding scripted action sequences
  under `tests/fixtures/scripts/`. Each integration test runs a
  fixture plan through the full pipeline with `HeadlessDriver` +
  `FixtureReplayExecutor` and asserts the final plan state plus the
  emitted event stream match expectations in
  `tests/fixtures/expected/`. Coverage target: full work → reflect
  → triage → work cycle, interrupt-and-retry, interrupt-and-takeover,
  crash-recovery-scenario-A, crash-recovery-scenario-B, copy-back
  rejection abort, nested plan (parent + child) with descent rule
  enforcement.
- **Results:** _pending_

### Live gated test against real Claude Code `[testing]`
- **Status:** not_started
- **Dependencies:** Implement `LlmHarnessExecutor` with Claude Code
  stub adapter, End-to-end integration test corpus
- **Description:** Create a manual/CI-gated test that runs the full
  cycle against a real Claude Code instance with a fixture plan,
  asserting only structural invariants: phases complete, no hard
  errors, lock acquired/released, staging cleaned up, `plan-state.md`
  consistent, ingestion event log written (if E's real implementation
  is also landed by then). Not run on every commit. Runs on a
  clean-room dev-root layout torn down after the test. Produces
  characterisation data for the cold-spawn latency risk from §5.4.
- **Results:** _pending_

### Dogfood task: host the orchestrator seed plan on Mnemosyne v1 `[dogfood]`
- **Status:** not_started
- **Dependencies:** End-to-end integration test corpus, Implement
  shutdown sequence, Implement `RatatuiDriver`, Live gated test
  against real Claude Code
- **Description:** The v1 acceptance test. Migrate the orchestrator
  seed plan and sub-E's sibling plan from `phase.md` +
  `run-backlog-plan.sh` to `plan-state.md` + Mnemosyne v1. This
  depends on sub-project G's migration logic being available — if G
  is not yet done, perform the migration manually as a one-shot for
  these two plans as part of this task's scope (document the manual
  steps clearly). Run a full work → reflect → triage cycle of the
  orchestrator plan on Mnemosyne v1 end-to-end: Ratatui TUI, real
  Claude Code harness via the stub adapter, real ingestion pipeline
  from sub-E, real vault at `<dev-root>/Mnemosyne-vault/`. Success
  criteria: cycle completes cleanly, all events emitted correctly,
  plan state persisted consistently, ingestion fired exactly once,
  no hard errors, the user can retire `run-backlog-plan.sh` for
  these two plans and proceed on Mnemosyne for all future sessions.
  This is the moment v1 ships.
- **Results:** _pending_

### Implementation notes and handoff documentation `[docs]`
- **Status:** not_started
- **Dependencies:** Dogfood task: host the orchestrator seed plan on
  Mnemosyne v1
- **Description:** Document any implementation decisions, discovered
  constraints, or design-doc deviations in `{{PLAN}}/memory.md`.
  Produce a short user-facing note for the main Mnemosyne docs
  explaining how the phase cycle works, when each phase fires, how
  to use the takeover prompt, and how to inspect plan state through
  Obsidian (Dataview query examples for common views). Update the
  main Mnemosyne `README.md` with a brief "v0.2.0 orchestrator"
  section.
- **Results:** _pending_
