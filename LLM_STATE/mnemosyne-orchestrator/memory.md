# Memory — Mnemosyne Orchestrator

This plan exists to drive the merge of LLM_CONTEXT functionality into Mnemosyne,
transforming Mnemosyne into a harness-independent LLM orchestrator that owns plan
management, the work cycle, knowledge, and harness session control.

This file is pre-populated with the architectural state captured from the originating
brainstorm session on 2026-04-12. Subsequent reflect phases append, sharpen, and
prune entries per the standard backlog-plan rules.

## Stable architectural decisions

Decisions considered settled unless a future session surfaces evidence that requires
revisiting. Entries added post-origin record which sub-project brainstorm produced
them.

### Fresh LLM context is a first-class design goal
Every LLM-using component in this project must be designed around many short, fresh
sessions rather than one long accumulating session. Context rot (drift, noise,
stale assumptions) is a primary failure mode; the phase cycle's session boundaries
are a feature, not a cost. Concretely: sub-project B's phase boundaries, sub-project
C's harness lifecycle, sub-project E's ingestion triggers, and any internal reasoning
sessions Mnemosyne spawns must all default to discrete fresh-context invocations
with explicit state handoff through files rather than implicit conversational state.
Applies project-wide, not just to sub-project E.

### Prefer integration with existing tooling over reinvention
When a sub-project's scope overlaps with existing, mature, adopted tools — Karpathy's
LLM Wiki concept, Infonodus-style knowledge-graph / insight / network-analysis tools,
established TUI frameworks, existing session / workflow / state-machine libraries,
existing knowledge-storage formats, existing PTY wrappers — the default is to adopt
or integrate rather than rebuild. Rationale: (1) reduces build surface; (2) eases
adoption because users already know the integrated tool's mental model, which is
critical for a tool that asks users to change their development workflow; (3) avoids
the NIH tax where effort goes into capabilities that already exist elsewhere, often
better than we would build them. Every sub-project brainstorm must surface and
answer at least one explicit "what existing tool covers this ground, and why are
we not using it?" question. "Built our own" is a valid answer when justified
(licence incompatibility, architectural mismatch, bootstrap discipline, specific
missing capability, or the tool is actively unsuitable); silent reinvention is not.
The largest integration surfaces are sub-project I (knowledge-graph explorers —
direct overlap with Infonodus and similar network-analysis tools), sub-project A
(knowledge store format / organisation — overlap with Karpathy's LLM Wiki conventions
and other LLM-facing wiki schemes), and sub-project C (harness adapter layer —
overlap with established PTY / session / process management libraries). B, F, D,
H, and G each have smaller but still real integration surfaces their brainstorms
must examine. Added 2026-04-12 in the work session that kicked off sub-project B.

### Hard errors by default
Soft fallbacks require explicit written rationale. Unexpected conditions,
invariant violations, I/O failures, and ambiguous state all fail hard with
clear diagnostics rather than attempting silent recovery. Documented
exceptions (e.g., sub-project E's ingestion pipeline not blocking the phase
cycle on store-lock contention) must name the rationale in the design doc.
Applies project-wide, not just to sub-project B. Added 2026-04-12 during
sub-project B's brainstorm in response to a direct user preference.

### No slash commands inside the harness
The harness (Claude Code and any future adapter) runs as a pure worker with
no user-facing command surface. Every user action flows through Mnemosyne's
TUI, never through harness-side slash commands or callbacks. This is the
other half of the parent-process inversion — it is not just "Mnemosyne
parents the harness" but "Mnemosyne is the only thing the user types at."
Retroactively affects sub-project H (the 7 legacy Claude Code skills become
Mnemosyne TUI actions, not re-exposed as harness slash commands) and
sub-project C (the adapter has no callback channel from harness to
Mnemosyne — output streams are one-way only). Added 2026-04-12 during
sub-project B's brainstorm.

### Obsidian is the committed maintenance/explorer UI
Mnemosyne's inspection, review, editing, and auditing surface is Obsidian.
Every file format, directory layout, and cross-reference decision targets
Obsidian specifically — Dataview-friendly kebab-case YAML frontmatter,
wikilinks instead of filesystem paths where cross-references exist, tags
as first-class metadata, directory structure designed for Obsidian's file
tree pane, a Mnemosyne-provided `.obsidian/` template with recommended
plugins (Dataview required, Templater optional). This commits the project
to Obsidian-native format discipline in v1; a future sub-project K
(Obsidian plugin client) enhances the experience but v1 must already ship
a maximally Obsidian-native baseline. Reframes the earlier "explorers are
load-bearing" decision: the *capability* (human can see, review, edit,
undo everything Mnemosyne does) remains load-bearing, but the
*implementation* is delegated to Obsidian rather than built in Mnemosyne.
Sub-project I's scope shrinks from "unified explorer framework" to
"document which Obsidian features cover which Mnemosyne data surfaces."
Added 2026-04-12 during sub-project B's brainstorm.

### Dedicated Mnemosyne-vault with symlinked per-project directories
The dev-root layout places a dedicated `<dev-root>/Mnemosyne-vault/`
(Mnemosyne-owned, its own `.git`, the committed Obsidian vault) alongside
project repos. The vault hosts Tier 2 global knowledge natively, hosts
runtime state (staging, interrupts, ingestion events, locks) natively, and
accesses per-project Mnemosyne content via one symlink per project under
`<vault>/projects/<project-name>` targeting `<project>/mnemosyne/`. Plans
stay in project repos (sovereign git ownership); the vault is a
view-over-symlinks, not the source of truth. Project repos are clean of
Mnemosyne-owned sibling directories — the only Mnemosyne presence inside
a project repo is `<project>/mnemosyne/`. Added 2026-04-12 during
sub-project B's brainstorm. Sub-project A finalises vault naming and
config-override mechanism.

**Hard pre-implementation blocker — resolved 2026-04-13, PASS 6/6 on
both platforms.** The cross-platform Obsidian + symlinks spike ran
successfully against `guivision-golden-macos-tahoe` and
`guivision-golden-linux-24.04` using Obsidian 1.12.7 + Dataview 0.5.67
with identical fixture and pinned versions. All six checks passed on
both platforms: Dataview `LIST FROM "projects"` spans the symlink,
graph view renders the `obsidian-spike ↔ boundary-note` cross-boundary
edge, backlinks panel indexes the cross-boundary reference, file
explorer expands `projects > example > {another-note, boundary-note,
README}` and opens files through the symlink, Obsidian's file watcher
detects external edits to project-side notes within the 5-second
target, and the vault-open safety checks surface only the standard
community-plugin trust modal (which is plugin-trust UX, not
symlink-related). Reproducible evidence committed under
`tests/fixtures/obsidian-validation/results/{macos,linux}/` with
per-platform `result.md` summary tables and per-check screenshots
plus OCR transcripts. **The vault-as-view-over-symlinks framing
stands; the hard-copy-staging fallback does NOT need to be activated.
Sub-project A's brainstorm and sub-project B's implementation plan
can proceed on the symlinked-vault baseline with no further
investigation.** Two platform-specific operational notes surfaced
during the run, neither affecting the symlink decision: (1)
Electron-in-virtio-gpu-under-tart on ARM64 Ubuntu requires
`--disable-gpu --no-sandbox` for visible rendering; (2) macOS
Notification Center widgets in the Tahoe golden image occlude part
of the Obsidian window area and should be dismissed at VM setup
time for future macOS GUI spikes. Original spike framing
(pre-resolution, kept for history): cross-platform Obsidian + symlinks
behaviour must be validated with a reproducible spike before any B code
is written; if Dataview/graph/backlink features fail on either platform,
the layout falls back to hard-copy staging with two-way sync.
**Promoted to a top-level `[do]` backlog task** during Session 4
(2026-04-12) — previously buried inside sub-project A's task
description, which was a hidden-blocker anti-pattern. Executed in
Session 5 (2026-04-13) after an initial ~2 commands' worth of
SSH-driven setup that was corrected to guivision-CLI-driven setup
per a user direction captured as durable feedback memory
`feedback_guivision_cli.md`.

### Per-project `mnemosyne/` directory replaces `LLM_STATE/` + `knowledge/` split
Each project repo has one Mnemosyne-owned directory at
`<project>/mnemosyne/` (lowercase) containing `plans/` (nested plan
hierarchy) and `knowledge/` (Tier 1 per-project knowledge). Replaces the
legacy v0.1.0 `<project>/LLM_STATE/` (plans only) and `<project>/knowledge/`
(Tier 1 only) split. Lowercase `mnemosyne/` disambiguates from the
`Mnemosyne/` project repo name and from the dev-root-level
`Mnemosyne-vault/`. Plans can nest arbitrarily inside
`<project>/mnemosyne/plans/`; a directory is a plan if and only if it
contains `plan-state.md`. `StagingDirectory::render` refuses to descend
into subdirectories containing `plan-state.md`, keeping "one plan per
Mnemosyne process" intact against the hierarchical layout.  Sub-project F
owns hierarchy semantics (reserved root plan locations, parent/child
tracking, promotion upward) but must respect the marker rule and descent
invariant as non-negotiable B contracts. Sub-project G's migration pass
renames existing plan dirs: `LLM_STATE/` → `mnemosyne/plans/`,
`knowledge/` → `mnemosyne/knowledge/`, preserving any existing nested
structure. Added 2026-04-12 during sub-project B's brainstorm.

### Self-containment from `LLM_CONTEXT/` via embedded prompts
The running Mnemosyne binary has zero runtime dependency on a sibling
`LLM_CONTEXT/` directory existing. The vendored copies of
`backlog-plan.md`, `create-a-multi-session-plan.md`, `coding-style.md`,
and `coding-style-rust.md` live inside Mnemosyne's source tree at
`{{PROJECT}}/prompts/` (top-level, not under `docs/` — these are prompts,
not documentation), are embedded into the binary at compile time via
`include_str!`, and are surfaced to phase prompts through a new
`{{PROMPTS}}` placeholder that `StagingDirectory::render` substitutes
per-phase to a staging-relative path. `StagingDirectory` also
materialises the embedded content into `<staging>/prompts/` as concrete
files during render, so the LLM reads them through its normal Read tool
against pre-substituted paths. Customisation of embedded prompts is a
v2 feature; v1 enforces "use what ships with the binary" for dogfood
discipline. Non-disruption constraint still applies to existing
LLM_CONTEXT users of the four dependent projects — LLM_CONTEXT stays
operational for them; Mnemosyne just has no runtime dependency on it.
Added 2026-04-12 during sub-project B's brainstorm.

### Path 1: ratatui v1, Obsidian plugin v2 (staged, not flipped)
V1 ships Ratatui TUI as the primary user interface. The
`InteractionDriver` boundary is hardened to a serialisable shape (every
call across it is designed as if it were serialised to JSON) enforced
at compile time by a shipping `IpcDriver` implementor with no client
attached in v1. This hardening unlocks a future Obsidian plugin client
(new sub-project K) as a pure-additive feature with no core rework.
Rejected the alternative "flip v1 to Obsidian-primary, headless daemon
+ plugin client" because: (1) V1's acceptance test is dogfooding the
orchestrator plan, which Path 2 would delay by the full Obsidian plugin
build cost; (2) ratatui integrates cleanly with existing IDE terminal
panes (Cursor, VSCode, JetBrains, Antigravity) while an Obsidian-primary
UI would pull users out of their IDE; (3) the multi-Mnemosyne-instance
architecture (multiple ratatui in terminal tabs) is zero-friction, while
multiple Obsidian windows against one vault would be painful; (4) the
workflow-in-Obsidian risk (Obsidian is not built for workflow
affordances) deserves a spike before committing to it as v1's primary
UI. New backlog candidates K (Obsidian plugin client, v1.5+) and L
(Obsidian terminal plugin spike, prerequisite for K) added to the
orchestrator backlog. Added 2026-04-12 during sub-project B's
brainstorm.

### Human and LLM are co-equal actors, not principal-and-agent
Both the human and the LLM reflect, triage, and curate. The orchestrator must
support human-driven reflection and triage as first-class flows, not just as
oversight on LLM output. This generates concrete requirements on sub-projects B
(phase cycle must expose human-only phase entry), H (skills-to-commands mapping
must preserve human-usable forms), and I (explorers must work as human curation
tools, not just LLM inspection surfaces). Every sibling sub-project brainstorm
must absorb this principle into its own decision envelope.

**Enforcement mechanism (from sub-project B):** B turns this principle into a
compile-time invariant via a pluggable `PhaseExecutor` trait with two
implementors — `LlmHarnessExecutor` (the LLM path) and `ManualEditorExecutor`
(the human path). Both run against the same phase state machine and staging
directory, so any new phase contract automatically applies to both actors.
This is why sub-project J ("human-mode phase affordances") folded into B
entirely — the trait chokepoint is a cleaner resolution than a separate
sub-project. Future executors (e.g., a mixed-mode dashboard) are additive.

### Explorers are the accountability substrate that makes auto-absorb safe
Post-session ingestion can only absorb automatically because explorers give the
human a full-CRUD window onto everything Mnemosyne has absorbed — including the
ability to review, edit, reject, and undo. Without explorers the auto-ingestion
model is unsafe; with them it is the right trade. Explorers are therefore
load-bearing for the entire orchestrator UX, not an optional polish layer, and
warrant their own sub-project (I). Scope includes knowledge (Tier 1 and Tier 2),
plan process state, session artifacts, and ingestion provenance.

### Mnemosyne is itself an LLM client via embedded Claude Code
The orchestrator spawns its own internal reasoning sessions (e.g., ingestion
analysis, reflection distillation, triage advice) in addition to the child
harness sessions it manages for plans. These internal sessions recursively use
sub-project C's adapter layer, which means C is a real first-class abstraction
rather than a shim — it is used by Mnemosyne itself, not just by users. Concrete
requirement on C: the adapter must support configurable tool profiles at spawn
time so internal reasoning sessions can be sandboxed to a narrower tool set than
user-facing sessions.

### TheExperimentalist is retired and deleted
Its conceptual scope (tracking exploratory development) is met by the LLM_CONTEXT
backlog/reflect/triage cycle plus a hierarchy of plans, and inherited by the
Mnemosyne orchestrator merge captured in this plan. The original Git-branching
framing in TheExperimentalist's README was the wrong abstraction for what was
needed — Git branches model parallel state, but exploratory development needs
temporal structure with reflection points, which the phase cycle already
provides. Decision to **delete completely** (not archive, not repurpose)
executed 2026-04-12 (Session 4): local clone at `{{DEV_ROOT}}/TheExperimentalist`
removed, GitHub repo `Linkuistics/TheExperimentalist` deleted, all references
on `www.linkuistics.com` scrubbed in a single commit and pushed to `main`.
Archival was rejected as adding no value once the website references were
gone — the conceptual story the project told is already carried by other
projects on the site. Entry kept in this memory file (rather than pruned)
because the "why the Git-branching abstraction was wrong" reasoning is
load-bearing for explaining why the orchestrator's phase cycle is the right
abstraction — future sub-project brainstorms may need that rationale when
they touch the exploratory-development framing.

### LLM_CONTEXT functionality merges into Mnemosyne
Mnemosyne becomes the single user-facing tool. The user runs `mnemosyne` to start
plans, advance phases, query knowledge, and curate. The `LLM_CONTEXT/` directory
eventually retires once Mnemosyne fully replaces it. Until then, LLM_CONTEXT and the
four projects depending on it (APIAnyware-MacOS, GUIVisionVMDriver, Modaliser-Racket,
RacketPro) must continue working unchanged.

### Mnemosyne becomes the parent process for LLM harness sessions
The current control flow (user runs `claude` → claude is the parent → claude touches
files via tools) is inverted: user runs `mnemosyne` → Mnemosyne is the parent →
Mnemosyne spawns Claude Code, Codex, Pi, or other harnesses as child processes via
PTY. The harness becomes a managed worker. This control inversion is the architectural
move that drives most other decisions in this plan.

### Mnemosyne owns all knowledge — Tier 1 and Tier 2
This is unchanged from current Mnemosyne v0.1.0 design. Per-project knowledge lives
in Mnemosyne's Tier 1 (`knowledge/` directory). Cross-project knowledge lives in
Tier 2 (the global store). The LLM_CONTEXT plan system does NOT host knowledge —
plan memory (`{{PLAN}}/memory.md`) is provisional, a scratch space for in-flight
reflections that may or may not eventually be promoted into Mnemosyne Tier 1.

### Global knowledge store moves from `~/.mnemosyne/` to a visible location under DEV_ROOT
Treated as a first-class git-tracked dev asset, visible alongside project repos.
Specific subpath TBD in sub-project A.

### Knowledge ingestion happens via post-session inspection by Mnemosyne
Because Mnemosyne is now the parent process, it can read each plan's outputs
(`memory.md`, `session-log.md`) directly after a session ends or at phase boundaries.
The LLM never invokes Mnemosyne CLI to "promote" anything. This eliminates the
LLM-discipline failure mode of "did the LLM remember to call back" and matches
Mnemosyne's existing philosophy of curation as a deliberate, separate cognitive step.

### Phase cycle reimplemented in Rust inside Mnemosyne
The work → reflect → triage cycle moves out of `run-backlog-plan.sh` and into Rust
code inside Mnemosyne. Placeholder substitution, prompt loading, phase state
management, and session lifecycle become Mnemosyne functions. The bash script is
retired as part of LLM_CONTEXT retirement.

### Harness adapter layer abstracts Claude Code, Codex, Pi, and future harnesses
Each adapter handles spawn, prompt-passing, output capture, terminal/PTY handling,
and lifecycle for one harness. v1 may ship with only Claude Code; others added later.
This is what makes the orchestrator harness-independent. The adapter layer serves
two distinct call sites: user-facing plan sessions and Mnemosyne's own internal
reasoning sessions (see the "Mnemosyne is itself an LLM client" decision), so it
must support configurable tool profiles at spawn time.

### Multi-plan work uses multiple Mnemosyne instances + locking, not a TUI multiplexer
Each Mnemosyne instance runs in its own terminal; the user multiplexes with existing
tools (tmux, terminal tabs, separate windows). A TUI multiplexer is explicitly cut
from v1 — it was the riskiest sub-project and this approach gets the same end result
much more cheaply. Locking on the shared knowledge store is a v1 requirement to
make concurrent instances safe.

### Plan hierarchy uses N-level filesystem nesting with leaf-dir plans + marker file + special root location
Plan dirs are leaves containing a marker file (e.g., `phase.md` or `.plan`).
Intermediate dirs are pure organization, not plans. The project root plan lives at
a special top-level location (e.g., `LLM_STATE/_root/`). Promotion of process state
(NOT knowledge) walks up the hierarchy. Provisionally chosen during the originating
brainstorm; sub-project F may revisit specifics.

### LLM_CONTEXT punch-list issues 1-3 landed as a stop-gap (Option C)
The small-fix version: `{{PROJECT}}` placeholders inside README/file content + work
prompt instructions to substitute. NOT the larger restructure (READMEs human-only,
`LLM_STATE/project.md`, `knowledge/` relocation), which is deferred and likely
subsumed by the orchestrator design entirely. The stop-gap establishes a convention
(`{{PROJECT}}`-prefixed references in LLM-Read files) that the new Mnemosyne plans
also follow, so the convention is consistent across both the legacy LLM_CONTEXT
projects and the new orchestrator bootstrap plans.

**Landed 2026-04-12** in APIAnyware-MacOS and GUIVisionVMDriver (one commit + push
to `main` in each repo). Issue 4 (untracked `llm-annotate-subagent.md`) was
deliberately out of scope; `knowledge/README.md` was correctly unaffected. A
stray literal `../TestAnyware/` in the racket-oo prompt was normalised in the
same pass — evidence the `{{DEV_ROOT}}` convention was already drifting even
inside pipeline-substituted files.

**Why the substitution-instruction paragraph is needed** — first concrete piece
of dogfooding evidence for the Bootstrap discipline constraint. `run-backlog-plan.sh`
substitutes `{{PROJECT}}`/`{{DEV_ROOT}}` in the prompt it passes to `claude`, but
that substitution does NOT propagate to files the LLM subsequently Reads via tools:
those files reach the LLM with raw placeholders. Each work prompt must therefore
tell the LLM to substitute them mentally on every Read. Having the orchestrator
(sub-project B) own the whole loop eliminates this gap — one of the clearest v2
requirements surfaced so far, and a concrete acceptance criterion for B's design.

## Sub-projects

The merge breaks into eleven sub-projects (ten active, one obsolete). Eight
were identified during the originating brainstorm; sub-projects I and J were
surfaced during sub-project E's brainstorm; sub-projects K and L were
surfaced during sub-project B's brainstorm; sub-project J was marked
obsolete during sub-project B's brainstorm as its scope folded into B
entirely. Each active sub-project is brainstormed in its own work session
of this plan, producing a design doc at
`{{PROJECT}}/docs/superpowers/specs/` and a sibling LLM_CONTEXT plan at
`{{PROJECT}}/LLM_STATE/` containing the implementation backlog.

| ID | Sub-project | Approximate complexity | Brainstorm | Notes |
|----|-------------|------------------------|------------|-------|
| A  | Move global knowledge store from `~/.mnemosyne/` to DEV_ROOT | Small-medium | not started | Scope simplified by B: A's job is "design the `Mnemosyne-vault/` location, config override, and bootstrap." The per-project `<project>/mnemosyne/` directory is a B decision that A can revise. |
| B  | Reimplement phase cycle in Rust inside Mnemosyne | Medium | **done 2026-04-12** | Design doc at `{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md`; sibling plan at `{{PROJECT}}/LLM_STATE/sub-B-phase-cycle/`. Produced cross-cutting architectural decisions: hard errors default, no slash commands in harness, Obsidian as committed explorer, dedicated Mnemosyne-vault with symlinks, per-project `mnemosyne/` directory, self-containment via embedded prompts, Path 1 staging (ratatui v1, Obsidian plugin v2). Folded sub-project J. Surfaced sub-projects K and L. |
| C  | Harness adapter layer (Claude Code, Codex, Pi, others) | Medium-large | not started | v1 may ship Claude Code only; must support configurable tool profiles at spawn time for Mnemosyne's own internal reasoning sessions. B has specified the `HarnessAdapter` trait shape C must implement; see B's design doc §4.1. No harness-to-Mnemosyne callback channel allowed (per "no slash commands" decision). |
| D  | Multi-instance concurrency model with shared-store locking | Small-medium | not started | Reduced from "multi-plan TUI" — TUI multiplexer cut from v1. Lock location is `<vault>/runtime/locks/<plan-id>.lock` per B's design. Per-plan scope, vault-scoped. |
| E  | Post-session knowledge ingestion model (parent reads child's outputs) | Medium | **done 2026-04-12** | Design doc committed at `501c15c`; sibling plan at `{{PROJECT}}/LLM_STATE/sub-E-ingestion/`. B implements the `ReflectExitHook` interface E's pipeline subscribes to. |
| F  | Plan hierarchy + permanent root plan in Mnemosyne's data model | Medium | not started | B's `plan-state.md` marker rule and `StagingDirectory::render` descent invariant are non-negotiable B contracts F must respect. F decides everything else about hierarchy semantics. |
| G  | Migration strategy: existing LLM_CONTEXT users + Mnemosyne v0.1.0 users transition smoothly | Medium | not started | Parallel and ongoing. B has added per-project directory rename (`LLM_STATE/` + `knowledge/` → `mnemosyne/plans/` + `mnemosyne/knowledge/`) and `phase.md` → `plan-state.md` migration as G's concrete tasks. |
| H  | Fold the 7 Mnemosyne Claude Code skills into Mnemosyne's internal cycle phases / commands | Small-medium | not started | Mostly mechanical; depends on B (now done). Legacy skills become `TuiAction` enum variants in Mnemosyne core, not harness slash commands. Every skill preserved in v1 must have a human-driven counterpart. |
| I  | Explorer framework — full-CRUD explorers for knowledge, plan state, sessions, ingestion provenance | Small-medium (re-scoped) | not started | Re-scoped during B's brainstorm: Obsidian is the committed explorer, so I's job shrinks from "build a unified explorer framework" to "document which Obsidian features cover which Mnemosyne data surfaces, with recommended vault configurations and Dataview query examples." |
| J  | Human-mode phase affordances | N/A | **obsolete 2026-04-12** | Folded into sub-project B as `ManualEditorExecutor`. The co-equal-actors principle is enforced by B's pluggable `PhaseExecutor` trait chokepoint. |
| K  | Obsidian plugin client | Medium (v1.5+) | not started | New. Surfaced during B's brainstorm as Path 1's deferred Obsidian-integration sub-project. Depends on B's IPC boundary landing in v1. Consumes the JSON protocol `IpcDriver` exposes. Not a v1 scope cut decision — staged to v1.5+ by design. |
| L  | Obsidian terminal plugin spike | Small (investigation) | not started | New. Prerequisite for K. Small investigation task evaluating existing Obsidian terminal plugins for PTY control and harness session hosting. Independent of B's implementation — can start any time. |

### Recommended sub-project ordering
**~~E~~ done → ~~B~~ done → C → A → F → D → H → I, with G running in
parallel. K is v1.5+ and depends on B's IPC protocol landing; L is a
small independent spike that can run at any time as a prerequisite for K.**

- E and B are complete. Their cross-cutting requirements have been threaded
  into the other sub-project notes so each sibling brainstorm absorbs them
  without re-derivation.
- C is the next brainstorm pick. It is B's critical sibling dependency for
  the `LlmHarnessExecutor` stub swap, so C landing unblocks B's dogfood
  task (the v1 acceptance test).
- A is independent enough to slip in next; its scope is now simplified by
  B's decisions (A designs the vault, B has already chosen the per-project
  directory name and layout).
- F continues the plan hierarchy thread; benefits from B being done and
  must respect B's marker rule and descent invariant.
- D is small under the new framing; can run early or alongside A.
- H is mechanical and follows from B.
- I's scope has been substantially reduced (Obsidian is the committed
  explorer) — it is now mostly documentation work and should run in
  parallel with A/F/D or slipped in when convenient.
- G runs parallel throughout — migration plans need to evolve as the
  design evolves. Specifically, G now owns the per-project directory
  rename surfaced by B's brainstorm.
- K (v1.5+) and L (small spike) are not part of the v1 scope cut. L can
  run at any time as a parallel task; K waits for B's implementation to
  land enough that the IPC protocol is stable.

Ordering may shift as brainstorms reveal new dependencies. The triage
phase is the right place to revisit it.

## Open questions

These are not blocking the bootstrap but need answers during the relevant sub-project
brainstorms.

### Specific DEV_ROOT subpath for the global knowledge store
Sub-project A. Candidates discussed informally include `DEV_ROOT/Mnemosyne-knowledge/`,
`DEV_ROOT/.mnemosyne-knowledge/`, or a subdirectory inside the Mnemosyne project
itself. Trade-offs: visibility, naming convention, separation from Mnemosyne tool repo.

### Whether v1 supports one harness or multiple from the start
Sub-project C. Claude Code is the user's primary harness today, so a v1 that ships
with only the Claude Code adapter is viable. Codex and Pi adapters can follow.

### Locking primitive for the shared knowledge store
Sub-project A or D. Candidates: file locks via `flock`, a `.lock` sentinel file,
SQLite-backed index with native locking, or a Rust crate's lock primitive. Granularity
question: whole store vs. per-axis vs. per-entry.

### Fate of the 7 existing Mnemosyne Claude Code skills
Sub-project H. Options: fully replaced by orchestrator phases, kept as legacy plugin
during transition, or promoted to first-class CLI subcommands. Likely a mix.

### What in `LLM_CONTEXT/` survives the merge
Sub-project G. `coding-style.md` and `coding-style-rust.md` are referenced by current
APIAnyware-MacOS work prompts. They could move into Mnemosyne, into a separate
docs repo, or stay in LLM_CONTEXT until LLM_CONTEXT itself retires.

### Which Obsidian features cover which Mnemosyne data surfaces
Sub-project I (re-scoped). B's brainstorm resolved the surface question —
Obsidian is the committed explorer, so the open work is no longer "CLI vs
TUI vs web vs framework" but "which Obsidian plugin + Dataview query covers
each data surface (Tier 1 knowledge, Tier 2 knowledge, plan state, session
artifacts, ingestion provenance), and what `.obsidian/` template ships by
default." Undo/history semantics collapse into Obsidian's file-based
history plus git in the vault; no custom undo stack needed.

## Constraints / non-goals

### Non-disruption is mandatory
The existing `{{DEV_ROOT}}/LLM_CONTEXT/` machinery and the four projects depending on
it (APIAnyware-MacOS, GUIVisionVMDriver, Modaliser-Racket, RacketPro) must keep
working unchanged throughout this build. The Mnemosyne orchestrator is built
alongside, not in place of, the existing system. Only when the orchestrator is
demonstrably equivalent or better do dependent projects migrate.

### Bootstrap discipline
This seed plan and all sub-project brainstorms run on the existing LLM_CONTEXT
machinery. No tool features may be assumed that LLM_CONTEXT does not already
support. This forces honest dogfooding — every limitation we hit while running
this plan is evidence about what v2 needs to fix.

**Dogfooded process improvements may land back into LLM_CONTEXT** when they pass
this test: (1) the change is additive documentation only, not runtime or schema;
(2) the four dependent projects (APIAnyware-MacOS, GUIVisionVMDriver,
Modaliser-Racket, RacketPro) can absorb it with zero migration work; (3) the
improvement flows forward into Mnemosyne v1 for free when sub-project B's
`include_str!`-embedded prompts vendor the canonical `backlog-plan.md` and
siblings. Bootstrap discipline forbids assuming v2 features; it does not forbid
landing v1 improvements evidenced by v1's own execution. First instance
(2026-04-12, Session 4 follow-on): the hidden-blocker anti-pattern lesson landed
as a new Tips bullet and TRIAGE phase scan step in LLM_CONTEXT's
`backlog-plan.md` (commit `a25af17`). This pattern — land the lesson in
LLM_CONTEXT, let Mnemosyne absorb it at vendoring time — is the default for any
future process improvements until Mnemosyne fully subsumes LLM_CONTEXT.

### Mnemosyne v0.1.0 keeps working
The existing Mnemosyne CLI, knowledge format, evaluation framework, and Claude Code
plugin all continue to function during the build. The Claude Code plugin is the
legacy integration path until the new orchestrator subsumes it. Items currently
listed in `{{PROJECT}}/TODO.md` (horizon scanning, evaluation phase 3/4, etc.) are
the legacy work pipeline; do not advance them as part of this plan unless they
directly enable orchestrator work.

**Scope of "keeps working" (clarified 2026-04-12):** non-disruption applies to v0.1.0
*running* during the build — not to how new orchestrator internals are designed.
Sub-project brainstorms are free to choose internal data models, schemas, and
abstractions that diverge from v0.1.0 as long as the v0.1.0 surface area keeps
functioning for users of the legacy integration path.

### No TUI multiplexer in v1
Explicitly out of scope. Multi-plan work uses multiple Mnemosyne instances +
locking + user-side multiplexing (tmux, terminal tabs).

### Never bulk-stage changes in APIAnyware-MacOS or GUIVisionVMDriver
Both repos carry substantial uncommitted WIP on `main` unrelated to this plan.
Any task touching these repos must `git add` specific paths only — never
`git add -A`, `git add .`, or `git commit -a`. Otherwise unrelated in-progress
work gets swept into orchestrator commits. This applies to every future work
session, not just the stop-gap task where it was first observed.

## Origin

This plan was created on 2026-04-12 in a single multi-message brainstorming session
that began as a fix for four LLM_CONTEXT punch-list issues, escalated to a
project-wide LLM-content reorganization, then pivoted twice: first to recognising
that Mnemosyne already implements most of the knowledge layer being designed, then
to the much larger architectural inversion captured here. The brainstorm used the
`superpowers:brainstorming` skill but deviated from its terminal step (writing-plans
skill) to instead produce this LLM_CONTEXT-format plan, per explicit user direction
to bootstrap with what works.
