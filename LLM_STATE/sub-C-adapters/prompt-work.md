# Work Phase — Sub-project C: Harness Adapter Layer

Read these plan-specific references before working a task:

1. `{{DEV_ROOT}}/LLM_CONTEXT/fixed-memory/coding-style.md` and
   `{{DEV_ROOT}}/LLM_CONTEXT/fixed-memory/coding-style-rust.md` for
   coding conventions.
2. `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-C-adapters-design.md`
   — the authoritative design document for this sub-project. Every task
   in the backlog derives from this spec. Consult it before starting
   any task; do not re-litigate design decisions here. If you discover
   a spec-level issue, surface it to the parent plan
   (`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/memory.md`).

## About this plan

This plan implements sub-project C of the Mnemosyne orchestrator merge —
the harness adapter layer (`HarnessAdapter` trait, `ClaudeCodeAdapter`,
`FixtureReplayAdapter`) that lets Mnemosyne spawn, control, observe, and
terminate LLM coding harnesses as managed child processes.

### Sub-project C is on the critical path for sub-project B

Sub-C exists primarily to unblock sub-B's v1 dogfood acceptance test.
B's `LlmHarnessExecutor` currently holds a stub adapter; landing C's real
`ClaudeCodeAdapter` is the swap that turns B's executor from "tested
against fixture replay" into "running real Claude Code sessions". The
final task in this plan's backlog (the C-1 dogfood acceptance gate) is
that swap, executed at the orchestrator-plan level.

## Key commands

For Rust work in this project, use:

- `cargo test` — full test suite
- `cargo clippy` — lint; no new warnings allowed
- `cargo +nightly fmt` — formatting per `coding-style-rust.md`

Run all three before declaring a task done.

## Constraints

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

## Dependencies on sibling sub-projects

Sub-project C's code is *consumed by* sub-project B's `LlmHarnessExecutor`.
B is in "brainstorm done, implementation not started" status, so the
trait declaration C ships will become part of B's compilation unit when
B's executor is built. Sub-C also forces five amendments to B's draft
trait surface — see §11.1 of the spec for the full list. Those amendments
are recorded as cross-sub-project requirements; B's implementation phase
absorbs them.

Sub-C does NOT block on any other sibling sub-project.
