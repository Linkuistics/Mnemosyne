# Reflect Phase — Sub-project C: Harness Adapter Layer

## Plan-specific reflect rules

This plan's memory file is structured around an authoritative design doc
(the sub-project C spec at
`{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-C-adapters-design.md`)
plus implementation-level notes. The spec is the source of truth for
design decisions; memory.md holds only implementation discoveries,
cross-sub-project dependency state, and open questions surfaced during
the build.

When distilling learnings, apply these plan-specific rules in addition
to the generic reflect discipline:

- **Design-level contradictions with the spec.** Do NOT silently absorb
  them into memory. Flag them to the user and escalate to the parent
  plan (`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`) so
  the spec can be updated deliberately. This is especially important
  for the five B-trait amendments and the sentinel-detection
  requirement — those are cross-sub-project commitments and any drift
  must be explicit.
- **Claude Code stream-json schema observations.** Add them to the
  "Verified Claude Code CLI surface" section of memory.md with the
  pinned `claude` version and the source command. These observations
  resolve §10 verification IOUs from the spec.
- **Risk watch list.** Pay particular attention to Risk 1 (stream-json
  schema drift) and Risk 2 (cold-spawn latency exceeds 5s p95 gate) —
  both have v1 implications if they materialise. Update the relevant
  risk entry with what was observed and what mitigation was taken.
- **Cross-sub-project dependency issues.** Update the "Dependencies on
  sibling sub-projects" section and, if warranted, add a cross-plan
  backlog entry to the affected sibling plan or to the parent
  orchestrator plan.
