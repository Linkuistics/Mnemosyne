# Work Phase — Sub-project A: Vault Location, Discovery, and Bootstrap

Read the following before doing anything else:

1. `{{PROJECT}}/README.md` for Mnemosyne's project conventions,
   architecture, CLI surface, and v0.1.0 status. Mnemosyne is a Rust
   project; build and test commands are documented there.
2. `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` for the work / reflect /
   triage phase cycle specification.
3. `{{DEV_ROOT}}/LLM_CONTEXT/coding-style.md` and
   `{{DEV_ROOT}}/LLM_CONTEXT/coding-style-rust.md` for coding
   conventions.
4. `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-A-global-store-design.md`
   — the authoritative design document for this sub-project. Every
   task below derives from this spec. Consult it before starting any
   task.
5. `{{PLAN}}/backlog.md` for the current implementation task list.
6. `{{PLAN}}/memory.md` for implementation-level decisions, cross-sub-project
   dependencies, and open questions.

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

### Key constraints during implementation

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
- **Test and build commands**: see `{{PROJECT}}/README.md`. For Rust
  work, use `cargo test`, `cargo clippy`, and `cargo +nightly fmt`
  per `{{DEV_ROOT}}/LLM_CONTEXT/coding-style-rust.md`.

### Dependencies on sibling sub-projects

Sub-project A's types (`VaultMarker`, `ResolvedRoots`) are consumed by
sub-projects B, D, E, and F. Land core types first so siblings can
depend on real implementations. See `{{PLAN}}/memory.md` under
"Dependencies on sibling sub-projects" for the full cross-plan
dependency map.

## Path placeholders

Any path beginning with `{{PROJECT}}`, `{{DEV_ROOT}}`, or `{{PLAN}}`
should be interpreted as the absolute path the shell script
substitutes before passing the prompt to the LLM. If you see a literal
`{{PROJECT}}`, `{{DEV_ROOT}}`, or `{{PLAN}}` token in any file you
Read inside the dev root, substitute it mentally with the correct
absolute path before passing it to the Read tool.

## Working a task

1. Display a summary of the current backlog: title, status, and the
   relative priority order (top of the backlog file = highest
   priority).
2. Ask the user if they have input on which task to work on next.
   Wait for their response. If they have a preference, work on that
   task; otherwise pick the highest-priority `not_started` task whose
   dependencies are all `done`.
3. Work the task using TDD: write the failing test first, implement
   the minimum code to pass, refactor, commit. Consult the design doc
   for any behavioural question.
4. Run the full test suite and clippy before declaring the task done.
   No task ships with failing tests or new warnings.
5. Record results in `{{PLAN}}/backlog.md` — replace `_pending_` with
   a concrete summary of what was built, tests added, and any
   surprises. Update the task `Status` to `done`.
6. Append a session log entry to `{{PLAN}}/session-log.md` per the
   format in `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md`:
   `### Session N (YYYY-MM-DD) — title`, bullets for what was
   attempted, what worked / didn't, what to try next, key learnings.
7. Write `reflect` to `{{PLAN}}/phase.md`.
8. Stop. Do not pick another task. Do not enter the reflect phase
   yourself — the next phase runs in a fresh session.
