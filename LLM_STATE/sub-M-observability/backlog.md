# Backlog — Sub-project M: Observability Framework

Implementation backlog for sub-project M. All tasks derive from the
design doc at
`{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-M-observability-design.md`
(committed at `53f7d4e`). Consult the spec before starting any task.

Tasks are listed in approximately recommended order, following the
implementation strategy in `{{PLAN}}/memory.md`: setup → metric catalogue
→ custom Layer → standard layers → composition → Risk 5 wiring →
C parallel-emit → CLI subcommands → adoption verification → integration
tests → re-entrancy test. The work phase picks the best next task with
input from the user.

## Task Backlog

### Task 1 — Cargo.toml deps + module skeleton `[setup]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Add the five new M-side deps to `Cargo.toml`:
  `tracing`, `tracing-subscriber` (with the `env-filter`, `fmt`, and
  `registry` features), `tracing-appender`, `metrics`, `metrics-util`.
  Pin to current minor versions and record the pinned versions in
  `{{PLAN}}/memory.md`. Confirm `serde`, `serde_json`, `chrono`,
  `thiserror` are already present (they should be from sub-B / sub-C
  earlier work). Create the empty module skeleton under
  `src/observability/`:
  `mod.rs`, `event.rs`, `metric_names.rs`, `event_layer.rs`,
  `ring_layer.rs`, `jsonl_layer.rs`, `metrics_layer.rs`,
  `tui_bridge.rs`, `harness.rs`. Each file contains only a module-level
  doc comment and `pub use`s as needed for later tasks. Wire
  `pub mod observability;` into `src/lib.rs`. Run `cargo build` to
  confirm the skeleton compiles. No tests yet.
- **Results:** _pending_

### Task 2 — `event.rs`: `MnemosyneEvent` enum + helper types `[types]`
- **Status:** not_started
- **Dependencies:** Task 1
- **Description:** Implement §4.1 of the spec verbatim. Define
  `MnemosyneEvent` enum with all variants
  (`PhaseLifecycle`, `HarnessLifecycle`, `HarnessOutput`, `Ingestion`,
  `Metric`, `Diagnostic`, `Error`) and the helper enums
  (`PhaseLifecycleKind`, `HarnessLifecycleKind`, `MetricUpdate`,
  `ErrorContext`, `TypedError`). Derive `Debug`, `Clone`,
  `serde::Serialize`, `serde::Deserialize` on every type.
  `HarnessOutput` carries `text_summary` (truncated to 256 bytes) and
  `byte_len`, NOT the full chunk text. Unit tests cover serde JSON
  round-trips for every variant. No subscriber wiring yet — pure type
  surface.
- **Results:** _pending_

### Task 3 — `metric_names.rs`: catalogue constants `[types]`
- **Status:** not_started
- **Dependencies:** Task 1
- **Description:** Implement §6 of the spec verbatim. Define
  `pub const &'static str` constants for every counter, histogram, and
  gauge listed in the spec's metric catalogue table. Group by category
  (counters / histograms / gauges) with section comments. No call sites
  yet — just the constant declarations.
- **Results:** _pending_

### Task 4 — Metric catalogue CI parity test `[verification]`
- **Status:** not_started
- **Dependencies:** Task 3
- **Description:** Add a CI runtime test in
  `tests/observability_metric_catalogue_test.rs` that:
  1. Enumerates all `metric_names::*` constants via a
     `build.rs`-generated fixture (or a manual list maintained alongside
     the constants — pick whichever is simpler at implementation time).
  2. Parses the `## 6. Metrics catalogue` section of
     `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-M-observability-design.md`.
  3. Asserts one-to-one match between code constants and documented
     names.
  Failure mode: a developer adding a metric constant without updating
  §6 (or vice versa) fails CI. This locks in metric name discipline
  before any call site exists.
- **Results:** _pending_

### Task 5 — Day-1 microbenchmark: typed-payload handoff `[verification]`
- **Status:** not_started
- **Dependencies:** Task 2 (types must exist)
- **Description:** Resolves §16 Q1 from the spec. Build two minimal
  prototypes of the typed-payload handoff used by `MnemosyneEventLayer`:
  1. **Thread-local trick** — store the `MnemosyneEvent` in a
     thread-local `RefCell<Option<MnemosyneEvent>>` immediately before
     the `tracing::event!` call; the custom Layer reads it from
     `on_event` synchronously on the same thread.
  2. **`Visit` API + `serde_json` round-trip** — emit the event via
     `tracing::event!(... mnemosyne_event = ?event)`, where the field
     value is the `Debug` representation; the custom Layer's `Visit`
     impl extracts the field, deserialises via `serde_json`.
  Run a 100k-event microbenchmark of each, record the per-event cost,
  and pick the winner. Document the choice and the benchmark numbers
  in `{{PLAN}}/memory.md` under "Verified surface." If the gap is
  small (< 2x), prefer (2) for simplicity; if (1) is meaningfully
  faster, use (1) and write a unit test that asserts thread-local
  consumption is synchronous.
- **Results:** _pending_

### Task 6 — `event_layer.rs`: `MnemosyneEventLayer` + `mnemosyne_event!` macro `[layer]`
- **Status:** not_started
- **Dependencies:** Task 2, Task 5 (handoff approach decided)
- **Description:** Implement the custom `tracing-subscriber::Layer` that
  recognises events with the `mnemosyne_event` field and dispatches the
  typed payload to internal subscribers via a single in-process
  broadcast (`crossbeam-channel` or `tokio::sync::broadcast`). Implement
  the `mnemosyne_event!` macro wrapper that emits via `tracing::event!`
  using the chosen handoff approach from Task 5. The Layer is
  approximately 200 lines per the spec's estimate. Unit tests: emit
  every `MnemosyneEvent` variant via the macro; assert the Layer
  parses and dispatches the typed payload identically. Tests also
  cover events from third-party crates (no `mnemosyne_event` field) —
  these must be routed to `MnemosyneEvent::Diagnostic` automatically.
- **Results:** _pending_

### Task 7 — `ring_layer.rs`: `InMemoryRingLayer` `[layer]`
- **Status:** not_started
- **Dependencies:** Task 6
- **Description:** Implement the per-session bounded ring buffer Layer.
  Default capacity 1000 events per session. Keys ring buffers by
  `session_id` (extracted from the active `tracing::Span` at event
  time); events without a session id go to a "process-scoped" ring
  shared across the Mnemosyne process. Public API:
  `dump_session(session_id, n) -> Vec<MnemosyneEvent>` returns the
  last `n` events in chronological order. Unit tests: ring eviction
  at capacity; per-session isolation; `dump_session` chronological
  order; process-scoped ring fallback.
- **Results:** _pending_

### Task 8 — `jsonl_layer.rs`: `JsonlPersistLayer` `[layer]`
- **Status:** not_started
- **Dependencies:** Task 6
- **Description:** Implement the JSONL persistence Layer using
  `tracing-appender::non_blocking`. Output paths follow §7 of the spec:
  `<vault>/runtime/events/<plan-id>/<session-id>.jsonl` for
  session-scoped events, `<vault>/runtime/events/<plan-id>/process.jsonl`
  for process-scoped. Uses a per-session `RollingFileAppender` (one
  file per session, no rotation within a session). The non-blocking
  writer's `WorkerGuard` is held by the `ObservabilityHarness` (Task
  12) for the lifetime of the Mnemosyne process. JSONL format per
  §7.1: one event per line, terminated by `\n`, every line independently
  parseable. Unit tests: round-trip events through a tmpdir; line-per-
  event format; truncation safety (parse the file after a synthetic
  truncation mid-line, assert earlier lines parse).
- **Results:** _pending_

### Task 9 — `metrics_layer.rs`: `MetricsRecorderLayer` `[layer]`
- **Status:** not_started
- **Dependencies:** Task 6
- **Description:** Bridge the `metrics` crate to `MnemosyneEvent::Metric`
  so metrics share the persistence path with everything else. Register
  a custom `metrics::Recorder` (≈50 lines) that captures every
  `counter!` / `gauge!` / `histogram!` call, wraps it in a
  `MnemosyneEvent::Metric(MetricUpdate { ... })`, and emits via the
  `mnemosyne_event!` macro. Back the recorder with a
  `metrics_util::registry::Registry` for percentile snapshots. Public
  API: `snapshot(session_id) -> MetricsSnapshot` returns the
  `MetricsSnapshot` struct shaped per §7.2 of the spec (counters as a
  map, histograms with count/sum/min/max/p50/p95/p99 percentiles
  computed via `metrics-util::Snapshotter`, gauges as a map). Unit
  tests: counter increment, gauge set, histogram record; snapshot at
  session end; percentile correctness against `metrics-util` fixtures;
  `events.dropped` counter is itself emittable via this layer.
- **Results:** _pending_

### Task 10 — `tui_bridge.rs`: `TuiBridgeLayer` `[layer]`
- **Status:** not_started
- **Dependencies:** Task 6
- **Description:** Implement the bounded `mpsc::Sender<MnemosyneEvent>`
  Layer for the ratatui TUI. Default channel capacity 256. Drop-oldest
  semantics on overflow: when the channel is full, pop the oldest
  buffered event, send the new one, increment the `events.dropped`
  counter (which itself flows through `MetricsRecorderLayer`). Public
  API: `subscribe() -> mpsc::Receiver<MnemosyneEvent>` (single
  receiver, owned by the TUI). Unit tests: bounded channel overflow
  drops oldest; counter increment on drop; single-receiver enforcement.
- **Results:** _pending_

### Task 11 — `StderrFmtLayer` wiring with `EnvFilter` `[layer]`
- **Status:** not_started
- **Dependencies:** Task 6
- **Description:** Wire the standard `tracing-subscriber::fmt::Layer`
  with `EnvFilter` reading from the `MNEMOSYNE_LOG` env var. Off by
  default (no env var → no stderr output). Useful for development
  debugging. This Layer does NOT depend on `MnemosyneEventLayer` —
  it formats the raw `tracing::Event` directly via fmt's standard
  formatter. Unit test: set `MNEMOSYNE_LOG=info`, emit a tracing
  event, assert the formatted output appears on stderr.
- **Results:** _pending_

### Task 12 — `harness.rs`: `ObservabilityHarness` startup function `[composition]`
- **Status:** not_started
- **Dependencies:** Tasks 6-11
- **Description:** Compose the full subscriber stack via
  `tracing_subscriber::Registry::default().with(...).with(...)…`.
  Layer composition order per §5.1 of the spec: EnvFilter →
  MnemosyneEventLayer → InMemoryRingLayer → MetricsRecorderLayer →
  TuiBridgeLayer → JsonlPersistLayer → StderrFmtLayer. Public API:
  `ObservabilityHarness::init(vault_runtime_dir, plan_id) -> (Self, WorkerGuard)`.
  The `WorkerGuard` from `tracing-appender::non_blocking` is held by
  `Self` for the process lifetime. `Self` exposes accessor methods
  for the public APIs of each layer that downstream code needs:
  `dump_session(session_id, n)`, `subscribe_tui()`, `metrics_snapshot(session_id)`.
  Unit tests: full Registry composition compiles and runs; `init` is
  idempotent (calling twice within the same process panics with a
  clear diagnostic — this matches `tracing-subscriber`'s
  `set_global_default` semantics).
- **Results:** _pending_

### Task 13 — Risk 5 dump path wiring `[diagnostics]`
- **Status:** not_started
- **Dependencies:** Task 12
- **Description:** Implement the Risk 5 forensics dump per §8 of the
  spec. Add a public function
  `observability::dump_event_tail(harness: &ObservabilityHarness, session_id, plan_id, phase, n) -> Result<PathBuf, IoError>`
  that calls `harness.dump_session(session_id, n)` and writes the
  result to
  `<vault>/runtime/interrupted/<plan-id>/<phase>-<timestamp>/event-tail.json`
  using a panic-safe wrapper (any failure inside the dump path logs
  to stderr but never masks the original error). Wire calls into
  the error paths of B's `PhaseRunner`, C's actor, and E's pipeline
  via the adoption tasks landed in those sibling backlogs. Unit
  test: synthetic error in a fixture context, assert the
  `event-tail.json` file exists with the right schema.
- **Results:** _pending_

### Task 14 — C `SpawnLatencyReport` parallel-emit (M side) `[migration]`
- **Status:** not_started
- **Dependencies:** Task 12
- **Description:** Define the metric histograms M will record
  (`HARNESS_COLD_SPAWN_LATENCY_MS`, `HARNESS_FIRST_CHUNK_LATENCY_MS`,
  `HARNESS_FIRST_OUTPUT_LATENCY_MS`) and document where in C's actor
  the parallel `metric!` calls go. The actual call sites are added
  by sub-C-adapters' adoption task (already landed in C's backlog by
  M's brainstorm). M side of the work: ensure the histograms are
  registered with `MetricsRecorderLayer` and that the `metric_names`
  constants exist (they should, from Task 3). Unit test: emit a
  histogram value, snapshot, assert percentiles include the value.
- **Results:** _pending_

### Task 15 — C parallel-emit verification integration test `[verification]`
- **Status:** not_started
- **Dependencies:** Task 14, sub-C-adapters' adoption task complete
- **Description:** Layer 3 integration test that runs a real C session
  (or a fixture-replay session that exercises the parallel-emit code
  path), reads both `<staging>/spawn-latency.json` and
  `<vault>/runtime/metrics/<plan-id>/<session-id>.json`, asserts the
  three latency values match within ±10ms (clock skew allowance).
  Test runs on every CI build during the parallel-emit window
  (between M v1 ship and M v1.1 cleanup). Once verification passes
  for ≥10 consecutive CI builds, the next triage adds a task to
  delete C's tactical writer (M v1.1 cleanup).
- **Results:** _pending_

### Task 16 — `mnemosyne metrics` CLI subcommand `[cli]`
- **Status:** not_started
- **Dependencies:** Task 9 (snapshot format)
- **Description:** Add a new `metrics` subcommand to the existing
  `mnemosyne` CLI under `src/commands/metrics.rs`. Reads JSONL events
  from `<vault>/runtime/events/<plan-id>/` and metric snapshots from
  `<vault>/runtime/metrics/<plan-id>/` for a given `--plan-id` or
  `--since <ISO8601>` range. Computes per-metric percentiles via
  `metrics-util::Snapshotter::histograms()`. Output formats: default
  pretty-printed table, `--json` for machine-readable. CSV format
  deferred to v1.5+. ≈300 lines. Integration test against a fixture
  vault containing a known event stream.
- **Results:** _pending_

### Task 17 — `mnemosyne diagnose` CLI subcommand `[cli]`
- **Status:** not_started
- **Dependencies:** Task 8 (JSONL format)
- **Description:** Add a new `diagnose` subcommand to the existing
  `mnemosyne` CLI under `src/commands/diagnose.rs`. Reads
  `<vault>/runtime/events/<plan-id>/<session-id>.jsonl` for a given
  `--session` and prints the last `--last N` (default 50) events in
  chronological order. Mirrors the in-memory `dump_session` API but
  works against persisted JSONL after the session has ended. Useful
  when the live `event-tail.json` dump is missing or insufficient.
  Integration test against a fixture session JSONL.
- **Results:** _pending_

### Task 18 — Cross-plan adoption verification `[coordination]`
- **Status:** not_started
- **Dependencies:** none (independent verification step)
- **Description:** **NOTE — most of this work is pre-completed by M's
  brainstorm.** The sub-B-phase-cycle, sub-C-adapters, and
  sub-E-ingestion sibling backlogs each had an adoption task added by
  the brainstorm session that produced this plan. This task is the
  verification step: re-read each sibling backlog and confirm the
  adoption stub is present, correctly worded, and references this
  plan's design doc. If any stub is missing or out of date, fix it
  and record the fix in this plan's session log. Also re-check
  `{{PLAN}}/memory.md`'s "Cross-plan adoption coordination" section
  to confirm the queued adoptions for sub-D / sub-F / sub-H / sub-I /
  sub-G are still queued (none of those sub-projects' brainstorms
  should have shipped an adoption task in advance — that would be a
  drift signal).
- **Results:** _pending_

### Task 19 — Layer 3 integration test: fixture-replay end-to-end `[testing]`
- **Status:** not_started
- **Dependencies:** Task 12, Task 13
- **Description:** Spawn a fixture-replay `PhaseExecutor` (from sub-B),
  drive a full work → reflect → triage cycle, verify:
  1. The JSONL stream at `<vault>/runtime/events/<plan-id>/<session-id>.jsonl`
     contains the expected event sequence.
  2. The metric snapshot at
     `<vault>/runtime/metrics/<plan-id>/<session-id>.json` has the
     expected counter values (`phase.started == 3`,
     `phase.exited.clean == 3`, `harness.spawned == 3`).
  3. The TuiBridgeLayer's mpsc receiver delivered the expected
     event count to a mock TUI consumer.
  4. The InMemoryRingLayer can dump the last 50 events for the cycle's
     final session.
  Uses sub-B's `FixtureReplayExecutor` from B's implementation phase;
  blocks on B's executor being landed (or stub equivalents being
  available).
- **Results:** _pending_

### Task 20 — Risk 5 dump end-to-end test `[testing]`
- **Status:** not_started
- **Dependencies:** Task 13
- **Description:** Layer 3 integration test that induces a synthetic
  error in a fixture `PhaseRunner`, asserts
  `<vault>/runtime/interrupted/<plan-id>/<phase>-<timestamp>/event-tail.json`
  exists, asserts it parses as `Vec<MnemosyneEvent>`, asserts it
  contains the last events leading up to the error in chronological
  order. Verifies that the dump path is panic-safe by injecting a
  synthetic disk-full error during the dump itself and confirming
  the original error is still surfaced (not masked).
- **Results:** _pending_

### Task 21 — JSONL property tests `[testing]`
- **Status:** not_started
- **Dependencies:** Task 8
- **Description:** Use `proptest` to generate random `MnemosyneEvent`
  values across all variants, serialise to JSONL, parse back, assert
  round-trip equality. Cover edge cases: empty strings, very long
  `text_summary` fields (tests the truncation invariant), all
  `OutputChunkKind` variants, all `Phase` variants. The property
  strategy lives in `src/observability/event.rs` behind a
  `#[cfg(test)]` block.
- **Results:** _pending_

### Task 22 — Re-entrancy stress test `[testing]`
- **Status:** not_started
- **Dependencies:** Task 12
- **Description:** Risk 1 mitigation. Integration test that emits 1M
  `MnemosyneEvent`s through the full Registry stack across multiple
  threads (4 emitter threads × 250k events each), asserts no panics,
  no stack overflows, and that all events arrive at every expected
  layer. Verifies the well-known "never emit `tracing::event!` from
  inside a Layer's `on_event`" rule is held; test will fail loudly
  if any future custom layer violates it. Also serves as a smoke
  test for sustained-load correctness of the non-blocking
  `tracing-appender` writer.
- **Results:** _pending_

### Task 23 — v1 ship gate `[release]`
- **Status:** not_started
- **Dependencies:** Tasks 1-22
- **Description:** Final v1 acceptance:
  1. All tasks 1-22 are `done`.
  2. `cargo test` green at the workspace level.
  3. `cargo clippy --all-targets` green (no new warnings).
  4. `cargo +nightly fmt --check` green.
  5. Spec §15 v1 cut checklist confirmed:
     - ✅ Crate stack integrated
     - ✅ `MnemosyneEvent` enum + helper types
     - ✅ All seven layers shipped and wired
     - ✅ Metric catalogue defined
     - ✅ Storage layout implemented
     - ✅ Risk 5 dump wired into B / C / E error paths
     - ✅ C parallel-emit live and verified
     - ✅ `mnemosyne metrics` and `mnemosyne diagnose` subcommands
     - ✅ Cross-plan adoption tasks landed in sibling backlogs
     - ✅ Test suite green
  6. Update parent orchestrator plan's `memory.md` recording M v1
     ship.
  7. The next triage cycle of sub-M (now post-v1) creates the v1.1
     cleanup task: delete C's tactical `SpawnLatencyReport` writer
     after the verification window has run for ≥10 consecutive CI
     builds.
- **Results:** _pending_
