# Sub-project M — Observability framework

---

## 1. Purpose

Mnemosyne needs a unified observability framework that serves three masters equally:

1. **Diagnostic-first** — when something goes wrong, give the user (or future maintainer) rich enough context to debug. Sub-project C's accepted Risk 5 ("v1 ships with diagnostic-poor failure modes") is the load-bearing concrete requirement: M must support *"give me the last N events from session X with full context"* as a one-`GenServer.call` operation.
2. **Live-display-first** — drive the Rust `ratatui` TUI's status bars, phase progress widgets, harness output panes, and metric sparklines in real time via the daemon's Unix-socket NDJSON protocol (sub-F §6/§7). Drive Obsidian dashboards via Dataview queries against persisted data.
3. **Long-term-analysis-first** — measure how dogfooding the orchestrator is going across many sessions: cold-spawn latency distributions, ingestion success rates, phase-cycle throughput, sentinel-detection precision, error-rate trends, actor-restart rates.

Equal weighting was chosen deliberately — narrowing to one master would force retrofit work later as the unaddressed masters became urgent. M's complexity budget pays for serving all three.

M also owns the migration path away from per-sub-project tactical instrumentation (currently: C's `%SpawnLatencyReport{}` emitted to `<staging>/spawn-latency.json` + `:internal_message` delivery) onto a single shared framework, and the cross-plan coordination of adoption tasks.

## 2. Principles

1. **Integration over reinvention.** Every component is a standard BEAM library composed in standard ways: `:telemetry` for transport, `:telemetry_metrics` for metric definitions, `:telemetry_metrics_prometheus` (optional v1.5+) for export, `Jason` for JSON, stdlib `GenServer` / `DynamicSupervisor` / `File` for long-lived subscribers. The single pieces of custom code are one `Mnemosyne.Observability.Handler` module (attached via `:telemetry.attach_many/4`) and four small GenServers (ring buffer, JSONL writer, TUI bridge, metrics aggregator). No custom event bus, no custom handler registry, no custom rotation logic.
2. **Hard errors by default** (project-wide). Handler attach failures, schema mismatches, disk write failures on subscriber initialisation, and `:telemetry_metrics` reporter start failures all fail loud during the 11-step daemon boot sequence (sub-A §A.Reference algorithm) and abort startup. Post-boot, bounded-mailbox overflow on the TUI bridge is the only tolerated non-fatal failure mode, justified by the `events.dropped` counter.
3. **Type discipline at the Mnemosyne boundary, ecosystem leverage everywhere else.** The sealed `Mnemosyne.Event.*` struct set is the single source of truth for *"what events Mnemosyne knows how to observe."* Downstream consumers pattern-match exhaustively on the struct name and field set. Below the boundary, `:telemetry` handles transport, event-name prefixes, span measurements, and third-party-library integration. Code below the boundary emits `:telemetry.execute/3` with the typed struct in `metadata.event`; handlers above the boundary downcast on `metadata.event.__struct__`.
4. **Always-on instrumentation; tactical measurement disclaims framework scope.** Every event emit is unconditional — no debug flag, no env var, no gated build. Cost per emit (one `:telemetry.execute/3` handler lookup, one `GenServer.cast` to the JSONL writer, one `:ets.update_counter/3` on metric tables) is small enough not to merit a flag. (Project-wide principle, established by sub-project C's `%SpawnLatencyReport{}` seed.)
5. **Vault-scoped persistence.** Operational data (event JSONL, metric snapshots) lives under `<vault>/runtime/` (gitignored, per sub-A §A5). The historical, user-browsable record lives under `<vault>/projects/<project>/mnemosyne/observability/` (git-tracked, Obsidian-friendly markdown with Dataview frontmatter). M's subtree integrates with A's runtime layout without collision.
6. **Cross-plan adoption is M's deliverable, not a triage-phase task.** When M's brainstorm lands its sibling implementation plan, it also lands adoption tasks into each existing sibling backlog (sub-B, sub-C, sub-D, sub-E, sub-F, plus once brainstormed sub-H/I).

## 3. Hex dep stack — all standard BEAM libraries

| Concern | Library | Why |
|---|---|---|
| Instrumentation API | `:telemetry` | de facto BEAM standard; zero-cost when no handlers attached; already pulled in by sub-C (`erlexec` chain) and F |
| Metric definitions | `:telemetry_metrics` | reporter-independent typed counter / last_value / sum / summary / distribution; ships from the `telemetry_metrics` Hex package |
| Metric reporter (v1) | `Telemetry.Metrics.ConsoleReporter` | stdlib-level console output for dev / dogfood cycles; no external dep |
| Metric reporter (v1.5+) | `:telemetry_metrics_prometheus` | Prometheus text-format endpoint; chosen over `prom_ex` because `prom_ex` bundles Phoenix/Ecto/Broadway plugins Mnemosyne does not consume |
| JSON serialisation | `Jason` | already in tree from sub-C and sub-F |
| Non-blocking file IO | stdlib `File.open(..., [:append, :raw, :binary, :delayed_write])` + a writer `GenServer` per session | no `tracing-appender`-style guard needed; `:delayed_write` batches writes and a supervised GenServer holds the file handle |
| Long-lived subscribers | `GenServer` + `DynamicSupervisor` + `:ets` | stdlib; ring buffer per session, metric aggregator, TUI bridge each run as their own GenServer |
| Backend logs | `Logger` (stdlib) with default console backend | used for third-party-library events that do not carry a typed struct; bridged via the handler module |

Choices the spec deliberately does NOT make:

- **No `prom_ex`** in v1 — ships its own Plug endpoint, dashboards, and grafana integration that Mnemosyne will not use. If a user wants Prometheus, the lightweight `:telemetry_metrics_prometheus` reporter is additive and can ship in v1.5 without disturbing v1.
- **No OpenTelemetry (`:opentelemetry`) exporter** in v1 — deferred to v2. `:telemetry` events can be bridged to OTel later via a handler module with no upstream changes.
- **No `observer_cli`** dependency in v1 — documented as a recommended optional layer developers can add via `mix.exs`, not shipped as a framework surface.
- **No `Phoenix.PubSub`** for the TUI bridge — Phoenix is a large transitive dependency for a single fan-out concern. We use `:pg` (stdlib since OTP 23) instead, which gives process-group fan-out with no extra hex deps.

## 4. Event model

### 4.1 The sealed `Mnemosyne.Event.*` struct set

The sealed set is the single source of truth for *"what Mnemosyne knows how to observe."* Every struct is defined under `lib/mnemosyne/event/` with `@enforce_keys` + `defstruct` and derives `Jason.Encoder`. Downstream consumers pattern-match on `%Struct{...}`.

```elixir
defmodule Mnemosyne.Event.PhaseLifecycle do
  @moduledoc """
  Phase-cycle state transitions emitted by sub-B's PhaseRunner.
  See sub-B design doc §4.4 for emission points.
  """
  @derive Jason.Encoder
  @enforce_keys [:kind, :qualified_id, :at]
  defstruct [
    :kind,            # :started | :exited_clean | :reflect_hook_fired
                      # | :interrupted | :executor_failed
                      # | :takeover_offered | :prior_interrupt_surfaced
    :phase,           # :work | :reflect | :compact | :triage (nil for :reflect_hook_fired)
    :qualified_id,    # path-based qualified plan id (sub-F)
    :executor_kind,   # :llm_harness | :manual_editor (nil unless :started)
    :transitioned_to, # next phase for :exited_clean
    :forensics_dir,   # Path for :interrupted / :takeover_offered / :prior_interrupt_surfaced
    :options,         # takeover options list for :takeover_offered
    :error,           # typed error term for :executor_failed
    :at               # DateTime
  ]
end
```

The other structs follow the same shape. The full sealed set, categorised by producer:

**From sub-B (phase cycle, §4.4):**
- `%Mnemosyne.Event.PhaseLifecycle{}` — seven `kind` variants as above.

**From sub-C (harness adapter, §3.3 + §11.4):**
- `%Mnemosyne.Event.HarnessOutput{kind, text_summary, byte_len, session_id, qualified_id, at}` — `kind` is `:stdout | :stderr | :tool_use | :tool_result | :internal_message`. Carries a truncated 256-byte text summary plus the full byte length. **Full chunk text lives in the harness session transcript** at `<vault>/runtime/transcripts/<session-id>.jsonl`; the ring buffer keeps tails only. Forwarded by sub-B's `LlmHarnessExecutor` when consuming C's session GenServer.
- `%Mnemosyne.Event.SessionLifecycle{transition, session_id, qualified_id, at}` — `transition` is `:ready | {:turn_complete, subtype} | {:exited, reason}`. Protocol-level signal; distinct from `PhaseLifecycle` (phase state machine) and from task-level sentinel detection (B's policy layer).
- `%Mnemosyne.Event.SpawnLatencyReport{session_id, cold_spawn_ms, first_chunk_ms, first_output_ms, adapter_kind, at}` — canonical shape preserved from C's seed. C's three-way parallel emission (§7.2 of C's spec) survives unchanged through the parallel-emit window.
- `%Mnemosyne.Event.SessionExitStatus{session_id, reason, exit_code, adapter_kind, at}` — `reason` is one of `:clean | :signaled | :timeout | :port_lost | {:error, term}`.
- `%Mnemosyne.Event.HarnessError{session_id, kind, message, context, at}` — `kind` tags the error category (`:spawn_failure | :stream_parse_error | :tool_call_boundary_error | :unexpected_exit`).

**From sub-F (hierarchy + routing, Task 24):**
- `%Mnemosyne.Event.ActorStateChange{actor_id, kind, from_state, to_state, reason, at}` — `kind` is `:started | :restarted | :terminated | :phase_entered | :phase_exited`. `actor_id` is a qualified plan id or expert id.
- `%Mnemosyne.Event.MessageRouted{message_id, message_kind, from_actor, to_actor, via, decision, at}` — `message_kind` is `:dispatch | :query`; `via` is `:declarative_rule | :level2_routing_agent | :direct`.
- `%Mnemosyne.Event.RuleFired{rule_id, clause, facts, outcome, at}` — `outcome` is `{:target_plan, qid} | {:target_expert, id} | :no_route`.
- `%Mnemosyne.Event.RuleCompileError{file, line, reason, at}` — emitted when `routing.ex` fails hot-reload compilation; must not crash the daemon.
- `%Mnemosyne.Event.RuleSuggestion{rule_candidate, context, trigger_agent, at}` — learning-loop event emitted by the Level 2 routing agent.
- `%Mnemosyne.Event.DispatchProcessed{dispatch_id, origin_plan, target_plan, outcome, at}` — emitted after `DispatchProcessor` writes to the target's `Received` section.
- `%Mnemosyne.Event.QueryAnswered{query_id, origin_plan, target_actor, answer_summary, at}` — emitted after `QueryProcessor` delivers a reply.
- `%Mnemosyne.Event.Level2Invoked{message_id, target_project, reason, at}` — emitted when declarative rules do not decide and L2 fallback starts.
- `%Mnemosyne.Event.Level2Rejected{message_id, reason, retarget, at}` — emitted when L2 rejects a dispatch with a retarget proposal.

**From sub-E (ingestion pipeline, §5):**
- `%Mnemosyne.Event.Ingestion.Applied{candidate_id, store_path, at}`
- `%Mnemosyne.Event.Ingestion.PromptRequired{candidate_id, reason, at}`
- `%Mnemosyne.Event.Ingestion.Deferred{candidate_id, reason, at}`
- `%Mnemosyne.Event.Ingestion.Rejected{candidate_id, reason, at}`
- `%Mnemosyne.Event.Ingestion.ResearchSession{candidate_id, session_id, at}`
- `%Mnemosyne.Event.Ingestion.CycleSummary{cycle_id, applied_count, deferred_count, rejected_count, duration_ms, at}`

Post-F, Stage 5 "dispatch to experts" emits a `MessageRouted{message_kind: :query, via: :declarative_rule, to_actor: <expert-id>}` as part of the same flow. E's existing channel plumbing collapses into M's bus during the parallel-emit window (see §9).

**From sub-A (vault bootstrap, §A amendment):**
- `%Mnemosyne.Event.Vault.BootReady{vault_path, project_count, started_at, ready_at}` — emitted as step 11 of the 11-step daemon boot sequence.
- `%Mnemosyne.Event.Vault.MarkerError{path, reason}` — emitted before the daemon exits non-zero on vault-identity parse failure.

**Generic escape hatches (M-owned):**
- `%Mnemosyne.Event.Metric{kind, name, value, tags, session_id, qualified_id, at}` — `kind` is `:counter | :gauge | :distribution | :summary | :last_value`. Bridges `:telemetry_metrics` updates into the same dispatch path so metrics share persistence with structured events.
- `%Mnemosyne.Event.Diagnostic{level, target, message, fields, at}` — unstructured escape hatch covering third-party library events that do not carry a Mnemosyne struct, and ad-hoc logging from Mnemosyne code that does not warrant a typed variant. Replaces scattered `Logger.info/1` calls.
- `%Mnemosyne.Event.Error{context, error, at}` — distinct from `Diagnostic{level: :error}`: `Error` events trigger the Risk 5 event-tail dump path.

The sealed set is **closed for v1**: adding a variant is a code change with a Decision Trail entry in this doc, not a plugin mechanism. Discipline prevents god-object drift (Risk 4): most new observability needs go through `Diagnostic` rather than growing the sealed set.

### 4.2 Emission via `:telemetry.execute/3`

Every typed event is emitted by calling `:telemetry.execute/3` with the struct passed in `metadata.event`:

```elixir
defmodule Mnemosyne.Observability do
  @spec emit(event :: struct()) :: :ok
  def emit(%_module{} = event) do
    :telemetry.execute(
      event_name_for(event),
      measurements_for(event),
      %{event: event}
    )
  end

  # Event name prefix is derived from the struct module path.
  # `%Mnemosyne.Event.PhaseLifecycle{}` -> [:mnemosyne, :phase_lifecycle]
  # `%Mnemosyne.Event.Ingestion.Applied{}` -> [:mnemosyne, :ingestion, :applied]
  defp event_name_for(%module{}) do
    module
    |> Module.split()
    |> Enum.drop(2)            # drop ["Mnemosyne", "Event"]
    |> Enum.map(&Macro.underscore/1)
    |> Enum.map(&String.to_atom/1)
    |> then(&[:mnemosyne | &1])
  end

  # Latency-bearing events project their duration fields into
  # `:telemetry` measurements so `:telemetry_metrics` summaries work.
  defp measurements_for(%Mnemosyne.Event.SpawnLatencyReport{} = e) do
    %{
      cold_spawn_ms: e.cold_spawn_ms,
      first_chunk_ms: e.first_chunk_ms,
      first_output_ms: e.first_output_ms
    }
  end
  defp measurements_for(_), do: %{}
end
```

Call sites throughout the daemon become a single `Mnemosyne.Observability.emit/1` call:

```elixir
Mnemosyne.Observability.emit(%Mnemosyne.Event.PhaseLifecycle{
  kind: :started,
  phase: :work,
  qualified_id: state.qualified_id,
  executor_kind: :llm_harness,
  at: DateTime.utc_now()
})
```

Third-party library events (e.g., from `erlexec`, `telemetry_poller`, or user-added libraries) flow through the standard `:telemetry` path with their own event names; they carry no `metadata.event` field. `Mnemosyne.Observability.Handler` (§5) detects the absence and routes them to `%Mnemosyne.Event.Diagnostic{}` automatically, capturing the event name as `target`, the measurements map, and the metadata map verbatim.

### 4.3 Why this hybrid (typed structs + `:telemetry` transport)?

Two project principles point in opposite directions:

- **"Integration over reinvention"** says use `:telemetry` as-is, opaque map metadata and all. But pure-`:telemetry` gives up exhaustive matching on event variants — every downstream consumer becomes a `case Map.get(metadata, "foo")` against stringly-typed keys.
- **"Every state transition is a typed message; hard errors by default"** says build a custom sealed event module. But pure-typed-bus gives up `:telemetry`'s handler framework, hot-attach / detach semantics, `:telemetry.span/3` measurement helpers, and the entire BEAM ecosystem around event names.

The hybrid honours both: M owns type discipline at the *event payload* boundary, `:telemetry` owns transport / event naming / handler lifecycle / third-party integration. The custom code is bounded to one handler module + four GenServer subscribers.

## 5. Subscriber stack

Attached at daemon boot via a single `:telemetry.attach_many/4` call on `Mnemosyne.Observability.Handler`. The handler module receives every event (filtered by the `[:mnemosyne | _]` prefix) and dispatches to four long-lived subscriber GenServers supervised by `Mnemosyne.Observability.Supervisor` under the daemon's root `Mnemosyne.Supervisor`.

```
Mnemosyne.Supervisor
 └─ Mnemosyne.Observability.Supervisor (Supervisor, :rest_for_one)
     ├─ Mnemosyne.Observability.Metrics          (Supervisor — telemetry_metrics + reporter)
     ├─ Mnemosyne.Observability.RingBuffer.Sup   (DynamicSupervisor — one GenServer per session)
     ├─ Mnemosyne.Observability.JsonlWriter.Sup  (DynamicSupervisor — one GenServer per session)
     ├─ Mnemosyne.Observability.TuiBridge        (GenServer owning a :pg group for fan-out)
     └─ Mnemosyne.Observability.Handler          (attaches to :telemetry after the above are up)
```

| Subscriber | Purpose | Storage | Bounded? |
|---|---|---|---|
| `Metrics` | hosts `Telemetry.Metrics.*` definitions; ships with `Telemetry.Metrics.ConsoleReporter` in v1, optional `:telemetry_metrics_prometheus` in v1.5 | in-memory `:ets` tables + session-end JSON snapshot | no (metric cardinality bounded by the catalogue in §6) |
| `RingBuffer` | per-session `:queue` of last N events (default N=1000); one GenServer per session under a DynamicSupervisor; process dies with the session; provides `dump_session/3` | in-memory | yes |
| `JsonlWriter` | per-session writer GenServer holding a `File` handle opened with `[:append, :raw, :binary, :delayed_write]`; encodes each event via `Jason.encode_to_iodata!/1` and writes an iodata line terminated by `\n` | file | the mailbox is bounded via `{:max_messages, 4096}` process flag; overflow causes a process exit which supervisor logs and restarts |
| `TuiBridge` | maintains a `:pg` group `:mnemosyne_tui`; every daemon-client attached over the Unix socket (sub-F §6) joins the group from the client-session process; the handler publishes events to the group via `:pg.get_members/2` + `send/2`; bounded per-client mailbox with drop-oldest + `events.dropped` counter | process-memory (per client) | yes |
| `Handler` | attaches last; parses `metadata.event` or synthesises `%Diagnostic{}`; dispatches to the three subscribers above in order | stateless (just a handler function) | n/a |

### 5.1 Startup order

`Mnemosyne.Observability.Supervisor` uses `:rest_for_one` so that if `Metrics` restarts, the rest of the stack restarts too (the ETS tables owned by the old `Metrics` process become stale). Startup order is deliberate: the four GenServers come up first, and `Handler.attach/0` runs last. This means no event is dispatched until all subscribers are ready — events emitted during partial boot are collected by `:telemetry`'s default dispatch (a no-op since no handler is attached yet).

### 5.2 Handler dispatch

```elixir
defmodule Mnemosyne.Observability.Handler do
  @handler_id :mnemosyne_observability

  def attach do
    events =
      [:mnemosyne]
      |> :telemetry.list_handlers()
      |> Enum.map(& &1.event_name)
      # Plus any known third-party prefixes we want to capture:
      |> Kernel.++([[:erlexec], [:telemetry_poller]])

    :telemetry.attach_many(@handler_id, events, &__MODULE__.handle_event/4, nil)
  end

  def handle_event(name, measurements, metadata, _config) do
    event = Map.get(metadata, :event) || synthesise_diagnostic(name, measurements, metadata)
    session_id = Map.get(metadata, :session_id) || extract_session_id(event)
    qualified_id = Map.get(metadata, :qualified_id) || extract_qualified_id(event)

    # Dispatch order: cheap in-memory first, expensive IO last.
    Mnemosyne.Observability.RingBuffer.record(session_id, event)
    Mnemosyne.Observability.Metrics.record(event, measurements)
    Mnemosyne.Observability.TuiBridge.publish(event, session_id, qualified_id)
    Mnemosyne.Observability.JsonlWriter.append(session_id, qualified_id, event)

    :ok
  rescue
    # Handler must never raise inside :telemetry dispatch —
    # a raise would cause :telemetry to detach the handler.
    error ->
      Logger.error("Observability handler error: #{inspect(error)}")
      :ok
  end
end
```

The `rescue` clause is load-bearing: `:telemetry` *detaches* a handler that raises inside `handle_event/4` to prevent cascading failures, which would silently blind Mnemosyne to all subsequent events. M's handler is defensively wrapped so a bug in one subscriber cannot take observability down. The raised error is logged through `Logger` directly (not through M — avoiding the obvious re-entrancy) and the handler stays attached.

### 5.3 Re-entrancy discipline

`:telemetry.execute/3` from inside `Handler.handle_event/4` would re-enter the dispatch, causing a recursion. This is the BEAM equivalent of the Rust Layer ordering bug (Risk 1). The discipline: **no code reachable from `Handler.handle_event/4` may call `Mnemosyne.Observability.emit/1` or `:telemetry.execute/3`.** Tests enforce this by attaching a witness handler that counts recursive dispatches; the re-entrancy integration test (§14) asserts the count stays zero across 1M event emits.

The subscribers themselves (`RingBuffer`, `JsonlWriter`, `TuiBridge`, `Metrics`) communicate with the handler via `GenServer.cast` (fire-and-forget, no handler in their reply path) which is re-entrancy-safe.

### 5.4 Session scoping

Every event carries an optional `session_id` (set when the event is emitted from inside an active harness session) and an optional `qualified_id` (set from the owning PlanActor's state). `RingBuffer` keys its per-session buffers by `session_id`; events without a session id go to a "process-scoped" ring shared across the daemon. `JsonlWriter` uses both keys to choose the output file path. `TuiBridge` forwards both through the `:pg` broadcast so client-side filters can narrow by plan or session.

## 6. Metrics catalogue (v1)

Metric definitions live in `Mnemosyne.Observability.Metrics` as `Telemetry.Metrics.*` declarations. `:telemetry_metrics` is reporter-independent: the same declarations drive the v1 `ConsoleReporter` and the v1.5 `:telemetry_metrics_prometheus` reporter without rewriting.

```elixir
defmodule Mnemosyne.Observability.Metrics do
  use Supervisor
  import Telemetry.Metrics

  def start_link(opts), do: Supervisor.start_link(__MODULE__, opts, name: __MODULE__)

  @impl true
  def init(_opts) do
    children = [
      {Telemetry.Metrics.ConsoleReporter, metrics: metrics()}
    ]
    Supervisor.init(children, strategy: :one_for_one)
  end

  def metrics do
    [
      # Phase lifecycle (sub-B)
      counter("mnemosyne.phase_lifecycle.started.count",
        event_name: [:mnemosyne, :phase_lifecycle],
        keep: &(&1.event.kind == :started)),
      counter("mnemosyne.phase_lifecycle.exited_clean.count",
        event_name: [:mnemosyne, :phase_lifecycle],
        keep: &(&1.event.kind == :exited_clean)),
      counter("mnemosyne.phase_lifecycle.executor_failed.count",
        event_name: [:mnemosyne, :phase_lifecycle],
        keep: &(&1.event.kind == :executor_failed)),
      counter("mnemosyne.phase_lifecycle.interrupted.count",
        event_name: [:mnemosyne, :phase_lifecycle],
        keep: &(&1.event.kind == :interrupted)),

      # Harness (sub-C)
      counter("mnemosyne.session_lifecycle.ready.count",
        event_name: [:mnemosyne, :session_lifecycle],
        keep: &(&1.event.transition == :ready)),
      counter("mnemosyne.session_exit_status.clean.count",
        event_name: [:mnemosyne, :session_exit_status],
        keep: &(&1.event.reason == :clean)),
      counter("mnemosyne.session_exit_status.error.count",
        event_name: [:mnemosyne, :session_exit_status],
        keep: &(&1.event.reason != :clean)),
      distribution("mnemosyne.spawn_latency.cold_spawn_ms",
        event_name: [:mnemosyne, :spawn_latency_report],
        measurement: :cold_spawn_ms,
        reporter_options: [buckets: [100, 250, 500, 1000, 2500, 5000, 10_000]]),
      distribution("mnemosyne.spawn_latency.first_chunk_ms",
        event_name: [:mnemosyne, :spawn_latency_report],
        measurement: :first_chunk_ms,
        reporter_options: [buckets: [100, 250, 500, 1000, 2500, 5000, 10_000]]),
      distribution("mnemosyne.spawn_latency.first_output_ms",
        event_name: [:mnemosyne, :spawn_latency_report],
        measurement: :first_output_ms,
        reporter_options: [buckets: [100, 250, 500, 1000, 2500, 5000, 10_000]]),
      last_value("mnemosyne.harness.live_sessions",
        event_name: [:mnemosyne, :session_lifecycle]),

      # Ingestion (sub-E)
      counter("mnemosyne.ingestion.applied.count",
        event_name: [:mnemosyne, :ingestion, :applied]),
      counter("mnemosyne.ingestion.deferred.count",
        event_name: [:mnemosyne, :ingestion, :deferred]),
      counter("mnemosyne.ingestion.rejected.count",
        event_name: [:mnemosyne, :ingestion, :rejected]),
      distribution("mnemosyne.ingestion.cycle_duration_ms",
        event_name: [:mnemosyne, :ingestion, :cycle_summary],
        measurement: :duration_ms,
        reporter_options: [buckets: [100, 500, 1000, 5000, 10_000, 30_000]]),

      # Routing / dispatch / rules (sub-F)
      counter("mnemosyne.message_routed.count",
        event_name: [:mnemosyne, :message_routed]),
      counter("mnemosyne.rule_fired.count",
        event_name: [:mnemosyne, :rule_fired]),
      counter("mnemosyne.level2_invoked.count",
        event_name: [:mnemosyne, :level2_invoked]),
      counter("mnemosyne.level2_rejected.count",
        event_name: [:mnemosyne, :level2_rejected]),
      counter("mnemosyne.dispatch_processed.count",
        event_name: [:mnemosyne, :dispatch_processed]),
      counter("mnemosyne.query_answered.count",
        event_name: [:mnemosyne, :query_answered]),
      counter("mnemosyne.rule_compile_error.count",
        event_name: [:mnemosyne, :rule_compile_error]),
      counter("mnemosyne.rule_suggestion.count",
        event_name: [:mnemosyne, :rule_suggestion]),

      # Sentinel detection (sub-B policy layer)
      counter("mnemosyne.sentinel.matched.count",
        event_name: [:mnemosyne, :diagnostic],
        keep: &(&1.event.target == "mnemosyne.sentinel")),

      # Drop counter (non-fatal overflow on bounded queues)
      counter("mnemosyne.events.dropped.count",
        event_name: [:mnemosyne, :diagnostic],
        keep: &(&1.event.target == "mnemosyne.events.dropped")),

      # Actor lifecycle (sub-F)
      counter("mnemosyne.actor_state_change.started.count",
        event_name: [:mnemosyne, :actor_state_change],
        keep: &(&1.event.kind == :started)),
      counter("mnemosyne.actor_state_change.restarted.count",
        event_name: [:mnemosyne, :actor_state_change],
        keep: &(&1.event.kind == :restarted)),
      counter("mnemosyne.actor_state_change.terminated.count",
        event_name: [:mnemosyne, :actor_state_change],
        keep: &(&1.event.kind == :terminated))
    ]
  end
end
```

**Catalogue discipline.** A compile-time `@external_resource` test in `test/observability/catalogue_test.exs` enumerates the metric names returned by `Mnemosyne.Observability.Metrics.metrics/0`, parses the `## 6. Metrics catalogue (v1)` section of this design doc, and asserts one-to-one correspondence. Adding a metric without updating §6 (or vice versa) fails CI. This locks the metric name contract into the spec review loop.

Unlike the Rust `metrics` crate's `const &'static str` approach, `:telemetry_metrics` uses string names by convention — so the compile-time typo protection is weaker. The catalogue test is the substitute protection.

## 7. Storage layout

```
<vault>/runtime/                                            # sub-A §A4 runtime subtree
├── events/
│   └── <qualified-id>/
│       ├── <session-id>.jsonl                              # one JSONL file per harness session
│       └── process.jsonl                                   # process-scoped events (no session)
├── metrics/
│   └── <qualified-id>/
│       └── <session-id>.json                               # snapshot at session end
├── transcripts/                                            # sub-C owns; M reads for analysis tooling
│   └── <session-id>.jsonl
└── interrupted/                                            # sub-B §3 owns the directory
    └── <qualified-id>/
        └── <phase>-<timestamp>/
            └── event-tail.json                             # Risk 5 dump (last N events)

<vault>/projects/<project-name>/mnemosyne/observability/    # git-tracked, Obsidian-facing (v1.5)
└── sessions/
    └── <session-id>.md                                     # Dataview frontmatter + summary
```

`runtime/events/` and `runtime/metrics/` are gitignored per sub-A §A5 — operational data, transient, and safe to wipe. The `observability/sessions/` markdown summaries under `<vault>/projects/...` are git-tracked because they're the historical record users browse via Dataview (v1.5 materialisation).

### 7.1 JSONL format

One event per line, `Jason.encode_to_iodata!/1` + `"\n"` terminator. Every line is independently parseable; truncation at any point leaves earlier lines intact. Line schema:

```json
{
  "ts": "2026-04-15T14:23:45.123Z",
  "qualified_id": "Mnemosyne/project-root/sub-F-hierarchy",
  "session_id": "01HXYZ...",
  "event_name": ["mnemosyne", "phase_lifecycle"],
  "event": { "__struct__": "Mnemosyne.Event.PhaseLifecycle", ... }
}
```

The `__struct__` field is automatic from `@derive Jason.Encoder` on a struct; downstream Elixir consumers round-trip via `Jason.decode!/1` + a small reconstruction helper that dispatches on the struct name. A forward-compat rule: **consumers must ignore unknown top-level keys and unknown variant fields**, to allow additive evolution of the sealed set without breaking `mnemosyne diagnose` against older logs.

### 7.2 Metrics snapshot format (`metrics/<qualified-id>/<session-id>.json`)

`Mnemosyne.Observability.Metrics` collects values into ETS tables behind `:telemetry_metrics`. At session end, `Mnemosyne.Observability.Metrics.snapshot/1` folds the tables into a map and writes it via `Jason.encode_to_iodata!/1`:

```json
{
  "session_id": "01HXYZ...",
  "qualified_id": "Mnemosyne/project-root",
  "started_at": "2026-04-15T14:20:00Z",
  "ended_at": "2026-04-15T14:35:12Z",
  "counters": {
    "mnemosyne.phase_lifecycle.started.count": 1,
    "mnemosyne.phase_lifecycle.exited_clean.count": 1,
    "mnemosyne.session_lifecycle.ready.count": 1,
    "mnemosyne.ingestion.applied.count": 7
  },
  "distributions": {
    "mnemosyne.spawn_latency.cold_spawn_ms": {
      "count": 1, "sum": 2340, "min": 2340, "max": 2340,
      "p50": 2340, "p95": 2340, "p99": 2340
    }
  },
  "gauges": {
    "mnemosyne.harness.live_sessions": 1
  }
}
```

Distribution percentiles come from the reporter's bucket histograms (see §6 `reporter_options`). `ConsoleReporter` does not expose percentile computation directly; for the v1 snapshot we read the raw values stored by a thin custom reporter, `Mnemosyne.Observability.Metrics.SnapshotReporter`, which subscribes to the same events `ConsoleReporter` does and maintains bucket histograms in ETS. `SnapshotReporter` is ≈150 lines. When v1.5 ships `:telemetry_metrics_prometheus`, `SnapshotReporter` is retained because Prometheus export is push-model; the snapshot file is still the source of truth for session-ended analyses.

### 7.3 Obsidian session summary format (v1.5)

Markdown with kebab-case YAML frontmatter (per the project's Obsidian-native format discipline):

```markdown
---
session-id: 01HXYZ...
qualified-id: Mnemosyne/project-root
phase: work
started-at: 2026-04-15T14:20:00Z
ended-at: 2026-04-15T14:35:12Z
exit-status: clean
harness-cold-spawn-latency-ms: 2340
harness-first-chunk-latency-ms: 2780
ingestion-applied-count: 7
ingestion-deferred-count: 0
sentinel-matched: true
tags: [mnemosyne/observability/session, mnemosyne/plan/mnemosyne-orchestrator]
---
# Session 01HXYZ... — Work phase, Mnemosyne/project-root

Brief textual summary auto-generated from event tail.

## Events of note

[[2026-04-15-1423-phase-started]]
[[2026-04-15-1424-harness-spawned]]

## Errors

None.
```

Wikilinks point to per-event note stubs; the v1.5 `Mnemosyne.Observability.ObsidianMaterialiser` produces these stubs as part of materialisation. v1 ships without these summaries — the JSONL and metric snapshots cover the data needs for the dogfood acceptance test, and Obsidian materialisation can land additively.

## 8. Risk 5 resolution — diagnostic-poor failure modes

Sub-project C's accepted Risk 5 says: *"v1 ships with diagnostic-poor failure modes — when a session fails in a way the actor doesn't anticipate, the user sees the message but no rich context."*

M's resolution:

1. **`Mnemosyne.Observability.RingBuffer.dump_session(session_id, n)`** returns the last `n` events for the given session in chronological order. Implemented as a `GenServer.call` against the per-session ring-buffer GenServer; if the session process has already exited, the call is routed through a small registry fallback to the most recent ring buffer for that session (the process stays alive for a brief grace window after the session ends, sized to the §14 Risk 5 integration test).
2. **`Mnemosyne.Observability.dump_event_tail(session_id, qualified_id, phase, n)`** writes the ring-buffer result to `<vault>/runtime/interrupted/<qualified-id>/<phase>-<timestamp>/event-tail.json` using a panic-safe `try/rescue` wrapper. Any failure inside the dump path is logged through `Logger` directly and never masks the original error.
3. **Every error path** in B's `PhaseRunner`, C's session GenServer, and E's pipeline calls `dump_event_tail/4` on its way out, alongside B's existing forensics-dir writer. Sub-B's `%PhaseLifecycle{kind: :executor_failed}` and C's `%HarnessError{}` emissions precede the dump so the ring buffer contains them before the dump reads.
4. **The TUI's error display** surfaces a "view event tail" action that opens the file (client-side; TUI is out of M's scope but consumes the file).
5. **The default ring size (1000 events)** captures roughly the last 5-15 minutes of activity for typical sessions — sized to fit the C-1 dogfood acceptance test envelope without runaway memory.

This makes Risk 5's requirement *"give me the last N events from session X with full context"* a one-`GenServer.call` operation. The error-path call is wrapped in a `try/rescue` helper so a failure inside `dump_session/2` cannot mask the original error.

## 9. Migration of C's `%SpawnLatencyReport{}` (staged, never breaks C-1)

Sub-C's v1 ships a tactical three-way parallel emission for `%SpawnLatencyReport{}` (C's design doc §7.2):

1. Info message delivered to the consumer process (`{:harness_event, _, %SpawnLatencyReport{}}`)
2. `:telemetry.execute([:mnemosyne, :harness, :claude_code, :spawn_latency], ...)`
3. JSON file at `<staging>/spawn-latency.json`

C explicitly disclaims framework scope for this instrumentation. M's job is to absorb it without breaking the C-1 dogfood acceptance gate.

| Phase | C's info message | C's staging file | C's `:telemetry` call | M's typed `%SpawnLatencyReport{}` handler |
|---|---|---|---|---|
| **C v1** (today, before M lands) | yes | yes | yes (no handler attached) | no |
| **M v1 lands** | yes | yes | yes | yes (parallel) |
| **M v1.1** (after parallel-emit window proves M correct) | yes | no | yes | yes |
| **M v2** (after full migration complete) | no | no | yes | yes |
| **Sub-G migration** (cleanup) | (file-deletion tasks scheduled by G) | — | — | — |

M's handler attaches to `[:mnemosyne, :spawn_latency_report]` and routes the struct through the normal dispatch path (RingBuffer + JsonlWriter + Metrics + TuiBridge). The `:telemetry` event name from C's §7.2 is bridged: `Handler.handle_event/4` also subscribes to `[:mnemosyne, :harness, :claude_code, :spawn_latency]` and rewrites incoming events into `%SpawnLatencyReport{}` structs before dispatch, so both naming conventions land on the same dispatch path.

The parallel-emit window is the safety net: M's dispatch is verified against C's staging file ground truth before C's auxiliary emission paths are removed. If M's recorded values diverge from the JSON file, M is wrong and gets fixed before the deletion step.

The verification check is mechanical: a `:live`-tagged Layer 3 integration test reads both the JSON file and the metric snapshot from the same session and asserts the three latency values match within ±10ms (clock skew allowance). The test runs on every CI build during the parallel-emit window.

## 10. Cross-plan adoption (M's deliverable, not triage scope)

When M's brainstorm lands its sibling implementation plan, it also lands adoption tasks into each existing sibling backlog. **This is M's own deliverable** — not a triage-phase task that may or may not get scheduled.

| Sibling | Adoption task |
|---|---|
| **sub-B-phase-cycle** | B's §4.4 already commits to emitting the seven `%PhaseLifecycle{}` variants via `Mnemosyne.Observability.emit/1`. B also forwards `%HarnessOutput{}` and `%SessionLifecycle{}` from C. M's adoption task: verify B's implementation wires `emit/1` at the §4.4 call sites and runs a Layer 3 test asserting the seven events arrive at the `RingBuffer` subscriber. |
| **sub-C-adapters** | C's §7.2 already commits to the three-way parallel emission. M's adoption task: land `Handler.handle_event/4`'s bridge from `[:mnemosyne, :harness, :claude_code, :spawn_latency]` to `%SpawnLatencyReport{}`, run the verification window, schedule the C-v1.1 cleanup task in C's sibling backlog. C also emits `%HarnessOutput{}`, `%SessionLifecycle{}`, `%SessionExitStatus{}`, `%HarnessError{}` per §11.4 — M verifies each reaches the `RingBuffer` subscriber. |
| **sub-D-concurrency** | D's collapsed scope (daemon singleton lock + advisory file locks + vault git concurrency) is primarily structural, not observational. Adoption task: emit `%Diagnostic{target: "mnemosyne.lock"}` for singleton-lock acquire / release / contend events. |
| **sub-E-ingestion** | Wrap each existing `IngestionEvent` emit with a parallel `Mnemosyne.Observability.emit/1` of the corresponding `%Ingestion.*{}` struct during the transition window. After verification, collapse E's standalone channel into M's bus and delete E's channel plumbing. (E's F-amendment is pending; this adoption task lands alongside that amendment.) |
| **sub-F-hierarchy** | F's Task 24 already commits to emitting the nine routing/actor/rule events via `:telemetry.execute/3`. M's adoption task: verify F's handler registrations for the nine structs, add `Telemetry.Metrics.*` entries for the five routing counters (`message_routed`, `rule_fired`, `level2_invoked`, `level2_rejected`, `rule_suggestion`). Sub-F Task 24 is dependent on M's v1 implementation, which is dependent on this amendment — the two are tracked in parallel and land together during the F implementation runway. |
| **sub-A-global-store** | A's 11-step daemon boot sequence (§A.Reference algorithm) step 11 emits `%Vault.BootReady{}`. A's `Mnemosyne.Vault.MarkerError` routes through M's `:telemetry` boundary before a hard-error exit. M's adoption task: add the two structs to the sealed set (done in §4.1 of this doc) and verify A's boot sequence calls `Mnemosyne.Observability.emit/1` at the right points. |
| **sub-H-skills** | Emit `%Diagnostic{target: "mnemosyne.tui_action", fields: %{action: ...}}` for each attached-client TUI action invocation. |
| **sub-I-obsidian** | Document the observability data surfaces (events, metrics, session summaries, plus `<vault>/plan-catalog.md` as a routing + catalogue surface) as part of the Obsidian coverage doc. Provide example Dataview queries against the v1.5 Obsidian session summary format. |
| **sub-G-migration** | Delete `<staging>/spawn-latency.json` from the staging schema after the M parallel-emit window closes. Migrate any pre-M Mnemosyne v0.1.0 logs (if any exist in user vaults) — none are expected since v0.1.0 has no structured logging. |

Each adoption task is added to the corresponding sibling backlog as a `not_started` task with a dependency on M's v1 implementation. M's brainstorm's session log records that the adoption tasks have been added; the orchestrator's triage phase verifies they're present.

## 11. Display surfaces (v1)

### 11.1 Rust TUI client (separate binary)

Per sub-F §6/§7, the TUI is a standalone Rust `ratatui` binary connected to the daemon over a Unix socket at `<vault>/runtime/daemon.sock` speaking NDJSON. The client subscribes to an observability event stream by sending a `{"op":"subscribe","channel":"observability","filters":{...}}` request. On the daemon side, the request is handled by a per-client session process that joins the `:mnemosyne_tui` `:pg` group; every event broadcast by `TuiBridge.publish/3` reaches every client-session in the group, which serialises the event to JSON and writes it on the Unix socket.

Rendering responsibilities stay in the TUI (M doesn't care about `ratatui` widgets):

- **Status bar**: live gauges — `mnemosyne.harness.live_sessions`, elapsed time of the current phase (computed client-side from the most recent `%PhaseLifecycle{kind: :started}` timestamp), rolling count of recent `%Error{}` events.
- **Event tail panel**: last 50 events received on the subscription, formatted compactly.
- **Error display**: when an `%Error{}` arrives, the TUI offers a "view event tail" action that sends a `{"op":"dump_event_tail","session_id":...}` request to the daemon; the daemon calls `Mnemosyne.Observability.dump_event_tail/4` and returns the JSON payload inline.
- **Metric sparklines (v1.5)**: small ASCII sparklines for the most recent N values of key distributions. v1 omits these for simplicity.

M's v1 surface to the TUI is three daemon protocol operations:

| Operation | Handler | Returns |
|---|---|---|
| `subscribe observability filters:{qualified_id?, session_id?}` | `TuiBridge` adds the client-session pid to the `:pg` group with filter metadata | streamed events until unsubscribe |
| `dump_event_tail session_id, n` | `RingBuffer.dump_session/3` | `Vec<Event>` inline |
| `metrics_snapshot qualified_id` | `Metrics.snapshot/1` | snapshot JSON inline |

The protocol wire format is sub-F's scope; M provides the GenServer-call surface. This is the BEAM version of the "M provides the channel, TUI consumes it" contract.

### 11.2 Obsidian (Dataview)

v1 ships JSONL events and metric snapshots. v1.5 ships the `Mnemosyne.Observability.ObsidianMaterialiser` that produces per-session markdown summaries with Dataview-friendly frontmatter (§7.3).

Example query for finding slow sessions:

```dataview
TABLE qualified-id, phase, harness-cold-spawn-latency-ms, exit-status
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

### 11.3 `ConsoleReporter` (development)

`Telemetry.Metrics.ConsoleReporter` prints metric updates to stdout via `Logger`. Off by default in production (`config :logger, level: :info` in `config/prod.exs` filters below `:info`); enabled during development via `config :logger, level: :debug` in `config/dev.exs`. Useful for immediate feedback during dogfood cycles. The TUI is the user's primary live surface; `ConsoleReporter` is a developer aid.

## 12. Analysis tooling

### 12.1 `mix mnemosyne.metrics` task (v1)

```sh
mix mnemosyne.metrics --qualified-id Mnemosyne/project-root --since 2026-04-01
```

Escript-style entry point at `lib/mix/tasks/mnemosyne.metrics.ex`. Reads JSONL events from `<vault>/runtime/events/<qualified-id>/` and metric snapshots from `<vault>/runtime/metrics/<qualified-id>/` for the given range. Computes per-metric percentiles from the stored bucket histograms. Prints a human-readable report to stdout.

≈300 lines, no new deps (reuses `Jason`). Output formats:

- Default: pretty-printed table to stdout via `IO.ANSI`
- `--json`: machine-readable JSON
- `--csv` (v1.5+): CSV for piping to external tools

The task lives alongside `mix mnemosyne.dev.record_fixture` (C's dev task) and the future `mix mnemosyne.adopt_project` (A's adopt-project command as a mix task during dev).

### 12.2 `mix mnemosyne.diagnose` task (v1)

```sh
mix mnemosyne.diagnose --session 01HXYZ... [--last 50]
```

Reads `<vault>/runtime/events/<qualified-id>/<session-id>.jsonl` and prints the last N events in chronological order. Mirrors the in-memory `RingBuffer.dump_session/2` API but works against persisted JSONL after the session has ended. Useful when the live `event-tail.json` dump is missing or insufficient.

### 12.3 Obsidian Dataview queries (v1.5)

Once `Mnemosyne.Observability.ObsidianMaterialiser` ships, ad-hoc analysis happens in Obsidian directly. The mix tasks stay for machine-readable cases and for users not running Obsidian.

## 13. Error handling

Per the project's "hard errors by default" principle:

- **Supervisor child start failure** (e.g., `Metrics` cannot open ETS, `JsonlWriter` cannot write to `<vault>/runtime/events/`) → fails at process startup; the daemon's root supervisor escalates; the 11-step boot sequence (sub-A Reference algorithm) exits non-zero with a clear diagnostic. Mnemosyne refuses to run.
- **`Handler.handle_event/4` rescue** (any raise inside dispatch) → log via `Logger.error/1`, stay attached, do not cascade. This is load-bearing: a raise without the rescue would cause `:telemetry` to detach the handler permanently, silently blinding Mnemosyne to all subsequent events. The rescue is the exception to "hard errors by default" and is justified here because (a) losing observability must be louder than the original error, not quieter; (b) the error is logged, not swallowed; (c) tests cover every subscriber path to keep the rescue rare.
- **`JsonlWriter` mailbox overflow** → the GenServer's `{:max_messages, 4096}` process flag triggers an abnormal exit; the `DynamicSupervisor` restarts the writer with a fresh file handle; the exit is counted as `mnemosyne.events.dropped.count` with `target: "mnemosyne.jsonl_writer.restart"`. Events in the dead process's mailbox are lost, but the new writer picks up cleanly. Bounded queue is the explicit non-fatal exception.
- **`TuiBridge` client mailbox full** → `:pg`-side send fails silently per BEAM semantics; M wraps each `send/2` in a try/catch that increments `mnemosyne.events.dropped.count` with `target: "mnemosyne.tui_bridge.drop"`. Justification: blocking the entire daemon on a slow TUI client would be worse than dropping events; the counter makes the loss observable.
- **`RingBuffer` ring at capacity** → drops oldest. By design — the queue is bounded and circular.
- **Risk 5 dump path failure** (e.g., disk write fails while writing `event-tail.json`) → wrapped in `try/rescue`, logs the secondary failure via `Logger`, never masks the original error.

Bounded-queue overflow and the handler rescue are the only tolerated failure modes. Every other path fails hard.

## 14. Testing

### Unit tests (ExUnit)

- **`Handler` dispatch**: emit a `%PhaseLifecycle{}` via `Observability.emit/1`, start a witness subscriber, assert the handler parses and dispatches the typed payload identically. Cover every struct in the sealed set via table-driven tests.
- **`RingBuffer`**: ring eviction at capacity; `dump_session/2` chronological order; per-session isolation; process-scoped fallback; reconstruction across session-end race conditions.
- **`JsonlWriter`**: round-trip events through a tmpdir; verify line-per-event format; verify truncation safety (every line independently parseable); `Jason.decode!/1` round-trip.
- **`Metrics`**: counter increment, `last_value` update, distribution record; snapshot at session end via `SnapshotReporter`; bucket layout correctness against synthetic latency values.
- **`TuiBridge`**: `:pg` group join / leave semantics; bounded-send behaviour; drop-oldest accounting.
- **Catalogue test**: enumerate `Mnemosyne.Observability.Metrics.metrics/0`, parse the `## 6.` section of this doc, assert one-to-one correspondence.
- **Third-party event routing**: emit a `:telemetry.execute([:erlexec, :exec, :stop], ...)` event with no `metadata.event`, assert `Handler` synthesises a `%Diagnostic{}` and forwards it.

### Layer 3 integration tests (`@moduletag :live`)

- **Fixture-replay end-to-end**: spawn sub-C's `FixtureReplay.Session`, drive a full `work → reflect → triage` cycle through a minimal `PhaseRunner` stub, verify the JSONL stream contains the expected event sequence and the metric snapshot has the expected counter values.
- **Risk 5 dump end-to-end**: induce a synthetic error in a `PhaseRunner` fixture, assert `<vault>/runtime/interrupted/<qualified-id>/<phase>-<ts>/event-tail.json` exists, assert it parses as a list of event structs via the reconstruction helper, assert it contains the last events leading up to the error. Inject a synthetic disk-full error during the dump itself and confirm the original error is still surfaced (not masked).
- **C parallel-emit verification** (only during the parallel-emit window): run a real C session against the `claude` binary on haiku, read both `<staging>/spawn-latency.json` and `<vault>/runtime/metrics/.../<session-id>.json`, assert the three latency values match within ±10ms.
- **TuiBridge wiring**: verify a mock daemon-client that joins the `:pg` group receives the expected event stream during a phase cycle. Verify the per-client drop counter increments under backpressure.
- **Re-entrancy (Risk 1 mitigation)**: emit 1M events through the full dispatch path across four concurrent producer processes, attach a witness handler counting recursive dispatches, assert the count stays zero, assert no `Handler` detach events, assert the `JsonlWriter` did not restart.

### Property tests (StreamData)

- JSONL round-trip: any event struct round-trips through `Jason.encode!` → write → read → `Jason.decode!` → reconstruct without loss. Run across all variants via `StreamData` generators.
- Catalogue integrity: any generated metric name passes through `Telemetry.Metrics.*` definitions without raising.

## 15. v1 / v1.5 / v2 cut

### v1 (this brainstorm's implementation plan)

- Hex dep stack (§3) integrated: `:telemetry`, `:telemetry_metrics`, `Jason`. `Telemetry.Metrics.ConsoleReporter` shipped. `SnapshotReporter` shipped.
- Sealed `Mnemosyne.Event.*` struct set (§4.1) defined in `lib/mnemosyne/event/*.ex`.
- `Mnemosyne.Observability.emit/1` + `Handler` (§4.2, §5) shipped and wired into the daemon boot sequence as a child of `Mnemosyne.Observability.Supervisor` under `Mnemosyne.Supervisor`.
- Subscribers (§5): `RingBuffer.Sup`, `JsonlWriter.Sup`, `TuiBridge`, `Metrics` all supervised and wired.
- Metric catalogue (§6) defined. Call sites in sub-B (phase lifecycle), sub-C (harness lifecycle + parallel-emit `SpawnLatencyReport`), sub-E (ingestion counters + cycle-duration), sub-F (routing + dispatch + actor lifecycle), sub-A (boot-ready + marker-error) are all wired via M's adoption tasks (§10). Sub-D / sub-H / sub-I call sites are added when those sub-projects' implementation plans land.
- Storage layout (§7.1, §7.2) implemented; vault runtime subtree integrated with A's layout at `<vault>/runtime/events/` and `<vault>/runtime/metrics/`.
- Risk 5 dump (§8) wired into B / C / E error paths via their adoption tasks.
- C's `%SpawnLatencyReport{}` parallel-emit (§9 phase `M v1 lands`) live.
- `mix mnemosyne.metrics` and `mix mnemosyne.diagnose` tasks shipped (§12.1, §12.2).
- Cross-plan adoption tasks landed in sibling backlogs (§10).
- Test suite (§14) at green, including re-entrancy integration test.

### v1.5

- `Mnemosyne.Observability.ObsidianMaterialiser` (§5, §7.3, §11.2).
- Deletion of C's tactical `<staging>/spawn-latency.json` writer (§9 phase `M v1.1`).
- Optional `:telemetry_metrics_prometheus` reporter wired into `Mnemosyne.Observability.Metrics`; daemon config `[observability] prometheus_endpoint = "..."` key exposed.
- Full Dataview query catalogue absorbed into sub-I's brainstorm output.
- TUI metric sparklines (§11.1).
- `mix mnemosyne.metrics --csv` output format.

### v2

- External metrics export via OpenTelemetry (`:opentelemetry` + OTLP exporter) as an alternative reporter. Bridges to sub-P (team mode) distributed tracing across multiple Mnemosyne instances.
- Anomaly detection on long-term metric trends.
- `observer_cli` documented as a recommended optional layer.

## 16. Open questions

These do not block the implementation plan but should be resolved during v1 build.

| # | Question | Resolution method | Resolution timing |
|---|---|---|---|
| 1 | Handler dispatch order — cheap-first or strict FIFO? §5.2 runs RingBuffer → Metrics → TuiBridge → JsonlWriter (cheap first). Does measurement confirm the ordering matters at C-1 envelope scale? | Instrument dispatch with `:telemetry.span/3` during Layer 3 test; compare cheap-first vs reversed on a 10k-event stream | Day 2 of v1 implementation |
| 2 | Default `RingBuffer` capacity (1000 events) — too many or too few for the C-1 dogfood envelope? | Measure event volume during the first dogfood run; tune accordingly | After first dogfood cycle |
| 3 | Whether `ObsidianMaterialiser` should land in v1 instead of v1.5 if the dogfood cycle generates user demand | Defer until v1 ships and dogfood feedback arrives | Post-v1 |
| 4 | Whether to ship `:telemetry_metrics_prometheus` in v1 alongside `ConsoleReporter` rather than deferring to v1.5 | Try it during v1 development and decide based on reporter attachment ergonomics | During v1 build |
| 5 | Distribution bucket layout — default from `:telemetry_metrics` vs custom for latency-friendly resolution | Inspect default percentile accuracy on synthetic latency data | Day 2 of v1 implementation |
| 6 | `SnapshotReporter` custom implementation vs pulling in a heavier reporter library | Build `SnapshotReporter` first (≈150 lines); re-evaluate only if bucket maintenance becomes painful | During v1 build |

## 17. Risks

### Risk 1 — `:telemetry` handler re-entrancy

Emitting `:telemetry.execute/3` (or `Observability.emit/1`) from inside `Handler.handle_event/4` causes recursive dispatch. This is the BEAM equivalent of the Rust `tracing-subscriber` Layer re-entrancy bug. The standard mitigation: subscribers must only `GenServer.cast` (no re-entry through the handler path); tests enforce via the witness counter (§14 re-entrancy integration test).

### Risk 2 — Bounded-queue overflows hide real problems

The `mnemosyne.events.dropped.count` counter is the observability surface for queue overflow, but if nobody looks at it, drops go unnoticed. Mitigation: the counter surfaces as a non-zero red badge in the TUI status bar; `mix mnemosyne.diagnose` prints a warning if any drops occurred in the session snapshot.

### Risk 3 — Migration window for C's `%SpawnLatencyReport{}` produces inconsistent data

During the parallel-emit window, both C's tactical writer and M's `:telemetry` path run. If they disagree (clock skew, timing race, off-by-one in the latency computation), the verification check (§14) fires and the implementation has to chase the discrepancy. Mitigation: ±10ms tolerance; the check runs on every CI build.

### Risk 4 — Sealed event struct set becomes a god module

Every new event variant adds a struct to `lib/mnemosyne/event/*`. As sub-projects land their adoption tasks, the variant list could balloon. Mitigation: the sealed set is closed for v1 with a Decision Trail entry required to add a new variant; most new needs go through `%Diagnostic{}`; the bar for a new struct is "multiple downstream consumers pattern-match on its fields."

### Risk 5 — `:telemetry_metrics` reporter selection costs

`:telemetry_metrics` definitions are cheap; reporter attachment is also cheap. But the `ConsoleReporter` writes through `Logger` on every metric update, which can be noisy in a dogfood cycle. Mitigation: configure `Logger` to filter `Telemetry.Metrics.ConsoleReporter` below `:warn` in dogfood environments; rely on `SnapshotReporter` + `mix mnemosyne.metrics` for introspection instead of tailing the console.

### Risk 6 — `Handler.handle_event/4` rescue swallows real bugs

The defensive `try/rescue` in §5.2 hides bugs from test runs if they happen inside a subscriber. Mitigation: unit tests for every subscriber cover every variant in the sealed set; the Layer 3 re-entrancy test asserts zero rescued errors; CI fails loud if any `Logger.error/1` call from the rescue path fires during tests.

## 18. Cross-sub-project requirements

### 18.1 Sub-B — concrete, §4.4 already committed

B's §4.4 commits to emitting seven `%PhaseLifecycle{}` variants via `Mnemosyne.Observability.emit/1` at the 13-step `run_phase/4` flow (steps 5, 11, 12, §3.4 step 5, §3.4 step 6, error branches, §3.2 Scenario A). B also forwards `%HarnessOutput{}` and `%SessionLifecycle{}` from C. M's v1 does not impose any new work on B beyond what §4.4 already specifies; the adoption task is a verification step.

### 18.2 Sub-C — concrete, §11.4 already committed

C's §11.4 commits to emitting `%HarnessOutput{}`, `%SessionLifecycle{}`, `%SpawnLatencyReport{}`, `%SessionExitStatus{}`, `%HarnessError{}` at every boundary via `:telemetry.execute/3`. C's §7.2 commits to the three-way parallel-emit discipline. M's v1 lands the consumer side and the verification test; after the window closes, a scheduled task in C's backlog removes the auxiliary staging-file writer (§9 phase `M v1.1`).

### 18.3 Sub-E — pending F-amendment

E's pipeline currently emits `IngestionEvent` via its own channel. E's F-amendment (pending as orchestrator Priority 1) will re-cast Stage 5 as dispatch-to-experts via F's router and the six ingestion event variants as `%Mnemosyne.Event.Ingestion.*{}` structs. M's adoption task rides on that amendment: wrap each existing emit with a parallel `Observability.emit/1` during the transition, then collapse E's channel into M's bus after verification.

### 18.4 Sub-F — concrete, Task 24 already scheduled

F's Task 24 commits to emitting nine event structs for actor state / message routing / rule firings / dispatch-processed / query-answered / Level 2 invocations via `:telemetry.execute/3`. M owns the handler and transport; F owns the event boundary. Task 24 depends on this amendment absorbing; this amendment depends on F's Task 24 event types being stable — the two land together during the F implementation runway. M's adoption task: land `Telemetry.Metrics.*` definitions for the five routing counters in §6, verify F's emission points match the struct fields documented in §4.1.

### 18.5 Sub-A — concrete, absorbed in A's Session-14 rewrite

A's 11-step daemon boot sequence (A design doc §A.Reference algorithm) step 11 emits `%Vault.BootReady{}`. A's `Mnemosyne.Vault.MarkerError` routes through M's `:telemetry` boundary before a hard-error exit. Both structs are in M's §4.1 sealed set. No further amendment needed on A's side; M's v1 just consumes the two events.

### 18.6 Sub-D — adoption task, scope collapsed by F

D's original per-plan advisory locks were eliminated by F's daemon commitment. D's collapsed scope (daemon singleton lock, advisory file locks for external-tool coordination, vault git concurrency) produces few observability events. Adoption task: emit `%Diagnostic{target: "mnemosyne.lock"}` for singleton-lock events. Additive; not part of D's core design.

### 18.7 Sub-H, Sub-I — adoption tasks, brainstormed later

These sub-projects haven't been brainstormed yet. M's adoption requirements (§10) attach to their backlogs as soon as their implementation plans exist, via the cross-plan landing protocol described in §10. For I specifically: the Obsidian coverage doc absorbs M's v1.5 session-summary format as a concrete documented surface.

### 18.8 Sub-G — staging-schema cleanup

G's migration plan absorbs the deletion of `<staging>/spawn-latency.json` from the staging schema (§9 phase `M v1.1`) and any v0.1.0 log file cleanup. Both are additive to G's existing scope.

## 19. Decision trail

The brainstorm session that produced this spec made five major decisions. Q1–Q5 are from the original Session 7 brainstorm against a Rust runtime; they remain substantively valid post-pivot and are preserved verbatim here with correction notes. Q6 records the BEAM pivot amendment (Session 15) and enumerates the runtime swap.

### Q1 — Weight the three masters (diagnostic / live / analysis)

**Options considered:**

- **A — Diagnostic-first only.** Simplest; smallest complexity budget; but means the TUI and long-term analysis surfaces get retrofitted later.
- **B — Live-display-first only.** TUI-optimised; means diagnostics and long-term analysis get retrofitted.
- **C — Long-term-analysis-first only.** Research-friendly; but the dogfood cycle and error diagnostics get retrofitted.
- **D — All three equally weighted.** Largest complexity budget up front; avoids retrofit work; honours the project's "hard errors by default" and "diagnostic-poor failure modes are Risk 5" constraints.

**Chosen: Option D.** The retrofit cost was judged greater than the up-front complexity cost. Three masters are paid for with one hybrid architecture (§4.3) + four GenServer subscribers (§5) + one handler dispatcher.

**Session-15 correction note.** Decision survives the BEAM pivot unchanged. The three masters are runtime-agnostic; the hybrid architecture re-casts onto `:telemetry` + typed structs without losing the decision's rationale.

### Q2 — Event transport: pure-`tracing` / pure-typed-bus / hybrid

**Options considered:**

- **A — Pure `tracing`** with stringly-typed field names; every consumer does `match` against field names.
- **B — Pure typed-bus** with a custom `MnemosyneEvent` enum and no `tracing` underneath.
- **C — Hybrid** — `MnemosyneEvent` enum at the Mnemosyne boundary, `tracing` everywhere below it.

**Chosen: Option C.** Pure `tracing` gives up exhaustive matching; pure typed-bus gives up span machinery, third-party integration, and the entire subscriber ecosystem. Hybrid honours both.

**Session-15 correction note.** Decision survives the pivot; the hybrid pattern re-casts onto `:telemetry` + typed Elixir struct set under `Mnemosyne.Event.*`. "`tracing`" in Q2 is now read as "BEAM's `:telemetry`", and "`MnemosyneEvent` enum" is now read as "sealed `Mnemosyne.Event.*` struct set." The rationale translates literally. See Q6 for the full runtime mapping.

### Q3 — Use existing tooling and libraries wherever possible

**Resolved via user steer mid-brainstorm.** The user's explicit guidance was *"use existing tooling and libraries wherever possible. This is not an interesting task."* — which collapsed several remaining clarifying questions in favour of the standard-tool answer at every fork. Five top-100 Rust crates; one ~200-line custom Layer; no bespoke event bus.

**Session-15 correction note.** The steer translates directly to BEAM: three top-100 Hex packages (`:telemetry`, `:telemetry_metrics`, `Jason`), two optional ones for v1.5 (`:telemetry_metrics_prometheus`, `logger_json`), one custom handler module + four small GenServers. The discipline is unchanged; the libraries are the BEAM equivalents of the Rust stack. See Q6.

### Q4 — Always-on instrumentation (project principle)

**Resolved on project-wide principle.** Every event emit is unconditional. The `MNEMOSYNE_LOG` env var only controls formatter visibility, not whether events are emitted.

**Session-15 correction note.** On BEAM, the knob becomes `config :logger, level: :info` (or `:debug`) in `config/dev.exs` / `config/prod.exs`. The principle is unchanged: events are always emitted; the log level only controls formatter verbosity. `ConsoleReporter` for `:telemetry_metrics` is analogously always subscribed; its output is filtered by `Logger` level.

### Q5 — Cross-plan adoption is M's deliverable, not triage scope

**Resolved via the cross-plan landing protocol.** M's brainstorm lands adoption tasks in sibling backlogs before exiting. The discipline is inherited by every cross-cutting brainstorm (sub-F's nine amendment tasks are the canonical example).

**Session-15 correction note.** Decision survives. §10 of this rewritten doc lists nine concrete adoption tasks across nine siblings (A, B, C, D, E, F, G, H, I). Two of them (B's §4.4 and F's Task 24) are already committed in those siblings' own design docs because this amendment is being written *after* those siblings were rewritten for BEAM. Observability is now a tight multi-party contract.

### Q6 — BEAM pivot (Session 15, 2026-04-15 via sub-F Session 9 commitment)

Sub-F committed Mnemosyne to a persistent BEAM daemon (Session 9, 2026-04-14). M was originally designed against Rust + `tracing` + the `metrics` crate; Session 15 re-casts the implementation on Elixir/OTP + `:telemetry` + `:telemetry_metrics`. The *design intent* survived unchanged: three masters equally weighted, hybrid typed-boundary + ecosystem transport, use existing libraries, always-on instrumentation, M owns adoption. The *runtime substrate* moved from a custom `tracing-subscriber::Layer` stack to `:telemetry.attach_many/4` + four supervised GenServers.

The re-cast was done inline across §1–§18 rather than as a supersede layer, following the sub-C/sub-B/sub-A precedent. Every Rust idiom was translated to its BEAM equivalent:

| Session 7 (Rust) | Session 15 (BEAM) |
|---|---|
| `tracing` crate | `:telemetry` library |
| `tracing-subscriber::Registry` + `Layer` composition | `:telemetry.attach_many/4` + one handler module |
| `tracing-subscriber::EnvFilter` | `Logger` level config + per-handler filter inside `handle_event/4` |
| `tracing-subscriber::fmt::Layer` | `Telemetry.Metrics.ConsoleReporter` + stdlib `Logger` console backend |
| `tracing-appender::non_blocking` | `File.open(..., [:append, :raw, :binary, :delayed_write])` inside a supervised `GenServer` |
| `metrics` crate + `metrics-util` | `:telemetry_metrics` + `SnapshotReporter` (custom ≈150 lines) |
| Custom `MnemosyneEventLayer` (≈200 lines) | `Mnemosyne.Observability.Handler` attach (<100 lines) |
| `tokio::sync::broadcast` or `crossbeam-channel` for in-process fan-out | `:pg` process group for TUI fan-out |
| `#[tracing::instrument]` | `:telemetry.span/3` wrapper |
| `metric_names::*` compile-time constants | `Telemetry.Metrics.*` definitions + a catalogue-integrity test against this doc |
| `InMemoryRingLayer` per-session `VecDeque` | `RingBuffer` GenServer per session under `DynamicSupervisor` |
| `JsonlPersistLayer` non-blocking writer | `JsonlWriter` GenServer per session under `DynamicSupervisor` |
| `TuiBridgeLayer` bounded `mpsc::Sender` | `TuiBridge` GenServer owning a `:pg` group + per-client drop-oldest |
| `MetricsRecorderLayer` `metrics_util::Registry` | `Metrics` supervisor hosting `ConsoleReporter` + `SnapshotReporter` |
| `ObsidianMaterialiseLayer` (v1.5) | `Mnemosyne.Observability.ObsidianMaterialiser` GenServer (v1.5) |
| `mnemosyne metrics` Rust CLI subcommand | `mix mnemosyne.metrics` task |
| `mnemosyne diagnose` Rust CLI subcommand | `mix mnemosyne.diagnose` task |
| `tracing-subscriber` Layer re-entrancy (Risk 1) | `:telemetry` handler re-entrancy (Risk 1) |
| `tracing-appender` queue overflow | GenServer mailbox overflow |

**Material findings that shaped the rewrite beyond straight translation:**

1. **`:telemetry` detaches raising handlers.** A raise inside `Handler.handle_event/4` would permanently blind the daemon. §5.2 mandates a `try/rescue` wrapper that logs via `Logger` directly (not through M) and returns `:ok`. This is a new failure mode not present in the Rust design and is called out in Risk 6.
2. **No compile-time constants for metric names.** `:telemetry_metrics` uses string names by convention. The metric-name-catalogue test (§6) is the substitute for the Rust `const &'static str` compile-time typo protection. The test parses the `## 6.` section of this doc and asserts correspondence.
3. **`:pg` replaces the TUI mpsc bridge.** Rust's in-process `mpsc::Sender` has no BEAM analogue that handles multi-client fan-out as a single primitive. `:pg` (stdlib since OTP 23) gives process-group membership without a `Phoenix.PubSub` dep. Per-client backpressure is handled client-side by the daemon's session process.
4. **F's routing surface materially grew §6.** The nine Task 24 event structs add eight new counters to the metric catalogue. The §6 table is ~50% larger than the Rust draft, not because of pivot effects but because F was still pending when the original brainstorm ran. This is a catch-up, not a regression.
5. **A's boot sequence lands two events in M's sealed set.** A's §A.Reference algorithm step 11 emits `%Vault.BootReady{}`; the hard-error exit path emits `%Vault.MarkerError{}`. Neither existed at the original brainstorm because A's amendment landed in Session 14. The §4.1 sealed set now enumerates them.

### Q7 — Reporter selection: `:telemetry_metrics_prometheus` vs `prom_ex` vs OpenTelemetry

**Options considered:**

- **A — `prom_ex`** — ships Phoenix / Ecto / Broadway plugins, a Plug endpoint, pre-built Grafana dashboards. The "Rails-like batteries included" choice.
- **B — `:telemetry_metrics_prometheus`** — thin reporter; Prometheus text-format endpoint only; no Phoenix dependency.
- **C — OpenTelemetry (`:opentelemetry` + OTLP exporter)** — most portable but largest dep tree.
- **D — `ConsoleReporter` only** — no external export; defer Prometheus to v1.5+.

**Chosen: D for v1, B additive in v1.5.** Mnemosyne is not a Phoenix app; `prom_ex`'s plugins are dead weight. OTel is reserved for v2 when sub-P (team mode) brings distributed-tracing use cases. v1 ships `ConsoleReporter` + `SnapshotReporter` as the minimum; v1.5 layers `:telemetry_metrics_prometheus` on top without rewriting any call site.

**Rationale strength.** `:telemetry_metrics` is reporter-independent by design: the same `counter(...)` / `distribution(...)` calls feed any reporter attached to the supervisor tree. Switching or adding reporters is purely additive. This is the load-bearing reason for choosing `:telemetry_metrics` over a reporter-bound alternative like `prom_ex`.

## 20. Origin

Sub-project M was surfaced during sub-project C's brainstorm on 2026-04-13 (Session 6 of the mnemosyne-orchestrator backlog plan), when C explicitly disclaimed framework scope for its tactical `SpawnLatencyReport` and recorded a forward pointer to a proposed Sub-M (§11.5 of `2026-04-13-sub-C-adapters-design.md`). The mnemosyne-orchestrator triage at the end of Session 6 promoted M from the tail of the backlog to second position (after sub-project A) because every other sub-project's structured-logging needs route through M once it lands.

The original brainstorm executed in Session 7 of the mnemosyne-orchestrator backlog plan, on 2026-04-13, using the `superpowers:brainstorming` skill. The user's explicit steer mid-brainstorm was *"use existing tooling and libraries wherever possible. This is not an interesting task."* — which collapsed several remaining clarifying questions in favour of the standard-tool answer at every fork.

**Session 15 amendment (2026-04-15).** Sub-F's Session 9 commitment to a persistent BEAM daemon invalidated every Rust-specific surface of the original Session 7 spec. Sub-A's Session 14 amendment added `%Vault.BootReady{}` and `%Vault.MarkerError{}` as new producers. Sub-B's Session 12 amendment formalised the seven `%PhaseLifecycle{}` events at §4.4. Sub-C's Session 11 amendment defined the five `Mnemosyne.Event.*` structs at §3.3 + §11.4 and the three-way parallel-emit discipline at §7.2. Sub-F's sibling plan at Task 24 (Session 13) committed to nine routing/actor/rule event structs. This amendment absorbs all five upstreams in one inline rewrite, following the sub-C/sub-B/sub-A precedent of replacing §1–§N with fresh content and preserving the original Q1–Q5 in the Decision Trail with correction notes. Q6 records the BEAM pivot translation table; Q7 records the reporter selection decision that was previously implicit.
