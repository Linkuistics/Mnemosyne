# Backlog — Mnemosyne Orchestrator

Tasks for merging LLM_CONTEXT functionality into Mnemosyne and building the persistent actor-daemon architecture committed in Session 9 (2026-04-14).

## Recent history

**Session 9 (2026-04-14) — Sub-project F brainstormed; architecture pivoted to BEAM daemon.** F's brainstorm expanded from "plan hierarchy + root plan" to the full v1 architecture: persistent BEAM daemon, two sealed actor types (PlanActor + ExpertActor), two message types (Dispatch + Query), `project-root` convention, path-based qualified IDs, dispatch asymmetry (same-project direct vs cross-project Level 2 agent), vault catalog, declarative routing with LLM-fallback learning loop, Elixir/OTP runtime, Rust TUI as separate client binary. Sunk-cost analysis showed the v0.1.0 Rust code was less than a day of effort; the BEAM pivot was the right call at the right time. Three new sub-projects surfaced: **N (domain experts)**, **O (mixture of models, v1.5+)**, **P (team mode, v2+)**. Design doc at `docs/superpowers/specs/2026-04-14-sub-F-hierarchy-design.md`. Documentation overhaul landed in the same session: new README, `docs/architecture.md`, rewritten `user-guide.md` and `configuration.md`. F's sibling plan scaffolding is **deferred until the BEAM PTY spike validates sub-C's approach** — this is the first critical implementation task.

**Session 8 (2026-04-14) — LLM_CONTEXT 2026-04 overhaul reconciliation.** Upstream LLM_CONTEXT shifted to a four-phase cycle (work → reflect → compact → triage), phase-file-factored composition, `memory-style.md` as single source of truth, `pre-work.sh` opt-in hook, `{{RELATED_PLANS}}` placeholder (now superseded by F's `{{VAULT_CATALOG}}`), `related-plans.md` schema (now deleted by F), and renamed driver/spec files. Session 8 pulled drift into all generated plans and fixed orchestrator references. No code changed.

**Session 7 (2026-04-13) — Sub-projects A and M brainstormed.** A committed explicit vault discovery, `mnemosyne.toml` marker, init/adopt commands, deletion of v0.1.0 hardcoded paths. M committed hybrid `tracing` + `MnemosyneEvent` architecture (now re-cast to `:telemetry` + typed Elixir struct events post-F).

**Session 5 (2026-04-13) — Obsidian symlink validation spike PASSED 6/6** on macOS and Linux via guivision + OCR evidence. Hard-copy-staging fallback not needed. Symlink approach stands.

**Sub-projects E (2026-04-12), B (2026-04-12), C (2026-04-13)** brainstormed in earlier sessions. Design docs live under `docs/superpowers/specs/`. All three need amendment tasks post-F.

## Priority 0 — Unblock F implementation

### BEAM PTY spike — validate sub-C approach `[spike]`
- **Status:** done (Session 10, 2026-04-15)
- **Dependencies:** F brainstorm complete (done)
- **Owner:** sub-C amendment
- **Description:** Validate that `erlexec` (or similar Elixir/Erlang PTY library) can cleanly spawn Claude Code with:
  - Bidirectional PTY I/O for stream-json protocol
  - Sentinel string detection on assistant-text stream
  - Process-group termination (SIGTERM → SIGKILL with 500ms grace)
  - Configurable tool profiles per spawn
  - Backpressure-friendly streaming output

  This is the one real ecosystem unknown after the BEAM commitment. A few hours of spike work will answer it. If `erlexec` works cleanly, sub-C's amendment task is straightforward and F's sibling plan scaffolding can proceed. If `erlexec` fails, the fallback is a small Rust PTY-wrapper binary invoked from Elixir as an Erlang Port.
- **Results:** **PASS**, with one important inversion. Spike at `spikes/beam_pty/` using Elixir 1.19.5 + Erlang/OTP 28 + erlexec 2.2.3. 8/8 tests green (6 sentinel unit + 2 live probes against the real `claude` CLI on haiku). Evidence at `spikes/beam_pty/results/full-run.log`.
  - **Inversion**: the "PTY" premise was wrong. `claude -p --input-format stream-json --output-format stream-json` is pure NDJSON over stdio — no pseudo-terminal required. Worse, erlexec's `:pty + :stdin` combination does NOT wire the caller's pipe to the child's real stdin: claude reads nothing and errors with `Input must be provided either through stdin or as a prompt argument when using --print`. Pipes-only (`[:monitor, :stdin, {:stdout, self()}, {:stderr, self()}, :kill_group]`) works perfectly — `:exec.send/2` delivers NDJSON to the child, output arrives as `{:stdout, ospid, binary}` messages for each line (`system/init`, `rate_limit_event`, `assistant/thinking`, `assistant/text`, `result/success`), `{:DOWN, ospid, :process, pid, reason}` fires on exit.
  - **Sentinel detection**: sliding-buffer matcher with window bounded to `sentinel_size - 1` bytes validated against single-chunk, two-chunk split, grapheme-by-grapheme drip, false-prefix, and false-overlap cases. Successfully detects claude's assistant text when instructed to emit the sentinel.
  - **Process-group termination**: `:kill_group` option + `:exec.kill(ospid, 15)` followed by a 500ms grace window and SIGKILL fallback kills the grandchild of `/bin/sh -c "sleep 60 & wait"`. Verified with `kill -0` liveness check.
  - **Tool profiles**: `--disallowed-tools` passes through at CLI flag level. Visible in the `system/init` event's `tools` array. No adapter work needed.
  - **Backpressure**: BEAM mailboxes are unbounded; draining via `receive` that processes each chunk before accepting the next is sufficient. No dropped data observed.
  - **Known noise**: user-global `settings.json` cmux SessionStart hooks emit ~10KB of hook JSON on every claude invocation. Silenced with `--setting-sources project,local --no-session-persistence`.
  - **Takeaway for sub-C amendment**: drop "PTY" from the stream-json path entirely. Wrap one GenServer per live session, parse NDJSON lines, dispatch to sub-M telemetry boundary, route tool-use events to in-session query handler. Detect `{"type":"result"}` as the protocol-level "turn over" signal, orthogonal to the phase-prompt sentinel (task-level "done"). Full recommendations in `spikes/beam_pty/README.md`.
  - **Unblocks**: P1.3 (sub-C amendment), P3.1 (sub-F sibling plan scaffolding), and by extension the entire sub-C + sub-F implementation runway.

  **Exit criteria**: either a working Elixir script that spawns Claude Code, runs a trivial prompt, captures output until a sentinel, and terminates the process cleanly — or a documented reason why the approach won't work, with a fallback plan.

  **Output:** short spike report at `docs/superpowers/specs/2026-MM-DD-beam-pty-spike.md` and a decision: proceed with `erlexec` or implement the Rust PTY-wrapper fallback.

## Priority 1 — F-triggered amendments to done brainstorms

F's architecture commitment affects every done brainstorm. Each amendment is a short work phase: read the existing design doc, identify Rust-specific or pre-F-specific sections, replace with BEAM/actor/dispatch-aware equivalents, commit.

**Execution note (triage 2026-04-15, updated):** All nine P1 tasks are now unblocked and can run in parallel. The BEAM PTY spike (P0) completed in Session 10 — pipes-only erlexec validated, PTY premise inverted — unblocking Sub-C amendment and by extension P3.1 (sub-F scaffolding). **Critical path:** Sub-C amendment (absorb spike results into C's design doc) and P3.1 (scaffold sub-F sibling plan) are the highest-leverage next tasks. Sub-G, Sub-H, and Sub-I amendments produce scope-framing documents for future brainstorms of those sub-projects (no existing design doc to amend — these capture F's impact on the brainstorm scope). See Priority 2 for the corresponding G/H/I brainstorm tasks.

### Sub-A amendment — daemon caller integration `[amendment]`
- **Status:** not_started
- **Dependencies:** F done
- **Description:** Amend sub-A's design doc to reflect:
  - `verify_vault` is called by the Elixir daemon at startup, not by per-invocation Rust CLI
  - Config discovery chain unchanged (`--vault` → `MNEMOSYNE_VAULT` → user config → hard error)
  - `mnemosyne.toml` schema unchanged; identity marker works identically
  - `init`, `init --from`, `adopt-project` commands become daemon subcommands (`mnemosyne daemon --init`, etc.)
  - v0.1.0 path deletion tasks (A tasks 11-12) no longer apply to Rust code; instead, the entire Rust CLI is retired in favor of daemon + TUI split
  
  Small amendment — mostly re-framing, not redesigning.
- **Results:** _pending_

### Sub-B amendment — actor-hosted phase cycle and schema pruning `[amendment]`
- **Status:** not_started
- **Dependencies:** F done
- **Description:** Amend sub-B's design doc to reflect:
  - `PhaseRunner` runs **inside a PlanActor GenServer**, not as a standalone process main loop
  - Phase transitions driven by `{:run_phase, _}` messages from attached clients
  - `plan-state.md` frontmatter schema pruning: remove `plan-id`, `host-project`, `dev-root` (all derivable from filesystem path). Add `description:` (required, ≤120 chars, non-placeholder)
  - `{{RELATED_PLANS}}` placeholder renamed to `{{VAULT_CATALOG}}` with new substitution content (full vault catalog with dispatch rules)
  - `related-plans.md` file concept deleted entirely
  - F's `DispatchProcessor` and `QueryProcessor` run as phase-exit hooks
  - `StagingDirectory::render` descent invariant unchanged
  - `ManualEditorExecutor` still exists as the human-driver path
  - TUI framing changes: TUI is a daemon client attached to a PlanActor, not a process main loop

  Moderate amendment — the phase-cycle mechanics survive but their process-architecture context changes.
- **Results:** _pending_

### Sub-C amendment — Elixir implementation and multi-adapter reservation `[amendment]`
- **Status:** done (Session 11, 2026-04-15)
- **Dependencies:** BEAM PTY spike complete (**done**, Session 10)
- **Description:** Amend sub-C's design doc to reflect:
  - Implementation in Elixir using pipes-only `erlexec` (spike validated; PTY premise inverted — stream-json is pure NDJSON over stdio, no pseudo-terminal needed)
  - erlexec opts: `[:monitor, :stdin, {:stdout, self()}, {:stderr, self()}, :kill_group, {:kill_timeout, 1}]`; `:stdin` bare atom required
  - Actor-style threading replaced by OTP GenServer supervision (one GenServer per session)
  - `HarnessSpawner` as an Elixir behavior instead of a Rust trait
  - Tool-call boundary for in-session Queries: C's adapter intercepts `ask_expert` and similar tool calls, routes them through F's router, delivers responses back as tool-call results
  - Multi-adapter support **reserved for sub-O** via the `[harnesses.*]` config section — v1 only implements Claude Code
  - Internal session spawning is how plan actors and F's Level 2 routing agent reason
  - `SpawnLatencyReport` emits via sub-M's `:telemetry` + typed struct events
  - Detect `{"type":"result"}` as protocol-level "turn over" signal (orthogonal to task-level sentinel)
  - cmux noise mitigation: `--setting-sources project,local --no-session-persistence` on all daemon-spawned sessions

  Significant amendment — most of C's core design survives but the implementation language and actor-style threading are completely re-cast. Spike results at `spikes/beam_pty/`.
- **Results:** Rewritten inline across §1–§11 of `docs/superpowers/specs/2026-04-13-sub-C-adapters-design.md`. The original Rust framing is replaced with fresh Elixir/OTP/erlexec content; Session 6 decisions are preserved as Q1–Q5 in Appendix A with explicit post-spike corrections, and new Q6–Q8 record the BEAM pivot, the BEAM PTY spike, and the tool-call boundary introduction. **No supersede-amendment layer; no stale Rust content under a disclaimer.** Doc is 1186 lines (down from 1311 original).
  - **§1 Scope** — fresh Elixir-native in/out of scope, new goal #9 ("tool-call-boundary extensibility"), non-goals now include "no PTY" as explicit with spike rationale, no `tokio`/async discussion, multi-node reserved to sub-P.
  - **§2 Architecture** — Mermaid diagram rewritten with GenServer + DynamicSupervisor + erlexec mailbox flow; module layout now `lib/mnemosyne/harness_adapter/`; supervision tree placement under `Mnemosyne.Supervisor` with `restart: :temporary` on session GenServers; dependency footprint at one new Hex dep (`erlexec`).
  - **§3 Adapter behaviour and typed events** — `@behaviour Mnemosyne.HarnessAdapter` with a single `spawn/1` callback plus `kind/0`; session GenServer contract as documented message shapes (`send_user_message`, `attach_consumer`, `await_exit`); `Mnemosyne.Event.*` sealed struct set (`HarnessOutput`, `SessionLifecycle`, `SpawnLatencyReport`, `SessionExitStatus`, `HarnessError`); contract #8 covers tool-call boundary explicitly ("injected tools are not a control channel").
  - **§4 ClaudeCode adapter** — complete rewrite. §4.1 pipes-only spawn with the exact erlexec opts and cmux mitigation flags as mandatory. §4.2 session GenServer state + message set with every `handle_info/2` / `handle_call/3` case documented. §4.3 stream-json parser locked against the spike's canonical event set. §4.3.2 protocol-level vs task-level completion semantics preserved. §4.4 two-phase SIGTERM→SIGKILL via `:exec.kill/2` + `Process.send_after/3`. **§4.5 tool-call boundary** — new substantive section: injected tool set (`ask_expert`, `dispatch_to_plan`, `read_vault_catalog`), three injection-mechanism candidates (MCP-over-Unix-socket preferred, stdin preamble fallback, plugin shim fallback), intercept flow via `Mnemosyne.Router.handle_tool_call/4` + `{:router_reply, _, _}`, why-not-control-channel argument. §4.6 error-reason table re-cast to `%SessionExitStatus{reason: _}` variants.
  - **§5 FixtureReplay adapter** — fresh GenServer implementation walking a JSON-Lines record list via `Process.send_after/3`; same client API as the live adapter; JSON-Lines format survives verbatim with a `Mnemosyne.Event.*` struct projection; `mix mnemosyne.dev.record_fixture` as the canonical dev task.
  - **§6 Tool profile enforcement** — Elixir pattern-match for `tool_profile_to_args/1`; `handle_info/2`-based defence-in-depth check; injected Mnemosyne tools always allowed regardless of profile.
  - **§7 Cold-spawn + warm-pool** — C-1 gate (p95 < 5 s, N≥10 cycles) preserved; latency instrumentation now emits as a `%SpawnLatencyReport{}` struct + `:telemetry.execute/3` call + staging JSON file (three-way parallel emission for sub-M's staged migration); v1.5 warm-pool sketch re-cast to a GenServer skeleton.
  - **§8 Testing** — ExUnit tags (`@moduletag :live`), three-layer strategy preserved; Layer 1 = parser/encoder/struct unit tests; Layer 2 = FixtureReplay-backed GenServer integration with multi-consumer attach coverage; Layer 3 = tagged live tests including tool-call boundary smoke and cmux noise mitigation assertions.
  - **§9 Risks** — rewritten set: schema drift (parser forward-compat), cold-spawn gate trip, tool-call-boundary injection brittleness (new), exec-port loss mid-session (new, with PlanActor re-spawn mitigation), diagnostic dump-buffer budget (accepted).
  - **§10 Open questions** — seven-question table. Q3/Q4 marked **resolved by spike**; Q1/Q2/Q5 carry forward as day-1 tasks; Q6 (tool-call boundary injection mechanism) newly open with a day-1 focused spike; Q7 (exec-port supervision) resolved as a design decision.
  - **§11 Cross-sub-project requirements** — re-cast. B gets one consumed typed event (`%SessionLifecycle{}`) + one executor requirement (sliding-buffer sentinel matcher over `%HarnessOutput{kind: :stdout}`). Rust-specific amendments 1-3 dropped (no BEAM analogue). New §11.3 spells out the F contract (tool-call boundary + `Mnemosyne.Router.handle_tool_call/4`). §11.4 is the telemetry + typed-struct pattern contract to sub-M. §11.5 reserves multi-adapter surface for sub-O with the "no Claude-Code-specific leak in the behaviour" discipline. §11.6 migration note for sub-G.
  - **Appendix A** — Decision Trail preserved with Q1-Q5 updated where the spike corrected a premise (Q1 "no PTY required" → "no PTY possible") and new Q6 (BEAM pivot), Q7 (BEAM PTY spike), Q8 (tool-call boundary) entries recording the post-brainstorm material. Post-write user clarification from the original brainstorm retained as history.
  - **Appendix B** — Rust Cargo.toml diff replaced with an `mix.exs` deps projection: `{:erlexec, "~> 2.2"}` new, `{:jason, ...}` + `{:telemetry, ...}` pre-existing, `extra_applications: [:logger, :exec]` entry shown.
  - **Appendix C** — Glossary updated: Rust entries removed, new entries for `erlexec`, `exec-port`, session GenServer, injected tools, task-level completion, tool-call boundary.

  Sibling plan updates:
  - `LLM_STATE/sub-C-adapters/backlog.md` top-notice rewritten to point at the rewritten §1–§11 as the starting point for the task-list rewrite; prior §12 breadcrumb removed.
  - `LLM_STATE/sub-C-adapters/memory.md` BEAM/Elixir pivot section updated accordingly; §12 breadcrumb removed; Q3/Q4 marked RESOLVED and Q6/Q7 newly recorded.

  **Pattern note**: durable feedback memory saved under `memory/feedback_pivot_rewrite.md` — significant pivots get fresh inline rewrites, not supersede layers. Applies to all remaining amendment tasks in this backlog (sub-A, sub-B, sub-D, sub-E, sub-M, sub-G, sub-H, sub-I).

  **Unblocks**: **P3.1** sub-F sibling plan scaffolding (no longer blocked on C's design direction) and the **sub-C implementation-phase backlog rewrite** (discrete task in the sub-C sibling plan).

### Sub-D brainstorm (scope collapsed) — daemon singleton + external-tool coordination `[brainstorm]`
- **Status:** not_started
- **Dependencies:** F done
- **Description:** **D's original scope (per-plan advisory locks for multi-process coordination) is collapsed by F's daemon commitment.** OTP actor mailboxes serialize writes naturally; the daemon is a singleton so per-plan locking is not needed for Mnemosyne's own coordination. D's remaining scope is much smaller:
  - **Daemon singleton lock** at `<vault>/runtime/daemon.lock` via `flock`. Prevents a second daemon from starting on the same vault.
  - **Advisory file locks for external-tool coordination**: when Obsidian or a user's editor writes to a plan file concurrently with the daemon, how does Mnemosyne handle the conflict? Detection strategies, rollback-and-retry, user-facing conflict resolution.
  - **Vault git concurrency**: the daemon commits to vault git periodically (routing rule suggestions, plan catalog regeneration, knowledge promotions). Git push/pull from the user's side should not corrupt daemon-in-flight commits.

  Much smaller brainstorm than originally scoped. Estimated a single short work phase.

  Output: design doc at `docs/superpowers/specs/2026-MM-DD-sub-D-coordination-design.md`. Sibling plan only if implementation tasks warrant one.
- **Results:** _pending_

### Sub-E amendment — expert-dispatched knowledge curation `[amendment]`
- **Status:** not_started
- **Dependencies:** F done (N's brainstorm not required — amendment describes the interface E uses from the sub-N contract)
- **Description:** Amend sub-E's design doc to reflect:
  - Stage 5 (knowledge store write) becomes **dispatch-to-experts**
  - Candidate entries are sent as Query messages to relevant experts
  - Each expert reviews the candidate in its own fresh context, decides absorb/reject/cross-link
  - Multi-expert absorption is allowed (single entry in multiple scopes with wikilinks)
  - Conflict between experts surfaces in the ingestion event log for human review
  - Pipeline stages 1-4 (extract, classify, contradict, score) unchanged
  - Implementation in Elixir with `GenStage` or `Broadway` for pipeline stages

  Moderate amendment — Stage 5 is re-cast, others stay.
- **Results:** _pending_

### Sub-M amendment — :telemetry + typed Elixir struct events `[amendment]`
- **Status:** not_started
- **Dependencies:** F done
- **Description:** Amend sub-M's design doc to reflect:
  - Hybrid pattern unchanged: typed events at boundaries, generic instrumentation underneath
  - Implementation re-cast from Rust `tracing` + `MnemosyneEvent` enum to Elixir `:telemetry` + `Mnemosyne.Event.*` structs
  - Typed structs (sealed set): `PhaseTransition`, `MessageRouted`, `RuleFired`, `RuleSuggestion`, `ActorStateChange`, `HarnessOutput`, `DispatchProcessed`, `QueryAnswered`, `Ingestion.*`, `SpawnLatencyReport`
  - `:telemetry` for transport, `prom_ex` or equivalent for Prometheus metrics export
  - F's event types added to the sealed set (dispatch/query events, actor lifecycle events, rule firings)
  - Adoption path for tactical seeds (C's `SpawnLatencyReport`) unchanged: parallel-emit + mechanical verification + seed deletion

  Significant amendment — the architectural split survives but the implementation stack changes entirely.
- **Results:** _pending_

### Sub-G amendment — daemon invocation pattern in migration `[amendment]`
- **Status:** not_started
- **Dependencies:** F done
- **Description:** Amend sub-G's (still-pending) design to account for:
  - Migration now renames `LLM_STATE/` → `<project>/mnemosyne/project-root/` (not `<project>/mnemosyne/plans/`)
  - `phase.md` → `plan-state.md` rename still applies
  - Migration gains a "start the daemon" step (user runs `mnemosyne daemon --init` post-migration)
  - Rust v0.1.0 CLI deletion scope expands: the entire previous CLI is retired (daemon + TUI split), not just the hardcoded `~/.mnemosyne/` paths
  - Per-project `project-root/` directory creation during migration
  - Vault catalog regeneration on first daemon start
  - Previous amendment tasks from Session 8 (`pre-work.sh`, `prompt-*.md`, `compact-baseline`, `related-plans.md` schema) still apply

  G's brainstorm can proceed after this amendment is absorbed.
- **Results:** _pending_

### Sub-H amendment — skills as attached-client TUI actions `[amendment]`
- **Status:** not_started
- **Dependencies:** F done
- **Description:** Amend sub-H's (still-pending) design to account for:
  - Skills become **attached-client TUI actions**, not standalone commands or harness slash commands
  - Each skill maps to a TUI command that the TUI sends to the daemon over the NDJSON protocol
  - The daemon routes the command to the appropriate actor, which executes the skill's behavior
  - Co-equal-actors principle unchanged: every skill must have a human-driven form, now explicitly via the TUI client
  - Dispatched tasks (new concept from F) may supersede some skills entirely (e.g., `/promote-global` becomes "dispatch to rust-expert for review")

  H's brainstorm can proceed after this amendment is absorbed.
- **Results:** _pending_

### Sub-I amendment — Obsidian as daemon client `[amendment]`
- **Status:** not_started
- **Dependencies:** F done
- **Description:** Amend sub-I's (still-pending) design to account for:
  - Obsidian is now **a concrete daemon client** (via Obsidian plugin in sub-K, or via direct file observation for v1)
  - Coverage document still describes which Obsidian features cover which data surfaces
  - Vault catalog (`<vault>/plan-catalog.md`) is a major new data surface to document
  - Routing module (`<vault>/routing.ex`) is a user-editable surface with syntax-highlighting concerns
  - Daemon event stream (via sub-K protocol) opens new "live view" possibilities

  I's brainstorm can proceed after this amendment is absorbed.
- **Results:** _pending_

## Priority 2 — Remaining sub-project brainstorms

### Brainstorm sub-project G — migration `[brainstorm]`
- **Status:** not_started
- **Dependencies:** Sub-G amendment complete (Priority 1)
- **Description:** Design the migration path from `LLM_STATE/` + `LLM_CONTEXT/` to `<project>/mnemosyne/` + the BEAM daemon. Scope includes: directory renames, `phase.md` → `plan-state.md`, daemon start step, Rust CLI retirement, per-project `project-root/` creation, vault catalog regeneration, and the Session 8 carry-forward items (`pre-work.sh`, `prompt-*.md`, `compact-baseline`, `related-plans.md` schema deletion). F-impact notes in the Sub-G amendment task (Priority 1) define the scope constraints.

  Output: design doc at `docs/superpowers/specs/YYYY-MM-DD-sub-G-migration-design.md` and sibling plan at `LLM_STATE/sub-G-migration/`.
- **Results:** _pending_

### Brainstorm sub-project H — skills as TUI actions `[brainstorm]`
- **Status:** not_started
- **Dependencies:** Sub-H amendment complete (Priority 1); Sub-B amendment complete
- **Description:** Design how the 7 Claude Code skills fold into attached-client TUI actions. F-impact notes in the Sub-H amendment task (Priority 1) define the framing: skills become daemon commands routed through the NDJSON protocol, not harness slash commands. Co-equal-actors principle means every skill must have a human-driven TUI form. Some skills may be superseded by dispatch-to-experts.

  Output: design doc at `docs/superpowers/specs/YYYY-MM-DD-sub-H-skills-design.md` and sibling plan at `LLM_STATE/sub-H-skills/`.
- **Results:** _pending_

### Brainstorm sub-project I — Obsidian coverage document `[brainstorm]`
- **Status:** not_started
- **Dependencies:** Sub-I amendment complete (Priority 1)
- **Description:** Document which Obsidian features cover which Mnemosyne data surfaces (Tier 1/2 knowledge, plan state, sessions, ingestion provenance, vault catalog, routing rules). F-impact notes in the Sub-I amendment task (Priority 1) add: Obsidian as daemon client, vault catalog as new data surface, daemon event stream possibilities. Produce the `.obsidian/` template that ships with v1.

  Output: design doc at `docs/superpowers/specs/YYYY-MM-DD-sub-I-obsidian-coverage-design.md`.
- **Results:** _pending_

### Brainstorm sub-project N — domain experts `[brainstorm]` (new, F-added)
- **Status:** not_started
- **Dependencies:** F done (satisfied). N's brainstorm needs F's design doc, not F's implementation task list — the sibling plan scaffolding is NOT a prerequisite for N's brainstorm. N's *implementation* plan may depend on F's scaffolding landing first.
- **Description:** Design the ExpertActor type F reserved as a type hole. Scope:
  - **Declaration file format** at `<vault>/experts/<expert-id>.md`: persona, knowledge scope, retrieval strategy, optional model override
  - **Persona authoring**: how users write effective personas; examples for each default expert
  - **Retrieval strategies**: `keyword` for v1 (grep-based scoring against question terms); `semantic` for v1.5+ (embedding-based); pluggable behind a `strategy:` field
  - **Default expert set**: initial candidates are `rust-expert`, `research-expert`, `distributed-systems-expert`, `software-architect`, `obsidian-expert`, `ffi-expert`. Finalize and ship with starter declarations.
  - **Ingestion integration**: how sub-E's Stage 5 dispatch-to-experts reaches N's actors (interface contract)
  - **Multi-expert absorption**: how the same knowledge entry lands in multiple experts' scopes with cross-linking
  - **Conflict detection**: how expert disagreements surface for human review
  - **Query granularity**: one actor per expert? per cluster? per entry? F assumes one actor per expert declaration file, N confirms or revises.

  Output: design doc at `docs/superpowers/specs/YYYY-MM-DD-sub-N-domain-experts-design.md` and sibling plan at `LLM_STATE/sub-N-domain-experts/`.
- **Results:** _pending_

### Brainstorm sub-project O — mixture of models `[brainstorm]` (new, F-added, v1.5+)
- **Status:** not_started
- **Dependencies:** F done; sub-N done; v1 implementation landed (or close)
- **Description:** **Reserved for v1.5+.** F reserved the schema hooks (`[harnesses.*]` daemon config section, `model:` actor field, `[fact_extraction].model` config). Sub-O implements:
  - Multi-adapter harness layer (multiple concurrent adapters in the daemon)
  - Per-actor model selection (routing resolves `model:` to the appropriate adapter + model combo)
  - Local-model adapters (Ollama, llama.cpp, similar)
  - Cost telemetry (tokens consumed per actor, per session, per model)
  - Economic discipline: users can override defaults, but the defaults should be sensible (expensive models for plan actors, cheap/local for fact extraction)

  Not started until v1 is implemented.

  Output: design doc at `docs/superpowers/specs/YYYY-MM-DD-sub-O-model-mixing-design.md` and sibling plan.
- **Results:** _pending_

### Brainstorm sub-project P — team mode `[brainstorm]` (new, F-added, v2+)
- **Status:** not_started
- **Dependencies:** F done; v1 implementation landed; optionally sub-O
- **Description:** **Reserved for v2+.** F reserved the schema hooks (`[peers]` daemon config section, `<peer>@<qualified-id>` syntax in qualified IDs). Sub-P implements:
  - Network transport for cross-daemon dispatch: BEAM distribution (`Node.connect/1`) or custom TCP
  - Peer discovery: static peer list in config, or mDNS, or DHT
  - Cross-daemon authentication: shared secret, TLS, cookie-based BEAM auth
  - Shared-vault conflict resolution: git-based sync, CRDT-based live sync, or a central vault service
  - Multi-user identity in daemon config
  - Distributed experts: should experts be replicated, partitioned, or centralized?
  - Curation workflow: whose expert accepts a dispatched candidate when multiple users have matching experts?

  A substantial brainstorm at v2 milestone.

  Output: design doc at `docs/superpowers/specs/YYYY-MM-DD-sub-P-team-mode-design.md` and sibling plan.
- **Results:** _pending_

### Brainstorm sub-project L — Obsidian terminal plugin spike `[brainstorm]`
- **Status:** not_started
- **Dependencies:** none (prerequisite for K but independent of other sub-projects)
- **Description:** Small investigation. Evaluate existing Obsidian terminal plugins (obsidian-terminal, obsidian-execute-code, others) for PTY control, streamed output capture, clean termination, and integration with external processes. Recommend fork, build-new, or use-existing for K's scope.

  **Unchanged by F commitment** — K is still a v1.5+ alternative client on F's socket protocol. L's evaluation informs K's implementation plan.

  Follow the guivision + OCR evidence pattern for any UI-inside-Obsidian testing.

  Output: design doc (short) at `docs/superpowers/specs/YYYY-MM-DD-sub-L-obsidian-terminal-spike.md`.
- **Results:** _pending_

### Brainstorm sub-project K — Obsidian plugin client `[brainstorm]`
- **Status:** not_started (v1.5+)
- **Dependencies:** F done; sub-L complete; v1 daemon implementation stable
- **Description:** Design the Obsidian plugin that consumes F's NDJSON client protocol and provides an Obsidian-integrated UI alternative to the Rust TUI. Command palette, plan-state panel via Dataview, terminal integration for hosting harness sessions, multi-plan dashboards.

  **Scope clarified by F.** K is now explicitly "another client on F's socket protocol" — not a re-implementation of the daemon inside Obsidian. The plugin talks NDJSON over Unix socket (or TCP for v2 remote daemons) to the Elixir daemon.

  K does not replace the Rust TUI. Both coexist.

  Output: design doc at `docs/superpowers/specs/YYYY-MM-DD-sub-K-obsidian-plugin-design.md` and sibling plan.
- **Results:** _pending_

## Priority 3 — Decisions and gates

### Scaffold sub-F sibling plan (post BEAM spike) `[scaffold]`
- **Status:** not_started (**unblocked** — spike done Session 10)
- **Dependencies:** BEAM PTY spike complete (**done**)
- **Description:** Spike validated pipes-only erlexec (no PTY, no Rust wrapper fallback). Scaffold sub-F's sibling plan at `LLM_STATE/sub-F-hierarchy/` with the task list from F's design doc §11. This is what F's brainstorm deferred because the implementation plan details depend on the spike's outcome.

  Output: `LLM_STATE/sub-F-hierarchy/` with backlog, memory, session-log, phase files.
- **Results:** _pending_

### Decide v1 scope cut `[decision]`
- **Status:** not_started
- **Dependencies:** all in-scope brainstorms complete (A, B, C, E, F, M done; D, G, H, I, N pending; K, L pending but v1.5+)
- **Description:** Once every in-scope sub-project has been brainstormed and its design doc and implementation plan exist, decide what's actually in v1 vs. deferred to v1.5/v2. Update memory.md with the v1 cut. Adjust dependent implementation plans accordingly.

  Sub-N (domain experts) is **in v1** — F relies on ExpertActor shipping alongside PlanActor. Without experts, Query messages have no interesting targets.

  Sub-O (MoM) and sub-P (team mode) are **v1.5+ and v2+ by design** — not candidates for v1 scope cut.

  Sub-K (Obsidian plugin client) remains v1.5+ per F's TUI-first commitment.

  Sub-I and sub-L are small documentation/spike work that can land wherever convenient.
- **Results:** _pending_

---

## Completed — done and recorded for history

### Brainstorm sub-project F — plan hierarchy + actor model + dispatch + declarative routing `[brainstorm]`
- **Status:** done (Session 9, 2026-04-14)
- **Description:** Originally scoped as "plan hierarchy + root plan"; expanded during Session 9 to absorb the full v1 architecture commitment.
- **Results:**
  - Design doc at `docs/superpowers/specs/2026-04-14-sub-F-hierarchy-design.md` (comprehensive)
  - Documentation overhaul: new `README.md`, new `docs/architecture.md`, rewritten `docs/user-guide.md`, rewritten `docs/configuration.md`
  - Committed architectural decisions (see memory.md for full list):
    - Mnemosyne is a persistent BEAM daemon (Elixir/OTP)
    - Two sealed actor types: `PlanActor` and `ExpertActor`
    - Two message types: `Dispatch` and `Query`
    - `project-root` as reserved plan directory name; `<project>/mnemosyne/plans/` collapses into it
    - Path-based qualified plan IDs, never stored
    - Dispatch asymmetry: same-project direct, cross-project via Level 2 routing agent
    - Vault catalog (`<vault>/plan-catalog.md`) replaces `related-plans.md`
    - Description discipline: 120-character hard cap
    - Declarative routing via pattern-matched Elixir (`<vault>/routing.ex`) with LLM fallback and learning loop
    - Rust TUI as separate client binary over Unix socket NDJSON
    - Reserved extensibility hooks for MoM (sub-O) and team mode (sub-P)
  - Three new sub-projects added: **N (experts)**, **O (MoM, v1.5+)**, **P (team mode, v2+)**
  - Nine amendment tasks landed on this backlog (A, B, C, E, G, H, I, M, plus D's scope collapse)
  - BEAM PTY spike identified as the critical unblocker for sub-F's sibling plan scaffolding
  - Sub-F sibling plan scaffolding **deferred** until spike validates sub-C approach

### Brainstorm sub-project A — global knowledge store `[brainstorm]`
- **Status:** done (Session 7, 2026-04-13)
- **Results:** Design doc `specs/2026-04-13-sub-A-global-store-design.md`. Explicit vault discovery, `mnemosyne.toml` marker, init/adopt commands, Tier 1/2 env-var overrides, gitignore policy. v0.1.0 migration scope dropped. Amendment task pending F commitment integration.

### Brainstorm sub-project M — observability framework `[brainstorm]`
- **Status:** done (Session 7, 2026-04-13)
- **Results:** Design doc `specs/2026-04-13-sub-M-observability-design.md`. Hybrid `tracing` + `MnemosyneEvent` architecture. Adoption stubs landed in sub-B/C/E; D/F/H/I/G stubs queued. Amendment task pending: re-cast to `:telemetry` + typed Elixir struct events.

### Brainstorm sub-project C — harness adapter layer `[brainstorm]`
- **Status:** done (Session 6, 2026-04-13)
- **Results:** Design doc `specs/2026-04-13-sub-C-adapters-design.md`. V1 Claude Code only. Four B amendments (stream-json, SessionLifecycle, SpawnLatencyReport, sentinel detection). Actor threading (3 threads). Process-group termination v1. Amendment task pending: Elixir implementation, BEAM PTY spike, multi-adapter reservation for sub-O.

### Brainstorm sub-project B — phase cycle `[brainstorm]`
- **Status:** done (Session 4, 2026-04-12)
- **Results:** Design doc `specs/2026-04-12-sub-B-phase-cycle-design.md`. Produced: hard errors, no slash commands, Obsidian explorer, vault+symlinks, per-project `mnemosyne/`, embedded prompts, Path 1. Folded J. Surfaced K, L. Amendment task pending: actor-hosted phase cycle, schema pruning, placeholder rename, `related-plans.md` deletion.

### Brainstorm sub-project E — post-session knowledge ingestion `[brainstorm]`
- **Status:** done (Session 3, 2026-04-12)
- **Results:** Design doc `specs/2026-04-12-sub-E-ingestion-design.md`. B's `ReflectExitHook` as E's subscription point. Amendment task pending: Stage 5 becomes dispatch-to-experts.

### Obsidian symlink validation spike `[spike]`
- **Status:** done (Session 5, 2026-04-13)
- **Results:** PASSED 6/6 on macOS and Linux. Evidence at `tests/fixtures/obsidian-validation/results/{macos,linux}/`. Hard-copy-staging fallback not needed. Canonical guivision + OCR evidence pattern established.
