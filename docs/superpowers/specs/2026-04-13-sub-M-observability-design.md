# Sub-project M — Observability framework

**Status:** brainstorm done, awaiting implementation
**Date:** 2026-04-13
**Author:** brainstormed via `superpowers:brainstorming` skill in the mnemosyne-orchestrator backlog plan, Session 7
**Predecessors:** sub-projects E (`2026-04-12-sub-E-ingestion-design.md`), B (`2026-04-12-sub-B-phase-cycle-design.md`), C (`2026-04-13-sub-C-adapters-design.md`)

---

## 1. Purpose

Mnemosyne needs a unified observability framework that serves three masters equally:

1. **Diagnostic-first** — when something goes wrong, give the user (or future maintainer) rich enough context to debug. Sub-project C's accepted Risk 5 ("v1 ships with diagnostic-poor failure modes") is the load-bearing concrete requirement: M must support *"give me the last N events from session X with full context"* as a one-method-call operation.
2. **Live-display-first** — drive the ratatui TUI's status bars, phase progress widgets, harness output panes, and metric sparklines in real time. Drive Obsidian dashboards via Dataview queries against persisted data.
3. **Long-term-analysis-first** — measure how dogfooding the orchestrator is going across many sessions: cold-spawn latency distributions, ingestion success rates, phase-cycle throughput, sentinel-detection precision, error-rate trends.

Equal weighting was chosen deliberately — narrowing to one master would force retrofit work later as the unaddressed masters became urgent. M's complexity budget pays for serving all three.

M also owns the migration path away from per-sub-project tactical instrumentation (currently: C's `SpawnLatencyReport`) onto a single shared framework, and the cross-plan coordination of adoption tasks.

## 2. Principles

1. **Integration over reinvention.** Every component is a standard, top-100-by-downloads Rust crate composed in standard ways. The single piece of custom code is one ~200-line `tracing-subscriber::Layer` that recognises Mnemosyne's typed events. No custom event-bus framework, no custom subscriber registry, no custom rotation logic.
2. **Hard errors by default** (project-wide). Schema mismatches, layer registration failures, channel send errors on bounded channels (other than the deliberate `events.dropped` counter on the TUI bridge), and disk write failures all fail loud.
3. **Type discipline at the Mnemosyne boundary, ecosystem leverage everywhere else.** One canonical `MnemosyneEvent` enum is the single source of truth for *"what events Mnemosyne knows how to observe."* Downstream consumers exhaustively pattern-match on it. Below the boundary, `tracing`'s field/span machinery handles transport, async context, filtering, and third-party crate events.
4. **Always-on instrumentation; tactical measurement disclaims framework scope.** Every event emit is unconditional — no debug flag, no env var, no gated build. Cost per emit (one channel send, one file write through `tracing-appender`'s non-blocking writer) is small enough not to merit a flag. (Project-wide principle, established by sub-project C's `SpawnLatencyReport`.)
5. **Vault-scoped persistence.** Operational data (event JSONL, metric snapshots) lives under `<vault>/runtime/` (gitignored). The historical, user-browsable record lives under `<vault>/projects/<project>/mnemosyne/observability/` (git-tracked, Obsidian-friendly markdown with Dataview frontmatter).
6. **Cross-plan adoption is M's deliverable, not a triage-phase task.** When M's brainstorm lands its sibling implementation plan, it also lands adoption tasks into each existing sibling backlog (sub-B, sub-C, sub-D, sub-E, and once brainstormed sub-F/H/I).

## 3. Crate stack — all standard

| Concern | Crate | Why |
|---|---|---|
| Instrumentation API | `tracing` | de facto Rust standard; zero-cost when disabled |
| Subscriber framework | `tracing-subscriber` (`Registry` + `EnvFilter` + `Layer`) | standard composition pattern; `Registry` is the canonical multi-layer host |
| Non-blocking file writer | `tracing-appender` | standard non-blocking guard + rolling-file appender |
| Metrics facade | `metrics` | typed `counter!`/`gauge!`/`histogram!` macros, allocation-free hot path |
| Metrics aggregation | `metrics-util` (`Registry` + bucket histograms) | standard percentile computation |
| Stderr fmt | `tracing-subscriber::fmt::Layer` | standard dev logging |
| JSON serialisation | `serde` + `serde_json` | already in tree |

**Total new crate dependencies: 5** (`tracing`, `tracing-subscriber`, `tracing-appender`, `metrics`, `metrics-util`). All are top-100 by downloads on crates.io. No custom replacements.

Choices the spec deliberately does NOT make:

- No `tokio-console` integration in v1 (it works against any `tracing` setup automatically; M doesn't need to commit to it).
- No Prometheus / OTel exporter in v1 (deferred to v2; the `metrics` crate's exporter ecosystem makes this additive).
- No `tracing-flame` or `tracing-tree` in v1 (additive layers any user can plug in via `MNEMOSYNE_LOG`).

## 4. Event model

### 4.1 The canonical typed event

```rust
/// The single source of truth for "what Mnemosyne knows how to observe."
/// Every downstream consumer pattern-matches exhaustively on this enum.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MnemosyneEvent {
    /// Wraps B's PhaseEvent lifecycle variants (PhaseStarted, PhaseExited,
    /// PhaseInterrupted). Forwarded from PhaseRunner via parallel emit
    /// during the migration window, then consumed only via M.
    PhaseLifecycle(PhaseLifecycleKind),

    /// Wraps C's OutputChunkKind::SessionLifecycle ("ready",
    /// "turn_complete:<subtype>", "exited:<exit_code>"). Distinct from
    /// PhaseLifecycle: this is the harness's protocol state, not the
    /// Mnemosyne phase state machine.
    HarnessLifecycle(HarnessLifecycleKind),

    /// Wraps the rest of OutputChunkKind (Stdout, Stderr, ToolUse,
    /// ToolResult, InternalMessage). Carries a truncated text summary
    /// plus the byte length, NOT the full chunk text — full text lives
    /// in the harness session transcript at <vault>/runtime/transcripts/
    /// and the ring buffer keeps tails, not full payloads.
    HarnessOutput {
        kind: OutputChunkKind,
        text_summary: String,   // truncated to 256 bytes
        byte_len: usize,        // full chunk length for size accounting
    },

    /// Wraps E's IngestionEvent (Applied, PromptRequired, Deferred,
    /// Rejected, ResearchSession, CycleSummary). Forwarded via parallel
    /// emit during the migration window; eventually E's channel collapses
    /// into M's bus.
    Ingestion(IngestionEventPayload),

    /// Counter / gauge / histogram update from the `metrics` crate,
    /// bridged through MetricsRecorderLayer so metrics share the same
    /// persistence path as everything else.
    Metric(MetricUpdate),

    /// Ad-hoc structured log record. Covers events from third-party crates
    /// that don't carry the `mnemosyne_event` field, and Mnemosyne's own
    /// non-categorised diagnostics. Replaces the `eprintln!`-style logging
    /// scattered through current draft code.
    Diagnostic {
        level: tracing::Level,
        target: String,
        message: String,
        fields: serde_json::Value,
    },

    /// A typed error event with structured context. Distinct from
    /// Diagnostic { level: ERROR } — Error events have a typed variant
    /// and trigger the Risk 5 dump path.
    Error {
        context: ErrorContext,  // session_id, plan_id, phase, originating component
        error: TypedError,      // serialised AdapterError / ExecutorError / IngestionError
    },
}
```

Helper enums (`PhaseLifecycleKind`, `HarnessLifecycleKind`, `MetricUpdate`, `ErrorContext`, `TypedError`) are defined alongside in `src/observability/event.rs` and follow the same exhaustive-variant discipline.

### 4.2 Emission via `tracing::event!`

Mnemosyne code emits events through a thin macro wrapper that ultimately calls `tracing::event!` with a single typed payload field:

```rust
mnemosyne_event!(
    Level::INFO,
    MnemosyneEvent::PhaseLifecycle(PhaseLifecycleKind::Started {
        phase: Phase::Work,
        plan_id: ctx.plan_id.clone(),
        at: Utc::now(),
    })
);
```

Expands roughly to:

```rust
tracing::event!(
    target: "mnemosyne",
    Level::INFO,
    mnemosyne_event = ?event
);
```

The `?event` form uses `tracing`'s `Debug` field encoding; the custom `MnemosyneEventLayer` (§5) recognises the field, downcasts to `MnemosyneEvent` via a small unsafe-free trick (the macro stores the enum in a thread-local immediately before the `event!` call and the layer reads it immediately after; both run synchronously on the same thread), and dispatches the typed payload.

> **Implementation note for v1.** If the thread-local handoff feels brittle, the alternative is `tracing-subscriber`'s `Visit` API with `serde_json` round-trip via the `Debug` field — slower but simpler. The implementation phase picks based on a microbenchmark; the choice does not affect this design.

Third-party crate events (e.g., `reqwest` during a research session) flow through the standard subscriber path with no `mnemosyne_event` field; the `MnemosyneEventLayer` notices the absence and routes them to `MnemosyneEvent::Diagnostic` automatically, capturing the standard `tracing` fields verbatim.

### 4.3 Why this hybrid (typed events + `tracing` transport)?

Two project principles point in opposite directions:

- **"Integration over reinvention"** says use `tracing` as-is, fields and all. But pure-`tracing` gives up exhaustive matching on event variants — every downstream consumer becomes a `match` against stringly-typed field names.
- **"Every state transition is a typed message; hard errors by default"** says build a custom `MnemosyneEvent` enum. But pure-typed-bus gives up `tracing`'s span machinery, async-context tracking, third-party-crate ecosystem, and the entire `tracing-subscriber` Layer pattern.

The hybrid honours both: M owns the type-discipline at the *event payload* boundary, `tracing` owns transport / spans / filtering / third-party integration. The custom code is bounded to one Layer (≈200 lines).

## 5. Subscriber stack

Composed via `tracing_subscriber::Registry::default().with(Layer1).with(Layer2)…` at Mnemosyne process startup. All Layers run for every event in registration order; cheap layers (filtering) run first, expensive layers (file IO via non-blocking guard) run last.

| Layer | Purpose | Storage | Bounded? |
|---|---|---|---|
| **`EnvFilter`** | parses `MNEMOSYNE_LOG` env var; filters by target + level before any payload work happens | — | no |
| **`MnemosyneEventLayer`** | recognises events with the `mnemosyne_event` field; parses into typed `MnemosyneEvent`; dispatches to all internal subscribers via a single in-process broadcast | in-process | no |
| **`InMemoryRingLayer`** | per-session bounded `VecDeque<MnemosyneEvent>` (default cap 1000); provides `dump_session(session_id, n)` → Vec | in-memory | yes |
| **`JsonlPersistLayer`** | wraps `tracing-appender::non_blocking` writing to per-session JSONL files | file | non-blocking writer queue is bounded; overflow drops with counter increment |
| **`MetricsRecorderLayer`** | bridges `metrics` crate updates into `MnemosyneEvent::Metric`; backs a `metrics_util::Registry` for percentile snapshots | in-memory + file at session end | no (metric registry is unbounded by name; bounded in practice by the catalogue in §6) |
| **`TuiBridgeLayer`** | bounded `mpsc::Sender<MnemosyneEvent>` to ratatui; on overflow drops oldest with `events.dropped` counter increment | channel (cap 256) | yes |
| **`ObsidianMaterialiseLayer`** (v1.5) | at session end, writes Dataview-friendly markdown summary | file | no |
| **`StderrFmtLayer`** | standard `tracing-subscriber::fmt`; filtered by `EnvFilter`; off by default | stderr | no |

**The `MnemosyneEventLayer` is the only piece of new custom code.** Every other layer is either a stock `tracing-subscriber` layer or a thin (~30-50 line) custom layer that consumes the broadcast feed.

### 5.1 Layer composition order

```
Registry
  → EnvFilter (drop noise early)
  → MnemosyneEventLayer (parse typed payload)
  → InMemoryRingLayer (cheap, in-memory)
  → MetricsRecorderLayer (cheap, in-memory)
  → TuiBridgeLayer (cheap, channel send)
  → JsonlPersistLayer (file IO via non-blocking guard)
  → ObsidianMaterialiseLayer (only at session end, no per-event cost; v1.5)
  → StderrFmtLayer (only when MNEMOSYNE_LOG is set; off by default)
```

Order is deliberate: cheap layers run before expensive ones so that filtering decisions and ring-buffer updates happen even if a downstream layer slows down.

### 5.2 Session scoping

Every event carries an optional `session_id` (set when the event is emitted from inside an active harness session) and an optional `plan_id` (set from the surrounding `tracing::Span`). `InMemoryRingLayer` keys its ring buffer by `session_id`; events without a session id go to a "process-scoped" ring shared across the Mnemosyne process. `JsonlPersistLayer` uses both keys to choose the output file path.

Spans are entered via `tracing::info_span!("phase", plan_id = %ctx.plan_id, phase = ?phase)` at `PhaseRunner::run_phase` entry, and `tracing::info_span!("harness_session", plan_id = %ctx.plan_id, session_id = %session.id())` at `LlmHarnessExecutor::execute` entry. Span context propagates through `tracing`'s standard mechanism, including across `tokio` task boundaries via `tracing::Instrument`.

## 6. Metrics catalogue (v1)

All metric names are `const &'static str` constants in `src/observability/metric_names.rs`. Typos fail at compile time — this is the typed-discipline answer for metric names without abandoning the `metrics` crate.

```rust
pub mod metric_names {
    // Counters
    pub const PHASE_STARTED: &str = "phase.started";
    pub const PHASE_EXITED_CLEAN: &str = "phase.exited.clean";
    pub const PHASE_EXITED_ERROR: &str = "phase.exited.error";
    pub const PHASE_INTERRUPTED: &str = "phase.interrupted";
    pub const HARNESS_SPAWNED: &str = "harness.spawned";
    pub const HARNESS_EXITED_CLEAN: &str = "harness.exited.clean";
    pub const HARNESS_EXITED_ERROR: &str = "harness.exited.error";
    pub const INGESTION_APPLIED: &str = "ingestion.applied";
    pub const INGESTION_DEFERRED: &str = "ingestion.deferred";
    pub const INGESTION_REJECTED: &str = "ingestion.rejected";
    pub const SENTINEL_MATCHED: &str = "sentinel.matched";
    pub const EVENTS_DROPPED: &str = "events.dropped";

    // Histograms (all milliseconds)
    pub const PHASE_DURATION_MS: &str = "phase.duration_ms";
    pub const HARNESS_COLD_SPAWN_LATENCY_MS: &str = "harness.cold_spawn_latency_ms";
    pub const HARNESS_FIRST_CHUNK_LATENCY_MS: &str = "harness.first_chunk_latency_ms";
    pub const HARNESS_FIRST_OUTPUT_LATENCY_MS: &str = "harness.first_output_latency_ms";
    pub const INGESTION_CYCLE_DURATION_MS: &str = "ingestion.cycle_duration_ms";
    pub const SENTINEL_MATCH_LATENCY_MS: &str = "sentinel.match_latency_ms";

    // Gauges
    pub const HARNESS_LIVE_SESSIONS: &str = "harness.live_sessions";
    pub const INGESTION_IN_FLIGHT_OPS: &str = "ingestion.in_flight_ops";
}
```

Call sites use the constants:

```rust
use crate::observability::metric_names::*;
metrics::counter!(PHASE_STARTED).increment(1);
metrics::histogram!(HARNESS_COLD_SPAWN_LATENCY_MS).record(latency_ms);
```

A compile-time test in `tests/observability_test.rs` enumerates all `metric_names::*` constants and asserts each is documented in this design doc. Adding a metric without updating the catalogue fails CI.

## 7. Storage layout

```
<vault>/runtime/                                            # B's runtime subtree
├── events/
│   └── <plan-id>/
│       ├── <session-id>.jsonl                              # one JSONL file per harness session
│       └── process.jsonl                                   # process-scoped events (no session)
├── metrics/
│   └── <plan-id>/
│       └── <session-id>.json                               # snapshot at session end
└── interrupted/
    └── <plan-id>/
        └── <phase>-<timestamp>/
            └── event-tail.json                             # Risk 5 dump (last N events)

<vault>/projects/<project-name>/mnemosyne/observability/    # git-tracked, Obsidian-facing
└── sessions/
    └── <session-id>.md                                     # Dataview frontmatter + summary
```

`runtime/events/` and `runtime/metrics/` are gitignored — operational data, transient. The `observability/sessions/` markdown summaries under `<vault>/projects/...` are git-tracked because they're the historical record users browse via Dataview.

### 7.1 JSONL format

One `MnemosyneEvent` per line, encoded via `serde_json::to_string`, terminated by `\n`. Every line is independently parseable; truncation at any point leaves earlier lines intact. Schema:

```json
{
  "ts": "2026-04-13T14:23:45.123Z",
  "plan_id": "mnemosyne-orchestrator",
  "session_id": "01HXYZ...",
  "span_path": ["phase:work", "harness_session:01HXYZ..."],
  "event": { /* MnemosyneEvent variant */ }
}
```

`span_path` records the active span chain at emit time, derived from `tracing::Span::current()` walked upward.

### 7.2 Metrics snapshot format (`metrics/<plan-id>/<session-id>.json`)

```json
{
  "session_id": "01HXYZ...",
  "plan_id": "mnemosyne-orchestrator",
  "started_at": "2026-04-13T14:20:00Z",
  "ended_at": "2026-04-13T14:35:12Z",
  "counters": {
    "phase.started": 1,
    "phase.exited.clean": 1,
    "harness.spawned": 1,
    "ingestion.applied": 7
  },
  "histograms": {
    "harness.cold_spawn_latency_ms": {
      "count": 1, "sum": 2340, "min": 2340, "max": 2340,
      "p50": 2340, "p95": 2340, "p99": 2340
    }
  },
  "gauges": {
    "harness.live_sessions": 1
  }
}
```

Computed via `metrics_util::Snapshotter` at session end. Histogram percentiles use the standard exponential bucket layout from `metrics-util`.

### 7.3 Obsidian session summary format (v1.5)

Markdown with kebab-case YAML frontmatter (per the project's Obsidian-native format discipline):

```markdown
---
session-id: 01HXYZ...
plan-id: mnemosyne-orchestrator
phase: work
started-at: 2026-04-13T14:20:00Z
ended-at: 2026-04-13T14:35:12Z
exit-status: clean
harness-cold-spawn-latency-ms: 2340
harness-first-chunk-latency-ms: 2780
ingestion-applied-count: 7
ingestion-deferred-count: 0
sentinel-matched: true
tags: [mnemosyne/observability/session, mnemosyne/plan/mnemosyne-orchestrator]
---

# Session 01HXYZ... — Work phase, mnemosyne-orchestrator

Brief textual summary auto-generated from event tail.

## Events of note
[[2026-04-13-1423-phase-started]]
[[2026-04-13-1424-harness-spawned]]
...

## Errors
None.
```

Wikilinks point to per-event note stubs; the v1.5 `ObsidianMaterialiseLayer` produces these stubs as part of materialisation. v1 ships without these summaries — the JSONL and metric snapshots cover the data needs for the dogfood acceptance test, and Obsidian materialisation can land additively.

## 8. Risk 5 resolution — diagnostic-poor failure modes

Sub-project C's accepted Risk 5 says: *"v1 ships with diagnostic-poor failure modes — when a session fails in a way the actor doesn't anticipate, the user sees the message but no rich context."*

M's resolution:

1. **`InMemoryRingLayer::dump_session(session_id, n) -> Vec<MnemosyneEvent>`** is the one-method-call diagnostic primitive. It returns the last `n` events for the given session in chronological order.
2. **Every error path in B's `PhaseRunner`, C's actor, and E's pipeline** calls `dump_session` on its way out and writes the result to `<vault>/runtime/interrupted/<plan-id>/<phase>-<timestamp>/event-tail.json`, alongside B's existing forensics dir.
3. **The TUI's error display** surfaces a "view event tail" action that opens the file.
4. **The default ring size (1000 events)** captures roughly the last 5-15 minutes of activity for typical sessions — sized to fit the C-1 dogfood acceptance test envelope without runaway memory.

This makes Risk 5's requirement *"give me the last N events from session X with full context"* a one-method-call operation. The error-path call is wrapped in a panic-safe helper so a failure inside `dump_session` cannot mask the original error.

## 9. Migration of C's `SpawnLatencyReport` (staged, never breaks C-1)

C ships a tactical `SpawnLatencyReport` struct emitted as an `InternalMessage` chunk and written to `<staging>/spawn-latency.json`. C explicitly disclaims framework scope for this instrumentation. M's job is to absorb it without breaking the C-1 dogfood acceptance gate.

| Phase | C's tactical writer | M's `metric!` calls | InternalMessage chunk | `<staging>/spawn-latency.json` |
|---|---|---|---|---|
| **C v1** (today) | yes | no | yes | yes |
| **M v1 lands** | yes | yes (parallel) | yes | yes |
| **M v1.1** (after parallel-emit period proves M correct) | no | yes | no | no |
| **G migration** (cleanup) | gone (code deleted) | gone (file deleted from staging schema) | — | — |

The parallel-emit window is the safety net: M's instrumentation is verified against C's ground-truth file before C's tactical writer is removed. If M's recorded values diverge from the JSON file, M is wrong and gets fixed before the deletion step.

The verification check is mechanical: a Layer 3 integration test reads both the JSON file and the metric snapshot from the same session and asserts the three latency values match within ±10ms (clock skew allowance). The test runs on every CI build during the parallel-emit window.

## 10. Cross-plan adoption (M's deliverable, not triage scope)

When M's brainstorm lands its sibling implementation plan, it also lands adoption tasks into each existing sibling backlog. **This is M's own deliverable** — not a triage-phase task that may or may not get scheduled.

| Sibling | Adoption task |
|---|---|
| **sub-B-phase-cycle** | Add `tracing::instrument` on `PhaseRunner::run_phase`. Emit `MnemosyneEvent::PhaseLifecycle` at the numbered steps 5 (PhaseStarted), 10 (PhaseExited), 11 (post-reflect-hook). Replace `eprintln!` with `tracing::event!`-via-`mnemosyne_event!` macro. Wire `TuiBridgeLayer`'s `mpsc::Receiver<MnemosyneEvent>` into the TUI's event loop. |
| **sub-C-adapters** | Add `tracing::instrument` to actor handlers. Add the parallel `metric!` calls for cold-spawn / first-chunk / first-output latency alongside C's existing `SpawnLatencyReport` writer (parallel-emit window). After M v1 ships and the verification window passes, delete C's writer. |
| **sub-D-concurrency** | Emit `LockAcquired` / `LockReleased` / `LockContended` events as `MnemosyneEvent::Diagnostic` with `target = "mnemosyne::lock"`. Increment `metrics::counter!("lock.acquired" / ...)`. |
| **sub-E-ingestion** | Wrap each existing `IngestionEvent` emit with a parallel `MnemosyneEvent::Ingestion(_)` emit during the transition window. After verification, collapse E's standalone channel into M's bus and delete E's channel plumbing. |
| **sub-F-hierarchy** | Emit plan-discovery and parent-walk events as `MnemosyneEvent::Diagnostic` with `target = "mnemosyne::hierarchy"`. |
| **sub-H-skills** | Emit `TuiActionInvoked { action }` events as `MnemosyneEvent::Diagnostic` with `target = "mnemosyne::tui_action"`. |
| **sub-I-obsidian** | Document the observability data surfaces (events, metrics, session summaries) as part of the Obsidian coverage doc. Provide example Dataview queries against the v1.5 Obsidian session summary format. |
| **sub-G-migration** | Delete `<staging>/spawn-latency.json` from the staging schema after the M parallel-emit window closes. Migrate any pre-M Mnemosyne v0.1.0 logs (if any exist in user vaults) — none are expected since v0.1.0 has no structured logging. |

Each adoption task is added to the corresponding sibling backlog as a `not_started` task with a dependency on M's v1 implementation. M's brainstorm's session log records that the adoption tasks have been added; the orchestrator's triage phase verifies they're present.

## 11. Display surfaces (v1)

### 11.1 Ratatui TUI

Subscribes via `TuiBridgeLayer`'s `mpsc::Receiver<MnemosyneEvent>`. Renders:

- **Status bar (top)**: live gauges — `harness.live_sessions` (from `MetricsRecorderLayer`), the elapsed time of the current phase (computed locally in the TUI from the most recent `PhaseLifecycle::Started` event timestamp; not a recorded metric), and a rolling count of recent `Error` events.
- **Event tail panel (side)**: last 50 events from the channel, formatted compactly. Updated on each `MnemosyneEvent` arrival.
- **Error display**: when an `Error` event arrives, the TUI offers a "view event tail" action that calls `InMemoryRingLayer::dump_session` and renders the result in a modal pane.
- **Metric sparklines (optional, v1.5)**: small ASCII sparklines for the most recent N values of key histograms. v1 omits these for simplicity.

Implementation lives in sub-B's TUI module; M provides the `MnemosyneEvent` channel and the `dump_session` API as its public interface.

### 11.2 Obsidian (Dataview)

v1 ships JSONL events and metric snapshots. v1.5 ships the `ObsidianMaterialiseLayer` that produces per-session markdown summaries with Dataview-friendly frontmatter.

Example query for finding slow sessions:

```dataview
TABLE plan-id, phase, harness-cold-spawn-latency-ms, exit-status
FROM "projects/Mnemosyne/mnemosyne/observability/sessions"
WHERE harness-cold-spawn-latency-ms > 5000
SORT started-at DESC
```

Example query for ingestion success rate over time:

```dataview
TABLE
  dateformat(started-at, "yyyy-MM-dd") AS day,
  sum(ingestion-applied-count) AS applied,
  sum(ingestion-deferred-count) AS deferred,
  sum(ingestion-rejected-count) AS rejected
FROM "projects/Mnemosyne/mnemosyne/observability/sessions"
GROUP BY day
SORT day DESC
```

Sub-project I's brainstorm absorbs the full query catalogue as part of the Obsidian coverage doc.

### 11.3 Stderr (development)

Standard `tracing-subscriber::fmt` Layer with `EnvFilter` from the `MNEMOSYNE_LOG` env var:

```sh
MNEMOSYNE_LOG=info,mnemosyne::ingestion=debug mnemosyne ...
```

Off by default. Useful for development and debugging, not for normal operation. The TUI is the user's primary live surface.

## 12. Analysis tooling

### 12.1 `mnemosyne metrics` CLI subcommand (v1)

```sh
mnemosyne metrics --plan-id mnemosyne-orchestrator --since 2026-04-01
```

Reads JSONL events from `<vault>/runtime/events/<plan-id>/` and metric snapshots from `<vault>/runtime/metrics/<plan-id>/` for the given range. Computes per-metric percentiles via `metrics-util::Snapshotter::histograms()`. Prints a human-readable report to stdout.

≈300 lines, no new dependencies. Output formats:

- Default: pretty-printed table to stdout
- `--json`: machine-readable JSON
- `--csv` (v1.5+): CSV for piping to external tools

The subcommand lives in `src/commands/metrics.rs` alongside the existing `query` / `promote` / `curate` subcommands.

### 12.2 `mnemosyne diagnose` CLI subcommand (v1)

```sh
mnemosyne diagnose --session 01HXYZ... [--last 50]
```

Reads `<vault>/runtime/events/<plan-id>/<session-id>.jsonl` and prints the last N events in chronological order. Mirrors the in-memory `dump_session` API but works against persisted JSONL after the session has ended. Useful when the live `event-tail.json` dump is missing or insufficient.

### 12.3 Obsidian Dataview queries (v1.5)

Once the `ObsidianMaterialiseLayer` ships, ad-hoc analysis happens in Obsidian directly. The CLI subcommands stay for machine-readable cases and for users not running Obsidian.

## 13. Error handling

Per the project's "hard errors by default" principle:

- **Layer registration failure** (e.g., disk inaccessible, permission denied on `<vault>/runtime/events/`) → fail at process startup with a clear diagnostic. Mnemosyne refuses to run.
- **`tracing-appender` non-blocking writer queue overflow** → drop the event with `events.dropped` counter increment + one-line warning to stderr. Bounded queue is the explicit non-fatal exception, justified by: (a) overflow only happens under sustained pressure beyond the writer's drain rate; (b) blocking the entire Mnemosyne process on disk IO would be worse than dropping events; (c) the counter makes the loss observable.
- **`MnemosyneEventLayer` parse failure** (typed payload deserialisation) → fail loud as `Diagnostic { level: ERROR, message: "MnemosyneEventLayer parse failure", fields: { raw: ... } }`. Indicates a contract mismatch between an emit site and the layer; should fail tests before reaching production.
- **TUI bridge channel full** → drop oldest with `events.dropped` counter increment. Same justification as the appender queue.
- **`InMemoryRingLayer` ring at capacity** → drop oldest. By design — the ring is bounded and circular.
- **Risk 5 dump path failure** (e.g., disk write fails while writing `event-tail.json`) → wrapped in panic-safe helper, logs the secondary failure to stderr, never masks the original error.

Bounded-queue overflow is the only tolerated failure mode. Every other path fails hard.

## 14. Testing

### Unit tests

- `MnemosyneEventLayer` round-trip: emit a `MnemosyneEvent`, verify the layer parses and dispatches the typed payload identically.
- `InMemoryRingLayer`: ring eviction at capacity; `dump_session` chronological order; per-session isolation.
- `JsonlPersistLayer`: round-trip events through a tmpdir; verify line-per-event format; verify truncation safety (every line independently parseable).
- `MetricsRecorderLayer`: counter increment, gauge set, histogram record; snapshot at session end; percentile correctness against `metrics-util` fixtures.
- `TuiBridgeLayer`: bounded channel overflow drops oldest, increments `events.dropped`.
- Metric name catalogue test (CI runtime): enumerate all `metric_names::*` constants via a `build.rs`-generated test fixture, parse the `## 6. Metrics catalogue` table from this design doc, assert one-to-one match between code constants and documented names. Failure mode: a developer adding a metric constant without updating §6 (or vice versa) fails CI.

### Layer 3 integration tests

- **Fixture-replay harness end-to-end**: spawn a fixture-replay `PhaseExecutor`, drive a full work → reflect → triage cycle, verify the JSONL stream contains the expected event sequence and the metric snapshot has the expected counter values.
- **Risk 5 dump end-to-end**: induce a synthetic error in `PhaseRunner`, assert `<vault>/runtime/interrupted/<plan-id>/<phase>-<ts>/event-tail.json` exists, assert it parses as `Vec<MnemosyneEvent>`, assert it contains the last events leading up to the error.
- **C parallel-emit verification** (only during the parallel-emit window): run a real C session, read both `<staging>/spawn-latency.json` and `<vault>/runtime/metrics/.../<session-id>.json`, assert the three latency values match within ±10ms.
- **TuiBridgeLayer wiring**: verify B's TUI receives the expected `MnemosyneEvent` stream during a phase cycle (tested via a mock TUI consumer).

### Property tests

- JSONL round-trip: any `MnemosyneEvent` value round-trips through serialise → write → read → deserialise without loss. Run across all variants via `proptest` strategy.

## 15. v1 / v1.5 / v2 cut

### v1 (this brainstorm's implementation plan)

- Crate stack (§3) integrated.
- `MnemosyneEvent` enum and helper types defined in `src/observability/event.rs`.
- `MnemosyneEventLayer`, `InMemoryRingLayer`, `JsonlPersistLayer`, `MetricsRecorderLayer`, `TuiBridgeLayer`, `StderrFmtLayer` all shipped and wired into a `ObservabilityHarness` startup function.
- Metric catalogue (§6) defined. Call sites in sub-B (phase lifecycle counters / histograms), sub-C (cold-spawn / first-chunk / first-output histograms in parallel-emit mode), and sub-E (ingestion counters / cycle-duration histogram) are all wired by M's adoption tasks (§10). Sub-D / sub-F / sub-H / sub-I call sites are added when those sub-projects' implementation plans land.
- Storage layout (§7.1, §7.2) implemented; vault runtime subtree integrated with B's existing layout.
- Risk 5 dump (§8) wired into B / C / E error paths.
- C's `SpawnLatencyReport` parallel-emit (§9 phase 2) live.
- `mnemosyne metrics` and `mnemosyne diagnose` CLI subcommands shipped (§12.1, §12.2).
- Cross-plan adoption tasks landed in sibling backlogs (§10).
- Test suite (§14) at green.

### v1.5

- `ObsidianMaterialiseLayer` (§5, §7.3, §11.2).
- Deletion of C's tactical `SpawnLatencyReport` writer (§9 phase 3).
- Full Dataview query catalogue absorbed into sub-I's brainstorm output.
- TUI metric sparklines (§11.1).
- `mnemosyne metrics --csv` output format.

### v2

- External metrics export (Prometheus / OpenTelemetry) via the `metrics` crate's exporter ecosystem.
- Distributed tracing across multiple Mnemosyne instances (sub-D concurrency aware).
- Anomaly detection on long-term metric trends.
- `tokio-console` integration documented as a recommended optional layer.

## 16. Open questions

These do not block the implementation plan but should be resolved during v1 build.

| # | Question | Resolution method | Resolution timing |
|---|---|---|---|
| 1 | `MnemosyneEvent` typed-payload handoff: thread-local trick vs. `Visit` API + `serde_json` round-trip | Microbenchmark both approaches against a representative event mix | Day 1 of v1 implementation |
| 2 | Default `InMemoryRingLayer` capacity (1000 events) — too many or too few for the C-1 dogfood envelope? | Measure event volume during the first dogfood run; tune accordingly | After first dogfood cycle |
| 3 | Whether `ObsidianMaterialiseLayer` should land in v1 instead of v1.5 if the dogfood cycle generates user demand | Defer until v1 ships and dogfood feedback arrives | Post-v1 |
| 4 | Whether `tokio-console` should be a recommended layer in v1 docs | Try it during v1 development and decide based on developer ergonomics | During v1 build |
| 5 | Histogram bucket layout: `metrics-util` default vs. custom for latency-friendly resolution | Inspect default percentile accuracy on synthetic latency data | Day 2 of v1 implementation |

## 17. Risks

### Risk 1 — `tracing-subscriber` Layer ordering / re-entrancy bugs
*MEDIUM impact, LOW likelihood.*

Composing multiple custom Layers can produce subtle re-entrancy bugs (e.g., a Layer emits a `tracing::event!` from inside `on_event`, which re-enters the Registry, which re-enters the same Layer). The standard mitigation is well-known: never emit `tracing::event!` from inside a Layer's `on_event`; use direct subscriber-level dispatch instead.

**Mitigation**: code review checklist for every new custom Layer; a `re_entrancy_test.rs` integration test that emits 1M events through the full Registry stack and asserts no panics or stack overflows.

### Risk 2 — Bounded-queue overflows hide real problems
*LOW impact, MEDIUM likelihood.*

The `events.dropped` counter is the observability surface for queue overflow, but if nobody looks at it, drops go unnoticed. A user might miss critical events without realising why.

**Mitigation**: the TUI status bar surfaces `events.dropped` as a non-zero red badge; the `mnemosyne diagnose` CLI subcommand prints a warning if the session's snapshot has any drops; v1's documentation calls out the counter as a thing to watch.

### Risk 3 — Migration window for C's `SpawnLatencyReport` produces inconsistent data
*LOW impact, LOW likelihood.*

During the parallel-emit window, both C's tactical writer and M's `metric!` calls run. If they disagree (clock skew, timing race, off-by-one in the latency computation), the verification check fires and the implementation has to chase the discrepancy.

**Mitigation**: the verification test (§9, §14) catches discrepancies before they reach production; clock skew tolerance is ±10ms which covers expected `Instant::now()` jitter.

### Risk 4 — `MnemosyneEvent` enum becomes a god object
*MEDIUM impact, MEDIUM likelihood.*

Every new event variant grows the enum. As sub-projects land their adoption tasks, the variant list could balloon and the enum becomes a maintenance hot spot.

**Mitigation**: most new events go through `MnemosyneEvent::Diagnostic` (a typed escape hatch with structured fields), not new top-level variants. Adding a new top-level variant requires a brief justification in this design doc's evolution log; the bar is "the new variant must have multiple downstream consumers that pattern-match on it." Diagnostic is the default.

### Risk 5 — `metrics` crate facade adds layering overhead
*LOW impact, LOW likelihood.*

The `metrics` crate's facade pattern adds a vtable indirection per emit. For very high-frequency metrics this could matter.

**Mitigation**: Mnemosyne's metric volume is low (single-digit per phase, double-digit per session). Vtable cost is in the nanoseconds per call; immeasurable against the rest of phase-cycle latency. If a future high-frequency metric appears (e.g., per-token tracking during streaming output), revisit by emitting directly through `tracing::event!` rather than the `metrics` facade.

## 18. Cross-sub-project requirements

### 18.1 To Sub-B — additive, no re-brainstorm needed

- B's `PhaseRunner` adds `tracing::instrument` annotations and `mnemosyne_event!` macro calls at the numbered steps 5, 10, 11. Additive — no changes to the trait surface or the existing `PhaseEvent` channel.
- B's TUI module gains a `tracing-subscriber::Layer`-driven `mpsc::Receiver<MnemosyneEvent>` consumer. M provides the `TuiBridgeLayer`; B's TUI consumes it.
- B's existing `PhaseEvent` channel stays in place during the parallel-emit window. After M v1 ships and verification passes, B's channel can collapse into M's bus (deferred to v1.5 or v2; not blocking).

### 18.2 To Sub-C — additive, no re-brainstorm needed

- C's actor adds `tracing::instrument` annotations on actor handlers. Additive.
- C adds parallel `metric!` calls for cold-spawn / first-chunk / first-output latency alongside the existing `SpawnLatencyReport` writer (§9 parallel-emit window).
- After verification (§9, §14) passes, C deletes the tactical writer. Sub-G owns the staging-schema cleanup.

### 18.3 To Sub-E — additive, no re-brainstorm needed

- E's pipeline emits parallel `MnemosyneEvent::Ingestion(_)` events alongside the existing `IngestionEvent` channel during the parallel-emit window. Additive.
- After verification, E's channel collapses into M's bus. Sub-G or a dedicated triage task owns the deletion.

### 18.4 To Sub-D — adoption task

- D's locking implementation emits `LockAcquired` / `LockReleased` / `LockContended` events as `MnemosyneEvent::Diagnostic` with `target = "mnemosyne::lock"`. Additive — not part of D's core design, just adoption.

### 18.5 To Sub-F, Sub-H, Sub-I — adoption tasks, brainstormed later

These sub-projects haven't been brainstormed yet. M's adoption requirements (§10) attach to their backlogs as soon as their implementation plans exist, via the cross-plan landing protocol described in §10.

### 18.6 To Sub-G — staging-schema cleanup

- G's migration plan absorbs the deletion of `<staging>/spawn-latency.json` from the staging schema (§9 phase 3) and any v0.1.0 log file cleanup. Both are additive to G's existing scope.

## 19. Decision trail (brainstorm summary)

The brainstorm session that produced this spec made five major decisions:

1. **All three masters equally weighted (Q1 → D).** Diagnostic + live + analysis, accepted as the honest medium-large complexity envelope. Narrowing to one master was rejected as deferring inevitable retrofit work.
2. **Hybrid tracing + typed events (Q2 → C).** `tracing` for transport / spans / async / third-party integration; one canonical `MnemosyneEvent` enum for type discipline. Pure-tracing was rejected as losing exhaustive matching; pure-typed-bus was rejected as reinventing too much of `tracing`.
3. **Use existing tooling and libraries wherever possible (user steer).** Five top-100 standard crates; one ~200-line custom Layer. No bespoke event bus, subscriber framework, or rotation logic.
4. **Always-on instrumentation (project principle).** No debug flags, no env-var gates, no build-time toggles. The `MNEMOSYNE_LOG` env var only controls stderr formatting visibility, not whether events are emitted.
5. **M owns cross-plan adoption (this brainstorm's deliverable).** Adoption tasks are landed in sibling backlogs by M's brainstorm output, not deferred to triage.

## 20. Origin

Sub-project M was surfaced during sub-project C's brainstorm on 2026-04-13 (Session 6 of the mnemosyne-orchestrator backlog plan), when C explicitly disclaimed framework scope for its tactical `SpawnLatencyReport` and recorded a forward pointer to a proposed Sub-M (§11.5 of `2026-04-13-sub-C-adapters-design.md`). The mnemosyne-orchestrator triage at the end of Session 6 promoted M from the tail of the backlog to second position (after sub-project A) because every other sub-project's structured-logging needs route through M once it lands.

This brainstorm executed in Session 7 of the mnemosyne-orchestrator backlog plan, on 2026-04-13, using the `superpowers:brainstorming` skill. The user's explicit steer mid-brainstorm was *"use existing tooling and libraries wherever possible. This is not an interesting task."* — which collapsed several remaining clarifying questions in favour of the standard-tool answer at every fork.
