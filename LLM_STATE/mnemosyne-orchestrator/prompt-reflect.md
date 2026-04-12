# Reflect Phase — Mnemosyne Orchestrator

Read the following:

1. `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` for the reflect phase
   specification.
2. The latest entry in `{{PLAN}}/session-log.md`.
3. `{{PLAN}}/memory.md` for the existing architectural state of this plan.

## Your task

Distill learnings from the most recent work session and update
`{{PLAN}}/memory.md`. The plan's memory file is structured into sections
(stable architectural decisions, sub-projects, open questions, constraints).
Place new entries in the right section.

For each learning surfaced in the session log, ask:

- **Is it new?** Add it to the appropriate section as a new entry.
- **Does it sharpen an existing entry?** Update the existing entry with more
  precision. Do not duplicate.
- **Does it contradict an existing entry?** Do NOT silently overwrite. Flag the
  contradiction explicitly to the user and discuss before resolving — stable
  decisions are stable for a reason; a contradiction means either the prior
  decision was wrong or the new evidence is incomplete.
- **Does it resolve an open question?** Move the question into the appropriate
  decisions section as a stable entry, and remove it from "Open questions."
- **Does it surface a new open question?** Add it to "Open questions."
- **Does it change the recommended sub-project ordering or dependency
  structure?** Update the sub-projects section accordingly.
- **Does it suggest a sub-project's complexity estimate is wrong?** Update the
  sub-projects table.
- **Does it make an existing entry redundant or obsolete?** Remove the
  redundant entry. Memory is not append-only — it should reflect what's
  currently believed to be true and useful, not the historical record. The
  session log is the historical record.

Do NOT modify `{{PLAN}}/backlog.md` in this phase — that is the triage phase's
job.

When done, write `triage` to `{{PLAN}}/phase.md`. Stop.
