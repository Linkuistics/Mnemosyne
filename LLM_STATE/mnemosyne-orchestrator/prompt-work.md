# Work Phase — Mnemosyne Orchestrator (plan-specific)

## About this plan

This plan exists to drive the merge of LLM_CONTEXT functionality into Mnemosyne,
transforming Mnemosyne into a harness-independent LLM orchestrator. The
architectural state is fully captured in `{{PLAN}}/memory.md` — read it
carefully before doing any task. The most important constraints to keep
in mind during work:

- **Non-disruption**: the existing `{{DEV_ROOT}}/LLM_CONTEXT/` machinery
  and the four projects depending on it (APIAnyware-MacOS,
  GUIVisionVMDriver, Modaliser-Racket, RacketPro) must keep working
  unchanged throughout this build. LLM_CONTEXT now ships a four-phase
  cycle (work → reflect → compact → triage), phase-file-factored layout
  under `phases/`, `fixed-memory/memory-style.md`, a `pre-work.sh`
  opt-in executable hook, and `run-plan.sh` as the driver — changes
  landed upstream that Mnemosyne v1 must eventually absorb.
- **Bootstrap discipline**: this seed plan and all sub-project plans
  run on the existing LLM_CONTEXT machinery. Do not assume features
  Mnemosyne does not yet have; do not modify the existing LLM_CONTEXT
  system except as the punch-list stop-gap task explicitly directs.
- There is also an existing Mnemosyne TODO at `{{PROJECT}}/TODO.md`
  tracking pre-orchestrator work (horizon scanning, evaluation phase
  3/4, etc.). Treat that as the legacy work pipeline that will
  eventually be subsumed by the orchestrator merge — do not progress
  those items as part of this plan unless they directly enable
  orchestrator work.

## Task types

Backlog tasks fall into three categories. Handle each as follows.

### Brainstorm tasks (most sub-project tasks)

Invoke the `superpowers:brainstorming` skill. Drive the brainstorm to a
design that resolves the questions listed in the task description.
Produce a design doc at
`{{PROJECT}}/docs/superpowers/specs/YYYY-MM-DD-<sub-project-name>-design.md`
following the brainstorming skill's spec format. Then create a sibling
plan at `{{PROJECT}}/LLM_STATE/<sub-project-name>/` with the standard
plan files per `{{DEV_ROOT}}/LLM_CONTEXT/create-plan.md` — at minimum
`backlog.md`, `memory.md`, `session-log.md` (header only), and
`phase.md` (containing `work`). Add `related-plans.md` and
`prompt-work.md` only if needed; `prompt-reflect.md`,
`prompt-compact.md`, and `prompt-triage.md` are almost always absent.
**Do NOT create `latest-session.md`** — it is written by the work phase
per cycle and deleted by `run-plan.sh` before each work phase starts.
Populate `backlog.md` from the design doc.

### Decision tasks

Discuss the decision with the user, capture the chosen direction in
`{{PLAN}}/memory.md` (extending the appropriate section), and record
what was decided in `{{PLAN}}/backlog.md` task results. No code or
sub-plans produced; the artifact is the updated memory.

### Stop-gap / do tasks

Execute the work directly in the relevant repos under `{{DEV_ROOT}}/`.
Cd into each repo as needed. Per-repo commits and pushes are required,
following each repo's own commit conventions (check recent `git log` in
each).
