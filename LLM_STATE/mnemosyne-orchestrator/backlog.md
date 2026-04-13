# Backlog — Mnemosyne Orchestrator

Initial tasks for merging LLM_CONTEXT functionality into Mnemosyne. Tasks are listed
in approximately recommended order; the work phase picks the best next task with
input from the user.

Ordering reflects the sub-project dependency chain recorded in `{{PLAN}}/memory.md`:
**~~E done~~ → ~~B done~~ → ~~C done~~ → ~~M done~~ → A → F → D → H → I**, with
**G** running in parallel throughout and **K/L** as parallel v1.5+ sidequests
(L is a small independent spike; K is the Obsidian plugin client that depends
on B's implementation landing and on L's recommendation). Sub-project **J is
obsolete** — folded into B as `ManualEditorExecutor`, see memory.md for the
rationale.

**Sub-project M (Observability) brainstormed in Session 7 (2026-04-13)** —
design doc at
`{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-M-observability-design.md`
(commit `53f7d4e`), sibling plan at
`{{PROJECT}}/LLM_STATE/sub-M-observability/` with 23 implementation tasks.
Hybrid `tracing` + typed `MnemosyneEvent` enum architecture; five standard
crates; one ~200-line custom Layer. C's tactical `SpawnLatencyReport` migrates
via a parallel-emit window with mechanical ±10ms verification. Risk 5 resolved
via `InMemoryRingLayer::dump_session`. Adoption tasks landed in sub-B / sub-C /
sub-E sibling backlogs by the brainstorm itself; sub-D / sub-F / sub-H / sub-I /
sub-G adoption tasks are queued in sub-M's memory.md and land as those
sub-projects' brainstorms complete. Architectural decisions distilled into
`{{PLAN}}/memory.md` under "Always-on instrumentation; tactical measurement
disclaims framework scope" and the new "Hybrid tracing + typed events" entry.

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

### Brainstorm sub-project A — DEV_ROOT global knowledge store `[brainstorm]`
- **Status:** done
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
- **Results:** **Done 2026-04-13 (Session 7).** Design doc at
  `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-A-global-store-design.md`
  (commit `c81fd48`); sibling plan at `{{PROJECT}}/LLM_STATE/sub-A-global-store/`
  with a fifteen-task implementation backlog (commit pending in this
  session). The brainstorm locked five forking decisions (discovery
  model, init flow shape, gitignore policy, migration scope, identity
  verification) through clarifying questions and then presented five
  design sections for section-by-section approval. Key decisions:
  **(1) Vault discovery is explicit** via `--vault` flag → `MNEMOSYNE_VAULT`
  env var → user config file (`dirs::config_dir().join("mnemosyne/config.toml")`)
  → hard error; no walk-up, no implicit dev-root concept. **(2) Vault
  identity** is verified by a `<vault>/mnemosyne.toml` marker with
  schema-versioned `[vault]` table; the same file hosts optional
  override sections for language profiles and context mappings,
  eliminating the dotfile-without-extension. **(3) `init` has two
  subcommands**: `mnemosyne init <path>` (fresh, default path
  `~/Mnemosyne-vault/`) and `mnemosyne init --from <git-url> [<path>]`
  (clone); plus `mnemosyne config use-vault <path>` (switch vault) and
  `mnemosyne adopt-project <project-path>` (per-machine symlink
  mount). **(4) Gitignore policy**: track `knowledge/`, `archive/`,
  curated `.obsidian/` subset, and `mnemosyne.toml`; gitignore
  `runtime/`, `cache/`, `projects/`, and noisy workspace files.
  **(5) Migration scope dropped** because Mnemosyne has no real v0.1.0
  users — the `migrate` subcommand does not exist, and v0.1.0's
  hardcoded `~/.mnemosyne/` paths in `src/main.rs` plus
  `Config::load(dir)` in `src/config.rs` are deleted outright in B's
  implementation (task 11 and task 12 of sub-A's sibling plan).
  Tier 1 / Tier 2 roots are independently addressable with env-var
  overrides (`MNEMOSYNE_TIER1_ROOT`, `MNEMOSYNE_TIER2_ROOT`) for
  test fixtures, satisfying E's requirement. Surfaced one new
  project-wide open question ("Team-mode usage of Mnemosyne") which
  has been appended to `{{PLAN}}/memory.md`. Produced five
  `{{PLAN}}/memory.md` updates (new entries for explicit discovery,
  schema-versioned marker identity, v0.1.0-is-deletable-not-transitionable;
  update to the existing "Global knowledge store moves from `~/.mnemosyne/`"
  entry; update to the Sub-projects table row for A).

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

### Decide v1 scope cut `[decision]`
- **Status:** not_started
- **Dependencies:** all sub-project brainstorms (~~E done~~, ~~B done~~,
  ~~C done~~, ~~M done~~, A, F, D, H, I, G — plus sub-project L's short
  spike if K is being considered for v1.5 alongside v1). J is obsolete
  (folded into B). K is explicitly v1.5+ and not part of the v1 scope
  cut.
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
