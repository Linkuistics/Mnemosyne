# Triage Phase — Sub-project C: Harness Adapter Layer

## Plan-specific triage rules

In addition to the generic triage discipline, apply these plan-specific
rules when reviewing the backlog:

- **Cross-sub-project dependencies.** Pay particular attention to B's
  executor eventually consuming the trait + adapter — the C-1 dogfood
  gate is the swap-in moment. Update `Dependencies` fields accordingly
  when implementation reveals new downstream blockers.
- **C-1 dogfood gate triggers warm-pool follow-ups.** If the C-1
  dogfood acceptance gate trips, unblock the §7.4 warm-pool reset
  spike task and the §7.5 v1.5 warm-pool implementation task.
- **Risk watch list materialisation.** If any risk in the memory.md
  "Risk watch list" section materialises during a work session, add
  concrete mitigation tasks as follow-ups.
- **Routing cross-plan learnings.** If learnings from the recent
  session affect **sibling plans** (other sub-projects in
  `{{PROJECT}}/LLM_STATE/`), add backlog entries to those plans rather
  than duplicating here. In particular: any new amendment to B's trait
  or executor surface that emerges from implementation goes into B's
  plan, not this one. If they affect the **parent orchestrator plan**
  (`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/`), add them there.
