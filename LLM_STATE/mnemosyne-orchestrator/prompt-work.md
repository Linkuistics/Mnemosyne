# Work Phase — Mnemosyne Orchestrator (plan-specific)

Read the following for coding conventions:

- `/Users/antony/Development/LLM_CONTEXT/fixed-memory/coding-style.md`
- `/Users/antony/Development/LLM_CONTEXT/fixed-memory/coding-style-rust.md`

## About this plan

This plan exists to drive the merge of LLM_CONTEXT functionality into Mnemosyne,
transforming Mnemosyne into a harness-independent LLM orchestrator. The
architectural state is fully captured in
`/Users/antony/Development/Mnemosyne/LLM_STATE/mnemosyne-orchestrator/memory.md` —
read it carefully before doing any task. The most important constraints to keep
in mind during work:

- **Non-disruption**: the existing `/Users/antony/Development/LLM_CONTEXT/`
  machinery and the four projects depending on it (APIAnyware-MacOS,
  GUIVisionVMDriver, Modaliser-Racket, RacketPro) must keep working unchanged
  throughout this build.
- **Bootstrap discipline**: this seed plan and all sub-project plans run on the
  existing LLM_CONTEXT machinery. Do not assume features Mnemosyne does not yet
  have; do not modify the existing LLM_CONTEXT system except as the punch-list
  stop-gap task explicitly directs.
- There is also an existing Mnemosyne TODO at
  `/Users/antony/Development/Mnemosyne/TODO.md` tracking pre-orchestrator work
  (horizon scanning, evaluation phase 3/4, etc.). Treat that as the legacy work
  pipeline that will eventually be subsumed by the orchestrator merge — do not
  progress those items as part of this plan unless they directly enable
  orchestrator work.

## Task types

Backlog tasks fall into three categories. Handle each as follows.

### Brainstorm tasks (most sub-project tasks)

Invoke the `superpowers:brainstorming` skill. Drive the brainstorm to a design
that resolves the questions listed in the task description. Produce a design
doc at
`/Users/antony/Development/Mnemosyne/docs/superpowers/specs/YYYY-MM-DD-<sub-project-name>-design.md`
following the brainstorming skill's spec format. Then create a sibling plan at
`/Users/antony/Development/Mnemosyne/LLM_STATE/<sub-project-name>/` with the
standard plan files (`phase.md`, `backlog.md`, `memory.md`, `session-log.md`,
`latest-session.md`) and, if plan-specific guidance is needed, a thin
`prompt-work.md`. Populate `backlog.md` from the design doc.

### Decision tasks

Discuss the decision with the user, capture the chosen direction in
`/Users/antony/Development/Mnemosyne/LLM_STATE/mnemosyne-orchestrator/memory.md`
(extending the appropriate section), and record what was decided in
`/Users/antony/Development/Mnemosyne/LLM_STATE/mnemosyne-orchestrator/backlog.md`
task results. No code or sub-plans produced; the artifact is the updated memory.

### Stop-gap / do tasks

Currently only the LLM_CONTEXT punch-list task. Execute the work directly in
the relevant repos under `/Users/antony/Development/`. Cd into each repo as
needed. Per-repo commits and pushes are required, following each repo's own
commit conventions (check recent `git log` in each).
