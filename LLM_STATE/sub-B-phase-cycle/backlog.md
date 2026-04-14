# Backlog — Sub-project B: Phase Cycle Reimplementation

Implementation backlog for sub-project B. All tasks derive from the design
doc at
`{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md`.
Consult the spec before starting any task.

Tasks are listed in approximately recommended order. **Task 0 (the Obsidian +
symlinks validation spike) PASSED on both macOS and Linux** in Session 5
(2026-04-13) of the parent orchestrator plan, executed end-to-end via the
`guivision` CLI; evidence at `tests/fixtures/obsidian-validation/results/{macos,linux}/`
(commit `98ef7db`). With Task 0 cleared, downstream tasks are unblocked. The
ordering follows the dependency chain: core types → plan-state → staging →
executors → runner → drivers → runtime → dogfood. The work phase picks the
best next task with input from the user.

## Task Backlog

### Task 0 — Obsidian + symlinks validation spike (cross-platform) `[spike]`
- **Status:** done
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
- **Results:** **PASS on both platforms.** Executed at the orchestrator-plan
  level in Session 5 (2026-04-13) of `mnemosyne-orchestrator`, driven
  end-to-end via the `guivision` CLI against
  `guivision-golden-macos-tahoe` and `guivision-golden-linux-24.04` with
  Obsidian 1.12.7 + Dataview 0.5.67 pinned identically. The orchestrator
  spike's six-check enumeration (Dataview / graph view / backlinks / file
  tree+open / file watcher / safety checks) covers this task's seven
  checks (file tree+file opening collapsed into a single explorer check,
  wikilinks subsumed by Dataview cross-boundary queries). All checks
  passed on both platforms. Evidence:
  `{{PROJECT}}/tests/fixtures/obsidian-validation/results/{macos,linux}/`
  with per-platform `result.md` summary tables and per-check screenshots
  + OCR transcripts. Commit `98ef7db` ("test: add Obsidian symlink
  validation spike fixture and evidence"). Architectural consequence
  (vault-as-view-over-symlinks stands; hard-copy fallback NOT activated)
  recorded in `{{DEV_ROOT}}/Mnemosyne/LLM_STATE/mnemosyne-orchestrator/memory.md`
  under the "Dedicated Mnemosyne-vault" decision. **Task 0 is cleared;
  all downstream B implementation tasks are now unblocked.** Two
  platform-specific operational notes surfaced and should be remembered
  by any future B GUI work on the same images: (1) Electron-in-virtio-gpu-
  under-tart on ARM64 Ubuntu requires `--disable-gpu --no-sandbox` for
  visible rendering; (2) macOS Notification Center widgets in the Tahoe
  golden image occlude part of the Obsidian window area and should be
  dismissed at VM setup time.

### Absorb Sub-C trait amendments into B's design + types `[types]` `[amendment]`
- **Status:** not_started
- **Dependencies:** Task 0
- **Description:** Sub-project C's brainstorm (Session 6 of the parent
  orchestrator plan, 2026-04-13) produced **four additive amendments to
  B's `HarnessSession` trait and `OutputChunkKind` enum, plus one
  executor-level requirement**, all forced by C's locked Q1 decision
  (bidirectional `stream-json`) and two post-write user clarifications
  (disambiguating "no callback channel" as control-only and separating
  task-level from protocol-level completion). All five changes post-date
  B's brainstorm and must be absorbed *before* the "Define core abstractions
  and types" task is started so that types are defined in their amended
  shape from the outset rather than being rewritten mid-implementation.

  **The five amendments** (sources: `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-C-adapters-design.md`
  commits `71fd307` and `b1a8cea`; cross-references in
  `{{DEV_ROOT}}/Mnemosyne/LLM_STATE/mnemosyne-orchestrator/memory.md`
  under the "Harness adapter layer" and "No slash commands inside the
  harness" entries):

  1. **`HarnessSession::send_user_message(&self, text)`** — new trait
     method. TUI forwards user-typed messages into an in-flight session
     mid-turn via this call. Required by C's Q1 (bidirectional stream-json
     supports reading model output *and* forwarding user messages
     simultaneously, no PTY needed).
  2. **`HarnessSession` methods change from `&mut self` to `&self` with
     `Send + Sync` bound.** The `ClaudeCodeSession` actor owns the only
     mutable state; the trait surface is now shareable across threads.
     This is what lets the executor clone an `Arc` to hand one handle to
     the output drainer and another to the input forwarder.
  3. **`LlmHarnessExecutor` storage changes from `Box<dyn HarnessSession>`
     to `Arc<dyn HarnessSession>`** and gains a `user_input_sender()`
     method that the TUI uses to wire user-typed messages into the
     session. The executor now spawns **two threads** per session: an
     output-drainer that reads `OutputChunk`s off the session and pushes
     them to `output_tx`, and an input-forwarder that reads user messages
     off a `crossbeam-channel` receiver and calls `send_user_message` on
     the session.
  4. **`OutputChunkKind` gains a `SessionLifecycle` variant** with
     documented stable text formats `"ready"`, `"turn_complete:<subtype>"`,
     and `"exited:<status>"`. This is the protocol-level observation
     channel surfacing Claude Code's `result` events and session
     state transitions without violating the "no control channel"
     rule (observation is allowed; control is forbidden). See
     memory.md "No slash commands inside the harness" for the
     control-vs-observation distinction.
  5. **`LlmHarnessExecutor` runs `Stdout` chunks through a configurable
     completion-sentinel matcher with sliding-buffer detection.** This
     is the task-level completion signal (distinct from protocol-level
     `SessionLifecycle::TurnComplete`) — each phase prompt ends with
     "when finished say READY FOR THE NEXT PHASE" and the executor
     watches assistant-text output for that sentinel. Sentinel
     detection lives in B (not C) because sentinels are coupled to
     phase prompts (which B owns) and the mechanism is harness-agnostic.
     The matcher must be sliding-buffer based (sentinel may span
     multiple chunks) and configurable per phase. **Validated by the
     BEAM PTY spike** (Session 10, 2026-04-15; `spikes/beam_pty/`):
     sliding-buffer with window bounded to `sentinel_size - 1` bytes
     works across single-chunk, split, drip, false-prefix, and
     false-overlap cases. See memory.md for details.

  **Work to perform.** (a) Update `{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md`
  §2.2 (trait definitions) and §4.1 (`HarnessAdapter` / `HarnessSession`
  trait shape) to record all five amendments inline, with a dated note
  pointing at C's design doc as the origin. (b) Adjust the downstream
  "Define core abstractions and types" task description so that its
  type list includes the amended `OutputChunkKind::SessionLifecycle`
  variant, `HarnessSession` surface (`&self` + `Send + Sync` +
  `send_user_message`), and the executor-level sentinel-matcher type.
  (c) If any other task in this backlog references `Box<dyn
  HarnessSession>` or the old `&mut self` shape, update it consistently.
  (d) Add a note to the `PhaseRunner::run_phase` task (and/or the
  `LlmHarnessExecutor` task if separate) that the executor spawns two
  threads per session and owns the sentinel matcher.

  **No runtime code** — this task is spec + backlog editing only. It
  exists as a gate so the downstream implementation tasks are written
  against the correct contract. Acceptance: the spec and backlog are
  internally consistent with C's design doc and with orchestrator
  `memory.md`, verified by a manual re-read of §2.2, §4.1, and every
  task description that touches `HarnessSession` / `OutputChunk` /
  `LlmHarnessExecutor`.
- **Results:** _pending_

### Absorb LLM_CONTEXT 2026-04 overhaul into B's design + types `[amendment]`
- **Status:** not_started
- **Dependencies:** Task 0
- **Description:** LLM_CONTEXT's upstream shape shifted after B's
  brainstorm: it now ships a four-phase cycle (work → reflect →
  compact → triage, compact conditional on a wc-word-count trigger),
  phase-file-factored composition (shared `phases/<phase>.md` +
  optional per-plan `prompt-<phase>.md` override), a
  `fixed-memory/memory-style.md` file read by both reflect and
  compact, an opt-in `pre-work.sh` executable hook invoked before
  every work phase, a `{{RELATED_PLANS}}` synthesised placeholder,
  a `related-plans.md` schema using Parents/Children only (siblings
  auto-discovered), and ISO 8601 UTC timestamps for session-log
  entries. All seven changes post-date B's 2026-04-12 brainstorm and
  must be absorbed into the design doc and backlog *before* the
  "Define core abstractions and types" task is started so that types,
  `PhaseRunner` logic, and the prompts-vendor module land in their
  amended shape from the outset.

  **Work to perform.**

  (a) Update `{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md`
  to reflect the four-phase cycle. §2.2 and §3 (the state machine and
  phase transitions) must include `compact` as a phase, document the
  conditional trigger (`wc -w memory.md <= compact-baseline + HEADROOM`
  with HEADROOM a runtime constant initially 1500 words), and specify
  that reflect ALWAYS writes `compact` as its nominal next phase —
  the runner-level logic decides whether to invoke compact or skip
  to triage. Compact is "strictly lossless rewrite of memory.md";
  reflect is the only lossy-pruning phase.

  (b) Amend the `plan-state.md` YAML frontmatter phase enum in §2.2.2
  to include `compact`, making the full set `work | reflect | compact
  | triage`. Add a `compact_baseline: u64` field (or equivalent) to
  plan state, matching `run-plan.sh`'s per-plan `compact-baseline`
  file semantics.

  (c) Specify the phase composition mechanism in §2.2.3 and §3: a
  phase prompt is produced by taking the embedded shared phase file
  (from `prompts::PHASE_<PHASE>`), appending the plan's optional
  `prompt-<phase>.md` override if present, then substituting
  placeholders. `StagingDirectory::render` materialises the result
  into the staging directory. Forbid any plan from *replacing* the
  shared phase content; overrides are additive only.

  (d) Add an "optional `pre-work.sh` hook" section to the spec. The
  contract: executable at `<plan>/pre-work.sh`, absent or
  non-executable files are silently skipped, runs from the project
  root, only before work phase (never reflect/compact/triage), runs
  after the defensive `rm -f latest-session.md` cleanup, non-zero
  exit aborts the whole cycle. Specify the `PhaseRunner` integration
  point where this hook is invoked.

  (e) Add `{{RELATED_PLANS}}` to the substitution placeholder list in
  the spec. Value is a synthesised block built by walking the project
  (for siblings) and the Parents/Children declared in
  `related-plans.md` (for peer projects). `run-plan.sh`'s synthesis
  algorithm is the reference: walk `$PROJECT/LLM_STATE/` for sibling
  plans (dirs containing `plan-state.md`), then for each Parent /
  Child entry walk that peer project's `LLM_STATE/` for its plans.
  `{{RELATED_PLANS}}` is a forward-only placeholder — it is not
  reverse-substituted on copy-back.

  (f) Amend the `related-plans.md` spec section: Parents + Children
  only; no Siblings section. Siblings are auto-discovered by walking
  the project's plan tree.

  (g) Specify ISO 8601 UTC-with-seconds timestamps for session-log
  entries (`date -u '+%Y-%m-%dT%H:%M:%SZ'`) in the session-log format
  section. Latest-session.md is written by the work phase (not
  pre-created) and deleted by the runner before each work phase
  starts; the runner appends it to session-log.md post-hoc after the
  work phase exits successfully. No LLM phase ever reads
  session-log.md.

  (h) Update the embedded prompts task (below) with the new vendor
  list: `phases/{work,reflect,compact,triage}.md`,
  `fixed-memory/{coding-style,coding-style-rust,memory-style}.md`,
  `create-plan.md`. Remove any reference to `backlog-plan.md` or
  `create-a-multi-session-plan.md` — those are deleted/renamed
  upstream.

  (i) Update every other task in this backlog that references a
  three-phase cycle, `run-backlog-plan.sh`, `BACKLOG_PLAN_SPEC`, or
  the old vendor list, so descriptions are internally consistent with
  the amendments.

  **No runtime code** — this task is spec + backlog editing only. It
  exists as a gate so the downstream implementation tasks are written
  against the correct contract. Acceptance: the spec and backlog are
  internally consistent with LLM_CONTEXT's current upstream shape,
  verified by a manual re-read of §2.2, §3, and every task
  description that touches phases, prompts, placeholders,
  related-plans, or the runner state machine. Session-log entries in
  the orchestrator plan already use the new ISO 8601 format, which is
  a ground truth to verify against.
- **Results:** _pending_

### Absorb BEAM pivot into B's design, spec, and backlog `[amendment]`
- **Status:** not_started
- **Dependencies:** Task 0
- **Description:** The orchestrator's Session 9 committed Mnemosyne to a
  persistent BEAM daemon (Elixir/OTP), replacing the original Rust
  single-process architecture. This is a load-bearing amendment that
  touches every task in this backlog. Concrete work:

  (a) Rewrite `{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md`
  to replace Rust-specific constructs with Elixir/OTP equivalents:
  traits → behaviours, structs → typed structs/maps, `Arc<dyn ...>` →
  GenServer references, `tokio` tasks → OTP processes, `include_str!` →
  embedded resources or `priv/` files, `serde_yaml` → a YAML library
  (e.g. `yaml_elixir`), `ratatui` → daemon client TUI, `fs2` file locks
  → `:flock` or equivalent.

  (b) Rewrite PhaseRunner as a `PlanActor` GenServer: phase transitions
  arrive as `{:run_phase, _}` messages instead of direct function calls.
  The 13-step `run_phase` flow becomes a GenServer `handle_call` or
  `handle_cast` handler.

  (c) Apply `plan-state.md` schema pruning: remove `plan-id`,
  `host-project`, `dev-root` fields; add `description:` field.

  (d) Rename `{{RELATED_PLANS}}` placeholder to `{{VAULT_CATALOG}}`
  throughout the spec, backlog, and memory. Delete references to
  `related-plans.md` — vault catalog replaces it.

  (e) Rewrite the TUI task: it is now a daemon client, not the process
  main loop. `RatatuiDriver` is replaced by a client that connects to
  the daemon.

  (f) Add DispatchProcessor and QueryProcessor as phase-exit hooks in
  the spec.

  (g) Update every task in this backlog that references Rust crates,
  Rust traits, Rust-specific patterns, or the old single-process
  architecture.

  **No runtime code** — this task is spec + backlog + memory editing
  only. It exists as a gate so downstream implementation tasks are
  written against the correct Elixir/OTP contract. Acceptance: the spec
  and backlog contain zero stale Rust references and are internally
  consistent with the BEAM daemon architecture.
- **Results:** _pending_

### Define core abstractions and types `[types]`
- **Status:** not_started
- **Dependencies:** Task 0, Absorb Sub-C trait amendments into B's design + types, Absorb LLM_CONTEXT 2026-04 overhaul into B's design + types, Absorb BEAM pivot into B's design, spec, and backlog
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
  loading the vendored upstream files from `{{PROJECT}}/prompts/`.
  Expose as `&'static str` constants: `PHASE_WORK`, `PHASE_REFLECT`,
  `PHASE_COMPACT`, `PHASE_TRIAGE` (the four shared phase prompts),
  `CODING_STYLE`, `CODING_STYLE_RUST`, `MEMORY_STYLE` (the three
  fixed-memory files — `MEMORY_STYLE` is read by both reflect and
  compact and is the lossless-rewrite contract's stability anchor),
  and `CREATE_PLAN` (the plan-creation spec). Implement
  `materialise_into(prompts_dir: &Path) -> Result<()>` that writes
  every file into a target directory preserving upstream's
  `phases/` and `fixed-memory/` subdirectory layout. Copy the actual
  content from `{{DEV_ROOT}}/LLM_CONTEXT/phases/{work,reflect,compact,triage}.md`,
  `{{DEV_ROOT}}/LLM_CONTEXT/fixed-memory/{coding-style,coding-style-rust,memory-style}.md`,
  and `{{DEV_ROOT}}/LLM_CONTEXT/create-plan.md` into
  `{{PROJECT}}/prompts/` as part of this task. The pre-overhaul
  filenames `backlog-plan.md` and `create-a-multi-session-plan.md`
  are deleted/renamed upstream and must NOT appear in `prompts.rs`
  or in the vendored tree. Unit tests verify: materialise produces
  the expected files at the expected paths, binary-level byte-equal
  content with the vendored sources, subdirectory layout preserved.
- **Results:** _pending_

### Implement placeholder substitution algorithm `[substitution]`
- **Status:** not_started
- **Dependencies:** Define core abstractions and types
- **Description:** Implement forward and reverse substitution as pure
  functions against five placeholders (`{{DEV_ROOT}}`, `{{PROJECT}}`,
  `{{PLAN}}`, `{{PROMPTS}}`, `{{RELATED_PLANS}}`). Forward: walk a
  file's text and replace each placeholder with its resolved value —
  absolute paths for `{{DEV_ROOT}}` / `{{PROJECT}}` / `{{PLAN}}` /
  `{{PROMPTS}}`, a synthesised multi-line block of sibling / parent /
  child plan paths for `{{RELATED_PLANS}}`. Reverse: walk a file's
  text and replace each absolute path with its placeholder, using
  longest-match-first ordering (PLAN before PROJECT before DEV_ROOT)
  to avoid prefix collisions. `{{RELATED_PLANS}}` is forward-only —
  reverse substitution does not need to reconstruct it. Use `regex`
  with a static precompiled pattern. Unit tests cover: all five
  placeholders substituted correctly, longest-match-first ordering
  against prefix collision inputs, round-trip (forward then reverse)
  returns the original for non-`{{RELATED_PLANS}}` placeholders, edge
  cases (empty file, file with no placeholders, file with multiple
  occurrences of the same placeholder).
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
  directory. Test coverage: full happy path (work → reflect → compact
  → triage → work round-trip AND work → reflect → triage → work
  round-trip when the compact trigger skips compact), illegal
  transition rejection, copy-back rejection aborts cleanly, hook
  fires exactly once on reflect exit, hook does NOT fire on work /
  compact / triage exit, state persistence at every transition
  boundary, event emission order, compact-trigger wc-word-count
  logic skips compact under the headroom and runs it above.
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
  phase (`work` → backlog.md + latest-session.md; `reflect` →
  memory.md (reads latest-session.md as input); `compact` → memory.md
  (lossless rewrite); `triage` → backlog.md). Spawns `$EDITOR`
  (resolved from the `EDITOR` environment variable; fall back to
  hard error if unset) blocked on the target files. Waits for editor
  exit. Returns `Ok(())` on clean exit, `ExecutorError::EditorFailed`
  on non-zero. `interrupt()` is a no-op. Integration tests use a
  scripted editor stub (e.g., a shell script that edits a file then
  exits) to exercise the full flow through `PhaseRunner`, including
  the compact phase path.
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
  LLM_CONTEXT's `run-plan.sh` to `plan-state.md` + Mnemosyne v1. This
  depends on sub-project G's migration logic being available — if G
  is not yet done, perform the migration manually as a one-shot for
  these two plans as part of this task's scope (document the manual
  steps clearly). Run a full work → reflect → (compact) → triage
  cycle of the orchestrator plan on Mnemosyne v1 end-to-end: Ratatui
  TUI, real Claude Code harness via the stub adapter, real ingestion
  pipeline from sub-E, real vault at `<dev-root>/Mnemosyne-vault/`.
  Must exercise both the compact-skipped and compact-run branches of
  the wc-word-count trigger. Success criteria: cycle completes
  cleanly, all events emitted correctly, plan state persisted
  consistently, ingestion fired exactly once, no hard errors, the
  user can retire LLM_CONTEXT's `run-plan.sh` for these two plans
  and proceed on Mnemosyne for all future sessions. This is the
  moment v1 ships.
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

### Adopt sub-M observability framework — phase lifecycle instrumentation + TUI bridge consumer `[m-adoption]`
- **Status:** not_started
- **Dependencies:** sub-M Task 12 (`ObservabilityHarness`) landed; this
  task lives in B's backlog because B owns the call sites being
  instrumented
- **Description:** Landed by sub-project M's brainstorm
  (2026-04-13, Session 7 of the mnemosyne-orchestrator plan). M's
  design doc at
  `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-M-observability-design.md`
  §10 specifies M's cross-plan adoption requirements; this task is B's
  share.

  Concretely:
  1. Add `tracing::instrument` annotations on `PhaseRunner::run_phase`
     and on each `PhaseExecutor::execute` implementation. Span name
     `phase`, fields `plan_id`, `phase`. The span context is consumed
     by M's `MnemosyneEventLayer` to attribute events to the right
     phase scope.
  2. Emit `MnemosyneEvent::PhaseLifecycle` events at the numbered steps
     of `PhaseRunner::run_phase`:
     - Step 5: `PhaseLifecycleKind::Started { phase, plan_id, at }`
     - Step 10: `PhaseLifecycleKind::ExitedClean { phase, transitioned_to, at }`
     - Step 11: `PhaseLifecycleKind::ReflectHookFired { plan_id, at }`
     - Interrupted-state path: `PhaseLifecycleKind::Interrupted { phase, forensics_dir, at }`
     - ExecutorFailed path: `PhaseLifecycleKind::ExecutorFailed { phase, error, at }`
     Use the `mnemosyne_event!` macro (provided by sub-M Task 6) rather
     than calling `tracing::event!` directly.
  3. Replace any draft `eprintln!`-style debug logging in `PhaseRunner`
     with `mnemosyne_event!(Level::DEBUG, MnemosyneEvent::Diagnostic { ... })`
     calls. The `EnvFilter` (from M Task 11) controls visibility; the
     events are always emitted.
  4. Wire `TuiBridgeLayer`'s `mpsc::Receiver<MnemosyneEvent>` into B's
     TUI module event loop. The TUI should subscribe via
     `harness.subscribe_tui()` (M Task 12 API) at startup and consume
     events in the existing `tokio::select!` over the TUI input
     channel + new MnemosyneEvent channel. Status bar gauges and the
     event log tail panel render from this stream per §11.1 of M's
     design doc.
  5. Wire the Risk 5 dump path: in `PhaseRunner::run_phase`'s error
     branches (steps 1-13 hard-error boundaries, executor failure
     branch, interrupt branch), call
     `observability::dump_event_tail(harness, session_id, plan_id, phase, 1000)`
     before returning the error. The dump path is defined by sub-M
     Task 13.

  TDD: write tests that drive a fixture `PhaseRunner::run_phase` cycle
  and assert the expected `MnemosyneEvent` sequence appears on the TUI
  bridge channel. Layer 3 integration test exists in sub-M Task 19;
  this task only needs unit tests on B's side.

  No changes to B's existing trait surface or to the `PhaseEvent`
  channel. Both B's `PhaseEvent` channel and M's `MnemosyneEvent` bus
  coexist during the parallel-emit window. After M v1 ships and the
  verification window passes, a future task collapses B's channel into
  M's bus (deferred — not part of M v1 scope).
- **Results:** _pending_
