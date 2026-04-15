# Work Phase — Sub-project F: Plan Hierarchy, Actor Model, Dispatch, Declarative Routing

Read for this plan:

- `{{PROJECT}}/docs/superpowers/specs/2026-04-14-sub-F-hierarchy-design.md`
  — the authoritative design document for this sub-project. Every task
  in the backlog derives from this spec. Consult it (at least §1–§10
  plus §11 for the task list) before starting any task.

## About this plan

This plan implements sub-project F of the Mnemosyne orchestrator merge —
the persistent BEAM daemon, two sealed actor types (PlanActor,
ExpertActor), two message types (Dispatch, Query), the
`project-root` plan hierarchy convention, path-based qualified plan
IDs, declarative routing with pattern-matched Elixir + Level 2
LLM fallback, the client-daemon NDJSON protocol, and the vault
catalog that replaces the old `related-plans.md` file concept.

The design is fully specified in the design doc referenced above;
this plan is the implementation work, not a design phase. Do not
re-litigate design decisions here; surface them to the parent plan
(`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`) if you
discover a spec-level issue during implementation.

## Key constraints

- **Task 0 is the implementation readiness gate** and is a hard
  pre-implementation blocker. Until sub-B's and sub-C's downstream
  task lists have been rewritten against their Session-11 / Session-12
  design-doc rewrites, and until sub-A's and sub-M's amendments have
  been absorbed, no Task 1+ work starts. Re-read F's §11 against the
  latest versions of the A/B/C/M design docs as part of the gate —
  any consumed interface mismatch rewrites F's design doc inline
  before implementation.
- **Amendment tasks rewrite specs inline, not as supersede layers.**
  If a downstream pivot invalidates parts of F's design doc during
  implementation, rewrite the affected sections inline and record the
  correction as a new Decision Trail entry in Appendix A. Do not
  append supersede blocks.
- **Hard errors by default**: invariant violations, parse errors,
  I/O failures, and ambiguous state all fail hard with clear
  diagnostics naming the offending file and line. Soft fallbacks
  require explicit written rationale in the design doc. Never
  silently log-and-continue.
- **Filesystem is the durable substrate**: daemon restart rebuilds
  all actor state from files. Do not hold in-memory state that the
  filesystem cannot reproduce. Individual actor crashes never
  destroy state.
- **Path-based qualified IDs are never stored**: F-5 is load-bearing
  for the "filesystem is authoritative" discipline. No frontmatter
  field caches the qualified ID.
- **OTP actor mailboxes serialize writes** — F's daemon commitment
  collapses sub-D's per-plan advisory-lock scope. Do not reintroduce
  per-plan locking.
- **Obsidian-native formats**: every file Mnemosyne writes uses
  markdown with YAML frontmatter, kebab-case property names for
  Dataview, wikilinks for cross-references, tags as first-class
  metadata.
- **TDD**: every invariant, every dispatch/query edge case in §9, and
  every routing clause is unit-tested first. Tests drive
  implementation. No code ships without its tests.

## Commands

Elixir work uses `mix test` (ExUnit), `mix format`, `mix credo`
(if configured), and `mix dialyzer`. Run the full test suite plus
format check before declaring any task done — no task ships with
failing tests, formatting diffs, or new Dialyzer warnings. Live
tests that spawn real `claude` sessions are tagged `@moduletag
:live` and gated behind `mix test --include live`. See
`{{PROJECT}}/README.md` for Mnemosyne's full build/test surface.

## Dependencies on sibling sub-projects

Sub-project F depends on A (vault discovery / `verify_vault`), B
(`PhaseRunner` as `PlanActor` internal state), C (harness adapter
for fact-extraction and Level 2 agent), E (indirectly — Stage 5
dispatch-to-experts integration hits F's router), and M
(`:telemetry` + typed `Mnemosyne.Event.*` struct pattern).
Integration tasks (22–24) land only after the depended-on siblings
have real implementations — until then, implement against the
module-level contracts in the respective sibling design docs and
leave integration-task-level cross-wiring for the end.

## BEAM PTY spike

The spike that unblocked this plan's scaffolding lives at
`{{PROJECT}}/spikes/beam_pty/`. It validated pipes-only
`erlexec` (no PTY needed, no Rust wrapper fallback). The spike's
README records the specific erlexec options required
(`[:monitor, :stdin, {:stdout, self()}, {:stderr, self()},
:kill_group, {:kill_timeout, 1}]`) plus the cmux-noise mitigation
flags (`--setting-sources project,local
--no-session-persistence`). Consult the spike README before
any Task 15 / Task 18 work that spawns a harness session.
