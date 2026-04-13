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
session, (5) deterministic validate + apply. Stages 1/2/5 are pure Rust.
Stages 3/4 use the harness adapter (sub-project C).

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
  requirement on C that must be honoured in its design.
- **Sub-project A (store location)** — required for Stage 5 file writes.
  Until A lands, use a configurable Tier 1 root path defaulting to
  `{{PROJECT}}/knowledge/` and a Tier 2 root defaulting to
  `~/.mnemosyne/knowledge/` (the existing v0.1.0 location). Update both when
  A lands.
- **Sub-project B (phase cycle)** — provides the reflect-exit hook that
  triggers ingestion. Until B lands, ingestion is invocable only via the
  manual entry point (goal #6's human-triggered ingestion), not automatically
  from a cycle.
- **Sub-project D (concurrency)** — required for the store write lock in
  Stage 5. Until D lands, use a single-writer assumption and a
  filesystem-level `.lock` sentinel as a placeholder.

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
