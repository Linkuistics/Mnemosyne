# Work Phase — Sub-project A: Vault Location, Discovery, and Bootstrap

## Additional reads

- `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-A-global-store-design.md`
  — the authoritative design document for this sub-project. Every
  task below derives from this spec. Consult it before starting any
  task.

## About this plan

This plan implements sub-project A of the Mnemosyne orchestrator merge
— the vault discovery mechanism, identity marker, vault directory
layout, `init` / `init --from` / `config use-vault` / `adopt-project`
CLI commands, Tier 1 / Tier 2 root resolution, and the deletion of
v0.1.0's hardcoded `~/.mnemosyne/` paths.

The design is fully specified in the design doc referenced above; this
plan is the implementation work, not a design phase. Do not re-litigate
design decisions here; surface them to the parent plan
(`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`) if you
discover a spec-level issue during implementation.

## Key constraints

- **v0.1.0 has no real users.** Legacy `~/.mnemosyne/` paths are
  deletable, not transitionable. Delete them outright in this plan's
  task 11 — do not leave fallback shims, compatibility layers, or
  deprecation warnings. The `Config::load(dir)` call site in
  `src/config.rs` becomes dead code in task 12 and should be deleted.
- **Hard errors by default**: illegal states, invariant violations,
  I/O failures, and unexpected conditions all fail hard with clear
  diagnostics naming the offending path. Soft fallbacks require
  explicit written rationale in the code or the design doc. Never
  silently log-and-continue. This applies especially to the
  `verify_vault` function and the `init` pre-flight checks.
- **Non-disruption of concurrent sub-projects**: sub-projects B, D, E,
  F, and I run in parallel with A. Changes must not break sibling
  brainstorms' assumptions. Core types land first so sibling plans
  can depend on real types rather than stubs.
- **TDD**: every type and every invariant in spec §A1–A10 is
  unit-tested first. Tests drive implementation. No code ships
  without its tests.
- **Obsidian-native formats**: every file Mnemosyne writes uses
  markdown with YAML frontmatter, kebab-case property names for
  Dataview, wikilinks for cross-references, tags as first-class
  metadata. The `.obsidian/` template authored in task 6 must honour
  this discipline.

## Commands

For Rust work, use `cargo test`, `cargo clippy`, and `cargo +nightly
fmt`. See `{{PROJECT}}/README.md` for the full build and test command
surface. Run the full test suite and clippy before declaring any task
done — no task ships with failing tests or new warnings.

## Dependencies on sibling sub-projects

Sub-project A's types (`VaultMarker`, `ResolvedRoots`) are consumed by
sub-projects B, D, E, and F. Land core types first so siblings can
depend on real implementations. See `{{PLAN}}/memory.md` under
"Dependencies on sibling sub-projects" for the full cross-plan
dependency map.
