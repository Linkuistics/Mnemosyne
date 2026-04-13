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

**Cross-cutting infrastructure sub-projects: pick the standard tool at every
internal decision, not just at the headline "what tool" question** (sharpened
2026-04-13 during sub-project M's brainstorm). When a sub-project's scope is
"design infrastructure that everything else uses" (M is the exemplar; D, F, H,
I will face the same shape), the default is more aggressive than the general
rule above: after the architectural fork question lands, treat every remaining
decision as "pick the standard tool" rather than as a fresh design fork, and
bound the custom code as tightly as possible. M's deliverable is one
~200-line `tracing-subscriber::Layer`; everything else is composed stock
`tracing-subscriber` layers, stock `metrics` / `metrics-exporter-prometheus`,
and stock crates. The user steer that produced this sharpening was "use
existing tooling and libraries wherever possible — this is not an interesting
task," and it compressed what would have been 5–8 more clarifying rounds into
one design-presentation message. Future cross-cutting brainstorms (D, F, H, I)
should ask "what existing crate covers this ground" before "what should we
build" at *every* sub-decision, and collapse the standard-tool decisions into
a single design pass once the foundation question is answered.

### Hard errors by default
Soft fallbacks require explicit written rationale. Unexpected conditions,
invariant violations, I/O failures, and ambiguous state all fail hard with
clear diagnostics rather than attempting silent recovery. Documented
exceptions (e.g., sub-project E's ingestion pipeline not blocking the phase
cycle on store-lock contention) must name the rationale in the design doc.
Applies project-wide, not just to sub-project B. Added 2026-04-12 during
sub-project B's brainstorm in response to a direct user preference.

### Always-on instrumentation; tactical measurement disclaims framework scope
When a sub-project needs measurement to validate an acceptance gate (or
for diagnostic confidence), the default is to emit it unconditionally —
no debug flag, no env var, no gated build. The cost of a few timestamps +
one channel send + one file write per session is so small it doesn't
merit a flag, and the benefit (no "I wish I had measurements from that
one weird session" moments) compounds across the whole dogfood cycle.
First instance: C's `SpawnLatencyReport` is emitted on every session,
written to `<staging>/spawn-latency.json`, and surfaced as an
`InternalMessage` chunk. The framing "make the acceptance gate measurable
by default" generalises to every other sub-project's instrumentation
needs.

**Tactical instrumentation must explicitly disclaim being a framework.**
Purpose-built measurement (e.g., C's `SpawnLatencyReport`) should be
documented as a tactical seed, not the start of a metrics framework, and
should not accrete framework-shaped scope creep. The broader observability
story lives in sub-project M (brainstormed 2026-04-13, hybrid
`tracing` + typed-event design) which owns the framework, the migration
path, and the sibling-backlog adoption coordination. Added 2026-04-13 during
sub-project C's brainstorm.

**Staged migration of tactical seeds into M's framework: parallel-emit +
mechanical verification window, never atomic cutover** (added 2026-04-13
during sub-project M's brainstorm). The seed-to-framework migration runs
in three independently-reversible steps: (1) the seed-owning sub-project's
v1 ships its tactical writer unchanged; (2) M's v1 lands `metric!` calls
in parallel at the same measurement points, emitting both streams; (3) a
mechanical verification window confirms M's data matches the seed's
ground-truth within an explicit tolerance (±10ms for C's `SpawnLatencyReport`),
after which M v1.1 deletes the seed's writer and sub-G's migration deletes
the staging-schema entry. Each step is independently reversible until
verification proves the framework matches reality. This pattern generalises
to any other sub-project that grows tactical instrumentation seeds before M
lands (B / D / E / F may each produce their own); the migration discipline
is the same regardless of which seed is being absorbed. The pattern's value
is correctness confidence — never trust a framework migration that didn't
run a parallel-emit window against the ground-truth it replaces.

**"Observability-friendly without committing to a framework" is the
discipline for sub-projects shipping before a cross-cutting framework
lands** (added 2026-04-13 during sub-project M's brainstorm; validated
retroactively by sub-project C). C's design-doc deliberately documented
"every state transition is a typed message; every error is a typed
variant" without committing to a logging crate. That posture made
retrofitting M onto C purely additive — `tracing::instrument` on actor
handlers and `metric!` calls at three measurement points, no redesign.
The discipline is reusable for any future sub-project whose implementation
ships before M's framework reaches it: own the type discipline at
boundaries, leave logging crate selection open, document measurement
points as named locations the framework can later attach to. The
opposite anti-pattern (commit to ad-hoc `println!`/`log` calls scattered
across the codebase) would force a redesign at framework adoption time.

### No slash commands inside the harness — control forbidden, observation required
The harness (Claude Code and any future adapter) runs as a pure worker with
no user-facing command surface. Every user action flows through Mnemosyne's
TUI, never through harness-side slash commands or callbacks. This is the
other half of the parent-process inversion — it is not just "Mnemosyne
parents the harness" but "Mnemosyne is the only thing the user types at."
Retroactively affects sub-project H (the 7 legacy Claude Code skills become
Mnemosyne TUI actions, not re-exposed as harness slash commands). Added
2026-04-12 during sub-project B's brainstorm.

**Control vs observation scope discipline (sharpened 2026-04-13 during
sub-project C's brainstorm):** The rule rules out the harness *commanding*
Mnemosyne — no slash commands, no programmatic callbacks, no LLM-invoked
Mnemosyne actions. It does NOT rule out Mnemosyne *observing* harness
state and reacting on its own side. Observation is allowed and necessary;
control is forbidden. The bidirectional stream-json output is the canonical
observation channel and flows one-way from harness to Mnemosyne. Three
distinct observation surfaces live on that channel:
1. **Chunk-level** — assistant text, tool use, tool result (normal output).
2. **Protocol-level** — `OutputChunkKind::SessionLifecycle` variants
   (`Ready`, `TurnComplete`, `Exited`) surface harness state transitions
   independent of LLM content. Used by B's executor to track "the harness
   is ready for input" and "the harness finished its current turn".
3. **Task-level** — prompt-instructed sentinel strings (e.g., "READY FOR
   THE NEXT PHASE") detected by B's executor via a sliding-buffer matcher
   in the assistant-text stream. Distinct from protocol-level turn-end
   because a single task may span many turns. See the task-level
   completion decision below.

An earlier draft of this entry said "output streams are one-way only" as
shorthand for "no callback channel". That phrasing conflated control with
observation and has been replaced. Future sub-project brainstorms that
touch the harness boundary should consult this entry and C's design-doc
§3.3 before re-framing the rule.

### Task-level completion via prompt-instructed sentinel strings
Protocol-level "turn over" (Claude Code's `result` event, surfaced as
`SessionLifecycle::TurnComplete`) tells you the model stopped emitting
tokens for this round. Task-level "I am done with the work" is a separate
signal that requires the LLM's own self-assessment. Conflating the two
would cause Mnemosyne to transition phases the moment a single turn ended
even when the LLM was mid-task. The mechanism is prompt-instructed sentinel
strings — each phase prompt ends with an instruction like "when finished
say READY FOR THE NEXT PHASE" — and B's executor runs a sliding-buffer
matcher over the assistant-text stream to detect the sentinel. Sentinel
detection lives in B (not C) because sentinels are coupled to phase
prompts and the mechanism is harness-agnostic: the same matcher works for
Claude Code, Codex, Pi, or any future adapter. Added 2026-04-13 during
sub-project C's brainstorm as the fifth cross-sub-project requirement
threaded back to B.

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
time for future macOS GUI spikes. Originally buried inside sub-project
A's task description, this blocker was promoted to a top-level `[do]`
backlog task in Session 4 (2026-04-12) per the hidden-blocker
anti-pattern lesson, then executed in Session 5 (2026-04-13) via
guivision-CLI-driven VM execution (durable feedback memory
`feedback_guivision_cli.md`).

### UI/integration spike validation uses guivision + OCR evidence
The Mnemosyne project's canonical pattern for validating UI and
integration assumptions (Obsidian features, vault behaviour, future
plugin behaviour) is to run the assumption inside a GUIVisionVMDriver
golden image, drive the GUI exclusively through the `guivision` CLI,
and capture per-check evidence as `guivision screenshot` + `guivision
find-text` (Vision OCR) artifacts committed under
`{{PROJECT}}/tests/fixtures/<spike-name>/results/<platform>/` with a
per-platform `result.md` summary table. OCR-based evidence is robust
enough for binary pass/fail acceptance without manual screenshot
inspection. Established 2026-04-13 by the Obsidian symlink spike
(Session 5), which passed 6/6 on both `guivision-golden-macos-tahoe`
and `guivision-golden-linux-24.04` with reproducible fixtures and
identical Obsidian + Dataview pins across platforms. Future spikes for
sub-projects I, K, L, and sub-project B's v1 dogfood acceptance test
should follow this pattern. SSH/rsync/VNC direct paths are explicitly
out per `feedback_guivision_cli.md` — evidence produced via SSH is
evidence about SSH, not about the tool under test.

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

### Global knowledge store moves from `~/.mnemosyne/` to a user-specified absolute path
Treated as a first-class git-tracked dev asset, visible alongside project repos.
Resolved 2026-04-13 in sub-project A's brainstorm: the vault is at a
user-specified absolute path (default `~/Mnemosyne-vault/`), discovered via
an explicit precedence chain rather than by walk-up. The DEV_ROOT framing
used in earlier drafts is discarded — there is no implicit dev-root concept.
See "Vault discovery is explicit" and "Vault identity via schema-versioned
marker" below for the specific mechanism.

### Vault discovery is explicit — env var → user config → flag override
Added 2026-04-13 in sub-project A's brainstorm. Every Mnemosyne invocation
resolves the active vault through a strict precedence chain: `--vault <path>`
CLI flag (highest) → `MNEMOSYNE_VAULT` env var (non-empty) → user config file
at `dirs::config_dir().join("mnemosyne/config.toml")` → hard error with an
actionable message. There is no walk-up search from `cwd`, no implicit
dev-root concept, and no registry of known vaults. The single-vault-per-
machine model is a deliberate simplification that composes cleanly with
Mnemosyne-as-LLM-client (internal reasoning sessions inherit the parent's
env var with no cwd dance), makes test isolation trivial (`MNEMOSYNE_VAULT=
$tmpdir`), and is consistent with the self-containment posture. Full detail
in sub-A's design doc §A1.

### Vault identity via `<vault>/mnemosyne.toml` schema-versioned marker
Added 2026-04-13 in sub-project A's brainstorm. A directory is a Mnemosyne
vault if and only if it contains a `mnemosyne.toml` file at its root with
a parseable `[vault]` table containing `schema_version`. Missing file,
parse error, or `schema_version` higher than the binary supports are all
hard errors with actionable messages naming the offending path. The same
file doubles as the host for optional vault-level overrides (language
profiles, context mappings), eliminating the dotfile-without-extension and
mirroring the Cargo.toml "project marker + config" pattern. Verification
happens at every invocation via `verify_vault`. Full detail in sub-A's
design doc §A3.

### v0.1.0 has no real users; legacy paths are deletable, not transitionable
Added 2026-04-13 in sub-project A's brainstorm after the user confirmed
there is no existing Mnemosyne usage to migrate. Consequences:
(1) sub-A produced no `migrate` subcommand — it is out of scope entirely;
(2) the hardcoded `~/.mnemosyne/` paths at eight sites in `src/main.rs`
are deleted outright in sub-A task 11 rather than transitioned through
a compatibility shim; (3) the `Config::load(dir)` / `Config::save(dir)`
call sites in `src/config.rs` plus `<store>/config.yml` are deleted in
sub-A task 12, with language profiles and context mappings moving to
binary-embedded defaults with optional overrides from the vault's
`mnemosyne.toml`; (4) any future sub-project surfacing v0.1.0 code paths
may treat them as deletable rather than load-bearing. This simplification
compounds: the single constraint "no real users" drops migration scope,
eliminates a CLI subcommand, deletes ~8 hardcoded paths, deletes one
YAML config file format, and simplifies test fixtures. Future
sub-projects should check this entry before assuming v0.1.0 compatibility
obligations apply.

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
and lifecycle for one harness. **v1 ships the Claude Code adapter only** —
decided 2026-04-13 during sub-project C's brainstorm; Codex and Pi adapters
follow post-v1. This is what makes the orchestrator harness-independent. The
adapter layer serves two distinct call sites: user-facing plan sessions and
Mnemosyne's own internal reasoning sessions (see the "Mnemosyne is itself an
LLM client" decision), so it must support configurable tool profiles at spawn
time. C's brainstorm also locked in bidirectional stream-json as the
harness-to-Mnemosyne protocol, process-group termination as a v1 correctness
requirement, and actor-style threading per session (BEAM-inspired,
crossbeam-channel message passing). See sub-project C's design doc for
trait shape, amendments to B's `HarnessSession`, and the tactical
`SpawnLatencyReport` instrumentation for the C-1 dogfood acceptance gate.

### Hybrid `tracing` + typed-event-enum is the project-wide pattern for typed-state cross-cutting components
Mnemosyne has two competing architectural principles that pull in opposite
directions when designing any cross-cutting component with both typed state
transitions and a need for spans / async-context / third-party integration.
"Integration over reinvention" pulls hard at `tracing` (the de facto Rust
standard, mature ecosystem, span semantics for free, third-party crate
events flow through automatically). "Hard errors by default; every state
transition is a typed message" pulls hard at a custom typed-event-enum
bus that downstream consumers exhaustively pattern-match. The hybrid
architectural pattern honours both: own a typed `MnemosyneEvent` enum at
the Mnemosyne boundary so downstream consumers (CLI display, Obsidian
ingestion, future Tauri client, sub-E ingestion pipeline) get
exhaustive-match safety; lean on `tracing` for transport, spans,
async-instrumentation, and third-party crate integration *below* the
boundary. The custom code is bounded to one ~200-line
`tracing-subscriber::Layer` that lifts typed events out of the `tracing`
event stream; everything else is composed standard layers. Established
2026-04-13 during sub-project M's brainstorm as the foundation of M's
design, but the pattern generalises beyond observability — it is the
right answer for any future Mnemosyne component that has the same
two-principle tension (typed-state IPC schemas, structured ingestion
events, future plugin protocols). Future cross-cutting brainstorms
should reach for this pattern before designing a custom event bus or
committing to ad-hoc `tracing` calls — the answer is almost always
"hybrid: typed at the boundary, `tracing` below."

### Cross-cutting sub-project brainstorms own their own cross-plan adoption stubs
When a sub-project's design imposes adoption work on sibling sub-projects
(M imposes `mnemosyne_event!` calls and `tracing::instrument` annotations
on B / C / D / E / F / H / I / G; future cross-cutting sub-projects will
impose similar work), the brainstorm session that produces the design doc
must also produce concrete adoption stub tasks in every affected sibling
backlog *before stopping*. Triage is not the right place to discover the
coordination requirement — by the time triage runs, the brainstorming
session's fresh context has already been lost. M's brainstorm exemplified
the pattern: §10 of M's design doc commits M to owning its own adoption
coordination, and the same brainstorm session appended adoption tasks to
sub-B-phase-cycle/backlog.md, sub-C-adapters/backlog.md, and
sub-E-ingestion/backlog.md before phase exit (D / F / H / I / G adoption
stubs queued in M's memory.md with a triage rule that lands them as those
sibling brainstorms complete). This is the right pattern for any future
cross-cutting sub-project (current candidates: D's locking primitives, F's
hierarchy semantics if they impose plan-state schema requirements on B/G,
I's Obsidian conventions if they impose frontmatter requirements on
A/B/E). Established 2026-04-13 during sub-project M's brainstorm. The
discipline matters because cross-cutting coordination gaps are silent: a
sibling sub-project that is missing an adoption stub will not surface the
gap until *its own* brainstorm runs, by which point the cross-cutting
sub-project's design rationale has decayed.

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

The merge breaks into thirteen sub-projects (twelve active, one obsolete).
Eight were identified during the originating brainstorm; sub-projects I and
J were surfaced during sub-project E's brainstorm; sub-projects K and L were
surfaced during sub-project B's brainstorm; sub-project M was surfaced during
sub-project C's brainstorm; sub-project J was marked obsolete during
sub-project B's brainstorm as its scope folded into B entirely. Brainstorms
complete: E, B, C, M (four of twelve active). Each active sub-project is
brainstormed in its own work session of this plan, producing a design doc
at `{{PROJECT}}/docs/superpowers/specs/` and a sibling LLM_CONTEXT plan at
`{{PROJECT}}/LLM_STATE/` containing the implementation backlog.

| ID | Sub-project | Approximate complexity | Brainstorm | Notes |
|----|-------------|------------------------|------------|-------|
| A  | Move global knowledge store from `~/.mnemosyne/` to user-specified absolute path | Small-medium | **done 2026-04-13** | Design doc at `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-A-global-store-design.md` (commit `c81fd48`); sibling plan at `{{PROJECT}}/LLM_STATE/sub-A-global-store/` with fifteen-task implementation backlog. Locked: explicit vault discovery chain (no walk-up, no dev-root); `mnemosyne.toml` schema-versioned identity marker; `init` / `init --from` / `config use-vault` / `adopt-project` commands; Tier 1 / Tier 2 env-var overrides for tests; v0.1.0 `~/.mnemosyne/` paths deletable rather than transitionable (no real users). Dropped the `migrate` subcommand entirely. Surfaced one new project-wide open question (team-mode use). |
| B  | Reimplement phase cycle in Rust inside Mnemosyne | Medium | **done 2026-04-12** | Design doc at `{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md`; sibling plan at `{{PROJECT}}/LLM_STATE/sub-B-phase-cycle/`. Produced cross-cutting architectural decisions: hard errors default, no slash commands in harness, Obsidian as committed explorer, dedicated Mnemosyne-vault with symlinks, per-project `mnemosyne/` directory, self-containment via embedded prompts, Path 1 staging (ratatui v1, Obsidian plugin v2). Folded sub-project J. Surfaced sub-projects K and L. |
| C  | Harness adapter layer (Claude Code, Codex, Pi, others) | Medium-large | **done 2026-04-13** | Design doc at `{{PROJECT}}/docs/superpowers/specs/2026-04-12-sub-C-harness-adapters-design.md` (commits `71fd307`, `b1a8cea`); sibling plan at `{{PROJECT}}/LLM_STATE/sub-C-harness-adapters/` (commit `9dac743`). **v1 ships Claude Code adapter only** (Codex and Pi deferred). Produced four additive amendments to B's `HarnessSession` trait (bidirectional stream-json, `OutputChunkKind::SessionLifecycle` protocol observation, tactical `SpawnLatencyReport`, and a fifth executor-level requirement: sentinel detection in B's executor for task-level completion). Actor-style threading per harness session (BEAM-inspired, crossbeam-channel message passing, three threads: actor + stdout-reader + stderr-reader). Process-group termination (SIGTERM→SIGKILL with 500ms grace) is a v1 correctness requirement, not a v1.5 follow-up. Warm-pool deferred to v1.5 behind the C-1 dogfood acceptance gate; if C-1 fails, a three-check spike explores `/clear`-based session reset (§7.4 of C's design doc). Surfaced sub-project M (Observability). |
| D  | Multi-instance concurrency model with shared-store locking | Small-medium | not started | Reduced from "multi-plan TUI" — TUI multiplexer cut from v1. Lock location is `<vault>/runtime/locks/<plan-id>.lock` per B's design. Per-plan scope, vault-scoped. |
| E  | Post-session knowledge ingestion model (parent reads child's outputs) | Medium | **done 2026-04-12** | Design doc committed at `501c15c`; sibling plan at `{{PROJECT}}/LLM_STATE/sub-E-ingestion/`. B implements the `ReflectExitHook` interface E's pipeline subscribes to. |
| F  | Plan hierarchy + permanent root plan in Mnemosyne's data model | Medium | not started | B's `plan-state.md` marker rule and `StagingDirectory::render` descent invariant are non-negotiable B contracts F must respect. F decides everything else about hierarchy semantics. |
| G  | Migration strategy: existing LLM_CONTEXT users + Mnemosyne v0.1.0 users transition smoothly | Medium | not started | Parallel and ongoing. B has added per-project directory rename (`LLM_STATE/` + `knowledge/` → `mnemosyne/plans/` + `mnemosyne/knowledge/`) and `phase.md` → `plan-state.md` migration as G's concrete tasks. |
| H  | Fold the 7 Mnemosyne Claude Code skills into Mnemosyne's internal cycle phases / commands | Small-medium | not started | Mostly mechanical; depends on B (now done). Legacy skills become `TuiAction` enum variants in Mnemosyne core, not harness slash commands. Every skill preserved in v1 must have a human-driven counterpart. |
| I  | Explorer framework — full-CRUD explorers for knowledge, plan state, sessions, ingestion provenance | Small-medium (re-scoped) | not started | Re-scoped during B's brainstorm: Obsidian is the committed explorer, so I's job shrinks from "build a unified explorer framework" to "document which Obsidian features cover which Mnemosyne data surfaces, with recommended vault configurations and Dataview query examples." |
| J  | Human-mode phase affordances | N/A | **obsolete 2026-04-12** | Folded into sub-project B as `ManualEditorExecutor`. The co-equal-actors principle is enforced by B's pluggable `PhaseExecutor` trait chokepoint. |
| K  | Obsidian plugin client | Medium (v1.5+) | not started | New. Surfaced during B's brainstorm as Path 1's deferred Obsidian-integration sub-project. Depends on B's IPC boundary landing in v1. Consumes the JSON protocol `IpcDriver` exposes. Not a v1 scope cut decision — staged to v1.5+ by design. |
| L  | Obsidian terminal plugin spike | Small (investigation) | not started | New. Prerequisite for K. Small investigation task evaluating existing Obsidian terminal plugins for PTY control and harness session hosting. Independent of B's implementation — can start any time. |
| M  | Observability framework — structured logging, metrics, event bus, tracing | Medium-large | **done 2026-04-13** | Design doc at `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-M-observability-design.md`; sibling plan at `{{PROJECT}}/LLM_STATE/sub-M-observability/` with 23-task implementation backlog. Architectural foundation: hybrid `tracing` + typed `MnemosyneEvent` enum (see "Hybrid `tracing` + typed-event-enum" decision). Crate stack: `tracing` + `tracing-subscriber` + `metrics` + `metrics-exporter-prometheus` + custom ~200-line `MnemosyneEventLayer`. Adoption tasks for sub-B / sub-C / sub-E landed directly in their backlogs as part of the brainstorm output (per "Cross-cutting sub-project brainstorms own their own adoption stubs" decision); sub-D / sub-F / sub-H / sub-I / sub-G stubs queued in M's memory.md with a triage rule. Staged migration of C's `SpawnLatencyReport` follows the parallel-emit + ±10ms verification pattern. Open implementation question: `mnemosyne_event!` macro typed-payload handoff (thread-local trick vs `Visit` API + serde round-trip) — implementation-phase task 5 owns the day-1 microbenchmark decision. |

### Recommended sub-project ordering
**~~E~~ done → ~~B~~ done → ~~C~~ done → ~~M~~ done → {A, F, D, L}
(parallel-able) → H → I, with G running in parallel throughout. K is
v1.5+ and depends on B's IPC protocol landing; L is a small independent
spike.**

- E, B, C, and M are complete. Their cross-cutting requirements have
  been threaded into the other sub-project notes so each sibling
  brainstorm absorbs them without re-derivation. B's `HarnessSession`
  trait has four additive amendments from C that B's implementation
  phase will pick up without requiring a B re-brainstorm. M's
  observability adoption stubs have been landed directly in
  sub-B / sub-C / sub-E backlogs (per "Cross-cutting sub-project
  brainstorms own their own adoption stubs" discipline).
- A is the natural next pick: independent, small-medium, and its scope
  is now simplified by B's decisions (A designs the vault location,
  config override, and bootstrap — B has already fixed the vault
  layout and per-project directory name).
- F continues the plan hierarchy thread; benefits from B being done and
  must respect B's marker rule and descent invariant.
- D is small under the new framing; can run early or alongside A.
- L (Obsidian terminal plugin spike) is a small independent investigation
  task and can run at any time as a prerequisite for K.
- H is mechanical and follows from B.
- I's scope has been substantially reduced (Obsidian is the committed
  explorer) — it is now mostly documentation work and should run in
  parallel with A/F/D or slipped in when convenient.
- G runs parallel throughout — migration plans need to evolve as the
  design evolves. G now owns the per-project directory rename surfaced
  by B's brainstorm and (post-verification) the deletion of C's
  `spawn-latency.json` staging entry once M's `SpawnLatencyReport`
  parallel-emit window proves equivalent.
- K (v1.5+) is not part of the v1 scope cut. K waits for B's
  implementation to land enough that the IPC protocol is stable.
- M's adoption stubs for sub-D / sub-F / sub-H / sub-I / sub-G are
  queued in M's memory.md and will land in those sibling backlogs as
  the corresponding brainstorms complete.

Ordering may shift as brainstorms reveal new dependencies. The triage
phase is the right place to revisit it.

## Open questions

These are not blocking the bootstrap but need answers during the relevant sub-project
brainstorms.

### Locking primitive for the shared knowledge store
Sub-project D (sub-project A's brainstorm only pinned the lock *directory*
at `<vault>/runtime/locks/`; the primitive and semantics remain D's call).
Candidates: file locks via `flock`, a `.lock` sentinel file, SQLite-backed
index with native locking, or a Rust crate's lock primitive. Granularity
question: whole store vs. per-axis vs. per-entry.

### Team-mode usage of Mnemosyne
Mnemosyne is currently a solo-developer tool by design and by every
cross-cutting assumption surfaced so far (sub-A's single-vault-per-machine
discovery, sub-D's single-machine locking model, sub-E's sequential
ingestion, sub-B's "one Mnemosyne instance per plan"). Multi-developer use
against a shared vault has not been brainstormed and will likely require
additions to D (distributed/pessimistic locking semantics), E (concurrent-
write conflict handling), the curation workflow (parallel-curation
discipline), and possibly the user config schema (per-user identity
attribution in the marker or user config). Sub-A's design accommodates
team mode at the *layout* level — the gitignore policy already separates
shared assets (knowledge, archive, `.obsidian/` template) from
machine-local state (runtime, cache, projects symlinks), which is the
exact split a team workflow would eventually need — but does not solve
the additional concurrency design team use would require. Surfaced
2026-04-13 during sub-project A's brainstorm. Owner: TBD; likely a
future cross-cutting brainstorm of its own once a concrete team use
case exists.

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
