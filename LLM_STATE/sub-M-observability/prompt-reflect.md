# Reflect Phase — Sub-project M: Observability Framework

Read the following:

1. `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` for the reflect phase
   specification.
2. The latest entry in `{{PLAN}}/session-log.md`.
3. `{{PLAN}}/memory.md` for the existing implementation state of this plan.

## Your task

Distill learnings from the most recent work session and update
`{{PLAN}}/memory.md`.

This plan's memory file is structured around an authoritative design doc
(the sub-project M spec) plus implementation-level notes. The spec is the
source of truth for design decisions; memory.md holds only implementation
discoveries, cross-sub-project dependency state, and open questions
surfaced during the build.

For each learning surfaced in the session log, ask:

- **Is it a design-level contradiction with the spec?** Do NOT silently
  absorb it into memory. Flag it to the user and escalate to the parent
  plan (`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`) so the
  spec can be updated deliberately. The design is stable; contradictions
  require deliberate resolution, not quiet drift. This is especially
  important for the hybrid `tracing` + typed events architecture, the
  five-crate stack, and the always-on instrumentation principle — those
  are cross-cutting commitments and any drift must be explicit.
- **Is it a discovery about one of the standard crates' actual
  behaviour?** (e.g., `tracing-appender` non-blocking guard semantics
  under sustained load, `metrics-util::Snapshotter` percentile accuracy,
  the exact shape of `tracing::Visit` that `MnemosyneEventLayer`
  implements.) Add it to the "Verified surface" section of memory.md
  with the pinned crate version and the source command / test / benchmark.
- **Is it a new implementation decision not covered by the spec?** Add
  it to `{{PLAN}}/memory.md` under an appropriate heading, with enough
  context to understand why the decision was made.
- **Does it sharpen an existing implementation note?** Update the
  existing note with more precision. Do not duplicate.
- **Does it resolve an open question listed in memory.md?** Remove it
  from the open-questions section and record the decision in the
  appropriate section.
- **Does it surface a new open question?** Add it to "Open questions."
- **Does it reveal a cross-sub-project dependency issue?** Update the
  "Cross-plan adoption coordination" section and, if warranted, add a
  cross-plan backlog entry to the affected sibling plan or to the
  parent orchestrator plan.
- **Does it materialise one of the risks in the "Risk watch list"
  section?** Update the relevant risk entry with what was observed and
  what mitigation was taken. Particular attention to Risk 1
  (`tracing-subscriber` Layer ordering / re-entrancy bugs) and Risk 4
  (`MnemosyneEvent` god-object) — both have v1 implications if they
  materialise.
- **Does it make an existing memory note redundant or obsolete?** Remove
  the redundant note. Memory is not append-only — it should reflect what
  is currently believed to be true and useful. The session log is the
  historical record.

Do NOT modify `{{PLAN}}/backlog.md` in this phase — that is the triage
phase's job.

When done, write `triage` to `{{PLAN}}/phase.md`. Stop.
