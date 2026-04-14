# Reflect Phase — Sub-project M: Observability Framework

Plan-specific reflect guidance beyond the shared phase file:

- **Design-level contradictions require escalation, not absorption.**
  The hybrid `tracing` + typed events architecture, the five-crate
  stack, and the always-on instrumentation principle are cross-cutting
  commitments. If a session surfaces a contradiction with any of these,
  do NOT silently absorb it into memory — flag it and escalate to the
  parent plan
  (`/Users/antony/Development/Mnemosyne/LLM_STATE/mnemosyne-orchestrator/memory.md`)
  so the spec can be updated deliberately. The design is stable;
  contradictions require deliberate resolution, not quiet drift.

- **Discoveries about standard-crate actual behaviour** (e.g.,
  `tracing-appender` non-blocking guard semantics under sustained load,
  `metrics-util::Snapshotter` percentile accuracy, the exact shape of
  `tracing::Visit` that `MnemosyneEventLayer` implements) should land
  in a "Verified surface" section of `memory.md` with the pinned crate
  version and the source command / test / benchmark that produced the
  observation.

- **Risk watch list.** Pay particular attention to Risk 1
  (`tracing-subscriber` Layer ordering / re-entrancy bugs) and Risk 4
  (`MnemosyneEvent` god-object) — both have v1 implications if they
  materialise. If a session observes movement on either, update the
  relevant risk entry in `memory.md` with what was observed and what
  mitigation was taken.

- **Cross-plan dependency state.** If a session reveals that a
  sibling-plan adoption task has landed, or that a cross-sub-project
  dependency has shifted, update the "Cross-plan adoption coordination"
  section of `memory.md`. (Structural updates to that table happen in
  triage — reflect only records the observation.)
