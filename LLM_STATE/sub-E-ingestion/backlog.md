# Backlog — Sub-project E: Post-Session Knowledge Ingestion

Implementation backlog for sub-project E. All tasks derive from the design
doc at
`{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-E-ingestion-design.md`.
Consult the spec before starting any task.

Tasks are listed in approximately recommended order. Deterministic Rust
work comes first (stages 1, 2, 5 and the six invariants), then event
emission and persistence, then LLM-integration stages (3 and 4) with a
fixture-replay adapter stub, then human-mode affordances, then end-to-end
tests. The work phase picks the best next task with input from the user.

## Task Backlog

### Define core ingestion types `[types]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Define the Rust types that flow through the pipeline:
  `Section`, `StoreSlice`, `ProposedOp`, `ReconciledOp`, `OpType`, `Nature`,
  `Confidence` (if not already in the crate), `EventSource`, `EventClass`,
  `PromptResolution`, `IngestionEvent`, `IngestionFailure`, `CycleId`,
  `PlanId`, `ProjectId`, `PromptId`, `DiffPreview`, `FallbackOp`. Follow the
  shape defined in §2 and §4 of the spec. These types are the contract the
  rest of sub-project E's code is written against — get them right first so
  downstream tasks have a stable target.
- **Results:** _pending_

### Implement Stage 1 — section parser `[pipeline]`
- **Status:** not_started
- **Dependencies:** Define core ingestion types
- **Description:** Deterministic parser that reads a plan's `memory.md` and
  the latest entry from `session-log.md` and returns `Vec<Section>`. Parses
  H2/H3 section headings as titles, captures section bodies verbatim, extracts
  `session_date` from session-log, carries `plan_path` and `host_project` as
  provenance. Pure Rust, no LLM, no filesystem writes. Unit tests against
  fixture plan files (capture small real-world samples from the existing
  LLM_CONTEXT plans as fixtures).
- **Results:** _pending_

### Implement Stage 2 — deterministic retrieval `[pipeline]`
- **Status:** not_started
- **Dependencies:** Define core ingestion types, Implement Stage 1
- **Description:** For each `Section`, extract candidate tags using the
  safe-default heuristic (stop-word-filtered lowercase nouns from the title
  intersected with the existing tag vocabulary), query the knowledge store,
  and produce a `StoreSlice` containing the 0–5 entries whose tags overlap.
  Store access is via a trait `KnowledgeStore` that can be implemented over
  a real filesystem store or a fixture in-memory store. Unit tests cover:
  empty store, no matches, 1 match, 5 matches, tag-only matches with no
  content match, and the stop-word filter.
- **Results:** _pending_

### Implement Rule 1 — confidence mapping `[invariants]`
- **Status:** not_started
- **Dependencies:** Define core ingestion types
- **Description:** Implement the confidence-derivation matrix from §3 Rule 1
  as a pure function: `(existing_entry_state, nature, origins) -> Confidence`.
  Dedicated unit test for every row of the matrix, plus the two monotonic
  invariants (never-falls-via-ingestion, never-climbs-to-high-in-one-cycle).
- **Results:** _pending_

### Implement Rule 2 — contradiction gating `[invariants]`
- **Status:** not_started
- **Dependencies:** Define core ingestion types, Implement Rule 1
- **Description:** Implement the contradiction-gating rule from §3 Rule 2
  as a pure function: `(ProposedOp, Entry) -> GatedOp`, where the result is
  either the original `supersede` op (if the confidence and origins rule
  passes) or a downgraded `append_observation` op with a `contradicts`
  marker. Unit tests cover: equal-confidence passes, lower-confidence fails,
  insufficient-origins fails, sufficient-origins passes, same-project-gap
  edge case.
- **Results:** _pending_

### Implement Rule 3 — axis derivation `[invariants]`
- **Status:** not_started
- **Dependencies:** Define core ingestion types
- **Description:** Implement `derive_axis(tags: &[Tag]) -> Axis` using a
  table-driven rule set loaded from Mnemosyne's config. Fall back to
  `techniques/` if no rule matches. Unit tests for each axis, the fallback,
  and an empty-tags case.
- **Results:** _pending_

### Implement Rule 4 — tier routing `[invariants]`
- **Status:** not_started
- **Dependencies:** Define core ingestion types, Implement Rule 1
- **Description:** Implement the tier-routing rules from §3 Rule 4 as a
  pure function: `(ReconciledOp, StoreState) -> TierWriteTarget`, where the
  result names which tier(s) to write to. Handles new entries (always
  Tier 1), appended observations on single-tier entries, appended
  observations on dual-tier entries (Tier 2 wins), and the graduation
  trigger case. Unit tests for each branch. Graduation detection (not the
  prompt firing — just the detection) is part of this rule.
- **Results:** _pending_

### Implement Rule 6 — file-write safety `[invariants]`
- **Status:** not_started
- **Dependencies:** Define core ingestion types
- **Description:** Implement a `SafeFileWriter` wrapper that enforces the
  §3 Rule 6 invariants at runtime: writes only inside `<tier1_root>/knowledge/`
  or `<tier2_root>/knowledge/`, no overwrites, no plan-file writes, required
  lock held. Every invariant violation returns an error and aborts the
  cycle. Unit tests attempt each violation class and assert rejection.
- **Results:** _pending_

### Implement the ingestion-event log `[persistence]`
- **Status:** not_started
- **Dependencies:** Define core ingestion types
- **Description:** Implement the jsonl append-only event log at
  `{{mnemosyne_data}}/ingestion-log/YYYY-MM-DD.jsonl`. One function appends
  a serialised `IngestionEvent` to the current day's file, rotating daily.
  Another reads events by date range. Implement the minimum write path
  first; the SQLite derivable index comes later. Unit tests cover serialise/
  deserialise round-trip, daily rotation, and read-back.
- **Results:** _pending_

### Implement Stage 5 — validate and apply `[pipeline]`
- **Status:** not_started
- **Dependencies:** Implement Rule 1, Implement Rule 2, Implement Rule 3,
  Implement Rule 4, Implement Rule 6, Implement the ingestion-event log
- **Description:** Wire the six invariants together into Stage 5. Takes a
  `Vec<ReconciledOp>`, runs each through Rules 1–4, applies Rule 6's safety
  checks, emits `IngestionEvent` records, and performs the actual writes to
  Tier 1 / Tier 2. Uses a `PromptGate` trait for Rule 5 events so the UI
  event channel can be stubbed for testing. Integration tests drive Stage 5
  with fixture ReconciledOps against a tempdir store.
- **Results:** _pending_

### Implement the event channel and UI stub `[ui]`
- **Status:** not_started
- **Dependencies:** Define core ingestion types
- **Description:** Implement the `IngestionEvent` channel (a tokio mpsc) and
  a simple stdout/stdin UI stub that prints events and reads
  `[a/d/r/research]` responses from the terminal. This is the "sub-project E
  can ship ahead of UI chrome" decoupling from §4 of the spec. The stub is
  sufficient for all end-to-end integration tests; a real TUI/GUI replaces
  it later (not in sub-project E's scope).
- **Results:** _pending_

### Define the harness adapter trait and stub `[adapter]`
- **Status:** not_started
- **Dependencies:** Define core ingestion types
- **Description:** Define the minimum `HarnessAdapter` trait sub-project E
  needs: `spawn_session(tool_profile, prompt, context) -> SessionHandle`,
  with methods for capturing output and signalling completion. Tool profiles
  are `IngestionMinimal | ResearchBroad`. Implement a `FixtureAdapter` that
  replays responses from JSON fixture files. This is a temporary
  placeholder; the real implementation is sub-project C's responsibility.
  Record the required interface as a hard dependency in sub-project C's
  backlog (cross-plan task).
- **Results:** _pending_

### Implement Stage 3 — per-section LLM ops `[pipeline]`
- **Status:** not_started
- **Dependencies:** Implement Stage 2, Define the harness adapter trait and
  stub
- **Description:** For each `(Section, StoreSlice)` pair, spawn a fresh
  session via the adapter, pass the narrowly-scoped context, and parse the
  returned `ProposedOp`. Draft the per-section prompt text and iterate it
  against fixture inputs. Integration tests use the FixtureAdapter to
  replay recorded responses deterministically. Live tests are gated and
  run manually.
- **Results:** _pending_

### Implement Stage 4 — reconciliation `[pipeline]`
- **Status:** not_started
- **Dependencies:** Implement Stage 3
- **Description:** Spawn one fresh session with the minimal reconciliation
  context (op metadata only, no bodies, no store content), parse the
  reconciled op list. Draft the reconciliation prompt text. Integration
  tests via FixtureAdapter. Tests cover: no-change case (common), cross-
  section merge, duplicate collapse, ordering constraint.
- **Results:** _pending_

### Wire the full pipeline end-to-end `[pipeline]`
- **Status:** not_started
- **Dependencies:** Implement Stage 1, Implement Stage 2, Implement Stage 3,
  Implement Stage 4, Implement Stage 5, Implement the event channel and UI
  stub
- **Description:** Compose stages 1–5 into a single `run_ingestion_cycle()`
  entry point. Accepts a plan path, a knowledge store handle, an adapter, an
  event channel, and a cycle ID. Returns a `CycleSummary`. End-to-end
  integration test runs a fixture plan through the full pipeline with a
  fixture adapter and asserts the final store state plus the emitted
  events match expectations.
- **Results:** _pending_

### Implement research session launch affordance `[ui]`
- **Status:** not_started
- **Dependencies:** Implement the event channel and UI stub, Define the
  harness adapter trait and stub
- **Description:** When the UI receives a `PromptRequired` event and the user
  chooses `research`, spawn a fresh session with the `ResearchBroad` tool
  profile, pass the full research context (parent op, target entry,
  originating section, full plan memory, session-log latest entry, research
  prompt), stream output back to the UI. Persist the research session
  transcript to `{{mnemosyne_data}}/ingestion-log/research/<session_id>.jsonl`.
  The `[apply/defer/reject]` buttons remain active; research is advisory.
- **Results:** _pending_

### Implement human-triggered ingestion entry point `[human-mode]`
- **Status:** not_started
- **Dependencies:** Wire the full pipeline end-to-end
- **Description:** Expose a `run_ingestion_on_plan(plan_path)` function that
  the user can invoke directly without waiting for a reflect-phase hook.
  Useful when the user has edited `memory.md` by hand. Wire it into the
  stub UI as a command. Integration tests cover: ingesting on a freshly
  edited plan, re-running on a previously-ingested plan, running on a
  section-filtered subset.
- **Results:** _pending_

### Implement review-first mode `[human-mode]`
- **Status:** not_started
- **Dependencies:** Wire the full pipeline end-to-end
- **Description:** Add `ingestion.mode = silent | review_first` config flag.
  In `review_first`, Stage 5 halts after Stage 4 reconciliation and emits a
  special `ReviewRequired` event containing all proposed ops. The UI lets
  the user approve, edit, or reject each op individually before Stage 5
  applies anything. Same pipeline, same invariants. Integration tests cover:
  review-first approval path, per-op rejection, per-op edit, full-cycle
  rejection.
- **Results:** _pending_

### Expose Stage 5 primitives as direct CRUD `[human-mode]`
- **Status:** not_started
- **Dependencies:** Implement Stage 5
- **Description:** Expose `create_entry`, `append_observation`,
  `supersede_within_entry`, `graduate_to_tier_2` as public functions that
  bypass the LLM pipeline but still flow through Rule 1–6 invariants and
  emit `IngestionEvent` records with `source: Human { user_action }`. The
  explorer's CRUD UI will call these when it lands. Unit tests cover:
  human write through each primitive, invariant enforcement on bad inputs,
  event emission with correct source field.
- **Results:** _pending_

### SQLite derivable index over the event log `[persistence]`
- **Status:** not_started
- **Dependencies:** Implement the ingestion-event log
- **Description:** Build the SQLite index described in §4 of the spec as a
  derivable cache of the jsonl event log. Supports filter-by-cycle,
  filter-by-plan, filter-by-host-project, filter-by-op-type,
  filter-by-event-class, filter-by-source, date-range queries, and full-text
  search over rationales. Includes a rebuild-from-jsonl command for recovery.
  Integration tests cover: initial build, incremental append, rebuild from
  scratch, all filter types.
- **Results:** _pending_

### End-to-end integration test corpus `[testing]`
- **Status:** not_started
- **Dependencies:** Wire the full pipeline end-to-end
- **Description:** Capture a small corpus of realistic plan-memory files
  (adapted from the existing LLM_CONTEXT plans with names anonymised if
  needed) and corresponding recorded adapter fixture responses. Build a
  test harness that runs the full pipeline against each fixture plan and
  asserts the final store state plus event records match expectations.
  This is the primary regression safety net for the sub-project.
- **Results:** _pending_

### Live gated test against real Claude Code `[testing]`
- **Status:** not_started
- **Dependencies:** Wire the full pipeline end-to-end, Define the harness
  adapter trait and stub (upgraded to a real adapter via sub-project C)
- **Description:** Create a manual/CI-gated test that runs the full pipeline
  against a real Claude Code instance with real fixture plan inputs,
  asserting only structural invariants (shape of ops, no safety violations,
  cycle completes). Not run on every commit. Produces characterisation
  data for the latency/variance risks flagged in §5 of the spec.
- **Results:** _pending_

### Implementation notes and handoff documentation `[docs]`
- **Status:** not_started
- **Dependencies:** Wire the full pipeline end-to-end
- **Description:** Document any implementation decisions, discovered
  constraints, or design-doc deviations in `{{PLAN}}/memory.md`. Produce a
  short user-facing note for the main Mnemosyne docs explaining how
  ingestion works at a conceptual level, when it fires, and how the review
  panel surfaces its activity.
- **Results:** _pending_
