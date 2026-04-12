# Work Phase — Mnemosyne Orchestrator

Read the following before doing anything else:

1. `{{PROJECT}}/README.md` for Mnemosyne's project conventions, architecture,
   CLI surface, and current v0.1.0 status. Mnemosyne is a Rust project; build
   and test commands and the existing knowledge format are documented there.
2. `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` for the work / reflect / triage
   phase cycle specification.
3. `{{DEV_ROOT}}/LLM_CONTEXT/coding-style.md` and
   `{{DEV_ROOT}}/LLM_CONTEXT/coding-style-rust.md` for the project's coding
   conventions.
4. `{{PLAN}}/backlog.md` for the current task list.
5. `{{PLAN}}/memory.md` for the architectural decisions, sub-projects, open
   questions, and constraints already established for this plan.

## About this plan

This plan exists to drive the merge of LLM_CONTEXT functionality into Mnemosyne,
transforming Mnemosyne into a harness-independent LLM orchestrator. The
architectural state is fully captured in `{{PLAN}}/memory.md` — read it
carefully before doing any task. The most important constraints to keep in mind
during work:

- **Non-disruption**: the existing `{{DEV_ROOT}}/LLM_CONTEXT/` machinery and the
  four projects depending on it (APIAnyware-MacOS, GUIVisionVMDriver,
  Modaliser-Racket, RacketPro) must keep working unchanged throughout this
  build.
- **Bootstrap discipline**: this seed plan and all sub-project plans run on the
  existing LLM_CONTEXT machinery. Do not assume features Mnemosyne does not yet
  have; do not modify the existing LLM_CONTEXT system except as the punch-list
  stop-gap task explicitly directs.
- There is also an existing Mnemosyne TODO at `{{PROJECT}}/TODO.md` tracking
  pre-orchestrator work (horizon scanning, evaluation phase 3/4, etc.). Treat
  that as the legacy work pipeline that will eventually be subsumed by the
  orchestrator merge — do not progress those items as part of this plan unless
  they directly enable orchestrator work.

## Task types

Backlog tasks fall into three categories. Handle each as follows.

### Brainstorm tasks (most sub-project tasks)

Invoke the `superpowers:brainstorming` skill. Drive the brainstorm to a design
that resolves the questions listed in the task description. Produce a design
doc at `{{PROJECT}}/docs/superpowers/specs/YYYY-MM-DD-<sub-project-name>-design.md`
following the brainstorming skill's spec format. Then create a sibling
LLM_CONTEXT plan at `{{PROJECT}}/LLM_STATE/<sub-project-name>/` containing the
seven plan files (`phase.md`, `backlog.md`, `memory.md`, `session-log.md`,
`prompt-work.md`, `prompt-reflect.md`, `prompt-triage.md`), with the
implementation backlog populated from the design doc. Follow the conventions
in `{{DEV_ROOT}}/LLM_CONTEXT/create-a-multi-session-plan.md` and
`{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md`.

### Decision tasks

Discuss the decision with the user, capture the chosen direction in
`{{PLAN}}/memory.md` (extending the appropriate section), and record what was
decided in `{{PLAN}}/backlog.md` task results. No code or sub-plans produced;
the artifact is the updated memory.

### Stop-gap / do tasks

Currently only the LLM_CONTEXT punch-list task. Execute the work directly in
the relevant repos under `{{DEV_ROOT}}/`. Cd into each repo as needed. Per-repo
commits and pushes are required, following each repo's own commit conventions
(check recent `git log` in each).

## Path placeholders

Any path in this plan or any plan derived from it that begins with
`{{PROJECT}}` should be interpreted as the absolute path of the Mnemosyne
project root. Any path beginning with `{{DEV_ROOT}}` should be interpreted as
the absolute path of the dev root (parent of all project repos). The
`run-backlog-plan.sh` script substitutes these placeholders before passing the
prompt to the LLM, so by the time you read this, they should already be
absolute paths.

If you ever see a literal `{{PROJECT}}` or `{{DEV_ROOT}}` token in any file you
Read inside the dev root (including READMEs, notes, and sub-plan content),
substitute it mentally with the absolute path supplied above before passing the
path to the Read tool. This convention is the Option C stop-gap from the
LLM_CONTEXT punch list and is used uniformly across all plans created from
this seed plan.

## Working a task

1. Display a summary of the current backlog: title, status, and the relative
   priority order (top of the backlog file = highest priority).
2. Ask the user if they have input on which task to work on next. Wait for
   their response. If they have a preference, work on that task; otherwise pick
   the highest-priority `not_started` task whose dependencies are all
   `done`.
3. Work the task following the appropriate handling for its type (above).
4. Record results in `{{PLAN}}/backlog.md` — replace `_pending_` with a concrete
   summary of what was produced or decided. Update the task `Status` to `done`.
5. Append a session log entry to `{{PLAN}}/session-log.md` per the format in
   `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` (`### Session N (YYYY-MM-DD) — title`,
   bullets for what was attempted, what worked / didn't, what to try next, key
   learnings).
6. Write `reflect` to `{{PLAN}}/phase.md`.
7. Stop. Do not pick another task. Do not enter the reflect phase yourself —
   the next phase runs in a fresh session.
