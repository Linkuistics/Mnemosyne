# Memory — Mnemosyne Orchestrator

This plan exists to drive the merge of LLM_CONTEXT functionality into Mnemosyne,
transforming Mnemosyne into a harness-independent LLM orchestrator that owns plan
management, the work cycle, knowledge, and harness session control.

This file is pre-populated with the architectural state captured from the originating
brainstorm session on 2026-04-12. Subsequent reflect phases append, sharpen, and
prune entries per the standard backlog-plan rules.

## Stable architectural decisions

Decisions made in the originating brainstorm and considered settled unless a future
session surfaces evidence that requires revisiting.

### TheExperimentalist is retired
Its conceptual scope (tracking exploratory development) is met by the LLM_CONTEXT
backlog/reflect/triage cycle plus a hierarchy of plans. The original Git-branching
framing in TheExperimentalist's README was the wrong abstraction for what was needed
— Git branches model parallel state, but exploratory development needs temporal
structure with reflection points, which the phase cycle already provides. The repo
will be deleted, archived, or repurposed in a separate cleanup task (backlog item).

### LLM_CONTEXT functionality merges into Mnemosyne
Mnemosyne becomes the single user-facing tool. The user runs `mnemosyne` to start
plans, advance phases, query knowledge, and curate. The `LLM_CONTEXT/` directory
eventually retires once Mnemosyne fully replaces it. Until then, LLM_CONTEXT and the
four projects depending on it (APIAnyware-MacOS, GUIVisionVMDriver, Modaliser-Racket,
RacketPro) must continue working unchanged.

### Mnemosyne becomes the parent process for LLM harness sessions
The current control flow (user runs `claude` → claude is the parent → claude touches
files via tools) is inverted: user runs `mnemosyne` → Mnemosyne is the parent →
Mnemosyne spawns Claude Code, Codex, Pi, or other harnesses as child processes via
PTY. The harness becomes a managed worker. This control inversion is the architectural
move that drives most other decisions in this plan.

### Mnemosyne owns all knowledge — Tier 1 and Tier 2
This is unchanged from current Mnemosyne v0.1.0 design. Per-project knowledge lives
in Mnemosyne's Tier 1 (`knowledge/` directory). Cross-project knowledge lives in
Tier 2 (the global store). The LLM_CONTEXT plan system does NOT host knowledge —
plan memory (`{{PLAN}}/memory.md`) is provisional, a scratch space for in-flight
reflections that may or may not eventually be promoted into Mnemosyne Tier 1.

### Global knowledge store moves from `~/.mnemosyne/` to a visible location under DEV_ROOT
Treated as a first-class git-tracked dev asset, visible alongside project repos.
Specific subpath TBD in sub-project A.

### Knowledge ingestion happens via post-session inspection by Mnemosyne
Because Mnemosyne is now the parent process, it can read each plan's outputs
(`memory.md`, `session-log.md`) directly after a session ends or at phase boundaries.
The LLM never invokes Mnemosyne CLI to "promote" anything. This eliminates the
LLM-discipline failure mode of "did the LLM remember to call back" and matches
Mnemosyne's existing philosophy of curation as a deliberate, separate cognitive step.

### Phase cycle reimplemented in Rust inside Mnemosyne
The work → reflect → triage cycle moves out of `run-backlog-plan.sh` and into Rust
code inside Mnemosyne. Placeholder substitution, prompt loading, phase state
management, and session lifecycle become Mnemosyne functions. The bash script is
retired as part of LLM_CONTEXT retirement.

### Harness adapter layer abstracts Claude Code, Codex, Pi, and future harnesses
Each adapter handles spawn, prompt-passing, output capture, terminal/PTY handling,
and lifecycle for one harness. v1 may ship with only Claude Code; others added later.
This is what makes the orchestrator harness-independent.

### Multi-plan work uses multiple Mnemosyne instances + locking, not a TUI multiplexer
Each Mnemosyne instance runs in its own terminal; the user multiplexes with existing
tools (tmux, terminal tabs, separate windows). A TUI multiplexer is explicitly cut
from v1 — it was the riskiest sub-project and this approach gets the same end result
much more cheaply. Locking on the shared knowledge store is a v1 requirement to
make concurrent instances safe.

### Plan hierarchy uses N-level filesystem nesting with leaf-dir plans + marker file + special root location
Plan dirs are leaves containing a marker file (e.g., `phase.md` or `.plan`).
Intermediate dirs are pure organization, not plans. The project root plan lives at
a special top-level location (e.g., `LLM_STATE/_root/`). Promotion of process state
(NOT knowledge) walks up the hierarchy. Provisionally chosen during the originating
brainstorm; sub-project F may revisit specifics.

### LLM_CONTEXT punch-list issues 1-3 are completed as a stop-gap (Option C)
The small-fix version: `{{PROJECT}}` placeholders inside README/file content + work
prompt instructions to substitute. NOT the larger restructure (READMEs human-only,
`LLM_STATE/project.md`, `knowledge/` relocation), which is deferred and likely
subsumed by the orchestrator design entirely. The stop-gap establishes a convention
(`{{PROJECT}}`-prefixed references in LLM-Read files) that the new Mnemosyne plans
also follow, so the convention is consistent across both the legacy LLM_CONTEXT
projects and the new orchestrator bootstrap plans.

## Sub-projects

The merge breaks into eight sub-projects, identified during the originating
brainstorm. Each is brainstormed in its own work session of this plan, producing
a design doc at `{{PROJECT}}/docs/superpowers/specs/` and a sibling LLM_CONTEXT
plan at `{{PROJECT}}/LLM_STATE/` containing the implementation backlog.

| ID | Sub-project | Approximate complexity | Notes |
|----|-------------|------------------------|-------|
| A  | Move global knowledge store from `~/.mnemosyne/` to DEV_ROOT | Small-medium | Includes locking model for concurrent access; tied to D |
| B  | Reimplement phase cycle in Rust inside Mnemosyne | Medium | Replaces `run-backlog-plan.sh` |
| C  | Harness adapter layer (Claude Code, Codex, Pi, others) | Medium-large | v1 may ship Claude Code only |
| D  | Multi-instance concurrency model with shared-store locking | Small-medium | Reduced from "multi-plan TUI" — TUI multiplexer cut from v1 |
| E  | Post-session knowledge ingestion model (parent reads child's outputs) | Medium | Conceptual core of the inversion |
| F  | Plan hierarchy + permanent root plan in Mnemosyne's data model | Medium | Provisional structural choice already made |
| G  | Migration strategy: existing LLM_CONTEXT users + Mnemosyne v0.1.0 users transition smoothly | Medium | Parallel and ongoing |
| H  | Fold the 7 Mnemosyne Claude Code skills into Mnemosyne's internal cycle phases / commands | Small-medium | Mostly mechanical; depends on B |

### Recommended sub-project ordering
**E → B → C → A → F → D → H, with G running in parallel.**

- E first: it's the conceptual core; everything downstream depends on knowing how
  Mnemosyne observes and absorbs a session's output.
- B and C unblock the orchestration story (phase cycle + harness adapter).
- A is independent enough to slip in early; tied to D's locking model.
- F continues the plan hierarchy thread; benefits from B being done.
- D is small under the new framing; can run early or alongside A.
- H is mechanical and follows from B.
- G runs parallel throughout — migration plans need to evolve as the design evolves.

Ordering may shift as brainstorms reveal new dependencies. The triage phase is the
right place to revisit it.

## Open questions

These are not blocking the bootstrap but need answers during the relevant sub-project
brainstorms.

### Specific DEV_ROOT subpath for the global knowledge store
Sub-project A. Candidates discussed informally include `DEV_ROOT/Mnemosyne-knowledge/`,
`DEV_ROOT/.mnemosyne-knowledge/`, or a subdirectory inside the Mnemosyne project
itself. Trade-offs: visibility, naming convention, separation from Mnemosyne tool repo.

### Whether v1 supports one harness or multiple from the start
Sub-project C. Claude Code is the user's primary harness today, so a v1 that ships
with only the Claude Code adapter is viable. Codex and Pi adapters can follow.

### Locking primitive for the shared knowledge store
Sub-project A or D. Candidates: file locks via `flock`, a `.lock` sentinel file,
SQLite-backed index with native locking, or a Rust crate's lock primitive. Granularity
question: whole store vs. per-axis vs. per-entry.

### Fate of the 7 existing Mnemosyne Claude Code skills
Sub-project H. Options: fully replaced by orchestrator phases, kept as legacy plugin
during transition, or promoted to first-class CLI subcommands. Likely a mix.

### TheExperimentalist repo fate
Separate decision (backlog item). Delete, archive read-only, or repurpose. Not
blocking any other work.

### What in `LLM_CONTEXT/` survives the merge
Sub-project G. `coding-style.md` and `coding-style-rust.md` are referenced by current
APIAnyware-MacOS work prompts. They could move into Mnemosyne, into a separate
docs repo, or stay in LLM_CONTEXT until LLM_CONTEXT itself retires.

## Constraints / non-goals

### Non-disruption is mandatory
The existing `{{DEV_ROOT}}/LLM_CONTEXT/` machinery and the four projects depending on
it (APIAnyware-MacOS, GUIVisionVMDriver, Modaliser-Racket, RacketPro) must keep
working unchanged throughout this build. The Mnemosyne orchestrator is built
alongside, not in place of, the existing system. Only when the orchestrator is
demonstrably equivalent or better do dependent projects migrate.

### Bootstrap discipline
This seed plan and all sub-project brainstorms run on the existing LLM_CONTEXT
machinery. No tool features may be assumed that LLM_CONTEXT does not already
support. This forces honest dogfooding — every limitation we hit while running
this plan is evidence about what v2 needs to fix.

### Mnemosyne v0.1.0 keeps working
The existing Mnemosyne CLI, knowledge format, evaluation framework, and Claude Code
plugin all continue to function during the build. The Claude Code plugin is the
legacy integration path until the new orchestrator subsumes it. Items currently
listed in `{{PROJECT}}/TODO.md` (horizon scanning, evaluation phase 3/4, etc.) are
the legacy work pipeline; do not advance them as part of this plan unless they
directly enable orchestrator work.

### No TUI multiplexer in v1
Explicitly out of scope. Multi-plan work uses multiple Mnemosyne instances +
locking + user-side multiplexing (tmux, terminal tabs).

## Origin

This plan was created on 2026-04-12 in a single multi-message brainstorming session
that began as a fix for four LLM_CONTEXT punch-list issues, escalated to a
project-wide LLM-content reorganization, then pivoted twice: first to recognising
that Mnemosyne already implements most of the knowledge layer being designed, then
to the much larger architectural inversion captured here. The brainstorm used the
`superpowers:brainstorming` skill but deviated from its terminal step (writing-plans
skill) to instead produce this LLM_CONTEXT-format plan, per explicit user direction
to bootstrap with what works.
