# Triage Phase — Sub-project M: Observability Framework

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
The generic triage rule forbids reading foreign plan content directly;
this step uses **dispatched subagents** instead of direct reads.

For each of the queued sibling-adoption entries in this plan's
`memory.md` "Cross-plan adoption coordination" table that is still
marked as `not_landed`:

1. Check whether the corresponding sibling plan now exists (its
   directory is listed in the `{{RELATED_PLANS}}` block above as a
   sibling). If it does not exist yet, skip — the brainstorm has not
   run.
2. If the sibling plan exists, **dispatch a subagent** (general-purpose)
   with a self-contained brief:
   > Plan at `<sibling-path>`. Sub-project M (Observability Framework)
   > needs to land an adoption task in your backlog. The task content
   > is: `<copy the stub from this plan's memory.md adoption table>`.
   > Read the target's `backlog.md` to check whether an equivalent
   > task already exists. If not, add the task using the Edit tool.
   > Do NOT commit. Return a one-line summary of what you did.
3. Collect the subagent's return value. If the task landed, update
   this plan's `memory.md` entry for that sibling from `not_landed` to
   `landed`.

The queued sibling-adoption set is: sub-D (`sub-D-concurrency`),
sub-F (`sub-F-hierarchy`), sub-H (`sub-H-skills`),
sub-I (`sub-I-obsidian-coverage`), sub-G (`sub-G-migration`). If a
brainstormed sub-project lands at a different path than these defaults,
follow the actual path from the `{{RELATED_PLANS}}` block and update
`memory.md` to match.

This coordination step is M's own deliverable, NOT a generic triage
responsibility. It is permitted to modify this plan's `memory.md`
adoption table because the table is operational coordination state,
not distilled learnings — this is the one documented exception to the
"triage does not modify `memory.md`" rule.

## Routing other cross-plan learnings

If learnings from the recent session affect **sibling plans** other
than the queued-adoption set above (e.g., a discovery about
`tracing-subscriber` Layer composition that affects how B's TUI module
should consume the `TuiBridgeLayer`), dispatch a subagent to that
sibling plan per the standard triage cross-plan propagation rules
rather than duplicating the same task here.
