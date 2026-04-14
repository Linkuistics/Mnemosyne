# Work Phase — Sub-project B: Phase Cycle Reimplementation in Rust

Read for this plan:

- `{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md`
  — the authoritative design document for this sub-project. Every task
  in the backlog derives from this spec. Consult it before starting any
  task.

## About this plan

This plan implements sub-project B of the Mnemosyne orchestrator merge —
the long-running process model, phase cycle state machine, executor
abstractions, placeholder substitution mechanism, and plan-state file
format that replace the LLM_CONTEXT `run-plan.sh` driver. The cycle is
**four phases** (work → reflect → compact → triage) mirroring LLM_CONTEXT's
current upstream shape; compact is conditional on a wc-word-count trigger
against a `compact-baseline` integer file, per `run-plan.sh`'s semantics.

The design is fully specified in the design doc referenced above; this
plan is the implementation work, not a design phase. Do not re-litigate
design decisions here; surface them to the parent plan
(`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`) if you discover
a spec-level issue during implementation.

## Key constraints

- **The Obsidian + symlinks validation spike is the first task** and is a
  hard pre-implementation blocker. Until it passes on both macOS and Linux
  (via GUIVisionVMDriver golden images), no other sub-B task starts. If
  the spike fails, open a brainstorm-mode discussion to resolve the
  fallback layout before resuming.
- **Non-disruption**: existing Mnemosyne v0.1.0 must keep running during
  the build. New code lives alongside, not in place of, existing code.
  The dogfood acceptance test is the final task in this plan's backlog
  and is the moment v1 replaces `run-plan.sh` for the orchestrator
  seed plan.
- **Type-level co-equal-actors**: every phase execution, LLM or human, must
  flow through the same `PhaseRunner::run_phase` chokepoint. Tests that
  exercise `run_phase` with multiple executors verify this at the type
  level. Do not add execution paths that bypass the chokepoint.
- **Hard errors by default**: illegal states, invariant violations, I/O
  failures, and unexpected conditions all fail hard with clear diagnostics.
  Soft fallbacks require explicit written rationale in the code or the
  design doc. Never silently log-and-continue.
- **Obsidian-native formats**: every file Mnemosyne writes uses markdown
  with YAML frontmatter, kebab-case property names for Dataview, wikilinks
  for cross-references, tags as first-class metadata.
- **TDD**: every type and every invariant in §2.2 and §3 of the spec is
  unit-tested first. Tests drive implementation. No code ships without
  its tests.

## Commands

Rust work uses `cargo test`, `cargo clippy`, and `cargo +nightly fmt`.
Run the full test suite and clippy before declaring any task done —
no task ships with failing tests or new warnings. See
`{{PROJECT}}/README.md` for Mnemosyne's full build/test surface.

## Dependencies on sibling sub-projects

Sub-project B depends on C (adapter), D (locking), E (ingestion hook),
and conceptually on A (vault location) and F (plan hierarchy semantics).
Until those land, use the stubs documented in `{{PLAN}}/memory.md` under
"Dependencies on sibling sub-projects." Do not block on sibling work —
implement against the stubs, then swap to real implementations when the
siblings land.

## GUIVisionVMDriver for the symlinks validation spike

The first task uses GUIVisionVMDriver (at `{{DEV_ROOT}}/GUIVisionVMDriver/`)
to run the Obsidian + symlinks validation on both macOS and Linux VMs.
Consult `{{DEV_ROOT}}/GUIVisionVMDriver/instructions-for-llms-using-this-as-a-tool.md`
for the commands. Capture evidence (screenshots, accessibility snapshots)
and commit them to `tests/fixtures/obsidian-validation/`.
