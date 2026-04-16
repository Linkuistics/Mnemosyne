# Backlog — Sub-project N: Domain Experts (ExpertActor internals)

Implementation backlog for sub-project N. All tasks derive from §1–§11
of the design doc at
`{{PROJECT}}/docs/superpowers/specs/2026-04-15-sub-N-domain-experts-design.md`.
Consult the spec before starting any task.

**Sibling plan scaffolded** in Session 16 of the orchestrator plan
(2026-04-15) immediately after the sub-N brainstorm. The spec is
frozen; this plan executes against it.

**Implementation runway:** unlike sub-F which has a hard Task 0 gate
blocking all implementation, sub-N splits gracefully. **Tasks 1–15 can
start immediately** because they are pure Elixir modules, fixture
authoring, event-struct definitions, and early-deliverable message
shapes — none of which depend on F's supervisor integration. **Tasks
16+ gate on sub-F delivering the `Mnemosyne.Actor` behaviour +
`ActorSupervisor` child-spec API** (which does not require all of
sub-F's 28 tasks to be done — F can ship those early as its first PR).

## Task Backlog

### Task 0 — Actor-integration readiness gate `[gate]`
- **Status:** not_started
- **Dependencies:** sub-F delivers `Mnemosyne.Actor` behaviour,
  `ActorSupervisor` child-spec registration API, and `Router` accepting
  custom message structs in a merged PR (sub-F Tasks 1–3 per sub-F's
  §11.1 scaffolding tasks).
- **Description:** Meta-task guarding the start of actor-integration
  work. Before any Task 16+ runs, verify:
  1. `Mnemosyne.Actor` behaviour module exists in the codebase with
     the callbacks sub-N needs: `init/1`, `handle_actor_message/2`,
     `snapshot/1`.
  2. `Mnemosyne.ActorSupervisor` is up and registerable via the
     child-spec API F commits to in §4.5 of its design doc.
  3. `Mnemosyne.Router` accepts the three sub-N message structs
     (`ExpertQuery`, `ExpertDialogueReply`, `ExpertAbsorbCandidate`).
     This gate can be cleared by sub-N itself during Task 14 if
     F's router is already extensible by the time sub-N reaches
     Task 14.
  4. `<vault>/experts/` directory is created and walked by the time
     Task 21 (DeclarationLoader) runs.

  Tasks 1–15 proceed without this gate. The gate only blocks
  Tasks 16–28.
- **Results:** _pending_

---

## Phase 1 — Pure Elixir modules (no F dependency)

### Task 1 — `Mnemosyne.Expert.Declaration` parser and validator `[impl]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Implement `Mnemosyne.Expert.Declaration` as a pure
  module with `parse/1` (takes a file path, returns
  `{:ok, %Declaration{}}` or `{:error, {:declaration_error, reason}}`)
  and `validate/1` (takes a parsed struct, returns `:ok` or
  `{:error, reason}`). Cover all frontmatter schema fields from
  spec §3.2: `description` (≤120 chars), `kind` (literal `expert`),
  `schema-version` (integer, currently `1`), `tags` (non-empty
  kebab-case list), `scope.tier2` (non-empty glob list),
  `scope.tier1` (optional glob list), `retrieval.strategy` (literal
  `keyword_section`), `retrieval.top_n` (int default 8),
  `retrieval.max_bytes_per_entry` (int default 4096), `model`
  (string or nil), `dialogue.allow_clarifying_questions` (bool
  default true), `dialogue.max_clarification_rounds` (int default 3,
  hard cap 3). Body must be ≤8 KB (hard error on overflow).
  Reserved IDs per spec §3.4 rejected: `uncategorized`, `vault`,
  `plan`, `daemon`, `router`, `mnemosyne`. Every hard-error case
  has a dedicated Layer 1 unit test. Uses `YamlElixir` for
  frontmatter (already a dep).
- **Results:** _pending_

### Task 2 — `Mnemosyne.Expert.ScopeMatcher` `[impl]`
- **Status:** not_started
- **Dependencies:** Task 1 (Declaration struct type)
- **Description:** Implement the public scope-matching API that sub-E
  will consume: `match_candidate/2` takes a candidate's tag list and
  a list of loaded `%Declaration{}` structs and returns either
  `{:matched, [expert_id]}` or `:orphan`. Pure set-intersection —
  exact-string matching, no stemming, no case-insensitivity. Layer 1
  unit tests cover single-match, multi-match, orphan, empty
  candidate tags, empty expert tags (both directions), and case
  sensitivity regressions. Property test via `StreamData`: for any
  random `{expert_tags, candidate_tags}` pair, the result corresponds
  exactly to the experts whose tags intersect. See spec §6.1.
- **Results:** _pending_

### Task 3 — `Mnemosyne.ExpertRetrieval` behaviour definition `[impl]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Define the `@behaviour Mnemosyne.ExpertRetrieval`
  with callbacks `retrieve(query_text, scope, config)` returning
  `{:ok, [retrieved]} | {:error, term()}` and `name/0` returning an
  atom. The `retrieved` type is `%{path: String.t(), score: float(),
  snippet: String.t(), full_bytes: non_neg_integer()}`. See spec §4.1.
- **Results:** _pending_

### Task 4 — `Mnemosyne.ExpertRetrieval.KeywordSection` implementation `[impl]`
- **Status:** not_started
- **Dependencies:** Task 3
- **Description:** Implement the v1 retrieval strategy per spec §4.2:
  glob expansion over tier2 (relative to `<vault>/`) and tier1
  (relative to `<vault>/projects/*/mnemosyne/` with literal `*`
  meaning "any project"), term extraction with stopword filtering
  (~50-word English list baked in, tokens <3 chars dropped, ≥1 term
  post-filter required), parallel `rg --count-matches` via
  `Task.async_stream` bounded to `System.schedulers_online/0`,
  section-aware scoring per the §4.2 formula with module-level
  constant weights (1.0 × frontmatter tags, 0.6 × headings,
  0.4 × first paragraph, 0.2 × body, 0.1 × recency with 30-day
  half-life), per-file 64 KB read cap with `%ExpertFileTruncated{}`
  event, top-N snippet extraction with frontmatter + `max_bytes`
  body, 5-second retrieval wall-clock cap returning
  `{:error, :retrieval_timeout}`. Layer 1 unit tests via
  `test/support/fixtures/expert_retrieval/` corpus from Task 9.
  Determinism regression test (same input → same output on 10
  consecutive runs).
- **Results:** _pending_

### Task 5 — `Mnemosyne.Expert.Verdict` typed structs `[impl]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Define the typed structs for Stage 5 verdicts per
  spec §6.2: `%Mnemosyne.Expert.Verdict.Absorb{expert_id,
  ingestion_event_id, written_path}`, `%Verdict.Reject{expert_id,
  ingestion_event_id, reason}`, `%Verdict.CrossLink{from_expert,
  to_expert, ingestion_event_id}`, `%Verdict.MalformedResponse{
  expert_id, ingestion_event_id}`. Each has `@enforce_keys` and
  `@derive Jason.Encoder`. Layer 1 unit tests verify required fields
  enforce and JSON round-trips are lossless.
- **Results:** _pending_

### Task 6 — `Mnemosyne.Expert.Dialogue` struct + helpers `[impl]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Define the in-memory dialogue struct per spec §2.5:
  `%Mnemosyne.Expert.Dialogue{dialogue_id, expert_id, started_at,
  last_activity_at, turns}` where `turns :: [%Turn{role:
  :consumer | :expert, kind: :question | :clarifying_question |
  :answer, text: String.t(), retrieved_at: [path]}]`. Plus pure
  helper functions: `new/2`, `append_turn/3` (returns new dialogue
  with updated `last_activity_at`), `concatenate_text/1` (returns
  full transcript as a single string for retrieval input),
  `turn_count/1`, `clarification_count/1`. No ETS here — this is
  pure in-memory. Layer 1 unit tests.
- **Results:** _pending_

### Task 7 — `Mnemosyne.Expert.PromptBuilder` `[impl]`
- **Status:** not_started
- **Dependencies:** Tasks 1, 6
- **Description:** Build the session prompt per spec §5.5 and §6.2.
  Three distinct prompt shapes: (a) fresh Query (persona verbatim +
  retrieved snippets + question + sentinel instructions for
  `READY WITH ANSWER` or `READY WITH CLARIFICATION`), (b)
  dialogue-reply turn (persona + fresh retrieved snippets + full
  prior transcript as conversation history + new reply + sentinel
  instructions + optional forcing directive if clarification cap
  hit), (c) ingestion curation (persona + context block + full
  candidate markdown + adjacent-entries block (up to 3) + sentinel
  instructions for `READY ABSORB` / `READY REJECT <reason>` /
  `READY CROSS_LINK <expert-id>`). Pure module — takes structs,
  returns strings. Layer 1 unit tests snapshot the three shapes.
- **Results:** _pending_

---

## Phase 2 — Event structs and fixtures

### Task 8 — Default expert declarations in `priv/experts/` `[authoring]`
- **Status:** not_started
- **Dependencies:** Task 1 (parser validates these as smoke test)
- **Description:** Author the six starter expert declarations per
  spec §3.5: `rust-expert.md` (§3.1 is the template), plus
  `elixir-expert.md`, `research-expert.md`, `software-architect.md`,
  `obsidian-expert.md`, `ffi-expert.md`. Each declaration: complete
  frontmatter (description ≤120 chars, tags ≥5 kebab-case entries
  covering the domain's vocabulary, scope.tier2 globs for the
  expert's dedicated knowledge directory, optional scope.tier1
  cross-project globs where meaningful, retrieval strategy
  `keyword_section` with default config, `dialogue.max_clarification_rounds: 3`),
  plus a persona body with `# Persona`, `## How you answer`,
  `## How you curate` sections following the rust-expert example.
  Each under 8 KB. Parse-success fixture test in Task 1 asserts all
  six parse correctly.
- **Results:** _pending_

### Task 9 — Knowledge fixture corpus for retrieval tests `[authoring]`
- **Status:** not_started
- **Dependencies:** Task 4
- **Description:** Author ~20 hand-crafted markdown files under
  `test/support/fixtures/expert_retrieval/` with known tag
  distributions, section positions, and content. Coverage includes:
  (1) files matching a specific expert's scope with strong
  frontmatter tag hits, (2) files matching with only body hits,
  (3) files with heading-position hits, (4) files with known recency
  (mtime fixed in test setup), (5) files that exceed 64 KB for the
  truncation test, (6) files with malformed frontmatter for the
  parse-error path. Used by Task 4's scoring and determinism tests.
- **Results:** _pending_

### Task 10 — Declaration fixture set for failure modes `[authoring]`
- **Status:** not_started
- **Dependencies:** Task 1
- **Description:** Author one canonical valid declaration plus one
  failure-mode file per hard-error case from spec §7.3 under
  `test/support/fixtures/expert_declarations/`: malformed
  frontmatter, missing `description`, description >120 chars, body
  >8 KB, missing `kind`, wrong `kind`, unknown `schema-version`,
  empty `tags`, duplicate tags, non-kebab-case tags, missing
  `scope.tier2`, empty `scope.tier2`, malformed glob, unknown
  `retrieval.strategy`, negative `retrieval.top_n`, non-boolean
  `dialogue.allow_clarifying_questions`, `dialogue.max_clarification_rounds: 5`,
  reserved expert ID, placeholder description (`"TODO"`). Used by
  Task 1's validation tests.
- **Results:** _pending_

### Task 11 — `Mnemosyne.Event.Expert.*` sealed struct set `[impl]`
- **Status:** not_started
- **Dependencies:** Task 5 (shared struct conventions)
- **Description:** Define all 30 event structs per spec §9.5 under
  `lib/mnemosyne/event/expert/`. Each struct has `@enforce_keys`,
  `@derive Jason.Encoder`, and a consistent `:at` timestamp field.
  Groups: query/dialogue events (10), ingestion events (10),
  declaration-loader events (8), operational events (2). Layer 1
  unit tests verify `@enforce_keys` enforcement and JSON round-trip
  for every struct.
- **Results:** _pending_

---

## Phase 3 — Router message structs (early deliverable for sub-F)

### Task 12 — Router message structs `[impl]`
- **Status:** not_started
- **Dependencies:** none (sub-F router accepts the structs at routing
  time, not at compile time — the structs can land in sub-N's module
  tree and sub-F imports them when its router is ready)
- **Description:** Define the three message structs per spec §5.2:
  `%Mnemosyne.Message.ExpertQuery{target_expert_id,
  origin_qualified_id, origin_session_id, question, dialogue_id,
  started_at}`, `%Mnemosyne.Message.ExpertDialogueReply{
  target_expert_id, origin_qualified_id, origin_session_id,
  dialogue_id, reply, started_at}`,
  `%Mnemosyne.Message.ExpertAbsorbCandidate{target_expert_id,
  candidate, candidate_tags, source_stage, ingestion_event_id,
  started_at}`. Each has `@enforce_keys` and
  `@derive Jason.Encoder`. Layer 1 unit tests. These structs are
  **early deliverable** — they ship in sub-N's first PR so sub-F and
  sub-E can both code against real types.
- **Results:** _pending_

---

## Phase 4 — Singleton GenServers (pure Elixir infrastructure)

### Task 13 — `Mnemosyne.Expert.DialogueRegistry` GenServer `[impl]`
- **Status:** not_started
- **Dependencies:** Task 6 (Dialogue struct)
- **Description:** Implement the singleton GenServer per spec §2.2
  and §2.5. Owns the `:expert_dialogues` ETS table (`:set` type,
  key = `dialogue_id`). API: `new/2` (creates fresh dialogue,
  returns `{:ok, dialogue_id}`), `get/1` (returns `%Dialogue{}` or
  `:not_found`), `append_turn/3`, `expire/1`, `list/0` (for
  diagnostics only). Periodic sweeper via `:timer.send_interval/2`
  firing every 60 s; reaps entries where `last_activity_at + TTL <
  now`. TTL configurable via `daemon.toml` `[experts]
  dialogue_ttl_seconds` (default 1800). Emits `%ExpertDialogueExpired{}`
  on reap. Dialogue turn cap enforced at 8 (`:error,
  :dialogue_max_turns_exceeded` on append attempt exceeding cap).
  Sweeper time-travel test takes `now` as an argument. Layer 1 +
  Layer 2 unit tests.
- **Results:** _pending_

### Task 14 — `Mnemosyne.Expert.DeclarationLoader` GenServer `[impl]`
- **Status:** not_started
- **Dependencies:** Task 1, Task 11, Task 0 not required (walks
  filesystem at boot, file watcher optional at startup)
- **Description:** Implement the singleton GenServer per spec §2.2
  and §7.2. Runs `File.ls/1` on `<vault>/experts/` at `init/1`,
  parses every `*.md`, validates, registers valid ones in an
  internal ETS or Map cache, emits `%DeclarationError{}` for
  failures and `%ExpertRegistryReady{count, skipped}` when done.
  File watcher via `FileSystem` hex dep (add to `mix.exs` per spec
  Appendix B). Handles four file-event cases per spec §7.2: (1) new
  file added, (2) existing file edited (validate + update cache +
  cast to running actor, or skip if invalid keeping prior
  declaration — the committed "never silently lose an expert"
  discipline), (3) file deleted (expire dialogues + terminate
  actor), (4) file renamed (delete + add). API: `list/0`, `get/1`,
  `subscribe/0` (pg join for actor notification). Layer 1 + Layer 2
  unit tests including the mid-run hot-reload scenarios.
- **Results:** _pending_

### Task 15 — Early-deliverable PR to unblock sub-E `[deliverable]`
- **Status:** not_started
- **Dependencies:** Tasks 1, 2, 5, 11, 12
- **Description:** Ship a focused PR containing just the interfaces
  sub-E's amendment task needs to code against:
  `Mnemosyne.Expert.Declaration`, `Mnemosyne.Expert.ScopeMatcher`,
  `Mnemosyne.Expert.Verdict` structs, `Mnemosyne.Event.Expert.*`
  event structs, and `Mnemosyne.Message.ExpertAbsorbCandidate`. This
  lets sub-E's amendment task run in parallel with sub-N's remaining
  tasks (16+). The PR does NOT include `ExpertActor` itself — that's
  Task 16. Document the public API surface sub-E can rely on.
- **Note:** Sub-E's design doc was rewritten inline in Session 17
  (2026-04-16). That rewrite consumed and locked in the following
  contract shapes — Task 15's output **must match them exactly**:
  `%Mnemosyne.Message.ExpertAbsorbCandidate{}` struct fields,
  `Mnemosyne.Expert.ScopeMatcher.match_candidate/2` signature,
  the three verdict variants `:absorb` / `:reject` /
  `:cross_link_suggested`, and `%Mnemosyne.Event.Expert.Conflict{}`.
  Do not alter these shapes without coordinating with sub-E.
- **Results:** _pending_

---

## Phase 5 — ExpertActor itself (gates on Task 0)

### Task 16 — `Mnemosyne.ExpertActor` skeleton `[impl]`
- **Status:** not_started
- **Dependencies:** Task 0 (gate), Tasks 1, 13, 14
- **Description:** Implement the GenServer implementing F's
  `Mnemosyne.Actor` behaviour per spec §2.1 and §7.1. `init/1` loads
  declaration from `DeclarationLoader`, fails `Faulted` on missing.
  `handle_actor_message/2` dispatches on message type (three kinds
  from Task 12). `snapshot/1` returns
  `%Mnemosyne.Expert.Snapshot{declaration_id, messages_processed,
  last_activity_at}`. Idle timeout handling per spec §7.1: 5 min
  default transitions to `Dormant` via self-message, except when a
  dialogue is open (registry check) in which case the timer resets.
  Supervision placement: `ActorSupervisor` child spec registered by
  `DeclarationLoader` at boot. Layer 2 integration test with
  FixtureReplay harness covers the basic receive-dispatch-reply
  loop.
- **Results:** _pending_

### Task 17 — Query flow (fresh query path) `[impl]`
- **Status:** not_started
- **Dependencies:** Task 16, Task 3, Task 4, Task 7, sub-C's
  `FixtureReplay` adapter available
- **Description:** Implement `handle_actor_message/2` for
  `%ExpertQuery{}` per spec §5.3 step 1. Flow: create dialogue in
  registry → run retrieval → build prompt → spawn session via
  `Mnemosyne.HarnessAdapter.spawn/1` → attach as consumer → drive
  sliding-buffer sentinel matcher on assistant-text stream with
  5-min wall-clock bound → parse disposition → append turn → emit
  events (`%QueryStarted{}`, `%DialogueTurn{}`, `%QueryAnswered{}`)
  → return result to router. Bounded by `turn_timeout_seconds` with
  kill via sub-C process group. Layer 2 tests with FixtureReplay
  canned streams covering answer + clarifying-question + missing
  sentinel + turn timeout paths.
- **Results:** _pending_

### Task 18 — Multi-turn dialogue flow `[impl]`
- **Status:** not_started
- **Dependencies:** Task 17
- **Description:** Implement `handle_actor_message/2` for
  `%ExpertDialogueReply{}` per spec §5.3 step 2. Flow: lookup
  dialogue → reject if not_found_or_expired → validate
  target_expert match → check turn cap → retrieve on concatenated
  transcript text → build prompt with full transcript as
  conversation history → spawn fresh session → sentinel match →
  append turn → emit events → return. Layer 2 test covering the
  full turn-1 → turn-2 flow with FixtureReplay emitting
  clarifying-question then answer.
- **Results:** _pending_

### Task 19 — Clarification cap enforcement `[impl]`
- **Status:** not_started
- **Dependencies:** Task 18
- **Description:** Implement the clarification-cap enforcement per
  spec §5.7. Track clarification count in the dialogue struct. When
  count equals `dialogue.max_clarification_rounds`, append the
  forcing directive to the next prompt. If the forced-answer round
  still emits `READY WITH CLARIFICATION`, emit
  `%ClarificationCapReached{forced_failed: true}` and return
  `{:error, :clarification_budget_exhausted, dialogue_id}`. Layer 2
  test with FixtureReplay rigged to emit clarification forever.
- **Results:** _pending_

### Task 20 — `%ExpertAbsorbCandidate{}` flow `[impl]`
- **Status:** not_started
- **Dependencies:** Task 17 (session-spawn machinery), Task 4
  (retrieval for adjacent entries)
- **Description:** Implement `handle_actor_message/2` for the
  ingestion message per spec §6.2. Flow: build curation prompt
  (persona + context + candidate + up-to-3 adjacent entries via
  retrieval on candidate body + instruction block with ingestion
  sentinels) → spawn session → sentinel match → apply verdict: on
  `READY ABSORB`, resolve write target from first `scope.tier2`
  glob entry, atomic write with temp-then-rename, append provenance
  frontmatter (`absorbed-by`, `ingestion-event-id`, `absorbed-at`),
  emit `%ExpertAbsorb{}` with written path; on `READY REJECT
  <reason>`, emit `%ExpertReject{}`, return
  `{:rejected, reason}`; on `READY CROSS_LINK <expert-id>`, emit
  `%ExpertCrossLinkSuggested{}`, return
  `{:cross_link_suggested, to_expert}`. Layer 2 tests for all three
  verdict paths including provenance frontmatter verification.
- **Results:** _pending_

---

## Phase 6 — Hot reload wiring and error paths

### Task 21 — Hot reload integration in `DeclarationLoader` + `ExpertActor` `[impl]`
- **Status:** not_started
- **Dependencies:** Tasks 14, 16
- **Description:** Wire the four file-event cases from Task 14 into
  running actor behavior per spec §7.2. Case 2 (edit): actor
  receives `{:declaration_updated, new_declaration}` cast, drains
  current work (in-flight session finishes or times out normally),
  calls internal `:reinit_with_new_declaration` — no process restart,
  just state update. Case 3 (delete): actor receives
  `{:declaration_removed}`, rejects new messages with
  `:expert_removed`, drains in-flight, transitions to
  `ShuttingDown → Dormant → terminated`. Open dialogues for the
  removed expert are expired via `DialogueRegistry.expire/1`.
  Layer 2 tests for each case including the committed "bad edit
  never silently loses a working expert" discipline test.
- **Results:** _pending_

### Task 22 — Full error-disposition matrix coverage `[test]`
- **Status:** not_started
- **Dependencies:** Tasks 17, 18, 19, 20
- **Description:** Layer 2 test per row of the §7.3 disposition
  matrix. Every failure mode produces the specified event or
  tool-call error with the specified machine-readable code. No
  silent fall-through. This is the regression gate on the
  hard-errors-by-default discipline for sub-N. Test harness helpers
  for stubbing `DeclarationLoader`, `DialogueRegistry`,
  `FixtureReplay` to inject each failure mode cleanly.
- **Results:** _pending_

---

## Phase 7 — Init flow integration (sub-A consumer)

### Task 23 — Extend `mnemosyne init` to copy `priv/experts/*.md` `[impl]`
- **Status:** not_started
- **Dependencies:** Task 8 (default declarations exist in
  `priv/experts/`), sub-A's init command landed
- **Description:** Amend sub-A's init task to include a step that
  copies `priv/experts/*.md` into `<vault>/experts/` at init time.
  Uses `@external_resource` in the Elixir compile-time embedding
  pattern sub-B uses for phase prompts. This is a small cross-plan
  amendment; sub-N's backlog owns it because sub-N owns the
  `priv/experts/` contents. Coordinate with sub-A sibling-plan to
  slot the change into sub-A's init implementation task. Layer 2
  test: run init against a tmp vault, assert all six declarations
  land at `<vault>/experts/`.
- **Results:** _pending_

### Task 24 — Extend `mnemosyne init` to create `<vault>/knowledge/uncategorized/` `[impl]`
- **Status:** not_started
- **Dependencies:** sub-A's init command landed
- **Description:** Tiny amendment to sub-A's init: create the
  `<vault>/knowledge/uncategorized/` directory at init time.
  Required for Stage 5 orphan-candidate writes per spec §6.1. No
  content; just the directory. Can be a single-line addition to the
  existing `File.mkdir_p!/1` block in sub-A's init flow.
- **Results:** _pending_

---

## Phase 8 — Layer 3 live tests

### Task 25 — Live fresh query round-trip test `[test :live]`
- **Status:** not_started
- **Dependencies:** Task 17, real `claude` CLI available
- **Description:** Implement the `@moduletag :live` test from spec
  §8.3 case 1. Minimal vault with `rust-expert.md` and one knowledge
  file `<vault>/knowledge/rust/lifetimes.md`. Send
  `%ExpertQuery{question: "What does 'static mean on a trait bound?"}`
  via the router. Assertions: session spawned, assistant output
  contains text AND `READY WITH ANSWER` sentinel, tool result has
  `kind: "answer"`, citation list includes the one knowledge file,
  full flow completes in <2 min on haiku. Budget haiku pricing.
- **Results:** _pending_

### Task 26 — Live multi-turn dialogue test `[test :live]`
- **Status:** not_started
- **Dependencies:** Task 18, Task 25 setup
- **Description:** Implement §8.3 case 2. Ambiguous question (e.g.,
  "How do I handle errors in async code?"). Turn 1 returns
  `clarifying_question`, simulated reply via
  `%ExpertDialogueReply{}`, turn 2 returns `answer`. Assert
  sub-M event log has exactly two `%ExpertDialogueTurn{}` events
  with matching `dialogue_id`.
- **Results:** _pending_

### Task 27 — Live Stage 5 ingestion absorb test `[test :live]`
- **Status:** not_started
- **Dependencies:** Task 20, Task 25 setup
- **Description:** Implement §8.3 case 3. Empty
  `<vault>/knowledge/rust/` directory. Hand-crafted Rust-scoped
  candidate with tags `[rust, lifetimes]` and a concrete claim
  about borrow-checker behavior. Assert: `READY ABSORB`, file
  written at `<vault>/knowledge/rust/<slug>.md`, provenance block
  in frontmatter, `%ExpertAbsorb{}` event fired.
- **Results:** _pending_

### Task 28 — Live cmux-noise mitigation smoke test `[test :live]`
- **Status:** not_started
- **Dependencies:** Task 25 setup
- **Description:** Implement §8.3 case 4. Assert that
  `--setting-sources project,local --no-session-persistence` is in
  effect for every sub-N-spawned session and that no cmux
  SessionStart hook JSON appears in the captured assistant-text
  stream. This is a regression gate on the sub-C cross-cutting
  discipline.
- **Results:** _pending_

---

## Phase 9 — v1 acceptance

### Task 29 — v1 acceptance dogfood pass `[acceptance]`
- **Status:** not_started
- **Dependencies:** All prior tasks (1-28)
- **Description:** Run the full sub-N suite against a real minimal
  vault with all six default experts loaded. Exercise Query,
  dialogue, and Stage 5 absorb flows end-to-end. Fix any gaps
  discovered. Verify `mix test` passes (Layer 1 + Layer 2),
  `mix test --only live` passes (Layer 3), `mix format --check-formatted`
  passes, `mix dialyzer` has no new warnings. Sign-off event:
  `%Expert.V1Ready{}` emitted via `Observability.emit/1`. This is
  the gate that says sub-N is ready to be integrated into the
  v1 release cut decision.
- **Results:** _pending_

---

## Task summary

- **Phase 1 (Tasks 1–7):** Pure Elixir modules — can start
  immediately, no F dependency.
- **Phase 2 (Tasks 8–11):** Default declarations, fixture libraries,
  event structs — authoring and compile-time work, no runtime
  dependencies.
- **Phase 3 (Task 12):** Router message structs — early deliverable
  for sub-F.
- **Phase 4 (Tasks 13–14):** Singleton GenServers — Elixir runtime
  but no F dependency.
- **Phase 5 (Tasks 15–20):** Early-deliverable PR for sub-E
  (Task 15) then `ExpertActor` itself (Tasks 16–20) gated on Task 0.
- **Phase 6 (Tasks 21–22):** Hot reload wiring and error-matrix
  coverage.
- **Phase 7 (Tasks 23–24):** Init-flow integration with sub-A.
- **Phase 8 (Tasks 25–28):** Layer 3 `@moduletag :live` tests.
- **Phase 9 (Task 29):** v1 acceptance.

## Out of scope for this plan

- Vector-store / semantic retrieval → sub-Q (new brainstorm task on
  orchestrator backlog)
- Knowledge ontology → sub-R (new brainstorm task on orchestrator
  backlog)
- Dynamic expert creation via TUI → sub-H
- Rust TUI observation of expert dialogues → separate TUI plan
- Obsidian plugin views → sub-K (v1.5+)
- Per-expert model override → sub-O (v1.5+)
- Team-mode distributed experts → sub-P (v2+)
