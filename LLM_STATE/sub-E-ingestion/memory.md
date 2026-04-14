# Memory — Sub-project E: Post-Session Knowledge Ingestion

This plan implements sub-project E of the Mnemosyne orchestrator merge. The
design is already fully specified; this plan is the implementation work.

## Primary reference

**`{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-E-ingestion-design.md`**
is the authoritative design document. Every task in this plan's backlog
derives from that spec. If any implementation question arises that the spec
does not answer, the answer goes into this memory file (and possibly back into
the spec) rather than being invented ad hoc.

## Parent plan

The orchestrator-level plan lives at
`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/` and coordinates this sub-plan
with its siblings (A, B, C, D, F, G, H, and the proposed I/J). The parent
plan's `memory.md` holds cross-sub-project architectural state. This file
holds only sub-project-E-specific state.

## Key architectural anchors (quick reference; spec is canonical)

These are the decisions most load-bearing for implementation. Consult the
design doc for full context before acting on any of them.

### Pipeline shape
Five stages: (1) deterministic section parse, (2) deterministic retrieval,
(3) N fresh per-section LLM sessions, (4) one fresh reconciliation LLM
session, (5) dispatch-to-experts via Query messages. Stages 1–4 are
structurally unchanged from the original design. Stage 5 no longer performs
direct store writes — instead it dispatches candidate entries to ExpertActors
(sub-N) who review in fresh context and absorb, reject, or cross-link.
Multi-expert absorption is allowed. Implementation is Elixir (GenStage or
Broadway for pipeline stages), not Rust — per the BEAM pivot committed in
orchestrator Session 9.

### Six invariants in Stage 5
1. Confidence mapping (experiential/speculative → confidence level per table)
2. Contradiction gating (supersede allowed only under a confidence rule)
3. Axis derivation (deterministic `suggest_axis(tags)`)
4. Tier routing (Tier 1 always for new; Tier 2 via graduation prompt)
5. Interactive UI events (graduation + supersession fire prompts)
6. File-write safety (no writes outside knowledge/, no plan-file writes, etc.)

### Fresh LLM context is a first-class goal
Every LLM session in the pipeline sees the smallest possible context that
answers its one question. No session holds the full store, the full plan
memory, or multi-section reasoning. This is load-bearing for the whole
design and must not be eroded during implementation.

### Human and LLM are co-equal actors
Every state change the LLM can trigger must have a human-driven equivalent
through the same Stage 5 invariants. Implementation must expose the write
primitives as UI actions, not only as pipeline outputs.

### Auto-absorb by default; live prompts only on graduation + supersession
Most ingestion writes apply silently and land in the Ingestion Events
Explorer. Only two event classes interrupt the user with a live prompt.

## BEAM pivot and F amendment

The orchestrator's Session 9 committed Mnemosyne to a persistent BEAM daemon
(Elixir/OTP). Sub-E was brainstormed assuming Rust. The F amendment re-casts
sub-E as follows:

- **Stages 1–4 unchanged** in logic; re-implemented in Elixir. GenStage or
  Broadway for pipeline stage composition (replaces the tokio channel wiring).
- **Stage 5 becomes dispatch-to-experts.** Instead of directly writing to the
  knowledge store, Stage 5 sends candidate entries as Query messages to
  ExpertActors (sub-N). Experts review in fresh context and absorb, reject,
  or cross-link. Multi-expert absorption is allowed.
- **Six invariants preserved.** Rules 1–4 and 6 still apply before dispatch;
  Rule 5 (interactive UI events) still fires graduation + supersession
  prompts. The invariants gate what gets dispatched, not what gets written —
  experts handle the write.
- **Sub-N dependency added.** ExpertActor internals (persona, retrieval,
  curation) are sub-N's scope. Sub-E dispatches to experts; sub-N defines
  what experts do with the dispatched entries.
- **Rust references retired.** `Vec<Section>`, `tokio mpsc`, `SafeFileWriter`,
  and other Rust-specific framing in the original backlog tasks will be
  re-interpreted as Elixir equivalents during implementation.

This amendment is tracked in the orchestrator's backlog as one of five
parallel amendment tasks (A, B, D, E, M) currently unblocked.

## Implementation strategy

**Phase ordering.** Build deterministic stages first (1, 2, 5 with all six
invariants). Build event emission and jsonl logging next. Only then build
stages 3 and 4, using a fixture-replay adapter stub before the real
sub-project C adapter lands. This keeps forward progress independent of
sub-project C's timeline.

**TDD.** Every rule in §3 of the spec has a dedicated unit test written
first. The six rules are the cheapest insurance against silent regressions
once the pipeline is running.

**No premature optimisation.** LLM session latency is flagged as a risk but
the backstop (dropping Stage 4 as degradation path) means the pipeline can
ship slow. Do not optimise session spawn until measurements justify it.

## Dependencies on sibling sub-projects

- **Sub-project C (harness adapter)** — required for Stages 3 and 4. Until C
  lands, use a fixture-replay stub adapter that reads recorded session
  outputs from JSON files. The fixture-replay capability is a cross-cutting
  requirement on C that must be honoured in its design. Post-F: C is now
  Elixir (PTY via `erlexec` or Rust wrapper Port).
- **Sub-project A (store location)** — required for Stage 5 expert dispatch
  targeting. Until A lands, use a configurable Tier 1 root path defaulting to
  `{{PROJECT}}/mnemosyne/knowledge/` and a Tier 2 root defaulting to
  `<vault>/knowledge/`. Update both when A lands.
- **Sub-project B (phase cycle)** — provides the reflect-exit hook that
  triggers ingestion. Until B lands, ingestion is invocable only via the
  manual entry point (goal #6's human-triggered ingestion), not automatically
  from a cycle. Post-F: PhaseRunner runs inside PlanActor GenServer.
- **Sub-project D (concurrency)** — post-F scope collapsed: OTP mailbox
  serialization replaces advisory file locks. D now covers daemon singleton
  lock + advisory locks for external tools (Obsidian, git).
- **Sub-project N (domain experts)** — **new dependency added by F.** Stage 5
  dispatches candidate entries to ExpertActors owned by sub-N. Sub-N defines
  expert personas, retrieval strategies, knowledge curation logic, and the
  default expert set. Sub-E must define the Query message contract that
  experts consume.
- **Sub-project F (hierarchy + actors)** — owns the `Actor` behaviour,
  message types (Dispatch/Query), and vault catalog. Stage 5's expert
  dispatch uses F's Query message infrastructure.

These stub fallbacks let sub-project E make forward progress in isolation;
real integrations replace the stubs as their sibling sub-projects land.

## Open questions (implementation-level)

1. **Stage 2 tag extraction algorithm.** Safe default from the spec:
   stop-word-filtered lowercase nouns from section title, intersected with
   existing tag vocabulary. Validate this against real plan-memory corpora
   during implementation.
2. **Concrete prompt texts** for Stage 3, Stage 4, and research sessions.
   Draft during implementation, iterate against recorded fixtures.
3. **"Different project" canonical identity.** Safe default: repo directory
   name under DEV_ROOT. Consider per-repo `project.id` config if real-world
   usage demands it.
4. **Tier 1 frozen-entry pointer comment format.** Trivial; decide when
   writing Rule 4's graduation logic.

These are the open questions listed in §5.1 of the spec. Others may surface
during implementation and should be added here.
