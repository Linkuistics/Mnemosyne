# Sub-project E — Post-Session Knowledge Ingestion Model

Designed via the `superpowers:brainstorming` skill on 2026-04-12. Every
decision was presented to the user at a decision point during the original
session. Substantively rewritten on 2026-04-16 to absorb the BEAM/actor
pivot committed in orchestrator Session 9 (sub-F) and the Stage 5
expert-dispatch contracts produced by sub-N's Session 16 brainstorm
(2026-04-15). This is an inline rewrite — no supersede layer — per the
project-wide "amendments rewrite inline" discipline. History lives in
Appendix A; corrections are folded into the original question entries.

---

## Overview

Sub-project E designs how Mnemosyne — running as a persistent BEAM/OTP
daemon hosting plan and expert actors — reads each plan's outputs after a
session completes and turns them into knowledge updates owned by domain
experts.

The design is:

- A **five-stage pipeline** in Elixir: three deterministic stages (parse,
  retrieve, validate+dispatch) and two fresh-context LLM stages (per-section
  ops, reconciliation), composed via `GenStage` (or `Broadway` if the
  back-pressure semantics warrant it).
- Governed by **six deterministic invariants**: confidence mapping,
  contradiction gating, axis derivation, tier routing, interactive-event
  gating, and file-write safety. The first four and the sixth gate **what
  gets dispatched**; the fifth still fires live UI prompts on graduation
  and supersession.
- **Stage 5 is dispatch-to-experts.** Instead of writing directly into the
  store, Stage 5 fans out `%Mnemosyne.Message.ExpertAbsorbCandidate{}`
  messages in parallel to every tag-matching `ExpertActor` (sub-N) via
  sub-F's router. Each expert reviews in fresh context and emits one of
  three verdicts (`:absorb`, `:reject`, `:cross_link_suggested`). Multi-
  expert absorption is accepted (physical duplication keyed by
  `ingestion-event-id`). Orphan candidates (zero matching experts) bypass
  the fan-out entirely and write straight to
  `<vault>/knowledge/uncategorized/`.
- Firing **live interactive prompts** on two high-stakes pre-dispatch event
  classes (Tier 2 graduation and supersession) through sub-M's observability
  bus into sub-K-attached clients (Rust TUI in v1, Obsidian plugin in
  v1.5+), with an optional on-demand **research LLM** that can be launched
  from any prompt to investigate a decision before the user resolves it.
- Exposing every action to both **LLM and human actors on equal footing**.
  No ingestion workflow is LLM-only; every state change reachable via the
  pipeline is also reachable via direct user action through an attached
  client, through the same invariants.
- Emitting **structured ingestion events** as members of sub-M's sealed
  `Mnemosyne.Event.*` set. Sub-E does not define its own event types —
  it consumes the six `Mnemosyne.Event.Ingestion.*` variants declared in
  sub-M §4.1, plus its sub-E-owned `%Mnemosyne.Event.Ingestion.OrphanCandidate{}`
  for the orphan path. The Ingestion Events Explorer is a view rendered by
  attached clients over sub-M's event log; sub-E does not own a parallel
  channel.

The design assumes the persistent-daemon architecture established in
`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`: Mnemosyne runs
continuously as a single BEAM application hosting `PlanActor`s and
`ExpertActor`s, spawns child harness sessions via sub-C's adapter for
LLM-driven phases, and routes messages between actors through sub-F's
declarative router.

---

## Table of Contents

1. [Scope and Goals](#1-scope-and-goals)
2. [Pipeline Architecture](#2-pipeline-architecture)
3. [Rules and Invariants](#3-rules-and-invariants)
4. [Event Flow and the Ingestion Events Explorer](#4-event-flow-and-the-ingestion-events-explorer)
5. [Open Questions, Cross-Sub-Project Requirements, and Risks](#5-open-questions-cross-sub-project-requirements-and-risks)

---

## 1. Scope and Goals

### In scope

The mechanism by which Mnemosyne reads a plan's outputs after a cycle and
turns them into knowledge updates owned by domain experts: trigger timing,
file inputs, pipeline stages, decision rules, contradiction handling,
confidence assignment, axis assignment, tier routing, expert dispatch,
interactive event flow, research sessions, event emission via sub-M's bus,
and the requirements this design generates on sibling sub-projects.

### Deliberately out of scope

- **Harness child session spawning** — sub-project C. Sub-project E assumes C
  provides a capability to spawn a fresh Claude Code session with a scoped
  prompt, a configurable tool profile, and streaming output capture, all in
  Elixir over `erlexec` pipes.
- **Phase cycle state machine** — sub-project B. E specifies only the
  hook contract (`Mnemosyne.ReflectExitHook`) and the file-read contract.
- **Vault discovery and Tier 1/Tier 2 root resolution** — sub-project A.
  E assumes both roots are resolvable paths at daemon boot.
- **Concurrency between daemon and external editors** — sub-project D
  (post-F scope: daemon singleton lock + advisory locks for Obsidian/git).
  OTP mailbox serialization handles intra-daemon coordination — Stage 5
  does not need its own write lock for daemon-internal writes.
- **ExpertActor internals** — sub-project N. E dispatches via the contracts
  in sub-N §6 and §9.4 and does not reach into expert state.
- **Attached-client UI chrome (TUI vs Obsidian plugin vs web)** — sub-K and
  later. E decouples from clients via sub-M's observability bus, and a
  fixture-replay test harness is sufficient to drive end-to-end ingestion
  tests with no client attached.
- **Curate command redesign** — folded into sub-H's TUI-actions scope.
- **Horizon-scanning-driven ingestion** — separate path, not addressed here.
- **Migration of existing Mnemosyne v0.1.0 entries** — sub-project G.
  E designs greenfield. The v0.1.0 sunk cost is small enough that the BEAM
  pivot writes off the previous Rust-CLI ingestion wiring outright.

### Goals, in priority order

1. **Eliminate the LLM-discipline failure mode for promotion.** No tool
   calls from child harnesses back to Mnemosyne to "promote" knowledge.
   Parent reads files, parent decides, parent dispatches to experts,
   experts write. The question "did the LLM remember to promote that?"
   must become structurally impossible.
2. **Maximise fresh LLM context.** Decompose ingestion into the smallest
   possible per-session LLM contexts, each with one conceptually scoped job.
   Never ask one LLM session to hold the full store, the full plan memory,
   and the full ingestion decision set at once. This is load-bearing —
   it follows from the project-wide "fresh LLM context is a first-class
   architectural goal" decision.
3. **Preserve Mnemosyne's knowledge philosophy.** Auto-absorb must not
   silently corrupt the "living beliefs" semantic. Supersession,
   contradiction, and confidence inflation are gated by rules pre-dispatch
   and surfaced for user review on high-stakes events. Experts may also
   reject candidates downstream of the rules; the rule layer is a floor,
   not a ceiling.
4. **Auto-absorb by default, interactive on high-stakes transitions.**
   Low-stakes ingestion (new entries, appended observations, confidence-
   preserving updates) flows silently into the explorer. Graduation to
   Tier 2 and supersession of existing content fire live interactive
   prompts before dispatch.
5. **Every ingestion event is a member of sub-M's sealed event set.**
   Chronological browsing, search, filter, re-opening deferred prompts,
   research session transcripts, and full audit history across LLM-driven
   and human-driven operations alike are rendered by attached clients
   over sub-M's bus. Sub-E does not own a parallel channel.
6. **Human and LLM are co-equal actors.** Every action the LLM is authorised
   to take in the ingestion pipeline must have a human-driven equivalent
   that produces the same state change through the same invariants and
   the same dispatch path to experts. The LLM is never a bypass. No
   workflow may be LLM-only.

---

## 2. Pipeline Architecture

Ingestion is a five-stage pipeline that fires exactly once per cycle,
triggered by sub-B's `Mnemosyne.ReflectExitHook` callback when the reflect
phase exits cleanly. Stages 1, 2, and 5 are deterministic Elixir; stages 3
and 4 are fresh-context LLM sessions spawned via sub-C's harness adapter.
Stage composition is a `GenStage` pipeline rooted at the
`Mnemosyne.Ingestion.Pipeline` GenServer; back-pressure between stages is
explicit, and Stage 3's per-section sessions fan out via a `Task.Supervisor`
inside the Stage 3 consumer.

```
reflect phase exits cleanly (sub-B PhaseRunner inside PlanActor)
    │
    ▼ Mnemosyne.ReflectExitHook.on_reflect_exit(context)
    │ (called non-blockingly via Task.Supervisor.start_child/3
    │  under the PlanActor's task supervisor; ingestion does
    │  NOT block phase advancement to triage)
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 1 — SECTION PARSE (deterministic Elixir)                  │
│ Input:  <plan_dir>/memory.md, <plan_dir>/session-log.md latest  │
│ Output: [%Section{}] where each %Section{} carries:             │
│          - title (from H2/H3 heading)                           │
│          - body_markdown                                        │
│          - plan_qualified_id (provenance)                       │
│          - session_date (from session-log)                      │
│          - host_project                                         │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 2 — RETRIEVAL (deterministic Elixir)                      │
│ For each %Section{}:                                            │
│   - Extract candidate tags from the section title via the      │
│     stop-word-filtered lowercase-noun heuristic, intersected    │
│     with the current tag vocabulary                             │
│   - Query the knowledge store via the KnowledgeStore behaviour │
│   - Produce a %StoreSlice{}: 0–5 existing entries whose tags    │
│     overlap the candidate tag set                               │
│ Output: [{%Section{}, %StoreSlice{}}]                           │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 3 — PER-SECTION LLM OPS (N fresh sessions in parallel)    │
│ For each {%Section{}, %StoreSlice{}}:                           │
│   Spawn a fresh Claude Code session via sub-C with tool profile│
│   `:ingestion_minimal`. Fan-out is bounded by                   │
│   `Mnemosyne.Ingestion.Stage3Supervisor` (Task.Supervisor with  │
│   max_children = config :ingestion :stage3_concurrency).        │
│   Session context contains ONLY:                                │
│     - the one %Section{}                                        │
│     - the one %StoreSlice{}                                     │
│     - session_date, host_project, plan_qualified_id             │
│     - the per-section ingestion-op prompt                       │
│   Session returns a %ProposedOp{} parsed from the assistant     │
│   text via sub-C's tool-call boundary or sentinel parsing:     │
│     - op_type :: :new | :append_observation | :supersede |      │
│                  :no_op                                         │
│     - target_path :: nil | binary                              │
│     - title :: binary                                           │
│     - tags :: [binary]                                          │
│     - body :: binary                                            │
│     - nature :: :experiential | :speculative                   │
│     - rationale :: binary                                      │
│ Output: [%ProposedOp{}]                                         │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 4 — RECONCILIATION (1 fresh LLM session)                  │
│ Spawn ONE fresh session with a minimal context:                │
│   - the full list of %ProposedOp{} (titles + op_types only,    │
│     no full bodies, no store content)                          │
│   - the section titles they came from                           │
│   - the reconciliation prompt                                   │
│ Returns:                                                        │
│   - cross-section merges (two ops should become one)           │
│   - duplicates (two ops are the same knowledge)                │
│   - ordering constraints (op A must apply before op B)         │
│   - "no changes" (the common case — echo the list)             │
│ Output: [%ReconciledOp{}]                                       │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 5 — VALIDATE + DISPATCH (deterministic Elixir + actors)   │
│ For each %ReconciledOp{}:                                       │
│   1. Apply Rules 1–4 + 6 (§3) to produce a gated op            │
│   2. If the gated op triggers Rule 5 (graduation or            │
│      supersession), emit                                        │
│      %Mnemosyne.Event.Ingestion.PromptRequired{}               │
│      and await the user's resolution (apply/defer/reject) via  │
│      sub-M's bus + attached-client round trip                  │
│   3. Wrap the (possibly resolved) op as                        │
│      %Mnemosyne.Message.ExpertAbsorbCandidate{}                │
│      with a fresh `ingestion_event_id`                          │
│   4. Call Mnemosyne.Expert.ScopeMatcher.match_candidate/2:     │
│      - {:matched, [expert_id]} → fan out the candidate to each │
│        expert via Mnemosyne.Router.dispatch/2                   │
│      - :orphan → write directly to                              │
│        `<vault>/knowledge/uncategorized/<slug>.md` via          │
│        Mnemosyne.Ingestion.SafeFileWriter and emit             │
│        %Mnemosyne.Event.Ingestion.OrphanCandidate{}             │
│   5. Run the verdict collector (§4 below) for fan-out cases    │
│   6. Emit %Mnemosyne.Event.Ingestion.Applied{}                 │
│      / Deferred{} / Rejected{} per op disposition              │
│   At cycle end:                                                 │
│   7. Emit %Mnemosyne.Event.Ingestion.CycleSummary{}            │
└─────────────────────────────────────────────────────────────────┘
```

### LLM budget per cycle

For a typical reflect cycle producing ~3 sections, the per-cycle LLM session
count is `N + 1 + K` where `N` is the number of sections (Stage 3), `1` is
Stage 4 reconciliation, and `K` is the number of fan-out experts that
actually run a curation session in Stage 5 (typically 1–3 per matched
candidate; 0 for orphan candidates). Wall time is dominated by adapter spawn
cost × (N + 1 + K). The Stage 3 sessions parallelize naturally via
`Task.Supervisor`; Stage 5 expert curations parallelize via OTP mailboxes
across `ExpertActor` processes. Stage 4 stays serial (one session) but is
droppable as a degradation path.

### Key architectural properties

- **Fresh contexts by construction.** Stage 3 sessions each see one section
  and one small slice (typically 0–5 entries). Stage 4 sees only op metadata,
  not content. Each Stage 5 expert curation sees one candidate plus up to
  three adjacent entries from its own scope (sub-N §6.2). No single LLM
  session at any point holds the full plan memory, the full store, or
  multi-section reasoning. This is exactly the fresh-context goal (goal #2).
- **Deterministic/LLM boundary is crisp.** Stages 1, 2, and the Stage 5
  rule layer are pure Elixir — fast, testable, cacheable. Stages 3, 4, and
  the per-expert curation sessions inside Stage 5's fan-out are the only
  places the pipeline can return non-deterministic output, and each has a
  narrowly scoped job.
- **Graceful degradation.** Stage 4 reconciliation can be dropped as a
  degradation path (accepting cross-section incoherence risk) if LLM
  latency becomes a problem. The design does not *require* Stage 4 for
  correctness; it improves cross-section coherence.
- **OTP mailbox serialization replaces locks.** Each `ExpertActor` is a
  GenServer; its mailbox serializes incoming `%ExpertAbsorbCandidate{}`
  messages naturally. Multiple experts process in parallel across
  processes. Sub-D's collapsed scope (daemon singleton lock + external-tool
  advisory locks) covers the remaining concurrency surface.

### Files read by the pipeline

- **`<plan_dir>/memory.md`** — primary source. Stage 1 parses it into
  sections. The plan's memory.md is produced by the reflect-phase LLM as
  distilled, free-form prose. Sub-project E does not impose a format on
  memory.md; Stage 1's parsing is syntactic (section headings) and Stage 3's
  classification is semantic (LLM-driven).
- **`<plan_dir>/session-log.md`** — secondary source, for provenance.
  Stage 1 reads only the latest entry to extract `session_date` and any
  explicit project identification.
- **`<plan_dir>/backlog.md`** — **not read**. Task results are plan-process
  state, not knowledge.

Plan files are **read-only** to the ingestion pipeline. Invariant 6
(file-write safety) enforces this at the Elixir level via
`Mnemosyne.Ingestion.SafeFileWriter`.

### Trigger contract: `Mnemosyne.ReflectExitHook`

Sub-B §4.2 (rewritten for Elixir in Session 12) defines the
`Mnemosyne.ReflectExitHook` behaviour. Sub-E implements:

```elixir
@behaviour Mnemosyne.ReflectExitHook

@impl true
def on_reflect_exit(%{
      qualified_id: qualified_id,
      plan_dir: plan_dir,
      vault_runtime_dir: runtime_dir,
      session_log_latest_entry: latest_entry,
      ingestion_fired_setter: set_fired
    }) do
  Mnemosyne.Ingestion.Pipeline.run_cycle(%{
    qualified_id: qualified_id,
    plan_dir: plan_dir,
    runtime_dir: runtime_dir,
    latest_session_entry: latest_entry,
    on_stage5_start: set_fired
  })
end
```

Sub-B invokes the hook non-blockingly via `Task.Supervisor.start_child/3`
under the PlanActor's task supervisor, between persisting `plan-state.md`
and running sub-F's phase-exit hooks. Ingestion runs only on **clean reflect
exit** and only when the plan-state's `last-exit.ingestion-fired` flag is
`false`. Sub-E's pipeline calls the `set_fired` closure at the start of
Stage 5 to flip the flag; if Stage 5 never runs (e.g. Stage 1–4 fail), the
flag stays `false` and the next clean reflect exit re-attempts ingestion.

### Internal struct types

The pipeline's data types are Elixir structs with `@enforce_keys` and
`@type` typespecs declared in `lib/mnemosyne/ingestion/types.ex`:

```elixir
%Mnemosyne.Ingestion.Section{
  title :: String.t(),
  body_markdown :: String.t(),
  plan_qualified_id :: String.t(),
  session_date :: Date.t(),
  host_project :: String.t()
}

%Mnemosyne.Ingestion.StoreSlice{
  entries :: [%Mnemosyne.Knowledge.Entry{}]
}

%Mnemosyne.Ingestion.ProposedOp{
  op_type :: :new | :append_observation | :supersede | :no_op,
  target_path :: nil | String.t(),
  title :: String.t(),
  tags :: [String.t()],
  body :: String.t(),
  nature :: :experiential | :speculative,
  rationale :: String.t()
}

%Mnemosyne.Ingestion.ReconciledOp{
  proposed :: %ProposedOp{},
  merged_with :: [%ProposedOp{}],
  ordering_after :: [reference()]
}

%Mnemosyne.Ingestion.GatedOp{
  reconciled :: %ReconciledOp{},
  effective_confidence :: :prospective | :medium | :high,
  contradicts :: nil | String.t(),
  fallback_op :: nil | %ReconciledOp{},
  rule_5_event_class :: nil | :graduation | :supersession,
  axis :: String.t(),
  tier_target :: :tier1 | :tier2 | :tier1_then_tier2
}
```

The `%Mnemosyne.Message.ExpertAbsorbCandidate{}` and the
`%Mnemosyne.Event.Ingestion.*{}` events are owned by sub-N and sub-M
respectively; sub-E does not redefine them.

---

## 3. Rules and Invariants

Stage 5's deterministic prelude (steps 1 and 2 in the §2 diagram) enforces
six rules that make the auto-absorb path safe **before** any candidate is
dispatched to an expert. These rules live in pure Elixir modules under
`lib/mnemosyne/ingestion/rules/`, are unit-testable in isolation, and
override any conflicting LLM-proposed op. Experts are not allowed to relax
the rules — they may only add their own additional rejections downstream.

### Rule 1 — Confidence mapping

The per-section LLM's only confidence-related output is the `nature` field:
`:experiential` or `:speculative`. Mnemosyne derives the confidence level
deterministically according to this matrix:

| Pre-existing entry state | Section's `nature` | Result |
|--------------------------|--------------------|--------|
| No existing entry (new op) | `:experiential` | New entry at `:medium` |
| No existing entry (new op) | `:speculative` | New entry at `:prospective` |
| Existing `:prospective` | `:experiential` | Promoted to `:medium`, observation appended |
| Existing `:medium`, new origin is a *different project* | `:experiential` | **Triggers graduation path (Rule 4)** |
| Existing `:medium`, same project | `:experiential` | Stay `:medium`, append observation, add origin if new |
| Existing `:high` | any | Stay `:high`, append observation, add origin if new project |
| Any existing entry | `:speculative` | No confidence change; observation appended with a "speculative" marker |

Climbing to `:high` confidence within a single ingestion cycle is forbidden;
it requires multi-project evidence observed across at least two separate
cycles. This is the "validated across multiple contexts" rule enforced as a
machine invariant. Confidence never falls via ingestion — that is solely a
user action through an attached client's CRUD surface or a curate session.
Ingestion is monotonic upward or flat.

### Rule 2 — Contradiction gating

When a Stage 3 LLM proposes `:supersede`, Stage 5 accepts it only if:

```
proposed_op.effective_confidence >= target_entry.confidence
AND
proposed_op.origins strictly dominate target_entry.origins
  (at least as many distinct projects, OR strictly newer evidence
  in the same project after a significant gap)
```

If the rule fails, Stage 5 silently downgrades the op to
`:append_observation` with a `contradicts: <target_path>` marker. The
contradiction becomes visible inline in the target entry.

If the rule passes, Stage 5 still fires a **live UI prompt** (Rule 5)
because supersession is always a high-stakes transition — not because the
confidence rule forbids it, but because moving existing content into
`## Superseded` is a class of change the user must see before dispatch.

### Rule 3 — Axis derivation

Stage 5 computes the axis for new entries deterministically from the
op's `tags` field. The mapping is table-driven and lives in the daemon's
configuration (so new axes can be added without code changes):

```elixir
def derive_axis(tags), do: first_match(tags, axis_tag_rules())
```

The LLM never produces an axis field. Its job is to produce good tags;
Mnemosyne's job is to route them.

### Rule 4 — Tier routing

1. **New entries** always land in Tier 1 of the host project. Never
   directly in Tier 2.
2. **Appended observations** to an entry that exists in only one tier go
   to that tier.
3. **Appended observations** to an entry that exists in both tiers go to
   the Tier 2 version; the Tier 1 version is preserved as a frozen
   historical initial-observation record but is not further updated.
4. **Graduation** fires when Rule 1's "medium + new-project origin → should
   climb to high" case is reached. Stage 5 does **not** auto-write the
   graduation — instead it fires an interactive prompt (Rule 5) **before**
   dispatching the candidate. On user approval, the graduation is dispatched
   to the matching tier-2 experts as an `%ExpertAbsorbCandidate{}` carrying
   the accumulated observations and the post-graduation `:high` confidence;
   the Tier 1 entry remains in place, frozen, with a pointer comment.
5. **Stage 2 retrieval** considers Tier 2 first, then falls back to Tier 1,
   so the pipeline naturally targets the canonical post-graduation version
   when one exists.

Note that **Rule 4 still produces a `tier_target` field on the `%GatedOp{}`,
but Stage 5 does not honour it via direct writes for the Tier 2 case.**
Instead the field becomes an input to scope matching and to the candidate
metadata that experts see — experts use it to decide which of their own
declared scope directories to write into. The orphan path is the only place
sub-E itself writes to the knowledge store; everywhere else the write is
delegated.

### Rule 5 — Interactive UI events

Exactly two event classes interrupt the running UI with a live prompt
**before dispatch**:

- **Graduation** — an entry is about to be dispatched to Tier 2 experts at
  `:high` confidence because a new-project origin was observed.
- **Supersession** — a `:supersede` op passed Rule 2's confidence gating and
  is about to dispatch a candidate that will move existing content into
  `## Superseded` once any expert absorbs it.

Each prompt offers **four** actions: `[apply] [defer] [reject] [research]`.

- **`apply`** — Stage 5 proceeds to dispatch the op as gated. The collector
  records the user-approved disposition.
- **`defer`** — Stage 5 substitutes the safe non-destructive fallback
  (append-coexist for supersessions, stay-in-Tier-1-only for graduations)
  and dispatches the fallback instead. The explorer records "deferred,
  review later." The decision is **not** re-prompted automatically; the
  user revisits via the explorer's pending panel.
- **`reject`** — Stage 5 discards the op entirely. No dispatch, no fallback.
  The explorer records the rejection with the op's rationale.
- **`research`** — Stage 5 continues to await resolution; a fresh research
  LLM session is spawned (see §4 "Research sessions") and streams advisory
  output through sub-M's bus to the attached client alongside the prompt.
  The user can launch multiple research sessions on the same prompt; each
  is a fresh context.

If the prompt times out (no client attached, or no client interaction
within the configured `prompt_resolution_timeout`), the system falls back
to the safe non-destructive path. A timed-out graduation leaves the Tier 1
entry as-is; a timed-out supersession becomes an append-coexist. Both are
visible in the explorer for later resolution.

Every other op class — new entry at `:medium`, new entry at `:prospective`,
append-observation with or without `contradicts` marker, origin-added-to-
existing — proceeds silently to dispatch and lands in the explorer as an
informational record.

### Rule 6 — File-write safety invariants

`Mnemosyne.Ingestion.SafeFileWriter` will never:

- Write a file outside `<vault>/knowledge/uncategorized/` (the only
  sub-E-owned write target post-amendment).
- Overwrite an existing file. Orphan-path writes are creates only; if a
  slug collides, the writer suffixes a counter and surfaces a
  `%Mnemosyne.Event.Diagnostic{level: :warn}` event.
- Touch plan files (`<plan_dir>/memory.md`, `session-log.md`, `backlog.md`).
  Plan files are read-only to the ingestion pipeline.
- Write outside the vault root (`<vault>` is captured at boot time and
  asserted on every write via `Path.safe_relative_to/2`).

Any invariant violation aborts the pipeline cycle with
`%Mnemosyne.Event.Error{}` and a `%Mnemosyne.Event.Ingestion.CycleSummary{}`
showing the failure. The plan cycle itself (work/reflect/triage) is **not
blocked** by an ingestion failure — ingestion is best-effort from the
phase cycle's perspective per the `ReflectExitHook` non-blocking contract.

Tier 2 file-write safety is **enforced by sub-N's `ExpertActor` write path**,
not by sub-E. Each `ExpertActor` resolves its absorb-write target from the
first entry of its declared `scope.tier2` glob (sub-N §6.2) and does an
atomic temp-then-rename inside that directory. Sub-E's `SafeFileWriter`
does not need to validate Tier 2 paths because sub-E never writes to
Tier 2 directly post-amendment.

---

## 4. Event Flow and the Ingestion Events Explorer

Rule 5 introduces two kinds of ingestion output that flow into Mnemosyne's
attached clients: **live interactive prompts** (blocking Stage 5 dispatch
for a specific op) and **informational records** (non-blocking, flowing
into the Ingestion Events Explorer view). §4 specifies how those flow
through sub-M's observability bus without coupling the ingestion pipeline
to client implementation details.

### Event types — consuming sub-M's sealed set

Sub-E does **not** define its own event enum. All ingestion events are
members of the sealed `Mnemosyne.Event.*` set declared in sub-M §4.1 and
emitted via `Mnemosyne.Observability.emit/1`. The relevant variants:

```elixir
%Mnemosyne.Event.Ingestion.Applied{candidate_id, store_path, at}
%Mnemosyne.Event.Ingestion.PromptRequired{candidate_id, reason, at}
%Mnemosyne.Event.Ingestion.Deferred{candidate_id, reason, at}
%Mnemosyne.Event.Ingestion.Rejected{candidate_id, reason, at}
%Mnemosyne.Event.Ingestion.ResearchSession{candidate_id, session_id, at}
%Mnemosyne.Event.Ingestion.CycleSummary{cycle_id, applied_count,
                                         deferred_count, rejected_count,
                                         duration_ms, at}
```

Sub-E owns one additional variant in its namespace, declared in sub-M's
sealed set as part of the F amendment absorption:

```elixir
%Mnemosyne.Event.Ingestion.OrphanCandidate{candidate_id, written_path,
                                           candidate_tags, at}
```

Expert verdict events are owned by sub-N and consumed by sub-E's collector
(see §4 "Verdict collector" below):

```elixir
%Mnemosyne.Event.Expert.AbsorbRequested{expert_id, ingestion_event_id, ...}
%Mnemosyne.Event.Expert.Absorb{expert_id, ingestion_event_id, written_path, at}
%Mnemosyne.Event.Expert.Reject{expert_id, ingestion_event_id, reason, at}
%Mnemosyne.Event.Expert.CrossLinkSuggested{from_expert, to_expert, ...}
%Mnemosyne.Event.Expert.CrossLinkDeadEnd{chain, ingestion_event_id, at}
%Mnemosyne.Event.Expert.Conflict{ingestion_event_id, absorbing, rejecting, at}
%Mnemosyne.Event.Expert.IngestionVerdictTimeout{expert_id, ingestion_event_id, at}
```

`PromptRequired` is a **request-response pattern over the observability
bus**: the event carries a `candidate_id` and a `reason`; the corresponding
resolution arrives back via a dedicated client→daemon command on sub-K's
NDJSON protocol (`{"command": "resolve_ingestion_prompt", "candidate_id": ...,
"resolution": "apply" | "defer" | "reject"}`). Sub-E's Stage 5 holds a
`Map<candidate_id, GenServer.from()>` for outstanding prompts and matches
the resolution against it. There is no oneshot channel embedded in the
event struct (which is now an emitted, possibly logged, value rather than
an ephemeral message).

### Pipeline flow

1. Stage 5 processes `%ReconciledOp{}`s sequentially within a cycle —
   sequential ordering simplifies the rule application and prompt logic —
   but the dispatch fan-out for each op runs in parallel via OTP messaging.
   The phase cycle itself is not blocked by ingestion.
2. For non-prompt ops (the common case), Stage 5 runs Rules 1–4 + 6 to
   build a `%GatedOp{}`, calls `ScopeMatcher.match_candidate/2`, and
   either dispatches to experts (matched case) or writes to
   `<vault>/knowledge/uncategorized/` (orphan case).
3. For prompt ops, Stage 5 emits `%Ingestion.PromptRequired{}`, parks a
   `GenServer.from()` in the outstanding-prompts map, and awaits the
   resolution. Other ops in the same cycle continue in parallel via the
   GenStage consumer, so one slow user response doesn't block unrelated
   dispatches.
4. After all dispatches return verdicts (or time out per the verdict
   collector), Stage 5 emits per-op `Applied{}`/`Deferred{}`/`Rejected{}`
   events plus a single `CycleSummary{}` and exits.

Sub-K's attached clients subscribe to sub-M's bus, pattern-match on
variants, and render:

- `%Ingestion.Applied{}` → append row to the explorer's chronological feed
- `%Ingestion.PromptRequired{}` → open a modal with the candidate diff
  preview and the four action buttons
- `%Ingestion.ResearchSession{}` → stream advisory output into a side
  panel (the streaming output itself is `%Ingestion.ResearchSession{}`
  events tagged with the parent `candidate_id`)
- `%Ingestion.Deferred{}` / `%Ingestion.Rejected{}` → append to the
  explorer with a badge
- `%Ingestion.OrphanCandidate{}` → append to the explorer's
  "uncategorized" filter
- `%Ingestion.CycleSummary{}` → toast or status-bar summary

### Verdict collector

Per sub-N §6.3, sub-E's Stage 5 collector is responsible for:

- **Waiting for all dispatched verdicts** in parallel, bounded by a
  collector timeout. Sub-N suggests `turn_timeout_seconds + 30s = 5.5 min`
  as the default. Timeouts mark the missing verdict as
  `%ExpertIngestionVerdictTimeout{}` (sub-N's event) and proceed with
  whatever landed.
- **Contentful disagreement detection**: if at least one `:absorb` landed
  AND at least one `:reject` with a non-trivial reason (reason text ≥ 10
  characters, not just "out of scope"), emit
  `%Mnemosyne.Event.Expert.Conflict{}`. This is a surfaced signal, not an
  auto-resolution — humans decide what to do with it. Both writes already
  happened; the conflict event is advisory.
- **Handling second-round cross-links**: when a verdict is
  `:cross_link_suggested`, the collector issues one follow-up
  `%ExpertAbsorbCandidate{}` to the named expert. **Non-recursive,
  max depth 2** — if the second-round expert also emits `READY CROSS_LINK`,
  the collector ignores the suggestion, emits
  `%Mnemosyne.Event.Expert.CrossLinkDeadEnd{}`, and does not spawn a third
  round. This prevents infinite cross-link loops.
- **Orphan handling**: if `ScopeMatcher.match_candidate/2` returned
  `:orphan`, Stage 5 writes the candidate directly to
  `<vault>/knowledge/uncategorized/<proposed-slug>.md` via the SafeFileWriter
  and emits `%Mnemosyne.Event.Ingestion.OrphanCandidate{}`. **No expert is
  involved in the orphan path.** This is the only post-amendment case where
  sub-E itself writes a knowledge file.
- **Multi-expert absorption** is accepted: physical duplication across two
  expert directories produces two files keyed by the same
  `ingestion_event_id` in their provenance frontmatter. Wikilinks between
  duplicates are NOT auto-inserted by sub-N v1 (see sub-N §6.4).

### Research sessions

Clicking `[research]` on an `%Ingestion.PromptRequired{}` modal does not
resolve the prompt. The attached client sends a
`{"command": "spawn_ingestion_research", "candidate_id": ...}` over the
NDJSON protocol; the daemon spawns a fresh research session via sub-C with
the `:research_broad` tool profile, streams its output through sub-M's
bus tagged as `%Ingestion.ResearchSession{}` events, and the client renders
them in a side panel. The `[apply/defer/reject]` buttons remain active
throughout — research is advisory.

Research session inputs:

- The parent `%ReconciledOp{}` (full content)
- The target entry being superseded or graduated (full content)
- The originating `%Section{}` from Stage 1 (full body)
- The full `<plan_dir>/memory.md` (for surrounding reflect-phase context)
- The `<plan_dir>/session-log.md` latest entry (for provenance)
- A research prompt framing the session's role as advisory, not executive

| Tool | Stages 3/4 (`:ingestion_minimal`) | Research sessions (`:research_broad`) |
|------|----------------------------------|---------------------------------------|
| File read (scoped)            | yes, narrow | yes, full |
| File write                    | **no**      | **no**    |
| Shell execution               | no          | yes (read-only commands) |
| Web search                    | no          | **yes**   |
| Knowledge store query         | no          | **yes**   |
| Harness-spawn (recursive)     | no          | no        |
| `ask_expert` (sub-C §4.5)     | no          | yes (advisory) |

The "no file write" invariant is enforced by sub-C's tool-call boundary at
runtime via the configurable tool profiles, not by prompt discipline. A
research session that attempts a write has the tool call rejected at the
boundary, the session sees an error in its tool result, and the rejection
is logged as a `%Mnemosyne.Event.HarnessError{kind: :tool_call_boundary_error}`
event.

- Research sessions are bound to the lifetime of their parent prompt. If
  the prompt is resolved (apply/defer/reject), any still-streaming research
  sessions for that prompt are cancelled via sub-C's session
  process-group termination.
- Multiple concurrent research sessions per prompt are allowed; each is a
  fresh context.
- Research session transcripts are persisted alongside other harness
  transcripts at `<vault>/runtime/transcripts/<session_id>.jsonl` (sub-C's
  ownership, sub-M §4.1 references). The
  `%Ingestion.ResearchSession{candidate_id, session_id}` event ties the
  transcript to the parent candidate so the explorer can later show *what
  advice was consulted* when the decision was made.

### The Ingestion Events Explorer

The explorer is **a view rendered by sub-K's attached clients over sub-M's
event log**, not a separate data surface owned by sub-E. Sub-E owns only
the ingestion-event variants in sub-M's sealed set; sub-M owns the event
log and its persistence.

Persistence layout (owned by sub-M, surfaced here for reference):

```
<vault>/runtime/events/
  YYYY-MM-DD.jsonl             # daily-rotated append-only event log
                               # contains all %Mnemosyne.Event.*{} variants
  index.sqlite                 # derivable cross-view index, rebuildable
```

- The jsonl files are the **source of truth**. They are append-only and
  human-readable.
- The SQLite index is a **derivable cache** for fast cross-view queries
  (filter by plan, project, op type, event class, date range). It can be
  rebuilt at any time from the jsonl log.

Ingestion-explorer features that sub-K clients render:

- Chronological feed of applied/deferred/rejected/orphan events
- Filter by cycle, plan, host project, op type, event class, source
- Search across rationales and section titles
- Click-through to the affected entry
- **Re-opening deferred prompts** — clicking a deferred record re-fires the
  modal with the same op, diff preview, and target entry. Stage 5 stores
  the full `%ReconciledOp{}` plus the computed diff preview on deferral
  (in the `Deferred{}` event metadata) so re-opening does not require
  re-running the pipeline.
- **Viewing research transcripts** alongside the events that launched them

Every emitted ingestion event must include at minimum: `candidate_id`,
`cycle_id`, `plan_qualified_id`, `host_project`, `section_title`, `op_type`,
`target_path` (when applicable), `confidence_before`, `confidence_after`,
`nature`, `rationale`, `event_class` (if prompted), `user_resolution` (if
prompted), `research_session_ids` (if any were launched), `source`
(`{:llm, session_id}` or `{:human, user_action_id}`), `at` (UTC timestamp).

### Decoupling from client implementation

Stage 5 knows only about `Mnemosyne.Observability.emit/1` and the
outstanding-prompts map keyed by `candidate_id`. It does not know whether
the attached client is the Rust TUI, an Obsidian plugin, a web frontend,
or a test harness. This is critical for two reasons:

1. **Sub-project E can ship ahead of any client UI chrome.** A simple
   in-process test handler that synthesizes prompt resolutions deterministically
   is sufficient to run end-to-end ingestion tests with no client attached.
2. **The client layer is the responsibility of separate sub-projects** (the
   Rust TUI ships in v1 alongside the daemon; sub-K's Obsidian plugin in
   v1.5+; web frontend in v2+). Sub-E must not constrain those decisions.

### Human-driven ingestion (goal #6)

The pipeline exposes direct entry points for the user to drive ingestion
manually, without a harness child session triggering it:

1. **Human-triggered full ingestion.** "Run ingestion on this plan now" —
   `Mnemosyne.Ingestion.Pipeline.run_cycle/1` is invocable on demand
   against any plan directory via a sub-K client command. All downstream
   stages run as normal. Useful when the user has edited `memory.md`
   manually and wants it absorbed.
2. **Review-first mode (`ingestion.mode = :review_first`).** A user
   preference in `daemon.toml` that inserts a gate after Stage 4: Stage 5
   halts and emits a special review-required event surfacing **all**
   `%GatedOp{}`s for the cycle, grouped and edit-capable. The user can
   approve, edit, or reject each op individually before Stage 5 dispatches
   anything. Same pipeline, same invariants — a gate, not a different code
   path.
3. **Direct CRUD via attached clients.** The Stage 5 dispatch entry point
   is exposed as a client command (`{"command": "ingest_candidate",
   "candidate": {...}}`) so a human can hand-craft a candidate and feed it
   through the same dispatch + scope-matching path that LLM-driven Stage 5
   uses. Bypass writes (e.g., direct edits to a knowledge file in Obsidian)
   are **not** ingestion — they're editor writes that sub-D's external-tool
   coordination handles separately. A human-driven dispatch is
   indistinguishable at the invariant level from an LLM-driven dispatch;
   only the `source` field on the event differs (`{:human, user_action_id}`
   vs `{:llm, session_id}`).

### Testability

- **Stages 1, 2, and the Stage 5 rule layer** are unit-testable in pure
  Elixir with no LLM calls. Synthetic `%Section{}`s, `%StoreSlice{}`s, and
  `%ReconciledOp{}`s drive the rules in isolation. Every rule in §3 has a
  dedicated unit test: confidence mapping matrix, contradiction gating
  decisions, axis derivation, tier routing, file-write safety. Property
  tests via `StreamData` cover the matrix invariants
  (never-falls-via-ingestion, never-climbs-to-high-in-one-cycle).
- **Stages 3 and 4** are integration-tested against sub-C's
  `FixtureReplay` adapter — each per-section or reconciliation session is
  replayed from a captured JSON fixture instead of calling Claude. Sub-C's
  fixture-replay capability is now a committed v1 deliverable per its
  rewritten design doc, so sub-E does not need to ship its own stub
  adapter.
- **Stage 5 expert dispatch** is integration-tested against a stub
  `ExpertActor` registered in `Mnemosyne.ActorSupervisor` under a test
  qualified ID. The stub returns canned verdicts (`:absorb`, `:reject`,
  `:cross_link_suggested`) per test scenario. The full sub-N `ExpertActor`
  is *also* integration-testable against sub-N's own test fixtures, but
  sub-E's tests stub it to keep the pipeline tests independent of sub-N's
  internals.
- **Full pipeline** is end-to-end testable against a small in-repo fixture
  plan (memory.md + session-log.md) with a fixture knowledge store, a
  fixture adapter, and stub experts — no live LLM, no real filesystem
  outside a tempdir, no real daemon socket.
- **Live gated tests** run against a real Claude Code instance and real
  `ExpertActor`s in a manual/CI-gated mode (`@moduletag :live`), producing
  real non-deterministic output validated only against structural
  invariants (shape of ops, no safety violations, cycle completes,
  prompts resolved by a scripted client). Not run on every commit.

---

## 5. Open Questions, Cross-Sub-Project Requirements, and Risks

### Open questions within sub-project E

Implementation-level questions to be resolved during the build, not during
brainstorming. Decision cost is low enough to leave to the code.

1. **Stage 2 tag extraction algorithm.** Concrete algorithm for extracting
   candidate tags from section title + body. Safe default: stop-word-filtered
   lowercase nouns from the section title, intersected with the current tag
   vocabulary, no body scanning. Reconciliation catches missed entries if
   the heuristic is too narrow. Sub-R (knowledge ontology, v1.5+) will
   eventually provide a richer tag-vocabulary surface; for v1 the heuristic
   reads the flat tag set under `<vault>/experts/*.md` `tags:` fields.
2. **Stage 3 per-section prompt text.** Drafted during implementation,
   iterated against recorded fixtures.
3. **Stage 4 reconciliation prompt text.** Same.
4. **Research session prompt text.** Same.
5. **"Different project" detection for Rule 1.** Requires a canonical
   notion of project identity. Safe default: project directory name under
   the vault's `<vault>/projects/` symlink directory. Configurable via
   per-vault override in `daemon.toml` if needed.
6. **Tier 1 frozen-entry pointer comment format.** Trivial; decided during
   implementation.
7. **Stage 5 candidate slug derivation for orphan path.** Default:
   kebab-case the section title plus a 6-character ULID suffix to avoid
   collisions. The collision-suffix path in Rule 6 still applies as a
   defence in depth.
8. **GenStage vs Broadway for pipeline composition.** Default: GenStage
   producer-consumer chain rooted at `Mnemosyne.Ingestion.Pipeline`.
   Switch to Broadway if back-pressure or batching pressure justifies it
   during implementation. The decision is internal to sub-E and does not
   affect any cross-sub-project contract.

### Deferred items — explicitly out of scope for E

1. **Attached-client UI chrome** — sub-K and later. Sub-E is decoupled via
   sub-M's observability bus and the NDJSON command surface.
2. **Curate command redesign** — folded into sub-H's TUI-actions scope.
3. **Tier 2 cross-machine synchronisation** — sub-A (already addressed via
   git-backed vault) and sub-P (team mode, v2+).
4. **Horizon-scanning-driven ingestion** — separate path, not addressed
   here.
5. **Migration of existing Mnemosyne v0.1.0 entries** — sub-project G.
6. **Vector retrieval for Stage 2 or Stage 3 store slicing** — sub-Q
   (vector-store infrastructure, v1.5+). Sub-E v1 uses tag-overlap
   retrieval only.
7. **Tag ontology for richer scope matching** — sub-R (knowledge ontology,
   v1.5+). Sub-E v1 uses the same exact-string matching as sub-N.

### Cross-sub-project requirements

E generates the following concrete requirements on sibling sub-projects.
These must be recorded in each sibling's `memory.md` when their brainstorms
run.

#### Sub-B (phase cycle)

- Implements the `Mnemosyne.ReflectExitHook` behaviour callback by
  delegating to `Mnemosyne.Ingestion.Pipeline.run_cycle/1`.
- Sub-B invokes the hook non-blockingly via `Task.Supervisor.start_child/3`
  under the PlanActor's task supervisor, between persisting `plan-state.md`
  and running sub-F's phase-exit hooks. Only on clean reflect exit, only
  when `last-exit.ingestion-fired` is `false`.
- Phases must support both LLM-driven and human-driven execution paths
  (sub-B's `LlmHarnessExecutor` + `ManualEditorExecutor`). The
  human-driven path still triggers ingestion via the same hook.
- The PlanActor must be pausable and user-takeoverable. A running LLM
  phase can be interrupted and the user can take over the remaining work
  manually. On takeover, the harness session terminates cleanly via
  sub-C's process-group termination, its partial output is captured, and
  the user resumes from the same input state.

#### Sub-C (harness adapter)

- **Configurable tool profiles** at spawn time. Minimum set:
  `:ingestion_minimal` (no tools), `:research_broad` (file read, shell
  read-only, web search, knowledge store query, `ask_expert`). Runtime
  tool enforcement (rejecting disallowed tool invocations at the §4.5
  tool-call boundary) is sub-C's responsibility, not sub-E's.
- **Fixture-replay mode.** Adapter accepts a recorded response instead of
  making a live call, enabling deterministic end-to-end ingestion tests
  with no live LLM. Now committed in sub-C's rewritten design doc.
- **Cheap session spawn.** Stage 3 fires N sessions per cycle and latency
  × N is directly visible to the user. Target: < 3 seconds per cold
  spawn keeps a 5-section cycle under 20 seconds. Sub-C's
  `SpawnLatencyReport` instrumentation surfaces this measurement.
- **Streaming output support** for research sessions — advisory text
  streams through sub-M's bus live, not batched at session end.
- **Process-group termination** so research sessions can be cancelled when
  their parent prompt resolves.

#### Sub-A (vault layout)

- Tier 1 (`<project>/mnemosyne/knowledge/`) and Tier 2
  (`<vault>/knowledge/`) roots are addressable independently and are
  resolved at daemon boot via sub-A's `Mnemosyne.Vault` module.
- The vault init flow must create `<vault>/knowledge/uncategorized/` as
  part of the initial directory structure (orphan-candidate target).
  This is recorded in sub-A's §A6 init flow.

#### Sub-D (concurrency, post-F collapsed scope)

- Sub-D owns the daemon singleton lock and external-tool advisory locks.
  Sub-E does not need its own write lock for daemon-internal ingestion
  writes — OTP mailbox serialization handles intra-daemon coordination.
- External-tool conflict (e.g., a user editing
  `<vault>/knowledge/uncategorized/foo.md` in Obsidian while sub-E is
  writing it) is sub-D's domain. Sub-E's `SafeFileWriter` does an atomic
  temp-then-rename and surfaces collisions as
  `%Mnemosyne.Event.Diagnostic{level: :warn}`.

#### Sub-F (hierarchy, router, actor model)

- `Mnemosyne.Router.dispatch/2` accepts
  `%Mnemosyne.Message.ExpertAbsorbCandidate{}` as a routable message.
  Sub-F's router emits a `%Mnemosyne.Event.MessageRouted{message_kind:
  :dispatch, via: :direct, to_actor: "expert:" <> id}` for each fan-out
  destination.
- `expert:<id>` is a valid qualified ID (sub-F's qualified-ID scheme).
- Stage 5's outstanding-prompts map is owned by sub-E's pipeline GenServer
  — the router does not need to know about prompt resolution semantics.

#### Sub-N (domain experts)

- `Mnemosyne.Expert.ScopeMatcher.match_candidate/2` is the scope-matching
  entry point sub-E calls, with the contract from sub-N §6.1:

  ```elixir
  @spec match_candidate(
    candidate_tags :: [String.t()],
    vault_experts :: [Mnemosyne.Expert.Declaration.t()]
  ) :: {:matched, [String.t()]} | :orphan
  ```

- `%Mnemosyne.Message.ExpertAbsorbCandidate{}` is the dispatch message
  shape. Sub-E populates a fresh `ingestion_event_id` on each candidate.
  Multi-expert absorption produces multiple physical files keyed by the
  matching `ingestion_event_id`.
- The three verdict variants (`:absorb`, `:reject`, `:cross_link_suggested`)
  are the return contract from `ExpertActor` to sub-E's collector.
- Sub-E's collector handles `%ExpertCrossLinkDeadEnd{}` and
  `%ExpertConflict{}` per sub-N §6.3.
- Sub-E's amendment **does not block on sub-N's full implementation** —
  the early-deliverable PR (sub-N Task 15) ships `ScopeMatcher` +
  message struct + verdict + event structs, which is the full surface
  sub-E codes against. Sub-N's later implementation tasks proceed in
  parallel.

#### Sub-M (observability framework)

- Sub-E emits all events via `Mnemosyne.Observability.emit/1`. The six
  `%Mnemosyne.Event.Ingestion.*{}` variants from sub-M §4.1 plus
  `%Mnemosyne.Event.Ingestion.OrphanCandidate{}` are sub-E's full event
  vocabulary.
- Sub-M's bus carries the prompt-resolution back-channel data via
  attached-client commands (sub-K NDJSON protocol); sub-E's pipeline
  GenServer holds the outstanding-prompts map.
- Sub-M's `:telemetry` handler `try/rescue` discipline (§5.2) applies to
  every sub-E event handler.
- The "parallel-emit" transition from sub-M's adoption matrix is
  **resolved by this amendment**: sub-E no longer has a standalone
  `IngestionEvent` channel to parallel-emit alongside. Sub-E emits
  directly to sub-M's bus from Stage 5; the transition window collapses
  to zero, and sub-M's adoption task for sub-E becomes "verify sub-E's
  emission against the §4.1 schema" rather than "wrap and migrate."

#### Sub-H (TUI actions / skill fold)

- Every existing Mnemosyne Claude Code skill that becomes a phase/command
  must have a human-driven attached-client equivalent. The seven skills
  (`/begin-work`, `/reflect`, `/setup-knowledge`, `/create-plan`,
  `/curate-global`, `/promote-global`, `/explore-knowledge`) are currently
  LLM-driven. Sub-H's design must specify both the LLM-driven form (as
  phase prompt or daemon command) and the human-driven form (as TUI
  attached-client action).

### New backlog candidates

Two new sub-projects surfaced during the original 2026-04-12 brainstorm
that have since been added to the orchestrator backlog:

**Sub-project I — Obsidian coverage document.** Re-scoped per sub-F's
amendment to "document which Obsidian features cover which Mnemosyne data
surfaces" rather than build a unified explorer framework. The Ingestion
Events Explorer becomes one Obsidian view (Dataview query over sub-M's
event log, surfaced via sub-K's plugin client), not a sub-E-owned widget.

**Sub-project H — Skills as TUI attached-client actions.** Re-scoped per
sub-F's amendment to "TUI commands routed through the daemon NDJSON
protocol" rather than standalone CLI subcommands. Possibly subsumes the
manual-phase-affordances surface absorbed into sub-B.

### Risks

Five substantive risks under the BEAM/actor architecture.

1. **LLM-call latency dominates cycle wall-time.** N+1 fresh sessions per
   cycle plus K expert curation sessions × cold-spawn latency could make
   ingestion noticeably slow (tens of seconds on a busy cycle). Mitigation:
   sub-C's `SpawnLatencyReport` instrumentation measures cold spawn early
   on realistic cycles; if real, sub-C can prioritise warm session reuse
   or pooling. Backstop: drop Stage 4 as a degradation path and accept
   cross-section incoherence risk in exchange for one fewer session per
   cycle. Stage 5's expert curation sessions parallelize naturally across
   `ExpertActor` processes via OTP mailboxes, so wall-time scales with
   `max(K experts × per-curation latency)` not `K × latency`.
2. **LLM classification variance producing unstable confidence outcomes.**
   Two sessions processing the same section could yield different
   `:experiential`/`:speculative` classifications, producing different
   confidence outcomes across runs. Mitigation: classification is a
   narrow, fresh-context question (low variance expected), and Rule 1's
   invariants are monotonic — variance only affects initial confidence of
   brand-new entries, which the user can correct via direct CRUD edit
   through an attached client. Worth watching but not a blocker.
3. **Silent ingestion of misread observations.** Auto-absorb means that if
   Stage 3 misreads a section as experiential when it was speculative, a
   `:medium`-confidence entry enters Tier 1 (or an expert's Tier 2) without
   user notice. The explorer and the Rule 1 ceiling (single session can
   never produce `:high`) both bound the damage. Mitigation: sub-K's
   client must badge "unreviewed ingestion events" prominently so the
   explorer doesn't become an invisible graveyard. Experts may also
   reject post-rule downstream, providing a second filter layer.
4. **OTP mailbox backpressure under burst dispatch.** A cycle producing
   30 candidates dispatched to a popular expert (e.g., `rust-expert`)
   queues 30 `%ExpertAbsorbCandidate{}` messages to that expert's mailbox,
   each triggering a fresh-context curation session. Mitigation:
   `ExpertActor` processes its mailbox sequentially per sub-N §6.4, so
   the queue drains in expert-curation-latency order. The collector
   timeout (5.5 min default) bounds wait time and surfaces stragglers as
   `%ExpertIngestionVerdictTimeout{}`. If the burst pattern proves
   problematic, sub-N's `ExpertActor` can adopt a pool-of-sessions
   strategy without changing sub-E's contract.
5. **`:telemetry` handler crash blinding the bus.** Per sub-M §5.2,
   `:telemetry` silently detaches handlers that raise. Sub-E's emission
   path itself does not handle events (it only emits), so this risk
   surfaces in any sub-E test handler or attached-client subscriber.
   Mitigation: every sub-E test handler wraps its `handle_event/4` body
   in `try/rescue` and logs errors via `Logger` directly, per sub-M's
   project-wide discipline.
6. **UI surface area expansion from goal #6.** The co-equal-actors
   principle turns Mnemosyne into a full interactive application with
   CRUD across multiple data surfaces, pausable LLM sessions, manual
   reflection/triage editors, and review-first ingestion gating. Sub-E
   imposes this cost on sub-K and sub-H but does not bear it directly.
   Worth flagging to the v1-scope-cut decision: if attached-client scope
   becomes the critical path, review-first mode and some human-equivalent
   phase actions can be deferred to v1.5+. The minimum bar for the
   co-equal-actors principle to hold is "human can resolve an
   `Ingestion.PromptRequired` modal and run a manual ingestion via the
   Rust TUI" — that should not be deferred.

---

## Appendix A — Decision trail

Each numbered decision below was presented to the user during the
2026-04-12 brainstorm and approved at a decision point. Listed in the
order they were resolved. Q15 and Q16 are added by the 2026-04-16
amendment session.

1. **Auto-absorb, no staging.** Ingestion writes directly into the
   knowledge store (Tier 1 by default; Tier 2 via graduation, post-amendment
   via expert dispatch). Curation remains a separate manual session users
   invoke independently. Option A of Q1.
   *(Correction note, 2026-04-16: "writes" is now "dispatches to experts
   that write"; the auto-absorb-without-staging principle is preserved at
   the dispatch boundary.)*
2. **Fires after reflect only.** One ingestion firing per cycle, triggered
   by reflect-phase exit. Option A of Q2.
   *(Correction note, 2026-04-16: trigger is now sub-B's
   `Mnemosyne.ReflectExitHook` callback, invoked non-blockingly under a
   `Task.Supervisor` in the PlanActor.)*
3. **Free-form `memory.md`, embedded Claude Code ingestion.** Plan memory
   format stays unconstrained; Mnemosyne is itself an LLM client via its
   own harness adapter, using Claude Max subscription pricing rather than
   per-token metered API access. Option A of Q3.
4. **Fresh LLM context is a first-class architectural goal** (user
   clarification during Q3 refinement). Many short fresh sessions beat
   one long multi-purpose session. This principle is load-bearing for the
   rest of the design.
5. **Pipeline shape: parse → retrieve → N per-section ops → reconciliation
   → validate+apply.** Three deterministic stages, two LLM stages. Option
   D of Q4 with an explicit reconciliation step.
   *(Correction note, 2026-04-16: "validate+apply" is now "validate+
   dispatch"; Stage 5's writes are delegated to experts except for the
   orphan path.)*
6. **Contradiction handling: confidence-gated supersession with
   append-coexist fallback.** Option C of Q5.
7. **Confidence assignment: LLM classifies experiential/speculative;
   Mnemosyne applies deterministic mapping with monotonic accumulation
   rules.** Option D of Q6.
8. **Axis assignment: deterministic `derive_axis(tags)`.** LLM produces
   tags, Mnemosyne routes. Option (a) of Q7.
9. **Tier routing: Option Y with "Tier 2 wins after graduation"
   sub-rule.** Tier 1 always for new; auto-promotion candidate fires
   interactive prompt on the accumulation rule. Option Y of Q8.
   *(Correction note, 2026-04-16: graduation now fires the prompt
   pre-dispatch; on apply, the candidate goes to Tier 2 experts via the
   normal fan-out, not via a sub-E direct write.)*
10. **Continuously-running daemon hosting harness sessions** (refined from
    the original "continuously-running TUI" framing during Q8 review and
    finalised by sub-F's commitment in orchestrator Session 9 to the
    persistent BEAM daemon). Mnemosyne is a long-running BEAM application
    hosting plan and expert actors, with attached clients (Rust TUI in
    v1, Obsidian plugin in v1.5+) connecting over a Unix socket. The
    TUI vs Obsidian-vs-web decision is no longer in E's scope — sub-K and
    later own it.
11. **Interactive prompts on graduation and supersession only.** Three-
    action prompts extended to four with `[research]`. Defer = safe
    non-destructive fallback. Option C of Q9, extended.
    *(Correction note, 2026-04-16: the prompt fires pre-dispatch; defer
    substitutes the fallback before dispatch, not after a write.)*
12. **Research LLM sessions** launchable from any `PromptRequired` modal,
    advisory-only, fresh context, broader tool profile than ingestion
    sessions. User refinement during Section 4 review.
13. **Explorers are a first-class accountability substrate** (user
    clarification during Section 5 review). The review panel is one
    explorer in a family; every Mnemosyne data surface gets an explorer;
    explorers are full CRUD surfaces.
    *(Correction note, 2026-04-16: per the project-wide commitment to
    Obsidian as the v1 explorer, the explorer framework is delegated to
    Obsidian + sub-K rather than built as a Mnemosyne-owned widget. The
    "full CRUD surface" requirement holds, now satisfied by Obsidian's
    file CRUD plus sub-K's command-palette actions.)*
14. **Human and LLM are co-equal actors** (user clarification during
    Section 5 review). Every LLM-driven action has a human-driven
    equivalent through the same invariants. No workflow is LLM-only.
    New goal #6 added.
15. **BEAM/Elixir/OTP daemon pivot** (orchestrator Session 9, 2026-04-14;
    formalised in sub-F's design doc). Sub-E re-cast from Rust to Elixir
    inline in this amendment session. The pipeline becomes a `GenStage`
    composition; types become structs with `@enforce_keys`; the
    `IngestionEvent` enum is retired in favour of sub-M's sealed
    `Mnemosyne.Event.*` set. Rationale: the BEAM design absorbs every
    concurrency primitive sub-E was building by hand (lock discipline,
    parallel session fan-out, supervision, hot reload) and unifies the
    event surface with the rest of Mnemosyne's typed observability story.
    Inline rewrite per "amendments rewrite inline, not as supersede
    layers" — §1–§5 re-cast, no amendment block appended.
16. **Stage 5 becomes dispatch-to-experts** (sub-N Session 16,
    2026-04-15; absorbed in this amendment session). Stages 1–4 unchanged
    in logic. Stage 5's deterministic prelude (Rules 1–4 + 6 + Rule 5
    prompts) still runs and gates **what gets dispatched**, but the
    write itself is delegated to `ExpertActor`s via fan-out of
    `%Mnemosyne.Message.ExpertAbsorbCandidate{}` through sub-F's router.
    Three verdict variants (`:absorb`, `:reject`, `:cross_link_suggested`),
    non-recursive max-depth-2 cross-link follow-up, contentful-disagreement
    `%ExpertConflict{}`, and parallel-fan-out with each absorbing expert
    writing into its own scope directory. Orphan candidates (zero
    tag-matching experts) bypass the fan-out and write to
    `<vault>/knowledge/uncategorized/` via sub-E's `SafeFileWriter` —
    the only post-amendment case where sub-E itself writes a knowledge
    file. Multi-expert absorption produces physical duplication keyed by
    `ingestion_event_id`; wikilinks between duplicates are NOT
    auto-inserted in v1. Concrete contracts come from sub-N §6 and §9.4
    (`ScopeMatcher.match_candidate/2`, `%ExpertAbsorbCandidate{}`,
    verdict structs, event structs). Sub-E's amendment can run in
    parallel with sub-N's later implementation tasks because sub-N Task
    15 ships the full surface as an early-deliverable PR.

---
