# Triage Phase — Sub-project C: Harness Adapter Layer

Read the following:

1. `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` for the triage phase
   specification.
2. `{{PLAN}}/backlog.md` for the current implementation task list.
3. `{{PLAN}}/memory.md` for the implementation state, including any new
   decisions or open questions captured during the most recent reflect.

## Your task

Review and adjust the implementation backlog in light of memory.md changes
and any follow-up work that emerged from the most recent work session.

For each task in the backlog:

- **Still relevant?** If a task has been superseded by a memory.md update,
  by another task's completion, or by a spec clarification, remove it or
  mark it `obsolete` with a brief note.
- **Priority changed?** Move tasks up or down in file order to reflect
  current priority. The work phase picks from the top by default.
- **Dependencies still right?** If implementation has revealed that a task
  blocks more downstream work than originally estimated, update the
  `Dependencies` fields accordingly. Pay particular attention to
  cross-sub-project dependencies (B's executor will eventually consume
  the trait + adapter; the C-1 dogfood gate is the swap-in moment).
- **Needs splitting?** If a task has grown large enough that it would
  consume more than one work session, split it into smaller tasks rather
  than letting it sprawl.
- **Marked done?** Tasks completed in the most recent work session should
  have their `Status` updated to `done` (the work phase should have done
  this already, but verify).

Then, add new tasks for any follow-ups noted in `{{PLAN}}/memory.md` that
haven't yet made it into the backlog. Common cases:

- Implementation surprises that require follow-up work (e.g., a stream-json
  schema field shape was different from the spec draft and the parser
  needs adjustment beyond what the verification task captured).
- New open questions that warrant their own investigation task.
- The C-1 dogfood acceptance gate tripping → unblock the §7.4 warm-pool
  reset spike task and the §7.5 v1.5 warm-pool implementation task.
- Risks in the "Risk watch list" section materialising — add mitigation
  tasks as concrete follow-ups.

If learnings from the recent session affect **sibling plans** (other
sub-projects in `{{PROJECT}}/LLM_STATE/`), add backlog entries to those
plans rather than duplicating the same task here. Particularly: any new
amendment to B's trait or executor surface that emerges from
implementation goes into B's plan, not this one. If they affect the
**parent orchestrator plan**
(`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/`), add them there. This
avoids cross-plan duplication.

Do NOT modify `{{PLAN}}/memory.md` in this phase — that is the reflect
phase's job.

When done, write `work` to `{{PLAN}}/phase.md`. Stop.
