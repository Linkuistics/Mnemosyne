# Backlog — Mnemosyne Orchestrator

Initial tasks for merging LLM_CONTEXT functionality into Mnemosyne. Tasks are listed
in approximately recommended order; the work phase picks the best next task with
input from the user.

Ordering reflects the sub-project dependency chain recorded in `{{PLAN}}/memory.md`:
**~~E done~~ → ~~B done~~ → ~~C done~~ → A → F → D → H → I**, with **G** and the
newly-surfaced **M (Observability)** running in parallel (placed last for
file-order convenience, not priority), and **K/L** as parallel v1.5+ sidequests
(L is a small independent spike; K is the Obsidian plugin client that depends
on B's implementation landing and on L's recommendation). Sub-project
**J is obsolete** — folded into B as `ManualEditorExecutor`, see memory.md
for the rationale. Sub-project **M (Observability — logging, metrics
collection, display, analysis)** was surfaced during sub-project C's
brainstorm (Session 6, 2026-04-13) as a new candidate; C's `SpawnLatencyReport`
is a tactical seed that M will own and integrate into a broader event/metrics
framework.

**The Obsidian symlink validation spike PASSED 6/6 on both platforms** in
Session 5 (2026-04-13) — driven via the `guivision` CLI against
`guivision-golden-macos-tahoe` and `guivision-golden-linux-24.04` with
Obsidian 1.12.7 + Dataview 0.5.67 pinned identically. The vault-as-view-
over-symlinks framing stands; the hard-copy-staging fallback does NOT
need to be activated; sub-project A's brainstorm and sub-project B's
implementation can proceed on the symlinked-vault baseline without
further investigation. Evidence at
`{{PROJECT}}/tests/fixtures/obsidian-validation/results/{macos,linux}/`
(commit `98ef7db`); architectural consequence captured in
`{{PLAN}}/memory.md`. The spike also established a canonical
guivision + OCR evidence pattern (see the "UI/integration spike
validation" memory entry) which future spikes — sub-project L's
terminal-plugin spike, sub-project I and K UI work, and sub-project B's
v1 dogfood acceptance test — should follow. Sub-project B's sibling-plan
Task 0 (the same spike) has been marked done in
`{{PROJECT}}/LLM_STATE/sub-B-phase-cycle/backlog.md` with a pointer to
this evidence, unblocking B's downstream implementation tasks.

Sub-project E's brainstorm completed 2026-04-12; its design doc lives at
`{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-E-ingestion-design.md` and its
implementation sibling plan at `{{PROJECT}}/LLM_STATE/sub-E-ingestion/`. Sub-project
B's brainstorm completed 2026-04-12; its design doc lives at
`{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md` and its
implementation sibling plan at `{{PROJECT}}/LLM_STATE/sub-B-phase-cycle/`.
Sub-project C's brainstorm completed 2026-04-13; its design doc lives at
`{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-C-adapters-design.md`
(commits `71fd307` and `b1a8cea`) and its implementation sibling plan at
`{{PROJECT}}/LLM_STATE/sub-C-adapters/` (commit `9dac743`). The full trail of
E's, B's, and C's decisions is distilled into `{{PLAN}}/memory.md`; the
raw session record is in `{{PLAN}}/session-log.md`.

## Task Backlog

### Brainstorm sub-project C — harness adapter layer `[brainstorm]`
- **Status:** done
- **Dependencies:** none (sub-projects B and E both complete; their cross-cutting
  requirements for C are captured in `{{PLAN}}/memory.md` and expanded below)
- **Description:** Design the adapter abstraction over Claude Code, Codex, Pi,
  and future harnesses. Cover: spawn semantics, prompt passing (CLI arg vs.
  stdin vs. file), output capture, terminal/PTY handling, lifecycle (start /
  attach / detach / end), what's common across adapters and what's per-adapter,
  v1 scope (one harness or multiple), how missing harnesses are detected and
  reported.

  **Priority note.** C is the next brainstorm pick. It is B's critical sibling
  dependency for swapping B's temporary `LlmHarnessExecutor` stub to a real
  implementation — C landing unblocks B's v1 dogfood acceptance test.

  **Requirements inherited from sub-project B** (see `{{PLAN}}/memory.md` and
  B's design doc §4.1):
  - `HarnessAdapter` trait shape is specified by B; C implements against that
    shape. The trait covers spawn, prompt passing, one-way streaming output,
    lifecycle, and tool profile application.
  - **No harness-to-Mnemosyne callback channel.** Per the "no slash commands
    in the harness" architectural decision, output streams are one-way only.
    Every user action must flow through Mnemosyne's TUI, never through harness
    slash commands or callbacks.
  - **Cold-spawn budget: <3s** — honoured jointly with E's ingestion pipeline
    viability requirement. The five-stage pipeline dies if spawn is slow.
  - **Fixture-replay mode is mandatory**, not optional. B's `FixtureReplayExecutor`
    is a first-class `PhaseExecutor` implementor and drives deterministic
    testing of the phase cycle. C's adapter layer must support a replay mode
    that reads canned harness output from test fixtures.
  - **v1 may ship Claude Code only.** The adapter abstraction must exist in
    v1 (for `FixtureReplayExecutor` alone) but only one real harness needs
    an implementor. Codex and Pi adapters can follow.

  **Requirements inherited from sub-project E** (see `{{PLAN}}/memory.md`):
  configurable tool profiles at spawn time — `IngestionMinimal` and
  `ResearchBroad` are the minimum profile set, since Mnemosyne's own internal
  reasoning sessions (ingestion Stage 3/4, research modals) are a first-class
  call site, not just user-facing plan sessions. Runtime tool enforcement
  must happen at the adapter level, not as prompt suggestion. The adapter
  also needs streaming output support so long-running sessions can surface
  partial progress.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-C-adapters-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-C-adapters/`.
- **Results:** **DONE in Session 6 (2026-04-13).** Brainstorm produced the
  full design via the `superpowers:brainstorming` skill across five locked
  decision points plus two post-write user clarifications. Outputs:
  - **Design doc**: `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-C-adapters-design.md`
    (1311 lines, two commits: `71fd307` initial + `b1a8cea` post-write
    amendment).
  - **Sibling implementation plan**: `{{PROJECT}}/LLM_STATE/sub-C-adapters/`
    (commit `9dac743`), with 24 unconditional implementation tasks plus
    2 conditional warm-pool tasks gated on the C-1 dogfood acceptance
    criterion.

  Five locked decisions: **(Q1) bidirectional `--input-format stream-json
  + --output-format stream-json` process model** (no PTY, no terminal
  parsing — Claude Code's structured I/O on stdin/stdout supports both
  reading model output and forwarding user-typed messages mid-session,
  resolving the "user wants live interaction" constraint without going
  to PTY). **(Q2) `src/harness/` module** under the existing single
  binary crate (no workspace conversion). **(Q3) Cold-spawn only in v1**,
  warm-pool work conditional on a measurable acceptance gate (`C-1`:
  p95 < 5s across N≥10 dogfood cycles); a half-day warm-pool reset
  spike (3-check protocol: structured envelope → `/clear`-as-text →
  pre-spawned single-use degradation) fires only if C-1 trips. **(Q4)
  JSON Lines `FixtureRecord` format** (`Output`/`Delay`/`ExpectUserInput`/`Exit`
  records) recorded via a new `mnemosyne dev record-fixture` subcommand
  driving the real adapter under instrumentation. **(Q5) Two-profile
  CLI flag mapping** (`IngestionMinimal` → `--allowed-tools "" --permission-mode default`;
  `ResearchBroad` → no allowed-tools flag + `--permission-mode acceptEdits`)
  with stream-side defence-in-depth tool enforcement as the second layer.

  Two mid-design revisions surfaced via user pushback: **actor-style
  `ClaudeCodeSession`** with single-owner-per-state discipline (three
  threads per session: actor + stdout-reader + stderr-reader; all
  mutable state in the actor; `crossbeam-channel` for typed inboxes/
  outboxes; replaces the original interior-mutability sketch), and
  **process-group termination as a v1 correctness requirement**
  (`process_group(0)` at spawn, `nix::sys::signal::killpg` at
  terminate, two-phase SIGTERM→SIGKILL escalation with 500ms grace;
  not deferred to v1.5).

  Two post-write user clarifications added in commit `b1a8cea`:
  **(1) The "no callback channel" rule disambiguated** — it forbids
  *control* flowing from harness to Mnemosyne (slash commands, programmatic
  callbacks), not *observation* of harness state. Resolved by adding an
  `OutputChunkKind::SessionLifecycle` variant (fourth amendment to B's
  trait) with documented stable text formats `"ready"` /
  `"turn_complete:<subtype>"` / `"exited:<status>"` for protocol-level
  state observation. **(2) Task-level vs protocol-level completion
  separated** — protocol-level "turn over" (Claude Code's `result` event,
  surfaced as `SessionLifecycle`) is necessary but not sufficient; what
  the user actually wants is "the LLM has decided the work is done",
  detected via prompt-instructed sentinel strings (e.g. "READY FOR
  REFLECT") that B's executor watches for in `Stdout` chunks via a
  sliding-buffer matcher. Sentinel detection lives in B (not C) because
  sentinels are coupled to phase prompts (which B owns), the mechanism
  is harness-agnostic, and C should stay focused on harness mechanics.

  **Cross-sub-project requirements going back to Sub-B**: four additive
  trait amendments + one executor-level requirement, all forced by Q1
  and the post-write clarifications, all post-dating B's brainstorm:
  (1) `HarnessSession::send_user_message(&self, text)` new method;
  (2) `HarnessSession` methods change from `&mut self` to `&self` with
  `Send + Sync` bound; (3) `LlmHarnessExecutor` storage changes from
  `Box<dyn HarnessSession>` to `Arc<dyn HarnessSession>` and gains a
  `user_input_sender()` TUI-facing method, spawning two threads
  (output-drainer + input-forwarder); (4) `OutputChunkKind` gains a
  `SessionLifecycle` variant; (5) `LlmHarnessExecutor` runs `Stdout`
  chunks through a configurable completion-sentinel matcher with
  sliding-buffer detection. These amendments land in B's implementation
  phase, not as a B re-brainstorm. Sub-B's sibling-plan needs an
  amendment task added during its next triage to absorb them.

  **New sub-project surfaced**: **Sub-M (Observability — logging,
  metrics collection, display, analysis)**. C's `SpawnLatencyReport` is
  a tactical seed specific to the C-1 acceptance gate, not the start of
  a metrics framework. Sub-M will own the broader story (structured
  logging crate choice, event bus design, metrics aggregation, Obsidian
  explorer integration, the relationship to E's ingestion event stream).
  Added to this backlog as a new brainstorm candidate alongside G.

  **Three new Cargo deps** justified individually under
  integration-over-reinvention: `which` (binary discovery), `nix`
  (`killpg` for process-group termination), `crossbeam-channel`
  (`Sync` channels for the actor architecture). No PTY crate, no
  tokio, no async runtime.

  **C-1 acceptance gate** is the swap-in moment for sub-B's stub
  adapter: when the dogfood orchestrator plan completes 10 full
  work → reflect → triage cycles against the real `ClaudeCodeAdapter`
  on the user's primary dev machine with p95 cold-spawn < 5s, both C
  v1 and (jointly) B v1 are accepted. The real adapter then becomes
  B's adapter of record permanently.

### Brainstorm sub-project A — DEV_ROOT global knowledge store `[brainstorm]`
- **Status:** not_started
- **Dependencies:** none (B brainstorm has fixed the vault framing; A finalises
  the vault location and config override mechanism)
- **Description:** Design the relocation of the Mnemosyne global knowledge store
  from `~/.mnemosyne/` to a visible location under `{{DEV_ROOT}}`. Cover:
  specific subpath, init flow, migration from existing `~/.mnemosyne/`
  installations, git workflow (one repo or multiple), interaction with
  sub-project D's locking model, what happens when Mnemosyne is used outside a
  DEV_ROOT-anchored workflow, sync between machines, gitignored vs. tracked
  subdirectories.

  **Requirements inherited from sub-project B** (see `{{PLAN}}/memory.md`):
  - **Vault layout is fixed by B** — dedicated `<dev-root>/Mnemosyne-vault/`
    with its own `.git`, hosting Tier 2 knowledge natively, hosting runtime
    state natively, and accessing per-project Mnemosyne content via one
    symlink per project at `<vault>/projects/<project-name>` targeting
    `<project>/mnemosyne/`. A designs the vault location (subpath, naming,
    config override), init flow, and bootstrap. A may propose revisions to
    the per-project `<project>/mnemosyne/` directory name, but the
    vault-as-view-over-symlinks framing is load-bearing.
  - **Symlink validation spike RESOLVED (PASS 6/6, 2026-04-13)** — the
    cross-platform Obsidian + symlinks validation spike was executed via
    the `guivision` CLI against `guivision-golden-macos-tahoe` and
    `guivision-golden-linux-24.04` with Obsidian 1.12.7 + Dataview 0.5.67
    pinned identically. All six checks passed on both platforms; evidence
    at `tests/fixtures/obsidian-validation/results/{macos,linux}/`,
    commit `98ef7db`. The vault-as-view-over-symlinks baseline stands.
    A's brainstorm proceeds on this baseline with NO obligation to absorb
    the hard-copy-staging fallback.
  - **Obsidian-native format discipline** — Dataview-friendly kebab-case
    YAML frontmatter, wikilinks for cross-references, tags as first-class
    metadata, a Mnemosyne-provided `.obsidian/` template with Dataview
    required and Templater optional. A's storage format decisions must
    honour this discipline.

  **Requirements inherited from sub-project E** (see `{{PLAN}}/memory.md`):
  Tier 1 and Tier 2 roots must be independently addressable and exposed as
  startup config. Ingestion Stage 5 writes to Tier 1 by default with
  graduation writes targeting Tier 2; tests must be able to point both roots
  at fixture directories. A single "knowledge root" that conflates both tiers
  is insufficient.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-A-global-store-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-A-global-store/`.
- **Results:** _pending_

### Brainstorm sub-project F — plan hierarchy and permanent root plan `[brainstorm]`
- **Status:** not_started
- **Dependencies:** none (sub-project B brainstorm complete; B's contracts on
  F are non-negotiable and listed below)
- **Description:** Design the plan hierarchy data model in Mnemosyne. Confirm or
  revise the provisional choice (N-level filesystem nesting with leaf-dir plans
  + marker file + special root plan location). Cover: what the permanent root
  plan holds (cross-cutting backlog, process state — NOT knowledge), how
  process state walks up the hierarchy, how Mnemosyne discovers and indexes
  plans across many projects, how a sub-plan's triage promotes a cross-cutting
  task to an ancestor.

  **Non-negotiable contracts inherited from sub-project B** (see `{{PLAN}}/memory.md`):
  - **`plan-state.md` marker rule** — a directory is a plan if and only if it
    contains `plan-state.md`. F cannot revise this.
  - **`StagingDirectory::render` descent invariant** — rendering refuses to
    descend into subdirectories containing `plan-state.md`, keeping "one plan
    per Mnemosyne process" intact against the hierarchical layout. F cannot
    revise this.
  - **Plans live at `<project>/mnemosyne/plans/`** with arbitrary nesting. F
    designs everything else about hierarchy semantics — reserved root plan
    locations, parent/child tracking, cross-cutting promotion, discovery and
    indexing — on top of these B contracts.

  **Coordination point with sub-project E** (see `{{PLAN}}/memory.md`): E's
  ingestion design assumes a single host project per plan. Multi-project
  plan hierarchies would require Rule 4 (tier routing) re-examination — F's
  brainstorm must either confirm the single-host assumption or explicitly
  surface the re-design needed in E's implementation plan.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-F-hierarchy-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-F-hierarchy/`.
- **Results:** _pending_

### Brainstorm sub-project D — multi-instance concurrency model `[brainstorm]`
- **Status:** not_started
- **Dependencies:** sub-project A (soft — vault location is still being
  finalised, but B has fixed the plan lock path structure, so D can run in
  parallel with A as long as the vault-root substitution is treated as a
  late binding)
- **Description:** Design how multiple Mnemosyne processes run concurrently
  against the shared knowledge store. Cover: locking primitive choice (`flock` /
  `.lock` file / SQLite-backed index / something else), reader-writer semantics,
  granularity (whole store vs. per-axis vs. per-entry), behavior under
  contention, behavior under crashed locks, how plan files (which are owned by
  one instance at a time) interact with the global store (which is shared).

  **Requirements inherited from sub-project B** (see `{{PLAN}}/memory.md`):
  - **Per-plan advisory lock path is fixed** at `<vault>/runtime/locks/<plan-id>.lock`.
    D designs the locking *primitive* and *semantics*, not the lock file
    location.
  - **Two distinct lock scopes coexist**: per-plan locks (B's phase cycle
    chokepoint — only one Mnemosyne instance drives a given plan at a time)
    and the store-level lock (E's Stage 5 ingestion write lock). D must
    design both consistently.
  - **Vault-scoped runtime state** — `<vault>/runtime/` hosts staging
    directories, interrupts, ingestion events, and locks. D's lock design
    sits inside this runtime subtree.

  **Requirements inherited from sub-project E** (see `{{PLAN}}/memory.md`):
  the store write lock must be acquirable by ingestion Stage 5. Whole-store
  granularity is acceptable for v1 — per-axis / per-entry locking can wait
  for v2. Ingestion must abort gracefully if the lock is unavailable (and
  retry on the next cycle) rather than blocking the phase cycle.

  Explicitly NOT a TUI multiplexer — that's cut from v1. Each Mnemosyne instance
  runs in its own terminal; user-side tmux/terminal-tabs handles the
  multiplexing.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-D-concurrency-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-D-concurrency/`.
- **Results:** _pending_

### Brainstorm sub-project H — fold Mnemosyne Claude Code skills into orchestrator `[brainstorm]`
- **Status:** not_started
- **Dependencies:** none (sub-project B brainstorm complete; J's scope folded
  into B, so H now applies the co-equal-actors principle directly)
- **Description:** Design how the 7 existing Mnemosyne Claude Code skills
  (`/begin-work`, `/reflect`, `/setup-knowledge`, `/create-plan`, `/curate-global`,
  `/promote-global`, `/explore-knowledge`) get absorbed into the orchestrator's
  TUI actions and phase prompts. Cover: which become phase prompts, which
  become Mnemosyne TUI actions, which are eliminated by the new architecture,
  what happens to the existing plugin during the transition, deprecation path
  for the plugin itself.

  **Requirements inherited from sub-project B** (see `{{PLAN}}/memory.md`):
  - **Legacy skills become `TuiAction` enum variants in Mnemosyne core, not
    harness slash commands.** The "no slash commands in harness" decision
    rules out any path that re-exposes a skill as a harness-side callback.
    H's mapping exercise produces TuiAction variants (and phase prompts
    where appropriate), nothing else.
  - **Co-equal-actors is a compile-time invariant via B's `PhaseExecutor`
    trait.** Every TuiAction must work under both `LlmHarnessExecutor` and
    `ManualEditorExecutor` — the human-driven counterpart is enforced by
    the trait chokepoint, not by H-level discipline. H's mapping exercise
    must still identify per-skill what the human-driven form looks like,
    but the enforcement mechanism is already in place.
  - **Sub-project J is obsolete** — J's "human-mode phase affordances"
    scope folded into B. H no longer needs to coordinate with J; H applies
    the co-equal-actors principle directly to the 7 skills being folded in.

  **Requirements inherited from sub-project E** (see `{{PLAN}}/memory.md`):
  every existing Claude Code skill must have a human-driven UI equivalent
  alongside its LLM-driven form — the co-equal-actors principle forbids any
  skill being replaced by an LLM-only workflow.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-H-skills-fold-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-H-skills/`.
- **Results:** _pending_

### Brainstorm sub-project I — Obsidian coverage of Mnemosyne data surfaces `[brainstorm]`
- **Status:** not_started
- **Dependencies:** none (sub-project B brainstorm complete; I's scope is
  substantially reduced from the original "build a unified explorer
  framework" — see below)
- **Description:** **Scope re-framed by sub-project B's brainstorm.** The
  *capability* (human can review, edit, reject, and undo anything Mnemosyne
  writes) is still load-bearing for auto-ingestion safety, but the
  *implementation* is delegated to Obsidian, which is now the committed
  explorer UI for v1. I's job shrinks from "build a unified explorer
  framework" to:
  1. Document which Obsidian features and Dataview queries cover which
     Mnemosyne data surfaces — Tier 1 knowledge, Tier 2 knowledge, plan
     process state (backlog / phase / memory / session-log files across
     every plan Mnemosyne manages), session artifacts (transcripts,
     child-session outputs), ingestion provenance (ingestion events,
     supersession history, graduation history, research transcripts).
  2. Define the default `.obsidian/` template that Mnemosyne ships with —
     required plugins (Dataview), optional plugins (Templater), vault
     configuration, example queries per data surface.
  3. Specify how Mnemosyne's storage format decisions (frontmatter schema,
     wikilinks, tags, directory layout) support the documented queries.
  4. Specify how the ingestion Stage 5 supersession chains surface in
     Obsidian (wikilinks + a Dataview query vs. a dedicated view vs.
     frontmatter back-references) without forcing the user to chase
     cross-references manually.

  **Undo/history semantics collapse** — Obsidian's file-based history plus
  git in the vault covers review and rollback. No custom undo stack is
  needed. I's brainstorm may still surface gaps but must prefer existing
  Obsidian affordances over net-new Mnemosyne code.

  **Accountability substrate remains load-bearing.** E's auto-ingestion
  trade-off is only safe *because* the human can see, review, edit, and
  roll back every write Mnemosyne makes. If I's brainstorm surfaces an
  Obsidian gap that breaks this guarantee, I must either resolve it with
  format/layout changes or escalate to B's implementation plan before
  E's implementation can proceed.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-I-obsidian-coverage-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-I-obsidian-coverage/`
  (name reflects the re-scoped job, not the original explorer-framework
  framing).
- **Results:** _pending_

### Brainstorm sub-project L — Obsidian terminal plugin spike `[brainstorm]`
- **Status:** not_started
- **Dependencies:** none (prerequisite for K but independent of B)
- **Description:** Small investigation task. Evaluate existing Obsidian
  terminal plugins (obsidian-terminal, obsidian-execute-code, others)
  for PTY control, streamed output capture, clean termination, and
  integration with external processes like Claude Code. Surfaced during
  sub-project B's brainstorm (2026-04-12) as a prerequisite for
  sub-project K: if existing plugins don't support the capabilities K
  needs for hosting harness sessions inside Obsidian views, K must
  fork an existing plugin or build a Mnemosyne-specific terminal
  plugin from scratch. L's output directly informs K's scope.

  Cover: plugin inventory, capability comparison, licence
  compatibility, maintenance status, the specific gap between
  existing plugins and K's requirements, and a recommendation on
  fork-vs-build-new.

  **Evidence pattern.** If L's investigation requires running any plugin
  inside Obsidian to verify capabilities, follow the canonical
  guivision + OCR evidence pattern established by the symlink spike
  (Session 5, 2026-04-13) — see the "UI/integration spike validation"
  entry in `{{PLAN}}/memory.md`. Drive Obsidian via the `guivision` CLI
  against `guivision-golden-{macos-tahoe,linux-24.04}` with pinned
  Obsidian + plugin versions, capture per-check evidence as
  `guivision screenshot` + `guivision find-text` artifacts, and commit
  results under `{{PROJECT}}/tests/fixtures/sub-L-terminal-spike/`. SSH /
  rsync / VNC-direct paths are out per `feedback_guivision_cli.md`.

  Output: design doc (short — a few hundred lines) at
  `{{PROJECT}}/docs/superpowers/specs/YYYY-MM-DD-sub-L-obsidian-terminal-spike.md`.
  No sibling plan needed if the recommendation is "use plugin X
  as-is"; sibling plan needed if the recommendation is "build our
  own" or "fork plugin X."
- **Results:** _pending_

### Brainstorm sub-project K — Obsidian plugin client `[brainstorm]`
- **Status:** not_started
- **Dependencies:** sub-project B (brainstorm done; implementation
  must land far enough that the `IpcDriver` protocol is stable),
  sub-project L (Obsidian terminal plugin spike must complete first)
- **Description:** Design the Obsidian plugin client that consumes
  Mnemosyne's stable IPC protocol (hardened in sub-project B via the
  `IpcDriver` compile-time enforcement mechanism) and provides a
  first-class Obsidian-integrated UI alternative to the Ratatui TUI.
  Surfaced during sub-project B's brainstorm (2026-04-12) as "Path 1:
  stage it" — v1 ships ratatui TUI, v1.5+ adds Obsidian plugin as an
  alternative co-equal client of the same Mnemosyne core.

  **Scope.** Mnemosyne command palette inside Obsidian, phase-state
  panel driven by Dataview queries against live `plan-state.md`
  files, terminal-plugin integration for hosting harness sessions
  visibly inside Obsidian (depends on L's evaluation), streaming
  output rendered to an Obsidian view, multi-plan dashboards using
  Dataview to cross-reference live phase state with historical plan
  data.

  **Hard constraint.** The plugin must speak the IPC protocol defined
  during B's implementation. It must not require changes to the Rust
  core beyond additive `PhaseEvent` and `TuiAction` enum variants.
  The plugin lives in its own TypeScript codebase — either as a
  `plugin/` subdirectory under Mnemosyne or as a sibling
  `mnemosyne-obsidian` repo.

  **Non-goal.** K does not replace ratatui. Both UIs coexist in v2;
  a user on SSH or inside an IDE terminal pane uses ratatui, a user
  living in Obsidian uses the plugin.

  Output: design doc at
  `{{PROJECT}}/docs/superpowers/specs/YYYY-MM-DD-sub-K-obsidian-plugin-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-K-obsidian-plugin/`.
- **Results:** _pending_

### Brainstorm sub-project G — migration strategy `[brainstorm]`
- **Status:** not_started
- **Dependencies:** none (runs in parallel with other sub-projects)
- **Description:** Design how existing LLM_CONTEXT users (the four projects:
  APIAnyware-MacOS, GUIVisionVMDriver, Modaliser-Racket, RacketPro) and existing
  Mnemosyne v0.1.0 users transition to the unified orchestrator. Cover:
  per-project migration steps, deprecation timeline for the LLM_CONTEXT
  directory and `run-backlog-plan.sh`, deprecation timeline for the Mnemosyne
  Claude Code plugin, what data needs to be migrated vs. what stays in place,
  rollback story, how the existing Mnemosyne v0.1.0 TODO items relate to the
  orchestrator timeline.

  **Concrete rename tasks inherited from sub-project B** (see `{{PLAN}}/memory.md`):
  - **Per-project directory rename** — existing `<project>/LLM_STATE/` →
    `<project>/mnemosyne/plans/` and `<project>/knowledge/` →
    `<project>/mnemosyne/knowledge/`, preserving any existing nested
    structure. Applies to the four LLM_CONTEXT projects and any Mnemosyne
    v0.1.0 installations with a `knowledge/` directory.
  - **Plan marker file rename** — `phase.md` → `plan-state.md` with YAML
    frontmatter. Existing `phase.md` contents (a single word: `work` /
    `reflect` / `triage`) migrate into the frontmatter.
  - **Tier 2 global knowledge relocation** — `~/.mnemosyne/` →
    `<vault>/knowledge/`. The `<vault>/` location itself is sub-project A's
    decision; G owns the move.
  - **Vault bootstrap** — for each existing project, G's migration script
    must establish the `<vault>/projects/<project-name>` symlink pointing at
    the renamed `<project>/mnemosyne/` directory. The cross-platform
    Obsidian + symlinks validation spike PASSED on both macOS and Linux
    (Session 5, 2026-04-13; commit `98ef7db`), so the symlink-based
    bootstrap stands; the hard-copy + two-way-sync fallback is NOT needed.

  G runs in parallel with other sub-projects — its design needs to evolve as
  the others reveal what's actually being changed. Listed last in file order
  for convenience, not because it's low priority: it can be picked up at any
  time the other brainstorms stall or produce migration-relevant output.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-G-migration-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-G-migration/`.
- **Results:** _pending_

### Brainstorm sub-project M — observability (logging, metrics, display, analysis) `[brainstorm]`
- **Status:** not_started
- **Dependencies:** none (parallel-able with all other brainstorms; surfaced
  during sub-project C's brainstorm in Session 6, 2026-04-13)
- **Description:** Design Mnemosyne's observability framework. Cover:
  structured logging strategy (crate choice — `tracing` / `slog` / `log` /
  custom; subscriber configuration; per-module log levels), event bus
  shape (how phase events, harness lifecycle events, ingestion events,
  user actions, error variants, and tactical instrumentation reports flow
  to a single typed event stream), metrics aggregation (counters,
  histograms, gauges; in-memory ring buffer vs. file-backed; per-session
  / per-plan / per-process scopes), display surfaces (live in the
  Ratatui TUI, persistent in Obsidian via Dataview queries, exported to
  external tools), analysis tooling (querying historical events,
  computing percentiles, surfacing anomalies), the relationship to
  sub-project E's ingestion event stream (which is itself a kind of
  event stream and should compose cleanly with whatever M designs),
  the relationship to sub-project I's Obsidian coverage (events and
  metrics are explorer surfaces too), the migration path for C's
  tactical `SpawnLatencyReport` instrumentation onto the new framework.

  **Surfaced during sub-project C's brainstorm.** C's design deliberately
  ships a *tactical* `SpawnLatencyReport` instrumentation that is
  emitted as an `InternalMessage` chunk and written to
  `<staging>/spawn-latency.json` — purpose-built for the C-1 acceptance
  gate and explicitly *not* the start of a metrics framework. C's
  design records a forward-pointer to this proposed sub-project M so
  that future maintainers know the broader observability story has its
  own home, and so M's brainstorm starts from a clean slate without
  having to retrofit a tactical artifact into the architectural shape.
  See §11.5 of C's design doc.

  **Cross-cutting requirement on every sub-project.** Once M lands, every
  other sub-project's "I want to know what's happening inside this
  component" question routes through M's framework rather than through
  ad-hoc `eprintln!`-style logging or per-component metrics collection.
  M should therefore be brainstormed early enough that B, C, D, and E
  can adopt it during their implementation phases rather than retrofitting
  later. Parallel-able with G — both can run any time.

  **Risk acknowledgement (from C's spec, §9 Risk 5).** v1 of C ships
  with diagnostic-poor failure modes — when a session fails in a way
  the actor doesn't anticipate, the user sees the message but no rich
  context (no recent output buffer, no per-thread state dump). C
  records this as an *accepted* v1 limitation pointing at this sub-project
  as the future home for structured logging. M's brainstorm should
  treat that limitation as a concrete first-day requirement: the
  framework must support "give me the last N events from session X
  with full context" as a primary diagnostic operation.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-M-observability-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-M-observability/`.
- **Results:** _pending_

### Decide v1 scope cut `[decision]`
- **Status:** not_started
- **Dependencies:** all sub-project brainstorms (~~E done~~, ~~B done~~,
  ~~C done~~, A, F, D, H, I, G, M — plus sub-project L's short spike if
  K is being considered for v1.5 alongside v1). J is obsolete (folded
  into B). K is explicitly v1.5+ and not part of the v1 scope cut.
- **Description:** Once every in-scope sub-project has been brainstormed
  and its design doc and implementation plan exist, decide what's actually
  in v1 vs. deferred to v2. Update `{{PLAN}}/memory.md` with the v1 cut.
  Adjust dependent implementation plans accordingly. This is the
  scope-discipline gate before implementation begins in earnest.

  **Sub-project I remains the sensitive point.** Sub-project E's
  brainstorm declared explorers load-bearing for auto-ingestion safety.
  Sub-project B's brainstorm (2026-04-12) re-scoped I's implementation
  obligation: the *capability* (human can review/edit/undo anything
  Mnemosyne writes) is still load-bearing, but the *implementation* is
  delegated to external tools — specifically Obsidian, which is now the
  committed explorer UI. This means I's scope shrinks from "build a
  unified explorer framework" to "document which Obsidian features and
  Dataview queries cover which Mnemosyne data surfaces, with example
  vault configurations." I's brainstorm should absorb this re-scoping
  rather than re-litigating it.

  **Sub-project K (Obsidian plugin client) is not a v1 scope cut
  decision.** K is v1.5+ by design — Path 1 commits to shipping v1 on
  ratatui and adding Obsidian plugin integration later without core
  rework. The v1 scope cut decision should confirm this staging and not
  re-open the plugin-in-v1 discussion unless one of the other
  sub-project brainstorms surfaces new evidence against Path 1.
- **Results:** _pending_
