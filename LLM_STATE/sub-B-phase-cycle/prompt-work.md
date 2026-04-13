# Work Phase — Sub-project B: Phase Cycle Reimplementation in Rust

Read the following before doing anything else:

1. `{{PROJECT}}/README.md` for Mnemosyne's project conventions, architecture,
   CLI surface, and v0.1.0 status. Mnemosyne is a Rust project; build and
   test commands are documented there.
2. `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` for the work / reflect / triage
   phase cycle specification.
3. `{{DEV_ROOT}}/LLM_CONTEXT/coding-style.md` and
   `{{DEV_ROOT}}/LLM_CONTEXT/coding-style-rust.md` for coding conventions.
4. `{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md`
   — the authoritative design document for this sub-project. Every task
   below derives from this spec. Consult it before starting any task.
5. `{{PLAN}}/backlog.md` for the current implementation task list.
6. `{{PLAN}}/memory.md` for implementation-level decisions, cross-sub-project
   dependencies, and implementation strategy notes.

## About this plan

This plan implements sub-project B of the Mnemosyne orchestrator merge —
the long-running process model, phase cycle state machine, executor
abstractions, placeholder substitution mechanism, and plan-state file
format that replace `{{DEV_ROOT}}/LLM_CONTEXT/run-backlog-plan.sh`.

The design is fully specified in the design doc referenced above; this
plan is the implementation work, not a design phase. Do not re-litigate
design decisions here; surface them to the parent plan
(`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`) if you discover
a spec-level issue during implementation.

### Key constraints during implementation

- **The Obsidian + symlinks validation spike is the first task** and is a
  hard pre-implementation blocker. Until it passes on both macOS and Linux
  (via GUIVisionVMDriver golden images), no other sub-B task starts. If
  the spike fails, open a brainstorm-mode discussion to resolve the
  fallback layout before resuming.
- **Non-disruption**: existing Mnemosyne v0.1.0 must keep running during
  the build. New code lives alongside, not in place of, existing code.
  The dogfood acceptance test is the final task in this plan's backlog
  and is the moment v1 replaces `run-backlog-plan.sh` for the orchestrator
  seed plan.
- **Type-level co-equal-actors**: every phase execution, LLM or human, must
  flow through the same `PhaseRunner::run_phase` chokepoint. Tests that
  exercise `run_phase` with multiple executors verify this at the type
  level. Do not add execution paths that bypass the chokepoint.
- **Hard errors by default**: illegal states, invariant violations, I/O
  failures, and unexpected conditions all fail hard with clear diagnostics.
  Soft fallbacks require explicit written rationale in the code or the
  design doc. Never silently log-and-continue.
- **Obsidian-native formats**: every file Mnemosyne writes uses markdown
  with YAML frontmatter, kebab-case property names for Dataview, wikilinks
  for cross-references, tags as first-class metadata.
- **TDD**: every type and every invariant in §2.2 and §3 of the spec is
  unit-tested first. Tests drive implementation. No code ships without
  its tests.
- **Test and build commands**: see `{{PROJECT}}/README.md`. For Rust work,
  use `cargo test`, `cargo clippy`, and `cargo +nightly fmt` per
  `{{DEV_ROOT}}/LLM_CONTEXT/coding-style-rust.md`.

### Dependencies on sibling sub-projects

Sub-project B depends on C (adapter), D (locking), E (ingestion hook),
and conceptually on A (vault location) and F (plan hierarchy semantics).
Until those land, use the stubs documented in `{{PLAN}}/memory.md` under
"Dependencies on sibling sub-projects." Do not block on sibling work —
implement against the stubs, then swap to real implementations when the
siblings land.

### GUIVisionVMDriver for the symlinks validation spike

The first task uses GUIVisionVMDriver (at `{{DEV_ROOT}}/GUIVisionVMDriver/`)
to run the Obsidian + symlinks validation on both macOS and Linux VMs.
Consult `{{DEV_ROOT}}/GUIVisionVMDriver/instructions-for-llms-using-this-as-a-tool.md`
for the commands. Capture evidence (screenshots, accessibility snapshots)
and commit them to `tests/fixtures/obsidian-validation/`.

## Path placeholders

Any path beginning with `{{PROJECT}}`, `{{DEV_ROOT}}`, or `{{PLAN}}` should
be interpreted as the absolute path the shell script substitutes before
passing the prompt to the LLM. If you see a literal `{{PROJECT}}`,
`{{DEV_ROOT}}`, or `{{PLAN}}` token in any file you Read inside the dev
root, substitute it mentally with the correct absolute path before passing
it to the Read tool.

## Working a task

1. Display a summary of the current backlog: title, status, and the
   relative priority order (top of the backlog file = highest priority).
2. Ask the user if they have input on which task to work on next. Wait
   for their response. If they have a preference, work on that task;
   otherwise pick the highest-priority `not_started` task whose
   dependencies are all `done`.
3. Work the task using TDD: write the failing test first, implement the
   minimum code to pass, refactor, commit. Consult the design doc for
   any behavioural question.
4. Run the full test suite and clippy before declaring the task done.
   No task ships with failing tests or new warnings.
5. Record results in `{{PLAN}}/backlog.md` — replace `_pending_` with
   a concrete summary of what was built, tests added, and any
   surprises. Update the task `Status` to `done`.
6. Append a session log entry to `{{PLAN}}/session-log.md` per the
   format in `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md`:
   `### Session N (YYYY-MM-DD) — title`, bullets for what was attempted,
   what worked / didn't, what to try next, key learnings.
7. Write `reflect` to `{{PLAN}}/phase.md`.
8. Stop. Do not pick another task. Do not enter the reflect phase
   yourself — the next phase runs in a fresh session.
