# Sub-project N — Work phase

## Primary reference

`{{PROJECT}}/docs/superpowers/specs/2026-04-15-sub-N-domain-experts-design.md`
is the authoritative design doc. Read §1–§11 before starting any
implementation task. Appendix A is the decision trail, Appendix B is
the `mix.exs` projection, Appendix C is the glossary.

## About this plan

Sub-project N implements the `ExpertActor` type declared as a type
hole in sub-F §4.3, plus the surrounding machinery: declaration
format, retrieval strategy chokepoint, dialogue protocol with
multi-turn clarification, and the Stage 5 ingestion contract that
sub-E's amendment will consume.

The spec is frozen; this plan executes against it. Drift from the
spec means rewriting the spec inline (not appending a supersede
block) per the project's pivot-rewrite discipline captured in
`{{PROJECT}}/memory/feedback_pivot_rewrite.md`.

## Key constraints

- **Task 0 is a readiness gate** for actor-integration tasks. Pure
  Elixir tasks (Declaration parsing, ScopeMatcher, KeywordSection
  retrieval, PromptBuilder, Verdict structs, Dialogue structs) can
  start immediately without waiting on sub-F's supervisor integration.
  Tasks that spawn or supervise `ExpertActor` gate on F delivering
  `Mnemosyne.Actor` behaviour + `ActorSupervisor` child-spec API.
- **Two explicit timers** (retrieval 5s structural vs session 5 min
  reasoning budget) must not be collapsed. The daemon config surface
  is `[experts] turn_timeout_seconds`, `[experts] dialogue_ttl_seconds`.
- **Inline rewrite discipline** applies if upstream interfaces drift
  — rewrite the design doc's affected sections rather than stacking
  amendments. See `{{PROJECT}}/memory/feedback_pivot_rewrite.md`.
- **Hard errors by default.** Every failure mode in spec §7.3 emits
  an event or surfaces as a typed tool-call error. No silent falls to
  a less-functional mode. One documented exception: the 64 KB
  per-file retrieval truncation (spec §7.4).
- **Filesystem as substrate.** Persistent state is on disk (expert
  declarations tracked in vault git; provenance in frontmatter).
  Dialogue transcripts are transient in ETS — not on disk — per the
  design-phase decision. Audit trail of dialogues lives in sub-M's
  event log.
- **Obsidian-native formats.** YAML frontmatter for machine-readable
  fields, markdown body for persona prose, Dataview-friendly
  kebab-case field names.
- **TDD.** Write tests alongside implementation; Layer 1 unit tests
  are the spine of the test harness and must exist before Layer 2
  GenServer integration tests for the same module.

## Coding style

At the moment you are about to write or modify code, and not before,
check `{{DEV_ROOT}}/LLM_CONTEXT/fixed-memory/`:

- Always read `coding-style.md` (universal rules).
- Also read `coding-style-elixir.md` if you are about to touch Elixir
  (which is most of this plan's work). If no file matches the
  language, there is no language-specific guidance.

If a task involves no code (pure docs, planning, backlog triage),
skip this section entirely.

## Elixir commands

Standard Mnemosyne Elixir workflow:

- `mix test` — full default suite (Layer 1 unit + Layer 2
  FixtureReplay integration). Target <5 s total wall clock for
  sub-N's contribution.
- `mix test --only live` — Layer 3 `@moduletag :live` tests only.
  Requires `claude` CLI + Anthropic API credentials. Targets haiku;
  budget ~$0.02–$0.05 per full live-test run.
- `mix format --check-formatted` — enforce project format.
- `mix dialyzer` — type-flow check; sub-N's modules should pass
  without new warnings.
- `mix mnemosyne.dev.record_fixture` (sub-C) — for recording
  FixtureReplay input streams from real sessions, then replaying
  them in Layer 2 tests.

Live tests must NOT run in parallel within one test run — they share
the daemon socket and vault directory. `setup_all` in
`test/live/expert/` serializes them.

## Sibling dependencies (what's needed from where)

- **Sub-F** — `Mnemosyne.Actor` behaviour, `ActorSupervisor`
  child-spec registration, `Router` message delivery, five-state
  lifecycle, BEAM PTY spike (done). F Task 0 gate may clear
  independently of sub-N's Task 0 gate; sub-N can start pure-Elixir
  work immediately.
- **Sub-B** — `Sentinel.SlidingBuffer` module. Consumed directly for
  sentinel matching. Sub-B's own downstream task-list rewrite does
  not block sub-N (sub-N just needs the module to exist in the
  codebase).
- **Sub-C** — `ClaudeCodeAdapter`, session GenServer pattern,
  `FixtureReplay` adapter, tool-call boundary with injected tool
  set, cmux mitigation flags. Sub-C's downstream task-list rewrite
  does not block sub-N's Layer 1 unit tests but does block Layer 2
  GenServer integration tests.
- **Sub-M** — `Mnemosyne.Observability.emit/1`, sealed event set.
  Sub-N contributes `Mnemosyne.Event.Expert.*` to sub-M's §4.1 set.
- **Sub-A** — vault layout (`<vault>/experts/`,
  `<vault>/knowledge/`, `<vault>/runtime/snapshots/`). Sub-A's init
  flow must copy `priv/experts/*.md` to `<vault>/experts/` — a small
  amendment to sub-A's init task.
- **Sub-E** — not a dependency; sub-N delivers the Stage 5 contract
  for sub-E's amendment task to consume. Sub-N's Task 15 delivers an
  early PR with scope matcher + message structs so sub-E can start
  its amendment in parallel.

## Out of scope for sub-N (hand-off for other plans)

- **Vector-store retrieval** → sub-Q (new brainstorm task on
  orchestrator backlog)
- **Knowledge ontology** → sub-R (new brainstorm task on
  orchestrator backlog)
- **Dynamic expert creation via TUI** → sub-H
- **Rust TUI integration for expert observation** → separate TUI plan
- **Obsidian plugin views of expert dialogues** → sub-K (v1.5+)
