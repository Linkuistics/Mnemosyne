# Session Log

### Session 1 (2026-04-12) — LLM_CONTEXT punch-list stop-gap
- Attempted: the stop-gap task in its entirety — APIAnyware-MacOS README
  rewrites (issues 1 and 2), APIAnyware-MacOS work-prompt updates, and
  GUIVisionVMDriver README consolidation (issue 3).
- Worked: all edits landed cleanly with targeted `git add`. Two repos, one
  commit and push each, both on `main`. No test/build verification was
  attempted — all changes are Markdown content with no executable effect.
- Didn't work / friction: native `Edit` tool requires native `Read` (not
  `ctx_read`) before editing; six parallel edits failed on first attempt
  and had to be retried after a batch of native Reads. Worth noting for
  future work sessions that touch files via Edit.
- To try next: begin the brainstorm sub-projects in the recommended order.
  E (post-session knowledge ingestion) is the conceptual keystone and is
  not blocked — likely the right next pick, pending user input.
- Key learnings:
  - The existing work prompts already use `{{PROJECT}}`/`{{DEV_ROOT}}` tokens
    because `run-backlog-plan.sh` substitutes before passing to the LLM, but
    that substitution does NOT propagate to files the LLM later Reads. The
    substitution-instruction paragraph is the stop-gap bridge until the
    orchestrator owns the whole loop.
  - Both APIAnyware-MacOS and GUIVisionVMDriver `main` branches carry
    substantial uncommitted WIP. Future tasks touching these repos must
    stage specific paths, never `git add -A` or `git add .`, to avoid
    accidentally sweeping up unrelated work.
  - The racket-oo prompt had a literal `../TestAnyware/` in its key-commands
    block — evidence that the `{{DEV_ROOT}}` convention was already being
    applied inconsistently even in files the pipeline substitutes. Normalised
    as part of this commit.
  - `knowledge/README.md` was correctly noted in the backlog as unaffected
    and was not touched. Issue 4 (untracked `llm-annotate-subagent.md`) was
    also left alone per the backlog's explicit scope exclusion.

### Session 2 (2026-04-12) — Sub-project E brainstorm: ingestion model
- Attempted: the full brainstorm for sub-project E (post-session knowledge
  ingestion model), using the `superpowers:brainstorming` skill, producing
  both the design doc and the sibling implementation plan per the work
  prompt's instructions for brainstorm tasks.
- Worked: the brainstorm drove cleanly through nine design questions
  (ingestion philosophy, trigger timing, input format, pipeline shape,
  contradiction handling, confidence assignment, axis assignment, tier
  routing, interactive event scope) to a complete, user-approved design.
  The five-section design presentation flow from the brainstorming skill
  worked well — each section got confirmation before the next was started,
  and late refinements (research sessions, explorer framing, co-equal
  actors principle) landed without requiring any earlier section to be
  rewritten. Spec self-review passed with no issues. The design doc
  committed cleanly as one 870-line file (`501c15c`).
- Worked (meta): the user surfaced three load-bearing principles mid-
  brainstorm that were not in the seed memory — fresh-context-as-first-
  class-goal, human-and-LLM-as-co-equal-actors, and explorers-as-
  accountability-substrate. All three were absorbed into the design
  without disrupting earlier decisions. The brainstorm's question-at-a-
  time cadence made this easy: each new principle landed as a refinement
  of the in-flight question, not as a retroactive edit.
- Worked: the Edit-requires-native-Read friction from Session 1 was
  anticipated this time — used native Read before native Edit on the
  orchestrator's backlog.md. No retries needed.
- Didn't work / friction: none substantive. The brainstorm took nine
  clarifying questions + five design sections + a ~900-line doc write
  + sibling plan creation — a very long single work phase. Fresh-context
  principle says the next phase should unambiguously be a new session,
  not a continuation.
- To try next: reflect phase should distill the three meta-decisions
  (fresh context, co-equal actors, explorers) into the orchestrator's
  memory.md so they're load-bearing for all downstream sibling
  brainstorms, not just for E. Triage should add sub-projects I (explorer
  framework) and J (human-mode phase affordances) as new backlog items,
  and cross-link E's generated requirements into B/C/A/D/F/H so each
  sibling sub-project absorbs its share when its brainstorm runs. The
  next brainstorm pick is likely sub-project B or C — both E's immediate
  downstream dependents.
- Key learnings:
  - **Fresh LLM context as a first-class design goal** is a user
    principle that should shape architectural decisions for every
    LLM-using system in this project, not just sub-project E. Saved as
    a durable global memory (`feedback_llm_context_rot.md`).
  - **Human and LLM are co-equal actors**, not principal-and-agent. The
    user surfaced this as "we must allow for *human* reflection and
    triaging of the LLM system." It generates concrete requirements on
    sub-projects B, H, and I. Every sibling brainstorm must absorb this
    into its own decision envelope.
  - **Explorers are the accountability substrate** that makes auto-
    absorb safe. The user called this "critical" — it is not an optional
    polish layer. A full-CRUD explorer framework is load-bearing for
    the whole orchestrator UX and should be its own sub-project (I).
  - **Mnemosyne is itself an LLM client** via embedded Claude Code. The
    orchestrator spawns its own internal reasoning sessions, not just
    child plan sessions. This recursively uses sub-project C's adapter
    layer and justifies C as a real first-class abstraction rather than
    a shim. It also means sub-project C must support configurable tool
    profiles at spawn time.
  - **Existing Mnemosyne v0.1.0 preservation is not a design constraint
    for new sub-projects.** The non-disruption constraint applies to
    v0.1.0 *running* during the build, not to how new internals are
    designed. User explicitly clarified this during Section 1 review.
  - **The brainstorming skill's five-section design presentation flow
    handled late additions well** — sections 4 and 5 absorbed two
    substantive refinements (research sessions, explorer reframing,
    co-equal actors) without rewriting earlier sections. The
    section-by-section approval gates caught the additions early
    enough that they flowed into the design doc naturally.

### Session 3 (2026-04-12) — Sub-project B brainstorm: phase cycle in Rust
- Attempted: the full brainstorm for sub-project B (reimplement the work
  → reflect → triage cycle in Rust inside Mnemosyne), producing the
  design doc and the sibling LLM_CONTEXT plan per the work prompt's
  conventions for brainstorm tasks.
- Worked: the brainstorm drove through five clarifying questions
  (placeholder substitution strategy, phase state richness, human/LLM
  entry relationship, pause/takeover mechanics, interaction model) to
  a complete user-approved design. Each question presented 2–4 options
  with a recommendation and trade-off analysis; the user's responses
  were decisive and often added meta-decisions that reshaped the whole
  design (Obsidian lock-in, no slash commands in harness, dedicated
  vault with symlinks, fold knowledge under LLM_STATE with rename to
  `mnemosyne/`, plan hierarchy support). The design doc committed at
  ~2000 lines in seven sections plus a 17-item decision trail appendix.
  Mermaid diagrams used throughout per the durable Mermaid-over-ASCII
  feedback preference saved to auto-memory during Session 2's aftermath.
  Spec self-review caught a TBD placeholder, one stale
  `plan-state.toml` reference in the decision trail, and several live
  path references to the legacy `LLM_STATE/` layout that needed
  updating to `mnemosyne/plans/` after the fold-and-rename decision
  landed mid-design.
- Worked (meta): the brainstorm surfaced three cross-cutting
  architectural decisions that shaped every subsequent question:
  integration-over-reinvention (Karpathy's LLM Wiki + Infonodus
  inspiration, recorded as a seed-plan meta-principle that every
  future sub-project brainstorm must answer), hard-errors-by-default
  (recorded as durable auto-memory feedback), and Mermaid-for-diagrams
  (recorded as durable auto-memory feedback earlier but reinforced
  here). Each landed mid-session as a user nudge and was absorbed
  into the design without disrupting in-flight decisions.
- Worked (meta-meta): Path 1 (stage it: ratatui v1, Obsidian plugin
  v2) survived two serious challenges — first when the user asked
  about Obsidian-as-UI-for-Mnemosyne, then when the user flagged
  concerns about IDE integration and multi-Mnemosyne architecture.
  Both challenges were addressed by leaning on Path 1's specific
  advantages (dogfood timeline, IDE-terminal integration, multi-tab
  friction, workflow-in-Obsidian risk deserving a spike first) and
  the user locked in Path 1 with two additional supporting arguments.
  The IPC boundary hardening (via `IpcDriver` compile-time
  enforcement) is what makes Path 1 cheap — it lets v1 ship with
  ratatui while pre-committing to an additive-only Obsidian plugin
  future.
- Didn't work / friction: the brainstorm was even longer than
  Session 2 — five clarifying questions plus five design sections
  plus multiple mid-stream architectural redirects (Obsidian vault
  structure, fold `knowledge/` under `mnemosyne/`, hierarchy
  accommodation, prompts-not-docs correction) plus the 2000-line doc
  write plus the sibling plan creation plus the seed-plan state
  updates. Fresh-context principle says the next phase must
  unambiguously be a new session. The task-list tool was valuable
  here — the 9-item skill checklist kept me oriented through multiple
  direction changes.
- To try next: reflect phase should distill the seven new meta-
  decisions (hard errors default, no slash commands, Obsidian
  committed explorer, dedicated vault with symlinks, per-project
  `mnemosyne/` directory, self-containment via embedded prompts,
  Path 1 staging) into the orchestrator's memory.md so they're
  load-bearing for all downstream sibling brainstorms (already
  applied during the work phase itself; reflect should sanity-check
  the wording and prune any redundancy). Triage should update the
  sub-project ordering to reflect B being done, J being obsolete, and
  new K/L in the parallel-work track, plus add the per-project
  directory rename task to sub-project G's backlog once G's
  brainstorm runs. The next brainstorm pick is sub-project C —
  critical for B's dogfood acceptance test since C's real harness
  adapter replaces B's temporary ClaudeCodeAdapter stub.
- Key learnings:
  - **Integration-over-reinvention is a cross-cutting principle**
    that saved the project from re-inventing a knowledge explorer
    when Obsidian + Dataview covers the ground completely. The
    "what existing tool covers this ground?" question is now a
    required brainstorm output for every sub-project.
  - **The parent-process inversion is about command authority,
    not just process hierarchy.** "No slash commands in the harness"
    is the other half of the inversion — the user commands
    Mnemosyne directly, and Mnemosyne commands the harness through
    prompts and lifecycle signals. The harness has no user-facing
    command surface in v1.
  - **Obsidian-as-committed-explorer dramatically simplifies
    sub-project I.** I shrinks from "build a unified explorer
    framework" to "document which Obsidian features cover which
    Mnemosyne data surfaces." The accountability-substrate
    capability is still load-bearing; the implementation is
    delegated to Obsidian.
  - **Type-level co-equal-actors via a pluggable executor trait** is
    the elegant resolution of sub-project J's fold-or-split
    decision. J's scope was small enough (one executor implementor)
    that folding into B was clearly correct, and the trait-based
    design makes the co-equal-actors principle a compile-time
    invariant rather than a documentation promise.
  - **The substitution gap closes cleanly via a pre-rendered
    staging directory** (Option A of Q1). The alternatives (FUSE,
    MCP custom read tool, never-use-placeholders) each had real
    drawbacks that Option A avoided. The staging directory also
    gives Mnemosyne a natural interception point for sub-project E's
    ingestion pipeline (which already had to inspect `memory.md`
    after reflect).
  - **Cross-platform Obsidian + symlinks validation is a hard
    pre-implementation blocker**, not an abstract risk. The spike
    uses GUIVisionVMDriver's golden images to run the same
    validation on macOS and Linux with reproducible evidence
    committed to `tests/fixtures/obsidian-validation/`. If the
    spike fails, the entire symlink layout is redesigned to a
    hard-copy + two-way-sync fallback before any other code is
    written.
  - **Sub-project B's IPC boundary hardening is the cheapest way to
    unlock Obsidian-plugin-client as a future feature.** Shipping
    `IpcDriver` in v1 with no client attached is the compile-time
    enforcement mechanism that prevents the core from drifting into
    non-serialisable patterns. This is the design move that makes
    Path 1 (stage it) genuinely staged rather than a one-way door.
  - **The user's meta-decisions compounded productively.** Seven
    new cross-cutting architectural decisions landed during this
    brainstorm (listed above). Each was a small nudge at the time
    but shaped the whole design envelope. The brainstorming skill's
    one-question-at-a-time cadence made this work — each new
    meta-decision arrived as a refinement of the in-flight question
    rather than a retroactive edit.

### Session 4 (2026-04-12) — TheExperimentalist deletion + symlink spike promotion
- Attempted: two backlog grooming tasks bundled into a single work
  session because each is small and they share a theme (cleaning up
  backlog hidden state): (1) execute the TheExperimentalist retirement
  decision that was previously sitting unresolved as task 11, and (2)
  promote the Obsidian symlink validation spike from being a hidden
  blocker buried inside sub-project A's task description to a top-level
  `[do]` backlog task so it can run in parallel with any brainstorm
  task. User explicitly authorized both at once.
- Worked: both tasks landed cleanly in one session. TheExperimentalist
  deletion executed in three coordinated cleanups: website scrub in
  `www.linkuistics.com` (deleted `projects/theexperimentalist.html`,
  removed three product/reference cards across `index.html`,
  `k9m2x7f4w8.html`, `projects/reagent.html`), committed and pushed as
  `054f46f chore: remove TheExperimentalist — project retired` (4 files
  changed, 134 deletions); local clone `rm -rf`'d; GitHub repo
  `Linkuistics/TheExperimentalist` deleted via `gh repo delete --yes`
  (token already had `delete_repo` scope). Post-deletion verification:
  zero case-insensitive `experimentalist` matches across 42 files in
  `www.linkuistics.com`, `gh repo view` confirms 404, local directory
  no longer exists.
- Worked: symlink-spike promotion landed as a new top-level `[do]`
  task inserted immediately after sub-project C in the backlog, with
  full rationale (why it's a do-task not a brainstorm or decision, why
  it's promoted out of A, concrete execution checklist covering macOS +
  Linux via GUIVisionVMDriver golden images, Dataview + graph +
  backlinks + file watcher exercise points, reproducible evidence
  requirement at `tests/fixtures/obsidian-validation/`, and the binary
  pass-fail outcome feeding back into memory.md's vault-layout
  decision with a named fallback).
- Worked: memory.md updates for both decisions. TheExperimentalist
  entry rewritten to reflect "deleted completely" with rationale
  preserved (the Git-branching-is-wrong-abstraction reasoning is
  load-bearing for future brainstorms that touch exploratory-
  development framing). Vault-layout decision entry extended with a
  paragraph noting the spike has been promoted to a top-level task and
  why (the hidden-blocker anti-pattern).
- Didn't work / friction: none substantive. The Edit-after-native-Read
  friction from Session 1 reappeared once — I started with ctx_read on
  the three website files, then had to read them natively before
  editing. The task-list tool helped keep orientation across the
  5-file edit + delete + commit + push + repo delete + three plan-file
  updates sequence.
- To try next: reflect phase should sanity-check the rewritten
  TheExperimentalist memory entry (is the "kept for rationale" framing
  correct, or should it prune harder now that the repo is gone?) and
  the spike-promotion note (is the hidden-blocker observation durable
  enough to warrant its own generalized memory entry, or is it
  adequately captured in the single vault-layout paragraph?). Triage
  should confirm the new task ordering (is "Execute spike" correctly
  placed right after sub-project C, or should it be position 1 because
  it unblocks B's implementation regardless of C's brainstorm
  landing?). Next work pick is almost certainly sub-project C
  brainstorm — the plan's recommended next pick with B and E done, and
  now that task 11 is resolved there's no small-grooming alternative
  competing for the next slot.
- Key learnings:
  - **Hidden blockers buried inside task descriptions are a
    phase-cycle failure mode.** The triage phase may not surface them
    as executable work until the parent task runs. The remedy is to
    promote cross-cutting blockers to top-level tasks the moment they
    are identified, even if the original surfacing sub-project still
    owns the design rationale. Worth preserving as a durable learning
    — this is the second time in this plan where a cross-cutting
    concern was initially nested (the first was sub-project J folded
    into B).
  - **Bundling small grooming tasks into a single work session is
    occasionally correct** despite the "one task per work phase"
    convention. Both tasks here were <20 minutes of execution each
    and shared a theme; splitting them across two work sessions would
    have added fresh-context overhead without any reflection benefit
    because neither task produced the kind of insights that reward a
    fresh session. The convention's intent (fresh reflection after
    substantive work) is met as long as the combined session doesn't
    push past the point where context rot starts mattering. Recording
    this as a calibration data point, not a new rule.
  - **`gh` token scopes carried `delete_repo` already**, which avoided
    the interactive auth-refresh friction that would have required
    pausing the work session. Worth remembering that Linkuistics-org
    tokens on this machine are pre-authorized for destructive repo
    operations — future backlog-grooming tasks that delete repos do
    not need an auth-refresh step.
  - **The TheExperimentalist memory entry is kept after deletion**
    because the "Git-branching is the wrong abstraction for
    exploratory development" reasoning is load-bearing for the
    orchestrator's phase-cycle framing. Pruning the entry entirely
    would lose the rationale that future brainstorms may need when
    they touch exploratory-development framing (especially sub-project
    F's plan hierarchy brainstorm). Calibration for future reflect
    phases: prune facts, keep rationales.
- Follow-on (same session, post-`reflect` write): the hidden-blocker
  learning was landed back into `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md`
  as a durable process improvement — one new Tips bullet ("Promote
  cross-cutting blockers to top-level tasks immediately") and one new
  TRIAGE phase step (active scan of task descriptions for embedded
  blockers that should be promoted). Committed as
  `a25af17 Promote cross-cutting blockers to top-level tasks` in the
  LLM_CONTEXT repo and pushed to main. This is the first dogfooding
  → LLM_CONTEXT feedback loop closure since the seed plan started;
  prior work-phase friction (substitution gap, Edit-after-native-Read)
  was captured as Mnemosyne-side v2 requirements, not landed back
  into LLM_CONTEXT. Bootstrap discipline was honoured because the
  edit is additive documentation that does not change LLM_CONTEXT's
  runtime behaviour, so the four dependent projects
  (APIAnyware-MacOS, GUIVisionVMDriver, Modaliser-Racket, RacketPro)
  absorb the improvement without any migration cost. The update also
  flows naturally into Mnemosyne v1 when sub-project B's
  `include_str!`-embedded prompts vendor the canonical
  `backlog-plan.md`, so there is no duplication penalty across the
  LLM_CONTEXT → Mnemosyne transition. Key learnings from the
  follow-on:
  - **Dogfooding-validated process improvements are the right
    category to land back into LLM_CONTEXT even under bootstrap
    discipline.** The discipline forbids assuming v2 features; it
    does not forbid landing v1 improvements evidenced by v1's own
    execution. The test is whether the four dependent projects can
    absorb the change with zero migration work — additive
    documentation qualifies, runtime or schema changes would not.
  - **The LLM_CONTEXT → Mnemosyne embedded-prompts pipeline makes
    this update free.** The lesson will flow into Mnemosyne's
    binary automatically when B's implementation runs, so there is
    no "update both places" cost. This pattern — land the lesson in
    LLM_CONTEXT, let Mnemosyne absorb it at vendoring time — is
    the right default for any future process improvements until
    Mnemosyne fully subsumes LLM_CONTEXT.
  - **Extending a work phase past the `reflect` write for a small
    meta-process edit is defensible when the alternative is
    forgetting the lesson.** A strict reading of "one task per work
    phase" would defer this edit to a future work session, which
    risks losing the lesson in the fresh-context handoff. The
    extension here was ~5 minutes of two-file edits plus one commit
    and push, well inside the "no context rot yet" window. Recording
    this as a second calibration data point for when to extend vs.
    defer.

### Session 5 (2026-04-13) — Obsidian symlink validation spike (macOS + Linux) — PASS 6/6 on both
- Attempted: the `[do]` task "Execute Obsidian symlink validation spike
  (macOS + Linux)" — the load-bearing pre-implementation blocker for the
  "dedicated Mnemosyne-vault with symlinked per-project directories"
  architectural decision. Required running the fixture at
  `{{PROJECT}}/tests/fixtures/obsidian-validation/` against Obsidian +
  Dataview in VM golden images for both platforms via GUIVisionVMDriver,
  capturing per-check evidence, and recording the outcome.
- Worked: full 6/6 pass on both macOS Tahoe (`guivision-golden-macos-tahoe`)
  and Ubuntu 24.04 ARM64 (`guivision-golden-linux-24.04`). Obsidian 1.12.7
  with Dataview 0.5.67 pinned identically on both runs. Per-check evidence
  (screenshots + OCR transcripts) committed under
  `tests/fixtures/obsidian-validation/results/macos/` and `.../results/linux/`,
  each with a summary `result.md`. VERSIONS.md updated with the per-run
  record.
  - **Check 1** Dataview `LIST FROM "projects" WHERE contains(file.tags,
    "#spike")` rendered both project-side notes (`• another-note`,
    `• boundary-note`) through the symlink on both platforms.
  - **Check 2** Graph view rendered the `obsidian-spike ↔ boundary-note`
    cross-boundary edge plus the full 6-node layout on both platforms.
  - **Check 3** Backlinks panel for `knowledge/obsidian-spike.md` listed
    `projects/example/boundary-note` under "Linked mentions" with a
    "2 backlinks" footer on both platforms (one from the vault README,
    one from the project-side boundary-note).
  - **Check 4** File explorer expanded `projects > example > {README,
    another-note, boundary-note}` and opened each through the symlink on
    both platforms.
  - **Check 5** External append via `guivision exec "echo ... >>
    .../example-project/mnemosyne/boundary-note.md"` was detected by
    Obsidian's file watcher within the target window: macOS ~3s end-to-end
    (in-file search found the added text), Linux <10s end-to-end (global
    search index picked up the change).
  - **Check 6** Obsidian's standard community-plugin trust modal fires on
    first launch (plugin-trust UX, not symlink-related); accepting it
    proceeds into a normal vault on both platforms — no symlink warning,
    no refusal, no hidden subtree.
- Worked (tooling): fully driven via the `guivision` CLI — `guivision exec`,
  `guivision upload`, `guivision input {key,type,click,move}`, `guivision
  screenshot`, `guivision find-text`, `guivision agent
  {windows,window-focus}`. After the user's early correction, all VM-side
  work went through guivision exclusively. The per-VM VNC password for
  `guivision screenshot` / `guivision find-text` / `guivision input *`
  comes from the VM spec JSON file `/tmp/mnemo-<platform>-spec.json` passed
  via `--connect` — the CLI has no `--vnc-password` flag.
- Didn't work / friction:
  - **Using SSH/rsync for setup (first attempt).** I initially used
    `ssh admin@$IP` and `rsync` to populate the macOS VM instead of
    `guivision exec` and `guivision upload`. The user corrected this
    immediately: GUIVisionVMDriver exists specifically so an LLM-driven
    session drives the VM through one uniform interface; mixing SSH is
    wrong. Saved as a feedback memory
    (`feedback_guivision_cli.md`), added to MEMORY.md index. Setup was
    redone correctly via guivision after the correction.
  - **Electron-on-Ubuntu-24.04-under-tart requires `--disable-gpu`.** First
    Linux launch of Obsidian rendered an entirely black framebuffer —
    Xorg was running, gnome-shell was running, `xdotool
    getactivewindow` correctly reported the Obsidian window title, but
    `scrot` and `guivision screenshot` both showed pure black. After
    relaunching as `/home/admin/squashfs-root/obsidian --disable-gpu
    --no-sandbox`, software compositing kicked in and the display
    rendered correctly. Symptom is Electron-specific, not symlink-related.
  - **Scroll-event targeting on both platforms.** `guivision input scroll
    --dy N` and keyboard scroll shortcuts (PageDown, Ctrl/Cmd+End) did
    not reliably advance Obsidian's reading-view viewport to the
    appended line — clicks kept landing on the file explorer or the
    properties panel rather than the editor body. Workaround: use
    Obsidian's in-file search (Cmd+F) on macOS and global search
    (Ctrl+Shift+F) on Linux, both of which scroll to matches in the
    search results regardless of viewport position. Both approaches
    produced clean OCR evidence of the appended text.
  - **Negative-value flag parsing.** `guivision input scroll ... --dy -5`
    was rejected by swift-argument-parser as a missing value because
    `-5` looks like an option prefix. Workaround: `--dy=-5` or positive
    values. Not a blocker; logged for future reference.
  - **`zsh` as the default VM shell on macOS golden image.** One SSH
    command used a bash-style `===` separator which zsh parsed as an
    unknown command name. Harmless, but fixed by quoting or using
    `bash -s` for multi-line remote scripts.
  - **macOS Notification Center widgets occluding Obsidian.** The
    `guivision-golden-macos-tahoe` image has Notification Center
    widgets (Featured / Forecast / Month / "What's new in Tahoe"
    Discover popover) rendered at top-right. They sit above the
    Obsidian window area and appear to intercept some scroll events.
    Not a symlink issue but worth documenting for future macOS VM
    spikes — consider dismissing Notification Center as a setup step.
- To try next:
  - Pick **sub-project C (harness adapter layer)** as the next
    brainstorm — per memory.md's "Recommended sub-project ordering", it
    is the next brainstorm in the chain and unblocks B's dogfood
    acceptance test by letting the `LlmHarnessExecutor` stub be swapped
    for a real adapter. Alternatively pick A, D, F, H, I, or G depending
    on user preference — all are unblocked by this spike passing.
  - **Commit and push the spike evidence** before the next work phase
    runs; the fixture's committed state carries the load-bearing "vault
    layout passes on both platforms" signal to every downstream
    sub-project brainstorm.
- Key learnings:
  - **The vault-as-view-over-symlinks framing holds on both platforms.**
    Obsidian's Dataview, graph view, backlink index, file explorer, file
    watcher, and vault-open safety checks all work correctly across the
    `<vault>/projects/example → ../../example-project/mnemosyne` relative
    symlink on both macOS Tahoe and Ubuntu 24.04 ARM64, with identical
    Obsidian + Dataview versions. The hard-copy-staging fallback
    recorded in memory.md does NOT need to be activated. Sub-project A
    and sub-project B's implementation plans can proceed on the
    symlinked-vault baseline without blocking on further investigation.
  - **guivision is the right tool for spike evidence.** Every legitimate
    piece of evidence in this run came through `guivision <subcommand>`.
    SSH was useful as a debugging convenience but is not acceptable as
    a production automation path — evidence produced via SSH is evidence
    about SSH, not about the tool-under-test. This is saved as a
    durable feedback memory
    (`feedback_guivision_cli.md`) so future VM-driven work defaults
    correctly.
  - **Electron on ARM64 Linux in virtio-gpu needs software rendering.**
    If a future spike or test run launches any Electron app inside
    `guivision-golden-linux-24.04`, it should pass `--disable-gpu
    --no-sandbox` up front. The `--no-sandbox` alternative is not
    needed if `chrome-sandbox` has its suid bit restored after
    AppImage extraction. Worth considering whether to bake this into
    the golden image itself so future runs don't re-learn the lesson.
  - **Identical Obsidian + Dataview pins survived the cross-platform
    run unchanged.** Obsidian 1.12.7 shipped as a cask on macOS and as
    an ARM64 AppImage on Linux, and Dataview 0.5.67 from the pinned
    GitHub release tag installed identically via `setup-vault.sh`'s
    `curl` step. The fixture's pinning discipline works end-to-end.
  - **OCR-based evidence capture is robust enough to drive CI-style
    acceptance.** All six checks passed via a combination of `guivision
    screenshot` + `guivision find-text` (Vision OCR on the captured
    framebuffer). No manual screenshot inspection was needed for
    pass/fail, only for sanity verification. This is a viable template
    for automating future symlink / vault / Obsidian spikes as the
    fixture set grows.
  - **The task took more than one obvious phase's worth of work but
    remained one logical unit.** Running both platforms in a single
    work phase is an exception to the "one task per work phase" rule,
    justified because the spike outcome is inherently cross-platform
    (a per-platform PASS is not a meaningful result on its own — the
    architectural decision is binary on the AND of the two). The
    fresh-context cost was acceptable because neither run produced
    new learnings that invalidated the other run's setup, and the
    evidence artifacts are fully reproducible from the committed
    fixture.

### Session 6 (2026-04-13) — Sub-project C brainstorm: harness adapter layer
- Attempted: the full brainstorm for sub-project C (harness adapter layer
  abstracting Claude Code, Codex, Pi, and future harnesses), using the
  `superpowers:brainstorming` skill, producing both the design doc and
  the sibling implementation plan per the work prompt's instructions for
  brainstorm tasks. C is the next-most-critical brainstorm pick after the
  completed B/E pair because it's the swap-target for B's `LlmHarnessExecutor`
  stub and therefore on the critical path for B's v1 dogfood acceptance test.
- Worked: the brainstorm drove cleanly through five locked decision points
  (Q1 process model, Q2 module location, Q3 warm-pool deferral, Q4 fixture
  format, Q5 tool-profile flag mapping) plus two mid-design revisions
  (Section 2 actor architecture rewrite, Section 2 micro-revision adopting
  crossbeam-channel) plus two post-write user clarifications (the "no
  callback channel" rule disambiguation and the sentinel-driven task-level
  completion design). Spec self-review passed cleanly on both the initial
  write (1251 lines, commit `71fd307`) and the post-write amendment
  (+60 lines to 1311, commit `b1a8cea`). Sibling plan committed as `9dac743`
  with 24 unconditional implementation tasks plus 2 conditional warm-pool
  tasks gated on the C-1 dogfood acceptance criterion.
- Worked (meta): the user surfaced two genuinely architecture-improving
  pushbacks during the design presentation. Both made the design *better*,
  not just different:
  - **The actor-style threading model**: I initially sketched
    interior-mutability with `Mutex<Option<Child>>` etc. The user's
    Erlang/Elixir preference for message-passing through channels turned
    out to be the structurally-cleaner answer here — defence-in-depth
    tool enforcement has a natural home in the actor's event-handling
    switch, no shared mutable state simplifies reasoning, and the BEAM
    heritage is load-bearing for the maintenance story rather than
    ornamental. Three threads per session (actor + stdout-reader +
    stderr-reader) costs little on Linux/macOS and the duplication is
    shallow.
  - **Process-group termination as a v1 correctness requirement**: I
    initially had it as a "known v1 limitation" with a v1.5 follow-up
    note. The user pointed out that orphaned subprocess / leaked-port
    bugs are nearly invisible until they bite, and doing it right once
    is much cheaper than chasing the leak later. Pulled the work into
    v1: `process_group(0)` at spawn (stable Rust `CommandExt`),
    `nix::sys::signal::killpg` at terminate, two-phase SIGTERM→SIGKILL
    escalation with 500ms grace.
- Worked (meta): the post-write `/clear` insight from the user reshaped
  the v1.5 warm-pool design from "spawn fresh processes per pool entry"
  into "reset and reuse existing processes via Claude Code's session-
  reset mechanism", changing what could be a 600-line pool manager into
  potentially 30 lines once the spike validates the reset path. Captured
  as the §7.4 three-check spike protocol (structured envelope → `/clear`
  text → pre-spawned single-use degradation) gated on the C-1 acceptance
  test failing. The user's casual "I wonder if we can use /clear" turned
  into the entire warm-pool implementation strategy.
- Worked (meta): the post-write user clarification on "no callback channel"
  was the single most consequential intervention of the session. My initial
  spec phrasing collapsed two distinct concerns — *control* (forbidden:
  harness calling Mnemosyne) and *observation* (allowed and necessary:
  Mnemosyne reading harness state) — into one rule. The user's gentle
  pushback ("I think a harness to Mnemosyne channel would be useful, not
  least because we want to know when the harness has reached a stopping
  point") forced the disambiguation, which led to adding `OutputChunkKind::SessionLifecycle`
  as a fourth amendment to B's trait for protocol-level state observation.
  The user's followup ("I would think the prompts include 'when finished
  say READY FOR THE NEXT PHASE'") then introduced the genuinely-correct
  task-level completion mechanism: prompt-instructed sentinel strings
  detected by B's executor via a sliding-buffer matcher. The two
  clarifications together added a fifth cross-sub-project requirement
  (sentinel detection in B's executor) and made the spec strictly more
  faithful to the original architectural intent.
- Didn't work / friction: I approached the original Section 2 with an
  "interior mutability + Mutex wrappers" sketch that needed two rounds
  of revision (actor model → crossbeam adoption) before settling. Could
  have caught the actor-model fit during initial design if I'd taken
  the user's BEAM preference more seriously upfront — it was visible in
  prior session logs. The lesson: when the user has a stated architectural
  preference recorded in feedback memory, lead with it during the first
  draft instead of presenting the obvious-Rust answer and waiting for
  the pushback.
- Didn't work / friction: my initial framing of "no callback channel
  from harness to Mnemosyne" was sloppy enough to confuse the user about
  whether observation was permitted. The architectural rule from
  `mnemosyne-orchestrator/memory.md` is about *control* not *observation*,
  but I phrased it as if the two were the same thing. Fixed in commit
  `b1a8cea` with explicit disambiguation in §3.3.
- To try next: pick the next brainstorm task. Per the orchestrator memory
  (recommended sub-project ordering), A (DEV_ROOT global store) is the
  natural next pick — it's small-medium, independent, and B has already
  fixed the vault-as-view-over-symlinks framing so A's scope is now
  "design the vault location, config override, and bootstrap" rather
  than "design the vault from scratch". F (plan hierarchy) is also
  ready, as is L (the small Obsidian terminal-plugin spike) which is
  attractively independent. M (Observability), the new sub-project
  surfaced in this session, is also parallel-able and has the strongest
  cross-cutting impact — it's worth picking earlier rather than later
  because every other sub-project's structured-logging needs route
  through M's framework once it lands. User input will decide.
- Key learnings:
  - **Fresh-context brainstorms can amend earlier-completed siblings'
    interfaces.** Sub-C's brainstorm produced four trait amendments to
    sub-B's draft `HarnessSession` surface, all forced by Q1 (bidirectional
    stream-json) which post-dated B's brainstorm. The amendments are
    additive and B is in "brainstorm done, implementation not started"
    status, so they land in B's implementation phase without requiring a
    B re-brainstorm. This is the project's "cross-sub-project requirements
    threaded through fresh-context sessions" pattern in action — and it's
    the reason fresh-context sessions are a feature rather than a cost:
    each brainstorm gets to amend earlier interfaces with the benefit of
    newer constraints, rather than carrying every old assumption forward
    unexamined.
  - **The "no callback channel" rule needs careful scope discipline.**
    It rules out the harness *calling* Mnemosyne (slash commands,
    programmatic callbacks, LLM-invoked Mnemosyne actions) — but does
    NOT rule out Mnemosyne *observing* harness state and reacting on
    its own side. The bidirectional stream-json output is the canonical
    observation channel; `SessionLifecycle` chunks surface protocol-level
    state transitions (ready / turn complete / exited); sentinel detection
    in B's executor surfaces task-level "the LLM has finished the work"
    signals. All three are observation, none are control. Future sub-project
    brainstorms that touch the harness boundary should consult §3.3 of
    C's spec before re-framing this.
  - **Protocol-level "turn over" and task-level "I am done with the work"
    are different signals.** Claude Code's `result` event tells you the
    model stopped emitting tokens for this round; the LLM judging itself
    done with the task is a separate signal that requires the LLM's own
    self-assessment, surfaced via prompt-instructed sentinel strings.
    Conflating the two would cause Mnemosyne to transition phases the
    moment Claude Code finished a single turn even when the LLM was
    mid-task. Sentinel detection lives in B (not C) because sentinels
    are coupled to phase prompts and the mechanism is harness-agnostic.
  - **Always-on instrumentation is cheaper than gated instrumentation.**
    The `SpawnLatencyReport` is emitted on every session, written to
    `<staging>/spawn-latency.json`, and surfaced as an `InternalMessage`
    chunk — no debug flag, no env var. The cost (4 timestamps + 1 channel
    send + 1 file write per session) is so small it doesn't merit a flag,
    and the benefit (no "I wish I had measurements from that one weird
    session" moments) compounds across the entire dogfood cycle and
    beyond. The "make the C-1 acceptance gate measurable by default"
    framing turned out to be a generally-useful instrumentation discipline
    that should propagate to other sub-projects.
  - **Tactical instrumentation should explicitly disclaim being a
    framework.** C's `SpawnLatencyReport` is purpose-built for the C-1
    gate and is documented as a tactical seed, not the start of a
    metrics framework. The proposed sub-project M (Observability) is
    the future home for the broader story. This separation prevents
    the tactical artifact from accreting framework-shaped scope creep,
    and gives M's brainstorm full freedom to pick its own structured
    logging crate, event bus shape, and migration path. Surfacing M as
    a new sub-project candidate this session is itself a win — it would
    have been very easy to silently grow C's spec into a framework
    without realising it.
  - **The brainstorm session model handles late-arriving pushbacks
    gracefully when the spec is short and consistent.** The post-write
    user clarification round added a fourth trait amendment, a new
    enum variant, three documented text formats, a new §4.3.3 subsection,
    and an entire fifth executor-level requirement back to B — all
    landing as a clean amendment commit (`b1a8cea`, +60 lines) without
    touching any earlier section's logic. The structural symmetry of
    the spec (§3 declarations → §4 implementation → §11 cross-sub-project
    requirements → Appendix A decision trail) made each piece trivially
    locatable. The lesson: spec doc structure that lets each amendment
    land in exactly the sections that "want" it is much easier to
    maintain than a spec where a small change ripples through many
    unrelated sections.

### Session 7 (2026-04-13) — sub-project M (Observability) brainstorm

- **Attempted:** Brainstormed sub-project M (the observability framework
  surfaced during sub-project C's brainstorm in Session 6). Picked M over
  the default top-priority sub-project A on the cross-cutting argument:
  every other sub-project's structured-logging needs route through M, so
  brainstorming M before B / D / E / etc. enter heavy implementation
  avoids retrofitting tactical instrumentation seeds onto the framework
  later. Used the `superpowers:brainstorming` skill end-to-end: explored
  context (B / C / E design docs, current Mnemosyne crate which has
  zero existing structured-logging dep), drove two focused clarifying
  questions, presented the full design in one pass after the user
  signalled "use existing tooling and libraries wherever possible — this
  is not an interesting task", got approval, wrote the spec doc, did
  spec self-review (fixed three small inconsistencies), got user
  approval on the written spec, then created the sibling LLM_CONTEXT
  plan with all seven plan files plus the 23-task implementation
  backlog. Finally landed M's adoption tasks directly into the three
  existing sibling backlogs (sub-B, sub-C, sub-E) per §10 of the design
  doc — M owns its own cross-plan adoption coordination rather than
  deferring it to triage.
- **What worked:**
  - **The "single architectural fork question" framing.** Q1 (north
    star: diagnostic / live / analysis / all three) and Q2 (foundation:
    pure tracing / pure typed bus / hybrid) collapsed the full
    observability design space into two real decisions. Once the user
    answered "all three balanced" + "hybrid", the rest of the design
    fell out mechanically — crate stack, layer composition, storage
    layout, metric catalogue, and migration strategy were all forced
    by those two answers. Ten queued clarifying questions (storage
    shape, scope keys, display surfaces, analysis tooling, ...) all
    had standard-tool answers under the user's "not an interesting
    task" steer and didn't need separate rounds.
  - **User steer dramatically accelerated the brainstorm.** "Use
    existing tooling and libraries wherever possible. This is not an
    interesting task." was the signal to stop framing every decision
    as a fork and start picking the boring/standard answer at every
    one. Compressed what would have been 5-8 more clarifying rounds
    into a single design-presentation message. The brainstorming
    skill's checklist was respected (every step happened) but the
    rounds collapsed where the user explicitly de-prioritised the
    interesting-architectural-decision framing.
  - **Cross-plan adoption tasks landed immediately as part of the
    brainstorm output, not deferred.** §10 of the design doc commits
    M to owning its own adoption coordination ("M's deliverable, not
    triage scope"). Concretely: this brainstorm appended adoption
    tasks to sub-B-phase-cycle/backlog.md, sub-C-adapters/backlog.md,
    and sub-E-ingestion/backlog.md before stopping the session.
    Tasks for sub-D / sub-F / sub-H / sub-I / sub-G are queued in
    sub-M's memory.md with a triage rule that lands them as those
    sibling brainstorms complete. This closes a coordination gap that
    would otherwise have forced triage to discover the requirement
    later.
  - **Hybrid tracing + typed events is the right architectural
    answer for Mnemosyne's two competing principles.** "Integration
    over reinvention" pushes hard at `tracing` (the de facto Rust
    standard, mature ecosystem, span semantics for free). "Every
    state transition is a typed message; hard errors by default"
    pushes hard at a custom typed-event-enum bus. The hybrid honours
    both: typed `MnemosyneEvent` enum at the Mnemosyne boundary
    (downstream consumers exhaustively pattern-match), `tracing`
    transport / spans / async-instrumentation below the boundary
    (stock `tracing-subscriber` Layer pattern, third-party crate
    events flow through automatically). Custom code is bounded to
    one ~200-line `Layer`. This pattern generalises to any future
    cross-cutting sub-project that has the same two-principle
    tension.
  - **The staged migration of C's `SpawnLatencyReport`** (parallel-
    emit window with mechanical ±10ms verification) is exactly the
    pattern future tactical-seeds-becoming-framework-features should
    follow. C v1 ships the tactical writer; M v1 lands the parallel
    `metric!` calls; M v1.1 deletes C's writer after the verification
    window passes; sub-G's migration deletes the staging-schema entry.
    Every step is independently reversible until the verification
    proves M's data matches C's ground-truth. This pattern should
    generalise to B / D / E if they grow tactical seeds before M
    lands.
  - **C's "observability-friendly" architectural posture turned out
    to be load-bearing.** C's design deliberately documented
    "every state transition is a typed message, every error is a typed
    variant" without committing to a logging crate. That posture
    meant retrofitting M onto C is purely additive (`tracing::instrument`
    on actor handlers, `metric!` calls at three measurement points)
    rather than requiring a redesign. This validates the "leave
    yourself observability-friendly without committing to a framework"
    discipline for future sub-projects that ship before M's
    framework lands.
- **What didn't work / what surprised:**
  - **Initial framing of M as "interesting and exotic" was wrong.**
    My queued clarifying questions assumed observability would need
    multiple rounds of architectural fork decisions (storage shape,
    scope strategy, display surfaces, analysis tooling, ...). The
    user's steer correctly diagnosed that none of these were
    interesting — they all had standard-tool answers under the
    "integration over reinvention" principle once the foundation
    decision (tracing + typed events hybrid) was made. Lesson for
    future cross-cutting brainstorms: after the architectural fork
    question lands, check whether the remaining decisions are
    actually forks or just "pick the standard tool" — and if the
    latter, collapse them into a single design-presentation message
    rather than asking N more rounds.
  - **The `mnemosyne_event!` macro typed-payload handoff is
    open-ended in a way the spec doesn't fully resolve.** Spec §16 Q1
    leaves the handoff approach (thread-local trick vs `Visit` API +
    serde round-trip) as a day-1 microbenchmark for the implementer
    to pick. Both approaches will work; one will be faster. This is
    the right level of openness — committing to a specific approach
    in the spec without measurement would be premature optimisation
    in either direction. Implementation-phase task 5 owns the
    decision.
- **What to try next:** Sub-project A is the natural next pick — it
  remains the highest-priority not-started brainstorm with simplified
  scope (B fixed the vault layout, A finalises location / config
  override / init flow). After A, the next brainstorm pick is
  judgement: F (plan hierarchy, must respect B's contracts), D
  (concurrency, soft-dependent on A's vault root), or H (skills fold,
  mechanical). All three are independent enough to run in any order;
  pick by topic affinity / momentum at the next work session.
- **Key learnings:**
  - **"Use existing tooling and libraries wherever possible" is a
    project-wide principle for cross-cutting concerns.** When a
    sub-project's scope is "design infrastructure that everything
    else uses," the right default is to pick the standard tool at
    every decision and bound the custom code as tightly as possible.
    M's custom code is one ~200-line `tracing-subscriber::Layer`;
    everything else is composing standard layers. This sets the
    reference point for future cross-cutting brainstorms (D, F, H, I)
    — they should ask "what existing crate covers this ground" before
    asking "what should we build."
  - **"M owns its own cross-plan adoption" is a brainstorm-output
    discipline, not just a §10 documentation note.** The brainstorm
    actually appended adoption tasks to three sibling backlogs as
    part of the session output, before stopping. Triage was not
    asked to schedule the coordination — the brainstorm itself
    delivered it. This is the right pattern for any future
    cross-cutting sub-project: the brainstorm session that produces
    the design doc also produces the adoption stubs in every
    affected sibling backlog. Future brainstorms (D, F, H, I, etc.)
    should follow this discipline if their scope is cross-cutting.
  - **Hybrid (tracing + typed events) generalises beyond
    observability.** The architectural pattern — own type discipline
    at the boundary, lean on `tracing` for everything below the
    boundary — is reusable for any future component that has typed
    state transitions and wants spans / async-context / third-party
    integration without a custom event bus. This is worth recording
    in the orchestrator memory as a project-wide pattern, not just
    a sub-M decision.
  - **Brainstorming a cross-cutting sub-project early pays for
    itself in avoided rework.** M was promoted from the tail of
    the backlog to position 2 in Session 6's triage specifically
    because every other sub-project's structured-logging needs
    route through M. Brainstorming M before D / F / H / I means
    those sub-projects' implementation phases pick up M's
    framework directly rather than retrofitting later. The cost
    is paid now (one brainstorm session); the benefit compounds
    across every other sub-project's implementation.
  - **Skill-driven brainstorm + user-driven steer is a strong
    combination.** The `superpowers:brainstorming` skill provided
    the structural discipline (explore → questions → approaches →
    design → spec → review → plan); the user's mid-brainstorm steer
    ("not an interesting task") provided the velocity-shaping
    decision. Neither alone would have produced this session — the
    skill kept the structure intact while the user's steer kept the
    pace honest. Lesson: skill checklists are fine but they're not
    a substitute for the user's judgement about which decisions
    deserve careful exploration vs which deserve a one-pass
    standard-answer pass.

### Session 8 (2026-04-13) — sub-project A brainstorm: vault location, discovery, and bootstrap
- Attempted: the full brainstorm for sub-project A — design the
  Mnemosyne vault's directory layout, how the binary discovers it,
  how `mnemosyne init` bootstraps a fresh one, how existing
  `~/.mnemosyne/` installations migrate, git-tracking policy,
  Tier 1 / Tier 2 addressability for E, and the cross-sub-project
  contracts A locks.
- Worked: five forking decisions were driven through clarifying
  questions before any design surface was presented —
  (1) discovery model (explicit env var → user config → flag,
  no walk-up),
  (2) init flow shape (two non-interactive subcommands: fresh and
  clone),
  (3) gitignore policy (track knowledge + archive + curated
  `.obsidian/`; gitignore runtime, cache, projects symlinks,
  workspace files),
  (4) migration scope (dropped entirely — user confirmed no real
  v0.1.0 usage),
  (5) vault identity (schema-versioned `mnemosyne.toml` marker,
  later merged with the optional-override config into a single
  file at user suggestion).
  After the five locks, five design sections were presented for
  section-by-section approval (layout, discovery, init, Tier 1 /
  Tier 2, cross-sub-project contracts). Every section was
  approved with at most one round of revision. The brainstorm
  produced the authoritative spec at
  `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-A-global-store-design.md`
  (commit `c81fd48`) and the sibling plan at
  `{{PROJECT}}/LLM_STATE/sub-A-global-store/` with a
  fifteen-task implementation backlog (commit landed in this
  session after the spec).
- Didn't work / friction: one mid-design correction about the
  symlink target (pointing at `<project>/` vs `<project>/mnemosyne/`)
  was applied and then walked back when the user re-checked
  memory.md and confirmed `<project>/mnemosyne/` is the renamed
  `LLM_STATE/`, not a single plan. Lesson: when "revising" a
  decision from memory.md, the first action should be re-reading
  the entry to confirm the current framing, not "how would I
  improve it." Memory entries are dense and easy to misread under
  reasoning pressure.
- To try next: pick another brainstorm from the orchestrator
  backlog. Highest-priority unblocked candidates are F (plan
  hierarchy), D (concurrency — soft-depends on A, now done), H
  (skills fold-in), I (Obsidian coverage), or G (migration
  strategy, parallelisable throughout). L (terminal plugin spike)
  is also fully independent and small.
- Key learnings:
  - **"No real users" is a compounding simplification.** The
    single constraint dropped an entire CLI subcommand
    (`migrate`), an entire decision dimension (preserve-vs-fresh
    git history), eliminated eight hardcoded `~/.mnemosyne/` paths
    in `main.rs` as deletable rather than transitionable, and
    killed one config file format (the v0.1.0 YAML `config.yml`
    → moved to embedded defaults + optional overrides inside
    `mnemosyne.toml`). Cross-sub-project sub-project brainstorms
    should test their scope assumptions against "what if there
    are no real users of the thing I'm replacing" — the answer
    often drops migration scope wholesale.
  - **Merging the vault marker file into the config file was a
    late improvement.** The initial design had `.mnemosyne-vault`
    (marker) + `config.toml` (optional overrides) as two separate
    files. User pointed out the dotfile-without-extension was
    awkward and suggested merging them into `mnemosyne.toml` at
    the vault root. This mirrors `Cargo.toml`'s "project marker +
    config" pattern, eliminates the dotfile, and costs nothing
    because the key spaces never overlap. Small moves like this
    matter — they clarify the design for human readers without
    changing the semantics.
  - **Each clarifying question should lock a specific fork
    before moving on.** Earlier brainstorms (M) sharpened this
    pattern. Sub-A followed it consistently: Q1 (discovery model)
    → Q2 (init flow) → Q3 (gitignore) → Q4 (migration) → Q5
    (marker). By the time the five design sections were presented,
    every section's content was already determined by the locked
    decisions; sections were presentation + verification, not
    fresh design discussion. Five questions in one session
    produced a fully specified spec.
  - **Cross-cutting adoption stubs are now a reflex.** Sub-A's
    sibling plan includes task 13: author `mnemosyne_event!`
    calls at vault discovery, init, and adopt-project boundaries
    as a placeholder for when sub-M's framework lands. This is
    the "cross-cutting brainstorms own their own sibling adoption
    stubs" discipline established by M, now applied by A to M.
    The pattern scales: sub-A is consumed by M in the other
    direction from how M was authored, but the stub obligation
    flows both ways. Every future brainstorm should continue
    checking M's adoption stubs list against its own boundaries.
  - **The spec dropped from ~12 questions to 5 because the user
    stopped one round to say "we have no existing mnemosyne
    usage."** A useful pattern: when a brainstorm starts
    generating questions about migration, backwards compat, or
    transition strategy, check whether those scopes are real
    before driving them through the forks exercise. A single
    well-placed constraint from the user can collapse hours of
    unnecessary design work. In this session it landed at Q4
    and compressed the remaining forks significantly.

### Session 8 (2026-04-14T08:52:02Z) — LLM_CONTEXT 2026-04 overhaul reconciliation

- **Attempted:** Analyse LLM_CONTEXT's post-overhaul shape (commits
  `9513ca3` "Rewrite create-plan.md for phase-file-factored layout",
  `9eb6602` "Add phases/ directory", `d6f215b` "Move coding-style files
  under fixed-memory/ and add memory-style.md", `096d8dc` "Rewrite
  run-plan.sh for shared-phase composition", `92d5ab2` rename
  `run-backlog-plan.sh` → `run-plan.sh`, `7bb39b0` "Absorb backlog-plan.md
  into README", plus the four-phase cycle introduction and
  `pre-work.sh` hook additions) and reconcile all six generated
  Mnemosyne plans (`mnemosyne-orchestrator`, `sub-A-global-store`,
  `sub-B-phase-cycle`, `sub-C-adapters`, `sub-E-ingestion`,
  `sub-M-observability`) with the new upstream shape. Path 1 per the
  work-phase branch decision: fix the drift here and now rather than
  deferring.

- **What worked:**
  - Classified the impact as mostly-additive to Mnemosyne's merge
    plan — no design decisions invalidated, just stale filenames and
    a three-phase-shaped state machine that needs widening.
  - Converted 24 absolute `/Users/antony/Development/...` references in
    `mnemosyne-orchestrator/prompt-work.md` and
    `sub-M-observability/prompt-{work,reflect,triage}.md` to
    `{{DEV_ROOT}}` / `{{PROJECT}}` / `{{PLAN}}` placeholders. The
    remaining four sub-plan `prompt-work.md` files (sub-A, sub-B,
    sub-C, sub-E) were already clean.
  - Fixed the `{{DEV_ROOT}}/GUIVisionVMDriver` reference in
    `sub-B-phase-cycle/related-plans.md` to use the placeholder.
  - Deleted `mnemosyne-orchestrator/related-plans.md` — the four
    LLM_CONTEXT projects listed there were a non-disruption
    constraint, not a cross-plan propagation relationship. That
    constraint is already captured in `prompt-work.md` and
    `memory.md`, so the related-plans file was overstating its role.
  - Rewrote three orchestrator memory.md entries with drift:
    "Self-containment from LLM_CONTEXT/ via embedded prompts" (new
    vendor list: `phases/{work,reflect,compact,triage}.md`,
    `fixed-memory/{coding-style,coding-style-rust,memory-style}.md`,
    `create-plan.md`), "Phase cycle reimplemented in Rust" (widened
    to four phases with compact-trigger semantics),
    "run-backlog-plan.sh substitutes..." → "run-plan.sh substitutes
    `{{DEV_ROOT}}` / `{{PROJECT}}` / `{{PLAN}}` / `{{RELATED_PLANS}}`".
    Added one new entry: "`fixed-memory/memory-style.md` is the single
    source of truth for memory entry rules" (read by both reflect and
    compact; B must vendor it; the compact lossless-rewrite contract
    is only meaningful relative to that stable rule set).
  - Amended the orchestrator G (migration) task description with the
    new per-plan artefacts G now inherits: `pre-work.sh`,
    `prompt-compact.md` / `prompt-reflect.md` / `prompt-triage.md`
    preservation, script-managed `compact-baseline`,
    `related-plans.md` schema change, and the phase enum widening to
    include `compact`.
  - Landed a new `[amendment]` task in sub-B's backlog:
    "Absorb LLM_CONTEXT 2026-04 overhaul into B's design + types".
    It gates "Define core abstractions and types" (parallels the
    existing Sub-C trait amendments task) and enumerates eight
    concrete spec edits: four-phase cycle, phase enum widening,
    phase composition mechanism, `pre-work.sh` hook contract,
    `{{RELATED_PLANS}}` placeholder, `related-plans.md` schema,
    ISO 8601 timestamps, vendor list. The existing "Implement
    embedded prompts module" task and "Implement placeholder
    substitution" task were rewritten against the new contract;
    "Implement PhaseRunner::run_phase" test coverage grew to cover
    both the compact-skipped and compact-run branches; the dogfood
    task references were updated from `run-backlog-plan.sh` to
    `run-plan.sh` and from "work → reflect → triage" to "work →
    reflect → (compact) → triage"; `ManualEditorExecutor`'s
    phase-target-file mapping gained the compact row.
  - Added four new entries to sub-B's memory.md quick-reference:
    "The cycle is four phases", "Vendored upstream files",
    "Phase composition: shared phase file + optional per-plan
    override", "Optional `pre-work.sh` executable hook". Also
    extended "The substitution gap closer" with `{{RELATED_PLANS}}`
    as the fifth placeholder.
  - Rewrote `sub-M-observability/prompt-triage.md`'s "Cross-plan
    adoption coordination" step to use dispatched subagents instead
    of instructing triage to read the parent orchestrator backlog
    directly. The direct-read pattern conflicts with the new
    triage discipline: "Never read foreign plan content — cross-plan
    awareness comes from the Related plans block and dispatched
    subagents." M's §10 cross-plan ownership obligation survives;
    the mechanism shifts to subagents. Sibling awareness of the
    parent orchestrator plan comes free from the auto-discovered
    `{{RELATED_PLANS}}` block.

- **What didn't:** Nothing abandoned. The work is pure reconciliation —
  no tests run, no code compiled, just spec/backlog/memory/prompt
  editing. The "no code" posture is by design for the overhaul task.

- **Suggests trying next:**
  - F is now the natural next brainstorm pick per the original
    orchestrator backlog ordering, with no blocker from the overhaul
    reconciliation. F should explicitly confirm it inherits the
    `related-plans.md` sibling auto-discovery semantics (i.e., F's
    plan-hierarchy discovery algorithm is the sibling-discovery
    algorithm LLM_CONTEXT uses today, generalised to nested plans).
  - Sub-B's new `[amendment]` task is work that must land before
    B's core-types implementation starts. Whoever picks B up next
    does the amendment task first.
  - Deferred consideration: whether the overhaul reconciliation
    should have updated the sub-E or sub-A memory.md files. Search
    showed neither plan had stale filename or cycle-shape
    references; they already use `{{DEV_ROOT}}` placeholders. No
    action was needed there and none taken.

- **Key learnings / discoveries:**
  - **The embedded-prompts vendor list is the forward-compatibility
    chokepoint for LLM_CONTEXT evolution.** Every future LLM_CONTEXT
    change that adds or renames a shared file forces a corresponding
    update in sub-B's `prompts.rs` constants. This argues for either
    (a) a build-time glob-and-embed macro that reads LLM_CONTEXT's
    tree directly, or (b) a dedicated "vendor-list reconciliation"
    task in B's backlog that re-derives the embedded file set from
    upstream at implementation time. Option (a) would also catch
    LLM_CONTEXT changes during B's own development, which is the
    most common failure mode. Both defer to B's implementation
    phase; captured in B's new memory entry "Vendored upstream
    files (embedded prompts)" but not yet a separate task.
  - **The new triage discipline (never read foreign plan content)
    forces cross-plan coordination to flow through subagents.** M's
    §10 cross-plan ownership obligation was written assuming
    direct reads of the parent backlog; the new discipline breaks
    that. The fix was mechanical — subagent dispatch — but the
    pattern is general: any future plan that wants to react to
    another plan's state must do so via subagents, not by reaching
    into the foreign plan's files. Worth watching when sub-F and
    sub-D brainstorms run, both of which will likely want
    cross-plan awareness.
  - **Drift in `memory.md` and `backlog.md` becomes load-bearing
    fast because both files are read by LLM phases.** Drift in
    `session-log.md` is fine — that file is never read by any LLM
    phase. The rule: fix drift aggressively in memory/backlog, leave
    session-log alone. This session explicitly left
    `session-log.md` historical references to `backlog-plan.md`
    untouched because they are accurate historical record of what
    was true at the time.
  - **Path placeholders vs. absolute paths is not cosmetic.** The
    scan found eight files with stale absolute paths; converting
    them to placeholders makes the plans portable to different dev
    roots and matches the pattern the four sub-plans brainstormed
    after the placeholder convention had settled (sub-A, sub-B,
    sub-C, sub-E) already follow. The four correct plans and the
    two incorrect ones (`mnemosyne-orchestrator`,
    `sub-M-observability`) are split on brainstorm date, not
    author.
  - **"Path 1: fix now" vs "Path 2: defer to reflect/triage" was the
    right call** for an overhaul of this size because the drift
    touches six plans' load-bearing files. A reflect/triage pass
    would have had to re-scan and re-derive the impact analysis from
    memory alone, which compounds the drift risk. Fix-at-source is
    the right default for upstream-infrastructure overhauls.

### Session 9 (2026-04-14T11:48:52Z) — sub-F brainstormed; architecture pivoted to BEAM daemon

- **What was attempted:** Drive the sub-F brainstorm from its narrow original scope ("plan hierarchy + root plan") through to a committed v1 architecture, producing a design doc plus comprehensive documentation overhaul (README, architecture.md, user-guide.md, configuration.md) for sharing the vision with others.

- **What worked:**
  - F's brainstorm expanded organically from plan hierarchy to the full v1 architecture. Each user push-back drove a deeper simplification: collapsing `plans/` → `project-root/`, removing cached qualified IDs, pivoting to project-level routing agents for cross-project dispatch, recognizing experts as actors, and ultimately committing to a persistent actor daemon on BEAM.
  - The sunk-cost analysis for BEAM commit was decisive — user's observation that "less than a day of Rust code is written and the brainstorm has taken less than a day" reframed the choice from "should we pivot?" to "when is pivoting cheapest?" — and the answer was "right now."
  - Documentation overhaul happened in the same work phase: F design doc (~2000 lines), new README with Mermaid diagrams, new `docs/architecture.md` as the comprehensive architectural overview, rewritten user-guide and configuration docs. All four docs frame Mnemosyne as a persistent daemon orchestrator, not a knowledge-store CLI.
  - Memory and backlog updated to reflect new architectural commitments, new sub-projects (N, O, P), and amendment tasks for every affected sibling (A, B, C, D, E, G, H, I, M).
  - The brainstorm itself exercised the insight that a design process "keeping arriving at OTP primitives" is the universe signaling which runtime to use. Every actor-model mechanism we were about to hand-roll (supervision, mailboxes, let-it-crash, message routing, distribution transparency) is a free BEAM primitive. Committing to Elixir reduces F's implementation effort by an estimated 2-3x relative to Rust-with-actix.

- **What didn't work / what was deferred:**
  - Sub-F sibling plan scaffolding was **deferred** because F's implementation plan depends on the BEAM PTY spike outcome (can `erlexec` cleanly spawn Claude Code?). Rather than write a plan in a language we might not use, the brainstorm committed to the BEAM spike as the critical next step.
  - Amendment tasks for A, B, C, D, E, G, H, I, M are written to the orchestrator backlog but not yet dispatched to the siblings' own backlogs — that's what F's own triage phase would normally do, but this work phase focused on landing the design doc + documentation overhaul, so amendment-dispatch is deferred to the next cycle.
  - The Rust v0.1.0 CLI code is logically retired but not physically deleted. The deletion is scoped into sub-G's migration plan.

- **What this suggests trying next:**
  - **BEAM PTY spike immediately.** This is the critical blocker for sub-F sibling plan scaffolding and for the whole v1 implementation path. A few hours of `erlexec` experimentation will either unblock the straightforward path or drive us to the Rust-PTY-wrapper fallback.
  - After the spike: scaffold F's sibling plan with its Elixir-specific implementation tasks, then begin landing the amendment tasks for A/B/C/E/M so those done brainstorms are consistent with the new architecture.
  - Sub-N (domain experts) brainstorm should come soon — F depends on ExpertActor shipping alongside PlanActor, and v1 needs a default expert set.
  - Sub-D's much-reduced brainstorm ("daemon singleton + external-tool coordination") is a quick win that can happen in parallel with any other work.

- **Key learnings / discoveries:**
  - **Fresh context is first-class** crystallized into a concrete architectural principle during F's brainstorm: context depth scales with decision specificity. Level 1 reasons broadly about its own plan, Level 2 reasons narrowly about one target project's code, Level 3 (if needed) reasons about one specific plan. Each level refocuses context rather than accumulating it. This is the architectural endpoint of fresh-context discipline.
  - **Filesystem-as-invariant beats metadata-as-invariant.** Three times during the F brainstorm we caught ourselves about to cache filesystem-derivable data in frontmatter (qualified ID, host project, dev root) and each time removing the cache simplified the design. The filesystem is authoritative; metadata projections can drift; the discipline is "if it can be computed, don't store it."
  - **Knowledge as consultative actors** rather than data stores is the key move that makes fresh-context discipline work for knowledge access. A data store serves reads; an actor serves questions. When you ask a question, the expert retrieves and synthesizes in its own fresh context, returning prose to your session. Your session never loads the expert's knowledge. This is the logical endpoint of "don't load what you can ask for."
  - **When a design process independently arrives at an existing framework's decisions, that's the universe telling you to use that framework.** We spent significant design energy specifying things OTP gives for free: supervision, mailboxes, let-it-crash, message passing, distribution transparency. Committing to Elixir reframes all of that work as "use the primitives that already exist" rather than "build our own versions."
  - **Mixture of experts + mixture of models falls out as an economic consequence** of the actor model, not as a feature bolted on. Expert actors are narrow-scoped consultation tasks that often don't need Opus-class models. Per-actor model selection turns this into a first-class capability. Team mode similarly falls out as a transport change, not an architectural change — actors are already message-passing entities; making them cross-machine is one more implementation of the same primitive.

### Session 10 (2026-04-14T23:03:06Z) — BEAM PTY spike PASS (with premise inversion)

- **Attempted**: validate that `erlexec` can drive the real `claude` CLI over
  a PTY with bidirectional stream-json, sentinel detection, process-group
  termination, configurable tool profiles, and backpressure-friendly output.
  Installed Erlang/OTP 28 + Elixir 1.19.5 via Homebrew. Scaffolded an
  Elixir mix project at `spikes/beam_pty/` with erlexec 2.2.3 and jason.
  Wrote a `BeamPty.Sentinel` sliding-buffer matcher (6 unit tests) and a
  `BeamPty.ClaudeSession` thin wrapper over `:exec.run/2` (2 live probes
  tagged `:live`).

- **Worked**: 8/8 tests pass. Pipes-only erlexec configuration
  (`[:monitor, :stdin, {:stdout, self()}, {:stderr, self()}, :kill_group,
  {:kill_timeout, 1}]`) cleanly drives claude end-to-end: `:exec.send/2`
  delivers NDJSON to stdin, `{:stdout, ospid, binary}` messages carry each
  stream-json event (init / rate_limit / assistant thinking / assistant
  text / result), `{:DOWN, ospid, :process, pid, reason}` fires on exit.
  Sentinel detector correctly finds `READY FOR THE NEXT PHASE` in claude's
  assistant text. Process-group termination (`:kill_group` + SIGTERM +
  500ms grace + SIGKILL) kills the grandchild of a `/bin/sh -c "sleep 60 &
  wait"` spawn. `--disallowed-tools` passes through at the CLI flag level
  and is visible in claude's `system/init` event.

- **Didn't work initially**: (1) `claude` on PATH resolves to a cmux
  wrapper script that injects `--session-id` and `--settings` flags
  incompatible with the probe — fixed by pointing at the real binary at
  `/Users/antony/.local/bin/claude`. (2) erlexec's `:pty + :stdin`
  combination: stdin is NOT wired to the child's real fd 0, so claude
  reads nothing and errors with "Input must be provided either through
  stdin or as a prompt argument when using --print". (3) DOWN message
  format from erlexec is custom: `{:DOWN, OsPid, :process, Pid, Reason}`
  — first element is the integer OS pid, not a monitor ref. (4) With
  PTY, all child output arrives tagged `:stderr` rather than `:stdout`
  because PTY slave merges both. (5) Sending `:eof` immediately after
  the user message closed stdin before claude could read it.

- **Suggests trying next**: absorb sub-C's amendment task (P1.3)
  immediately now that the approach is validated. The amendment should:
  (a) drop "PTY" from the stream-json path entirely; (b) specify
  pipes-only `erlexec` opts; (c) wrap the session in a `GenServer` with
  the NDJSON line parser and the sub-M telemetry boundary; (d) detect
  `{"type":"result"}` as the protocol-level "turn over" signal,
  orthogonal to the phase-prompt sentinel (task-level "done with the
  work"); (e) describe cmux-hook noise mitigation via
  `--setting-sources project,local --no-session-persistence`. Once
  amended, P3.1 (sub-F sibling plan scaffolding) is also unblocked.

- **Key learnings / discoveries**:
  - **The PTY premise was wrong.** Sub-C's stream-json path is stdio
    NDJSON, not a terminal. A PTY is only needed if sub-C ever wants to
    drive claude's interactive TUI (slash commands, arrow keys, ANSI
    redraws), which the memory invariant "no slash commands in the
    harness — control forbidden, observation required" explicitly rules
    out. This is a meaningful simplification for sub-C.
  - **erlexec uses a C++ port program**, not a NIF. `exec-port` is
    spawned as a separate OS process that handles PTY allocation,
    signals, and process groups without blocking BEAM schedulers. This
    is why erlexec can safely use `ptrace`, `setreuid`, and process
    groups.
  - **`:stdin` bare atom is required** if you want `:exec.send/2` to
    work. erlexec defaults stdin to `:null` (cat echo test exited with
    status 0 immediately otherwise).
  - **cmux SessionStart hooks pollute stream-json output**. Any claude
    invocation triggers ~10KB of hook JSON from user-global settings.
    `--setting-sources project,local` silences them cleanly.
  - **Sentinel sliding-buffer invariant**: window retained between
    feeds is exactly `sentinel_size - 1` bytes. Verified empirically
    after feeding 10KB of non-matching data (buffer stays at 23 bytes
    for the 24-byte sentinel). This is load-bearing for long-running
    phase sessions that may emit MB of assistant text.
  - **`{"type":"result"}`** is the protocol-level turn-over marker —
    complementary to the phase-prompt sentinel for task-level done.
    Sub-C should detect both.

### Session 11 (2026-04-14T23:23:09Z) — sub-C design doc rewritten inline for BEAM/Elixir pivot

- **Task chosen**: Priority 1.3 "Sub-C amendment — Elixir implementation and multi-adapter reservation" from `mnemosyne-orchestrator/backlog.md`. Named in `memory.md` as the critical-path unblocker for sub-F sibling plan scaffolding and as the task with freshest spike evidence (Session 10, `spikes/beam_pty/`).
- **What was attempted (first pass)**: produced a self-contained §12 "Amendment — BEAM/Elixir Pivot (2026-04-15)" block appended to the existing Rust-framed spec, with an AMENDMENT NOTICE banner and a "§12 is authoritative where earlier sections conflict" declaration. The supersede-layer approach landed as commit `7cb0919`.
- **What was attempted (second pass)**: the user rejected the supersede-layer pattern — "because we have made a significant pivot, I don't want to make things as superseded - I want fresh content to match the new approach." The spec was **rewritten inline across §1-§11** with fresh Elixir/OTP/erlexec content, and the §12 block was removed entirely. Feedback saved as durable memory at `~/.claude/projects/-Users-antony-Development-Mnemosyne/memory/feedback_pivot_rewrite.md` so the same pattern lands on every remaining amendment task.
- **What worked**:
  - The rewritten spec is now 1186 lines (down from 1511 with §12 appended) and reads as if the BEAM pivot was the original design. Every Rust idiom — `src/harness/`, `crossbeam-channel`, `nix`/`killpg`, `std::process::Command`, `Arc<dyn HarnessSession>`, three-threads-per-session — is gone and replaced with its BEAM equivalent: `lib/mnemosyne/harness_adapter/`, `erlexec` + `Process.send_after/3`, `:exec.kill/2`, `:exec.run/2`, session GenServer under `DynamicSupervisor`.
  - **New substantive content** that the supersede-layer had punted on: a full §4.5 "Tool-call boundary for in-session Queries" — the F-introduced contract that lets a live plan-cycle session use injected Mnemosyne tools (`ask_expert`, `dispatch_to_plan`, `read_vault_catalog`) as the routing-layer primitives for Queries and Dispatches. Covers injection-mechanism candidates (MCP over Unix socket preferred, stdin preamble fallback, plugin shim fallback), intercept flow via `Mnemosyne.Router.handle_tool_call/4` and `{:router_reply, _, _}` info messages, and a why-not-control-channel argument.
  - **Substantive §3 rewrite** — the `@behaviour Mnemosyne.HarnessAdapter` surface defines a single `spawn/1` callback plus `kind/0`, and the session GenServer contract is expressed as a documented message set (`send_user_message`, `attach_consumer`, `await_exit`, …) rather than additional behaviour callbacks. The `Mnemosyne.Event.*` sealed typed-struct set (`HarnessOutput`, `SessionLifecycle`, `SpawnLatencyReport`, `SessionExitStatus`, `HarnessError`) is the boundary to consumers.
  - **§4.1.2 spawn options table** — every erlexec opt is documented with its purpose, including the mandatory `:stdin` atom and the `{:cd, cwd}` working-directory wiring. The cmux mitigation flags (`--setting-sources project,local --no-session-persistence`) are declared mandatory on every daemon-spawned session, not optional.
  - **§4.3 stream-json parser** is locked against the spike's canonical event set: `system/init`, `rate_limit_event`, `assistant/thinking`, `assistant/text`, `assistant/tool_use`, `assistant/tool_result`, `user` echo, `result`. Sources at `spikes/beam_pty/results/full-run.log`; fixtures will be copied into `test/fixtures/harness_adapter/captured_stream_json/` on day 1.
  - **§10 open questions** — Q3 (prompt-as-arg vs stdin envelope) and Q4 (stream-json field names) are **resolved by the spike**; Q1/Q2/Q5 carry forward as day-1 tasks; Q6 (tool-call boundary injection mechanism) is newly open with a day-1 spike; Q7 (exec-port supervision) is resolved as a design decision (sessions translate loss to `{:exec_port_lost, _}`, PlanActor owns re-spawn).
  - **§11 cross-sub-project requirements** — Rust-specific amendments 1-3 back to B are dropped (no BEAM analogue exists for receiver typing, `Arc<dyn>`, `Send + Sync`). The two substantive requirements on B (consume `%SessionLifecycle{}` events; run a sliding-buffer sentinel matcher over `%HarnessOutput{kind: :stdout}`) survive and are stated freshly.
  - **Appendix A Decision Trail** preserves Q1-Q5 from the original Session 6 brainstorm as history, with spike corrections (e.g., Q1's "no PTY required" upgraded to "no PTY possible — `:pty + :stdin` actively breaks the input path") and adds Q6 (BEAM pivot), Q7 (BEAM PTY spike), Q8 (tool-call boundary) entries.
  - **Appendix B** — Cargo.toml diff replaced with an `mix.exs` deps projection showing one new Hex dep (`erlexec`), pre-existing `jason` + `:telemetry`, and `extra_applications: [:logger, :exec]`.
  - Sub-C sibling plan `backlog.md` top-notice rewritten; prior §12 breadcrumb removed. Sub-C sibling plan `memory.md` BEAM/Elixir pivot section updated to describe the inline rewrite and to record the pivot-rewrite pattern as user preference.
- **What didn't**:
  - The first-pass supersede-layer approach was committed and pushed (`7cb0919`) before the user corrected the approach. The second commit supersedes it by rewriting the content, so the history shows both the initial attempt and the corrected rewrite — users looking at git blame will see two closely-spaced commits against the same file.
  - Sub-C sibling plan's Rust-specific task list (Tasks 1–24) in `backlog.md` is still the pre-pivot text. Rewriting that task list to Elixir is a discrete sub-C work phase, not this orchestrator task. The top-of-file notice flags it.
- **What this suggests trying next**:
  1. **Sub-C sibling plan backlog rewrite** — discrete work inside the sub-C plan. Task sequence survives; every Rust idiom gets replaced. The rewritten spec is the reference.
  2. **P3.1 sub-F sibling plan scaffolding** — unblocked by the spike and by the rewritten spec (tool-call boundary contract is now concrete). F's design doc §11 lists the task set; scaffolding is mechanical.
  3. **Sub-B amendment (P1.2)** — same pivot-rewrite pattern applies. The F-committed `{{VAULT_CATALOG}}` rename, the `plan-state.md` schema pruning, and the PlanActor-hosted phase cycle all get rewritten inline in sub-B's design doc, not layered under a supersede notice.
  4. **All remaining amendment tasks (sub-A, sub-D, sub-E, sub-M, sub-G, sub-H, sub-I)** — follow the inline-rewrite pattern per the durable feedback memory.
- **Key learnings / discoveries**:
  - **Pivots demand inline rewrites, not supersede layers.** A spec where §2-§8 describes the old approach and §12 says "§12 wins" forces every future reader to context-switch between two incompatible framings and trust a disclaimer to resolve conflicts. The stale content drifts further from reality every time the new approach evolves. Fresh content avoids both failure modes and produces a doc that reads as if the pivot was the original design. This is now durable feedback memory at `feedback_pivot_rewrite.md`.
  - **The Decision Trail appendix is where history belongs** — brainstorm decisions, alternatives considered, and post-write clarifications are load-bearing context for future maintainers. Rewriting §1-§11 and leaving Appendix A's Q1-Q5 intact preserves the reasoning without polluting the current-truth spec.
  - **Corrections in the Decision Trail are more useful than a separate "supersede" layer.** When the BEAM PTY spike corrected Q1's "no PTY required" to "no PTY possible — `:pty + :stdin` actively breaks the input path", the correction lives inside Q1's entry as a dated paragraph, not as a new entry at the bottom of the doc. Readers tracing the reasoning see the correction where it belongs.
  - **Dependency manifests replace wholesale.** Appendix B went from Cargo.toml diff to `mix.exs` projection with no attempt to map deps one-to-one. That is the right level of granularity — the file is a projection of the new reality, not a migration guide.
  - **Cross-sub-project requirements are re-evaluated, not translated.** B's amendments 1-3 from the Session 6 brainstorm (receiver typing, `Arc<dyn>`, `Send + Sync`) don't translate; they drop. The two substantive requirements survive (consume `%SessionLifecycle{}`, run sentinel matcher) but are restated in fresh BEAM terms. A mechanical translation would have preserved dead requirements.
  - **First-commit undo pattern.** When the first pass lands in git and the user corrects the approach, the second commit is a fresh rewrite, not a git revert. History preserves both; readers can see the correction as a learning signal. Corresponds to the memory rule "never amend published commits" — we added a new commit instead of rewriting `7cb0919`.

### Session 12 (2026-04-15T00:39:09Z) — sub-B amendment: inline rewrite for Elixir/BEAM

- **Attempted:** P1.2 — rewrite sub-B's design doc inline to absorb the three pending amendments (sub-C consumption contracts, LLM_CONTEXT 2026-04 four-phase overhaul, sub-F BEAM daemon + actor pivot). Follow the sub-C precedent from Session 11: no supersede layer, rewrite §1–§6 wholesale, preserve history in Appendix A's Decision Trail.

- **What worked:**
  - Fresh rewrite of `docs/superpowers/specs/2026-04-12-sub-B-phase-cycle-design.md` (2296 lines). Every Rust idiom from the 2026-04-12 brainstorm is replaced with its Elixir/OTP equivalent: `Box<dyn PhaseExecutor>` → `@behaviour`, `include_str!` → `@external_resource` + `File.read!/1`, `serde_yaml` + `gray_matter` → `yaml_elixir`, `tokio::mpsc` → PlanActor mailbox + `emit_fn` closure, `ratatui` process main loop → separate Rust TUI client binary (sub-F), `fs2::FileExt::lock_exclusive` → sub-D daemon singleton flock, `nix::killpg` → sub-C's `:kill_group` via erlexec.
  - The four-phase cycle (`work → reflect → compact → triage`) is now first-class in `PlanState`, `PhaseRunner.run_phase/4`, and the phase enum. Conditional compact with `wc -w memory.md > compact-baseline + HEADROOM` (HEADROOM = 1500 words) is implemented at compact-entry, not inside the phase executor — the LLM never decides.
  - Schema pruning per sub-F landed: `plan-id` / `host-project` / `dev-root` removed; `description:` (≤120 chars, hard-capped, non-placeholder) added with six load-time invariants; `mnemosyne-pid` → `daemon-pid`; sticky `compact-baseline` added.
  - `{{VAULT_CATALOG}}` replaces `{{RELATED_PLANS}}`; the placeholder set is now five-way (`{{DEV_ROOT}}`, `{{PROJECT}}`, `{{PLAN}}`, `{{PROMPTS}}`, `{{VAULT_CATALOG}}`). `related-plans.md` is deleted outright (G's migration scope).
  - F's `DispatchProcessor` / `QueryProcessor` added as phase-exit hooks on work/reflect/triage (not compact — compact is strictly lossless memory rewriting). Sub-E's `ReflectExitHook` behaviour preserved but invoked non-blockingly via `Task.Supervisor` inside the PlanActor.
  - Sub-C consumption contract fully woven into §4.1: the `%Mnemosyne.Event.SessionLifecycle{}` pattern-match surface (`:ready`, `{:turn_complete, _}`, `{:exited, _}`), the sliding-buffer sentinel matcher on `%HarnessOutput{kind: :stdout}` (validated by the BEAM PTY spike), and the control-vs-observation clarification. Rust Amendments 1–3 explicitly retired.
  - `pre-work.sh` hook absorbed into §2.3.3 step 2 as a new `Mnemosyne.PreWorkHook` module; runs only before work, after the defensive `latest-session.md` cleanup, from the project root, non-zero exit aborts the cycle.
  - Appendix A structure mirrors sub-C's: Q1–Q17 original decisions with Session-12 correction notes inline, Q18 (LLM_CONTEXT overhaul — seven items), Q19 (sub-F actor commitment — four items), Q20 (BEAM pivot + PTY spike — Rust→BEAM translation table + three spike findings). Post-write clarifications block preserves control-vs-observation and task-vs-protocol-level completion distinctions.
  - Sibling plan updated: sub-B's `backlog.md` top-notice rewritten, three amendment tasks marked done with detailed Results pointing at the rewritten sections, and a new gate task ("Rewrite downstream implementation tasks against Session-12 design doc") added with a twelve-item checklist for the next amendment work phase. Sub-B's `memory.md` BEAM-pivot + Session-11 sections consolidated into a single Session-12 anchor block with direct design-doc section pointers.

- **What didn't work / what I deliberately deferred:**
  - The downstream implementation task list in `sub-B-phase-cycle/backlog.md` (everything from "Define core abstractions and types" onwards) still carries pre-pivot Rust framing. Rewriting all twenty-plus task descriptions in this work phase would have been scope creep; instead the new gate task captures the exact twelve changes needed so the next work phase can tackle it. This mirrors how sub-C left its own sibling-plan backlog: spec rewritten, task-list rewrite follow-up deferred.
  - I did not touch sub-B's `session-log.md` — the runner handles that append post-hoc.

- **What this suggests trying next:**
  - **Rewrite sub-B's downstream implementation task list** against the new design doc. This is the gate the new follow-up task captures. Should be a single work phase; mostly mechanical translation with some task deletions (InteractionDriver, IpcDriver, HeadlessDriver, RatatuiDriver, full-startup-sequence, per-plan lock stub).
  - **P1.5 (sub-E amendment)** and **P1.6 (sub-M amendment)** are good next candidates — both follow the same inline-rewrite pattern now validated by sub-C (Session 11) and sub-B (Session 12). Sub-M's amendment is moderate (re-cast `tracing` + Rust enum to `:telemetry` + typed Elixir structs). Sub-E's amendment is moderate (Stage 5 becomes dispatch-to-experts; pipeline stages 1–4 unchanged; implementation re-cast to `GenStage` or `Broadway`).
  - **P3.1 (F sibling plan scaffolding)** is still unblocked. Worth considering as a higher-leverage task before more amendments because it unblocks sub-N and the whole F implementation runway. Momentum argument: knock out one more amendment first (E or M, whichever is ready), then pivot to F scaffolding.

- **Key learnings / discoveries:**
  - **The inline-rewrite pattern scales.** Sub-C proved it at 1186 lines; sub-B extends it to 2296 lines with three simultaneous upstream shifts to absorb. The discipline: preserve Appendix A Q1–Qn, add Qn+1…Qm for each new shift, re-cast §1–§main inline with zero stale references to the replaced framing, and verify with a grep pass across the anti-patterns (`include_str`, `Box<dyn`, `serde_yaml`, `tokio`, `ratatui` outside Appendix A, etc.).
  - **Three amendments can stack.** The original plan had P1.2 as a single "Sub-B amendment" ticket, but the amendment actually absorbed three distinct upstream shifts: LLM_CONTEXT 2026-04 overhaul (seven items), sub-F actor commitment (four items), BEAM pivot (whole runtime substrate). Each got its own Appendix A entry (Q18, Q19, Q20) rather than being folded together — this keeps the decision trail auditable and matches how sub-C handled its own multi-shift amendment (Q6-Q8 there).
  - **Four-phase cycle as state-machine extension is small.** The original three-phase enum grows by one variant; the runner's compact-trigger check sits at step 1 as an exception; `compact-baseline` is a sticky integer field. No other parts of the design had to move. This validates LLM_CONTEXT's upstream choice to make compact a first-class phase rather than a mid-reflect detour.
  - **Description discipline is load-bearing.** The ≤120-char hard cap on `description:` is a load-time invariant in `PlanState.load/1` (`{:error, :description_too_long, seen, cap}`). Without the cap, the vault catalog's rhythm falls apart — 20 plans × 120 chars ≈ 2.5KB is negligible for phase-prompt context budget; 20 plans × 400 chars starts to bite. The cap is not decorative.
  - **B owns the sentinel matcher; C owns the event transport.** This split emerged cleanly in sub-C §11.1 and carries through B §4.1 Requirement 2 + the new `Mnemosyne.Sentinel.SlidingBuffer` module. Sentinels are coupled to phase prompts (which B owns), the mechanism is harness-agnostic, and the sliding-buffer algorithm is validated by the BEAM PTY spike across five edge cases. Keeps the B/C interface narrow.
  - **No B-owned TUI is a simplification, not a loss.** The original sub-B design had `ratatui` / `HeadlessDriver` / `IpcDriver` as B deliverables; the BEAM pivot moves the TUI to sub-F as a separate Rust client binary connecting over the NDJSON socket. B's responsibility ends at the PlanActor message contract (five messages). Everything downstream of that is sub-F's scope. The `InteractionDriver` trait abstraction is deleted entirely — the IPC boundary it was trying to harden is now the socket protocol, which is sub-F's territory.

### Session 13 (2026-04-15T00:57:02Z) — Sub-F sibling plan scaffolded (P3.1)

- **What was attempted**: Execute P3.1 ("Scaffold sub-F sibling plan post BEAM spike"). Memory.md flagged this as the critical next step since Session 10, unblocked by the spike passing and by the sub-B (Session 12) and sub-C (Session 11) inline amendment rewrites. Deliverable: `LLM_STATE/sub-F-hierarchy/` with the standard plan-file set populated from F's design-doc §11.
- **What worked**:
  - Five files written under the new plan directory: `backlog.md` (439 lines, 28 implementation tasks + Task 0 readiness gate), `memory.md` (214 lines, primary-reference + F-1..F-12 architectural anchors + sibling-contract block), `prompt-work.md` (96 lines, follows sub-B's pattern), `phase.md` (`work`), `session-log.md` (`# Session Log` header only).
  - Backlog organized by F §11's section structure: Elixir scaffolding (Tasks 1–9 including VaultDirectory, Actor behaviour, PlanActor wrapping sub-B's PhaseRunner, ExpertActor stub, ActorSupervisor, Router.Server, Dispatch/QueryProcessor, CatalogRenderer), Daemon binary (Tasks 10–13), Declarative routing (Tasks 14–17), Level 2 routing agent (Tasks 18–21), Integration (Tasks 22–24 — the cross-sibling hookups), Tests (Tasks 25–28 per §10).
  - Task 0 readiness gate mirrors sub-B's pattern: blocks Task 1+ on four conditions — sub-B's downstream task-list rewrite against its Session-12 design doc, sub-C's downstream task-list rewrite against its Session-11 design doc, sub-A amendment absorbed, sub-M amendment absorbed. Also mandates re-reading F's §11 against the latest versions of those three design docs when the gate fires, with inline-rewrite of F's design doc if any consumed interface has drifted.
  - Out-of-scope carry-forward notes are explicit: the Rust `mnemosyne-tui` binary belongs to a separate future plan (F-11 ships only the daemon-side client-listener in Task 11); F's §11.8 cross-plan landings are already done by F's Session-9 triage; sub-N ExpertActor internals are type-holed in Task 3 only; sub-O and sub-P are schema-hook-reserved in Task 13 only.
  - `latest-session.md` and `related-plans.md` intentionally omitted per project convention (latest-session is written per cycle and deleted by run-plan.sh; same-project siblings auto-discovered).
  - Orchestrator backlog updated: P3.1 marked done with detailed Results; Recent history block compacted (Sessions 10–12 collapsed into one entry, Session 13 added at top). Orchestrator memory.md updated: sub-project table F row now reflects "sibling plan scaffolded Session 13"; "Recommended sub-project ordering" now points at sub-F scaffolded as a done step; "Critical next steps" list re-prioritized — (1) Sub-N brainstorm, (2) A/E/M amendments, (3) sub-B/sub-C task-list rewrites, (4) sub-F Task 0 readiness check once the preceding land.
- **What didn't**:
  - Minor edit mishap: first edit to the Recent-history block accidentally relabeled the existing Session 9 header to Session 12. Caught on re-read and corrected — Session 9 framing restored and Sessions 10–12 added as a collapsed entry above it.
  - F's §11.6 heading literally reads "Rust TUI client (a separate implementation plan, sub-B's scope)" which is a doc typo — "sub-B's scope" contradicts sub-B's scope (phase cycle) and F-11's own statement that the TUI is F's commitment. Resolved ambiguity by excluding TUI tasks from sub-F's backlog entirely and flagging them under out-of-scope carry-forward notes as a "separate implementation plan to be scaffolded later." Should be recorded as a doc-level correction to F's spec in a future work phase; not done this session to stay scoped.
- **What this suggests trying next**:
  - Next critical-path work is **Sub-N brainstorm** (v1-critical ExpertActor internals — sub-F's Task 3 stub is now in place waiting for it). Alternatively, the four A/E/M amendment tasks whose inputs are now concrete from sub-B's §4 and sub-C's §11 rewrites.
  - A low-effort follow-up: fix F's §11.6 heading typo ("sub-B's scope" → "separate implementation plan") so future work-phase readers aren't confused. Could fold into the next work phase that touches F's design doc.
  - Sub-B's downstream task-list rewrite (gate task in sub-B's backlog) is the closest gate-condition to unlocking sub-F Task 1. Worth prioritizing when choosing between A/E/M amendments.
- **Key learnings or discoveries**:
  - The "amendment tasks rewrite specs inline, not as supersede layers" discipline (now a stable architectural principle in memory.md since Session 12) applies recursively: sub-F's Task 0 gate explicitly mandates inline rewrites of F's own design doc if any consumed interface has drifted at implementation time. The discipline composes.
  - Scaffolding a sibling plan from a design doc is a mechanical translation — the 28-task list falls out of §11 almost verbatim. The real work is the dependency ordering and the out-of-scope carry-forward notes, both of which require judgment about what survives and what is deferred.
  - Section §11.6 of F's doc surfaces a naming/scoping ambiguity that was not caught in F's brainstorm. This kind of drift between brainstorm framing and implementation-phase reading is exactly what the "memory.md and backlog.md drift is load-bearing" principle flags — spec drift matters the moment a fresh LLM phase tries to act on it.

### Session 14 (2026-04-15T01:51:21Z) — Sub-A amendment absorbed via inline rewrite

- **Attempted**: P1 task "Sub-A amendment — daemon caller integration." Goal was to absorb the BEAM daemon pivot (Session 9) and sub-F's architectural commitments (Session 9) into sub-A's design doc, following the inline-rewrite discipline established by sub-C (Session 11) and sub-B (Session 12). One of the four conditions on sub-F's Task 0 readiness gate.
- **Worked**: Rewrote `docs/superpowers/specs/2026-04-13-sub-A-global-store-design.md` from 635 → 1242 lines. §A1–§A10 re-cast in place for the Elixir daemon; reference algorithm rewritten in Elixir; cross-sub-project contracts table updated; §A4 vault layout absorbs every sub-F surface (`daemon.toml`, `routing.ex`, `plan-catalog.md`, `experts/`, `runtime/{daemon.sock,daemon.lock,daemon.pid,mailboxes/}`); §A10 walk-up now searches for `mnemosyne/project-root/` ancestor instead of `mnemosyne/plans/`; daemon singleton lock replaces sub-D's per-plan locks. Q1–Q5 preserved verbatim with Session-14 correction notes; new Q6 (BEAM pivot) and Q7 (sub-F commitments) record the amendment substance; Appendix B added for `mix.exs` deps projection (only one new hex dep: `{:toml, "~> 0.7"}`); Appendix C added for the post-amendment glossary. Sibling plan memory and backlog top-notices updated; orchestrator memory table-row flipped to "F amendment absorbed"; orchestrator backlog task moved to done with detailed Results block; "Critical next steps" updated to remove sub-A and add sub-M as the lone remaining orchestrator-level F Task 0 gate.
- **Didn't work / surprises**: Originally scoped as "small amendment — mostly re-framing, not redesigning." It expanded to ~600 net new lines because sub-F's Session-9 commitments and sub-C/sub-B's Session-11/12 rewrites land **concrete** new vault-layer surfaces (four new tracked files at vault root, four new gitignored runtime files, the `plans/` → `project-root/` rename) that all had to be absorbed in §A4 + §A5 + §A6 + §A10 in one pass. The "small re-framing" estimate didn't account for the cumulative downstream reach. Validates the inline-rewrite discipline at smaller scale than sub-B (2296 lines, three simultaneous shifts) but bigger than expected.
- **What this suggests trying next**: Sub-M amendment is now the **last remaining orchestrator-level sub-F Task 0 gate condition** (the other two gates — sub-B and sub-C downstream task-list rewrites — live in sibling plans, not the orchestrator backlog). Sub-M re-casts hybrid `tracing` + `MnemosyneEvent` enum to `:telemetry` + typed `Mnemosyne.Event.*` structs, and concrete inputs are now landed: B's §4.4 spelled out seven `%PhaseLifecycle{}` event variants, C's §11.4 spelled out the telemetry boundary contract, and A's amendment now adds four candidate event types (`VaultResolved`, `VaultVerificationFailed`, `VaultInitialised`, `ProjectAdopted`) to the catalog. Picking sub-M next would close the orchestrator's contribution to the sub-F Task 0 gate. Sub-N brainstorm remains the v1-critical alternative (independently executable, ExpertActor stub waiting at sub-F's Task 3).
- **Key learnings / discoveries**:
  - **Every Session-7 forking decision survived the language swap unchanged in spirit**. Q1 (env-and-config discovery), Q2 (`init` with `--from` flag), Q3 (gitignore-by-policy), Q4 (delete v0.1.0 outright), Q5 (TOML marker) all stand. The runtime swap only changed *how* (Elixir stdlib calls vs Rust crates) and *how often* (once per daemon boot vs per CLI invocation). **Validates** the "design language ≠ runtime language" framing — A's discovery chain is a pure function of inputs, language-neutral.
  - **The two-file split (`mnemosyne.toml` for vault identity vs `daemon.toml` for runtime knobs) is load-bearing**: swapping daemon versions or reconfiguring sub-O harnesses must not touch the vault identity marker. This emerged while writing §A3 — the brainstorm hadn't fully separated the two concerns. Worth flagging in the memory if it recurs in sub-F implementation.
  - **A's vault root must be a real directory, not a symlink** because `<vault>/projects/<name>/` symlinks are created relative to it. The reference algorithm now adds an explicit `File.lstat!/1` rejection check at boot. This is a new invariant the brainstorm didn't surface (it only thought about target symlinks, not the root itself).
  - **A's `adopt-project` now has a live-daemon notification path** via `runtime/daemon.pid` liveness check + `:rescan` NDJSON message over `runtime/daemon.sock`. This bridges the bootstrap-subcommand vs running-daemon split cleanly: the bootstrap subcommand mutates the vault, then notifies the live daemon if one exists. New surface for sub-F to consume.
  - **Inline rewrite discipline composes recursively**: sub-A's amendment cites sub-C's Session-11 contract (§4.5 tool-call boundary) and sub-B's Session-12 contracts (§4.2 ReflectExitHook, §4.4 typed events, §A6 embedded prompts via `@external_resource`) by section number. Future amendments will keep accumulating these cross-references. Worth being deliberate about section-number stability across rewrites.
  - **Three of four sub-F Task 0 gate conditions remain after this work**: sub-B downstream task-list rewrite (sibling), sub-C downstream task-list rewrite (sibling), sub-M amendment (orchestrator). The fourth (sub-A amendment) is now done.

### Session 15 (2026-04-15T04:24:20Z) — Sub-M amendment absorbed via inline rewrite

- **What was attempted.** Pick and execute the sub-M amendment — the last remaining orchestrator-level amendment task, and (at session start) the last orchestrator-level gate condition blocking sub-F's Task 0. Rewrite `docs/superpowers/specs/2026-04-13-sub-M-observability-design.md` inline across §1–§20 to target Elixir/OTP idioms, following the sub-C/sub-B/sub-A precedent. Update sub-M sibling plan and the orchestrator backlog + memory.

- **What worked.**
  - The design doc was rewritten inline from 626 → 870 lines. Every Rust-specific surface was replaced rather than layered with a supersede block. §1–§20 now reads as a native BEAM design.
  - The sealed `Mnemosyne.Event.*` struct set grew from 7 Rust variants to 20+ Elixir structs grouped by producer: sub-B (one struct with seven `kind` variants), sub-C (five structs), sub-F (nine), sub-E (six `Ingestion.*` variants), sub-A (two `Vault` variants), M-owned escape hatches (`Metric`, `Diagnostic`, `Error`). All producer-side contracts came from already-landed sibling specs (B §4.4, C §7.2 + §11.4, F Task 24, A §A.Reference algorithm step 11).
  - §5.2 correctly mandates a `try/rescue` wrapper on `Handler.handle_event/4` because `:telemetry` detaches raising handlers — this is a new failure mode not present in the Rust design, captured as Risk 6 and the rewrite's most load-bearing BEAM-specific finding.
  - §6 metric catalogue grew from 20 to 23 `Telemetry.Metrics.*` definitions; the extra eight came from sub-F's routing surface (`message_routed`, `rule_fired`, `level2_invoked`, `level2_rejected`, `dispatch_processed`, `query_answered`, `rule_compile_error`, `rule_suggestion`) plus three `actor_state_change` counters. The catalogue-integrity test (parse §6, assert correspondence with `Metrics.metrics/0`) replaces the Rust `const &'static str` compile-time typo protection.
  - Session-7 Q1–Q5 decisions all survived the runtime swap unchanged because none were language-specific. Preserved verbatim in the Decision Trail with per-decision Session-15 correction notes. Q6 (BEAM pivot) added with a 19-row Rust→BEAM translation table; Q7 (reporter selection) added as a new decision entry (`ConsoleReporter` + `SnapshotReporter` for v1, `:telemetry_metrics_prometheus` additive in v1.5, OpenTelemetry reserved for v2).
  - Sub-M sibling plan memory.md: the "BEAM pivot — pending amendment" section was replaced by an "absorbed via inline rewrite" block with a skimmable design-doc anchors index (§3–§12, Q6). Sibling plan backlog.md: Task 0 "BEAM amendment" block replaced with a top-notice + a new Task 0 gate mirroring sub-B/sub-C discipline — an 18-item checklist covering every stale Rust-flavored task below.
  - Orchestrator backlog: sub-M amendment marked done with a full Results block capturing §-by-§ changes; Session 15 recent-history entry added; execution note rewritten to reflect that all four orchestrator-level amendment tasks (A, B, C, M) are now done and the remaining F Task 0 blockers are both sibling-level.
  - Orchestrator memory: sub-M row in the sub-projects table flipped to "done (F amendment absorbed)" with a detailed note; recommended-ordering line updated; critical-next-steps renumbered (sub-N brainstorm is now (1), sub-B/sub-C downstream rewrites (2), sub-E amendment (3)).

- **What didn't work.** Nothing substantive failed. The Write tool requires a native Read before overwriting even when ctx_read has already cached the file; one round-trip retry. Minor scope drift: I initially planned to preserve the original doc's section numbering exactly, but §4.1 naturally needed an enumerated list of 20+ struct producers — the section retains its heading but expanded substantially. This matches the sub-A/sub-B precedent where amendment doc growth concentrates in producer-contract sections.

- **What this suggests trying next.**
  - **Sub-N brainstorm** is now the critical track. v1-critical (ExpertActor must ship alongside PlanActor); independently executable; sub-F Task 3 stub is in place waiting for it. No upstream amendments pending. Follow the `superpowers:brainstorming` skill; design doc at `docs/superpowers/specs/YYYY-MM-DD-sub-N-domain-experts-design.md`; sibling plan at `LLM_STATE/sub-N-domain-experts/`.
  - **Sub-E amendment** can run in parallel with sub-N if sub-N's interface contract lands first. M's §4.1 sealed set now has the six `Ingestion.*` variants ready for E to consume. B's §4.2 `ReflectExitHook` behaviour feeds it from the other side.
  - **Sub-B and sub-C downstream task-list rewrites** are the two remaining sub-F Task 0 blockers but live in those sibling plans, not this orchestrator. They are the critical track for F's implementation runway.
  - **Sub-M's own downstream task-list rewrite** (new Task 0 gate added this session) is internal to sub-M and not on F's critical path; can be scheduled alongside sub-M's v1 implementation when the time comes.

- **Key learnings or discoveries.**
  - **`:telemetry` handler re-entrancy is a different failure mode than Rust Layer re-entrancy.** `:telemetry` *detaches* handlers that raise, silently blinding the daemon. Any BEAM observability design must mandate a `try/rescue` wrapper on the handler that logs via `Logger` directly (not through M) — otherwise a bug in any subscriber permanently loses observability. The Rust draft did not face this because `tracing-subscriber::Layer` failures log but do not detach.
  - **`:telemetry_metrics` is reporter-independent by design.** The same `counter(...)` / `distribution(...)` definitions feed any reporter attached to the supervisor tree. This was the load-bearing reason for choosing `:telemetry_metrics` over `prom_ex`: switching or adding reporters is purely additive. `prom_ex` bundles Phoenix/Ecto/Broadway plugins Mnemosyne does not consume and ships its own Plug endpoint, dashboards, and Grafana integration — dead weight.
  - **Multi-party producer contracts make amendments easier, not harder.** This amendment had concrete producer-side contracts from five upstreams (A, B, C, E, F) that were all rewritten for BEAM before M's amendment. The sealed struct set essentially wrote itself from the producer contracts. The cross-plan adoption discipline (memory: "Cross-cutting brainstorms own adoption stubs") has a corollary: when multiple upstreams are rewritten before the cross-cutting amendment, the amendment becomes a collation task rather than a design task.
  - **`:pg` (stdlib since OTP 23) is the right primitive for TUI fan-out.** Rust's in-process `mpsc::Sender` has no single BEAM analogue for multi-client fan-out; `:pg` gives process-group membership without a `Phoenix.PubSub` dep, which would have been disproportionate for a single concern. Per-client backpressure is handled by the daemon's per-client session process wrapping `send/2` in a try/catch that increments a drop counter.
  - **The inline-rewrite discipline scales to five simultaneous upstreams.** Previous validations: sub-A at 1242 lines absorbing two shifts, sub-C at 1186 lines with one shift, sub-B at 2296 lines absorbing three simultaneous shifts, now sub-M at 870 lines absorbing five (A, B, C, E, F). Section-number stability across rewrites continues to hold: every downstream cite (`A cites B §4.4`, `M cites A §A.Reference algorithm step 11`, `M cites F Task 24`, etc.) still resolves post-rewrite.

### Session 16 (2026-04-15T05:50:05Z) — Sub-N brainstormed, sibling plan scaffolded, sub-Q and sub-R surfaced

- **Attempted**: Drive the v1-critical sub-N (domain experts) brainstorm end-to-end through the `superpowers:brainstorming` skill, producing the design doc, scaffolding the sibling plan, and capturing follow-on backlog work for the orchestrator.

- **What worked**:
  - Locked the design in seven approval-gated clarifying questions plus eight approval-gated design sections, all in one session. The one-question-at-a-time format kept each decision crisp without overwhelming the user.
  - User pushback during section 2 ("dialogue state on disk feels transient — store in memory") caught a real architectural mistake and led to a clean revision: `DialogueRegistry` as an ETS-backed singleton GenServer, sweeper-driven TTL, audit trail via sub-M's event log instead of parallel JSONL files. Better outcome than the original on-disk framing.
  - User pushback on timer framing ("5 seconds is probably not long enough; LLMs can spend a lot of time frobnosticating") surfaced a conflation — the 5 s was for the deterministic ripgrep pipeline, not for LLM reasoning. Separating the two (retrieval = 5 s structural guard; per-turn session = 5 min default reasoning budget; dialogue TTL = 30 min idle) led to a cleaner mental model that generalizes beyond sub-N.
  - The `expert_id` dogfooding swap (`elixir-expert` in, `distributed-systems-expert` out) emerged naturally from thinking about what would match every Mnemosyne session memo — a useful fork in what was otherwise a pro-forma "confirm the defaults" step.
  - Spec self-review caught one real inconsistency: the `dialogue.max_clarification_rounds` hard cap was stated as 7, but the arithmetic (1 initial question + N clarifications + N replies + 1 final answer) constrains it to 3 within the 8-turn dialogue limit. Fixed inline in spec §3.2, §5.7, and Appendix C before the user review gate.
  - The sibling plan's task structure split cleanly into phases that respect the real dependency DAG: Tasks 1–14 can start immediately (pure Elixir + fixtures + event structs + singleton GenServers), Task 15 is an early deliverable for sub-E's amendment, Task 16+ gates on sub-F delivering the actor behaviour + supervisor API. This means sub-N and sub-E amendment can parallelize with minimal coordination cost.

- **What didn't work / friction**:
  - The first cut of the dialogue-state design committed to JSONL files on disk because I was over-indexing on "filesystem as substrate." The user correctly distinguished "non-transient durable info belongs on disk" from "transient in-flight state can stay in memory," which is a real refinement to the overall project principle.
  - Almost set the retrieval-pipeline cap too tight because of the timer conflation. Pushback prevented a design mistake that would have landed in the spec.
  - The backlog-read path hit lean-ctx's "output too large" ceiling multiple times on the orchestrator's large backlog.md; had to fall back to Read with offsets. Not a brainstorm issue, just friction.

- **What this suggests trying next**:
  - **Sub-E amendment** is the obvious next work phase. Sub-N's design doc §6 already commits the Stage 5 contract; sub-N's Task 15 will deliver the real interface (scope matcher + message structs + verdict structs + event structs) as an early PR that sub-E can code against in parallel with sub-N's implementation. Sub-E amendment has been unblocked by this session and has no other dependencies.
  - The **two-timer pattern** (deterministic structural guard vs LLM reasoning budget vs idle session timer) generalizes. Similar triples probably exist in sub-E (Stage 2/3 LLM calls vs pipeline cap), sub-F (Level 2 routing agent vs routing-rule evaluation), and sub-C (cold-spawn latency vs session lifetime). Worth a memory entry and probably worth a future audit of existing specs to make sure nothing collapses these concepts.
  - **Sub-Q and sub-R** (vector-store infrastructure, knowledge ontology) are both cross-cutting research threads that surfaced naturally from doing sub-N properly. Adding them to the backlog rather than trying to solve them inline was the right call per the user's own "significant pivots get fresh rewrites" discipline. They stay v1.5+ and do not block sub-N v1.

- **Key learnings / discoveries**:
  - **"Fresh context is first-class" doesn't mean "everything on disk."** The dialogue-state revision is a clean instance of the distinction. Non-transient durable information belongs on disk because durability is load-bearing. Transient in-flight state can live in memory because restart-resilience is not load-bearing for short-lived interactions. Both honor fresh context; they differ on what substrate they use.
  - **Behaviour chokepoints are a feature, not just future-proofing.** The `Mnemosyne.ExpertRetrieval` behaviour lets sub-N ship v1 with one strategy AND lets sub-Q land later as a drop-in. This is the second time this pattern has emerged in Mnemosyne's design (sub-C's `HarnessAdapter` is the first). Worth noting: every future sub-project should ask "what's the chokepoint here?" before committing to a single implementation.
  - **Cross-cutting research threads that surface during one brainstorm deserve their own brainstorm slots, not inline absorption.** Sub-Q and sub-R both emerged from sub-N's clarifying questions. Neither fits cleanly inside sub-N's scope; both deserve fresh-context investigation. The user's "separate research task" framing was the right move — it preserves context for the later investigation and keeps sub-N's v1 scope honest.
  - **Dialogue caps have arithmetic consequences.** The `max_clarification_rounds` × 2 + 2 = `max_total_turns` relationship was not obvious until the self-review pass. Any future spec that caps a multi-party interaction should make the arithmetic explicit up front to avoid similar inconsistencies.
  - **The one-question-at-a-time format is a real discipline win**. Batching questions would have let me mis-commit on dialogue state because I wouldn't have paused for user input at the exact moment pushback mattered. The skill's format is load-bearing for quality, not just for user comfort.
