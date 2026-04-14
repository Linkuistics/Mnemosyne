# Memory — Sub-project B: Phase Cycle Reimplementation

This plan implements sub-project B of the Mnemosyne orchestrator merge. The
design is already fully specified; this plan is the implementation work.

## Primary reference

**`{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md`**
is the authoritative design document. Every task in this plan's backlog
derives from that spec. If any implementation question arises that the spec
does not answer, the answer goes into this memory file (and possibly back into
the spec) rather than being invented ad hoc.

## Parent plan

The orchestrator-level plan lives at
`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/` (currently — will be at
`{{PROJECT}}/mnemosyne/plans/mnemosyne-orchestrator/` after sub-project G's
migration). It coordinates this sub-plan with its siblings (A, C, D, E, F,
G, H, and the newly-surfaced K and L). The parent plan's `memory.md` holds
cross-sub-project architectural state. This file holds only sub-project-B-
specific implementation state.

## Key architectural anchors (quick reference; spec is canonical)

These are the decisions most load-bearing for implementation. Consult the
design doc for full context before acting on any of them.

### The six core abstractions
`PlanContext`, `PlanState` (plus `plan-state.md` with YAML frontmatter),
`PhaseRunner`, `PhaseExecutor` (trait + three implementors: Llm, Human,
FixtureReplay), `StagingDirectory`, `InteractionDriver` (trait + three
implementors: Ratatui, Headless, Ipc). See §2.2 of the spec.

### The substitution gap closer
`StagingDirectory::render` pre-renders plan files with `{{DEV_ROOT}}`,
`{{PROJECT}}`, `{{PLAN}}`, `{{PROMPTS}}`, and `{{RELATED_PLANS}}`
substituted to absolute paths (or, for `{{RELATED_PLANS}}`, a
synthesised block of sibling/parent/child plan paths) before the
harness sees them. `copy_back` reverses the substitution on
write-through. This is the load-bearing v1 acceptance criterion from
the seed plan's memory. The five placeholders match LLM_CONTEXT's
`run-plan.sh` upstream to keep bootstrap-discipline dogfooding honest.

### The cycle is four phases: work → reflect → compact → triage
Compact is conditional: `run-plan.sh` upstream (and B's runner) checks
whether `wc -w memory.md <= compact-baseline + HEADROOM` (global
constant, initially 1500 words); if so, compact is skipped and the
next phase is triage, otherwise compact runs and rewrites `memory.md`
losslessly before advancing to triage. After a successful compact, the
script writes the new word count to `compact-baseline` as the new
baseline. Reflect **always** writes `compact` as its nominal next
phase; the compaction-trigger check happens outside the phase itself
so the LLM never has to decide "is compaction needed". B's
`PhaseRunner` must replicate this: reflect's executor writes `compact`
to `plan-state.md`, then runner-level logic (not phase executor logic)
either invokes compact or skips to triage. The `plan-state.md` YAML
frontmatter phase enum must include `compact`.

### Vendored upstream files (embedded prompts)
B's `include_str!`-embedded prompts vendor the current upstream shape:
`phases/{work,reflect,compact,triage}.md` (shared phase files),
`fixed-memory/{coding-style,coding-style-rust,memory-style}.md` (fixed
reference material — memory-style.md in particular is read by both
reflect and compact), and `create-plan.md` (plan-creation spec). These
replace the pre-overhaul vendor list (`backlog-plan.md` and
`create-a-multi-session-plan.md`), which are deleted/renamed upstream
and must not reappear in B's `prompts.rs` constants. The vendor list
is the forward-compatibility chokepoint for LLM_CONTEXT evolution —
every future LLM_CONTEXT change that adds a shared file must either
land in this vendor list or be explicitly scoped out.

### Phase composition: shared phase file + optional per-plan override
LLM_CONTEXT's `run-plan.sh` composes each phase's prompt as `phases/<phase>.md`
concatenated with an optional `<plan>/prompt-<phase>.md`, then substitutes
the placeholders. B's `StagingDirectory::render` must follow the same
contract: the shared phase file is read from the embedded prompts, the
per-plan override (if it exists) is appended, placeholders are
substituted, and the result is materialised into the staging directory.
Typical plans have no override files (reflect/compact/triage) or only
`prompt-work.md`. The override mechanism is additive; `run-plan.sh`
does not allow plans to replace the shared phase content.

### Optional `pre-work.sh` executable hook
LLM_CONTEXT's `run-plan.sh` invokes `<plan>/pre-work.sh` (if present
and executable) before every work phase, from the project root, with
exit-non-zero aborting the whole cycle. B's `PhaseRunner` must
replicate this contract: runs only before work (not reflect, compact,
triage), runs after the defensive `rm -f latest-session.md`, aborts
the cycle on non-zero exit.

### `plan-state.md` is the crash-recovery linchpin
YAML frontmatter carries `ingestion-fired` flag that sub-project E flips at
the start of Stage 5, giving exactly-once firing across Mnemosyne crashes
with file-backed durability and no database. The file replaces the legacy
`phase.md` single-word format.

### Obsidian is the committed explorer UI
Every file format, directory layout, and cross-reference decision targets
Obsidian specifically. Wikilinks instead of filesystem paths,
Dataview-friendly kebab-case YAML frontmatter, tags as first-class metadata,
vault structure designed for Obsidian's file tree pane.

### Per-project directory: `<project>/mnemosyne/` (lowercase)
Replaces the legacy `<project>/LLM_STATE/` (plans only) and
`<project>/knowledge/` (Tier 1 only) split. Contains `plans/` (nested plan
hierarchy) and `knowledge/` (Tier 1 per-project). The vault at
`<dev-root>/Mnemosyne-vault/` has one symlink per project under its
`projects/` subdirectory.

### Plan hierarchy via marker-based discovery
A directory is a plan if and only if it contains `plan-state.md`. Plans can
nest arbitrarily. `StagingDirectory::render` refuses to descend into
subdirectories containing `plan-state.md`, keeping "one plan per Mnemosyne
process" intact. Exact hierarchy semantics are sub-project F's scope.

### Co-equal actors via the executor trait
LLM-driven and human-driven phase executions flow through the same
`PhaseRunner::run_phase` chokepoint. The only permissible difference
between them is the `source` field on emitted events. Sub-project J
effectively folds into B as `ManualEditorExecutor`.

### Hard errors by default
Soft fallbacks require explicit written rationale. Illegal phase
transitions, lock contention, schema version mismatches, copy-back
rejections, symlink cycles, unwritable plan-state — all fail hard with
diagnostics. See feedback rationale in the seed plan's memory.

### Path 1: ratatui v1, Obsidian plugin v2
V1 ships ratatui TUI as the primary UI. The `InteractionDriver` boundary
is hardened (via `IpcDriver` compile-time enforcement) so a future Obsidian
plugin client (sub-project K) can attach without core rework. Both clients
can coexist.

## Implementation strategy

### Phase ordering
**First task is the Obsidian + symlinks validation spike.** Cross-platform
(macOS + Linux via GUIVisionVMDriver golden images). This is a hard
pre-implementation blocker — if the spike fails, the vault layout requires
re-design before any further code is written. See §5.2 of the spec.

After the spike passes, implementation proceeds in dependency order:

1. **Core types** (the six abstractions). No logic yet, just shapes.
2. **`plan-state.md` parse/serialise** (serde_yaml + gray_matter). Pure
   Rust, unit-testable, no dependencies beyond the types.
3. **`StagingDirectory::render`** with all invariants including the
   skip-child-plans rule. Unit tests against fixture plan directories.
4. **`StagingDirectory::copy_back`** with reverse substitution. Unit tests
   covering the longest-match-first algorithm and the rejection rule.
5. **`FixtureReplayExecutor`** and `FixtureReplayAdapter` stub. The
   simplest executor, enabling early end-to-end tests before Claude Code
   integration.
6. **`PhaseRunner::run_phase`** with the full 13-step flow. Integration
   tests using `FixtureReplayExecutor` and a fixture plan.
7. **`ManualEditorExecutor`**. Requires `$EDITOR` spawn; testable via a
   scripted editor stub.
8. **`LlmHarnessExecutor`** + Claude Code adapter stub (real adapter
   comes from sub-project C; until then, a minimal process-spawn stub).
9. **`ReflectExitHook` interface** and the hook-firing path in
   `PhaseRunner`. Integrates with sub-project E's pipeline.
10. **`HeadlessDriver`** for test automation.
11. **`IpcDriver`** for boundary hardening. Compile-time enforcement of
    the serialisable `InteractionDriver` boundary.
12. **`RatatuiDriver`** for the primary UI. The biggest single task.
13. **Symlink rescan logic** and vault bootstrap.
14. **Crash recovery** and startup sequence wiring.
15. **Per-plan advisory lock** (stub until sub-project D lands).
16. **End-to-end dogfood test**: run the orchestrator seed plan on
    Mnemosyne-hosted phase cycles, retiring LLM_CONTEXT's `run-plan.sh`
    for that plan. This is the v1 acceptance test.

### TDD
Every type in §2.2 of the spec has dedicated unit tests written first.
Every rule and invariant in §2.2.5 and §3 has its own test. The rules
are the cheapest insurance against silent regressions once the pipeline
is running.

### No premature optimisation
Harness cold-spawn latency is flagged as a risk but mitigations are
sub-project C's territory. Do not optimise session spawn until
measurements justify it. Do not pre-emptively add warm-pool reuse;
sub-project C handles it when real.

## Dependencies on sibling sub-projects

- **Sub-project C (harness adapter)** — required for `LlmHarnessExecutor`.
  Until C lands, use a minimal process-spawn stub for Claude Code plus a
  fixture-replay adapter stub. The fixture-replay capability is a
  cross-cutting requirement on C.
- **Sub-project D (concurrency)** — required for the per-plan advisory
  lock. Until D lands, use a `flock`-based stub that acquires
  `<vault>/runtime/locks/<plan-id>.lock` with exclusive lock. Swap for
  D's primitive when it lands.
- **Sub-project E (ingestion)** — already complete in design. B exposes
  the `ReflectExitHook` interface; E's implementation subscribes and
  fires when B calls it non-blockingly.
- **Sub-project A (store location)** — B tentatively uses
  `<dev-root>/Mnemosyne-vault/` as the vault default. A's brainstorm
  finalises the name and any config mechanism. B's abstractions are
  layout-agnostic; the path is a startup config value.
- **Sub-project F (plan hierarchy)** — B provides the `plan-state.md`
  marker rule and the descent invariant. F decides everything else
  about hierarchy semantics. B's implementation assumes flat-or-nested
  layouts work identically as long as F respects the marker rule.
- **Sub-project G (migration)** — B tests run against a greenfield vault
  layout; G handles the actual migration of the orchestrator seed plan
  and sub-E's sibling plan. The dogfood step (task 16 above) is
  coordinated with G.
- **Sub-project H (skills fold-in)** — B exposes the `TuiAction` enum
  as an extension point. H's implementation adds new variants for
  skill-derived actions.
- **Sub-project K (Obsidian plugin client)** — v1.5+ future work. B's
  IPC boundary hardening makes K's work pure-additive.
- **Sub-project L (Obsidian terminal plugin spike)** — prerequisite for
  K. Independent of B's implementation.

These stub fallbacks let sub-project B make forward progress in isolation;
real integrations replace the stubs as sibling sub-projects land.

## Open questions (implementation-level)

1. **Exact YAML frontmatter field names** for `plan-state.md` beyond those
   listed in §2.2.2 of the spec. Refine with user review once Dataview
   queries are being written against the file.
2. **Staging allowlist for project-level files.** Safe default: `README.md`
   plus files under `<project>/docs/` explicitly referenced by the phase
   prompt (resolved at prompt-parse time). Refine if real prompts reveal
   this is too narrow.
3. **Keybindings for the Ratatui TUI.** Draft: arrow keys primary, vim
   bindings as alternates, configurable later.
4. **IPC protocol message shapes for `IpcDriver`.** Specified during
   implementation. Safe default: JSON lines with `protocol-version: 1` on
   every message. Concrete shapes committed as fixtures in
   `tests/fixtures/ipc-protocol/` as they are introduced.
5. **Error diagnostic formatting** for TUI display vs. stderr startup
   messages. Draft during implementation.
6. **`phase-prompt.readonly.md` content for manual mode.** Draft: raw
   prompt text; extend with current-plan-state summary and recent
   session-log entries if the bare prompt proves too sparse.
7. **Relative vs. absolute symlink targets** in the vault. Design says
   relative; validate that relative paths work correctly with Obsidian
   across vault moves during the symlinks validation spike.

These are the open questions listed in §5.5 of the spec. Others may surface
during implementation and should be added here.

## BEAM pivot (Session 9 orchestrator decision)

The orchestrator's Session 9 committed Mnemosyne to a **persistent BEAM
daemon (Elixir/OTP)**, replacing the original Rust single-process design.
This fundamentally changes sub-B's implementation language, concurrency
model, and architectural assumptions:

- **Language/runtime:** Rust → Elixir/OTP on the BEAM VM.
- **Concurrency model:** tokio tasks + threads → GenServers + OTP
  supervision trees. PhaseRunner runs inside a `PlanActor` GenServer;
  phase transitions arrive as `{:run_phase, _}` messages.
- **TUI:** No longer the process main loop (ratatui). The TUI is a
  daemon client that connects to the running daemon.
- **`plan-state.md` schema pruning:** Remove `plan-id`, `host-project`,
  `dev-root` fields; add `description:` field.
- **Placeholder rename:** `{{RELATED_PLANS}}` → `{{VAULT_CATALOG}}`.
- **`related-plans.md` deleted** — vault catalog replaces it.
- **Phase-exit hooks:** DispatchProcessor and QueryProcessor run as
  phase-exit hooks (not inline pipeline stages).

The backlog's "Absorb BEAM pivot" amendment task tracks the concrete
work to rewrite task descriptions, spec references, and type definitions
against the new Elixir/OTP architecture. All Rust-specific references
(crates, traits, `include_str!`, `serde_yaml`, `ratatui`, `tokio`,
`fs2`, etc.) must be replaced with their Elixir/OTP equivalents. The
six core abstractions survive conceptually but change form (traits →
behaviours, structs → typed maps/structs, `Arc<dyn ...>` → GenServer
references, etc.).

## Risk watch list

Ranked by impact × likelihood from §5.4 of the spec. Each has a mitigation
path; flag here if implementation reveals the risk materialising.

1. **Obsidian + symlinks behaviour not what we think it is.** Addressed by
   the pre-implementation validation spike (first task).
2. **Harness adapter cold-spawn latency.** Addressed by sub-project C's
   warm-pool strategy if it proves painful.
3. **Concurrent edits via two paths** (vault symlink vs. canonical project
   path). Low-likelihood; mitigated by user documentation.
4. **IPC boundary design drift.** Addressed by `IpcDriver` shipping in v1
   as compile-time enforcement.
5. **`schema-version` management discipline.** Mechanical; enforced by
   dedicated test.
6. **Reverse substitution collisions.** Low-likelihood; mitigated by
   scoping substitution to `.md` files and named placeholder tokens only.
