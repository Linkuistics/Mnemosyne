# Memory — Mnemosyne Orchestrator

Drives the merge of LLM_CONTEXT functionality into Mnemosyne, transforming it into a harness-independent LLM orchestrator owning plan management, the work cycle, knowledge, harness session control, and inter-plan routing. V1 architecture committed to a persistent BEAM daemon hosting plan and expert actors.

## Stable architectural decisions

### Fresh LLM context is a first-class design goal
Every LLM-using component must use many short, fresh sessions rather than one long accumulating session. Context rot (drift, noise, stale assumptions) is a primary failure mode; phase cycle session boundaries are a feature, not a cost. Governing heuristic: **context depth scales with decision specificity** — Level 1 reasons broadly about its own plan, Level 2 reasons narrowly about one target project's code, Level 3 (if needed) reasons about one specific plan. Each level refocuses context rather than accumulating it. B's phase boundaries, C's harness lifecycle, E's ingestion triggers, F's Level 2 routing agents, and N's expert consultations must default to discrete fresh-context invocations with explicit state handoff through files.

### Mnemosyne is a persistent actor daemon on BEAM
V1 runs as a single long-running Elixir/OTP application. `mnemosyne daemon` is the entry point; it hosts all actors, message routing, supervision, harness adapters, fact extraction, and ingestion. Previous per-plan-CLI-invocation framing is retired. OTP supervision, message passing, hot code reload, and distribution transparency are all BEAM primitives the design would otherwise hand-roll — when a design process independently arrives at OTP's decisions, that's the universe signaling the runtime. Gleam is reserved as a future migration target if Elixir's dynamic typing proves painful for the invariant-heavy design.

### Two sealed actor types: PlanActor and ExpertActor
Both implement a shared `Mnemosyne.Actor` behavior on top of `GenServer`. Set is **sealed** for v1 — adding a third type is a code change, not a plugin mechanism. PlanActor progresses through phase cycles with backlog + memory + session log. ExpertActor is consultative, query-native, persona-flavored, with a curated knowledge scope. F owns the `Actor` trait and both type-holes; sub-N (new) owns ExpertActor internals.

### Two message types: Dispatch and Query
**Dispatch** — fire-and-forget task delivery. Lands in target's `backlog.md` `## Received` section. Target's next phase cycle processes. Durable form: `<origin>/dispatches.yaml`.
**Query** — request-response reasoning. Target spawns fresh-context session, answers, response routes back through the harness tool-call boundary into the originating session inline. Durable form for deferred queries: `<origin>/queries.yaml`. In-session queries don't write files.
Both share target resolution, declarative routing, Level 2 fallback, audit trail conventions.

### project-root as reserved plan directory
Every adopted project has exactly one root plan at `<project>/mnemosyne/project-root/`. Name is **reserved**. Collapses the earlier `<project>/mnemosyne/plans/` container (which was always single-child). `knowledge/` stays as a sibling of `project-root/`, not inside it, preserving B's "plan membership is purely `plan-state.md`" invariant. Adoption check = `<project>/mnemosyne/project-root/plan-state.md` exists. Nested child plans nest arbitrarily under project-root.

### Filesystem-derivable data is never cached in metadata
If a value can be computed from the filesystem path or directory structure, it must not be stored in frontmatter or config. Filesystem is authoritative; metadata projections drift. Applies to qualified IDs, host-project names, dev-root paths, and any future candidate. See "Path-based qualified plan IDs, never stored" for the primary instance.

### Path-based qualified plan IDs, never stored
A plan's qualified ID is a pure function of its filesystem path: `strip_prefix(plan_path, "<vault>/projects/")`. Examples: `Mnemosyne/project-root`, `Mnemosyne/project-root/sub-F-hierarchy`. Never stored in `plan-state.md` frontmatter — storing would be a duplicate source of truth. Filesystem is authoritative; qualified IDs computed at read time. F's coordination amendment to B removes `plan-id`, `host-project`, `dev-root` from plan-state.md schema.

### Dispatch target resolution asymmetry
Same-project and cross-project dispatches use different mechanisms:
- **Same-project** → origin names the specific `target-plan: <qualified-id>`. Mnemosyne writes to the target's `Received` section mechanically. No LLM in the loop. Origin has enough local context to pick the right target.
- **Cross-project** → origin names only `target-project: <name>`. Mnemosyne spawns a **Level 2 routing agent** — fresh-context Claude Code session scoped to the target project's vault subtree (plans + source code), with authority to pick a specific target plan or reject with reasoning. Optional `suggested-target-plan:` hint the agent may override.
Origin doesn't have enough context to reason about foreign plans; Level 2 agent does. Context depth scales with decision specificity.

### Vault catalog replaces related-plans.md
`{{RELATED_PLANS}}` placeholder renamed to `{{VAULT_CATALOG}}`. Substituted content is the full vault catalog at `<vault>/plan-catalog.md` — every plan and expert, grouped by project, with 120-char descriptions and dispatch rules. `related-plans.md` file concept deleted entirely. Auto-regenerated on plan mutation and every phase-prompt render. Cached form committed to vault git as a human-visible "what's in my vault" dashboard in Obsidian.

### Description discipline: 120 character hard cap
Every plan and expert has a `description:` frontmatter field, ≤120 characters, enforced at load time (hard error on overflow). Keyword-dense, noun-phrase-led, no self-reference, no placeholders. Permanent scope declaration, not current-state description. Cap is load-bearing: forces LLM-skimmable catalog rhythm and prevents drift into verbose framings. Written at plan creation (brainstorm exit criterion), updated during reflect only if scope has genuinely drifted.

### Declarative routing with LLM-fallback learning loop
Routing rules live in `<vault>/routing.ex` — user-editable Elixir module with pattern-matched `defp route/2` clauses (starting with Elixir pattern matching; Erlog reserved for if expressiveness pressure arises). BEAM's native hot code reload makes edits take effect without daemon restart. Facts (concern keywords) are extracted from each message body by a small cheap LLM pass (Claude Haiku via sub-C adapter; sub-O ideal consumer for local models). When rules don't decide, Level 2 routing agent runs as fallback and may propose a new rule the user accepts into `routing.ex` — closing a learning loop where novel cases train the deterministic path.

### Prefer integration over reinvention
When scope overlaps with existing mature tools — Karpathy's LLM Wiki, Infonodus-style knowledge-graph tools, TUI frameworks, session/workflow/state-machine libraries, knowledge-storage formats, PTY wrappers, **OTP itself** — the default is adopt or integrate. Rationale: (1) reduces build surface; (2) eases adoption via familiar mental models; (3) avoids NIH tax. The BEAM commitment is the largest instance of this principle — the actor model, supervision, message passing, and distribution are all OTP primitives we would otherwise hand-roll badly. Every sub-project brainstorm must surface "what existing tool covers this ground, and why not use it?" Silent reinvention is not allowed.

**Cross-cutting infrastructure sub-projects pick the standard tool at every internal decision.** When scope is "design infrastructure everything else uses" (M is the exemplar; D, F, H, I face the same shape), after the architectural fork question lands, treat every remaining decision as "pick the standard tool" and bound custom code tightly.

### Hard errors by default
Soft fallbacks require explicit written rationale. Unexpected conditions, invariant violations, I/O failures, and ambiguous state fail hard with clear diagnostics. Documented exceptions must name the rationale in the design doc. Project-wide.

### Always-on instrumentation; tactical seeds disclaim framework scope
Measurement for acceptance gates or diagnostics is emitted unconditionally — no debug flag, no env var, no gated build. First instance: C's `SpawnLatencyReport`. **Tactical instrumentation must disclaim being a framework.** Purpose-built measurement is a tactical seed, not a metrics framework, and must not accrete framework-shaped scope creep. The observability framework lives in M, which owns the framework and sibling-backlog adoption coordination. **Staged migration of seeds into M's framework: parallel-emit + mechanical verification, never atomic cutover.** Three independently-reversible steps: (1) seed-owning sub-project's v1 ships its tactical writer unchanged; (2) M's v1 lands instrumentation at the same measurement points in parallel; (3) mechanical verification confirms M's data matches the seed within explicit tolerance, then the seed's writer is deleted.

### No slash commands in the harness — control forbidden, observation required
The harness runs as a pure worker with no user-facing command surface. Every user action flows through the client-daemon protocol. Three observation surfaces: chunk-level (assistant text, tool use, tool result), protocol-level (session lifecycle via stream-json), task-level (prompt-instructed sentinel strings detected by B's executor in assistant-text stream).

### Task-level completion via sentinel strings
Protocol-level "turn over" is `{"type":"result"}` in stream-json — the model stopped emitting tokens. Task-level "done with the work" requires the LLM's self-assessment — conflating the two causes premature phase transitions. Mechanism: each phase prompt ends with "when finished say READY FOR THE NEXT PHASE"; B's executor runs a sliding-buffer matcher over assistant-text. Sub-C detects both: `{"type":"result"}` for protocol-level, sentinel for task-level. Sentinel detection lives in B because sentinels are coupled to phase prompts.

### Obsidian is the committed maintenance/explorer UI
Every file format, directory layout, and cross-reference decision targets Obsidian: Dataview-friendly kebab-case YAML frontmatter, wikilinks for cross-references, tags as first-class metadata, directory structure for Obsidian's file tree, a Mnemosyne-provided `.obsidian/` template. V1 ships a maximally Obsidian-native baseline; K (Obsidian plugin client) enhances post-v1. The "explorers are load-bearing" capability remains; implementation is delegated to Obsidian. I's scope shrinks from "unified explorer framework" to "document which Obsidian features cover which data surfaces."

### Dedicated Mnemosyne-vault with symlinked per-project directories
`<dev-root>/Mnemosyne-vault/` (Mnemosyne-owned, its own `.git`, committed Obsidian vault) sits alongside project repos. Hosts Tier 2 global knowledge, expert declarations, routing rules, daemon config, and runtime state (staging, interrupts, ingestion events, mailboxes, daemon socket). Accesses per-project content via symlink: `<vault>/projects/<project-name>` → `<project>/mnemosyne/`. Plans stay in project repos (sovereign git); the vault is a view-over-symlinks. **Symlink approach validated** on macOS and Linux (spike passed 6/6, commit `98ef7db`) — no hard-copy fallback needed.

### UI/integration spikes use guivision + OCR evidence
Canonical pattern: run the assumption inside a GUIVisionVMDriver golden image, drive GUI exclusively through `guivision` CLI, capture evidence as `guivision screenshot` + `guivision find-text` artifacts at `{{PROJECT}}/tests/fixtures/<spike-name>/results/<platform>/`. OCR evidence sufficient for binary pass/fail. Future spikes for I, K, L, and B's v1 acceptance test follow this pattern. SSH/rsync/VNC direct paths are out.

### Per-project `mnemosyne/` replaces `LLM_STATE/` + `knowledge/`
Each project has `<project>/mnemosyne/` containing `project-root/` (nested plan tree) and `knowledge/` (Tier 1). Lowercase `mnemosyne/` disambiguates from `Mnemosyne/` repo and `Mnemosyne-vault/`. A directory is a plan iff it contains `plan-state.md`. `StagingDirectory::render` refuses descent into subdirectories containing `plan-state.md`, enforcing one-plan-per-staging. G renames: `LLM_STATE/` → `mnemosyne/project-root/`, `knowledge/` → `mnemosyne/knowledge/`.

### Phase cycle inside PlanActor
Four-phase cycle (work → reflect → compact → triage) runs **inside a PlanActor GenServer**, not as a standalone process main loop. B's `PhaseRunner` is the state machine embedded in the actor. Phase transitions driven by `{:run_phase, _}` messages from attached clients. B's `StagingDirectory`, placeholder substitution, `plan-state.md` management, and phase-boundary hooks all preserved. Compact is strictly lossless; reflect is the only lossy-pruning phase. F's `DispatchProcessor` and `QueryProcessor` run at phase exit via B's phase-boundary hooks.

### TUI stays in Rust as separate client binary
TUI (`mnemosyne-tui`) is a standalone Rust binary using `ratatui` + `tokio` + `serde_json`. Connects to the daemon over a local Unix socket at `<vault>/runtime/daemon.sock` speaking NDJSON. No Rustler / no NIFs — the socket is the integration boundary. Clean separation: daemon owns state and reasoning, TUI owns rendering and input. Multi-client is free: same protocol accepts Rust TUI, future Obsidian plugin, web UI, headless scripts.

### Self-containment from `LLM_CONTEXT/` via embedded prompts
The daemon has zero runtime dependency on `LLM_CONTEXT/`. Vendored phase files (`phases/work.md`, `phases/reflect.md`, `phases/compact.md`, `phases/triage.md`), fixed-memory files (`fixed-memory/memory-style.md` plus language guides), and `create-plan.md` live at `{{PROJECT}}/prompts/` and ship with the daemon as an Elixir embedded resource. Surfaced through `{{PROMPTS}}` placeholder substituted by `StagingDirectory::render`. Customisation is v2; v1 enforces "use what ships."

### `memory-style.md` is the single source of truth for memory entry rules
Both reflect (lossy/pruning) and compact (lossless rewrite) read `fixed-memory/memory-style.md`. The rules (assertion register, one fact per entry, cross-reference over re-explanation, short headings, drop session numbers/dates) are upstream of every plan's memory curation and must not drift between reflect and compact.

### Human and LLM are co-equal actors
Both reflect, triage, and curate. The orchestrator supports human-driven flows as first-class. Enforcement: B's `PhaseExecutor` trait with `LlmHarnessExecutor` and `ManualEditorExecutor`, both against the same state machine. Sub-J folded into B via this trait chokepoint. Every F mechanism (dispatches, queries, routing rule edits) must work with either a human or an LLM as the driver.

### Explorers are the accountability substrate for auto-absorb
Post-session auto-ingestion is safe only because explorers give humans full-CRUD over everything absorbed. Without explorers, auto-ingestion is unsafe. Scope: knowledge (Tier 1/2), plan state, session artifacts, ingestion provenance. Obsidian is the v1 explorer.

### Mnemosyne is an LLM client via embedded harness adapter
The daemon spawns internal reasoning sessions (fact extraction, Level 2 routing, ingestion analysis, reflection distillation) plus child harness sessions for plans. Internal sessions use C's adapter, making C a real first-class abstraction. Requirement on C: configurable tool profiles at spawn time so internal sessions can be sandboxed. Ingestion Stage 5 becomes dispatch-to-experts (F/sub-N integration).

### TheExperimentalist is retired and deleted
Conceptual scope met by the backlog/reflect/triage cycle plus plan hierarchy. Git-branching was the wrong abstraction — branches model parallel state, but exploratory development needs temporal structure with reflection points, which the phase cycle provides.

### LLM_CONTEXT merges into Mnemosyne
Mnemosyne becomes the single user-facing tool for plans, phases, knowledge, and curation. `LLM_CONTEXT/` retires once fully replaced. Until then, LLM_CONTEXT and its four dependents (APIAnyware-MacOS, GUIVisionVMDriver, Modaliser-Racket, RacketPro) continue unchanged.

### Mnemosyne owns all knowledge — Tier 1 and Tier 2
Per-project knowledge in Tier 1 (`<project>/mnemosyne/knowledge/`). Cross-project knowledge in Tier 2 (`<vault>/knowledge/`). Plan memory (`<plan>/memory.md`) is provisional scratch for in-flight reflections, not durable knowledge. Knowledge is **consultative** — accessed through ExpertActor queries, not loaded into plan sessions directly.

### Global knowledge store at user-specified absolute path
First-class git-tracked dev asset visible alongside project repos. User-specified path (default `~/Mnemosyne-vault/`), discovered via explicit precedence chain. DEV_ROOT framing discarded — no implicit dev-root.

### Vault root must be a real directory, not a symlink
`<vault>/projects/<name>/` entries are symlinks created relative to the vault root. If the root itself were a symlink, relative symlink targets would resolve incorrectly. Boot-time `File.lstat!/1` check rejects symlinked vault roots.

### Vault discovery is explicit — flag → env → config → hard error
Every daemon startup resolves the vault: `--vault <path>` → `MNEMOSYNE_VAULT` → user config at `dirs::config_dir().join("mnemosyne/config.toml")` → hard error. No walk-up, no implicit dev-root, no vault registry. Single-vault-per-machine. Detail in A's design doc §A1.

### Vault identity via `mnemosyne.toml` schema-versioned marker
A directory is a vault iff `mnemosyne.toml` at root contains a parseable `[vault]` table with `schema_version`. Missing file, parse error, or unsupported version are hard errors. The same file hosts optional vault-level overrides (language profiles, context mappings). Verified every invocation via `verify_vault`. **`daemon.toml` is a separate file** for runtime configuration knobs (socket paths, timeouts, harness settings). The two-file split is load-bearing: swapping daemon versions or reconfiguring harnesses must not touch vault identity.

### v0.1.0 has no real users — legacy paths deletable
Consequences: (1) no `migrate` subcommand; (2) hardcoded `~/.mnemosyne/` in old Rust CLI deleted outright; (3) legacy YAML config format deleted; (4) previous Rust CLI scope is subsumed by the daemon + TUI split — the bulk of the Rust codebase is written off in the BEAM pivot (sunk cost was tiny — less than a day of implementation effort).

### Knowledge ingestion via post-session inspection
Mnemosyne reads plan outputs (`memory.md`, `session-log.md`) after sessions end or at phase boundaries. The LLM never invokes CLI to "promote." E's pipeline stages: extract → classify → contradict → score → **dispatch to experts** (Stage 5, post-F amendment). Experts review candidate entries in fresh context and absorb, reject, or cross-link them.

### Harness adapter layer abstracts multiple harnesses
Each adapter handles spawn, prompt-passing, output capture, and lifecycle. **V1 ships Claude Code adapter only** in Elixir (pipes via `erlexec`; spike validated). Codex and Pi follow post-v1. Sub-O (mixture of models, v1.5+) adds multi-adapter support and per-actor model selection. Serves user-facing plan sessions and internal reasoning sessions. Protocol: bidirectional stream-json over stdio pipes. Process-group termination is a v1 requirement.

### Hybrid `:telemetry` + typed-event-struct is the project-wide pattern
BEAM equivalent of the earlier Rust "hybrid `tracing` + typed-event-enum" pattern. Typed `Mnemosyne.Event.*` structs at the boundary (exhaustive-match safety for CLI display, Obsidian ingestion, TUI event rendering, E's pipeline); `:telemetry` for transport, spans, third-party integration below the boundary. Custom code bounded to minimal event definitions + a small `:telemetry` handler module. Generalizes beyond observability to any component with the same tension.

`:telemetry` silently detaches handlers that raise — every `Handler.handle_event/4` body must be wrapped in `try/rescue` and log errors via `Logger` directly (not through M). Without this, any subscriber bug permanently blinds the daemon. Rust's `tracing-subscriber::Layer` logs but does not detach; BEAM does not share this safety net.

`:telemetry_metrics` is reporter-independent by design: the same `counter(...)` / `distribution(...)` definitions feed any reporter attached to the supervisor tree. Load-bearing criterion for choosing `:telemetry_metrics` over `prom_ex` — `prom_ex` bundles Phoenix/Ecto/Broadway plugins Mnemosyne does not consume and ships its own Plug endpoint, dashboards, and Grafana integration. Switching or adding reporters is purely additive.

### `:pg` handles intra-daemon multi-client fan-out
`:pg` (OTP stdlib since OTP 23) provides process-group membership for TUI session fan-out without a `Phoenix.PubSub` dependency. Per-client backpressure: the daemon's per-client session process wraps `send/2` in a `try/catch` that increments a drop counter on full mailbox.

### Behaviour chokepoints enable staged extension
Every sub-project that may need a second implementation strategy must identify its chokepoint behaviour before committing to v1. Two instances: `Mnemosyne.ExpertRetrieval` (lets sub-Q drop in a vector strategy without touching sub-N internals) and `Mnemosyne.HarnessAdapter` (lets sub-O add adapters without touching sub-C internals). Brainstorm exit criterion: "what's the chokepoint here?" answered explicitly.

### Cross-cutting brainstorms own adoption stubs
When a design imposes adoption work on siblings, the brainstorm must produce concrete stubs in every affected sibling backlog before stopping. Triage is too late — the brainstorm's fresh context is lost. F is the most recent and largest instance: the brainstorm commits to pushing amendment tasks for A, B, C, D, E, G, H, I, M plus new brainstorm tasks for N, O, P to the orchestrator backlog via F's own dispatch mechanism.
**Symmetric obligation:** Sub-projects brainstormed after a cross-cutting framework must pull queued stubs into their own backlogs during brainstorm, not wait for triage.

**Corollary:** When multiple upstreams are rewritten before a cross-cutting amendment, the amendment becomes a collation task, not a design task. The sealed struct/event set writes itself from already-landed producer contracts. Adoption-stub discipline has a compounding return: each pre-rewritten upstream reduces the design load of every downstream cross-cutting amendment.

### Cross-plan coordination via subagents and vault catalog
A plan's LLM phases must not read another plan's files. Cross-plan awareness arrives via `{{VAULT_CATALOG}}` (descriptions only, never content) and dispatched subagents. Level 2 routing agent is the primary mechanism for cross-project reasoning. This is the runtime complement to the adoption-stub discipline.

### Brainstorming methodology
Three linked disciplines that compound:

**Lock one fork per clarifying question.** Structure brainstorms as a linear sequence where each question locks a design fork and the answer feeds the next. By design-section presentation, content is determined; presentation becomes verification.

**Test scope assumptions early.** When generating migration/compatibility questions, check whether those scopes are real first. "Are there real v0.1.0 users?" dropped a CLI subcommand, eliminated eight hardcoded paths, killed one YAML config format. F's "Is sunk cost real?" drove the BEAM pivot.

**Re-read memory entries before revising.** Entries are dense and easy to misread under reasoning pressure. First action on a "let me revise that" impulse: re-read the entry to confirm current framing.

**Surface research threads for fresh slots, not inline absorption.** When clarifying questions spawn cross-cutting research with substantial prior literature (ontologies, vector stores), create a new sub-project stub and proceed. Inline absorption bloats v1 scope and loses the research context. Sub-Q and sub-R surfacing from sub-N are the canonical instances.

### Amendment tasks rewrite specs inline, not as supersede layers
When a pivot invalidates large portions of a design doc, rewrite §1-§N with fresh content. Do not append a "§X is authoritative where earlier sections conflict" amendment block — it forces readers to context-switch between incompatible framings and stale content drifts from reality. History belongs in the Decision Trail appendix, with corrections inline at the original question entry. Cross-sub-project requirements re-evaluate on pivot (drop dead requirements, restate survivors in new terms) rather than translate mechanically. Multiple amendments stack in a single rewrite pass — each gets its own Decision Trail entry (Qn+1, Qn+2, …) rather than being folded together. The discipline composes recursively: implementation-time readiness gates (e.g. sub-F's Task 0) mandate inline rewrites of their own consumed specs when upstream interfaces drift, applying the same principle at every layer. Downstream specs cite upstream by section number (e.g. A cites C's §4.5 and B's §4.2/§4.4); section-number stability across rewrites is load-bearing for accumulated cross-references. Validated at scale: sub-A at 1242 lines absorbing two shifts, sub-C at 1186 lines with one shift, sub-B at 2296 lines absorbing three simultaneous shifts, sub-M at 870 lines absorbing five simultaneous upstreams (A, B, C, E, F).

### Files as the durable substrate
Everything non-transient is a file. The daemon is an orchestrator, not a state store. Daemon restart rebuilds all actor state from files. Team mode (sub-P, v2+) becomes "multiple daemons over the same filesystem, syncing through git." Individual actor crashes don't destroy state. Files-as-substrate composes: every "where does this live?" question resolves to a file, and every "what happens on crash?" question resolves to replay from files.

### memory.md and backlog.md drift is load-bearing
Both files are consumed by LLM phases every cycle; stale content propagates misinformation into fresh context. `session-log.md` is immune (no LLM phase reads it). Fix drift immediately, especially after upstream changes affecting multiple plans.

### LLM_CONTEXT punch-list 1-3 landed (stop-gap)
Small-fix version: `{{PROJECT}}` placeholders + work prompt substitution instructions. NOT the larger restructure, deferred/subsumed by the orchestrator. Landed in APIAnyware-MacOS and GUIVisionVMDriver.

### Sub-C uses pipes-only erlexec, not PTY
Stream-json is stdio NDJSON, not a terminal interaction. PTY is only needed for interactive TUI (slash commands, arrow keys, ANSI redraws), which "No slash commands in the harness" explicitly rules out. Pipes-only erlexec opts (`[:monitor, :stdin, {:stdout, self()}, {:stderr, self()}, :kill_group, {:kill_timeout, 1}]`) cleanly drive the claude CLI end-to-end. `:stdin` bare atom is required — erlexec defaults stdin to `:null`. The Rust PTY-wrapper fallback is unnecessary.

### erlexec is a C++ port program, not a NIF
`exec-port` runs as a separate OS process handling PTY allocation, signals, and process groups outside BEAM schedulers. No NIF scheduler risks. This is why erlexec can safely use `ptrace`, `setreuid`, and process groups.

### cmux hooks require `--setting-sources` for clean stream-json
User-global cmux SessionStart hooks inject ~10KB of spurious JSON into stream-json output. `--setting-sources project,local` suppresses them. Required for all daemon-spawned claude sessions. Additionally, `--no-session-persistence` prevents cmux session-id injection.

### ExpertActor scope spans both tiers with explicit opt-in
Experts can read both Tier 2 (`<vault>/knowledge/`) and Tier 1 (`<project>/mnemosyne/knowledge/`) via `scope.tier2` and `scope.tier1` globs in the declaration. Literal `*` in the first segment of a `tier1` glob means "any project" — no implicit cross-project inclusion. Grep `<vault>/experts/*.md` for `tier1:` to audit cross-project scope. An expert that omits `scope.tier1` is Tier-2-only. Per-project opt-in is the cross-project-leakage control mechanism.

### Hybrid knowledge ownership: experts own Tier 2, Tier 1 is plan-owned
Experts have exclusive write authority over dedicated Tier 2 directories (resolved from the first entry of `scope.tier2`). Tier 1 is plan-owned read-only from the expert's perspective. Stage 5 candidates that belong in Tier 1 are plan-owned writes (sub-E handles directly); Tier 2 candidates flow through experts. Clean three-way Stage 5 decision: Tier 1 = plan-owned, Tier 2 = expert-owned, orphan = `knowledge/uncategorized/`.

### Stateless dialogue turns over ETS-backed registry
Expert dialogues use stateless turns over a singleton `Mnemosyne.Expert.DialogueRegistry` backed by ETS. Every turn spawns a brand-new fresh-context session; the actor does NOT hold a long-lived harness across turns. Dialogue TTL is idle-based (default 30 min, resets on activity). Daemon restart wipes the table — consumers get `:dialogue_not_found_or_expired` and fall back to a fresh `ask_expert`. Audit trail lives in sub-M's event log, NOT in a parallel dialogue file. Dialogue state is transient; belongs in ETS memory, not on disk. This is a specific application of "files as the durable substrate" — non-transient info goes on disk, transient in-flight state stays in memory.

### Tag-based exact-string scope matching for Stage 5
Stage 5 scope matching is pure set-intersection on frontmatter `tags:` fields. Exact-string matching only in v1 — `rust` ≠ `rustlang`, no stemming, no case-insensitivity. Orphan candidates (zero matching experts) bypass expert fan-out and write directly to `<vault>/knowledge/uncategorized/`. This is the load-bearing v1 simplification that sub-R will replace with a richer resolver behind the same `ScopeMatcher` interface.

### Parallel fan-out with each expert writing its own file for Stage 5
Stage 5 dispatches `%ExpertAbsorbCandidate{}` messages in parallel to every tag-matching expert, and each expert that returns `READY ABSORB` writes the candidate file into its own Tier 2 directory itself. Physical duplication across directories is accepted — two experts absorbing the same candidate produce two files with matching `ingestion-event-id` in their provenance frontmatter. Wikilinks between them are NOT auto-inserted by sub-N v1 (that's human triage or a future polish task). `READY CROSS_LINK <expert-id>` is interpreted as a rejection-with-suggestion; the sub-E collector second-round dispatches non-recursively (max depth 2). Contentful disagreement (absorb + reject with non-trivial reason) surfaces `%ExpertConflict{}` events for human review.

### Two explicit timers for sub-N: retrieval vs session
Retrieval pipeline timeout (5 seconds) is a **structural sanity check** on the ripgrep pipeline — not a reasoning budget. The per-turn session timeout (5 minutes default, configurable in `daemon.toml` under `[experts] turn_timeout_seconds`) bounds actual LLM reasoning time. The dialogue TTL (30 minutes default) is an **idle** timer that resets on every successful turn, so a dialogue whose turns each take 4 minutes of reasoning can run for hours. Three distinct concepts; do not collapse them. This pattern generalizes: deterministic work gets tight structural caps; LLM work gets generous reasoning budgets; dialogue/session idle timers are independent again.

### Keyword + section-aware retrieval behind behaviour chokepoint
V1 ships exactly one retrieval strategy, `Mnemosyne.ExpertRetrieval.KeywordSection`, behind an `@behaviour Mnemosyne.ExpertRetrieval` chokepoint. Uses ripgrep over scope globs + section-aware scoring with module-level constant weights (1.0 × frontmatter tags, 0.6 × headings, 0.4 × first paragraph, 0.2 × body, 0.1 × recency with 30-day half-life). Tuning weights requires a code change, not per-user config — this prevents per-vault drift on load-bearing scoring parameters. Future strategies (semantic via vector store — sub-Q) drop in behind the chokepoint without touching sub-N internals.

### Vector-store is cross-cutting Mnemosyne-wide infrastructure, not a sub-N internal
A vector database has consumers beyond expert retrieval — sub-F's vault-catalog search, sub-E's Stage 3 contradiction detection, plan actors searching Tier 1/2 knowledge, and sub-O's local-model infrastructure all benefit. Sub-Q is therefore a dedicated brainstorm on vector-store infrastructure as a shared daemon service, not a sub-N-internal retrieval strategy. The `ExpertRetrieval` behaviour chokepoint is sub-N's hook for consuming sub-Q's output. Reserved for v1.5+; does not block v1.

### Tag ontology is a research sub-project, not inline with sub-N
Tag vocabulary drift (`rust` vs `rustlang` vs `Rust`) is inevitable as the expert set grows; the correct answer is an ontology — but that's real research (formal ontologies vs folksonomies, synonym handling, hierarchy, polysemy) with 40 years of literature. Sub-R is a dedicated research brainstorm. Cross-cutting consumers: sub-N `ScopeMatcher`, sub-E Stage 2 classification, sub-F fact extraction, sub-Q vector-store, sub-H TUI presentation. Sub-A's vault layout has a reserved `tag-vocabulary.md` path for sub-R's future use. Reserved for v1.5+; does not block v1.

## Sub-projects

Seventeen sub-projects (sixteen active, one obsolete). Brainstorms complete: A, B, C, E, F, M, N (seven of sixteen). Each produces a design doc at `{{PROJECT}}/docs/superpowers/specs/` and a sibling plan at `{{PROJECT}}/LLM_STATE/`.

| ID | Sub-project | Complexity | Status | Notes |
|----|-------------|-----------|--------|-------|
| A  | Global knowledge store at user-specified path | Small-med | **done** (F amendment absorbed) | Design doc `specs/2026-04-13-sub-A-global-store-design.md`. Explicit vault discovery, `mnemosyne.toml` marker, `init`/`init --from`/`config use-vault`/`adopt-project` commands. F amendment absorbed via inline rewrite: §A1–§A10 re-cast for the Elixir daemon, §A4 layout picks up sub-F surfaces (`daemon.toml`, `routing.ex`, `plan-catalog.md`, `experts/`, `runtime/{daemon.sock,daemon.lock,daemon.pid,mailboxes/}`), `project-root/` rename absorbed in §A10 walk-up, daemon singleton lock replaces per-plan locks. Q1–Q5 preserved verbatim with correction notes; Q6 (BEAM pivot) and Q7 (sub-F commitments) added in Appendix A. |
| B  | Phase cycle | Medium | **done** | Design doc `specs/2026-04-12-sub-B-phase-cycle-design.md`. Produced: hard errors, no slash commands, Obsidian explorer, vault+symlinks, per-project `mnemosyne/`, embedded prompts. F amendment absorbed via inline rewrite: PhaseRunner inside PlanActor, `plan-state.md` schema pruned, `{{VAULT_CATALOG}}` replaces `{{RELATED_PLANS}}`, four-phase cycle first-class. Downstream implementation task list still carries pre-pivot Rust framing (gate task added for next work phase). |
| C  | Harness adapter layer | Med-large | **done** (F amendment absorbed) | Design doc `specs/2026-04-13-sub-C-adapters-design.md`. V1 Claude Code only. F amendment absorbed: spec rewritten inline for Elixir/erlexec (§1-§11 fresh, no supersede layer). Tool-call boundary for in-session Queries (§4.5) now concrete. Sub-C sibling plan backlog still has pre-pivot Rust task list (discrete sub-C work). |
| D  | Concurrency (scope collapsed by F) | Small | not started | F's daemon commitment replaces per-plan advisory locks with OTP mailbox serialization. D's scope collapses to: daemon singleton lock + advisory file locks for external-tool coordination (Obsidian, git). Much smaller brainstorm. |
| E  | Post-session knowledge ingestion | Medium | **done** (pending F amendment) | Design doc (`501c15c`). **F amendment pending**: Stage 5 becomes dispatch-to-experts instead of direct store write. |
| F  | Plan hierarchy + actor model + dispatch + declarative routing | **Large** | **done** (sibling plan scaffolded) | Design doc `specs/2026-04-14-sub-F-hierarchy-design.md`. Committed: persistent BEAM daemon, two sealed actor types, two message types, `project-root` convention, path-based qualified IDs, dispatch asymmetry, vault catalog, Datalog-ish declarative routing with Level 2 fallback, Elixir runtime, Rust TUI client. Sibling plan at `LLM_STATE/sub-F-hierarchy/` with 28 implementation tasks + Task 0 readiness gate; implementation blocked on sub-B + sub-C task-list rewrites and sub-A + sub-M amendment absorption. |
| G  | Migration: LLM_CONTEXT + v0.1.0 transition | Medium | not started | Parallel/ongoing. Owns rename (`LLM_STATE/` → `mnemosyne/project-root/`, `knowledge/` → `mnemosyne/knowledge/`) and `phase.md` → `plan-state.md`. **F amendment pending**: migration gains a "start the daemon" step + Rust CLI → Elixir daemon + Rust TUI split awareness. |
| H  | Fold 7 Claude Code skills | Small-med | not started | Mechanical; depends on B. Skills become attached-client TUI actions. **F amendment pending**: framed as client-attached actions, not standalone tools. |
| I  | Obsidian coverage (re-scoped) | Small-med | not started | Document which Obsidian features cover which data surfaces. **F amendment pending**: Obsidian as daemon client, not just file viewer. |
| J  | Human-mode phase affordances | N/A | **obsolete** | Folded into B as `ManualEditorExecutor`. |
| K  | Obsidian plugin client | Medium | not started (v1.5+) | Daemon-as-backend makes K more attractive. Client of F's Unix socket protocol. Re-evaluate at v1 scope cut. |
| L  | Obsidian terminal plugin spike | Small | not started | Prerequisite for K. Independent — can start any time. |
| M  | Observability framework | Med-large | **done** (F amendment absorbed) | Design doc `specs/2026-04-13-sub-M-observability-design.md`. F amendment absorbed via inline rewrite: §1–§20 re-cast onto `:telemetry` + sealed `Mnemosyne.Event.*` struct set + four GenServer subscribers (`RingBuffer`, `JsonlWriter`, `TuiBridge`, `Metrics`) + `Mnemosyne.Observability.Handler` attached via `:telemetry.attach_many/4`. Sealed set: 20+ Elixir structs grouped by producer (B/C/F/E/A + M escape hatches). §6 metric catalogue: 23 definitions via `Telemetry.Metrics.*`. Q1–Q5 preserved verbatim with correction notes; Q6 (BEAM pivot) and Q7 (reporter selection — `ConsoleReporter` + `SnapshotReporter` v1, `:telemetry_metrics_prometheus` v1.5, OpenTelemetry v2) added. Key failure mode: `:telemetry` detaches raising handlers; §5.2 mandates `try/rescue` logging via `Logger` (Risk 6). Downstream Rust task-list carries pre-pivot framing; Task 0 gate mirrors sub-B/sub-C discipline. |
| **N** | **Domain experts** | **Medium** | **done** | Design doc `specs/2026-04-15-sub-N-domain-experts-design.md` (1104 lines). ExpertActor implementation behind F's type hole, stateless dialogue over ETS, tag-based exact-string scope matching, keyword+section retrieval, hybrid knowledge ownership (experts own Tier 2, Tier 1 plan-read-only), parallel-fan-out Stage 5 with each expert writing its own absorb. Default expert set swap: `distributed-systems-expert` dropped, `elixir-expert` added for dogfooding. Sibling plan at `LLM_STATE/sub-N-domain-experts/` with 29 tasks across 9 phases; Tasks 1–15 can start immediately (pure Elixir + fixtures + early deliverable for sub-E amendment), Task 0 gates Task 16+ on sub-F delivering `Mnemosyne.Actor` behaviour + `ActorSupervisor` child-spec API. |
| **O** | **Mixture of models** | **Medium** (new) | not started (v1.5+) | **New sub-project added by F, reserved for v1.5+**. Multi-adapter harness layer, per-actor model selection, local-model adapters, cost telemetry. F reserves `[harnesses.*]` config and `model:` actor field. |
| **P** | **Team mode** | **Large** (new) | not started (v2+) | **New sub-project added by F, reserved for v2+**. Multi-daemon transport via BEAM distribution or custom TCP, peer discovery, cross-daemon auth, shared-vault conflict resolution. F reserves `[peers]` config and `<peer>@<qualified-id>` syntax. |
| **Q** | **Vector-store infrastructure** | **Medium** (new, research) | not started (v1.5+) | **New sub-project surfaced during sub-N brainstorm**. Vector DB as Mnemosyne-wide capability, not a sub-N-internal retrieval strategy. Consumers: sub-N expert retrieval, sub-F vault-catalog search, sub-E Stage 3 contradiction detection, plan actors, sub-O local-model infrastructure. Research-heavy brainstorm: embedding model choice, storage layout, update policies, crash recovery, first-time setup UX. Reserved for v1.5+; `Mnemosyne.ExpertRetrieval` behaviour chokepoint is the integration hook. |
| **R** | **Knowledge ontology** | **Medium** (new, research) | not started (v1.5+) | **New sub-project surfaced during sub-N brainstorm**. Addresses tag-vocabulary drift (`rust` vs `rustlang` vs `Rust`) across sub-N scope matching, sub-E Stage 2 classification, sub-F fact extraction, sub-Q vector-store, sub-H TUI presentation. Research-heavy brainstorm: formal ontology vs folksonomy, synonym handling, hierarchy, polysemy, vocabulary evolution. Sub-A's vault layout has a reserved `tag-vocabulary.md` path for sub-R's future use. Sub-N's `ScopeMatcher` is the integration hook. Reserved for v1.5+; does not block v1. |

### Recommended sub-project ordering (post-N)

Brainstorms complete: A, B, C, E, F, M, N. Immediate pipeline: sub-E amendment (unblocked — start via sub-N's Task 15 early-deliverable PR) → remaining brainstorms D, G, H, I → sub-O (v1.5) → sub-Q (v1.5, research) → sub-R (v1.5, research) → sub-P (v2).

Critical next steps: **(1)** Sub-E amendment — Stage 5 dispatch-to-experts. Can start immediately using sub-N's design doc §6 interface contracts; sub-N's own Task 15 early-deliverable PR (scope matcher + message structs + verdict structs + event structs) makes the amendment codeable against real types while the rest of sub-N implementation proceeds in parallel. **(2)** Sub-B and sub-C downstream task-list rewrites against their rewritten design docs (gate tasks in place in both sibling plans; they are the two remaining gate conditions for sub-F's Task 0). **(3)** Sub-F's Task 0 readiness check once (2) completes — then Task 1+ implementation can begin. **(4)** Remaining brainstorm/amendment tasks for D, G, H, I run in any order once capacity permits; all four produce scope-framing documents for future implementation. Sub-M's own downstream task-list rewrite is internal to sub-M and not on F's critical path. All four orchestrator-level amendment tasks (A, B, C, M) are now absorbed and sub-N brainstorm is done. Sub-Q and sub-R are research sub-projects reserved for v1.5+; neither blocks v1 shipping.

## Open questions

### Pattern-matched Elixir routing vs Erlog
Will user-authored `defp route/2` clauses be expressive enough, or will Erlog (embedded Prolog) be needed? Unknown until real users write rules. V1 ships with pattern matching; Erlog reserved for v1.5+ if the need arises.

### Default expert set resolved by sub-N brainstorm
Six starter expert declarations: `rust-expert`, `elixir-expert`, `research-expert`, `software-architect`, `obsidian-expert`, `ffi-expert`. `distributed-systems-expert` dropped (scope absorbed by `software-architect`); `elixir-expert` added for dogfooding (Mnemosyne is Elixir-on-BEAM). Authoring in sub-N's Task 8 against design doc §3.5/§3.1 template.

### Team-mode design
Solo-developer by design at v1 (A's single-vault discovery, F's single-daemon architecture, E's sequential ingestion). Multi-developer shared vault needs network transport (sub-P's scope), conflict resolution, curation workflow changes, possibly per-user identity. F reserves the schema hooks (`[peers]`, `<peer>@<id>`) but doesn't wire them. Sub-P brainstorm at v2 milestone.

### Fate of 7 Claude Code skills
H's decision. Options: fully replaced by TUI attached-client actions, kept as legacy plugin during transition, or promoted to daemon commands. Likely a mix.

### What in `LLM_CONTEXT/` survives the merge
G's decision. `coding-style.md` and `coding-style-rust.md` are referenced by APIAnyware-MacOS work prompts. Could move into Mnemosyne's embedded prompts, a separate docs repo, or stay until LLM_CONTEXT retires.

### Which Obsidian features cover which data surfaces
I's work (re-scoped). "Which plugin + Dataview query covers each surface (Tier 1, Tier 2, plan state, sessions, ingestion provenance), and what `.obsidian/` template ships by default." Undo/history collapses to Obsidian's file history + vault git.

## Constraints / non-goals

### Non-disruption is mandatory
`LLM_CONTEXT/` and its four dependents (APIAnyware-MacOS, GUIVisionVMDriver, Modaliser-Racket, RacketPro) keep working unchanged. Mnemosyne is built alongside. Migration only when demonstrably equivalent or better.

### Bootstrap discipline
This plan runs on existing LLM_CONTEXT machinery. No assumed features beyond what LLM_CONTEXT supports.

**Dogfooded improvements may land back into LLM_CONTEXT** when: (1) additive documentation only; (2) four dependents absorb with zero migration; (3) flows into v1 when B's embedded prompts vendor the canonical files.

### Mnemosyne v0.1.0 sunk cost is tiny
The previous Rust CLI implementation represented less than a day of implementation effort. F's BEAM commitment writes off: the CLI subcommand wiring (~a few hundred lines), the knowledge store read/write glue. The evaluation framework (`eval/`) stays as Rust — it's a standalone CLI over knowledge files. The Markdown+YAML knowledge format stays. Every design-level decision from A/B/C/E/M stays.

### Never bulk-stage in APIAnyware-MacOS or GUIVisionVMDriver
Both repos carry uncommitted WIP on `main`. Any task must `git add` specific paths — never `git add -A`, `git add .`, or `git commit -a`.

## Origin

Plan scope evolved from LLM_CONTEXT punch-list fixes to Mnemosyne as full orchestrator with actor-daemon on BEAM. Bootstrapped on LLM_CONTEXT machinery per user direction. V1 architecture documented in `docs/architecture.md` and `docs/superpowers/specs/2026-04-14-sub-F-hierarchy-design.md`.
