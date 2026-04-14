---
title: Sub-project C — Harness Adapter Layer Design
status: design-complete
original-brainstorm-date: 2026-04-13
date: 2026-04-15
author: brainstorm Session 6 + pivot rewrite Session 11
parent-plan: mnemosyne-orchestrator
sibling-plan: sub-C-adapters
depends-on:
  - sub-B-phase-cycle (brainstorm done 2026-04-12; F amendment pending)
  - sub-E-ingestion (brainstorm done 2026-04-12; F amendment pending)
  - sub-F-hierarchy (brainstorm done 2026-04-14 — committed Mnemosyne to a persistent BEAM daemon)
  - BEAM PTY spike (done 2026-04-15 — validated pipes-only `erlexec`)
unblocks:
  - sub-B-phase-cycle (v1 dogfood acceptance test)
  - sub-F-hierarchy (sibling plan scaffolding)
constrains:
  - sub-B-phase-cycle (PlanActor-hosted session consumption, `SessionLifecycle` event, sentinel executor — see §11.1)
reserves-for:
  - sub-O-model-mixing (multi-adapter surface area — see §11.5)
relates-to:
  - sub-M-observability (`:telemetry` + typed `Mnemosyne.Event.*` structs — see §11.4)
---

## Overview

This document specifies the design of Mnemosyne's harness adapter layer, the abstraction that lets the Mnemosyne daemon spawn, control, observe, and terminate LLM coding harnesses (Claude Code in v1, future adapters in sub-O / v1.5+) as managed OS child processes under OTP supervision. It is the implementation contract for sub-project C in the Mnemosyne orchestrator merge plan.

Mnemosyne runs as a persistent BEAM daemon (see sub-F's design doc), and the harness adapter layer is an Elixir component inside it. `PlanActor` GenServers consume adapter sessions to drive phase-cycle work, and F's Level 2 routing agent and sub-E's ingestion pipeline both spawn internal reasoning sessions through the same adapter — so C is a first-class abstraction across the whole daemon, not just a plan-cycle hosting layer.

C is on the critical path for sub-B's v1 dogfood acceptance test: B's phase-cycle executor currently holds a stub adapter; landing C's real `Mnemosyne.HarnessAdapter.ClaudeCode` is the swap that unblocks the orchestrator's first end-to-end run.

## Table of Contents

1. [Scope and Goals](#1-scope-and-goals)
2. [Architecture](#2-architecture)
3. [The adapter behaviour and typed events](#3-the-adapter-behaviour-and-typed-events)
4. [`Mnemosyne.HarnessAdapter.ClaudeCode`](#4-mnemosynneharnessadapterclaudecode)
5. [`Mnemosyne.HarnessAdapter.FixtureReplay`](#5-mnemosynneharnessadapterfixturereplay)
6. [Tool profile enforcement](#6-tool-profile-enforcement)
7. [Cold-spawn budget and v1.5 warm-pool](#7-cold-spawn-budget-and-v15-warm-pool)
8. [Testing strategy](#8-testing-strategy)
9. [Risks](#9-risks)
10. [Open implementation questions](#10-open-implementation-questions)
11. [Cross-sub-project requirements](#11-cross-sub-project-requirements)
12. [Appendix A — Decision Trail](#appendix-a--decision-trail)
13. [Appendix B — Dependency footprint (`mix.exs`)](#appendix-b--dependency-footprint-mixexs)
14. [Appendix C — Glossary](#appendix-c--glossary)

---

## 1. Scope and Goals

### 1.1 In scope

- A `Mnemosyne.HarnessAdapter` behaviour abstracting LLM coding harnesses (Claude Code, future adapters).
- Two concrete v1 implementations: `Mnemosyne.HarnessAdapter.ClaudeCode` and `Mnemosyne.HarnessAdapter.FixtureReplay`.
- OS child-process spawn, prompt delivery, output streaming, user-message injection, lifecycle, and process-group termination for the Claude Code adapter via `erlexec`.
- Tool profile enforcement at spawn time (CLI flags) and via stream-side defence-in-depth (runtime event inspection).
- A **tool-call boundary** for in-session Queries: the adapter intercepts a set of Mnemosyne-injected tools (`ask_expert`, `dispatch_to_plan`, `read_vault_catalog`) and routes them through F's daemon router before returning results to the running session.
- A JSON Lines fixture format for deterministic test replay.
- A dev-only fixture-recording Mix task (`mix mnemosyne.dev.record_fixture`).
- Latency instrumentation as a v1 acceptance-gate signal, emitted as `Mnemosyne.Event.SpawnLatencyReport` via `:telemetry`.
- Process-group termination via `erlexec`'s `:kill_group` from v1.
- A typed event set (`Mnemosyne.Event.*`) at the adapter boundary, consumed by B's executor, E's pipeline, and sub-M's observability layer.
- Top-level Elixir modules under `lib/mnemosyne/harness_adapter/` inside the Mnemosyne daemon application.

### 1.2 Deliberately out of scope

- **Codex and Pi adapters** — v1 ships Claude Code only. The behaviour exists in v1 with two implementors (live + fixture replay); additional harness implementations are sub-O's territory.
- **Warm-pool reuse / process pooling** — deferred to v1.5, conditional on a measurable acceptance gate (see §7).
- **PTY-based interactive harness control** — explicitly rejected. `claude --print --input-format stream-json --output-format stream-json` is pure NDJSON over stdio; a pseudo-terminal is not required and, per the spike, actively breaks the input path in combination with `erlexec`'s `:stdin` option. See Appendix A Q1.
- **Mid-session tool-profile swap** — v1 ships immutable profiles per session; swapping is a future `GenServer.call/3` if ever needed.
- **Structured logging / metrics framework** — C consumes sub-M's `:telemetry` + typed-struct pattern but does not own the framework. See §11.4.
- **A harness → Mnemosyne control channel** — forbidden by the "no slash commands inside the harness" rule. Observation is required and explicitly allowed; control is not.
- **Windows support** — the daemon targets macOS and Linux only; `erlexec`'s process-group primitives are POSIX-specific.
- **Authentication / API-key management** — Claude Code owns its credential storage; the adapter spawns the binary and trusts it.
- **Multi-node / distributed adapters** — team mode (sub-P, v2+) decides whether adapters run local-only or are addressable across BEAM distribution.

### 1.3 Goals, in priority order

1. **Sub-B's v1 dogfood acceptance test must work.** C's `ClaudeCode` adapter is the swap target for B's stub. Anything that delays this test delays the entire orchestrator merge.
2. **Hard errors by default.** Unexpected conditions, schema drift, malformed events, profile violations, exec-port loss — all fail loud and fast with clear diagnostics through explicit `{:error, reason}` returns that OTP supervisors observe. No silent degradation.
3. **Single-owner-per-state discipline.** Session state lives in exactly one GenServer per live session. All interactions are typed messages through `handle_call/3` / `handle_cast/2` / `handle_info/2`. No shared mutable state; no cross-process ETS rendezvous for hot-path state.
4. **Bidirectional interactive sessions.** A user attached to a live session through the TUI must be able to read streaming output, inject user messages, observe tool use, and interrupt in real time without leaving Mnemosyne's attached-client UI.
5. **Process-group termination from v1.** No leaked tool subprocesses, no orphaned MCP server children, no port leaks after `terminate/1`. `erlexec`'s `:kill_group` option is load-bearing.
6. **Defence-in-depth tool enforcement.** Spawn-time CLI flags + stream-side `handle_info/2` check. Trust the flag, verify the stream.
7. **Deterministic test replay.** A `FixtureReplay` adapter that mirrors the live adapter's GenServer shape and event flow, so threading bugs in consumers (B's executor, E's pipeline) surface in CI without a real `claude` binary.
8. **Minimal dependency footprint.** One new Hex dep (`erlexec`). Everything else is standard OTP (`GenServer`, `DynamicSupervisor`, `Process`, `:telemetry`) plus `jason`, which is already present.
9. **Tool-call-boundary extensibility.** C's GenServer exposes a hook for intercepting a whitelisted set of `tool_use` events and routing them through F's router. Sub-F's in-session Queries depend on this.

### 1.4 Non-goals

- **Performance optimisation beyond the cold-spawn gate.** The spec acknowledges measurable latency targets (5s p95 cold-spawn) but does not optimise beyond them in v1.
- **General-purpose process management library.** The adapter is purpose-built for LLM harness sessions; it does not aspire to be a re-usable subprocess wrapper.
- **Cross-platform parity beyond macOS + Linux.** Windows is not a v1 target.
- **Replacing or extending Claude Code's authentication model.** Adapter spawns the binary, trusts the binary's credential handling.
- **Custom NIFs for process management.** `erlexec` runs its C++ port program (`exec-port`) as a separate OS process outside BEAM schedulers; no NIF-scheduler risks are introduced.

---

## 2. Architecture

### 2.1 High-level diagram

```mermaid
flowchart TB
    subgraph BExecutor["Sub-B's phase-cycle executor<br/>(inside a PlanActor GenServer)"]
        PhaseRunner["PhaseRunner<br/>handle_info/2 on :harness_event"]
        SentinelMatcher["Sliding-buffer<br/>sentinel matcher"]
    end

    subgraph SubC["lib/mnemosyne/harness_adapter/ (this design)"]
        Behaviour["@behaviour HarnessAdapter<br/>typed Mnemosyne.Event.*"]
        CCAdapter["ClaudeCode adapter module<br/>+ Session GenServer"]
        FRAdapter["FixtureReplay adapter module<br/>+ Session GenServer"]
        Sup["HarnessAdapter.Supervisor<br/>(DynamicSupervisor)"]
    end

    subgraph CCInternal["ClaudeCode Session GenServer internals"]
        State["GenServer state:<br/>ospid, tool_profile,<br/>sentinel_buffer, staging_dir,<br/>spawned_at, first_chunk_seen"]
        Erlexec["erlexec / exec-port<br/>pipes-only NDJSON over stdio"]
        ChildProc["claude --print --verbose<br/>--input-format stream-json<br/>--output-format stream-json"]
    end

    subgraph DaemonRouter["Sub-F router"]
        Router["Mnemosyne.Router"]
        Experts["ExpertActor queries"]
        Dispatch["PlanActor dispatches"]
    end

    BExecutor -->|GenServer.call spawn| Behaviour
    Behaviour -->|start_child| Sup
    Sup -->|supervises| CCAdapter
    Sup -->|supervises| FRAdapter

    PhaseRunner -->|GenServer.cast send_user_message| CCAdapter
    CCAdapter -->|:harness_event messages| PhaseRunner
    PhaseRunner --> SentinelMatcher

    CCAdapter --> State
    State -->|:exec.send stdin envelopes| Erlexec
    Erlexec <-->|stdin/stdout/stderr pipes| ChildProc
    Erlexec -->|{:stdout, ospid, line}| State
    Erlexec -->|{:stderr, ospid, line}| State
    Erlexec -->|{:DOWN, _, _, _, _}| State

    CCAdapter -->|Mnemosyne-tool intercept| Router
    Router --> Experts
    Router --> Dispatch
    Router -->|tool_result| CCAdapter
    CCAdapter -->|:exec.send tool_result envelope| Erlexec
```

### 2.2 Module layout

```
lib/mnemosyne/harness_adapter/
├── harness_adapter.ex             # @behaviour + @callback specs; @type aliases
├── event.ex                       # Mnemosyne.Event.* typed structs
├── supervisor.ex                  # DynamicSupervisor for session GenServers
├── claude_code/
│   ├── adapter.ex                 # @behaviour impl; spawn/1 starts a supervised Session
│   ├── session.ex                 # GenServer per live session; handle_info for erlexec msgs
│   ├── stream_json.ex             # NDJSON line parser; dispatches typed events
│   ├── spawn.ex                   # argv builder; tool-profile → flags; cmux mitigation args
│   ├── input.ex                   # user-message / tool_result envelope serialisation
│   └── tool_call_boundary.ex      # intercepts Mnemosyne-injected tools; calls Router; replies
└── fixture_replay/
    ├── adapter.ex                 # @behaviour impl; spawn/1 starts a Replay GenServer
    ├── session.ex                 # GenServer that walks a FixtureRecord list on a timer
    └── format.ex                  # FixtureRecord JSON-Lines schema; encode/decode helpers
```

Plus, in `lib/mix/tasks/`:

```
lib/mix/tasks/
└── mnemosyne.dev.record_fixture.ex   # mix mnemosyne.dev.record_fixture --output <path> [...]
```

The `mnemosyne.dev.*` Mix task namespace is new. Dev tasks are visible in `mix help` but tagged "dev-only — not for end users". Future dev tools (e.g., `mix mnemosyne.dev.replay_fixture`) live under the same namespace.

### 2.3 Supervision tree placement

`Mnemosyne.HarnessAdapter.Supervisor` is a `DynamicSupervisor` child of the top-level `Mnemosyne.Supervisor`:

```elixir
children = [
  Mnemosyne.Vault.Registry,
  Mnemosyne.Router,
  Mnemosyne.HarnessAdapter.Supervisor,       # dynamic — session GenServers
  Mnemosyne.PlanActor.Supervisor,            # dynamic — plan actors
  Mnemosyne.ExpertActor.Supervisor,          # dynamic — expert actors
  Mnemosyne.Ingestion.PipelineSupervisor,
  ...
]
Supervisor.start_link(children, strategy: :one_for_one)
```

- `:one_for_one` at the top level: a crashing session GenServer does not take down the daemon. `DynamicSupervisor` at the middle level: transient sessions come and go without affecting the framework.
- Session GenServers are `restart: :temporary` — a crashed session is not auto-restarted, since plan state is handled at the PlanActor layer (the actor observes the crash via `Process.monitor/1` and decides whether to re-spawn).
- `exec-port` (erlexec's C++ port program) is supervised by erlexec's own supervision tree, which Mnemosyne starts as a dep at application boot.

### 2.4 Dependency footprint

See Appendix B for the mix.exs projection. Short form:

- **`erlexec` (new)** — C++ port program for spawning OS children with process-group termination, stdin wiring, and bidirectional pipe I/O. The only BEAM-native library that covers process-group kill of grandchildren. Not a NIF; `exec-port` runs as a separate OS process outside BEAM schedulers.
- **`jason` (pre-existing)** — NDJSON line encode/decode for stream-json events and the JSON-Lines fixture format. Already a daemon-wide dep.
- **`:telemetry` (pre-existing)** — OTP-blessed event/metrics transport, used by sub-M's observability layer. C emits `:telemetry.execute/3` calls at boundaries; histogram / counter handlers are attached under sub-M.

No PTY library, no custom NIF, no async runtime beyond BEAM's own scheduler.

---

## 3. The adapter behaviour and typed events

### 3.1 The `@behaviour` surface

```elixir
defmodule Mnemosyne.HarnessAdapter do
  @moduledoc """
  The behaviour every LLM coding harness adapter implements.

  Implementations supervise OS child processes and expose one GenServer
  per live session. The behaviour's `spawn/1` returns the session pid;
  all subsequent interactions are typed messages on that pid.
  """

  @type tool_profile :: :ingestion_minimal | :research_broad
  @type harness_kind :: :claude_code | :fixture_replay

  @type spawn_opts :: %{
          required(:prompt) => String.t(),
          required(:working_dir) => Path.t(),
          required(:tool_profile) => tool_profile(),
          required(:session_id) => String.t(),
          optional(:staging_dir) => Path.t(),
          optional(:router) => pid() | atom()
        }

  @callback kind() :: harness_kind()

  @callback spawn(spawn_opts()) :: {:ok, pid()} | {:error, term()}
end
```

The behaviour intentionally exposes a *single* callback (`spawn/1`) plus the adapter's `kind/0`. Everything else is the session GenServer contract, which is defined by a documented message set rather than additional behaviour callbacks — BEAM's convention is that GenServer contracts are message shapes, not function signatures.

### 3.2 The session GenServer contract

Every harness session GenServer started by an adapter answers the same documented call/cast/info messages. Consumers (B's executor, E's pipeline, F's router) program against these shapes, not against a polymorphic session type:

```elixir
# Client API on top of the GenServer (exposed as Mnemosyne.HarnessSession)
@spec send_user_message(pid(), String.t()) :: :ok | {:error, term()}
@spec terminate(pid()) :: :ok
@spec await_exit(pid(), timeout()) :: {:ok, Mnemosyne.Event.SessionExitStatus.t()} | {:error, term()}
@spec session_id(pid()) :: {:ok, String.t()}
@spec attach_consumer(pid(), pid()) :: :ok   # route :harness_event msgs to consumer pid
@spec detach_consumer(pid(), pid()) :: :ok
```

The implementation is a thin wrapper that calls `GenServer.call/3` or `GenServer.cast/2` on the session pid. B's executor may attach itself as a consumer; the TUI client may attach in addition so the user sees live output without B's executor double-forwarding.

Events flow *from* the session GenServer *to* the consumer pid as `{:harness_event, session_id, %Mnemosyne.Event.* {}}` info messages. This is BEAM's natural shape — consumers implement `handle_info/2` pattern-matching on the event type.

### 3.3 Typed events (`Mnemosyne.Event.*`)

The boundary between C's session GenServer and its consumers is a sealed set of typed structs. Every event consumers care about has its own struct; the struct names are stable and consumers pattern-match on them. This is the BEAM equivalent of the "hybrid `:telemetry` + typed struct" pattern committed in sub-M's design.

```elixir
defmodule Mnemosyne.Event.HarnessOutput do
  @enforce_keys [:session_id, :kind, :text, :at]
  defstruct [:session_id, :kind, :text, :at, :meta]

  @type kind :: :stdout | :stderr | :tool_use | :internal_message
  @type t :: %__MODULE__{
          session_id: String.t(),
          kind: kind(),
          text: String.t(),
          at: DateTime.t(),
          meta: map()
        }
end

defmodule Mnemosyne.Event.SessionLifecycle do
  @enforce_keys [:session_id, :transition, :at]
  defstruct [:session_id, :transition, :at, :meta]

  @type transition ::
          :ready
          | {:turn_complete, subtype :: String.t()}
          | {:exited, exit_status :: term()}
  @type t :: %__MODULE__{...}
end

defmodule Mnemosyne.Event.SpawnLatencyReport do
  @enforce_keys [:session_id, :spawned_at, :first_chunk_at, :init_event_at,
                 :harness_kind, :tool_profile]
  defstruct [...]
end

defmodule Mnemosyne.Event.SessionExitStatus do
  @enforce_keys [:session_id, :reason]
  defstruct [:session_id, :reason]

  @type reason ::
          {:clean_exit, exit_code :: integer()}
          | {:terminated, signal :: integer() | nil}
          | :crashed_before_ready
          | {:tool_profile_violation, map()}
          | {:protocol_error, reason :: term()}
end

defmodule Mnemosyne.Event.HarnessError do
  @enforce_keys [:session_id, :error, :at]
  defstruct [:session_id, :error, :at, :dump_ref]
end
```

These structs are what get routed to consumers via `{:harness_event, session_id, event}` info messages. The same structs are emitted via `:telemetry.execute/3` for sub-M's observability consumption — one event, two transports.

### 3.4 Contract requirements C honours from B's executor design

1. **Cold-start latency target: < 3 s per session.** Warm-pool reuse (§7) is the eventual strategy; v1 ships cold-spawn only under the C-1 gate.
2. **Tool profile enforcement is C's responsibility.** A harness that attempts a disallowed tool fails the session at the adapter level (`{:error, :tool_profile_violation}`), not via prompt suggestion.
3. **`terminate/1` is non-blocking and idempotent.** Returns `:ok` within microseconds; actual teardown is observed via the attached consumer receiving a `SessionLifecycle {:exited, _}` event.
4. **Event delivery is asynchronous and ordered.** Consumers receive `{:harness_event, _, _}` info messages in the order they were produced by the session GenServer; BEAM mailbox ordering is relied upon. No polling.
5. **`FixtureReplay` is a first-class adapter.** Fixture replay is a real `@behaviour` implementation, not a mock.
6. **Working directory on spawn is the staging root.** Passed through to `erlexec` via the `{:cd, cwd}` option.
7. **Session ID is passed through to Claude Code's session tracking.** Claude Code: `--session-id <name>` (verified day 1).
8. **No *control* channel from harness to Mnemosyne.** The harness cannot unilaterally call Mnemosyne actions, trigger phase transitions, or invoke arbitrary daemon functions. Observation is unrestricted — Mnemosyne reads the full stream-json output and reacts. **Tool-call boundary (§4.5) is not a control channel**: the daemon *injects* tools into the session so that the LLM asking a question becomes a daemon router call, but the daemon controls the tool set and the responses; the harness has no ability to probe or invoke anything outside the whitelisted set.

---

## 4. `Mnemosyne.HarnessAdapter.ClaudeCode`

### 4.1 Process model — bidirectional NDJSON over stdio

The Claude Code adapter spawns `claude` as an OS child via `erlexec` in pipes-only mode. Claude Code's `--print --input-format stream-json --output-format stream-json` flags make it an NDJSON-over-stdio server: one JSON event per line on stdin going in, one JSON event per line on stdout coming out, stderr reserved for diagnostics. **No pseudo-terminal is involved** — the BEAM PTY spike validated that PTY mode actively breaks the input path, and no TUI features are needed since "no slash commands in the harness" forbids the interactive Claude Code surface.

The decision rationale, alternatives considered (PTY-wrapped, headless one-shot), and the spike-corrected premise are recorded in Appendix A Q1.

#### 4.1.1 The spawn command

```
claude --print --verbose \
       --input-format stream-json --output-format stream-json \
       --session-id "<session_id from caller>" \
       --setting-sources project,local \
       --no-session-persistence \
       <profile-derived flags from §6>
```

- `--setting-sources project,local` and `--no-session-persistence` are mandatory on every daemon-spawned session. They suppress the ~10 KB of user-global cmux `SessionStart` hook JSON that would otherwise contaminate stdout before the first assistant event. Without these flags, the NDJSON stream is unusable in any environment where the user has cmux hooks configured. The spike validated both.
- The initial prompt is **not** passed as a CLI arg. With `--input-format stream-json`, the CLI-arg form returns `Input must be provided either through stdin or as a prompt argument when using --print` — the CLI insists that every prompt, including the first, arrives as an NDJSON user-message envelope on stdin. (Resolved by the spike; see §10 Q3.)

#### 4.1.2 The `erlexec` spawn call

```elixir
defp spawn_claude(argv, cwd) do
  exec_opts = [
    :monitor,
    :stdin,
    {:stdout, self()},
    {:stderr, self()},
    :kill_group,
    {:kill_timeout, 1},
    {:cd, to_charlist(cwd)}
  ]

  :exec.run(argv, exec_opts)
end
```

The option set is load-bearing and validated by the spike:

| Option | Purpose |
|---|---|
| `:monitor` | Delivers a `{:DOWN, _, :process, _, reason}` message to the GenServer when the OS child exits. |
| `:stdin` (bare atom) | Wires the caller's pipe to the child's real stdin. **Required** — erlexec defaults to `:null` if `:stdin` is not passed, and the child would read nothing. |
| `{:stdout, self()}` | Sends each child stdout line as `{:stdout, ospid, binary}` to the GenServer pid. |
| `{:stderr, self()}` | Sends each child stderr line as `{:stderr, ospid, binary}` to the GenServer pid. |
| `:kill_group` | Makes termination target the whole process group, including grandchildren (Claude Code's tool subprocesses, MCP server children). Spike verified against a `/bin/sh -c "sleep 60 & wait"` grandchild probe. |
| `{:kill_timeout, 1}` | One-second granularity on the SIGTERM→SIGKILL escalation; finer granularity is handled by explicit `:exec.kill/2` calls from the GenServer (§4.4). |
| `{:cd, cwd}` | Sets the child's working directory to B's staging root (contract #6). |

#### 4.1.3 The user-message envelope (stream-json input format)

```json
{"type":"user","message":{"role":"user","content":[{"type":"text","text":"<message text>"}]}}
```

One envelope per line on stdin, newline-terminated. `Mnemosyne.HarnessAdapter.ClaudeCode.Input.encode_user_message/1` produces the envelope via `Jason.encode!/1`; the GenServer calls `:exec.send(ospid, line <> "\n")` to push it to the child. The initial prompt and every subsequent user injection share this exact shape.

### 4.2 The Session GenServer

`Mnemosyne.HarnessAdapter.ClaudeCode.Session` is a `GenServer` that owns the single source of truth for one live session. It is started by the adapter's `spawn/1` under `Mnemosyne.HarnessAdapter.Supervisor` (a `DynamicSupervisor`).

#### 4.2.1 State

```elixir
defmodule Mnemosyne.HarnessAdapter.ClaudeCode.Session do
  use GenServer

  defstruct [
    :session_id,
    :ospid,                      # erlexec-assigned OS pid
    :tool_profile,
    :staging_dir,
    :spawned_at,                 # DateTime set at :exec.run/2 return
    :first_chunk_at,             # set on first :stdout message
    :init_event_at,              # set on system/init NDJSON line
    :consumers,                  # MapSet of pids receiving :harness_event
    :sentinel_buffer_hook,       # nil unless B's executor registered one
    :router,                     # pid/atom of the daemon router
    :injected_tools,             # MapSet of tool names owned by Mnemosyne
    :first_chunk_received,       # boolean — feeds CrashedBeforeReady heuristic
    :terminating                 # atomic-ish bool; idempotency guard
  ]
end
```

No ETS tables, no `:persistent_term` per session, no cross-process state. Everything lives in the GenServer struct; hot-path updates are function returns from `handle_info/2`.

#### 4.2.2 Message set (internal)

`handle_info/2` pattern-matches on these shapes:

| Message | Source | Action |
|---|---|---|
| `{:stdout, ospid, line}` | erlexec | Set `first_chunk_at` if unset. Parse the NDJSON line (§4.3). Tool-profile defence-in-depth (§6). Tool-call boundary intercept (§4.5). Dispatch typed event to consumers. |
| `{:stderr, ospid, line}` | erlexec | Forward as `%HarnessOutput{kind: :stderr}` event to consumers. Record in the error-tail ring buffer (§11.4). |
| `{:DOWN, _, :process, _, reason}` | erlexec `:monitor` | Emit `%SessionLifecycle{transition: {:exited, reason}}`, emit `%SessionExitStatus{}`, notify pending `await_exit` waiters, stop the GenServer. |
| `:crashed_before_ready_check` | `Process.send_after/3` (2 s timer from init) | If `first_chunk_received == false` and `{:DOWN, ...}` already arrived with non-zero reason, set exit reason to `:crashed_before_ready` before stopping. |
| `{:router_reply, tool_use_id, result}` | F's router (async) | Encode as a `user/tool_result` envelope; `:exec.send/2` it to the child; continue the session. |

`handle_call/3` answers synchronous client API calls:

| Call | Purpose |
|---|---|
| `{:send_user_message, text}` | Encode + `:exec.send/2`. Returns `:ok` or `{:error, reason}`. |
| `{:attach_consumer, pid}` | Add pid to `consumers`; `Process.monitor/1` it so auto-detach on consumer death. |
| `{:detach_consumer, pid}` | Remove from `consumers`. |
| `:session_id` | Returns the session_id string. |
| `{:await_exit, timeout}` | Registers the caller as a reply-to-on-DOWN; the GenServer answers from the `{:DOWN, ...}` handler. |

`handle_cast/2` answers `terminate` (see §4.4).

#### 4.2.3 Why one GenServer per session

- **Single-owner-per-state.** The GenServer holds every mutable field. All mutations are function returns from its own callbacks. No locking.
- **Natural backpressure.** BEAM mailboxes are unbounded but delivery is scheduled cooperatively; a slow consumer does not back-pressure the GenServer's own mailbox because the GenServer drains its mailbox in FIFO order on each `handle_info/2` call.
- **Fault isolation.** A session crash is isolated by its `DynamicSupervisor` parent; other sessions are unaffected.
- **Trivial multi-consumer attach.** The TUI client can attach alongside B's executor without any re-plumbing.
- **Maps onto erlexec naturally.** erlexec's `{:stdout, ...}` / `{:stderr, ...}` / `{:DOWN, ...}` messages land directly in the GenServer's mailbox. No custom inbox, no additional process.

### 4.3 Stream-json parser

#### 4.3.1 Event shapes (locked against the spike's canonical output)

`lib/mnemosyne/harness_adapter/claude_code/stream_json.ex` parses each NDJSON line with `Jason.decode!/1` and pattern-matches the result into a typed set. The field names below were captured from the spike's real session log at `spikes/beam_pty/results/full-run.log` and are the authoritative shape for v1.

| `type` | Fields observed | Interpretation |
|---|---|---|
| `"system"` (with `subtype: "init"`) | `session_id`, `tools`, `mcp_servers`, `model`, `permissionMode` | Session is initialised; emit `%SessionLifecycle{transition: :ready}`. Record `init_event_at`. Fire the cold-spawn latency report once all three timestamps are populated. |
| `"system"` (other subtypes) | varies | Forward as `%HarnessOutput{kind: :internal_message}` with the original JSON in `meta`. |
| `"rate_limit_event"` | token counters, throttle state | Forward as `%HarnessOutput{kind: :internal_message}`. Sub-M may attach metrics handlers. |
| `"assistant"` | `message.content`: list of blocks | Each block emits its own event, in order, sharing the parsed-at timestamp. |
| `"assistant"` → `content[].type = "thinking"` | `text` | Forward as `%HarnessOutput{kind: :internal_message, meta: %{block: :thinking}}`. |
| `"assistant"` → `content[].type = "text"` | `text` | Forward as `%HarnessOutput{kind: :stdout}`. Feed through B's sentinel matcher hook if registered. |
| `"assistant"` → `content[].type = "tool_use"` | `id`, `name`, `input` | (a) Defence-in-depth tool-profile check (§6); (b) if `name` is in `state.injected_tools`, intercept (§4.5); otherwise forward as `%HarnessOutput{kind: :tool_use}`. |
| `"assistant"` → `content[].type = "tool_result"` | `tool_use_id`, `content`, `is_error` | Forward as `%HarnessOutput{kind: :internal_message, meta: %{block: :tool_result}}`. |
| `"user"` | echo of what we sent | Forward as `%HarnessOutput{kind: :internal_message}`. Useful for verification rendering; not consumed by B's phase machine. |
| `"result"` | `subtype`, `is_error`, `result` | Protocol-level turn complete: emit `%SessionLifecycle{transition: {:turn_complete, subtype}}`. Record the subtype so `await_exit` can return it. |

The parser uses `Jason.decode!/1` with default options; on a decode error the session is terminated with `:protocol_error` (hard failure, per goal 2). Forward-compatibility against Claude Code schema additions comes from the pattern matching being "match the known shapes and forward anything else as `:internal_message`" — additive schema changes land as richer `meta` fields without breaking existing handlers.

#### 4.3.2 `SessionLifecycle` semantics — protocol-level vs task-level

`SessionLifecycle` events are **protocol-level** state transitions surfaced by the adapter. They tell consumers "the harness's protocol state has changed in a structured way" — the harness is ready (`:ready`), a turn has ended (`{:turn_complete, subtype}`), the process has exited (`{:exited, reason}`). Consumers pattern-match on these atom/tuple shapes.

These are explicitly **NOT task-level "the LLM thinks it has completed the assigned work" signals.** Protocol-level "turn over" tells you the model stopped emitting tokens for this round; it does *not* tell you whether the model judged the task complete or just paused for tool use, hit max_tokens mid-thought, or is asking a clarifying question. The two concerns are layered:

| Concept | Source | Mechanism | Detection layer |
|---|---|---|---|
| Protocol-level turn boundary | Claude Code's `"result"` event | Structured NDJSON line | C's adapter (this section) |
| Task-level "I am done with the work" | The LLM's own judgment | Prompt-instructed sentinel string in `%HarnessOutput{kind: :stdout}` | B's executor (§11.1 fifth requirement) |

A consumer that wants to know "the LLM has finished its job" must listen for **sentinel matches on stdout events** via B's executor, not for `{:turn_complete, _}` lifecycle events from C. Conflating the two would cause Mnemosyne to transition phases the moment Claude Code finished a single turn, even when the LLM was mid-task.

C's adapter has no knowledge of sentinel strings, phase semantics, or task completion criteria — it only surfaces what Claude Code's protocol tells it. Sentinel detection lives in B because sentinel strings are coupled to phase prompts (which B owns) and because the mechanism is harness-agnostic (every harness produces text output regardless of structured-event support, so future bare-LLM adapters get sentinel detection for free without C-side changes).

### 4.4 Process-group termination

A v1 correctness requirement, not a v1.5 deferral. `erlexec`'s `:kill_group` option plus explicit two-phase escalation in the GenServer.

#### 4.4.1 Spawn-side: process group by default

`:kill_group` in the `exec_opts` list (§4.1.2) makes `erlexec` create a process group for the child and target it on kill calls. The child's own PID becomes the PGID; Claude Code's tool subprocesses and MCP server children inherit the PGID via normal fork-then-exec semantics.

No additional work at spawn time.

#### 4.4.2 Terminate-side: two-phase SIGTERM → SIGKILL

```elixir
def handle_cast(:terminate, state) do
  if state.terminating do
    {:noreply, state}
  else
    :ok = :exec.kill(state.ospid, 15)   # SIGTERM to the group
    Process.send_after(self(), :escalate_kill, 500)
    {:noreply, %{state | terminating: true}}
  end
end

def handle_info(:escalate_kill, state) do
  case :exec.status(state.ospid) do
    {:status, _} -> :ok = :exec.kill(state.ospid, 9)   # SIGKILL the group
    _ -> :ok
  end
  {:noreply, state}
end
```

The 500 ms grace period matches `systemd-stop`'s default `TimeoutStopSec` tier and Docker `stop`'s default grace, a defensible "industry convention" choice.

`terminate` is **idempotent** via the `state.terminating` flag — a second cast is a no-op.

The actual observable end of the session is the `{:DOWN, _, :process, _, reason}` message from erlexec, which arrives whenever the OS child dies. The GenServer's `{:DOWN, ...}` handler emits the final `SessionLifecycle {:exited, reason}` event and stops.

#### 4.4.3 What this fixes

- Tool subprocesses spawned by Claude Code don't become orphans on terminate.
- MCP server children don't leak ports.
- `pgrep -f claude` after a clean shutdown shows zero residual processes on both macOS and Linux. Verified by the BEAM PTY spike against a grandchild probe.

### 4.5 Tool-call boundary for in-session Queries

Sub-F's daemon architecture introduces a new role for C: **in-session Queries**. A plan-cycle session may need to ask an expert a question ("how should I handle this lifetime annotation?") or dispatch a task to another plan ("this decision affects sub-A"). Those messages are F's `Query` and `Dispatch` primitives, routed through `routing.ex` by `Mnemosyne.Router`. The originating session cannot block on user input and cannot spawn its own child session directly — the daemon owns spawning.

C satisfies the requirement by injecting a whitelisted set of tools into the running Claude Code session and intercepting their `tool_use` events before forwarding anything to consumers.

#### 4.5.1 Injected tool set

| Tool name | Purpose |
|---|---|
| `ask_expert` | `Query` to an expert actor. Args: `{expert_id, question}`. Returns the expert's answer as the `tool_result`. |
| `dispatch_to_plan` | `Dispatch` a task to another plan's backlog. Args: `{target_plan, task_description, metadata}`. Returns `{:ok, dispatch_id}` as the `tool_result`. |
| `read_vault_catalog` | Read the full vault catalog (plans + experts + descriptions). Args: `{}`. Returns the catalog JSON as the `tool_result`. |

The set is frozen for v1. Additions in v1.5+ go through sub-O's multi-adapter work (the tool set becomes configurable per `[harnesses.*]` section) or directly as additional tools registered with the adapter at daemon boot.

#### 4.5.2 Injection mechanism (Q6 — resolved during implementation phase)

There are three live candidates:

- **(a) MCP server on a Unix socket**, pointed at by `--mcp-config`. Claude Code already speaks MCP; defining the Mnemosyne tools as an MCP server running inside the daemon (over a Unix socket under `<vault>/runtime/`) is the most documented path. The child process sees them as ordinary MCP tools and the adapter intercepts the `tool_use` events via stream-json.
- **(b) Tool-definition preamble on stdin**, inlined into the first user-message envelope. Depends on Claude Code accepting inline tool definitions via stream-json; not yet verified.
- **(c) Plugin shim via `--allowed-tools` whitelist**. Uses Claude Code's plugin layer to ship tool stubs that the daemon responds to. Most fragile — plugin shape may shift version-to-version.

Option (a) is preferred; the day-1 implementation task for §4.5 is a focused spike against the pinned `claude` version that validates the MCP-over-Unix-socket path end-to-end before the GenServer's tool-call boundary is wired. If (a) fails, (b) is the fallback; if (b) fails, (c). See §10 Q6.

#### 4.5.3 Intercept flow

When the stream-json parser observes an `assistant` event containing a `tool_use` block whose `name` is in `state.injected_tools`:

1. The GenServer extracts `(tool_use_id, name, input)` from the block.
2. It **does not** forward a `%HarnessOutput{kind: :tool_use}` event to consumers — the interception is invisible to B's executor and to the TUI.
3. It asynchronously calls `Mnemosyne.Router.handle_tool_call/4` with `(name, input, session_id, reply_ref: tool_use_id)`. The router resolves the target (expert or plan), runs the query/dispatch in a separate supervised task, and replies back to the session GenServer with `{:router_reply, tool_use_id, result}`.
4. The GenServer's `handle_info/2` clause for `{:router_reply, _, _}` serialises the result as a `user/tool_result` envelope and `:exec.send/2`s it to the child.
5. Claude Code sees the tool result and continues its turn.

Non-injected tool uses flow through unchanged — they are forwarded as `%HarnessOutput{kind: :tool_use}` events and, for `IngestionMinimal` sessions, run through the defence-in-depth check (§6).

#### 4.5.4 Why this is not a control channel

The harness is not calling Mnemosyne unilaterally. The daemon *injects* a specific tool set (the whitelist is the daemon's, not the harness's) and responds only to the tools it injected. The flow is:

```
daemon → injects whitelisted tools → harness uses them from within a tool-use turn →
    daemon intercepts the call → daemon decides the response → daemon returns tool_result →
    harness continues
```

which is structurally identical to any harness-to-MCP-tool call. The harness has no ability to probe outside the whitelist, no ability to trigger arbitrary daemon actions, and no ability to observe daemon state beyond what a `tool_result` returns. The "no slash commands" rule forbids the *user* typing slash commands into the harness; it forbids the *harness* running arbitrary Mnemosyne actions. Neither is weakened here.

### 4.6 Lifecycle and error handling

#### 4.6.1 `CrashedBeforeReady` heuristic

A session is `:crashed_before_ready` if:
- The `{:DOWN, ...}` arrives within **2 seconds** of `spawn/1` returning, AND
- `state.first_chunk_received == false`, AND
- The down-reason is non-normal (non-zero exit code or signal).

Detected via a `Process.send_after(self(), :crashed_before_ready_check, 2_000)` timer scheduled at GenServer init. When the timer fires, the handler checks the current state; when the real `{:DOWN, ...}` arrives, the handler also cross-checks. Whichever runs second produces the final classification.

Catches: missing API key, invalid model name, malformed config, OOM at startup, missing dependencies. The 2 s threshold is tunable as a module attribute.

#### 4.6.2 Error reasons

| Failure mode | `%SessionExitStatus{reason: _}` | Detection point |
|---|---|---|
| `claude` not on PATH | `{:spawn_failed, :claude_not_found}` | `:exec.which/1` / erlexec's spawn error |
| `erlexec` `:exec.run/2` failure | `{:spawn_failed, reason}` | `erlexec` error tuple |
| Tool profile mismatch in stream | `{:tool_profile_violation, %{profile: _, tool_name: _}}` | defence-in-depth check in `handle_info({:stdout, ...})` |
| Process exited fast with no output | `:crashed_before_ready` | `:crashed_before_ready_check` handler |
| Process killed by signal | `{:terminated, signal}` | `{:DOWN, ...}` handler |
| `:exec.send/2` broken pipe | `{:io_error, :broken_pipe}` | `handle_call({:send_user_message, _}, ...)` |
| NDJSON parse error | `{:protocol_error, jason_error}` | stream-json parser in `handle_info({:stdout, _, _}, _)` |
| `await_exit` after GenServer already stopped | caller gets `{:error, :noproc}` | standard `GenServer.call/3` behaviour |
| `send_user_message` after termination | `{:error, :session_terminated}` | guard in `handle_call` |
| `exec-port` crash mid-session | `{:exec_port_lost, reason}` | trapped by the session GenServer's `{:EXIT, _, _}` handler; see §10 Q7 |

Every error path calls `Mnemosyne.Observability.dump_event_tail/3` before the GenServer stops, flushing the session's recent event ring buffer to disk under `<staging>/harness-error-tail.log` for sub-M's triage tooling.

---

## 5. `Mnemosyne.HarnessAdapter.FixtureReplay`

### 5.1 Purpose and design parity

The fixture-replay adapter is a first-class `@behaviour` implementation, not a mock. It exists so that:

- **Sub-B's executor tests** can exercise the full phase-cycle + staging pipeline without a live `claude` process.
- **Sub-E's pipeline tests** can exercise the ingestion Stages 3/4 without spawning real LLM sessions.
- **Sub-C's own integration tests** can exercise the session GenServer's message handling, defence-in-depth, and lifecycle deterministically.

To maximise the value of the third use case, `FixtureReplay.Session` mirrors `ClaudeCode.Session`'s GenServer shape and event flow exactly — same client API, same `handle_call/3` / `handle_info/2` surface, same typed event set, same `{:harness_event, _, _}` delivery semantics. Consumers cannot tell the two apart except by asking `kind/0`.

**Rationale**: tests that exercise B's executor against `FixtureReplay` hit the same concurrency patterns as the live adapter, so message-ordering bugs in the executor's consumer path surface in CI without a real `claude` binary.

### 5.2 Fixture file format

The JSON-Lines format survives the pivot unchanged. Each line is a `FixtureRecord` encoded as a tagged JSON object:

```json
{"t":"output","chunk":{"kind":"stdout","text":"Analysing the backlog...\n","at":"2026-04-15T10:23:01Z"}}
{"t":"output","chunk":{"kind":"tool_use","text":"Read(/etc/hostname)","at":"2026-04-15T10:23:03Z"}}
{"t":"delay","ms":250}
{"t":"expect_user_input"}
{"t":"output","chunk":{"kind":"stdout","text":"READY FOR REFLECT\n","at":"2026-04-15T10:23:05Z"}}
{"t":"exit","status":{"kind":"clean_exit","exit_code":0}}
```

Record variants:

| `t` | Purpose |
|---|---|
| `output` | Emit a `%HarnessOutput{}` event to consumers via the standard `{:harness_event, _, _}` delivery. |
| `delay` | Pause the replay GenServer for `ms` milliseconds before processing the next record. Simulates streaming pacing for UI-rendering tests. Interruptible by `terminate`. |
| `expect_user_input` | Block until `send_user_message/2` is called. Text content is *not* validated; fixtures are output-only contracts. |
| `exit` | Terminal record: replay ends with the given `%SessionExitStatus{}` reason. |

`lib/mnemosyne/harness_adapter/fixture_replay/format.ex` owns encode/decode, with unit tests that round-trip every variant through `Jason.encode!` / `Jason.decode!` and through the struct<->map conversions. The runtime `%Mnemosyne.Event.*{}` structs and their on-disk representations stay in lockstep via a single `from_disk/1` / `to_disk/1` pair per type.

### 5.3 Fixture file layout and ownership

```
test/fixtures/harness_adapter/                 # owned by Sub-C
├── replay_clean_linear.jsonl
├── replay_multi_turn.jsonl
├── replay_terminated.jsonl
├── replay_tool_violation.jsonl
└── replay_crashed_before_ready.jsonl

test/fixtures/sub_b/harness_adapter/           # owned by Sub-B
├── work_phase_clean_exit.jsonl
├── reflect_phase_with_interjection.jsonl
└── triage_phase_terminated_mid_stream.jsonl

test/fixtures/sub_e/harness_adapter/           # owned by Sub-E
├── ingestion_stage3_section_classification.jsonl
└── ingestion_stage4_cross_section_synthesis.jsonl
```

C owns `test/fixtures/harness_adapter/` (fixtures exercising the adapter in isolation). B and E own their respective subtrees. Each owning sub-project commits and maintains its own fixtures; C's `format.ex` is the single source of truth for the schema.

### 5.4 Replay Session GenServer

```elixir
defmodule Mnemosyne.HarnessAdapter.FixtureReplay.Session do
  use GenServer

  defstruct [
    :session_id,
    :records,                  # list of FixtureRecord, head-consumed per step
    :consumers,                # MapSet of pids
    :awaiting_user_input?,     # true during an expect_user_input record
    :final_status,             # populated by exit record or terminate
    :terminating
  ]

  def init(%{session_id: sid, records: records}) do
    send(self(), :step)
    {:ok, %__MODULE__{session_id: sid, records: records, consumers: MapSet.new(),
                      awaiting_user_input?: false}}
  end

  def handle_info(:step, state) do
    case state.records do
      [%FixtureRecord.Output{chunk: chunk} | rest] ->
        broadcast(state, chunk)
        send(self(), :step)
        {:noreply, %{state | records: rest}}

      [%FixtureRecord.Delay{ms: ms} | rest] ->
        Process.send_after(self(), :step, ms)
        {:noreply, %{state | records: rest}}

      [%FixtureRecord.ExpectUserInput{} | _rest] ->
        {:noreply, %{state | awaiting_user_input?: true}}

      [%FixtureRecord.Exit{status: status} | _] ->
        broadcast_exit(state, status)
        {:stop, :normal, %{state | final_status: status}}

      [] ->
        broadcast_exit(state, %SessionExitStatus{reason: {:clean_exit, 0}})
        {:stop, :normal, state}
    end
  end

  def handle_call({:send_user_message, _text}, _from, %{awaiting_user_input?: true} = state) do
    [_consumed | rest] = state.records
    send(self(), :step)
    {:reply, :ok, %{state | records: rest, awaiting_user_input?: false}}
  end
  # ... plus the shared attach_consumer / detach_consumer / session_id / await_exit clauses
end
```

No erlexec, no subprocess, no process group. The GenServer walks the fixture list, sleeps via `Process.send_after/3`, and responds to client API calls with the same shape as the live adapter.

### 5.5 Fixture recording: `mix mnemosyne.dev.record_fixture`

A dev-only Mix task at `lib/mix/tasks/mnemosyne.dev.record_fixture.ex`:

```
mix mnemosyne.dev.record_fixture --output <path> [--prompt <prompt>]
                                                  [--profile research_broad|ingestion_minimal]
                                                  [--max-delay-ms <ms>]
                                                  [--interactive]
```

Implementation: spawns a real `Mnemosyne.HarnessAdapter.ClaudeCode` session against the user's `claude` binary, attaches itself as the session consumer, captures every `%HarnessOutput{}` event into a `%FixtureRecord.Output{}`, measures wall-clock gaps into `%FixtureRecord.Delay{ms: gap_ms}` (capped at `--max-delay-ms`, default 500 ms), terminates with the captured `%SessionExitStatus{}` as `%FixtureRecord.Exit{}`. Writes JSON Lines to `<path>`.

`--interactive` mode: mirrors live output to the user's terminal, accepts typed lines on stdin, forwards them to the live adapter via `send_user_message/2`, records both directions (output as `Output`, user inputs as `ExpectUserInput`). Recording terminates when the live session exits or the user types `:done`.

Recording is **the canonical way fixtures get into `test/fixtures/`**. Hand-editing is allowed but discouraged; recorded fixtures stay aligned with whatever the real `claude` binary actually emits, eliminating fixture-vs-reality drift.

---

## 6. Tool profile enforcement

### 6.1 Profile → CLI flag mapping

| `tool_profile` | `--disallowed-tools` flag | `--permission-mode` flag | Stream-side defence-in-depth |
|---|---|---|---|
| `:ingestion_minimal` | `--disallowed-tools "*"` (to be verified against pinned `claude`; §10 Q1) | `--permission-mode default` | Reject any non-injected `tool_use` block → `{:tool_profile_violation, _}` + terminate |
| `:research_broad` | (omit flag — all tools allowed) | `--permission-mode acceptEdits` | No rejection: all non-injected tools allowed by definition |

The injected Mnemosyne tools (`ask_expert`, `dispatch_to_plan`, `read_vault_catalog`) are **always** in the allowed set, regardless of profile — they are routing-layer primitives, not harness tools, and their presence is required for sub-F's in-session Queries.

```elixir
def tool_profile_to_args(:ingestion_minimal), do: ["--disallowed-tools", "*", "--permission-mode", "default"]
def tool_profile_to_args(:research_broad),    do: ["--permission-mode", "acceptEdits"]
```

### 6.2 Stream-side defence-in-depth check

In the GenServer's `handle_info({:stdout, _, line}, state)` clause, after parsing the line into an event, but **before** forwarding anything to consumers:

```elixir
defp check_tool_profile(%HarnessOutput{kind: :tool_use, text: name} = _event, state)
     when state.tool_profile == :ingestion_minimal do
  if MapSet.member?(state.injected_tools, name) do
    :ok      # Mnemosyne-injected tools are always allowed
  else
    {:error, {:tool_profile_violation, %{profile: :ingestion_minimal, tool_name: name}}}
  end
end
defp check_tool_profile(_event, _state), do: :ok
```

When this returns `{:error, _}`, the GenServer broadcasts `%SessionExitStatus{reason: violation}` to consumers and immediately casts `:terminate` to itself.

### 6.3 Why both layers (spawn-time flag + stream-side check)

Three failure modes the flag alone does not cover:

1. **Future Claude Code regression.** A new release could change the meaning of `--disallowed-tools "*"` (e.g., interpret it as "allow all" in a scoping change). The stream-side check catches the regression on first run.
2. **Profile mismatch bug.** If a future C code change mis-routes a profile, the stream-side check is the loudest canary.
3. **Adversarial prompt injection.** A malicious upstream knowledge entry could try to convince Claude to "use the Bash tool to help debug this" — even with the CLI flag, if Claude Code's own enforcement layer fails, the adapter catches it. Defence-in-depth against the precise scenario the tool profile is designed to prevent.

### 6.4 Future profile additions

The two-profile minimum set is v1's commitment. Likely v1.5 additions:

- `:research_read_only` — file/web read but no write/shell. Useful for research sessions that consult the web without risking edits.
- `:write_contained` — read + write but only inside the staging directory; no shell, no web. Useful for internal reasoning sessions that produce structured output files.

Both fit the existing behaviour as new atoms in the `tool_profile` type and new rows in the mapping table. No architectural changes required.

---

## 7. Cold-spawn budget and v1.5 warm-pool

### 7.1 v1 ships cold-spawn only

`ClaudeCode.spawn/1` calls `DynamicSupervisor.start_child/2`, which starts a fresh session GenServer, which calls `:exec.run/2` directly. No pool, no reuse, no session manager. Every session is a fresh `claude` process; every exit is reaped by the `{:DOWN, ...}` handler. Justified by:

- B's contract is satisfied either way.
- Cold-spawn cost is unmeasured on the user's primary dev machine until the dogfood acceptance test runs.
- Speculative pooling would add a session-manager GenServer, a reset-and-reuse mechanism, and a pool-depth controller — substantial surface for a problem not yet measured.
- The project's preference is hard-errors + minimal-deps + integration-over-reinvention, all of which favour the smallest possible v1 surface.

### 7.2 Latency instrumentation (always on)

`%Mnemosyne.Event.SpawnLatencyReport{}`:

```elixir
defmodule Mnemosyne.Event.SpawnLatencyReport do
  @enforce_keys [:session_id, :harness_kind, :tool_profile,
                 :spawned_at, :first_chunk_at, :init_event_at]
  defstruct [...]
end
```

| Metric | Derivation | What it measures |
|---|---|---|
| **Cold-spawn latency** | `init_event_at − spawned_at` | Time from `:exec.run/2` return to Claude Code's `system/init` line |
| **First-chunk latency** | `first_chunk_at − spawned_at` | Time to the first observable stdout chunk |
| **First-output latency** | `first_chunk_at − init_event_at` | Pure model-time, isolating Claude Code startup from inference time |

The report is emitted as a `%HarnessOutput{kind: :internal_message, meta: %{report: :spawn_latency, data: struct}}` event to consumers, as a `:telemetry.execute/3` call under `[:mnemosyne, :harness, :claude_code, :spawn_latency]`, **and** written to `<staging>/spawn-latency.json` alongside other staging artifacts. Always-on, no debug flag.

This instrumentation is a **tactical seed**, not a metrics framework. Sub-M (Observability) owns the broader story (see §11.4). The staged migration pattern applies: C's v1 keeps all three emission paths, sub-M's v1 attaches `:telemetry` histograms in parallel, mechanical verification confirms equivalence within tolerance, and a subsequent sub-M triage task deletes the `<staging>/spawn-latency.json` writer and the `:internal_message` delivery. The `%SpawnLatencyReport{}` struct itself survives as the canonical shape.

### 7.3 The C-1 dogfood acceptance gate

> **C-1 acceptance gate.** v1 of `Mnemosyne.HarnessAdapter.ClaudeCode` is accepted when the dogfood orchestrator plan completes a full work → reflect → compact → triage cycle, executed N≥10 times against the real adapter on the user's primary dev machine, and the cold-spawn latency distribution measured across all sessions in those cycles satisfies **p95 < 5 seconds**. If p95 ≥ 5 s, the warm-pool reset spike (§7.4) is unblocked and becomes a v1 task; if p95 < 5 s, the warm-pool work is deferred to v1.5 and the spike is recorded as a v1.5 candidate task only.

The 5-second threshold is higher than B's 3-second cold-spawn target by design: B's target is the goal under warm-pool reuse; C-1's gate is the threshold under which warm-pool isn't *worth building*. 5 s is the threshold where "phase startup feels sluggish" becomes likely enough that the optimisation pays for itself.

### 7.4 The warm-pool reset spike (conditional, ≈½ day)

Triggered if and only if C-1 trips. Independent of any Mnemosyne code changes — runs against the real `claude` binary in a tmpdir under an ExUnit harness.

#### 7.4.1 Three-check protocol

Executed in order; stops at the first check that succeeds.

**Check 1 — Structured stream-json reset envelope.** Inspect `claude --help` and a recorded session output for any control envelope along the lines of `{"type":"control","action":"reset"}` or `{"type":"clear"}`. If documented or observed, write a short test that spawns a `claude` process, sends a user message, sends the reset envelope, sends a second user message, and verifies the second message is processed in fresh context. **If this works, this is the chosen reset path.**

**Check 2 — `/clear` injected as user-message text.** If Check 1 turns up nothing, send `/clear` as the text content of a user-message envelope and verify the same fresh-context property. **If this works, this is the chosen reset path** — with a written caveat that it depends on Claude Code's slash-command interceptor running in stream-json input mode, which is not a documented contract.

**Check 3 — Pre-spawned single-use degradation.** If neither Check 1 nor Check 2 works, the warm-pool implementation falls back to "pre-spawn N processes per profile, hand each one out exactly once, replace asynchronously after handoff". Saves cold-spawn time on the *first* use of each pool entry but not on subsequent uses — meaningful but smaller savings than true reset-and-reuse.

#### 7.4.2 Spike output

A short markdown report at `test/fixtures/sub_c_warm_pool_spike/results/spike-report.md` containing which check passed, the exact reset envelope or text used, recorded session output verifying fresh-context behaviour, and a recommendation.

### 7.5 v1.5 warm-pool design sketch

If the gate trips and the spike returns "Check 1 or Check 2 works":

```elixir
defmodule Mnemosyne.HarnessAdapter.ClaudeCode.WarmPool do
  use GenServer
  # Per-profile pools maintained by a background-spawner helper task.
  # start_session/1 pulls from the available queue of the right profile,
  # sends a reset envelope (or /clear) via :exec.send/2 to clear state,
  # then sends the actual prompt as the next user-message envelope.
  # Empty pool → fall back to ClaudeCode.spawn/1 for a fresh process.
end
```

Pool depth per profile: `:ingestion_minimal` typically wants depth 3-4 (E's pipeline fires N≥3 ingestion sessions per cycle); `:research_broad` wants depth 1-2 (user-facing phases fire one at a time). Both configurable with sensible defaults.

**This entire section 7.5 is sketch-only.** None of it lands in v1 code. If the gate doesn't trip, it stays in the spec as a deferred-work record.

---

## 8. Testing strategy

Three test layers, each covering a distinct concern. All tests live under `test/mnemosyne/harness_adapter/`.

### 8.1 Layer 1 — Unit tests

Pure-function tests with no GenServer start, no erlexec, no fixtures.

| Test target | What it asserts |
|---|---|
| `ClaudeCode.StreamJson` | Each known event type parses from a canonical JSON sample (captured from the spike at `test/fixtures/harness_adapter/captured_stream_json/`) into the expected `%Mnemosyne.Event.*{}` struct; edge cases (empty content blocks, mixed text/tool_use blocks, missing optional fields) |
| `ClaudeCode.Spawn.tool_profile_to_args/1` | Each profile produces the documented argv |
| `ClaudeCode.Input.encode_user_message/1` | Round-trip against a captured user-message envelope from the spike |
| `FixtureReplay.Format` | Every `FixtureRecord` variant round-trips through Jason encode/decode and through `from_disk/1` / `to_disk/1` |
| `SessionExitStatus` / `HarnessOutput` / `SessionLifecycle` struct invariants | `@enforce_keys` catches missing fields at compile time; unit tests cover struct construction and pattern-matching |

Run in tens of milliseconds via `mix test`. No external deps.

### 8.2 Layer 2 — GenServer integration tests using `FixtureReplay`

Exercise the full session GenServer surface (start → consumer attach → stream of events → user input → terminate → exit-status) without spawning a real `claude` process. These are the tests that catch message-ordering bugs in consumers and in C's own GenServer.

| Test scenario | Fixture | Asserts |
|---|---|---|
| Clean linear session | `replay_clean_linear.jsonl` | Consumer receives N `%HarnessOutput{}` events in order, then `%SessionLifecycle{transition: {:exited, _}}`, then GenServer stops with `{:clean_exit, 0}` |
| Multi-turn with user interjection | `replay_multi_turn.jsonl` | Driver process calls `send_user_message/2` during an `expect_user_input` record; replay continues and emits post-interjection events |
| Mid-stream termination | `replay_terminated.jsonl` | Driver casts `:terminate` mid-stream; consumer receives an `{:exited, {:terminated, _}}` event; `await_exit/2` returns the terminated status |
| Tool-profile violation | `replay_tool_violation.jsonl` | Fixture emits a `tool_use` chunk under `:ingestion_minimal`; defence-in-depth fires; consumer receives `%SessionExitStatus{reason: {:tool_profile_violation, _}}` |
| Crashed-before-ready | `replay_crashed_before_ready.jsonl` | Fixture emits zero chunks then an exit within 100 ms; `await_exit/2` returns `{:clean_exit, _}` (the fixture replay does not emulate the real CrashedBeforeReady heuristic — that is live-adapter-only; this fixture tests the shape) |
| Multi-consumer attach | (programmatic, no fixture) | Two consumer pids attach; both receive the same events; one detaches mid-stream; only the remaining consumer receives subsequent events |

These use the production code paths: `FixtureReplay.Session` is the only non-subprocess exerciser of the GenServer-level consumer-attach flow, so message delivery and attach/detach correctness are exercised on every CI run.

### 8.3 Layer 3 — Integration tests against the real `claude` binary

Tagged `@moduletag :live`. Run with `mix test --only live`. Not run in normal CI; run locally during development and as part of the dogfood acceptance test.

| Test scenario | Asserts |
|---|---|
| Smoke test: spawn + first chunk | A simple `"say hi"` session spawns successfully and emits at least one `%HarnessOutput{kind: :stdout}` event |
| Tool profile enforcement (`:ingestion_minimal`) | A prompt that explicitly asks for `Read` tool use returns either zero `:tool_use` events or terminates with `{:tool_profile_violation, _}` |
| Tool profile enforcement (`:research_broad`) | A prompt that asks for file reading produces one or more `:tool_use` events |
| Multi-turn live | Spawn → first turn → `send_user_message/2` → second turn → assert second turn references first-turn content |
| Cold-spawn latency baseline | Run 10 spawn cycles, record `%SpawnLatencyReport{}`, assert p95 < 10 s (the *test gate*; the *acceptance gate* is 5 s and tighter) |
| Process group cleanup | Spawn, immediately `terminate/1`, assert `pgrep -f "claude.*<session-id>"` returns nothing within 1 second |
| `CrashedBeforeReady` against missing API key | Set an invalid API key in env, spawn, assert the consumer receives `%SessionExitStatus{reason: :crashed_before_ready}` within 5 s |
| cmux noise mitigation | Spawn with and without `--setting-sources project,local`; assert the with-flag run produces a clean NDJSON stream while the without-flag run contains the ~10 KB of SessionStart hook JSON |
| Tool-call boundary smoke | Inject the `read_vault_catalog` tool; prompt: "list the plans in my vault"; assert the session intercepts the `tool_use`, routes to a stub router, and `:exec.send`s the `tool_result` back; assert Claude Code consumes the result and continues |

Layer 3 is the canary that catches Claude Code version drift — when the stream-json schema changes or a flag gets renamed, Layer 3 fails first.

### 8.4 What is NOT in v1's test scope

- **Multi-adapter tests** — no multi-adapter surface in v1 (reserved for sub-O).
- **Performance tests beyond the 10-cycle latency baseline** — performance work is gated by C-1's acceptance gate.
- **Cross-node / distributed tests** — single-node Mnemosyne is the v1 model; distribution is sub-P's territory.
- **Fault-injection at the exec-port level** — v1 catches exec-port loss via the `{:EXIT, _, _}` handler but does not actively test it.

---

## 9. Risks

### Risk 1 — Claude Code stream-json schema drift across versions
*MEDIUM impact, MEDIUM likelihood.*

The stream-json schema is not a versioned contract. A future `claude` release could rename event fields or add new event types. C's parser would pattern-match-fall-through on the new shape and forward it as `:internal_message` with the original JSON in `meta` — no crash, but consumers that depended on typed interpretation of the new shape would miss it.

**Mitigation**: (1) a `claude --version` check at adapter init with a hard warning on mismatch; (2) Layer 3 integration tests fail fast on schema drift; (3) the "match the known shapes and forward anything else as `:internal_message`" parser design gives forward-compat for additive changes; (4) captured canonical JSON samples from the spike are the regression baseline.

### Risk 2 — Cold-spawn latency exceeds the 5 s p95 gate
*MEDIUM impact, MEDIUM likelihood.*

If the dogfood test trips C-1, warm-pool work moves into v1 mid-stream, expanding scope by the §7.4 spike plus ~1 day of GenServer wiring for reset-and-reuse. Schedule risk for the broader orchestrator merge.

**Mitigation**: (1) the spike is small and runs in parallel with other v1 work; (2) the v1.5 design is sketched in §7.5 so the implementation plan exists if needed; (3) the gate is published and measurable so the decision is uncontroversial when it fires.

### Risk 3 — Tool-call boundary injection mechanism is brittle
*MEDIUM impact, MEDIUM likelihood.*

§4.5 lists three candidate mechanisms for injecting Mnemosyne tools into a running Claude Code session; only the MCP-over-Unix-socket path (a) is strongly preferred, and it depends on Claude Code's MCP implementation accepting socket-based servers at the `--mcp-config` level. If (a) fails, (b) requires unspecified Claude Code behaviour, and (c) depends on the plugin layer. All three may shift version-to-version.

**Mitigation**: (1) day-1 spike against the pinned `claude` version validates the chosen mechanism end-to-end before the GenServer's tool-call boundary is wired; (2) the spike outcome is recorded as resolution to §10 Q6; (3) §1.3 goal 9 ("tool-call-boundary extensibility") keeps the intercept path abstract enough that changing the injection mechanism is a localised change.

### Risk 4 — `exec-port` loss mid-session
*LOW impact, LOW likelihood.*

`erlexec` runs a C++ port program (`exec-port`) as a separate OS process. If `exec-port` crashes or is killed by the OS while live sessions exist, every live session's erlexec state is lost and the GenServer receives an `{:EXIT, _, _}` message.

**Mitigation**: (1) the session GenServer traps exits and translates `exec-port` loss into `%SessionExitStatus{reason: {:exec_port_lost, _}}` so consumers observe a clean failure; (2) `PlanActor` owns re-spawn decisions at the plan layer — a live phase-cycle session that loses its harness simply restarts the current phase from staging, since plan state is durable on the filesystem; (3) sub-M dashboards track exec-port restart count as a health signal.

### Risk 5 (accepted) — Diagnostic dump buffer budget
*LOW impact, LOW likelihood.*

Every session maintains a ring buffer of recent events for `Mnemosyne.Observability.dump_event_tail/3`. At 1000 events per buffer per session and peak N≥10 concurrent sessions, the memory budget is bounded but non-trivial. v1 ships with a fixed 1000-event ring; the budget is revisited post-dogfood.

**Acceptance**: recorded as an accepted v1 limitation; sub-M's observability design owns the longer-term story.

---

## 10. Open implementation questions

Seven questions. Four resolved by the BEAM PTY spike; three remain as day-1 implementation tasks.

| # | Question | Resolution method | Status |
|---|---|---|---|
| 1 | Exact spelling of empty-tool-list / disallowed-tools value | Behavioural test against pinned `claude` — spawn under `:ingestion_minimal`, prompt for `Read`, verify denial | **Open — day 1** |
| 2 | Right `--permission-mode` for fully-headless operation | Behavioural test against tmpdir file edits under each candidate mode | **Open — day 1** |
| 3 | Whether `--print "<prompt>"` accepts the prompt arg alongside `--input-format stream-json` | — | **Resolved by spike**. It does not. Initial prompts ship as the first stdin NDJSON user-message envelope. |
| 4 | Exact stream-json field names | — | **Resolved by spike**. Canonical samples at `spikes/beam_pty/results/full-run.log`; copy to `test/fixtures/harness_adapter/captured_stream_json/` and lock the parser against them on day 1. |
| 5 | Warm-pool reset mechanism | Three-check spike protocol (§7.4) | **Conditional** — only fires if C-1 trips |
| 6 | Tool-call boundary injection mechanism (MCP Unix socket vs stdin preamble vs plugin shim) | Focused spike against the pinned `claude` version at the start of implementation phase, before the GenServer's intercept path is wired | **Open — day 1 for v1 blocker** |
| 7 | `exec-port` supervision expectations | Documented mitigation (§9 Risk 4): sessions translate loss to `{:exec_port_lost, _}`, PlanActor re-starts phases from staging | **Resolved as a design decision** |

---

## 11. Cross-sub-project requirements

### 11.1 Back to Sub-B — one typed event + one executor requirement

Sub-B consumes the session GenServer via `attach_consumer/2` and `handle_info/2`. The BEAM shift eliminates amendments 1-3 from the original Session 6 brainstorm (they addressed Rust trait-object plumbing — `&mut self` → `&self`, `Box<dyn>` → `Arc<dyn>`, `Send + Sync` — which have no BEAM analogue). Two substantive requirements survive:

1. **`Mnemosyne.Event.SessionLifecycle` is a consumed typed event.** B's phase-cycle machine pattern-matches on `%SessionLifecycle{transition: :ready}`, `%SessionLifecycle{transition: {:turn_complete, _}}`, and `%SessionLifecycle{transition: {:exited, _}}` as the protocol-level signals. These are **distinct from task-level completion signals** (see requirement 2 and §4.3.2).

2. **B's executor runs a sliding-buffer sentinel matcher on every `%HarnessOutput{kind: :stdout}` event** to detect task-level "the LLM has decided the work is done" signals. Sentinels are per-phase, defined in the phase prompt files (`phases/work.md`, `phases/reflect.md`, `phases/compact.md`, `phases/triage.md`) alongside the prompt instruction telling the LLM to emit them. When the matcher fires, B's executor emits an internal `{:phase_completion_detected, sentinel}` message that the phase state machine consumes as the trigger to wind down the session and transition phases.

   **Sliding-buffer matching is required.** A naive `String.contains?(chunk.text, sentinel)` check is unsafe: Claude Code's stream-json emits assistant text in content blocks that *can* split a multi-token sentinel across two chunks. The matcher must accumulate the last N characters of stdout text into a rolling buffer (sized to ≥ longest expected sentinel + slack, e.g., 256 bytes) and match against the buffer. The BEAM PTY spike validated this matcher against single-chunk, two-chunk split, grapheme-drip, false-prefix, false-overlap, and bounded-window cases — 6 unit tests green.

   **Why this lives in B and not C:**
   - **Coupling**: sentinels are coupled to phase prompts, which B owns. C should not know about phase semantics.
   - **Harness-agnosticism**: the sentinel mechanism works against any harness that produces text output, including future bare-LLM / Codex / Pi adapters that may not expose turn-boundary events. Putting detection in B's executor means future adapters get task-level completion detection for free.
   - **Layering**: C is the wire layer (process spawn, stream parse, lifecycle, tool-call boundary); B is the policy layer (phase semantics, prompt content, completion conditions). Sentinel detection is policy.
   - **Per-phase variation**: different phases use different sentinel strings.

   **Why this is consistent with "no callback channel":** the harness is not calling Mnemosyne. The harness emits a documented string in its normal output as part of fulfilling its prompt. Mnemosyne reads its own output stream and detects the string. The flow is harness → output → Mnemosyne reads — structurally identical to every other piece of stream observation.

### 11.2 To Sub-E — already covered

- E's Stage 3/4 reasoning sessions spawn with `tool_profile: :ingestion_minimal`. Already specified in E's design doc.
- E's Stage 5 (now "dispatch to experts" post-F amendment) is implemented as a Query via the tool-call boundary (§4.5), not as a direct adapter call.

### 11.3 To Sub-F — tool-call boundary contract

F's in-session Queries require C to expose §4.5's tool-call boundary: inject a whitelisted tool set, intercept matching `tool_use` events, route them through F's `Mnemosyne.Router`, and reply with `tool_result` envelopes. F's sibling plan scaffolding (orchestrator Priority 3.1) is unblocked by this design landing.

F also owns the `Mnemosyne.Router.handle_tool_call/4` function C calls from §4.5.3; the contract is a `{:router_reply, tool_use_id, result}` info message back to the session pid, where `result` is a JSON-encodable term.

### 11.4 To Sub-M — typed events + telemetry

Sub-M owns `:telemetry` + typed `Mnemosyne.Event.*` structs as the project-wide observability pattern. C consumes the pattern by:

- Defining its event structs under `Mnemosyne.Event.*` per §3.3.
- Emitting `:telemetry.execute/3` at every boundary: spawn, first chunk, init event, turn boundary, tool-use, tool-result, terminate, exit.
- Feeding the event-tail ring buffer maintained by `Mnemosyne.Observability` and calling `dump_event_tail/3` on every error path.

C's `%SpawnLatencyReport{}` remains a tactical seed subject to sub-M's parallel-emit migration: C keeps `<staging>/spawn-latency.json` and `:internal_message` delivery during the verification window, sub-M lands `:telemetry` histograms in parallel, mechanical verification confirms equivalence within tolerance, and a subsequent sub-M triage task deletes C's auxiliary emission paths. The `%SpawnLatencyReport{}` struct survives as the canonical shape.

### 11.5 Reserved for Sub-O — multi-adapter surface

V1 ships a single adapter (Claude Code) with a single implementation of the `Mnemosyne.HarnessAdapter` behaviour. Sub-O owns multi-adapter support, per-actor model selection, local-model adapters (Ollama, llama.cpp), and cost telemetry. F's daemon config reserves `[harnesses.*]` for per-adapter configuration.

The behaviour (§3.1) is written to avoid Claude-Code-specific leak: callback signatures use neutral terms (`session_id`, `prompt`, `tool_profile`, `working_dir`). Claude-Code-specific parsing lives behind the behaviour in `Mnemosyne.HarnessAdapter.ClaudeCode.*`. Sub-O extends without restructuring.

### 11.6 To Sub-G — migration note

The existing `<project>/adapters/claude-code/` directory is the **legacy v0.1.0 Claude Code plugin** (markdown skills consumed by Claude Code itself), not an adapter implementation. C does not touch it; G's migration plan owns retiring or renaming it. No Elixir code in C depends on its presence or absence.

---

## Appendix A — Decision Trail

This appendix records the major decisions reached during C's design, including alternatives considered and rationale. Recorded so future maintainers can see the reasoning, not just the conclusions. Q1-Q5 are from the original Session 6 brainstorm; Q6-Q8 are from the sub-F pivot and the BEAM PTY spike.

### Q1 — Process model: how the v1 Claude Code adapter talks to the `claude` binary

**Options considered:**

- **A — Headless `--output-format stream-json` (one-shot).** Simple, tiny dep tree, perfect match for B's chunk model, but no mid-session user interaction.
- **B — PTY-wrapped interactive session.** Full interactivity, but requires terminal escape parsing, brittle against TUI version drift, much more complex.
- **C — Bidirectional `--input-format stream-json` + `--output-format stream-json`.** Multi-turn interactive over a structured JSON-Lines channel on stdin/stdout. No PTY, no terminal escape parsing, supports user interjection mid-session.

**Chosen: Option C.** The user's concern about live interaction ruled out Option A; Option B's complexity and brittleness made it the wrong way to satisfy that concern. Option C achieves bidirectional interactivity using Claude Code's own structured I/O.

**BEAM PTY spike correction (Session 10, 2026-04-15).** Option C was originally framed as "no PTY required" but the spike discovered the stronger claim: a PTY actively *breaks* the input path when combined with `erlexec`'s `:stdin` option. `:pty + :stdin` does not wire the caller's pipe to the child's real stdin, and the child errors with `Input must be provided either through stdin or as a prompt argument when using --print`. Pipes-only erlexec (`[:monitor, :stdin, {:stdout, self()}, {:stderr, self()}, :kill_group]`) is the mandatory shape for v1.

### Q2 — Where the adapter code lives in the source tree

**Options considered:**

- **A — Top-level modules under `lib/mnemosyne/harness_adapter/`.** Slots into the existing daemon application layout.
- **B — Separate `:harness_adapter` umbrella app.** Dep isolation, faster incremental builds.

**Chosen: Option A.** Umbrella conversion would create cross-cutting friction for every other in-flight sub-project. The dep-isolation benefit is near-zero because the adapter only needs `erlexec` + the daemon's pre-existing `jason`. Module structure matches existing `lib/mnemosyne/*/` directories and keeps refactors local.

(The original Session 6 brainstorm was framed against a Rust workspace layout; Option B as first written was "Rust workspace crate". The BEAM pivot re-cast the question but did not change the answer: same rationale, same choice.)

### Q3 — Warm-pool reuse in v1, or punt to v1.5/v2

**Options considered:**

- **A — Punt to v1.5, ship cold-spawn-only in v1.** Simplest possible adapter; latency measured during dogfood; warm-pool work triggered only if measurements show real pain.
- **B — Build a minimal pool in v1.** Hides cold-spawn latency on the hot path but adds significant complexity for a problem not yet measured.
- **C — Build a pool only for `:ingestion_minimal` profile sessions.** Targets E's pipeline cost concentration.

**Chosen: Option A, with an amendment.** The amendment came from observing that `/clear` (or a structured equivalent) could potentially reset a pre-existing pool entry, which would change the v1.5 warm-pool strategy from "spawn fresh processes" to "reset and reuse existing processes" — a much more efficient model. The v1.5 task description therefore specifies the precursor spike (§7.4) that validates either a structured stream-json reset envelope (preferred) or `/clear`-as-user-message-text (fallback) before committing to an implementation strategy.

### Q4 — Fixture format

Resolved with no significant pushback. The JSON-Lines tagged format (`output` / `delay` / `expect_user_input` / `exit` records) was accepted as proposed. Per-sub-project fixture file ownership under `test/fixtures/<owning-sub>/harness_adapter/` was accepted. The `mix mnemosyne.dev.record_fixture` task as the canonical fixture creation path was accepted.

### Q5 — Tool profile → CLI flag mapping

Resolved with the two-profile mapping accepted as proposed. Defence-in-depth stream-side enforcement was accepted as the second-layer safeguard. The user accepted that adding more profiles is a v1.5 additive change and v1 ships with the minimum two.

### Q6 — BEAM pivot (Session 9, 2026-04-14 via sub-F brainstorm)

Sub-F committed Mnemosyne to a persistent BEAM daemon. C was originally designed against Rust + `crossbeam-channel` + `nix`; the pivot re-cast the implementation on Elixir/OTP + `erlexec`. The *design intent* survived unchanged: single-owner-per-state, typed messages, defence-in-depth tool profiles, process-group termination from v1, fixture replay parity, cold-spawn gate, hard errors. The *runtime substrate* moved from hand-rolled actor threading to OTP GenServers + DynamicSupervisor.

The re-cast was done inline across §1-§11 rather than as a supersede-amendment layer. Every Rust idiom that appeared in the Session 6 brainstorm was translated to its BEAM equivalent:

| Session 6 (Rust) | Current (BEAM) |
|---|---|
| `src/harness/` module tree | `lib/mnemosyne/harness_adapter/` module tree |
| `HarnessAdapter` / `HarnessSession` traits with `Send + Sync` | `@behaviour Mnemosyne.HarnessAdapter` + session GenServer contract |
| Three threads per session (actor + stdout-reader + stderr-reader) | One GenServer per session; erlexec messages arrive directly in its mailbox |
| `crossbeam_channel::select!` inbox dispatch | `handle_info/2` pattern match on erlexec and client messages |
| `std::process::Command` + `process_group(0)` + `nix::killpg` | `:exec.run/2` with `[:monitor, :stdin, {:stdout, self()}, {:stderr, self()}, :kill_group]` + `:exec.kill/2` |
| `tracing::instrument` | `:telemetry.span/3` |
| `which::which/1` | `:exec.which/1` |

### Q7 — BEAM PTY spike (Session 10, 2026-04-15)

A focused spike at `spikes/beam_pty/` validated the erlexec pipes-only path against the real `claude` binary. 8 tests green (6 sentinel unit tests + 2 live probes). Three material findings:

1. **PTY inverted.** The spike started from the assumption that a PTY would be needed for `claude`'s interactive I/O and discovered that stream-json is pure NDJSON over stdio and the `:pty + :stdin` combination in erlexec actively breaks the input path. v1 ships pipes-only.
2. **cmux noise must be suppressed.** User-global cmux SessionStart hooks emit ~10 KB of JSON before the first assistant event on every `claude` invocation. `--setting-sources project,local --no-session-persistence` silences them; both flags are mandatory on all daemon-spawned sessions.
3. **Two completion signals are orthogonal.** `{"type":"result"}` is Claude Code's protocol-level "turn over" signal; the phase-prompt sentinel is the task-level "I am done with the work" signal. Conflating them causes premature phase transitions. Surfaced as the `%SessionLifecycle{transition: {:turn_complete, _}}` event and the B-owned sentinel matcher respectively.

### Q8 — Tool-call boundary (Session 9 via sub-F brainstorm)

F's `Query` and `Dispatch` message types require the adapter to expose a hook for intercepting a whitelisted set of `tool_use` events and routing them through the daemon router. The hook is §4.5's tool-call boundary. Three candidate injection mechanisms exist (§4.5.2); the MCP-over-Unix-socket path is strongly preferred and gets a day-1 spike to validate before the GenServer's intercept path is wired.

### Post-write user clarification — observation channel and sentinel-driven completion

After the spec was first written and committed, the user pushed back on the framing of the "no callback channel from harness to Mnemosyne" rule with two distinct points:

1. **Disambiguation between control and observation.** The rule forbids *control* flowing from harness to Mnemosyne (slash commands, programmatic callbacks, the LLM invoking Mnemosyne actions via tool use), not *observation* of harness state by Mnemosyne. Mnemosyne reading the harness's structured output and reacting on its own side is exactly what the bidirectional stream-json design enables. The previous phrasing collapsed the two concepts and was sloppy. Resolved by adding the `SessionLifecycle` typed event (§3.3, §4.3.2) and by updating the stream-parser table to expose `system/init`, `result`, and `{:DOWN, ...}` as structured `%SessionLifecycle{}` events rather than burying them in `:internal_message` text or in GenServer-internal state.

2. **Task-level vs protocol-level completion semantics.** "When the LLM has finished" should be detected via a prompt-instructed sentinel string that Mnemosyne watches for. This is better than relying on Claude Code's `result` event because: (a) `result` is a *protocol-level* signal that fires whenever the model stops emitting tokens, which can happen mid-task; the sentinel is a *task-level* signal driven by the LLM's own judgment; (b) the sentinel mechanism is harness-agnostic; (c) sentinel strings are coupled to phase prompts, so they belong with B rather than with C. Resolved by recording sentinel detection as B's executor requirement (§11.1 requirement 2), with sliding-buffer matching for chunk-boundary edge cases, validated by the BEAM PTY spike.

---

## Appendix B — Dependency footprint (`mix.exs`)

```elixir
defp deps do
  [
    {:erlexec,   "~> 2.2"},       # NEW (sub-C) — C++ port program for OS child processes
    {:jason,     "~> 1.4"},       # pre-existing — NDJSON encode/decode
    {:telemetry, "~> 1.2"},       # pre-existing — sub-M observability transport
    # ... other daemon deps (GenServer/DynamicSupervisor/Process/Registry are stdlib OTP)
  ]
end
```

One new Hex dep. `erlexec`'s own transitive deps are minimal (`rebar3` for build, no runtime libraries beyond ERTS itself). The `exec-port` binary is compiled at dep fetch time.

`erlexec` is also added to `extra_applications` so the `exec` application starts with the daemon:

```elixir
def application do
  [
    extra_applications: [:logger, :exec],
    mod: {Mnemosyne.Application, []}
  ]
end
```

No PTY library, no custom NIF, no async runtime beyond BEAM.

---

## Appendix C — Glossary

| Term | Definition |
|---|---|
| **C-1 gate** | The dogfood acceptance criterion that gates whether warm-pool work happens in v1 or is deferred to v1.5. Defined in §7.3: p95 cold-spawn latency < 5 s across N≥10 dogfood cycles. |
| **Cold-spawn latency** | Time from `:exec.run/2` returning to Claude Code's `system/init` NDJSON line being observed. |
| **Defence-in-depth tool enforcement** | The two-layer tool profile enforcement strategy: spawn-time CLI flags + stream-side `handle_info/2` check. |
| **`erlexec`** | Hex dep providing a C++ port program (`exec-port`) that spawns OS child processes with stdin wiring, process-group termination, and bidirectional pipe I/O. Not a NIF. |
| **`exec-port`** | The C++ port program that `erlexec` starts as a separate OS process. Runs outside BEAM schedulers; handles fork/exec, signals, and process groups. |
| **Injected tools** | The whitelisted set of Mnemosyne-owned tools (`ask_expert`, `dispatch_to_plan`, `read_vault_catalog`) exposed to the running Claude Code session via §4.5's tool-call boundary. |
| **`Mnemosyne.HarnessAdapter`** | The Elixir `@behaviour` that abstracts an LLM coding harness. Implementors: `ClaudeCode`, `FixtureReplay`. |
| **Session GenServer** | The `GenServer` started by an adapter's `spawn/1` that owns all mutable state for one live harness session. One per session. |
| **`SessionLifecycle` event** | A `%Mnemosyne.Event.SessionLifecycle{}` struct surfacing protocol-level harness state transitions: `:ready`, `{:turn_complete, subtype}`, `{:exited, reason}`. Distinct from task-level completion (owned by B's sentinel matcher). |
| **Stream-json** | Claude Code's NDJSON protocol over stdin/stdout, enabled by `--input-format stream-json --output-format stream-json`. |
| **Task-level completion** | The LLM's own judgment that the assigned work is done, signalled via a prompt-instructed sentinel string matched by B's executor. Distinct from protocol-level turn boundary. |
| **Tool-call boundary** | §4.5's mechanism for intercepting `tool_use` events for injected tools and routing them through F's daemon router. Not a harness-to-Mnemosyne control channel. |
| **Tool profile** | A bundle of tool-access permissions applied to a session at spawn time. v1 profiles: `:ingestion_minimal`, `:research_broad`. |
| **Warm-pool** | A pool of pre-spawned `claude` processes maintained by the adapter to amortise cold-spawn latency. v1.5+ work, conditional on the C-1 gate. |
