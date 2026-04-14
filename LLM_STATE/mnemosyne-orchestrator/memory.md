# Memory — Mnemosyne Orchestrator

Drives the merge of LLM_CONTEXT functionality into Mnemosyne, transforming it into a harness-independent LLM orchestrator owning plan management, the work cycle, knowledge, harness session control, and inter-plan routing. V1 architecture committed to a persistent BEAM daemon hosting plan and expert actors.

## Stable architectural decisions

### Fresh LLM context is a first-class design goal
Every LLM-using component must use many short, fresh sessions rather than one long accumulating session. Context rot (drift, noise, stale assumptions) is a primary failure mode; phase cycle session boundaries are a feature, not a cost. B's phase boundaries, C's harness lifecycle, E's ingestion triggers, F's Level 2 routing agents, and N's expert consultations must default to discrete fresh-context invocations with explicit state handoff through files.

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
Protocol-level "turn over" means the model stopped emitting tokens. Task-level "done with the work" requires the LLM's self-assessment — conflating the two causes premature phase transitions. Mechanism: each phase prompt ends with "when finished say READY FOR THE NEXT PHASE"; B's executor runs a sliding-buffer matcher over assistant-text. Sentinel detection lives in B because sentinels are coupled to phase prompts.

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

### Vault discovery is explicit — flag → env → config → hard error
Every daemon startup resolves the vault: `--vault <path>` → `MNEMOSYNE_VAULT` → user config at `dirs::config_dir().join("mnemosyne/config.toml")` → hard error. No walk-up, no implicit dev-root, no vault registry. Single-vault-per-machine. Detail in A's design doc §A1.

### Vault identity via `mnemosyne.toml` schema-versioned marker
A directory is a vault iff `mnemosyne.toml` at root contains a parseable `[vault]` table with `schema_version`. Missing file, parse error, or unsupported version are hard errors. The same file hosts optional vault-level overrides (language profiles, context mappings). Verified every invocation via `verify_vault`.

### v0.1.0 has no real users — legacy paths deletable
Consequences: (1) no `migrate` subcommand; (2) hardcoded `~/.mnemosyne/` in old Rust CLI deleted outright; (3) legacy YAML config format deleted; (4) previous Rust CLI scope is subsumed by the daemon + TUI split — the bulk of the Rust codebase is written off in the BEAM pivot (sunk cost was tiny — less than a day of implementation effort).

### Knowledge ingestion via post-session inspection
Mnemosyne reads plan outputs (`memory.md`, `session-log.md`) after sessions end or at phase boundaries. The LLM never invokes CLI to "promote." E's pipeline stages: extract → classify → contradict → score → **dispatch to experts** (Stage 5, post-F amendment). Experts review candidate entries in fresh context and absorb, reject, or cross-link them.

### Harness adapter layer abstracts multiple harnesses
Each adapter handles spawn, prompt-passing, output capture, terminal/PTY, and lifecycle. **V1 ships Claude Code adapter only** in Elixir (PTY via `erlexec`, pending BEAM spike validation). Codex and Pi follow post-v1. Sub-O (mixture of models, v1.5+) adds multi-adapter support and per-actor model selection. Serves user-facing plan sessions and internal reasoning sessions. Protocol: bidirectional stream-json. Process-group termination is a v1 requirement.

### Hybrid `:telemetry` + typed-event-struct is the project-wide pattern
BEAM equivalent of the earlier Rust "hybrid `tracing` + typed-event-enum" pattern. Typed `Mnemosyne.Event.*` structs at the boundary (exhaustive-match safety for CLI display, Obsidian ingestion, TUI event rendering, E's pipeline); `:telemetry` for transport, spans, third-party integration below the boundary. Custom code bounded to minimal event definitions + a small `:telemetry` handler module. Generalizes beyond observability to any component with the same tension.

### Cross-cutting brainstorms own adoption stubs
When a design imposes adoption work on siblings, the brainstorm must produce concrete stubs in every affected sibling backlog before stopping. Triage is too late — the brainstorm's fresh context is lost. F is the most recent and largest instance: the brainstorm commits to pushing amendment tasks for A, B, C, D, E, G, H, I, M plus new brainstorm tasks for N, O, P to the orchestrator backlog via F's own dispatch mechanism.
**Symmetric obligation:** Sub-projects brainstormed after a cross-cutting framework must pull queued stubs into their own backlogs during brainstorm, not wait for triage.

### Cross-plan coordination via subagents and vault catalog
A plan's LLM phases must not read another plan's files. Cross-plan awareness arrives via `{{VAULT_CATALOG}}` (descriptions only, never content) and dispatched subagents. Level 2 routing agent is the primary mechanism for cross-project reasoning. This is the runtime complement to the adoption-stub discipline.

### Brainstorming methodology
Three linked disciplines that compound:

**Lock one fork per clarifying question.** Structure brainstorms as a linear sequence where each question locks a design fork and the answer feeds the next. By design-section presentation, content is determined; presentation becomes verification.

**Test scope assumptions early.** When generating migration/compatibility questions, check whether those scopes are real first. "Are there real v0.1.0 users?" dropped a CLI subcommand, eliminated eight hardcoded paths, killed one YAML config format. F's "Is sunk cost real?" drove the BEAM pivot.

**Re-read memory entries before revising.** Entries are dense and easy to misread under reasoning pressure. First action on a "let me revise that" impulse: re-read the entry to confirm current framing.

### Files as the durable substrate
Everything non-transient is a file. The daemon is an orchestrator, not a state store. Daemon restart rebuilds all actor state from files. Team mode (sub-P, v2+) becomes "multiple daemons over the same filesystem, syncing through git." Individual actor crashes don't destroy state. Files-as-substrate composes: every "where does this live?" question resolves to a file, and every "what happens on crash?" question resolves to replay from files.

### memory.md and backlog.md drift is load-bearing
Both files are consumed by LLM phases every cycle; stale content propagates misinformation into fresh context. `session-log.md` is immune (no LLM phase reads it). Fix drift immediately, especially after upstream changes affecting multiple plans.

### LLM_CONTEXT punch-list 1-3 landed (stop-gap)
Small-fix version: `{{PROJECT}}` placeholders + work prompt substitution instructions. NOT the larger restructure, deferred/subsumed by the orchestrator. Landed in APIAnyware-MacOS and GUIVisionVMDriver.

## Sub-projects

Fifteen sub-projects (fourteen active, one obsolete). Brainstorms complete: A, B, C, E, F, M (six of fourteen). Each produces a design doc at `{{PROJECT}}/docs/superpowers/specs/` and a sibling plan at `{{PROJECT}}/LLM_STATE/` (sibling plan scaffolding for F deferred until BEAM PTY spike validates sub-C approach).

| ID | Sub-project | Complexity | Status | Notes |
|----|-------------|-----------|--------|-------|
| A  | Global knowledge store at user-specified path | Small-med | **done** | Design doc `specs/2026-04-13-sub-A-global-store-design.md`. Explicit vault discovery, `mnemosyne.toml` marker, `init`/`init --from`/`config use-vault`/`adopt-project` commands. **F amendment pending**: Elixir daemon caller integration. |
| B  | Phase cycle | Medium | **done** (pending F amendment) | Design doc `specs/2026-04-12-sub-B-phase-cycle-design.md`. Produced: hard errors, no slash commands, Obsidian explorer, vault+symlinks, per-project `mnemosyne/`, embedded prompts. **F amendment pending**: PhaseRunner runs inside PlanActor; `plan-state.md` schema pruning (remove `plan-id`/`host-project`/`dev-root`, add `description`); `{{RELATED_PLANS}}` → `{{VAULT_CATALOG}}`; delete `related-plans.md`. |
| C  | Harness adapter layer | Med-large | **done** (pending F amendment + BEAM spike) | Design doc `specs/2026-04-13-sub-C-adapters-design.md`. V1 Claude Code only. **F amendment pending**: Elixir implementation via `erlexec` or similar; tool-call boundary for in-session Queries; multi-adapter reservation for sub-O. **BEAM PTY spike** required before implementation. |
| D  | Concurrency (scope collapsed by F) | Small | not started | F's daemon commitment replaces per-plan advisory locks with OTP mailbox serialization. D's scope collapses to: daemon singleton lock + advisory file locks for external-tool coordination (Obsidian, git). Much smaller brainstorm. |
| E  | Post-session knowledge ingestion | Medium | **done** (pending F amendment) | Design doc (`501c15c`). **F amendment pending**: Stage 5 becomes dispatch-to-experts instead of direct store write. |
| F  | Plan hierarchy + actor model + dispatch + declarative routing | **Large** | **done** | Design doc `specs/2026-04-14-sub-F-hierarchy-design.md`. Committed: persistent BEAM daemon, two sealed actor types, two message types, `project-root` convention, path-based qualified IDs, dispatch asymmetry, vault catalog, Datalog-ish declarative routing with Level 2 fallback, Elixir runtime, Rust TUI client. Sibling plan scaffolding deferred until BEAM PTY spike. |
| G  | Migration: LLM_CONTEXT + v0.1.0 transition | Medium | not started | Parallel/ongoing. Owns rename (`LLM_STATE/` → `mnemosyne/project-root/`, `knowledge/` → `mnemosyne/knowledge/`) and `phase.md` → `plan-state.md`. **F amendment pending**: migration gains a "start the daemon" step + Rust CLI → Elixir daemon + Rust TUI split awareness. |
| H  | Fold 7 Claude Code skills | Small-med | not started | Mechanical; depends on B. Skills become attached-client TUI actions. **F amendment pending**: framed as client-attached actions, not standalone tools. |
| I  | Obsidian coverage (re-scoped) | Small-med | not started | Document which Obsidian features cover which data surfaces. **F amendment pending**: Obsidian as daemon client, not just file viewer. |
| J  | Human-mode phase affordances | N/A | **obsolete** | Folded into B as `ManualEditorExecutor`. |
| K  | Obsidian plugin client | Medium | not started (v1.5+) | Daemon-as-backend makes K more attractive. Client of F's Unix socket protocol. Re-evaluate at v1 scope cut. |
| L  | Obsidian terminal plugin spike | Small | not started | Prerequisite for K. Independent — can start any time. |
| M  | Observability framework | Med-large | **done** (pending F amendment) | Design doc `specs/2026-04-13-sub-M-observability-design.md`. Hybrid `tracing` + `MnemosyneEvent`. **F amendment pending**: re-cast as `:telemetry` + typed Elixir struct events for BEAM runtime. |
| **N** | **Domain experts** | **Medium** (new) | not started | **New sub-project added by F**. Owns ExpertActor internals: persona format, retrieval strategies, knowledge curation, default expert set, ingestion Stage 5 integration. Brainstorms after F's sibling plan lands. |
| **O** | **Mixture of models** | **Medium** (new) | not started (v1.5+) | **New sub-project added by F, reserved for v1.5+**. Multi-adapter harness layer, per-actor model selection, local-model adapters, cost telemetry. F reserves `[harnesses.*]` config and `model:` actor field. |
| **P** | **Team mode** | **Large** (new) | not started (v2+) | **New sub-project added by F, reserved for v2+**. Multi-daemon transport via BEAM distribution or custom TCP, peer discovery, cross-daemon auth, shared-vault conflict resolution. F reserves `[peers]` config and `<peer>@<qualified-id>` syntax. |

### Recommended sub-project ordering (post-F)

Committed brainstorms: A, B, C, E, M, F → **land F's sibling plan after BEAM PTY spike** → amendment tasks for A, B, C, E, M merge into their implementation plans → remaining brainstorms D, G, H, I, N (with D dramatically collapsed) → sub-O (v1.5) → sub-P (v2).

Critical next step: **BEAM PTY spike (sub-C's amendment task)**. Few hours of work. If `erlexec` successfully spawns Claude Code under PTY with stream-json + sentinel detection + process-group termination, sub-C's implementation is straightforward Elixir. If it fails, fallback is a small Rust PTY-wrapper binary invoked as an Erlang Port.

## Open questions

### BEAM PTY story for sub-C
Can `erlexec` cleanly spawn Claude Code with PTY I/O, stream-json bidirectional capture, sentinel string detection, and process-group termination (SIGTERM→SIGKILL)? This is the one real ecosystem unknown after the BEAM commitment. A few hours of spike work will answer it. Fallback: small Rust PTY-wrapper binary invoked from Elixir as an Erlang Port.

**Owner:** sub-C's amendment task is the BEAM spike.

### Pattern-matched Elixir routing vs Erlog
Will user-authored `defp route/2` clauses be expressive enough, or will Erlog (embedded Prolog) be needed? Unknown until real users write rules. V1 ships with pattern matching; Erlog reserved for v1.5+ if the need arises.

### Default expert set
Which experts ship by default? Sub-N decides. Initial candidates: rust-expert, research-expert, distributed-systems-expert, software-architect, obsidian-expert, ffi-expert.

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

This plan originated as a fix for LLM_CONTEXT punch-list issues, escalated to project-wide reorganization, then pivoted to recognizing Mnemosyne already implements the knowledge layer, then to full architectural inversion (Mnemosyne as orchestrator for the entire work cycle), then to the actor-daemon commitment plus BEAM runtime in April 2026 (Session 9). Bootstrapped on LLM_CONTEXT machinery per user direction. V1 architecture is now committed and documented in `docs/architecture.md` + `docs/superpowers/specs/2026-04-14-sub-F-hierarchy-design.md`.
