# Backlog — Mnemosyne Orchestrator

Tasks for merging LLM_CONTEXT functionality into Mnemosyne and building the persistent actor-daemon architecture committed in Session 9 (2026-04-14).

## Recent history

**Triage (2026-04-15) — Sub-N done; all orchestrator amendment tasks complete; cross-plan propagations dispatched.** Done tasks cleared from backlog: Priority 0 BEAM spike, Sub-A/B/C/M amendments, Sub-N brainstorm, Sub-F sibling plan scaffold, entire Completed section. Sub-E amendment updated to reference sub-N Task 15 concrete interface contracts (`%ExpertAbsorbCandidate{}`, `ScopeMatcher`, verdict structs, `%Expert.*` events). Priority 1 execution note updated. Dispatched: sub-E (Task 15 types unblock the Stage 5 amendment), sub-M (sub-N adds `Mnemosyne.Event.Expert.*` as a new sealed-struct producer not yet in M's backlog), sub-F (sub-N Tasks 16+ gate on sub-F's `Mnemosyne.Actor` behaviour + `ActorSupervisor` API). Phase: work.

**Session 16 (2026-04-15) — Sub-N brainstormed + sibling plan scaffolded + sub-Q and sub-R surfaced as new research sub-projects.** Sub-N's design doc at `docs/superpowers/specs/2026-04-15-sub-N-domain-experts-design.md` (1104 lines). Sibling plan at `LLM_STATE/sub-N-domain-experts/` with 29 tasks across 9 phases, memory anchors N-1 through N-13, prompt-work.md. Seven brainstorm questions (knowledge scope, dialogue state, knowledge ownership, retrieval strategy, Stage 5 verdict aggregation, scope-matching mechanism, two-timer clarification), eight approval-gated design sections (scope/non-goals, architecture, declaration format, retrieval, query/dialogue protocol, Stage 5 ingestion, lifecycle/hot reload/errors, testing). Two user signals captured as new brainstorm tasks: **sub-Q** (vector-store infrastructure, v1.5+) — appealing as Mnemosyne-wide capability beyond just sub-N's retrieval strategy; **sub-R** (knowledge ontology, research task, v1.5+) — addressing tag-vocabulary drift across sub-N, sub-E, sub-F. Both reserved with no hard-gate on sub-N v1 shipping. Unblocks sub-F Task 3 stub replacement path and sub-E amendment's scope matcher + message struct dependency (sub-N's own Task 15 is an early-deliverable PR for sub-E's parallel work). **Critical track for next cycle**: sub-E amendment (unblocked — can start via the early-deliverable Task 15 interface contracts even before sub-N's actor implementation lands).

**Session 15 (2026-04-15) — Sub-M amendment absorbed; all orchestrator-level amendment tasks done.** Sub-M's design doc rewritten inline across §1–§20 (870 lines, up from 626) following the sub-C/sub-B/sub-A precedent. Sealed `Mnemosyne.Event.*` struct set expanded from 7 Rust variants to 20+ Elixir structs grouped by producer (B/C/F/E/A + M escape hatches). §6 metric catalogue at 23 `Telemetry.Metrics.*` definitions; §5.2 mandates a load-bearing `try/rescue` wrapper on the `:telemetry` handler because `:telemetry` detaches handlers that raise; `:pg` replaces `mpsc::Sender` for TUI fan-out; `mix mnemosyne.metrics` / `mix mnemosyne.diagnose` replace Rust CLI subcommands. Q1–Q5 preserved verbatim with Session-15 correction notes; Q6 (BEAM pivot translation table, 19 rows) and Q7 (reporter selection — `ConsoleReporter` + `SnapshotReporter` v1, `:telemetry_metrics_prometheus` v1.5, OpenTelemetry v2) added. Sibling plan top-notice + new Task 0 gate for downstream Rust task-list rewrite. **Unblocks F Task 0 condition (3) — the final orchestrator-level gate condition is now cleared.** Remaining F Task 0 gate conditions are both sibling-level: sub-B downstream task-list rewrite (sibling plan), sub-C downstream task-list rewrite (sibling plan). All four orchestrator amendment tasks (A, B, C, M) are now done.

**Triage (2026-04-15) — State confirmed; cross-plan propagations dispatched.** All four orchestrator-level F amendment tasks done (A, B, C, M). Two sibling-level F Task 0 conditions remain: sub-B and sub-C downstream task-list rewrites. No task removals, no priority changes. Active critical work: sub-N brainstorm (v1-critical, immediately executable) and sub-E amendment (parallel). Dispatched: sub-F Task 0 conditions (3)+(4) marked cleared; sub-E notified of M § 4.1 `Ingestion.*` struct availability. Phase: work.

**Session 14 (2026-04-15) — Sub-A amendment absorbed; three of four F Task 0 gate conditions met.** Sub-A's design doc rewritten inline across §A1–§A10 (1004 lines, up from 635) following the sub-C/sub-B precedent. All five Session-7 forking decisions survived the runtime swap unchanged because none were language-specific. §A4 vault layout picks up every sub-F surface; §A6 init flow rewritten as 12-step Elixir flow; §A10 Tier 1/Tier 2 resolution rewritten in Elixir. Sibling plan updated. **Unblocks F Task 0 condition (3).** Remaining F Task 0 gate conditions: sub-B downstream task-list rewrite (sibling plan), sub-C downstream task-list rewrite (sibling plan), sub-M amendment absorption (this backlog).

**Session 13 (2026-04-15) — Sub-F sibling plan scaffolded; implementation runway opened.** P3.1 landed. `LLM_STATE/sub-F-hierarchy/` written with 28 implementation tasks derived from F's §11 (Elixir scaffolding → daemon binary → declarative routing → Level 2 agent → integration → tests), plus a Task 0 readiness gate mirroring sub-B's pattern. Memory file captures F-1 through F-12 architectural anchors and the contract F consumes from siblings A/B/C/M. Rust `mnemosyne-tui` binary explicitly excluded as a future separate plan; F's §11.8 cross-plan landings excluded as already-done by F's Session-9 triage. The critical-next-step queue now advances: next is either sub-N brainstorm (v1-critical ExpertActor internals) or the remaining amendment tasks for A, E, M whose inputs are now concrete from B's §4 and C's §11.

**Sessions 10–12 (2026-04-15) — BEAM PTY spike passed, sub-C and sub-B amendments absorbed via inline rewrites.** Session 10 validated pipes-only `erlexec` at `spikes/beam_pty/` (no PTY needed, no Rust wrapper fallback). Session 11 rewrote sub-C's design doc inline for Elixir/erlexec (1186 lines, §1–§11 fresh, tool-call boundary in §4.5 made concrete). Session 12 rewrote sub-B's design doc inline absorbing three simultaneous pivots (LLM_CONTEXT overhaul, sub-F actor commitment, BEAM pivot) — 2296 lines with no supersede layer. Gate tasks added to both sibling backlogs for their downstream implementation-task-list rewrites.

**Session 9 (2026-04-14) — Sub-project F brainstormed; architecture pivoted to BEAM daemon.** F's brainstorm expanded from "plan hierarchy + root plan" to the full v1 architecture: persistent BEAM daemon, two sealed actor types (PlanActor + ExpertActor), two message types (Dispatch + Query), `project-root` convention, path-based qualified IDs, dispatch asymmetry (same-project direct vs cross-project Level 2 agent), vault catalog, declarative routing with LLM-fallback learning loop, Elixir/OTP runtime, Rust TUI as separate client binary. Sunk-cost analysis showed the v0.1.0 Rust code was less than a day of effort; the BEAM pivot was the right call at the right time. Three new sub-projects surfaced: **N (domain experts)**, **O (mixture of models, v1.5+)**, **P (team mode, v2+)**. Design doc at `docs/superpowers/specs/2026-04-14-sub-F-hierarchy-design.md`. Documentation overhaul landed in the same session: new README, `docs/architecture.md`, rewritten `user-guide.md` and `configuration.md`. F's sibling plan scaffolding is **deferred until the BEAM PTY spike validates sub-C's approach** — this is the first critical implementation task.

**Session 8 (2026-04-14) — LLM_CONTEXT 2026-04 overhaul reconciliation.** Upstream LLM_CONTEXT shifted to a four-phase cycle (work → reflect → compact → triage), phase-file-factored composition, `memory-style.md` as single source of truth, `pre-work.sh` opt-in hook, `{{RELATED_PLANS}}` placeholder (now superseded by F's `{{VAULT_CATALOG}}`), `related-plans.md` schema (now deleted by F), and renamed driver/spec files. Session 8 pulled drift into all generated plans and fixed orchestrator references. No code changed.

**Session 7 (2026-04-13) — Sub-projects A and M brainstormed.** A committed explicit vault discovery, `mnemosyne.toml` marker, init/adopt commands, deletion of v0.1.0 hardcoded paths. M committed hybrid `tracing` + `MnemosyneEvent` architecture (now re-cast to `:telemetry` + typed Elixir struct events post-F).

**Session 5 (2026-04-13) — Obsidian symlink validation spike PASSED 6/6** on macOS and Linux via guivision + OCR evidence. Hard-copy-staging fallback not needed. Symlink approach stands.

**Sub-projects E (2026-04-12), B (2026-04-12), C (2026-04-13)** brainstormed in earlier sessions. Design docs live under `docs/superpowers/specs/`. All three need amendment tasks post-F.

## Priority 0 — Unblock F implementation

_All Priority 0 tasks complete. Section retained as heading only for historical reference._

## Priority 1 — F-triggered amendments to done brainstorms

F's architecture commitment affects every done brainstorm. Each amendment is a short work phase: read the existing design doc, identify Rust-specific or pre-F-specific sections, replace with BEAM/actor/dispatch-aware equivalents, commit.

**Execution note (triage 2026-04-15, post-Session 16):** All orchestrator-level amendment tasks done (sub-A/B/C Sessions 11–14, sub-M Session 15). Sub-N brainstorm done (Session 16). Sub-F's Task 0 readiness gate has **two remaining conditions**, both sibling-level: (1) sub-B downstream task-list rewrite (sibling plan), (2) sub-C downstream task-list rewrite (sibling plan). **Critical track: Sub-E amendment** — Stage 5 dispatch-to-experts. Sub-N Task 15 delivers concrete types (`%ExpertAbsorbCandidate{}`, `ScopeMatcher`, verdict structs, `%Expert.*` events); amendment is fully unblocked and is the #1 next work item. **Secondary tracks (any order):** D, G, H, I amendment/brainstorm tasks produce scope-framing documents. The pivot-rewrite pattern (inline rewrite, not supersede layers) applies to all remaining amendments.

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
- **Status:** done
- **Dependencies:** F done, sub-N done (Task 15 early-deliverable PR delivers concrete interface contracts: `ScopeMatcher` behaviour, `%ExpertAbsorbCandidate{}` struct, verdict structs `READY ABSORB`/`READY REJECT`/`READY CROSS_LINK`, and `%Expert.*` event structs)
- **Description:** Amend sub-E's design doc to reflect:
  - Stage 5 (knowledge store write) becomes **parallel fan-out to tag-matching experts**
  - `ScopeMatcher.match/2` (tag-based exact-string set intersection on frontmatter `tags:`) selects the expert fan-out set
  - Orphan candidates (empty match set) write directly to `<vault>/knowledge/uncategorized/` via sub-E — no expert involvement
  - Each tag-matching expert receives `%ExpertAbsorbCandidate{}` and returns a verdict in its own fresh-context session
  - Each expert returning `READY ABSORB` writes the candidate file into its own Tier 2 directory (physical duplication accepted; no auto-wikilinks in v1)
  - `READY CROSS_LINK <expert-id>`: rejection-with-suggestion; sub-E collector second-round dispatches to the suggested expert non-recursively (max depth 2)
  - Contentful disagreement (absorb + reject with non-trivial reason) surfaces `%ExpertConflict{}` event for human review
  - Pipeline stages 1–4 (extract, classify, contradict, score) unchanged
  - Implementation in Elixir with `GenStage` or `Broadway` for pipeline stages

  Moderate amendment — Stage 5 is re-cast with concrete sub-N types now available; stages 1–4 stay.
- **Results:** **done 2026-04-16 (Session 17).** Inline-rewrote `2026-04-12-sub-E-ingestion-design.md` per the "amendments rewrite inline, not as supersede layers" discipline. §1–§5 + Appendix A re-cast onto BEAM/Elixir; §-numbering preserved for downstream cross-references. Decision Trail gained Q15 (BEAM pivot) and Q16 (Stage 5 expert dispatch) entries plus correction notes folded back into Q1, Q2, Q5, Q9, Q10, Q11, Q13. Concrete wirings landed: `Mnemosyne.ReflectExitHook` callback contract from sub-B §4.2; `GenStage` pipeline composition; struct types with `@enforce_keys`; `Mnemosyne.Expert.ScopeMatcher.match_candidate/2` call site; `%Mnemosyne.Message.ExpertAbsorbCandidate{}` dispatch shape; three verdict variants (`:absorb`/`:reject`/`:cross_link_suggested`); non-recursive max-depth-2 cross-link follow-up; `%Mnemosyne.Event.Expert.Conflict{}` for contentful disagreement; orphan path → `<vault>/knowledge/uncategorized/` via `SafeFileWriter` emitting sub-E-owned `%Mnemosyne.Event.Ingestion.OrphanCandidate{}`; full consumption of sub-M §4.1's six `%Mnemosyne.Event.Ingestion.*{}` variants (no parallel sub-E channel); prompt resolution as request-response over sub-M's bus + sub-K NDJSON command rather than embedded `oneshot`. Cross-sub-project requirements section restructured per-sibling (B/C/A/D/F/N/M/H). Risks 4 (OTP mailbox backpressure) and 5 (`:telemetry` handler crash) added; Risk 1 updated for parallel expert curation. Rust idiom check clean (only legitimate "Rust TUI" references survive). Sub-M's adoption matrix entry for sub-E becomes "verify schema" rather than "wrap and migrate" — the parallel-emit window collapses to zero because there's no longer a sub-E channel to wrap. **What this suggests next:** sub-E's sibling-plan backlog still has the Rust-framed implementation tasks (types, SafeFileWriter, `IngestionEvent` channel, etc.); a separate task-list rewrite mirroring the sub-B/sub-C/sub-M pattern is needed before sub-E implementation can begin. Not blocking sub-N implementation, which can proceed against the rewritten contract.

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

### Brainstorm sub-project Q — vector-store infrastructure `[brainstorm]` (research, v1.5+)
- **Status:** not_started
- **Dependencies:** sub-N done (satisfied). Sub-Q does not block sub-N v1 — the `ExpertRetrieval` behaviour chokepoint means sub-Q can land as a drop-in `SemanticRetrieval` strategy without touching sub-N.
- **Description:** **Surfaced during sub-N brainstorm Session 16.** When sub-N locked v1 on keyword+section retrieval, the user signaled that a vector DB as a Mnemosyne-wide capability is appealing and should be treated as cross-cutting infrastructure, not a sub-N-internal strategy. Sub-Q is the brainstorm that investigates how Mnemosyne should provide vector-store infrastructure as a shared daemon service consumable by multiple sub-projects.

  Potential consumers (cross-cutting):
  - **Sub-N expert retrieval** — semantic similarity on knowledge files within an expert's scope
  - **Sub-F vault-catalog search** — "find me plans or experts related to X" at the router level
  - **Sub-E Stage 3 contradiction detection** — current sub-E uses deterministic heuristics; semantic similarity could improve precision
  - **Plan actors searching Tier 1/2 knowledge directly** — beyond expert-mediated access
  - **Sub-O local models** — local embedding models share runtime deps with local inference models, so there's natural overlap with the mixture-of-models sub-project

  Scope for the brainstorm:
  - Embedding model selection: Ollama? llama.cpp? bundled ONNX model? What accuracy/size/CPU trade-offs matter?
  - Storage layout: ETS + flat binary? Lightweight SQLite with an extension? Where does the index live relative to `<vault>/runtime/`?
  - Update policies: on knowledge file write, how does the index stay in sync without blocking writes?
  - Crash recovery: partial embed states on daemon restart
  - Similarity function: cosine vs dot product; approximate vs exact nearest-neighbor
  - Write-time vs read-time embedding: batch nightly reindex vs on-demand?
  - First-time vault setup: embedding N thousand files takes time; what's the UX?
  - Integration boundary: a behaviour module that sub-N (and others) implements against — what does the API look like?

  **Research-heavy**: there are real choices with real trade-offs, and the sub-project should investigate before committing. Not a direct implementation task — a brainstorm that produces a design doc.

  Output: design doc at `docs/superpowers/specs/YYYY-MM-DD-sub-Q-vector-store-design.md` and sibling plan at `LLM_STATE/sub-Q-vector-store/`. Reserved for v1.5+; do not land in v1.
- **Results:** _pending_

### Brainstorm sub-project R — knowledge ontology `[brainstorm]` (research, v1.5+)
- **Status:** not_started
- **Dependencies:** sub-N done (satisfied). Sub-R does not block sub-N v1 — v1 uses exact-string tag matching behind the `ScopeMatcher` interface, and sub-R's richer resolver drops in behind the same interface.
- **Description:** **Surfaced during sub-N brainstorm Session 16.** When sub-N locked tag-based scope matching for Stage 5, the user observed that tag-vocabulary drift is inevitable (e.g., `rust` vs `rustlang` vs `Rust`) and that the right long-term answer is an ontology — and that this should be handled as a separate research task, not inline with sub-N. Sub-R is that research brainstorm.

  Cross-cutting consumers of the ontology:
  - **Sub-N expert scope matching** — the `ScopeMatcher` becomes ontology-aware (synonyms, hierarchy, polysemy)
  - **Sub-E Stage 2 classification** — tag inference from extracted facts needs a canonical vocabulary
  - **Sub-F fact extraction** — concern keywords in `routing.ex` rules should match a canonical vocabulary
  - **Sub-Q vector-store** — tag-based retrieval filters need vocabulary consistency
  - **Sub-H TUI skills** — displaying tags to the user needs consistent presentation

  Scope for the brainstorm (research-heavy):
  - **Formal ontology vs folksonomy vs hybrid**: what's the right model for a single-developer knowledge system?
  - **Synonym handling**: `rust` ≡ `rustlang` — dictionary-driven? user-editable? auto-inferred?
  - **Hierarchy**: `rust ⊂ systems-programming ⊂ programming` — what does this buy for scope matching and retrieval?
  - **Polysemy**: `async` in Rust vs JavaScript vs Python — how does context disambiguate?
  - **Vocabulary evolution**: how does the ontology grow as new domains appear without breaking existing tags?
  - **Hooks already reserved in v1**:
    - Sub-A's vault layout has a reserved `tag-vocabulary.md` file path (currently unused)
    - Sub-N's `ScopeMatcher` is a behaviour-shaped interface that a richer resolver can drop into
  - **Literature survey**: 40 years of ontology research in AI/KR; what's the minimum viable ontology for Mnemosyne's scale (single user, <10k knowledge entries initially)?
  - **Integration with existing tools**: Obsidian's tag system, Dataview queries — what's preserved, what's superseded?

  **Research-heavy**: this is not an implementation task. It's an investigation that produces a design doc plus recommendations. Implementation follows in a separate sub-project derived from sub-R's output.

  Output: design doc at `docs/superpowers/specs/YYYY-MM-DD-sub-R-knowledge-ontology-design.md` and sibling plan at `LLM_STATE/sub-R-knowledge-ontology/`. Reserved for v1.5+; do not land in v1.
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

### Decide v1 scope cut `[decision]`
- **Status:** not_started
- **Dependencies:** all in-scope brainstorms complete (A, B, C, E, F, M, N done; D, G, H, I pending; K, L pending but v1.5+)
- **Description:** Once every in-scope sub-project has been brainstormed and its design doc and implementation plan exist, decide what's actually in v1 vs. deferred to v1.5/v2. Update memory.md with the v1 cut. Adjust dependent implementation plans accordingly.

  Sub-N (domain experts) is **in v1** — F relies on ExpertActor shipping alongside PlanActor. Without experts, Query messages have no interesting targets.

  Sub-O (MoM) and sub-P (team mode) are **v1.5+ and v2+ by design** — not candidates for v1 scope cut.

  Sub-K (Obsidian plugin client) remains v1.5+ per F's TUI-first commitment.

  Sub-I and sub-L are small documentation/spike work that can land wherever convenient.
- **Results:** _pending_

