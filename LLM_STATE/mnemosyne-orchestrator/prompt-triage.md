# Triage Phase — Mnemosyne Orchestrator

Read the following:

1. `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` for the triage phase
   specification.
2. `{{PLAN}}/backlog.md` for the current task list.
3. `{{PLAN}}/memory.md` for the architectural state, including any new
   decisions or open questions captured during the most recent reflect.

## Your task

Review and adjust the backlog in light of memory.md changes and any follow-up
work that emerged from the most recent work session.

For each task in the backlog:

- **Still relevant?** If a task has been superseded by a memory.md update or
  by another task's completion, remove it (or mark it `obsolete` with a brief
  note).
- **Priority changed?** Move tasks up or down in file order to reflect current
  priority. The work phase picks from the top of the backlog by default, so
  ordering matters.
- **Dependencies still right?** If a brainstorm revealed that one sub-project
  blocks more downstream work than originally estimated, update the
  `Dependencies` field on the affected tasks.
- **Needs splitting?** If a task has grown large enough that it would consume
  more than one work session, split it into smaller tasks rather than letting
  it sprawl.
- **Marked done?** Tasks completed in the most recent work session should have
  their `Status` updated to `done` (the work phase should have done this
  already, but verify).

Then, add new tasks for any follow-ups noted in `{{PLAN}}/memory.md` that
haven't yet made it into the backlog. The most common case: a brainstorm
session created a sibling LLM_CONTEXT plan, and that plan's implementation work
needs to be tracked here at a high level (or, if better, the sibling plan
tracks it directly and this plan just notes the dependency).

If learnings from the recent session affect *sibling plans* in
`{{PROJECT}}/LLM_STATE/` (sub-project plans created by previous work sessions),
add backlog entries to those plans rather than duplicating the same task here.
This avoids cross-plan duplication.

Do NOT modify `{{PLAN}}/memory.md` in this phase — that is the reflect phase's
job.

When done, write `work` to `{{PLAN}}/phase.md`. Stop.
