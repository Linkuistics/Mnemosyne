# Sub-project E — Post-Session Knowledge Ingestion Model

**Status:** Design accepted 2026-04-12. Implementation plan TBD.
**Sub-project:** E (of the A–I set in the Mnemosyne orchestrator plan).
**Origin:** Brainstormed interactively on 2026-04-12 using the
`superpowers:brainstorming` skill. Every decision recorded here was presented
to the user and approved at a decision point during that session.

---

## Overview

Sub-project E designs how Mnemosyne — as the parent process hosting harness
child sessions — reads each plan's outputs after a session completes and
writes updates into Tier 1 and Tier 2 of the knowledge store.

The design is:

- A **five-stage pipeline**: three deterministic Rust stages (parse, retrieve,
  validate+apply) and two fresh-context LLM stages (per-section ops,
  reconciliation).
- Governed by **six deterministic invariants**: confidence mapping,
  contradiction gating, axis derivation, tier routing, interactive-event
  gating, and file-write safety.
- Firing **live interactive prompts** on two high-stakes event classes
  (Tier 2 graduation and supersession) in Mnemosyne's continuously-running
  UI, with an optional on-demand **research LLM** that can be launched from
  any prompt to investigate a decision before the user resolves it.
- Exposing every action to both **LLM and human actors on equal footing**.
  No ingestion workflow is LLM-only; every state change reachable via the
  pipeline is also reachable via direct user action through the UI, through
  the same invariants.
- Emitting **structured ingestion events** that back an Ingestion Events
  Explorer (one member of a family of Mnemosyne explorers designed in a
  separate future sub-project), providing the accountability substrate that
  makes auto-absorb tolerable.

The design assumes the parent-process inversion established in
`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`: Mnemosyne runs
continuously as a TUI/GUI application, spawns harness child sessions via a
harness adapter (sub-project C), and hosts interleaved LLM-driven and
human-driven work against shared state.

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
writes updates to Tier 1 and Tier 2 of the knowledge store: trigger timing,
file inputs, pipeline stages, decision rules, contradiction handling,
confidence assignment, axis assignment, tier routing, interactive event
flow, research sessions, event persistence, and the requirements this design
generates on sibling sub-projects.

### Deliberately out of scope

- **Harness child session spawning** — sub-project C. Sub-project E assumes C
  provides a capability to spawn a fresh Claude Code session with a scoped
  prompt, a configurable tool profile, and streaming output capture.
- **Phase cycle state machine** — sub-project B. E specifies only the
  hook point (after reflect exits) and the contract of reading plan files.
- **Physical location of the Tier 2 store** — sub-project A. E assumes the
  Tier 1 and Tier 2 roots are resolvable paths at startup.
- **Concurrency and locking for shared stores** — sub-project D. E specifies
  only the lock points it must acquire.
- **UI chrome (TUI vs. GUI)** — a future sub-project. E decouples the
  pipeline from the UI via an event channel, and a terminal stub is
  sufficient for testing.
- **Unified explorer framework for Mnemosyne data surfaces** — new backlog
  candidate "sub-project I" (see §5). E specifies only the ingestion-events
  data surface and generates requirements on the explorer framework.
- **Curate command redesign** — part of sub-project H or a future sub-project.
- **Horizon-scanning-driven ingestion** — E designs the cycle-driven
  ingestion path only.
- **Migration of existing Mnemosyne v0.1.0 entries** — sub-project G.

E designs greenfield. Preservation of existing Mnemosyne v0.1.0
implementation details is **not** a design constraint for this sub-project.
The non-disruption constraint from the seed memory applies to v0.1.0 *staying
running* during the build, not to how E reimagines ingestion internally.

### Goals, in priority order

1. **Eliminate the LLM-discipline failure mode for promotion.** No tool calls
   from child harnesses back to Mnemosyne. Parent reads files, parent decides,
   parent writes. The question "did the LLM remember to promote that?" must
   become structurally impossible.

2. **Maximise fresh LLM context.** Decompose ingestion into the smallest
   possible per-session LLM contexts, each with one conceptually scoped job.
   Never ask one LLM session to hold the full store, the full plan memory,
   and the full ingestion decision set at once. This is a load-bearing
   architectural goal, not an optimisation — it follows from the user's
   explicit requirement that fresh context is a first-class property of all
   LLM-using systems in this project.

3. **Preserve Mnemosyne's knowledge philosophy.** Auto-absorb must not
   silently corrupt the "living beliefs" semantic. Supersession, contradiction,
   and confidence inflation are gated by rules and surfaced for user review on
   high-stakes events.

4. **Auto-absorb by default, interactive on high-stakes transitions.**
   Low-stakes ingestion (new entries, appended observations, confidence-
   preserving updates) flows silently into the Ingestion Events Explorer.
   Graduation to Tier 2 and supersession of existing content fire live
   interactive prompts in Mnemosyne's running UI.

5. **Every ingestion event is persisted as a structured record** in a form
   that backs the Ingestion Events Explorer — chronological browsing, search,
   filter, re-opening deferred prompts, research session transcripts, and
   full audit history across LLM-driven and human-driven operations alike.

6. **Human and LLM are co-equal actors.** Every action the LLM is authorised
   to take in the ingestion pipeline must have a human-driven equivalent that
   produces the same state change through the same invariants. The LLM is a
   *delegatee* that the user can preempt, review-first, redo, supplement, or
   bypass entirely. No workflow may be LLM-only.

---

## 2. Pipeline Architecture

Ingestion is a five-stage pipeline that fires exactly once per cycle, triggered
when the reflect phase's child harness session exits. Three stages are
deterministic Rust; two are fresh-context LLM sessions spawned via the harness
adapter.

```
reflect session exits
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 1 — SECTION PARSE (deterministic)                         │
│ Input:  {{PLAN}}/memory.md, {{PLAN}}/session-log.md latest entry│
│ Output: Vec<Section> where each Section has:                    │
│          - title (from heading)                                 │
│          - body_markdown                                        │
│          - plan_path (provenance)                               │
│          - session_date (from session-log)                      │
│          - host_project                                         │
└─────────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 2 — RETRIEVAL (deterministic)                             │
│ For each Section:                                               │
│   - Extract candidate tags from the section title + body       │
│     via keyword/tag heuristics (fast, rough, deterministic)    │
│   - Query the knowledge store using those tags                  │
│   - Produce a StoreSlice: the small set of existing entries    │
│     whose tags overlap — typically 0–5 entries                  │
│ Output: Vec<(Section, StoreSlice)>                              │
└─────────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 3 — PER-SECTION LLM OPS (N fresh LLM sessions)            │
│ For each (Section, StoreSlice):                                 │
│   Spawn a fresh Claude Code session via the harness adapter    │
│   with tool profile `IngestionMinimal`.                         │
│   Session context contains ONLY:                                │
│     - the one Section (title + body)                            │
│     - the one StoreSlice (0–5 existing entries)                │
│     - the session_date, host_project, plan_path                │
│     - the ingestion-op prompt                                   │
│   Session returns a ProposedOp:                                 │
│     { op_type: new | append_observation | supersede | no_op,   │
│       target_path: Option<PathBuf>,                             │
│       title: String,                                            │
│       tags: Vec<Tag>,                                           │
│       body: String,                                             │
│       nature: experiential | speculative,                      │
│       rationale: String }                                       │
│ Output: Vec<ProposedOp>                                         │
└─────────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 4 — RECONCILIATION (1 fresh LLM session)                  │
│ Spawn ONE fresh Claude Code session with a minimal context:    │
│   - the full list of ProposedOps (titles + op_types only,     │
│     no full bodies, no store content)                          │
│   - the section titles they came from                           │
│   - the reconciliation prompt                                   │
│ Session identifies and returns:                                │
│   - cross-section merges (two ops should become one)           │
│   - duplicates (two ops are the same knowledge)                │
│   - ordering constraints (op A must apply before op B)         │
│   - "no changes" (the common case — echo the list)             │
│ Output: Vec<ReconciledOp>                                       │
└─────────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 5 — VALIDATE + APPLY (deterministic)                      │
│ For each ReconciledOp:                                          │
│   - Apply Mnemosyne invariants (§3)                             │
│   - If a high-stakes event is triggered (graduation or         │
│     supersession), emit a PromptRequired event and await       │
│     the user's resolution (apply/defer/reject)                  │
│   - Acquire the store write lock (sub-project D)               │
│   - Write to Tier 1 and/or Tier 2                               │
│   - Emit an Applied or Deferred or Rejected event to the       │
│     Ingestion Events Explorer                                   │
│   - Release the lock                                            │
└─────────────────────────────────────────────────────────────────┘
```

### LLM budget per cycle

For a typical reflect cycle producing ~3 sections, the per-cycle LLM budget is
**N + 1 = 4 fresh Claude Code sessions** (three per-section, one reconciliation).
Wall-time is dominated by adapter spawn cost × 4.

### Key architectural properties

- **Fresh contexts by construction.** Stage 3 sessions each see one section
  and one small slice (typically 0–5 entries). Stage 4 sees only op metadata,
  not content. No single LLM session at any point holds the full plan memory,
  the full store, or multi-section reasoning — this is exactly the fresh-
  context goal (goal #2).
- **Deterministic/LLM boundary is crisp.** Stages 1, 2, and 5 are pure Rust —
  fast, testable, cacheable. Stages 3 and 4 are the only places the pipeline
  can return non-deterministic output, and each has a narrowly scoped job.
- **Graceful degradation.** Stage 4 is a reconciliation step that can be
  dropped as a degradation path (accepting cross-section incoherence risk) if
  LLM latency becomes a problem. The design does not *require* Stage 4 for
  correctness; it improves cross-section coherence.

### Files read by the pipeline

- **`{{PLAN}}/memory.md`** — primary source. Stage 1 parses it into sections.
  The plan's memory.md is produced by the reflect-phase LLM as distilled,
  free-form prose. Sub-project E does not impose a format on memory.md;
  Stage 1's parsing is syntactic (section headings) and Stage 3's
  classification is semantic (LLM-driven).
- **`{{PLAN}}/session-log.md`** — secondary source, for provenance. Stage 1
  reads only the latest entry to extract `session_date` and any explicit
  project identification.
- **`{{PLAN}}/backlog.md`** — **not read**. Task results are plan-process
  state, not knowledge.

Plan files are **read-only** to the ingestion pipeline. Invariant 6 (file-write
safety) enforces this at the Rust level.

---

## 3. Rules and Invariants

Stage 5 enforces a small set of deterministic rules that make the auto-absorb
path safe. These rules live in Rust, are unit-testable in isolation, and
override any conflicting LLM-proposed op.

### Rule 1 — Confidence mapping

The per-section LLM's only confidence-related output is the `nature` field:
`experiential` or `speculative`. Mnemosyne derives the confidence level
deterministically according to this matrix:

| Pre-existing entry state | Section's `nature` | Result |
|--------------------------|--------------------|--------|
| No existing entry (new op) | `experiential` | New entry at `medium` |
| No existing entry (new op) | `speculative` | New entry at `prospective` |
| Existing `prospective` | `experiential` | Promoted to `medium`, observation appended |
| Existing `medium`, new origin is a *different project* | `experiential` | **Triggers graduation path (Rule 4)** |
| Existing `medium`, same project | `experiential` | Stay `medium`, append observation, add origin if new |
| Existing `high` | any | Stay `high`, append observation, add origin if new project |
| Any existing entry | `speculative` | No confidence change; observation appended with a "speculative" marker |

**Confidence never climbs via a single ingestion cycle.** Climbing to `high`
requires multi-project evidence observed across at least two separate cycles.
This is the "validated across multiple contexts" rule enforced as a machine
invariant.

**Confidence never falls via ingestion.** Demotion of existing entries is
solely a user action (through direct UI edit or through a curate session).
Ingestion is monotonic upward or flat.

### Rule 2 — Contradiction gating

When a Stage 3 LLM proposes `supersede`, Stage 5 accepts it only if:

```
proposed_op.effective_confidence >= target_entry.confidence
AND
proposed_op.origins strictly dominate target_entry.origins
    (at least as many distinct projects, OR strictly newer evidence
     in the same project after a significant gap)
```

If the rule fails, Stage 5 silently downgrades the op to `append_observation`
with a `contradicts: <target_path>` marker on the appended observation. The
contradiction becomes visible inline in the target entry and is picked up by
curate sessions and by the explorer.

If the rule passes, Stage 5 still fires a **live UI prompt** (Rule 5) because
supersession is always a high-stakes transition — not because the confidence
rule forbids it, but because moving existing content into `## Superseded` is
a class of change the user must see.

### Rule 3 — Axis derivation

Stage 5 computes the axis for new entries deterministically from the `tags`
field in the op. The mapping is table-driven and lives in Mnemosyne's
configuration (so new axes can be added without code changes):

```
axis = first_match(tags, axis_tag_rules)
       // fall back to `techniques/` if no rule matches
```

The LLM never produces an axis field. Its job is to produce good tags;
Mnemosyne's job is to route them.

### Rule 4 — Tier routing

1. **New entries** always land in Tier 1 of the host project. Never directly
   in Tier 2.
2. **Appended observations** to an entry that exists in only one tier go to
   that tier.
3. **Appended observations** to an entry that exists in both tiers go to
   **Tier 2 only**. Tier 2 is the canonical post-graduation copy; the Tier 1
   version is preserved as a frozen historical initial-observation record
   but is not further updated.
4. **Graduation** fires when Rule 1's "medium + new-project origin → should
   climb to high" case is reached. Stage 5 does **not** auto-write the
   graduation — instead it fires an interactive prompt (Rule 5). On user
   approval, Mnemosyne:
   - Copies the Tier 1 entry to Tier 2 (preserving its accumulated
     observations and all origins)
   - Updates the Tier 2 copy's confidence to `high`
   - Leaves the Tier 1 entry in place, frozen, with a pointer comment
     indicating graduation
5. **Ingestion retrieval** (Stage 2) considers Tier 2 first, then falls back
   to Tier 1, so the pipeline naturally targets the canonical post-graduation
   version when one exists.

### Rule 5 — Interactive UI events

Exactly two event classes interrupt the running UI with a live prompt:

- **Graduation** — an entry is about to be copied from Tier 1 to Tier 2 at
  `high` confidence because a new-project origin was observed.
- **Supersession** — a `supersede` op passed Rule 2's confidence gating and is
  about to move existing content into `## Superseded`.

Each prompt offers **four** actions: `[apply] [defer] [reject] [research]`.

- **`apply`** — Stage 5 writes the op as-is. The explorer records it as
  "user-approved, HH:MM".
- **`defer`** — Stage 5 downgrades the op to the safe non-destructive fallback
  (append-coexist for supersessions, stay-in-Tier-1-only for graduations).
  The explorer records "deferred, review later." The decision is **not**
  re-prompted automatically; the user revisits via the explorer's pending
  panel.
- **`reject`** — Stage 5 discards the op entirely. No write, not even the
  safe fallback. The explorer records the rejection with the op's rationale
  for retrospective inspection.
- **`research`** — Stage 5 continues to await resolution; a fresh research
  LLM session is spawned (see §4 "Research sessions") and streams advisory
  output into the UI alongside the prompt. The user can launch multiple
  research sessions on the same prompt; each is a fresh context.

**Defer semantics are the safe fallback.** At any moment, if the user is not
ready to make a decision, the system must gracefully fall back to the
non-destructive path. A deferred graduation leaves the Tier 1 entry as-is; a
deferred supersession becomes an append-coexist. Both are visible in the
explorer for later resolution.

Every other op class — new entry at `medium`, new entry at `prospective`,
append-observation with or without `contradicts` marker, origin-added-to-
existing — applies silently and lands in the explorer as an informational
record.

### Rule 6 — File-write safety invariants

Stage 5 will never:

- Write a file outside `<tier1_root>/knowledge/` or `<tier2_root>/knowledge/`.
- Overwrite an existing file. All writes are either *append-to-section* or
  *write-new-file-with-unique-name* operations; collisions abort the cycle.
- Touch plan files (`{{PLAN}}/memory.md`, `{{PLAN}}/session-log.md`,
  `{{PLAN}}/backlog.md`). Plan files are read-only to the ingestion pipeline.
- Proceed without holding the sub-project-D store lock for the entire apply
  phase.

Any invariant violation aborts the pipeline and logs a cycle-failure record
to the explorer. The plan cycle itself (work/reflect/triage) is not blocked
by an ingestion failure — ingestion is best-effort from the phase cycle's
perspective, and failures are surfaced through the explorer plus a visible
error badge in the UI.

---

## 4. Event Flow and the Ingestion Events Explorer

Rule 5 introduces two kinds of ingestion output that flow into Mnemosyne's
running UI: **live interactive prompts** (blocking Stage 5 for a specific op)
and **informational records** (non-blocking, flowing into the Ingestion Events
Explorer). §4 specifies how those flow through the system without coupling the
ingestion pipeline to UI implementation details.

### Event type

```rust
enum IngestionEvent {
    /// Non-blocking informational record. Ingestion proceeds immediately.
    /// Displayed in the explorer's chronological feed.
    Applied {
        cycle_id: CycleId,
        plan_id: PlanId,
        host_project: ProjectId,
        section_title: String,
        op_type: OpType,
        target_path: Option<PathBuf>,
        confidence_before: Option<Confidence>,
        confidence_after: Confidence,
        nature: Nature,
        rationale: String,
        source: EventSource,
        timestamp: DateTime,
    },

    /// Blocking prompt. Ingestion of this one op halts until the user
    /// responds. Other ops in the cycle continue in parallel.
    PromptRequired {
        prompt_id: PromptId,
        cycle_id: CycleId,
        event_class: EventClass,     // Graduation | Supersession
        op: ReconciledOp,
        diff_preview: DiffPreview,
        resolve_tx: oneshot::Sender<PromptResolution>,
    },

    /// Deferred prompt record. Re-openable from the explorer.
    Deferred {
        prompt_id: PromptId,
        original_event: PromptRequired,
        fallback_applied: FallbackOp,
        deferred_at: DateTime,
    },

    /// Rejected prompt record.
    Rejected {
        prompt_id: PromptId,
        op: ReconciledOp,
        rejected_at: DateTime,
    },

    /// Streaming research session output.
    ResearchSession {
        parent_prompt_id: PromptId,
        session_handle: ResearchSessionHandle,
        output_rx: mpsc::Receiver<ResearchOutputChunk>,
    },

    /// Non-blocking cycle-level record.
    CycleSummary {
        cycle_id: CycleId,
        applied_count: usize,
        deferred_count: usize,
        rejected_count: usize,
        failures: Vec<IngestionFailure>,
    },
}

enum PromptResolution { Apply, Defer, Reject }

enum EventSource {
    Llm { session_id: SessionId },
    Human { user_action: UserActionId },
}
```

### Flow

1. Stage 5 processes `ReconciledOp`s sequentially within a cycle but runs off
   a dedicated task — the phase cycle itself is not blocked by ingestion.
2. For non-prompt ops, Stage 5 applies the write and emits an `Applied`
   event; the next op proceeds immediately.
3. For prompt ops, Stage 5 emits `PromptRequired` with a oneshot channel,
   then awaits the resolution. Other ops in the same cycle continue in
   parallel, so one slow user response doesn't block unrelated writes.
4. Once all ops resolve, Stage 5 emits `CycleSummary` and releases the store
   lock.

The UI layer subscribes to the event channel, pattern-matches on variants,
and renders:

- `Applied` → append row to the explorer's chronological feed
- `PromptRequired` → open a modal or side panel with the diff preview and
  the four action buttons
- `ResearchSession` → stream advisory output into a side panel
- `Deferred` / `Rejected` → append to the explorer with a badge
- `CycleSummary` → toast or status-bar summary

### Research sessions

Clicking `[research]` on a `PromptRequired` modal does not resolve the
prompt. It opens a side panel showing streaming output from a freshly-spawned
research LLM session. The user can launch multiple research sessions in
sequence on the same prompt; each is a fresh context. The
`[apply/defer/reject]` buttons remain active throughout — research is advisory.

**Research session context:**

- The parent `ReconciledOp` (full content)
- The target entry being superseded or graduated (full content)
- The originating `Section` from Stage 1 (full body)
- The full `{{PLAN}}/memory.md` (for surrounding reflect-phase context)
- The `{{PLAN}}/session-log.md` latest entry (for provenance)
- A research prompt framing the session's role as advisory, not executive

**Research session tool profile — `ResearchBroad`:**

| Tool | Stages 3/4 (`IngestionMinimal`) | Research sessions (`ResearchBroad`) |
|------|--------------------------------|-------------------------------------|
| File read (scoped) | yes, narrow | yes, full |
| File write | **no** | **no** |
| Shell execution | no | yes (read-only commands) |
| Web search | no | **yes** |
| Knowledge store query | no | **yes** |
| Harness-spawn (recursive) | no | no |

The "no file write" invariant is enforced by the adapter at runtime, not by
prompt discipline. A research session that attempts a write aborts with an
error in its own output and is not retried.

**Lifecycle:**

- Research sessions are bound to the lifetime of their parent prompt. If the
  prompt is resolved (apply/defer/reject), any still-streaming research
  sessions for that prompt are cancelled.
- Multiple concurrent research sessions per prompt are allowed; each is a
  fresh context.
- Research session output is persisted alongside the parent prompt event in
  `{{mnemosyne_data}}/ingestion-log/research/<session_id>.jsonl`, referenced
  by ID from the parent event, so the explorer can later show *what advice
  was consulted* when the decision was made.

### The Ingestion Events Explorer

The explorer is **one member of a family of Mnemosyne explorers** (others in
§5's backlog candidate). Sub-project E owns only the ingestion-events data
surface and its requirements; the explorer *framework* is a separate concern.

**Storage:**

```
{{mnemosyne_data}}/ingestion-log/
  2026-04-12.jsonl             # daily-rotated append-only event log
  2026-04-13.jsonl
  research/
    <session_id>.jsonl          # research session transcripts
  index.sqlite                  # derivable cross-view index, rebuildable
```

- The jsonl files are the **source of truth**. They are append-only and
  human-readable.
- The SQLite index is a **derivable cache** for fast cross-view queries
  (filter by plan, project, op type, event class, date range). It can be
  rebuilt at any time from the jsonl log.

**Explorer responsibilities (E's data-surface requirements):**

- Chronological feed of applied/deferred/rejected events
- Filter by cycle, plan, host project, op type, event class, source
- Search across rationales and section titles
- Click-through to the affected entry
- **Re-opening deferred prompts** — clicking a deferred record re-fires the
  modal with the same op, diff preview, and target entry. Stage 5 stores the
  full `ReconciledOp` plus the computed `DiffPreview` on deferral so re-
  opening does not require re-running the pipeline.
- **Viewing research transcripts** alongside the events that launched them

**Required event metadata (for the explorer to do its job):**

Every written event record must include at minimum: `cycle_id`, `plan_id`,
`host_project`, `section_title`, `op_type`, `target_path`, `confidence_before`,
`confidence_after`, `nature`, `rationale`, `event_class` (if prompted),
`user_resolution` (if prompted), `research_session_ids` (if any were launched),
`source` (`Llm | Human`), `timestamp`. Stage 5 emits all of these synchronously
with the state change.

### Decoupling from UI implementation

Stage 5 knows only about the event channel and the oneshot resolution type.
It does not know whether the UI is a TUI, a GUI, a web frontend, or a test
harness. This is critical for two reasons:

1. **Sub-project E can ship ahead of any UI chrome.** A simple stdout/stdin
   stub that prints events and reads `[a/d/r]` responses from the terminal is
   sufficient to run and test ingestion end-to-end.
2. **The UI layer is a separate sub-project.** Sub-project E must not
   constrain the TUI vs. GUI decision.

### Human-driven ingestion (goal #6)

The pipeline exposes direct entry points for the user to drive ingestion
manually, without a harness child session triggering it:

1. **Human-triggered full ingestion.** "Run ingestion on this plan now" —
   Stage 1 is invocable on demand against any plan path. All downstream
   stages run as normal. Useful when the user has edited `memory.md` manually
   and wants it absorbed, or wants to re-run a previous cycle's ingestion.

2. **Review-first mode (`ingestion.mode = review_first`).** A user preference
   that inserts a gate after Stage 4: Stage 5 halts and emits a
   `PromptRequired`-style event showing **all** proposed ops for the cycle,
   grouped and edit-capable. The user can approve, edit, or reject each op
   individually before Stage 5 applies anything. Same pipeline, same
   invariants — a gate, not a different code path.

3. **Direct CRUD via the explorer.** The Stage 5 write primitives
   (`create_entry`, `append_observation`, `supersede_within_entry`,
   `graduate_to_tier_2`) are exposed as UI actions on the explorer's CRUD
   surfaces. They flow through the same Rule 1–6 invariants and emit the
   same event records. A human-driven write is indistinguishable at the
   invariant level from an LLM-driven write; only the `source` field on the
   event record differs (`Human { user_action }` vs `Llm { session_id }`).

### Testability

- **Stages 1, 2, 5** are unit-testable in pure Rust with no LLM calls. Fake
  `Section`s, fake `StoreSlice`s, fake `ReconciledOp`s drive the rules in
  isolation. Every rule in §3 has a dedicated unit test: confidence mapping
  matrix, contradiction gating decisions, axis derivation, tier routing,
  file-write safety.
- **Stages 3 and 4** are integration-tested with a **fixture-replay adapter**
  — each per-section or reconciliation session is replayed from a captured
  JSON fixture instead of calling Claude. The harness adapter (sub-project C)
  must expose a fixture-replay mode. This is recorded as a cross-sub-project
  requirement.
- **Full pipeline** is end-to-end testable against a small in-repo fixture
  plan (memory.md + session-log.md) with a fixture knowledge store and a
  fixture adapter — no live LLM, no real filesystem outside a tempdir.
- **Live gated tests** run against a real Claude Code instance in a
  manual/CI-gated mode, producing real non-deterministic output validated
  only against structural invariants (shape of ops, safety, cycle completes).
  Not run on every commit.

---

## 5. Open Questions, Cross-Sub-Project Requirements, and Risks

### Open questions within sub-project E

Implementation-level questions to be resolved during the build, not during
brainstorming. Decision cost is low enough to leave to the code.

1. **Stage 2 tag extraction heuristic.** Concrete algorithm for extracting
   candidate tags from section title + body. Safe default: stop-word-filtered
   lowercase nouns from the section title, intersected with the current tag
   vocabulary, no body scanning. Reconciliation catches missed entries if the
   heuristic is too narrow.
2. **Stage 3 per-section prompt text.** Drafted during implementation,
   iterated against recorded fixtures.
3. **Stage 4 reconciliation prompt text.** Same.
4. **Research session prompt text.** Same.
5. **"Different project" detection for Rule 1.** Requires a canonical notion
   of project identity. Safe default: repo directory name under DEV_ROOT.
   Configurable via a per-repo `project.id` if needed.
6. **Tier 1 frozen-entry pointer comment format.** Trivial; decided during
   implementation.

### Deferred items — explicitly out of scope for E

1. **UI chrome (TUI vs GUI)** — later sub-project; E is decoupled via event
   channel.
2. **Curate command redesign** — sub-project H or future.
3. **Tier 2 cross-machine synchronisation** — sub-project A.
4. **Horizon-scanning-driven ingestion** — separate path, not addressed here.
5. **Migration of existing Mnemosyne v0.1.0 entries** — sub-project G.

### Cross-sub-project requirements

E generates the following concrete requirements on sibling sub-projects.
These must be recorded in each sibling's `memory.md` when their brainstorms
run.

**For sub-project B (phase cycle in Rust):**
- Expose a "reflect phase exited" hook that ingestion subscribes to.
  Ingestion runs asynchronously off this hook and does not block the cycle
  advancing to triage.
- Expose the current plan's `{{PLAN}}` path and host project identity to the
  ingestion pipeline.
- Phases must support both LLM-driven and human-driven execution paths.
  Reflect, triage, and work can each be performed by the user directly
  through the UI without spawning a harness child. A "Reflect manually"
  action opens an editor on `memory.md` with `session-log.md` latest entry
  as read-only context; similar for triage and work.
- The cycle must be pausable and user-takeoverable. A running LLM phase can
  be interrupted and the user can take over the remaining work manually. On
  takeover, the harness session terminates cleanly, its partial output is
  captured, and the user resumes from the same input state.

**For sub-project C (harness adapter):**
- **Configurable tool profiles** at spawn time. Minimum set:
  `IngestionMinimal` (no tools), `ResearchBroad` (file read, shell read-only,
  web search, knowledge store query). Runtime tool enforcement (rejecting
  disallowed tool invocations) is the adapter's responsibility, not
  Mnemosyne's.
- **Fixture-replay mode.** Adapter accepts a recorded response instead of
  making a live call, enabling deterministic end-to-end ingestion tests with
  no live LLM.
- **Cheap session spawn.** Stage 3 fires N sessions per cycle and latency
  × N is directly visible to the user. Target: < 3 seconds per cold spawn
  keeps a 5-section cycle under 20 seconds. If cold spawn is expensive,
  warm-pool reuse is a valid implementation strategy.
- **Streaming output support** for research sessions — advisory text streams
  into the UI live, not batched at session end.

**For sub-project A (global store location):**
- Tier 1 and Tier 2 roots must be addressable independently — different
  physical locations, each exposed as a config value at Mnemosyne startup.

**For sub-project D (concurrency):**
- Stage 5 must acquire a store write lock before applying ops and release it
  after. Lock scope is the Tier 1 and Tier 2 store roots; whole-store
  granularity is acceptable for v1.
- Ingestion fails gracefully (abort the cycle, log to explorer) if the lock
  cannot be acquired within a timeout. The plan cycle is not blocked by
  ingestion lock contention.

**For sub-project F (plan hierarchy):**
- E assumes a plan belongs to exactly one host project. If F introduces
  plan-hierarchy structures where a plan has multiple host projects, Rule 4's
  Tier 1 routing needs re-examination. Flag this as a coordination point
  during F's brainstorm.

**For sub-project H (Claude Code skills fold):**
- Every existing Mnemosyne Claude Code skill that becomes a phase/command
  must have a human-driven UI equivalent. The seven skills (`/begin-work`,
  `/reflect`, `/setup-knowledge`, `/create-plan`, `/curate-global`,
  `/promote-global`, `/explore-knowledge`) are currently LLM-driven.
  Sub-project H's design must specify both the LLM-driven form (as phase
  prompt or CLI subcommand) and the human-driven form (as UI action).

### New backlog candidates

Two new sub-projects surfaced during this brainstorm that should be added to
the orchestrator backlog during the next triage:

**Sub-project I (proposed) — Explorer and maintenance UI framework.**
Design the unified explorer/maintenance UI framework for all Mnemosyne data
surfaces: ingestion events, plans, harness session history, knowledge entries
(Tier 1 + Tier 2), research session transcripts, curation decisions. Covers
the navigation model, a shared search/filter/index layer, SQLite-backed
cross-view indexing, live update vs. refresh semantics, the relationship
between explorers and live UI modals. Every explorer is a **full CRUD
surface**, not a read-only audit panel — users must be able to create, edit,
delete, annotate, and correct records directly from any explorer, with all
mutations flowing through the same invariants that govern LLM-driven writes.
This requirement is what makes the "human and LLM are co-equal actors"
principle real at the data-surface level. Must support explorers for:
ingestion events, plan files (backlog/memory/session-log), harness session
transcripts, knowledge entries with cross-tier views, research session
history.

**Sub-project J (proposed, tentative) — Human-mode affordances for phases.**
Possibly subsumable into sub-project B's scope if B absorbs it willingly.
Otherwise a dedicated sub-project covering: manual reflect editor, manual
triage editor, manual work editor, pausable/takeoverable LLM phases, and
direct access to phase outputs without going through a harness child. Should
be decided during triage whether this is its own sub-project or B's
responsibility.

### Risks

Three substantive risks, plus the UI-scope-expansion consequence of goal #6.

1. **LLM-call latency dominates cycle wall-time.** N+1 fresh sessions per
   cycle × cold-spawn latency could make ingestion noticeably slow (tens of
   seconds). Mitigation: measure early on realistic cycles; if real, the
   adapter layer (sub-project C) can prioritise warm session reuse or
   pooling. Backstop: drop Stage 4 as a degradation path and accept cross-
   section incoherence risk in exchange for one fewer session.
2. **LLM classification variance producing unstable confidence outcomes.**
   Two sessions processing the same section could yield different
   `experiential`/`speculative` classifications, producing different
   confidence outcomes across runs. Mitigation: classification is a narrow,
   fresh-context question (low variance expected), and Rule 1's invariants
   are monotonic — variance only affects initial confidence of brand-new
   entries, which the user can correct via curate or direct CRUD edit. Worth
   watching but not a blocker.
3. **Silent ingestion of misread observations.** Auto-absorb means that if
   Stage 3 misreads a section as experiential when it was speculative, a
   `medium`-confidence entry enters Tier 1 without user notice. The explorer
   and the Rule 1 ceiling (single session can never produce `high`) both
   bound the damage. Mitigation: the UI must badge "unreviewed ingestion
   events" prominently so the explorer doesn't become an invisible graveyard.
4. **UI surface area expansion from goal #6.** The co-equal-actors principle
   turns Mnemosyne from "a TUI wrapping a CLI" into "a full interactive
   application with CRUD across multiple data surfaces, pausable LLM
   sessions, manual reflection/triage editors, and review-first ingestion
   gating." This is not in E's scope to build but it is a cost E imposes on
   downstream UI sub-projects. Worth flagging to the v1-scope-cut decision:
   if UI scope becomes the critical path, review-first mode and some
   human-equivalent phase actions can be deferred to v2. The minimum bar
   for the co-equal-actors principle to hold is "human can edit the
   knowledge store directly from the explorer" — that should not be deferred.

---

## Appendix A — Decision trail

Each numbered decision below was presented to the user during the 2026-04-12
brainstorm and approved at a decision point. Listed in the order they were
resolved.

1. **Auto-absorb, no staging.** Ingestion writes directly to Tier 1 (and
   Tier 2 via graduation). Curation remains a separate manual session users
   invoke independently. Option A of Q1.
2. **Fires after reflect only.** One ingestion firing per cycle, triggered by
   reflect-phase exit. Option A of Q2.
3. **Free-form `memory.md`, embedded Claude Code ingestion.** Plan memory
   format stays unconstrained; Mnemosyne is itself an LLM client via its own
   harness adapter, using Claude Max subscription pricing rather than per-
   token metered API access. Option A of Q3.
4. **Fresh LLM context is a first-class architectural goal** (user
   clarification during Q3 refinement). Many short fresh sessions beat one
   long multi-purpose session. This principle is load-bearing for the rest
   of the design.
5. **Pipeline shape: parse → retrieve → N per-section ops → reconciliation
   → validate+apply.** Three deterministic stages, two LLM stages.
   Option D of Q4 with an explicit reconciliation step.
6. **Contradiction handling: confidence-gated supersession with
   append-coexist fallback.** Option C of Q5.
7. **Confidence assignment: LLM classifies experiential/speculative;
   Mnemosyne applies deterministic mapping with monotonic accumulation
   rules.** Option D of Q6.
8. **Axis assignment: deterministic `suggest_axis(tags)`.** LLM produces
   tags, Mnemosyne routes. Option (a) of Q7.
9. **Tier routing: Option Y with "Tier 2 wins after graduation" sub-rule.**
   Tier 1 always; auto-promotion candidate fires interactive prompt on the
   accumulation rule. Option Y of Q8.
10. **Continuously-running UI with TUI/GUI ambiguity** (user clarification
    during Q8 review). Mnemosyne is a long-running application hosting
    sequences of harness sessions interleaved with its own LLM calls and
    user prompts. TUI vs GUI decision is live but out of E's scope.
11. **Interactive prompts on graduation and supersession only.** Three-
    action prompts extended to four with `[research]`. Defer = safe non-
    destructive fallback. Option C of Q9, extended.
12. **Research LLM sessions** launchable from any PromptRequired modal,
    advisory-only, fresh context, broader tool profile than ingestion
    sessions. User refinement during Section 4 review.
13. **Explorers are a first-class accountability substrate** (user
    clarification during Section 5 review). The review panel is one
    explorer in a family; every Mnemosyne data surface gets an explorer;
    explorers are full CRUD surfaces. New sub-project I candidate added.
14. **Human and LLM are co-equal actors** (user clarification during
    Section 5 review). Every LLM-driven action has a human-driven
    equivalent through the same invariants. No workflow is LLM-only.
    New goal #6 added.

---

*End of sub-project E design document.*
