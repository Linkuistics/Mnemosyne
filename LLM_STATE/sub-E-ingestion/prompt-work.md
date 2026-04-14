# Work Phase — Sub-project E: Post-Session Knowledge Ingestion

Read:

- `{{DEV_ROOT}}/LLM_CONTEXT/fixed-memory/coding-style.md`
- `{{DEV_ROOT}}/LLM_CONTEXT/fixed-memory/coding-style-rust.md`
- `{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-E-ingestion-design.md`
  — the authoritative design document for this sub-project. Every task in
  the backlog derives from this spec. Consult it before starting any task.

## About this plan

This plan implements sub-project E of the Mnemosyne orchestrator merge —
the post-session knowledge ingestion pipeline. The design is fully specified
in the design doc referenced above; this plan is the implementation work,
not a design phase. Do not re-litigate design decisions here; surface them
to the parent plan
(`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`) if you discover
a spec-level issue during implementation.

## Constraints

- **Non-disruption**: existing Mnemosyne v0.1.0 must keep running. New code
  lives alongside, not in place of, existing code. Integration with existing
  Mnemosyne primitives is incidental-if-it-fits, not a design constraint.
- **Fresh LLM context is a first-class goal**: the pipeline's stage
  decomposition exists specifically to keep every LLM session minimally
  scoped. Do not collapse stages, do not pass larger contexts than the spec
  requires, and do not add cross-stage state that would defeat the purpose.
- **Human and LLM are co-equal actors**: every Stage 5 write primitive must
  be reachable from UI actions, not only from the pipeline. Tasks tagged
  `[human-mode]` enforce this at the API surface level.
- **TDD**: every Rule in §3 of the spec is unit-tested first. Tests drive
  the Stage 5 invariant implementation. No rule ships without its test.

## Commands

For Rust work use `cargo test`, `cargo clippy`, and `cargo +nightly fmt`
per `{{DEV_ROOT}}/LLM_CONTEXT/fixed-memory/coding-style-rust.md`. See
`{{PROJECT}}/README.md` for anything not covered.

## Dependencies on sibling sub-projects

Sub-project E depends on A, B, C, and D. Until those land, use the stubs
documented in `{{PLAN}}/memory.md` under "Dependencies on sibling
sub-projects." Do not block on sibling work — implement against the stubs,
then swap to real implementations when the siblings land.
