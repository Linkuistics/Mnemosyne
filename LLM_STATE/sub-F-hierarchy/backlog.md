# Backlog — Sub-project F: Plan Hierarchy, Actor Model, Dispatch, Declarative Routing

Implementation backlog for sub-project F. All tasks derive from §11 of
the design doc at
`{{PROJECT}}/docs/superpowers/specs/2026-04-14-sub-F-hierarchy-design.md`.
Consult the spec (§1–§10) before starting any task.

**Sibling plan scaffolded Session 13 (2026-04-15)** of the parent
orchestrator plan. The BEAM PTY spike (Session 10) validated pipes-only
erlexec, sub-C's design doc was rewritten inline for Elixir/BEAM in
Session 11, and sub-B's was rewritten in Session 12 — together these
unblock F's implementation runway. No implementation work has started
yet; Task 0 is the gate.

## Task Backlog

### Task 0 — Implementation readiness gate `[gate]`
- **Status:** not_started
- **Dependencies:** sub-B downstream task-list rewrite (sub-B backlog
  gate task); sub-C downstream task-list rewrite (sub-C backlog gate
  task); sub-A amendment absorbed; sub-M amendment absorbed
- **Description:** Meta-task guarding the start of real implementation
  work. Before any Task 1+ runs, verify:
  1. `{{PROJECT}}/LLM_STATE/sub-B-phase-cycle/backlog.md` Tasks 1+
     have been rewritten against the Session-12 design-doc rewrite
     (the 13-step `PhaseRunner.run_phase/4` flow, the
     `PhaseExecutor @behaviour`, sliding-buffer sentinel matcher,
     etc.).
  2. `{{PROJECT}}/LLM_STATE/sub-C-adapters/backlog.md` Tasks 1+ have
     been rewritten against the Session-11 design-doc rewrite
     (pipes-only erlexec, session GenServer, tool-call boundary in
     §4.5).
  3. ~~Sub-A's amendment task in the orchestrator backlog has been
     absorbed into `specs/2026-04-13-sub-A-global-store-design.md`.~~
     **CLEARED** — completed in orchestrator Session 14 (2026-04-15).
  4. ~~Sub-M's amendment task in the orchestrator backlog has been
     absorbed into `specs/2026-04-13-sub-M-observability-design.md`.~~
     **CLEARED** — completed in orchestrator Session 15 (2026-04-15).
  Also re-read F's §11 against the latest versions of those three
  design docs to confirm every consumed interface still matches.
  Any mismatch discovered here rewrites F's design doc inline
  (per the "amendment tasks rewrite specs inline" discipline) before
  implementation starts.
- **Results:** _pending_

---

## §11.1 Elixir scaffolding

### Task 1 — `Mnemosyne.VaultDirectory` `[foundation]`
- **Status:** not_started
- **Dependencies:** Task 0
- **Description:** Vault walk + frontmatter parse + invariant checks
  per §3.1 of the spec. Walks `<vault>/projects/<project>/` subtrees,
  collects every directory containing `plan-state.md`, parses the
  YAML frontmatter (using `yaml_elixir`), computes the path-based
  qualified ID (F-5), validates the description discipline (F-8,
  120-char cap, non-placeholder), and returns an in-memory
  `%VaultDirectory{plans: map, experts: map, errors: list}` struct.
  Hard errors on: missing description, >120 char description,
  placeholder description (`TODO`, `FIXME`, `...`, `tbd`), broken
  `<vault>/projects/<name>` symlinks, `project-root` deleted while
  children exist, plan-state frontmatter parse errors (§9.2, §9.3).
- **Results:** _pending_

### Task 2 — `Mnemosyne.Actor` behaviour `[foundation]`
- **Status:** not_started
- **Dependencies:** Task 0
- **Description:** Elixir `@behaviour` definition with callbacks
  shared by PlanActor and ExpertActor per §4.1. Callbacks cover
  actor init (from vault-directory plan entry), message handling
  (Dispatch and Query variants), state persistence hooks, and
  supervision integration. Keep the callback set minimal — anything
  specific to PlanActor or ExpertActor goes in the respective
  implementation modules, not this behaviour.
- **Critical-path note (2026-04-15):** This task (together with Task 5
  `ActorSupervisor`) is on the critical path for sub-N. Sub-N's
  internal Task 0 is an explicit gate: sub-N Tasks 16+
  (ExpertActor implementation) cannot start until F has shipped the
  `Mnemosyne.Actor` behaviour **and** the `ActorSupervisor`
  child-spec API. Sub-N Tasks 1–15 are independent of F and can run
  immediately; sub-N Phase 5+ is blocked on F delivering these two
  interfaces. Prioritise Tasks 2 and 5 to avoid delaying sub-N.
- **Results:** _pending_

### Task 3 — `Mnemosyne.ExpertActor` stub `[foundation]`
- **Status:** not_started
- **Dependencies:** Task 2
- **Description:** Implement `Mnemosyne.ExpertActor` as a GenServer
  satisfying the `Mnemosyne.Actor` behaviour but returning
  `{:error, :not_yet_implemented}` for every Query it receives.
  The type hole exists so the supervision tree, router, and
  dispatch processor can wire against ExpertActor targets before
  sub-N lands. Do not implement persona / retrieval / scope logic
  here — that is sub-N's scope.
- **Stub-replacement reference (2026-04-15):** When sub-N lands and
  this stub is replaced by a full implementation, the authoritative
  spec for ExpertActor's internals (persona format, ScopeMatcher,
  retrieval strategies, message structs, event structs) is
  `{{PROJECT}}/docs/superpowers/specs/2026-04-15-sub-N-domain-experts-design.md`.
  The stub replacement is not free-form — it must conform to that
  design doc. Sub-N Task 15 (early-deliverable PR) and Tasks 16+
  implement the actual ExpertActor; this Task 3 only provides the
  typed shell.
- **Results:** _pending_

### Task 4 — `Mnemosyne.PlanActor` wrapping sub-B's `PhaseRunner` `[foundation]`
- **Status:** not_started
- **Dependencies:** Task 2, sub-B's `PhaseRunner` module landed per
  the rewritten sub-B backlog
- **Description:** GenServer implementing `Mnemosyne.Actor` that
  hosts sub-B's `PhaseRunner` in its state. Message protocol per
  §4.5 of sub-B's rewritten spec: accept `{:run_phase, phase}` from
  attached clients, delegate to `PhaseRunner.run_phase/4`, emit
  `%PhaseLifecycle{}` events to sub-M. On Dispatch/Query arrival,
  append to the plan's backlog file via `DispatchProcessor`
  (Task 7) at phase exit, not on arrival. Crash recovery per §9.4:
  rebuild actor state from the plan's filesystem files on OTP
  restart; do not hold in-memory state the filesystem can't
  reproduce.
- **Results:** _pending_

### Task 5 — `Mnemosyne.ActorSupervisor` `[foundation]`
- **Status:** not_started
- **Dependencies:** Tasks 3, 4
- **Description:** OTP DynamicSupervisor supervising all actor
  GenServers with `restart: :transient` per §4.5 of F's design doc.
  Provides `start_plan_actor/1` and `start_expert_actor/1`
  functions. Children are started lazily on first message to a
  qualified ID; idle actors may be stopped to reclaim memory (v1
  policy: no idle eviction — simpler).
- **Critical-path note (2026-04-15):** Together with Task 2, this task
  gates sub-N Phase 5+. Sub-N's ExpertActor (Tasks 16+) calls
  `ActorSupervisor.start_expert_actor/1` directly; until this
  child-spec API exists sub-N cannot wire its GenServer into the
  supervision tree. Ship Tasks 2 and 5 before sub-N reaches its
  internal Task 0 gate.
- **Results:** _pending_

### Task 6 — `Mnemosyne.Router.Server` + mailbox NDJSON + cursor `[foundation]`
- **Status:** not_started
- **Dependencies:** Task 5
- **Description:** Router GenServer that owns message routing per
  §4.7. Accepts inbound messages (from clients, from phase-exit
  processors, from Level 2 agent responses), consults the routing
  module (Task 14), dispatches to target actors via the supervisor.
  Persists mailbox state to `<vault>/runtime/mailboxes/<qid>.jsonl`
  (NDJSON append) with a cursor file tracking last-processed index
  per §4.8. Crash recovery replays from the cursor.
- **Results:** _pending_

### Task 7 — `Mnemosyne.DispatchProcessor` `[foundation]`
- **Status:** not_started
- **Dependencies:** Tasks 1, 6
- **Description:** Phase-exit hook that parses `dispatches.yaml`
  from the plan's staging directory (§5.2), validates each entry
  (§9.1: cannot target self, target-plan only for same-project,
  target-project only for cross-project, `suggested-target-plan`
  scope check), routes through the router (Task 6), and writes to
  the target's `Received` section mechanically. Uses the
  `dispatches.yaml.processing` marker + index pattern for at-most-
  once delivery (§9.4 crash recovery). Called by `PhaseRunner` on
  non-compact phase exit.
- **Results:** _pending_

### Task 8 — `Mnemosyne.QueryProcessor` `[foundation]`
- **Status:** not_started
- **Dependencies:** Tasks 1, 6
- **Description:** Phase-exit hook mirroring `DispatchProcessor`
  for `queries.yaml` per §2.5 and §5.2. Same validation rules,
  same crash-recovery pattern. Query responses are routed back to
  the originating plan's `Dispatched` section as audit trail;
  mid-session Query responses flow via sub-C's tool-call boundary
  (§5.3) and are NOT written via this processor.
- **Results:** _pending_

### Task 9 — `Mnemosyne.CatalogRenderer` `[foundation]`
- **Status:** not_started
- **Dependencies:** Task 1
- **Description:** Render `<vault>/plan-catalog.md` from a
  `%VaultDirectory{}` per §3.2. Grouping by project, each plan
  and expert rendered with qualified ID + description + dispatch
  rule summary. Also provides the substitution content for the
  `{{VAULT_CATALOG}}` placeholder consumed by sub-B's phase
  prompts. Regeneration triggers per §3.3: plan mutation events
  from the router, and every phase-prompt render. File header
  marks it "machine-owned" (§9.3). Hard error on description
  violations discovered at render time.
- **Results:** _pending_

---

## §11.2 Daemon binary

### Task 10 — `mnemosyne daemon` mix task / escript `[daemon]`
- **Status:** not_started
- **Dependencies:** Tasks 5, 6, 13, and sub-A vault discovery
- **Description:** Entry point per §4.6. Startup sequence:
  1. Parse CLI args (`--vault`, `--log-level`).
  2. Resolve vault via sub-A (`--vault` → `MNEMOSYNE_VAULT` →
     user config → hard error).
  3. Call `VaultDirectory.load` + hook into sub-A's
     `verify_vault`.
  4. Acquire singleton lock (Task 12).
  5. Start OTP application tree rooted at
     `Mnemosyne.Application`.
  6. Start the `ClientListener` (Task 11).
  7. Install signal handlers (SIGTERM → graceful shutdown,
     SIGINT → graceful shutdown, SIGHUP → reload routing.ex).
  Shutdown path: stop listener, drain in-flight phase runs, stop
  supervisor, release lock, exit.
- **Results:** _pending_

### Task 11 — `Mnemosyne.ClientListener` + `Mnemosyne.ClientConnection` `[daemon]`
- **Status:** not_started
- **Dependencies:** Task 6
- **Description:** Unix socket listener at
  `<vault>/runtime/daemon.sock` per §7.1 accepting NDJSON commands
  per §7.2. Per-connection `GenServer` handling the command set:
  `attach-plan`, `detach`, `run-phase`, `list-plans`,
  `show-catalog`, `subscribe-events`, `review-rule-suggestion`,
  etc. Forward-compatibility per §7.3 (unknown commands →
  structured error, not crash). Multi-client semantics per §7.4
  (multiple attached clients on one plan share event stream;
  `run-phase` requires exclusive control).
- **Results:** _pending_

### Task 12 — Daemon singleton lock `[daemon]`
- **Status:** not_started
- **Dependencies:** Task 10
- **Description:** Acquire `flock` on
  `<vault>/runtime/daemon.lock` at startup. Second daemon fails
  with actionable diagnostic naming the first daemon's PID
  (§9.3). Lock released on shutdown via the OS (not a
  `defer` / `on_exit` — flock is released when the file handle
  closes, which happens on process exit regardless of path).
- **Results:** _pending_

### Task 13 — `daemon.toml` parser `[daemon]`
- **Status:** not_started
- **Dependencies:** Task 0
- **Description:** Parse `<vault>/daemon.toml` with reserved
  sections per F-12. V1 fields: `[daemon]` (log level, socket
  path override), `[fact_extraction]` (model selection),
  `[harnesses.*]` (reserved for sub-O; parse-and-ignore-unknown
  with forward-compat warning per §9.3), `[peers]` (reserved for
  sub-P; hard error if non-empty per §9.3). Use `toml_elixir`
  or equivalent.
- **Results:** _pending_

---

## §11.3 Declarative routing

### Task 14 — `Mnemosyne.UserRouting` behaviour + loader `[routing]`
- **Status:** not_started
- **Dependencies:** Task 6
- **Description:** Define the `@behaviour` every user `routing.ex`
  module must implement per §6.1: a `route/2` function with
  clauses pattern-matched on `(:dispatch | :query, facts)` →
  `{:target_plan, qid} | {:target_expert, id} | {:target_project,
  name} | :no_route`. Load user-supplied `<vault>/routing.ex` at
  startup via `Code.compile_file/1`. Hard error on compile
  failure at startup (§9.3); previous version stays loaded on
  hot-reload compile failure.
- **Results:** _pending_

### Task 15 — `Mnemosyne.FactExtractor.Server` `[routing]`
- **Status:** not_started
- **Dependencies:** Task 14, sub-C adapter landed
- **Description:** GenServer that extracts concern keywords
  (facts) from each outbound message body via a cheap LLM pass
  per §6.3. Uses sub-C's `HarnessAdapter` with a tightly
  constrained prompt and tool profile. Default model: Haiku.
  Reserved for sub-O to swap in local models. Facts feed the
  router's `route/2` call.
- **Results:** _pending_

### Task 16 — Hot code reload watcher for `routing.ex` `[routing]`
- **Status:** not_started
- **Dependencies:** Task 14
- **Description:** Watch `<vault>/routing.ex` for mtime changes
  (via `FileSystem` or periodic poll — whichever is simpler for
  v1) and recompile via `Code.compile_file/1` per §6.2. On
  compile success, hot-swap the loaded module (BEAM primitive).
  On compile failure, log a warning and leave the previous
  module loaded; surface the failure to attached TUI clients as
  a `%RuleCompileError{}` event.
- **Results:** _pending_

### Task 17 — Rule validation + actionable error reporting `[routing]`
- **Status:** not_started
- **Dependencies:** Task 14
- **Description:** Dialyzer-compatible typecheck that each
  `route/2` clause's return shape matches the allowed variant
  set. If a clause returns something unexpected at runtime,
  log the violating clause + input + actual return, treat as
  `:no_route`, and fall back to Level 2 per §6.4. Error
  reporting must name the line in `routing.ex` (preserve
  source locations via `Code.compile_file/1` options).
- **Results:** _pending_

---

## §11.4 Level 2 routing agent

### Task 18 — Level 2 prompt template + spawn path `[level2]`
- **Status:** not_started
- **Dependencies:** Task 14, sub-C adapter landed
- **Description:** Prompt template for the Level 2 routing agent
  per §6.4. Fresh-context Claude Code session, tool profile
  scoped to read-only access of the target project's vault
  subtree (plans + source code), with authority to pick a
  specific target plan or reject with reasoning. Spawned via
  sub-C's adapter. Input: the message being routed, the
  target project name, and the vault catalog excerpt for that
  project. Output: a YAML block with `target-plan:` or
  `routing-response:` fields.
- **Results:** _pending_

### Task 19 — Rejection + retarget feedback to origin `[level2]`
- **Status:** not_started
- **Dependencies:** Task 18
- **Description:** When Level 2 rejects or times out (§9.1),
  write a structured entry to the origin plan's `Dispatched`
  section per §2.6 recording the rejection reason and any
  retarget suggestion. The origin plan's next phase sees the
  rejection and can re-dispatch manually. Timeout threshold:
  5 minutes per §9.1.
- **Results:** _pending_

### Task 20 — Rule suggestion extraction `[level2]`
- **Status:** not_started
- **Dependencies:** Task 18
- **Description:** Parse optional `suggested-rule:` block from
  Level 2 output per §6.5. Normalize to an Elixir-AST-friendly
  shape and stage at `<vault>/runtime/rule-suggestions/<id>.ex`
  for user review. Do not auto-apply — accepting the suggestion
  is a user action via the TUI per the learning-loop design.
- **Results:** _pending_

### Task 21 — TUI event emission for pending rule suggestions `[level2]`
- **Status:** not_started
- **Dependencies:** Tasks 11, 20
- **Description:** Emit `%RuleSuggestion{}` events over the
  client event stream whenever a new suggestion is staged.
  Accepted via the `review-rule-suggestion` NDJSON command from
  an attached client (Task 11's command set). On acceptance,
  append the suggestion to `<vault>/routing.ex`, trigger hot
  reload (Task 16), and delete the staged file.
- **Results:** _pending_

---

## §11.5 Integration

### Task 22 — Hook `VaultDirectory.load` into sub-A's `verify_vault` `[integration]`
- **Status:** not_started
- **Dependencies:** Task 1, sub-A amendment absorbed, sub-A
  implementation landed
- **Description:** Extend sub-A's `verify_vault` to call
  `Mnemosyne.VaultDirectory.load/1` as its second step (after
  vault identity verification). This surfaces description
  discipline errors (F-8) and plan-state parse errors (§9.2)
  at daemon startup rather than deferring them to first use.
- **Results:** _pending_

### Task 23 — Hook `DispatchProcessor` + `QueryProcessor` into sub-B phase exit `[integration]`
- **Status:** not_started
- **Dependencies:** Tasks 7, 8, sub-B rewritten `PhaseRunner` landed
- **Description:** Register F's processors as phase-exit hooks on
  sub-B's `PhaseRunner` per step 13 of B's rewritten 13-step flow.
  Processors run only on non-compact phases (compact is strictly
  lossless). Failures in processors must surface as hard errors
  to the attached client; they must not silently swallow routing
  failures.
- **Results:** _pending_

### Task 24 — `:telemetry` events for actor state / routing / rules `[integration]`
- **Status:** not_started
- **Dependencies:** sub-M amendment absorbed, sub-M implementation
  landed
- **Description:** Emit typed `Mnemosyne.Event.*` structs at the
  boundary for every architecturally-meaningful state change per
  F §4 and §6. Events: `%ActorStateChange{}`, `%MessageRouted{}`,
  `%RuleFired{}`, `%RuleCompileError{}`, `%RuleSuggestion{}`,
  `%DispatchProcessed{}`, `%QueryAnswered{}`, `%Level2Invoked{}`,
  `%Level2Rejected{}`. Each event also flows through
  `:telemetry.execute/3` for subscribers. M owns the handler and
  transport; F owns the event boundary.
- **Results:** _pending_

---

## §11.7 Tests

### Task 25 — Unit tests (Elixir) `[tests]`
- **Status:** not_started
- **Dependencies:** Tasks 1–9 (incremental — unit tests land with each)
- **Description:** Per §10.1 — ExUnit tests for every module's
  pure logic: VaultDirectory walk + parse, path-based qualified
  ID computation, description validation, mailbox NDJSON
  encoding / cursor advance, dispatch/query YAML parse +
  validation, catalog rendering, routing clause invocation. Tests
  must cover every §9 edge case. TDD: tests written first.
- **Results:** _pending_

### Task 26 — Integration tests with fixture vaults `[tests]`
- **Status:** not_started
- **Dependencies:** Task 25 baseline established
- **Description:** Per §10.2 — fixture vaults committed to
  `{{PROJECT}}/test/fixtures/vaults/` each exercising a specific
  shape (empty vault, single project, multi-project, nested
  children, symlinks, broken symlinks, description violations).
  Integration tests drive `VaultDirectory.load`, catalog
  rendering, and router startup end-to-end against each fixture.
- **Results:** _pending_

### Task 27 — Dispatch / Query processor tests `[tests]`
- **Status:** not_started
- **Dependencies:** Tasks 7, 8, 26
- **Description:** Per §10.3 — end-to-end tests of
  `dispatches.yaml` and `queries.yaml` processing including
  the `.processing` marker crash-recovery protocol. Simulate
  daemon crash between marker-write and file-delete, restart,
  verify at-most-once delivery (no duplicated `Received`
  entries).
- **Results:** _pending_

### Task 28 — Optional e2e test `[tests]`
- **Status:** not_started
- **Dependencies:** Tasks 10, 11, 23, 24
- **Description:** Per §10.4 — full daemon-up, attach-plan,
  run-phase, dispatch, level2-reject, rule-suggestion, accept-
  suggestion round trip against a scripted fixture. Gated
  behind `mix test --only e2e` so it doesn't run on every
  push. Evidence captured as a JSON-Lines event trace.
- **Results:** _pending_

---

## Out of scope — carry-forward notes

These items are referenced by F's §11 but belong to a different plan
and must NOT be implemented here:

- **Rust `mnemosyne-tui` binary (§11.6)** — ratatui rendering, socket
  client, attach-detach UI, phase-run commands, live harness
  streaming, rule-suggestion review pane. This is a separate
  implementation plan to be scaffolded once F's daemon-side
  contract (Task 11) lands. F ships only the daemon side of the
  NDJSON protocol.
- **Cross-plan amendment landings (§11.8)** — done by F's own
  triage phase in Session 9 (2026-04-14) of the parent
  orchestrator plan. The amendment tasks are now on the
  orchestrator backlog under Priority 1. F does not execute them.
- **Sub-N ExpertActor internals** — F ships the type hole (Task 3)
  only. Persona format, retrieval strategies, default expert set
  all belong to sub-N's separate brainstorm and plan.
- **Sub-O mixture of models** — F reserves the schema hooks in
  Task 13 only. Multi-adapter wiring and per-actor model selection
  are v1.5+ work.
- **Sub-P team mode** — F reserves the schema hooks in Task 13
  only. Cross-daemon transport is v2+ work.
