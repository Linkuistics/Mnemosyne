# Work Phase — Sub-project M: Observability Framework

Read the following before doing anything else:

1. `{{PROJECT}}/README.md` for Mnemosyne's project conventions, architecture,
   CLI surface, and v0.1.0 status. Mnemosyne is a Rust project; build and
   test commands are documented there.
2. `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` for the work / reflect / triage
   phase cycle specification.
3. `{{DEV_ROOT}}/LLM_CONTEXT/coding-style.md` and
   `{{DEV_ROOT}}/LLM_CONTEXT/coding-style-rust.md` for coding conventions.
4. `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-M-observability-design.md`
   — the authoritative design document for this sub-project. Every task
   below derives from this spec. Consult it before starting any task.
5. `{{PLAN}}/backlog.md` for the current implementation task list.
6. `{{PLAN}}/memory.md` for implementation-level decisions, cross-sub-project
   dependencies, and implementation strategy notes.

## About this plan

This plan implements sub-project M of the Mnemosyne orchestrator merge —
the observability framework (`tracing` + typed `MnemosyneEvent` enum +
five composed `tracing-subscriber::Layer`s) that gives every other
sub-project a unified diagnostic / live-display / long-term-analysis
substrate.

The design is fully specified in the design doc referenced above; this
plan is the implementation work, not a design phase. Do not re-litigate
design decisions here; surface them to the parent plan
(`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`) if you discover
a spec-level issue during implementation.

### Sub-project M is cross-cutting

Every other sub-project's structured-logging needs route through M once
it lands. M's brainstorm has already landed adoption tasks in
`sub-B-phase-cycle`, `sub-C-adapters`, and `sub-E-ingestion` sibling
backlogs. Adoption tasks for sub-D / sub-F / sub-H / sub-I / sub-G are
queued in this plan's `memory.md` and land in those sibling backlogs as
those sub-projects' brainstorms complete (handled by this plan's triage
phase on each cycle).

### Key constraints during implementation

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
- **Test and build commands**: see `{{PROJECT}}/README.md`. For Rust
  work, use `cargo test`, `cargo clippy`, and `cargo +nightly fmt` per
  `{{DEV_ROOT}}/LLM_CONTEXT/coding-style-rust.md`.

### Dependencies on sibling sub-projects

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

## Path placeholders

Any path beginning with `{{PROJECT}}`, `{{DEV_ROOT}}`, or `{{PLAN}}` should
be interpreted as the absolute path the shell script substitutes before
passing the prompt to the LLM. If you see a literal `{{PROJECT}}`,
`{{DEV_ROOT}}`, or `{{PLAN}}` token in any file you Read inside the dev
root, substitute it mentally with the correct absolute path before passing
it to the Read tool.

## Working a task

1. Display a summary of the current backlog: title, status, and the
   relative priority order (top of the backlog file = highest priority).
2. Ask the user if they have input on which task to work on next. Wait
   for their response. If they have a preference, work on that task;
   otherwise pick the highest-priority `not_started` task whose
   dependencies are all `done`.
3. Work the task using TDD: write the failing test first, implement the
   minimum code to pass, refactor, commit. Consult the design doc for
   any behavioural question.
4. Run the full test suite and clippy before declaring the task done.
   No task ships with failing tests or new warnings.
5. Record results in `{{PLAN}}/backlog.md` — replace `_pending_` with
   a concrete summary of what was built, tests added, and any
   surprises. Update the task `Status` to `done`.
6. Append a session log entry to `{{PLAN}}/session-log.md` per the
   format in `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md`:
   `### Session N (YYYY-MM-DD) — title`, bullets for what was attempted,
   what worked / didn't, what to try next, key learnings.
7. Write `reflect` to `{{PLAN}}/phase.md`.
8. Stop. Do not pick another task. Do not enter the reflect phase
   yourself — the next phase runs in a fresh session.
