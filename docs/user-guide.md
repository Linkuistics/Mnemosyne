# Mnemosyne User Guide

> **Status:** Mnemosyne is in architectural design phase. This guide describes the **intended v1 user experience** based on the committed architecture. Code has not yet been written. Commands and flags are indicative, not final.

This guide walks through the Mnemosyne user experience: installation, first-time setup, daily workflow, and the mental model for using the daemon + TUI.

## Contents

1. [What Mnemosyne is](#what-mnemosyne-is)
2. [Mental model](#mental-model)
3. [Installation](#installation)
4. [First-time setup](#first-time-setup)
5. [Adopting a project](#adopting-a-project)
6. [Running the daemon](#running-the-daemon)
7. [Using the TUI](#using-the-tui)
8. [Working on a plan](#working-on-a-plan)
9. [Creating sub-plans](#creating-sub-plans)
10. [Consulting experts](#consulting-experts)
11. [Dispatching cross-plan tasks](#dispatching-cross-plan-tasks)
12. [Editing routing rules](#editing-routing-rules)
13. [Obsidian browsing](#obsidian-browsing)
14. [Multi-machine sync](#multi-machine-sync)
15. [Troubleshooting](#troubleshooting)

## What Mnemosyne is

Mnemosyne is a **persistent orchestration daemon** for AI-assisted development. Instead of opening individual Claude Code sessions and hoping they remember yesterday's context, you run Mnemosyne once on your machine and it hosts:

- **Plan actors** — one per plan you're working on, tracking phase cycles, backlogs, memory, and dispatched tasks
- **Expert actors** — consultative domain experts you can query like co-workers
- **Routing** — declarative rules that send tasks to the right place automatically
- **Ingestion** — automatic promotion of plan-phase learnings into durable knowledge

You drive the daemon through a **TUI client** (written in Rust with `ratatui`) or, in future versions, through Obsidian or a web UI. All clients attach to the same daemon over a local Unix socket, so you can run multiple clients simultaneously without conflicting.

For the architectural overview, see [`architecture.md`](architecture.md).

## Mental model

The shift from Mnemosyne v0.1.0 (a CLI tool) to v1 (a daemon) is significant. The key ideas:

### You don't "run Mnemosyne" per plan

The daemon is **always on**, like a background service. You start it once per machine (or let it auto-start on first login) and it hosts every plan across every adopted project.

### Plans are where work happens

A **plan** is a directory containing `plan-state.md`. Plans are nested — a project has a single `project-root` plan, which may have sub-plans, which may have sub-sub-plans. Each plan runs its own four-phase cycle (work → reflect → compact → triage) and has its own memory and backlog.

You spend most of your time driving one plan at a time through the TUI. The TUI attaches to that plan's actor in the daemon, and you see its current state, backlog, and streaming output from any active LLM session.

### Knowledge is consultative

Instead of reading knowledge files yourself or loading them into your LLM session, you **ask an expert**. Experts are actors with narrow domain scope and curated knowledge. When you query an expert, it spawns its own fresh-context session with just the relevant knowledge and returns an answer. Your session never pollutes with the expert's knowledge.

This is the key fresh-context discipline: **don't load what you can ask for**.

### Dispatches are how plans talk to each other

If a plan identifies a concern that belongs in a different plan — same project or different project — it dispatches a task. The daemon routes the dispatch via declarative rules (user-editable, hot-reloadable) or, when rules don't decide, a fresh-context routing agent.

Dispatches have audit trails on both ends: the origin's `Dispatched` section and the target's `Received` section.

## Installation

**V1 installation path** (indicative — not yet implemented):

```bash
# Install the Elixir daemon
mix archive.install hex mnemosyne

# Install the Rust TUI client
cargo install mnemosyne-tui

# Verify
mnemosyne --version
mnemosyne-tui --version
```

Prerequisites:

- **Erlang/OTP 27+** and **Elixir 1.17+** for the daemon
- **Rust 1.80+** for the TUI client (or download a pre-built binary)
- **Git** for vault and project versioning
- **Obsidian** (recommended) for browsing the vault
- **Claude Code** as the v1 harness (other harnesses in v1.5+)

## First-time setup

On first run, Mnemosyne creates a dedicated vault at `~/Mnemosyne-vault/` (configurable):

```bash
$ mnemosyne daemon --init
Creating vault at /Users/you/Mnemosyne-vault/...
  • Vault identity: mnemosyne.toml (schema v1)
  • Obsidian template: .obsidian/ with Dataview + Templater
  • Default routing rules: routing.ex (empty scaffolding)
  • Default daemon config: daemon.toml
  • Knowledge directories: knowledge/{languages,domains,tools,techniques,projects}
  • Experts directory: experts/ (starter set of 6 experts)
  • Runtime directory: runtime/ (ephemeral)

Daemon started. Socket: /Users/you/Mnemosyne-vault/runtime/daemon.sock
Attach a client with `mnemosyne-tui` in another terminal.
```

The vault is a **git repository**. Mnemosyne commits its own writes automatically; you can commit manual edits anytime. The vault can be synced between machines via `git push` / `git pull`.

## Adopting a project

Before you can drive a plan inside a project, you adopt the project into your vault:

```bash
$ cd ~/dev/my-awesome-project
$ mnemosyne adopt-project .
Adopting my-awesome-project into vault at /Users/you/Mnemosyne-vault/
  • Creating /Users/you/dev/my-awesome-project/mnemosyne/
  • Creating project-root plan
  • Creating vault symlink: /Users/you/Mnemosyne-vault/projects/my-awesome-project
  • Regenerating plan-catalog.md
  • Rescanning vault

Project adopted. Root plan: my-awesome-project/project-root
Run `mnemosyne-tui attach my-awesome-project/project-root` to start working.
```

Adoption does three things:

1. Creates `<project>/mnemosyne/project-root/` with an empty root plan.
2. Creates a vault symlink at `<vault>/projects/<project-name>` pointing at the project's mnemosyne directory. This makes the project visible in Obsidian and to the daemon.
3. Regenerates `<vault>/plan-catalog.md` so the new project appears in catalogs.

## Running the daemon

After initial setup, the daemon runs as a background process:

```bash
# Start in the background (daemonize)
$ mnemosyne daemon --background
Daemon started. PID: 48291. Logs: /Users/you/Mnemosyne-vault/runtime/daemon.log

# Or start in the foreground with log output
$ mnemosyne daemon
[info] Loading vault at /Users/you/Mnemosyne-vault/
[info] VaultDirectory cached: 12 plans, 6 experts
[info] Listening on /Users/you/Mnemosyne-vault/runtime/daemon.sock
[info] Daemon ready
```

Common daemon commands:

```bash
mnemosyne daemon                 # start in foreground
mnemosyne daemon --background    # start in background
mnemosyne stop                   # graceful shutdown
mnemosyne status                 # query daemon health
mnemosyne rescan                 # refresh vault catalog and routing rules
mnemosyne logs                   # tail daemon logs
```

## Using the TUI

The TUI is where you interact with plans day-to-day.

```bash
$ mnemosyne-tui
```

The TUI opens and shows the **plan catalog** — every plan in the vault, grouped by project, with current phase states. You navigate with arrow keys, select a plan, and press Enter to attach.

```
┌─ Mnemosyne ─────────────────────────────────────────────────┐
│ Plans in vault                                               │
│                                                              │
│ ▼ Mnemosyne                                                  │
│     project-root                 [reflect]  2h ago          │
│   ▼ sub-F-hierarchy              [work*]    active now      │
│       sub-F-dispatch             [idle]                     │
│     sub-B-phase-cycle            [triage]   1d ago          │
│                                                              │
│ ▼ APIAnyware-MacOS                                           │
│     project-root                 [idle]                     │
│     sub-ffi-callbacks            [reflect]  3h ago          │
│                                                              │
│ Experts                                                      │
│   rust-expert                    dormant                    │
│   research-expert                dormant                    │
│   ffi-expert                     active                     │
│                                                              │
│ [Enter] attach  [r] rescan  [q] quit                        │
└──────────────────────────────────────────────────────────────┘
```

### Plan view

Attaching to a plan shows the plan's state: current phase, backlog, memory summary, recent session log, and live harness output if a phase is running.

```
┌─ Mnemosyne/project-root/sub-F-hierarchy ──── [work] ─────────┐
│ Description: Plan hierarchy, actor model, inter-plan        │
│              dispatch, declarative routing.                  │
│                                                              │
│ Backlog (3 tasks)                                            │
│   1. Write design doc for F              [in_progress]      │
│   2. Scaffold sibling plan               [blocked: spike]   │
│   3. Land amendment tasks for A, B, C    [not_started]      │
│                                                              │
│ Memory (14 entries)                                          │
│   ▸ Plan hierarchy via project-root convention              │
│   ▸ Dispatch target resolution asymmetry                    │
│   ▸ Actor-per-plan in BEAM daemon                           │
│   ...                                                        │
│                                                              │
│ Session output (streaming)                                   │
│   Writing docs/superpowers/specs/2026-04-14-sub-F-...       │
│   ✓ Wrote 2188 lines to ...                                  │
│                                                              │
│ [w] work  [r] reflect  [c] compact  [t] triage              │
│ [d] dispatch  [q] query  [e] edit routing  [Esc] detach     │
└──────────────────────────────────────────────────────────────┘
```

Keybindings are configurable. Standard vim-like navigation works throughout.

## Working on a plan

A plan's life is a repeating four-phase cycle: **work → reflect → compact → triage**.

### Work phase

Press `w` in the plan view. The TUI sends `run_phase work` to the daemon, which:

1. Renders a staging directory for the plan (placeholders substituted, prompts embedded, vault catalog injected).
2. Spawns a Claude Code session via the harness adapter.
3. Streams output to the TUI in real time.
4. On session exit, runs the `DispatchProcessor` and `QueryProcessor` on any `dispatches.yaml` / `queries.yaml` files the phase produced.
5. Updates `plan-state.md` and transitions to idle (or directly to reflect if auto-advance is enabled).

You watch the work happen in the TUI. You can interrupt with `Ctrl-C` at any time — the daemon preserves the staging directory under `runtime/interrupted/` for forensic review.

### Reflect phase

Press `r`. The reflect phase reads the work phase's output and writes structured updates to `memory.md` — distilled architectural decisions, learnings, and next-step suggestions. Reflect is **lossy** — it prunes redundancy and restructures.

Reflect may also **write dispatches** to `dispatches.yaml` if it identifies cross-cutting concerns affecting other plans.

### Compact phase

Press `c`. Compact is **lossless** — it rewrites `memory.md` to remove redundancy without dropping information. Triggered automatically when `memory.md` exceeds a word-count threshold against its baseline. Manually triggerable but rarely needed.

### Triage phase

Press `t`. Triage plans the next work cycle: updates backlog priorities, processes incoming `Received` items from other plans, identifies the next work starting point. Triage is the gate before the next work phase.

## Creating sub-plans

As a plan grows, you'll want to split off focused sub-plans to keep each plan's scope tight and avoid context bloat.

From the plan view, press `n` (or run `mnemosyne-tui new-plan` from the shell):

```
┌─ Create sub-plan ───────────────────────────────────────────┐
│ Parent: Mnemosyne/project-root/sub-F-hierarchy               │
│                                                              │
│ Name:        sub-F-datalog                                   │
│ Description: Datalog rules format + Erlog integration for   │
│              routing.ex evolution.                           │
│              (120 char limit; keyword-dense; noun-phrase)   │
│                                                              │
│ [Create] [Cancel]                                            │
└──────────────────────────────────────────────────────────────┘
```

Creating a sub-plan scaffolds the new plan directory with initial files and registers a new PlanActor in the daemon. The catalog regenerates automatically.

## Consulting experts

From a running work phase, your LLM session has an `ask_expert` tool available. The session invokes it when it wants a domain expert's opinion:

```elixir
ask_expert(
  target: "ffi-expert",
  question: "What's the correct lifetime for a Rust closure passed to a C callback that may outlive the enclosing function?"
)
```

The daemon:

1. Routes the query to the `ffi-expert` actor.
2. The expert's actor spawns its own fresh-context session with: the expert's persona, retrieved knowledge from its scope (10 top entries matching the question), and the question itself.
3. The expert session reasons and returns an answer.
4. The daemon delivers the answer back to the originating session as a tool-call result.
5. The originating session continues its work with the answer available in its context.

From the human side, you can also query experts directly from the TUI:

```
[q] query → pick target expert → type question → response opens in a pane
```

Multiple experts can be queried with the same question (parallel consultation) — each one responds independently with its own perspective.

## Dispatching cross-plan tasks

When a work or reflect phase identifies a concern that belongs in a different plan, it writes a dispatch to `dispatches.yaml`. After the phase exits, the daemon's dispatch processor routes it.

### Same-project dispatch

The LLM writes:

```yaml
dispatches:
  - target-plan: Mnemosyne/project-root/sub-A-global-store
    reason: Lock directory pinning question belongs to sub-A's scope.
    body: |
      Confirm the lock dir at <vault>/runtime/locks/ is the final
      pin so sub-D can proceed with its brainstorm.
```

The daemon appends to the target plan's `backlog.md` under `## Received` and to the origin's `backlog.md` under `## Dispatched`. No LLM intervention — same-project targeting is mechanical.

### Cross-project dispatch

The LLM writes:

```yaml
dispatches:
  - target-project: APIAnyware-MacOS
    reason: FFI callback GC-protection pattern is APIAnyware territory.
    body: |
      Investigate whether gc-protect wrappers need to be applied
      to all callback registration sites, not just the ones in the
      current crash reproducer.
```

The daemon spawns a **Level 2 routing agent** — a fresh-context Claude Code session with read access to APIAnyware-MacOS's vault subtree (including source code). The agent reads relevant plans and code, decides the routing, and either appends to a specific plan's `Received` section or writes a rejection back to the origin's `Dispatched` section.

### Dispatching from the TUI

You can also dispatch manually from the TUI with `d`:

```
[d] dispatch → pick target (plan / project / expert) → type body → confirm
```

This is the **human driver path** — the same dispatch mechanism, but initiated by you rather than an LLM phase.

## Editing routing rules

Routing rules live in `<vault>/routing.ex`, a user-editable Elixir module. You edit it in your editor of choice:

```elixir
defmodule Mnemosyne.UserRouting do
  def route(:query, facts) do
    cond do
      "rust" in facts or "cargo" in facts ->
        {:target_expert, "rust-expert"}

      "ffi" in facts or "callback_registration" in facts ->
        {:target_expert, "ffi-expert"}

      true ->
        :no_route
    end
  end

  def route(:dispatch, facts) do
    cond do
      "migration" in facts or "rename" in facts ->
        {:target_plan, "Mnemosyne/project-root/sub-G-migration"}

      true ->
        :no_route
    end
  end
end
```

Save the file. The daemon detects the change (or you run `mnemosyne rescan`), recompiles the module via BEAM's hot code reload, and the new rules take effect immediately — **no daemon restart needed**.

If the file has a compile error, the previous version stays loaded and the error is surfaced in the TUI.

### Rule suggestions

When the Level 2 routing agent runs (because rules didn't decide a case), it may propose a new rule. The TUI shows it as **"Rule suggestion pending review"** in a notification pane:

```
┌─ Rule suggestion pending ──────────────────────────────────────┐
│ Source: Level 2 routing agent (APIAnyware-MacOS context)        │
│                                                                  │
│ Proposed rule:                                                   │
│                                                                  │
│   def route(:dispatch, facts) do                                │
│     if "gc_protect" in facts and "callback_registration" in     │
│        facts do                                                  │
│       {:target_plan,                                             │
│        "APIAnyware-MacOS/project-root/sub-ffi-callbacks"}       │
│     else                                                         │
│       super(:dispatch, facts)                                    │
│     end                                                          │
│   end                                                            │
│                                                                  │
│ Context: Today's dispatch from sub-F matched this target cleanly │
│                                                                  │
│ [a] accept (append to routing.ex)  [e] edit  [r] reject         │
└──────────────────────────────────────────────────────────────────┘
```

Accepting appends the rule to `routing.ex` and commits the change to vault git. Over time, the deterministic routing path grows as you accept suggestions — novel cases teach the system.

## Obsidian browsing

Open the vault in Obsidian:

```bash
obsidian ~/Mnemosyne-vault/
```

You'll see:

- **`plan-catalog.md`** — the auto-generated list of every plan and expert, with descriptions. Your at-a-glance "what's in my vault."
- **`projects/` directory** — symlinks to every adopted project's mnemosyne/ directory. Browse plans, read memories, follow wikilinks between them.
- **`knowledge/` directory** — Tier 2 global knowledge entries, organized by axis.
- **`experts/` directory** — expert declaration files (one per expert).
- **`routing.ex`** — the routing rules, visible as source code.

Obsidian's built-in features work naturally:

- **Dataview** queries across frontmatter give you dashboards: all plans in reflect phase, all knowledge tagged with `ffi`, all dispatches to APIAnyware-MacOS this week.
- **Graph view** visualizes the relationships between plans, knowledge entries, and experts via wikilinks.
- **File history** shows every change to every file, so you can audit and roll back.

Obsidian is **the primary explorer and curation UI** for v1. The Rust TUI is the driver for work cycles; Obsidian is the browser for everything else.

## Multi-machine sync

The vault is a git repository. To move to a new machine:

```bash
# On machine A:
$ cd ~/Mnemosyne-vault
$ git push origin main

# On machine B:
$ git clone git@github.com:you/mnemosyne-vault.git ~/Mnemosyne-vault
$ mnemosyne daemon --init-from-existing
Existing vault detected at /Users/you/Mnemosyne-vault/
  • Rebuilding project symlinks (may need manual path adjustments)
  • Regenerating plan-catalog.md
  • Starting daemon
```

Project symlinks are **not in git** (absolute paths are per-machine). The daemon rebuilds them on first startup, asking you to confirm each project's local path.

**True multi-user team mode** (multiple developers sharing a vault with cross-daemon dispatch) is a v2 feature. For v1, multi-machine sync is single-user-across-machines.

## Troubleshooting

### Daemon won't start

```
Error: another daemon is already running (pid 48291)
```

Check with `mnemosyne status`. If the other daemon is stale, `mnemosyne stop --force` will release the singleton lock.

### Daemon refuses to start due to routing.ex compile error

```
Error: routing.ex has compile errors — daemon refuses to start
  line 14: syntax error before: {
```

Edit `routing.ex` to fix the error, then start again. The daemon **refuses to start** with a broken routing module — this is deliberate to prevent silent routing failures.

### Plan description too long

```
Error: plan description exceeds 120 character cap
  plan: Mnemosyne/project-root/sub-F-hierarchy
  description: (127 chars) ...
```

Edit the plan's `plan-state.md` frontmatter, shorten the description, save. The daemon will pick up the change on next rescan.

### Vault catalog out of sync

```bash
$ mnemosyne rescan
```

This regenerates `plan-catalog.md`, reloads routing.ex, re-verifies vault invariants, and emits a summary of any problems.

### Harness session hangs or crashes

The actor transitions to Faulted. The TUI shows it in red. Investigate via `mnemosyne logs` and retry via the TUI's retry command (`R` in the faulted plan view).

---

## Further reading

- [Architecture overview](architecture.md) — the full architectural picture
- [Configuration reference](configuration.md) — `daemon.toml`, `mnemosyne.toml`, and related config files
- [Knowledge format](knowledge-format.md) — how knowledge entries are structured
- [Research sources](research-sources.md) — the cognitive science grounding
