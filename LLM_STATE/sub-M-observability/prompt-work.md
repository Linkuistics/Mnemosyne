# Work Phase — Sub-project M: Observability Framework

Read `/Users/antony/Development/LLM_CONTEXT/fixed-memory/coding-style.md`
and `/Users/antony/Development/LLM_CONTEXT/fixed-memory/coding-style-rust.md`
for coding conventions.

Read
`/Users/antony/Development/Mnemosyne/docs/superpowers/specs/2026-04-13-sub-M-observability-design.md`
— the authoritative design document for this sub-project. Every backlog
task derives from this spec. Consult it before starting any task.

## About this plan

This plan implements sub-project M of the Mnemosyne orchestrator merge —
the observability framework (`tracing` + typed `MnemosyneEvent` enum +
five composed `tracing-subscriber::Layer`s) that gives every other
sub-project a unified diagnostic / live-display / long-term-analysis
substrate.

The design is fully specified in the design doc above; this plan is the
implementation work, not a design phase. Do not re-litigate design
decisions here; surface them to the parent plan
(`/Users/antony/Development/Mnemosyne/LLM_STATE/mnemosyne-orchestrator/memory.md`)
if you discover a spec-level issue during implementation.

### Sub-project M is cross-cutting

Every other sub-project's structured-logging needs route through M once
it lands. M's brainstorm has already landed adoption tasks in
`sub-B-phase-cycle`, `sub-C-adapters`, and `sub-E-ingestion` sibling
backlogs. Adoption tasks for sub-D / sub-F / sub-H / sub-I / sub-G are
queued in this plan's `memory.md` and land in those sibling backlogs as
those sub-projects' brainstorms complete (handled by this plan's triage
phase on each cycle).

## Key constraints

- **Hard errors by default.** Layer registration failures, schema
  mismatches, disk write failures all fail loud. The only tolerated
  failure mode is bounded-queue overflow on the TUI bridge and the
  `tracing-appender` non-blocking writer queue — both increment the
  `events.dropped` counter. Every other path fails hard.
- **TDD.** Every Layer has unit tests written first. Layer 3 integration
  tests cover the full Registry composition end-to-end. Property tests
  cover JSONL round-trip across all `MnemosyneEvent` variants.
- **Five standard crates, no custom replacements.** Use `tracing`,
  `tracing-subscriber`, `tracing-appender`, `metrics`, `metrics-util`
  exactly as documented. The only piece of custom code is one
  ~200-line `MnemosyneEventLayer`. Do not introduce a custom event bus,
  custom subscriber framework, or custom rotation logic.
- **Always-on instrumentation.** Every event emit is unconditional. No
  debug flags, no env-var gates, no build-time toggles. The
  `MNEMOSYNE_LOG` env var only controls stderr formatting visibility,
  not whether events are emitted.
- **Type discipline at the Mnemosyne boundary, ecosystem leverage
  everywhere else.** The `MnemosyneEvent` enum is the single source of
  truth for "what events Mnemosyne knows how to observe." Downstream
  consumers exhaustively pattern-match on it. Below the boundary,
  `tracing`'s field/span machinery handles transport.
- **Vault-scoped persistence integrates with B's `<vault>/runtime/`
  subtree.** B owns staging / interrupts / ingestion events / locks; M
  owns events / metrics / observability summaries. Both live under the
  same `<vault>/runtime/` root.

## Commands

Build / test / lint commands live in
`/Users/antony/Development/Mnemosyne/README.md`. For Rust work, use
`cargo test`, `cargo clippy`, and `cargo +nightly fmt` per
`/Users/antony/Development/LLM_CONTEXT/fixed-memory/coding-style-rust.md`.

## Dependencies on sibling sub-projects

Sub-project M is largely self-contained at the framework level — it
defines types, layers, and a startup function. The cross-sub-project
work (instrumenting B's `PhaseRunner`, C's actor, E's pipeline) is
landed in those sibling plans as adoption tasks rather than in this
plan. Two consequences:

1. M's own backlog can mostly run independently of sub-B / sub-C /
   sub-E implementation progress.
2. Tasks 13 (Risk 5 dump wiring), 15 (C parallel-emit verification),
   and 19 (fixture-replay end-to-end test) DO depend on sibling-plan
   progress — they exercise call sites that live in those plans. If a
   prerequisite sibling task is not yet `done`, defer the M task and
   pick a different one.
