# Memory — Sub-project F: Plan Hierarchy, Actor Model, Dispatch, Declarative Routing

This plan implements sub-project F of the Mnemosyne orchestrator merge.
F's brainstorm is complete; this plan is the implementation work, not
a design phase. The design is fully specified in the spec referenced
below. If an implementation question arises that the spec does not
answer, the answer goes into this memory file (and possibly back into
the spec) rather than being invented ad hoc.

## Primary reference

**`{{PROJECT}}/docs/superpowers/specs/2026-04-14-sub-F-hierarchy-design.md`**
is the authoritative design document. Every task in this plan's backlog
derives from §11 of that spec. Consult §1–§10 of the spec before acting
on any implementation task.

## Parent plan

The orchestrator-level plan lives at
`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/` (currently — will move
to `{{PROJECT}}/mnemosyne/project-root/` after sub-project G's
migration lands). It coordinates this sub-plan with its siblings (A, B,
C, D, E, G, H, I, M, N). The parent plan's `memory.md` holds
cross-sub-project architectural state. This file holds only
sub-project-F-specific implementation state.

## Key architectural anchors (quick reference; spec is canonical)

These are the decisions most load-bearing for implementation. Consult
§1–§10 of the design doc for full context before acting on any of them.

### F-1. Mnemosyne is a persistent actor daemon on BEAM

V1 runs as a single long-running Elixir/OTP application. `mnemosyne
daemon` is the entry point; it hosts all actors, message routing,
supervision, harness adapters, fact extraction, and ingestion. OTP
supervision, message passing, hot code reload, and distribution
transparency are BEAM primitives — use them, do not hand-roll.

### F-2. Two sealed actor types: PlanActor and ExpertActor

Both implement a shared `Mnemosyne.Actor` behavior on top of
`GenServer`. The set is **sealed** for v1 — adding a third type is a
code change, not a plugin mechanism. PlanActor progresses through phase
cycles (wraps sub-B's `PhaseRunner`). ExpertActor is a type-hole stub
returning `:not_yet_implemented` until sub-N lands.

### F-3. Two message types: Dispatch and Query

Both share target resolution, declarative routing, Level 2 fallback,
and audit trail conventions. Dispatch is fire-and-forget to a target's
backlog `Received` section. Query is request-response via the tool-call
boundary in sub-C's adapter.

### F-4. Plan hierarchy via `project-root` convention

Every adopted project has exactly one root plan at
`<project>/mnemosyne/project-root/`. Name is **reserved**. Nested child
plans nest arbitrarily beneath it. A directory is a plan iff it
contains `plan-state.md`. `knowledge/` lives as a sibling of
`project-root/`, not inside it.

### F-5. Path-based qualified plan IDs, never stored

A plan's qualified ID is a pure function of its filesystem path:
`strip_prefix(plan_path, "<vault>/projects/")`. Examples:
`Mnemosyne/project-root`, `Mnemosyne/project-root/sub-F-hierarchy`.
Never stored in `plan-state.md` frontmatter. Filesystem is
authoritative; qualified IDs computed at read time.

### F-6. Dispatch target resolution is asymmetric

- **Same-project** → origin names the specific `target-plan:
  <qualified-id>`. Mnemosyne writes to the target's `Received` section
  mechanically. No LLM in the loop.
- **Cross-project** → origin names only `target-project: <name>`.
  Mnemosyne spawns a **Level 2 routing agent** — fresh-context Claude
  Code session scoped to the target project's vault subtree (plans +
  source code), with authority to pick a specific target plan or
  reject with reasoning.

### F-7. Vault catalog as LLM-facing enumeration

`{{VAULT_CATALOG}}` is substituted at phase-prompt render time with
the full vault catalog at `<vault>/plan-catalog.md` — every plan and
expert, grouped by project, with 120-char descriptions and dispatch
rules. Auto-regenerated on plan mutation and every phase-prompt
render. Replaces the old `{{RELATED_PLANS}}` placeholder. No plan's
LLM phase ever reads another plan's files directly.

### F-8. Description discipline: 120-character hard cap

Every plan and expert has a `description:` frontmatter field, ≤120
characters, enforced at load time (hard error on overflow).
Keyword-dense, noun-phrase-led, no self-reference, no placeholders.
Permanent scope declaration, not current-state description.

### F-9. Declarative routing with LLM-fallback learning loop

Routing rules live in `<vault>/routing.ex` — user-editable Elixir
module with pattern-matched `defp route/2` clauses. BEAM's native hot
code reload makes edits take effect without daemon restart. Facts
(concern keywords) extracted by a cheap LLM pass (Haiku via sub-C).
When rules don't decide, Level 2 routing agent runs as fallback and
may propose a new rule the user accepts into `routing.ex` — closing a
learning loop.

### F-10. Runtime: Elixir on BEAM

Erlang/OTP 28 + Elixir 1.19.x. Dependencies: `erlexec` (validated by
the BEAM PTY spike in Session 10), `jason`, `telemetry`, `yaml_elixir`.
Rust is used only for the TUI client binary (separate implementation
plan). Gleam is a future migration target if Elixir's dynamic typing
proves painful for the invariant-heavy design.

### F-11. TUI is a separate Rust client binary over Unix socket NDJSON

`mnemosyne-tui` is a standalone Rust binary using `ratatui` + `tokio`
+ `serde_json`. Connects to the daemon over a local Unix socket at
`<vault>/runtime/daemon.sock` speaking NDJSON. No Rustler / no NIFs —
the socket is the integration boundary. This plan scaffolds the
daemon side of that contract; the TUI binary itself belongs to a
**separate implementation plan to be scaffolded later**.

### F-12. Reserved extensibility hooks (sub-O, sub-P)

`daemon.toml` reserves `[harnesses.*]` sections for sub-O (mixture of
models, v1.5+). Actor declarations reserve the `model:` field. The
`[peers]` section and `<peer>@<qualified-id>` syntax are reserved for
sub-P (team mode, v2+). V1 parses these sections with forward
compatibility (unknown `[harnesses.foo]` → warn and ignore; non-empty
`peers: []` → hard error with "team mode not supported in v1").

## Contract with sibling sub-projects

### Consumes from sub-A

Vault discovery and identity verification. F's `VaultDirectory.load`
is called from A's `verify_vault` path at daemon startup. A owns the
vault discovery chain (`--vault` → `MNEMOSYNE_VAULT` → user config →
hard error); F owns what happens next.

### Consumes from sub-B

`Mnemosyne.PhaseRunner` and its 13-step `run_phase/4` flow. F's
`PlanActor` hosts this runner inside its `GenServer` state and
invokes it in response to `{:run_phase, _}` messages from attached
clients. F's `DispatchProcessor` and `QueryProcessor` are called by
B's `PhaseRunner` as phase-exit hooks on non-compact phases.

### Consumes from sub-C

`Mnemosyne.HarnessAdapter` behaviour. F's Level 2 routing agent and
fact-extraction LLM pass spawn sessions via the same adapter the
phase runner uses. F's Query messages are delivered mid-session via
C's tool-call boundary (`Mnemosyne.Router.handle_tool_call/4`).

### Consumes from sub-M

`:telemetry` + typed `Mnemosyne.Event.*` structs for observability.
F emits `%ActorStateChange{}`, `%MessageRouted{}`, `%RuleFired{}`,
`%RuleSuggestion{}`, and `%DispatchProcessed{}` / `%QueryAnswered{}`
structs at the boundary. M owns the transport and subscribers.

### Depends on (not yet landed at F's scaffolding time)

Implementation of F **cannot start** until the downstream task lists
for sub-B and sub-C are rewritten against their Session-11 / Session-12
design-doc rewrites. Both sibling backlogs carry explicit gate tasks
for that rewrite. F's Task 0 is the symmetric gate. Sub-A, sub-E, and
sub-M amendments are not blockers for F to begin — F can progress
against their design docs — but some integration tasks (Tasks 22–24)
become real only once those siblings land.

## Non-goals for this plan

- **Sub-N (ExpertActor internals)** — F ships the type hole only.
  Persona format, retrieval strategies, default expert set, and
  ingestion Stage 5 integration are all sub-N's scope.
- **Sub-O (mixture of models)** — F reserves the schema hooks; does
  not wire them.
- **Sub-P (team mode)** — F reserves the schema hooks; does not wire
  them.
- **Rust TUI binary (`mnemosyne-tui`)** — a separate implementation
  plan. This plan scaffolds only the daemon-side client-listener
  (Task 11) that the TUI will connect to.
- **Phase-cycle mechanics (B's scope)** — F consumes `PhaseRunner`;
  it does not reimplement or extend the phase flow.

## Bootstrap discipline

Until sub-G's migration lands and Mnemosyne v1 replaces LLM_CONTEXT,
this plan runs on existing LLM_CONTEXT machinery. Do not assume
features Mnemosyne does not yet have. The `run-plan.sh` cycle driver,
`phases/work.md`, fixed-memory conventions, and placeholder
substitutions all come from LLM_CONTEXT for the duration of this
plan's life.

## Hard errors by default

Every invariant violation, I/O failure, parse error, ambiguous state,
and unexpected condition fails hard with a clear diagnostic naming
the offending file and line. Soft fallbacks require explicit written
rationale in the design doc. Project-wide; F carries it into every
actor, router, and dispatch-processor path.

## Amendment tasks rewrite specs inline, not as supersede layers

If a downstream pivot invalidates parts of F's design doc during
implementation, rewrite the affected sections inline and record the
correction as a new Decision Trail entry in Appendix A. Do not
append supersede blocks. Validated at scale by sub-B (2296 lines,
three simultaneous pivots absorbed) and sub-C (1186 lines, one
major pivot absorbed).
