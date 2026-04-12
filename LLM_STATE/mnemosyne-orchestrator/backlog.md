# Backlog — Mnemosyne Orchestrator

Initial tasks for merging LLM_CONTEXT functionality into Mnemosyne. Tasks are listed
in approximately recommended order; the work phase picks the best next task with
input from the user.

## Task Backlog

### Complete LLM_CONTEXT punch-list stop-gap `[stop-gap]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Execute the small-fix version of the LLM_CONTEXT restructure
  punch list (Option C from the originating brainstorm: `{{PROJECT}}`-prefixed
  placeholders inside README/file content with work-prompt instructions to
  substitute). NOT the larger restructure (READMEs human-only,
  `LLM_STATE/project.md`, `knowledge/` relocation), which is deferred and likely
  subsumed by the orchestrator design.

  Scope:

  - **APIAnyware-MacOS issue 1**: rewrite the 6 subdirectory READMEs to use
    `{{PROJECT}}/...` placeholders inside their reading-list paths. Affected
    files (paths relative to `{{DEV_ROOT}}/APIAnyware-MacOS/`):
    - `generation/targets/racket-functional/README.md`
    - `generation/targets/racket-oo/README.md`
    - `generation/targets/racket-oo/apps/README.md`
    - `generation/targets/racket-oo/apps/hello-window/README.md`
    - `generation/targets/racket-oo/apps/counter/README.md`
    - `generation/targets/racket-oo/apps/ui-controls-gallery/README.md`
  - **APIAnyware-MacOS issue 2**: rewrite the root README's `../TestAnyware/`
    reference to use `{{DEV_ROOT}}/TestAnyware/`.
  - **APIAnyware-MacOS work prompts**: add a paragraph instructing the LLM that
    any README it Reads inside the project may contain `{{PROJECT}}` and
    `{{DEV_ROOT}}` placeholders, and to substitute them with the absolute paths
    supplied by the prompt before passing the path to the Read tool.
  - **GUIVisionVMDriver issue 3**: consolidate the mild duplication between
    "Integration Testing" (existing user-facing) and "Building from Source"
    (newer dev-facing) sections in `README.md`.

  `knowledge/README.md` is correctly unaffected — verified during the originating
  brainstorm that it has no cross-subdirectory references, despite being on the
  original punch list. Issue 4 (untracked
  `analysis/scripts/llm-annotate-subagent.md`) is unrelated and remains untouched.

  This task spans two repos under `{{DEV_ROOT}}/`. Per-repo commits and pushes
  are required. The convention this task establishes (`{{PROJECT}}`-prefixed
  references in LLM-Read files) is the same convention the Mnemosyne plans
  created by subsequent tasks will use, so finishing this first means subsequent
  brainstorm output is consistent from day one.
- **Results:** _pending_

### Brainstorm sub-project E — post-session knowledge ingestion model `[brainstorm]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Brainstorm and design how Mnemosyne (as parent process) reads
  each plan's outputs after a session and ingests them into Tier 1 / Tier 2.
  Cover: when ingestion fires (session end, phase boundary, both), what files
  are inspected, what triggers a new Mnemosyne entry vs. an update vs. no
  action, contradiction handling, confidence assignment, axis assignment (which
  Mnemosyne axis the entry belongs to), how the LLM signals "this is worth
  promoting" without invoking Mnemosyne CLI.

  E is the conceptual core of the inversion — almost every other sub-project
  depends on understanding how the parent observes and absorbs a session's
  output. Doing E first means downstream brainstorms have a stable foundation.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-E-ingestion-design.md`
  and a sibling LLM_CONTEXT plan at `{{PROJECT}}/LLM_STATE/sub-E-ingestion/`
  containing the implementation backlog.
- **Results:** _pending_

### Brainstorm sub-project B — phase cycle reimplementation in Rust `[brainstorm]`
- **Status:** not_started
- **Dependencies:** sub-project E (task above)
- **Description:** Brainstorm how the work → reflect → triage cycle moves from
  `{{DEV_ROOT}}/LLM_CONTEXT/run-backlog-plan.sh` into Mnemosyne's Rust code.
  Cover: plan discovery, phase state machine, prompt loading and substitution,
  harness invocation hand-off (depends on sub-project C), session lifecycle,
  error handling, recovery from interrupted sessions, the inter-phase
  Enter/Ctrl-C UX, what happens when the user kills mid-phase.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-B-phase-cycle-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-B-phase-cycle/`.
- **Results:** _pending_

### Brainstorm sub-project C — harness adapter layer `[brainstorm]`
- **Status:** not_started
- **Dependencies:** sub-project E
- **Description:** Design the adapter abstraction over Claude Code, Codex, Pi,
  and future harnesses. Cover: spawn semantics, prompt passing (CLI arg vs.
  stdin vs. file), output capture, terminal/PTY handling, lifecycle (start /
  attach / detach / end), what's common across adapters and what's per-adapter,
  v1 scope (one harness or multiple), how missing harnesses are detected and
  reported.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-C-adapters-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-C-adapters/`.
- **Results:** _pending_

### Brainstorm sub-project A — DEV_ROOT global knowledge store `[brainstorm]`
- **Status:** not_started
- **Dependencies:** none (but tied to sub-project D for locking)
- **Description:** Design the relocation of the Mnemosyne global knowledge store
  from `~/.mnemosyne/` to a visible location under `{{DEV_ROOT}}`. Cover:
  specific subpath, init flow, migration from existing `~/.mnemosyne/`
  installations, git workflow (one repo or multiple), interaction with
  sub-project D's locking model, what happens when Mnemosyne is used outside a
  DEV_ROOT-anchored workflow, sync between machines, gitignored vs. tracked
  subdirectories.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-A-global-store-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-A-global-store/`.
- **Results:** _pending_

### Brainstorm sub-project F — plan hierarchy and permanent root plan `[brainstorm]`
- **Status:** not_started
- **Dependencies:** sub-project E, sub-project B
- **Description:** Design the plan hierarchy data model in Mnemosyne. Confirm or
  revise the provisional choice (N-level filesystem nesting with leaf-dir plans
  + marker file + special root plan location). Cover: what the permanent root
  plan holds (cross-cutting backlog, process state — NOT knowledge), how
  process state walks up the hierarchy, how Mnemosyne discovers and indexes
  plans across many projects, how a sub-plan's triage promotes a cross-cutting
  task to an ancestor.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-F-hierarchy-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-F-hierarchy/`.
- **Results:** _pending_

### Brainstorm sub-project G — migration strategy `[brainstorm]`
- **Status:** not_started
- **Dependencies:** none (parallel)
- **Description:** Design how existing LLM_CONTEXT users (the four projects:
  APIAnyware-MacOS, GUIVisionVMDriver, Modaliser-Racket, RacketPro) and existing
  Mnemosyne v0.1.0 users transition to the unified orchestrator. Cover:
  per-project migration steps, deprecation timeline for the LLM_CONTEXT
  directory and `run-backlog-plan.sh`, deprecation timeline for the Mnemosyne
  Claude Code plugin, what data needs to be migrated vs. what stays in place,
  rollback story, how the existing Mnemosyne v0.1.0 TODO items relate to the
  orchestrator timeline.

  G runs in parallel with other sub-projects — its design needs to evolve as
  the others reveal what's actually being changed.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-G-migration-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-G-migration/`.
- **Results:** _pending_

### Brainstorm sub-project D — multi-instance concurrency model `[brainstorm]`
- **Status:** not_started
- **Dependencies:** sub-project A
- **Description:** Design how multiple Mnemosyne processes run concurrently
  against the shared knowledge store. Cover: locking primitive choice (`flock` /
  `.lock` file / SQLite-backed index / something else), reader-writer semantics,
  granularity (whole store vs. per-axis vs. per-entry), behavior under
  contention, behavior under crashed locks, how plan files (which are owned by
  one instance at a time) interact with the global store (which is shared).

  Explicitly NOT a TUI multiplexer — that's cut from v1. Each Mnemosyne instance
  runs in its own terminal; user-side tmux/terminal-tabs handles the
  multiplexing.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-D-concurrency-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-D-concurrency/`.
- **Results:** _pending_

### Brainstorm sub-project H — fold Mnemosyne Claude Code skills into orchestrator `[brainstorm]`
- **Status:** not_started
- **Dependencies:** sub-project B
- **Description:** Design how the 7 existing Mnemosyne Claude Code skills
  (`/begin-work`, `/reflect`, `/setup-knowledge`, `/create-plan`, `/curate-global`,
  `/promote-global`, `/explore-knowledge`) get absorbed into the orchestrator's
  phase cycle and CLI subcommands. Cover: which become phase prompts, which
  become Mnemosyne CLI subcommands, which are eliminated by the new architecture,
  what happens to the existing plugin during the transition, deprecation path
  for the plugin itself.

  Output: design doc at `{{PROJECT}}/docs/superpowers/specs/2026-MM-DD-sub-H-skills-fold-design.md`
  and a sibling plan at `{{PROJECT}}/LLM_STATE/sub-H-skills/`.
- **Results:** _pending_

### Decide v1 scope cut `[decision]`
- **Status:** not_started
- **Dependencies:** all sub-project brainstorms (tasks above)
- **Description:** Once sub-projects E, B, C, A, F, G, D, H have been
  brainstormed and their design docs and implementation plans exist, decide
  what's actually in v1 vs. deferred to v2. Update `{{PLAN}}/memory.md` with
  the v1 cut. Adjust dependent implementation plans accordingly. This is the
  scope-discipline gate before implementation begins in earnest.
- **Results:** _pending_

### Decide TheExperimentalist repo fate `[decision]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Decide whether the TheExperimentalist repo
  (`{{DEV_ROOT}}/TheExperimentalist`) gets deleted, archived (read-only with a
  redirect note pointing at this plan), or repurposed for something else.
  Update `{{PLAN}}/memory.md` with the decision. Execute the action. Not
  blocking any other work; can be picked up at any time.
- **Results:** _pending_
