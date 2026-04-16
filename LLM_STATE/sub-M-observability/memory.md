# Memory — Sub-project M: Observability Framework

This plan implements sub-project M of the Mnemosyne orchestrator merge. The
design is already fully specified; this plan is the implementation work.

## Primary reference

**`{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-M-observability-design.md`**
(committed at `53f7d4e`) is the authoritative design document. Every task
in this plan's backlog derives from that spec. If any implementation
question arises that the spec does not answer, the answer goes into this
memory file (and possibly back into the spec) rather than being invented
ad hoc.

## Parent plan

The orchestrator-level plan lives at
`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/` (currently — will be at
`{{PROJECT}}/mnemosyne/plans/mnemosyne-orchestrator/` after sub-project G's
migration). It coordinates this sub-plan with its siblings (A, B, C, D,
E, F, G, H, I, K, L). The parent plan's `memory.md` holds cross-sub-project
architectural state. This file holds only sub-project-M-specific
implementation state.

## Sub-M is cross-cutting and unblocks tactical instrumentation cleanup

Every other sub-project's structured-logging needs route through M once
it lands. Tactical instrumentation seeds — currently only C's
`SpawnLatencyReport`, but expected in B/D/E as their implementation phases
progress — migrate onto M's framework rather than accreting framework-
shaped scope creep in their owners.

M's brainstorm output therefore has two halves:

1. **This sibling plan** — the implementation backlog for the framework
   itself.
2. **Adoption tasks landed in sibling plans** — sub-B, sub-C, sub-E
   already exist and have adoption tasks added directly by the brainstorm.
   Sub-D, sub-F, sub-H, sub-I, sub-G adoption tasks are queued in this
   memory file (see "Cross-plan adoption coordination" below) and land in
   those sibling backlogs as those sub-projects' brainstorms complete.

## Key architectural anchors (quick reference; spec is canonical)

These are the decisions most load-bearing for implementation. Consult the
design doc for full context before acting on any of them.

### Hybrid `tracing` + typed events (Q2 — settled)

`tracing` for transport / spans / async / third-party crate integration;
one canonical `MnemosyneEvent` enum for type discipline at the Mnemosyne
boundary. The custom code is bounded to one `tracing-subscriber::Layer`
(`MnemosyneEventLayer`, ≈200 lines). All other layers are stock
`tracing-subscriber` layers or thin (~30-50 line) wrappers.

### Five standard crates, no custom replacements

`tracing` + `tracing-subscriber` + `tracing-appender` + `metrics` +
`metrics-util`. All top-100 by downloads on crates.io. No bespoke event
bus, subscriber framework, or rotation logic.

### Always-on instrumentation, no debug flags

Every event emit is unconditional. The `MNEMOSYNE_LOG` env var only
controls stderr formatting visibility, not whether events are emitted.
This is the project-wide always-on principle inherited from sub-project
C's `SpawnLatencyReport`.

### Vault-scoped persistence

Operational data lives under `<vault>/runtime/events/` and
`<vault>/runtime/metrics/` (gitignored, transient). The historical,
user-browsable Obsidian-facing record lives under
`<vault>/projects/<project>/mnemosyne/observability/sessions/` (git-tracked).
The vault runtime layout integrates with B's existing `<vault>/runtime/`
subtree (B owns staging, interrupts, ingestion events, locks; M owns
events, metrics, observability summaries).

### Risk 5 resolution via `InMemoryRingLayer::dump_session`

C's accepted Risk 5 ("v1 ships with diagnostic-poor failure modes") is
resolved by a one-method-call diagnostic primitive. Every error path in
B's `PhaseRunner`, C's actor, and E's pipeline calls
`InMemoryRingLayer::dump_session(session_id, n)` and writes the result
to `<vault>/runtime/interrupted/<plan-id>/<phase>-<timestamp>/event-tail.json`.
The TUI's error display surfaces a "view event tail" action that opens
the file.

### Staged migration of C's `SpawnLatencyReport` (parallel-emit window)

| Phase | C's tactical writer | M's `metric!` calls |
|---|---|---|
| C v1 (today) | yes | no |
| M v1 lands | yes | yes (parallel) |
| M v1.1 (after verification) | no | yes |
| G migration | gone | gone (file deleted from staging schema) |

The parallel-emit window is the safety net. A Layer 3 integration test
reads both `<staging>/spawn-latency.json` and the metric snapshot from
the same session, asserts the three latency values match within ±10ms.
The test runs on every CI build during the parallel-emit window. Only
once the verification window passes does C's writer get deleted.

## Implementation strategy — recommended task order

The §15 v1 cut from the spec maps to roughly 20 tasks. The recommended
order:

1. **Setup (Tasks 1-3)**: Cargo.toml deps, module skeleton, `MnemosyneEvent`
   enum + helper types. No subscriber wiring yet; just the type surface.
2. **Metric catalogue (Task 4)**: `metric_names::*` constants + CI test
   for one-to-one parity with §6 of the spec. This task is small but
   important — it locks in the metric name discipline before any call
   site exists.
3. **Custom Layer (Tasks 5-6)**: `MnemosyneEventLayer` (the only piece
   of new custom code) and the `mnemosyne_event!` macro wrapper.
   Microbenchmark the typed-payload handoff approach (thread-local
   trick vs `Visit` API + serde round-trip) per §16 Q1; pick the
   winner.
4. **Standard layers (Tasks 7-11)**: `InMemoryRingLayer`,
   `JsonlPersistLayer`, `MetricsRecorderLayer`, `TuiBridgeLayer`,
   `StderrFmtLayer`. Each ~30-50 lines plus tests.
5. **Composition (Task 12)**: `ObservabilityHarness` startup function
   that composes the Registry + all Layers and returns a handle the
   rest of Mnemosyne uses to access subscribers (e.g., `dump_session`).
6. **Risk 5 wiring (Task 13)**: `event-tail.json` writer + integration
   into B / C / E error paths.
7. **C parallel-emit (Tasks 14-15)**: add `metric!` calls in C's actor
   alongside the existing `SpawnLatencyReport` writer; write the
   verification integration test.
8. **CLI subcommands (Tasks 16-17)**: `mnemosyne metrics` and
   `mnemosyne diagnose`.
9. **Cross-plan adoption (Task 18)**: NOTE — this is largely
   pre-completed by M's brainstorm. The sub-B / sub-C / sub-E adoption
   tasks have already been added to those sibling backlogs by the
   brainstorm session that produced this plan. Task 18 is the
   verification step: re-read each sibling backlog and confirm the
   adoption stub is present.
10. **Integration tests (Tasks 19-21)**: fixture-replay end-to-end,
    Risk 5 end-to-end, JSONL property tests.
11. **Re-entrancy test (Task 22)**: Risk 1 mitigation — emit 1M events
    through the full Registry stack and assert no panics or stack
    overflows.

## Cross-plan adoption coordination

Adoption tasks landed by M's brainstorm session:

| Sibling plan | Status | Task title |
|---|---|---|
| `sub-B-phase-cycle` | landed by brainstorm | "Adopt sub-M observability framework — phase lifecycle instrumentation + TUI bridge consumer" |
| `sub-C-adapters` | landed by brainstorm | "Adopt sub-M observability framework — actor instrumentation + parallel-emit migration of `SpawnLatencyReport`" |
| `sub-E-ingestion` | landed by brainstorm | "Adopt sub-M observability framework — verify schema: sub-E emits directly to the M bus using the `%Mnemosyne.Event.Ingestion.*{}` structs from §4.1; no parallel-emit step needed; adoption task is to verify the emitted structs match §4.1 definitions when sub-E implementation begins" |
| `sub-N-domain-experts` | landed by learning (Session 16, 2026-04-15) | backlog Task 24 — absorb `Mnemosyne.Event.Expert.*` six-struct group into §4.1 sealed set + eight §6 metric definitions; gated on sub-N Task 15 early-deliverable PR |

Adoption tasks queued for sibling plans that don't yet exist:

| Sibling plan | Status | Trigger |
|---|---|---|
| `sub-D-concurrency` | queued | When sub-D's brainstorm lands its sibling plan, this plan's triage adds a "M observability adoption" task to D's backlog |
| `sub-F-hierarchy` | plan scaffolded (Session 13); F's Task 24 is the adoption task; F's Task 0 gate blocks on M's amendment. Sub-A gate condition cleared (Session 14, 2026-04-15); M amendment is now the sole remaining orchestrator-level gate condition for F Task 0 (other two gates are sibling-level: sub-B and sub-C task-list rewrites) | Sub-M amendment (backlog Task 0) is critical-path — F cannot start implementation until M's design doc is rewritten for Elixir/OTP |
| `sub-H-skills` | queued | Same trigger as D |
| `sub-I-obsidian-coverage` | queued | Same trigger as D — and I's coverage doc itself absorbs the v1.5 Obsidian session-summary format |
| `sub-G-migration` | queued | When G's brainstorm lands, add a "delete `<staging>/spawn-latency.json` from staging schema after M parallel-emit window closes" task |

The triage phase of this sub-M plan checks the orchestrator backlog
on each cycle: when any of the queued sibling plans transitions from
`not_started` to `done` (brainstormed), the next triage adds the
adoption task to that sibling's backlog.

## BEAM pivot absorbed via inline rewrite (Session 15, 2026-04-15)

The orchestrator rewrote the design doc inline across §1–§20 to target
Elixir/OTP idioms, following the sub-C/sub-B/sub-A precedent (no
supersede layer). The original Session 7 Q1–Q5 decisions are preserved
verbatim in the Decision Trail with Session-15 correction notes; new
Q6 records the BEAM pivot translation table (19 Rust→BEAM idioms); new
Q7 records the reporter selection (`Telemetry.Metrics.ConsoleReporter`
+ `SnapshotReporter` for v1, `:telemetry_metrics_prometheus` additive
in v1.5, OpenTelemetry reserved for v2).

### Design-doc anchors for implementation

Consult the rewritten spec before touching the Rust-flavored backlog
tasks below.

- **§3** — Hex dep stack: `:telemetry`, `:telemetry_metrics`, `Jason`
  for v1; `:telemetry_metrics_prometheus` for v1.5+; stdlib
  `GenServer` / `DynamicSupervisor` / `:ets` / `:pg` for subscribers.
  No `prom_ex`, no `Phoenix.PubSub`, no OpenTelemetry in v1.
- **§4.1** — sealed `Mnemosyne.Event.*` struct set. 20+ variants grouped
  by producer: sub-B (one struct with seven `kind` variants), sub-C
  (five), sub-F (nine), sub-E (six Ingestion variants), sub-A (two
  Vault variants), M-owned escape hatches (`Metric`, `Diagnostic`,
  `Error`). Sealed for v1; adding a variant needs a Decision Trail
  entry.
- **§4.2** — emission via `Mnemosyne.Observability.emit/1` which wraps
  `:telemetry.execute/3` with the struct in `metadata.event` and a
  derived event name prefix (`%Event.PhaseLifecycle{}` →
  `[:mnemosyne, :phase_lifecycle]`).
- **§5** — subscriber stack: `Mnemosyne.Observability.Handler` attached
  via `:telemetry.attach_many/4`, dispatching in order
  `RingBuffer → Metrics → TuiBridge → JsonlWriter` (cheap first). Four
  supervised subscribers under `Mnemosyne.Observability.Supervisor`
  with `:rest_for_one` strategy.
- **§5.2** — the load-bearing `try/rescue` inside `handle_event/4`:
  `:telemetry` detaches handlers that raise, so the wrapper must log
  via `Logger` (not through M) and return `:ok`. New failure mode vs
  the Rust design — called out in Risk 6.
- **§5.3** — re-entrancy discipline: no code reachable from the handler
  may call `Observability.emit/1` or `:telemetry.execute/3`. Subscribers
  communicate via `GenServer.cast` only. Re-entrancy integration test
  at 1M events + witness counter (§14).
- **§6** — metric catalogue: 23 `Telemetry.Metrics.*` definitions
  (4 phase counters, 3 harness counters + 3 latency distributions +
  1 last_value, 4 ingestion, 8 routing/actor, 1 sentinel, 1 drop, 3
  actor-lifecycle; note that a few use `keep:` filters that branch on
  struct fields). Catalogue test parses §6 and asserts one-to-one
  correspondence — substitute for the Rust `const &'static str`
  compile-time protection.
- **§7** — storage layout integrates with sub-A's `<vault>/runtime/`
  subtree. `events/<qualified-id>/<session-id>.jsonl`,
  `metrics/<qualified-id>/<session-id>.json`,
  `interrupted/.../event-tail.json` (Risk 5 forensics dump).
- **§8** — Risk 5: one `GenServer.call` to
  `Mnemosyne.Observability.RingBuffer.dump_session/2` with a grace
  window so the per-session ring process stays alive briefly after
  session end.
- **§9** — staged migration of C's `%SpawnLatencyReport{}` across four
  phases: pre-M → M v1 parallel-emit → M v1.1 staging-file deletion →
  M v2 full cleanup. ±10ms verification tolerance.
- **§10** — cross-plan adoption table: nine siblings (A/B/C/D/E/F/G/H/I).
  Seven are concrete commitments already landed in the siblings' own
  design docs (B §4.4, C §7.2 + §11.4, F Task 24, A §A.Reference
  algorithm step 11, plus E post-F-amendment).
- **§11.1** — TUI bridge uses `:pg` process group `:mnemosyne_tui`;
  client-session processes join on subscribe; `send/2` is the
  fan-out primitive; per-client drop-oldest + drop counter.
- **§12** — analysis tooling becomes `mix mnemosyne.metrics` and
  `mix mnemosyne.diagnose` (escript-style Mix tasks), not CLI
  subcommands.
- **Q6** — full Rust→BEAM translation table and five material findings
  that shaped the rewrite (raising-handler-detach, no-compile-time
  constants, `:pg` vs mpsc, F's growth of §6, A's two new structs).

### Downstream Rust task-list is stale — gate task pending

Tasks 1–23 below were brainstormed against the Rust runtime and still
reference `tracing`, `tracing-subscriber`, `metrics` / `metrics-util`,
`cargo`, `MnemosyneEvent` enum, thread-local tricks, `tokio::sync::mpsc`,
etc. **Do not execute any of them as written.** The next work phase
(backlog Task 0 gate) must rewrite the downstream task list against
the Session-15 spec rewrite, following the same discipline sub-B and
sub-C applied in their sibling plans: delete tasks that have no BEAM
analogue (e.g. Task 5's thread-local vs Visit-API microbenchmark),
rewrite Rust module-layout tasks as Elixir module-layout tasks, add
new tasks for `SnapshotReporter`, the `:pg`-based `TuiBridge`, the
`try/rescue` handler wrapper, the catalogue-integrity test, and the
nine new §6 metric definitions from sub-F's routing surface.

**Critical-path update (2026-04-15):** This amendment clears the sole
remaining orchestrator-level gate condition (3) for sub-F's Task 0.
The remaining F Task 0 blockers are both sibling-level task-list
rewrites (sub-B and sub-C), not amendments. Combined with this plan's
own downstream task-list rewrite (still pending), sub-M's internal
implementation runway is behind sub-B / sub-C / sub-F in the critical
path ordering.

### C's rewritten design doc as the concrete Elixir exemplar

Sub-C's rewritten spec remains the load-bearing producer-side reference
for the amendment implementation. Use C's §3.3 / §7.2 / §11.4 as the
canonical shape for M's sealed event set, the three-way parallel-emit
discipline, and the `:telemetry.execute/3` at every boundary contract.
Sub-B's §4.4 is the canonical PhaseLifecycle producer contract. Sub-F's
Task 24 is the canonical routing/actor producer contract. Sub-A's
§A.Reference algorithm step 11 is the canonical Vault.BootReady
producer.

### C's rewritten design doc as the concrete Elixir exemplar

Sub-C's design doc (`docs/superpowers/specs/2026-04-13-sub-C-adapters-design.md`)
was fully rewritten in Session 11 for Elixir/BEAM and now provides the
first concrete implementation of M's `:telemetry` + typed struct pattern.
The amendment session should use C's emission pattern as the template
other sub-projects will follow:

- **§3.3** defines `Mnemosyne.Event.*` typed structs (`HarnessOutput`,
  `SessionLifecycle`, `SpawnLatencyReport`, `SessionExitStatus`,
  `HarnessError`) with `@enforce_keys` + `defstruct` — the canonical
  shape for M's sealed event set in Elixir.
- **§7.2** shows the three-way parallel emission pattern for
  `%SpawnLatencyReport{}`: consumer info message + `:telemetry.execute/3`
  under `[:mnemosyne, :harness, :claude_code, :spawn_latency]` + staging
  JSON file — this is the staged migration path M owns.
- **§11.4** states the contract: C emits `:telemetry.execute/3` at every
  boundary (spawn, first chunk, init event, turn boundary, tool-use,
  tool-result, terminate, exit) and feeds M's event-tail ring buffer.

### B's rewritten design doc as the concrete phase-lifecycle producer contract

Sub-B's design doc (`docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md`)
was fully rewritten in Session 12 for Elixir/BEAM. §4.4 now defines seven
concrete `%PhaseLifecycle{}` typed event variants that M must consume as
part of its sealed `Mnemosyne.Event.*` struct set:

1. `%PhaseLifecycle{kind: :started}` — emitted at `run_phase/4` step 5
2. `%PhaseLifecycle{kind: :exited_clean}` — emitted at step 11
3. `%PhaseLifecycle{kind: :reflect_hook_fired}` — emitted at step 12
4. `%PhaseLifecycle{kind: :interrupted}` — emitted at §3.4 step 5
5. `%PhaseLifecycle{kind: :executor_failed}` — emitted on error branches
6. `%PhaseLifecycle{kind: :takeover_offered}` — emitted at §3.4 step 6
7. `%PhaseLifecycle{kind: :prior_interrupt_surfaced}` — emitted at §3.2 Scenario A

B also forwards `%HarnessOutput{}` and `%SessionLifecycle{}` events from C.
Combined with C's §11.4 telemetry contract (harness adapter side), M's
amendment now has concrete producer-side contracts from both B and C as
input for defining the sealed event struct set in Elixir.

## Open questions

These do not block implementation but should be resolved during v1 build.
Mirrors §16 of the spec.

### Q1 — `MnemosyneEvent` typed-payload handoff approach
Thread-local trick vs `Visit` API + `serde_json` round-trip. Microbenchmark
both approaches on day 1 of implementation. Choice does not affect the
design.

### Q2 — `InMemoryRingLayer` default capacity
Spec defaults to 1000 events per session. Whether this is too many or too
few for the C-1 dogfood envelope can only be measured during the first
dogfood run. Tune after first cycle.

### Q3 — Whether `ObsidianMaterialiseLayer` lands in v1 instead of v1.5
Deferred until v1 ships and dogfood feedback arrives. The decision
depends on user demand, not implementation cost (the layer is
straightforward).

### Q4 — `tokio-console` as a recommended layer in v1 docs
Try it during v1 development; decide based on developer ergonomics.

### Q5 — Histogram bucket layout
`metrics-util` default vs custom for latency-friendly resolution. Inspect
default percentile accuracy on synthetic latency data on day 2.

## Risk watch list

Mirrors §17 of the spec.

### Risk 1 — `tracing-subscriber` Layer ordering / re-entrancy bugs
*MEDIUM impact, LOW likelihood.* Mitigation: code review checklist + a
1M-event re-entrancy integration test (Task 22).

### Risk 2 — Bounded-queue overflows hide real problems
*LOW impact, MEDIUM likelihood.* Mitigation: `events.dropped` counter is
surfaced as a non-zero red badge in the TUI status bar; `mnemosyne diagnose`
prints a warning if any drops occurred in the session snapshot.

### Risk 3 — C parallel-emit window produces inconsistent data
*LOW impact, LOW likelihood.* Mitigation: verification test (Task 15)
catches discrepancies before production; ±10ms clock-skew tolerance.

### Risk 4 — `MnemosyneEvent` enum becomes a god object
*MEDIUM impact, MEDIUM likelihood.* Mitigation: most new events go
through `MnemosyneEvent::Diagnostic` (typed escape hatch); adding a new
top-level variant requires brief justification in the spec evolution log;
the bar is "the new variant must have multiple downstream consumers."

### Risk 5 — `metrics` crate facade adds layering overhead
*LOW impact, LOW likelihood.* Mitigation: Mnemosyne's metric volume is
low; vtable cost is in the nanoseconds. If a high-frequency metric
appears, emit it directly via `tracing::event!`.

## Verified surface (none yet)

Implementation discoveries about the standard crates' actual behaviour
(e.g., `tracing-appender` non-blocking guard semantics under sustained
load, `metrics-util::Snapshotter` percentile accuracy, the exact shape
of `tracing::Visit` that the custom Layer must implement) are recorded
here as they're observed during implementation. Each entry should
record the pinned crate version and the source of the observation
(test, benchmark, manual experiment, upstream docs).

_(empty until implementation begins)_
