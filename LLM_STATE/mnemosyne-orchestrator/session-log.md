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
