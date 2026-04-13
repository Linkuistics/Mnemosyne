# Work Phase — Sub-project C: Harness Adapter Layer

Read the following before doing anything else:

1. `{{PROJECT}}/README.md` for Mnemosyne's project conventions, architecture,
   CLI surface, and v0.1.0 status. Mnemosyne is a Rust project; build and
   test commands are documented there.
2. `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md` for the work / reflect / triage
   phase cycle specification.
3. `{{DEV_ROOT}}/LLM_CONTEXT/coding-style.md` and
   `{{DEV_ROOT}}/LLM_CONTEXT/coding-style-rust.md` for coding conventions.
4. `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-C-adapters-design.md`
   — the authoritative design document for this sub-project. Every task
   below derives from this spec. Consult it before starting any task.
5. `{{PLAN}}/backlog.md` for the current implementation task list.
6. `{{PLAN}}/memory.md` for implementation-level decisions, cross-sub-project
   dependencies, and implementation strategy notes.

## About this plan

This plan implements sub-project C of the Mnemosyne orchestrator merge —
the harness adapter layer (`HarnessAdapter` trait, `ClaudeCodeAdapter`,
`FixtureReplayAdapter`) that lets Mnemosyne spawn, control, observe, and
terminate LLM coding harnesses as managed child processes.

The design is fully specified in the design doc referenced above; this
plan is the implementation work, not a design phase. Do not re-litigate
design decisions here; surface them to the parent plan
(`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`) if you discover
a spec-level issue during implementation.

### Sub-project C is on the critical path for sub-project B

Sub-C exists primarily to unblock sub-B's v1 dogfood acceptance test.
B's `LlmHarnessExecutor` currently holds a stub adapter; landing C's real
`ClaudeCodeAdapter` is the swap that turns B's executor from "tested
against fixture replay" into "running real Claude Code sessions". The
final task in this plan's backlog (the C-1 dogfood acceptance gate) is
that swap, executed at the orchestrator-plan level.

### Key constraints during implementation

- **Hard errors by default.** Unexpected conditions, schema drift,
  malformed events, profile violations all fail loud and fast with clear
  diagnostics. No silent degradation. Soft fallbacks require explicit
  written rationale in the code.
- **TDD.** Every type and every behavioural rule in the spec has unit or
  integration tests written first. The three test layers (parser units /
  fixture integration / gated live integration) are documented in §8 of
  the spec.
- **Threading correctness via single-owner-per-state actor architecture.**
  No mutexes guarding fields except the unavoidable `Mutex<Option<JoinHandle>>`
  consumed by `wait()`. All session state lives inside one actor thread;
  all interactions are typed messages on typed channels (crossbeam-channel).
  See §4.2 of the spec.
- **Process-group termination from v1.** `process_group(0)` at spawn time,
  `nix::sys::signal::killpg` at terminate time, two-phase SIGTERM→SIGKILL
  escalation with a 500ms grace. Not a v1.5 deferral. See §4.4 of the spec.
- **Bidirectional stream-json, no PTY.** The Q1 decision is settled. Do
  not introduce `portable-pty` or any terminal escape parser. See §4.1
  of the spec.
- **Defence-in-depth tool enforcement.** Spawn-time CLI flags + stream-side
  runtime check inside the actor. Trust the flag, verify the stream.
  See §6 of the spec.
- **No callback channel from harness to Mnemosyne, but observation IS
  permitted.** The "no callback channel" rule means the harness cannot
  call into Mnemosyne; it does NOT forbid Mnemosyne from observing
  harness state. `SessionLifecycle` chunks are how protocol-level state
  transitions are surfaced; sentinel detection in B's executor is how
  task-level completion is detected. See §3.3 and §4.3.3.
- **Test and build commands**: see `{{PROJECT}}/README.md`. For Rust work,
  use `cargo test`, `cargo clippy`, and `cargo +nightly fmt` per
  `{{DEV_ROOT}}/LLM_CONTEXT/coding-style-rust.md`.

### Dependencies on sibling sub-projects

Sub-project C's code is *consumed by* sub-project B's `LlmHarnessExecutor`.
B is in "brainstorm done, implementation not started" status, so the
trait declaration C ships will become part of B's compilation unit when
B's executor is built. Sub-C also forces five amendments to B's draft
trait surface — see §11.1 of the spec for the full list. Those amendments
are recorded as cross-sub-project requirements; B's implementation phase
absorbs them.

Sub-C does NOT block on any other sibling sub-project. C runs standalone
and is the next-most-critical brainstorm pick after B (per the orchestrator
parent plan's recommended ordering).

## Path placeholders

Any path beginning with `{{PROJECT}}`, `{{DEV_ROOT}}`, or `{{PLAN}}` should
be interpreted as the absolute path the shell script substitutes before
passing the prompt to the LLM. If you see a literal `{{PROJECT}}`,
`{{DEV_ROOT}}`, or `{{PLAN}}` token in any file you Read inside the dev
root, substitute it mentally with the correct absolute path before passing
it to the Read tool.

## Working a task

1. Display a summary of the current backlog: title, status, and the
   relative priority order (top of the backlog file = highest priority).
2. Ask the user if they have input on which task to work on next. Wait
   for their response. If they have a preference, work on that task;
   otherwise pick the highest-priority `not_started` task whose
   dependencies are all `done`.
3. Work the task using TDD: write the failing test first, implement the
   minimum code to pass, refactor, commit. Consult the design doc for
   any behavioural question.
4. Run the full test suite and clippy before declaring the task done.
   No task ships with failing tests or new warnings.
5. Record results in `{{PLAN}}/backlog.md` — replace `_pending_` with
   a concrete summary of what was built, tests added, and any
   surprises. Update the task `Status` to `done`.
6. Append a session log entry to `{{PLAN}}/session-log.md` per the
   format in `{{DEV_ROOT}}/LLM_CONTEXT/backlog-plan.md`:
   `### Session N (YYYY-MM-DD) — title`, bullets for what was attempted,
   what worked / didn't, what to try next, key learnings.
7. Write `reflect` to `{{PLAN}}/phase.md`.
8. Stop. Do not pick another task. Do not enter the reflect phase
   yourself — the next phase runs in a fresh session.
