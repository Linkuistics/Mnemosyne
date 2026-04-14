# Triage Phase — Sub-project M: Observability Framework

## Plan-specific triage inputs

In addition to the shared triage reads, read the parent orchestrator
plan's backlog at
`/Users/antony/Development/Mnemosyne/LLM_STATE/mnemosyne-orchestrator/backlog.md`
to check whether any of the queued sibling adoption tasks
(sub-D / sub-F / sub-H / sub-I / sub-G) have transitioned from
`not_started` to `done` since the last triage.

## Plan-specific dependency attention

Pay particular attention to the cross-sub-project dependencies on
sub-B / sub-C / sub-E adoption-task progress. Tasks 13, 15, and 19 in
this plan depend on sibling-plan call sites being landed first. When
the C parallel-emit verification window (Task 15) closes successfully,
unblock the v1.1 cleanup task that deletes C's tactical
`SpawnLatencyReport` writer.

## Cross-plan adoption coordination (M-specific triage step)

This is a step unique to sub-project M's triage, documented in §10 of
the design doc as the mechanism by which M owns cross-plan adoption.

Read the parent orchestrator backlog and check whether any of the
queued sibling adoption tasks have unblocked:

- If `Brainstorm sub-project D` is now `done` (sub-D's sibling plan
  exists), add an "Adopt sub-M observability framework" task to
  `/Users/antony/Development/Mnemosyne/LLM_STATE/sub-D-concurrency/backlog.md`
  with the content documented in this plan's `memory.md` "Cross-plan
  adoption coordination" table. Then update this plan's `memory.md`
  entry to mark D's adoption as `landed`.
- Same logic for sub-F (`sub-F-hierarchy`), sub-H (`sub-H-skills`),
  sub-I (`sub-I-obsidian-coverage`), and sub-G (`sub-G-migration`).
- The sibling plan paths above use the names recorded in this plan's
  `memory.md` table. If a brainstormed sub-project lands at a different
  path, follow the actual path and update `memory.md` to match.

This coordination step is M's own deliverable, NOT a generic triage
responsibility.

If learnings from the recent session affect **sibling plans** other
than the queued-adoption set above (e.g., a discovery about
`tracing-subscriber` Layer composition that affects how B's TUI module
should consume the `TuiBridgeLayer`), add backlog entries to those
plans rather than duplicating the same task here.

**`memory.md` exception.** The cross-plan adoption coordination above
DOES update `memory.md`'s coordination table — that is the one
exception to the "triage does not modify `memory.md`" rule, and it is
permitted because the table is operational state, not distilled
learnings.
