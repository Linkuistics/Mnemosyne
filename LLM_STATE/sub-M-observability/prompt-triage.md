# Triage Phase — Sub-project M: Observability Framework

Read the following:

1. `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` for the triage phase
   specification.
2. `{{PLAN}}/backlog.md` for the current implementation task list.
3. `{{PLAN}}/memory.md` for the implementation state, including any new
   decisions or open questions captured during the most recent reflect.
4. The parent orchestrator plan's `backlog.md` at
   `{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/backlog.md` — to check
   whether any of the queued sibling adoption tasks (sub-D / sub-F /
   sub-H / sub-I / sub-G) have transitioned from `not_started` to `done`
   since the last triage.

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
  `Dependencies` fields accordingly. Pay particular attention to the
  cross-sub-project dependencies on sub-B / sub-C / sub-E adoption-task
  progress (Tasks 13, 15, and 19 in this plan depend on sibling-plan
  call sites being landed first).
- **Needs splitting?** If a task has grown large enough that it would
  consume more than one work session, split it into smaller tasks rather
  than letting it sprawl.
- **Marked done?** Tasks completed in the most recent work session should
  have their `Status` updated to `done` (the work phase should have done
  this already, but verify).

Then, add new tasks for any follow-ups noted in `{{PLAN}}/memory.md` that
haven't yet made it into the backlog. Common cases:

- Implementation surprises that require follow-up work (e.g., a
  `tracing-subscriber` Layer ordering issue that needs a separate
  mitigation task beyond the re-entrancy stress test).
- New open questions that warrant their own investigation task.
- Risks in the "Risk watch list" section materialising — add mitigation
  tasks as concrete follow-ups.
- The C parallel-emit verification window (Task 15) closing successfully
  → unblock the v1.1 cleanup task that deletes C's tactical
  `SpawnLatencyReport` writer.

## Cross-plan adoption coordination (M-specific triage step)

This is a step unique to sub-project M's triage. Read the parent
orchestrator backlog and check whether any of the queued sibling
adoption tasks have unblocked:

- If `Brainstorm sub-project D` is now `done` (sub-D's sibling plan
  exists), add an "Adopt sub-M observability framework" task to
  `{{PROJECT}}/LLM_STATE/sub-D-concurrency/backlog.md` with the content
  documented in `{{PLAN}}/memory.md`'s "Cross-plan adoption coordination"
  table. Then update this plan's memory.md entry to mark D's adoption
  as `landed`.
- Same logic for sub-F (`sub-F-hierarchy`), sub-H (`sub-H-skills`),
  sub-I (`sub-I-obsidian-coverage`), and sub-G (`sub-G-migration`).
- The sibling plan paths above use the names recorded in this plan's
  memory.md table. If a brainstormed sub-project lands at a different
  path, follow the actual path and update memory.md to match.

This coordination step is M's own deliverable, NOT a generic triage
responsibility, and is documented in §10 of the design doc as the
mechanism by which M owns cross-plan adoption.

If learnings from the recent session affect **sibling plans** other
than the queued-adoption set above (e.g., a discovery about
`tracing-subscriber` Layer composition that affects how B's TUI module
should consume the `TuiBridgeLayer`), add backlog entries to those
plans rather than duplicating the same task here. If they affect the
**parent orchestrator plan**
(`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/`), add them there.

Do NOT modify `{{PLAN}}/memory.md` in this phase — that is the reflect
phase's job. The cross-plan adoption coordination above DOES update
memory.md's coordination table — that is the one exception, and it is
permitted because the table is operational state, not distilled
learnings.

When done, write `work` to `{{PLAN}}/phase.md`. Stop.
