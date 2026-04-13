# Work Phase — Sub-project E: Post-Session Knowledge Ingestion

Read the following before doing anything else:

1. `{{PROJECT}}/README.md` for Mnemosyne's project conventions, architecture,
   CLI surface, and v0.1.0 status. Mnemosyne is a Rust project; build and
   test commands are documented there.
2. `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` for the work / reflect / triage
   phase cycle specification.
3. `{{DEV_ROOT}}/LLM_CONTEXT/coding-style.md` and
   `{{DEV_ROOT}}/LLM_CONTEXT/coding-style-rust.md` for coding conventions.
4. `{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-E-ingestion-design.md`
   — the authoritative design document for this sub-project. Every task
   below derives from this spec. Consult it before starting any task.
5. `{{PLAN}}/backlog.md` for the current implementation task list.
6. `{{PLAN}}/memory.md` for implementation-level decisions, cross-sub-project
   dependencies, and implementation strategy notes.

## About this plan

This plan implements sub-project E of the Mnemosyne orchestrator merge —
the post-session knowledge ingestion pipeline. The design is fully specified
in the design doc referenced above; this plan is the implementation work,
not a design phase. Do not re-litigate design decisions here; surface them
to the parent plan
(`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`) if you discover
a spec-level issue during implementation.

### Key constraints during implementation

- **Non-disruption**: existing Mnemosyne v0.1.0 must keep running. New code
  lives alongside, not in place of, existing code. Integration with existing
  Mnemosyne primitives is incidental-if-it-fits, not a design constraint.
- **Fresh LLM context is a first-class goal**: the pipeline's stage
  decomposition exists specifically to keep every LLM session minimally
  scoped. Do not collapse stages, do not pass larger contexts than the spec
  requires, and do not add cross-stage state that would defeat the purpose.
- **Human and LLM are co-equal actors**: every Stage 5 write primitive must
  be reachable from UI actions, not only from the pipeline. Tasks tagged
  `[human-mode]` enforce this at the API surface level.
- **TDD**: every Rule in §3 of the spec is unit-tested first. Tests drive
  the Stage 5 invariant implementation. No rule ships without its test.
- **Test and build commands**: see `{{PROJECT}}/README.md`. For Rust work,
  use `cargo test`, `cargo clippy`, and `cargo +nightly fmt` per
  `{{DEV_ROOT}}/LLM_CONTEXT/coding-style-rust.md`.

### Dependencies on sibling sub-projects

Sub-project E depends on A, B, C, and D. Until those land, use the stubs
documented in `{{PLAN}}/memory.md` under "Dependencies on sibling
sub-projects." Do not block on sibling work — implement against the stubs,
then swap to real implementations when the siblings land.

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
2. Ask the user if they have input on which task to work on next. Wait for
   their response. If they have a preference, work on that task; otherwise
   pick the highest-priority `not_started` task whose dependencies are all
   `done`.
3. Work the task using TDD: write the failing test first, implement the
   minimum code to pass, refactor, commit. Consult the design doc for any
   behavioural question.
4. Run the full test suite and clippy before declaring the task done. No
   task ships with failing tests or new warnings.
5. Record results in `{{PLAN}}/backlog.md` — replace `_pending_` with a
   concrete summary of what was built, tests added, and any surprises.
   Update the task `Status` to `done`.
6. Append a session log entry to `{{PLAN}}/session-log.md` per the format
   in `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md`:
   `### Session N (YYYY-MM-DD) — title`, bullets for what was attempted,
   what worked / didn't, what to try next, key learnings.
7. Write `reflect` to `{{PLAN}}/phase.md`.
8. Stop. Do not pick another task. Do not enter the reflect phase yourself
   — the next phase runs in a fresh session.
