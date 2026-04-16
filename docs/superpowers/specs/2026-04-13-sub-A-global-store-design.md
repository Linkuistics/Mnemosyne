# Sub-project A — Vault location, discovery, and bootstrap

**Status:** design complete (Session 14 amendment absorbed inline)
**Brainstorm date:** 2026-04-13
**Amendment date:** 2026-04-15 (Session 14 — BEAM daemon + sub-F commitments)
**Parent plan:** `LLM_STATE/mnemosyne-orchestrator/`
**Sibling implementation plan:** `LLM_STATE/sub-A-global-store/`

> **Amendment status:** rewritten inline 2026-04-15 (Session 14) to
> absorb the persistent BEAM daemon commitment (Session 9, sub-F) and
> the `project-root/` + vault-catalog + routing module commitments that
> sub-F locked in its design doc. The original Rust CLI framing is
> replaced throughout — no supersede-amendment layer. The Session-7
> forking decisions are preserved verbatim in **Appendix A (Decision
> Trail)** with inline correction notes; the new pivots are recorded as
> Q6 (BEAM pivot) and Q7 (sub-F architectural commitments). Every A1–A10
> header is kept stable so sibling plan references remain resolvable.

## Purpose

Replace Mnemosyne v0.1.0's hardcoded `~/.mnemosyne/` knowledge store
with an explicit, user-chosen, git-tracked **vault** that hosts Tier 2
global knowledge, runtime state (daemon socket, mailboxes, staging,
interrupts, ingestion events), the shipped Obsidian template, sub-F's
user-editable routing module, sub-F's auto-regenerated plan catalog,
sub-F's expert declaration files, daemon configuration, and the
per-project symlink mounts. Define how the **Mnemosyne daemon**
(persistent Elixir/OTP application) discovers the vault at startup, how
it verifies a directory really is a vault, how it bootstraps a fresh
one (or clones an existing one), how Tier 1 and Tier 2 knowledge roots
are exposed for PlanActors and ExpertActors at runtime, and which
contracts the rest of the orchestrator's sub-projects can rely on.

## Scope

In scope:

- Vault directory layout, including every F-committed surface
  (`routing.ex`, `daemon.toml`, `plan-catalog.md`, `experts/`,
  `runtime/daemon.sock`, `runtime/mailboxes/`), plus the B/E
  surfaces (`runtime/staging/`, `runtime/events/`,
  `runtime/interrupts/`). D's singleton lock is system-wide, not in
  the vault.
- What is git-tracked vs gitignored, and why.
- Vault discovery mechanism (precedence chain, user config schema),
  resolved **once per daemon start**, not per LLM phase invocation.
- Vault identity verification (the `mnemosyne.toml` marker).
- The `mnemosyne init` daemon subcommand (fresh and clone variants).
- The `mnemosyne adopt-project` daemon subcommand.
- Tier 1 / Tier 2 root addressability via env-var overrides for test
  harnesses and fixture runs.
- The cross-cutting contracts A locks for B, C, D, E, F, G, I, M.

Out of scope (explicit non-goals listed in detail at the end):

- Migration from `~/.mnemosyne/` (no existing usage; deletion not
  transition).
- Multi-vault registry / `mnemosyne vault list`.
- Automatic multi-machine sync.
- Tier 2 axis structure (deferred to a future knowledge-format
  brainstorm).
- Team-mode multi-developer use (sub-P scope; v2+).
- The contents of sub-F's `routing.ex`, `plan-catalog.md` regenerator,
  and `experts/<id>.md` files (all owned by sub-F and sub-N).
- No opinionated Obsidian template refinement (sub-I's scope).
- No daemon lifecycle, supervision, or actor wiring beyond "vault
  resolution happens inside the application boot hook" (sub-F owns the
  daemon shape; sub-B owns the phase runner that actors embed).

## Constraints absorbed from earlier brainstorms

Non-negotiable inputs from sub-projects B, C, E, F, M:

- **Persistent BEAM daemon** (F, Session 9): Mnemosyne v1 runs as a
  single long-lived Elixir/OTP application. Vault resolution and
  verification happen exactly once per daemon start (plus on explicit
  `mnemosyne rescan`). Internal harness-spawned sessions (fact
  extraction, Level 2 routing, ingestion) inherit the daemon's resolved
  vault path via a supervised GenServer; they do **not** re-run the
  discovery chain.
- **Dedicated vault with symlinked per-project directories** (B):
  `<dev-root>/Mnemosyne-vault/` (user-chosen absolute path, default
  `~/Mnemosyne-vault/`). The vault accesses each adopted project via
  `<vault>/projects/<project-name>/` symlinking to
  `<project>/mnemosyne/`. `<project>/mnemosyne/` is the per-project
  container holding `project-root/` (F's reserved root-plan directory)
  and `knowledge/`.
- **Symlink validation passed 6/6** (B, Session 5) on macOS Tahoe and
  Ubuntu 24.04 (commit `98ef7db`). The vault-as-view-over-symlinks
  framing stands; the hard-copy fallback is not needed.
- **`project-root/` replaces `plans/`** (F, Session 9): every adopted
  project has exactly one root plan at
  `<project>/mnemosyne/project-root/`. The earlier `plans/` container
  is collapsed away — it was always single-child. `knowledge/` stays
  as a sibling of `project-root/`, not inside it.
- **Path-based qualified plan IDs** (F): a plan's qualified ID is a
  pure function of its filesystem path —
  `strip_prefix(plan_path, "<vault>/projects/")`. Never stored in
  frontmatter. A's walk-up logic for Tier 1 resolution must respect
  this invariant.
- **User-editable routing module** (F): `<vault>/routing.ex` is an
  Elixir module with pattern-matched `defp route/2` clauses, hot-code
  reloaded by the daemon on file change. A ships it tracked in git;
  init scaffolds a minimal stub.
- **Vault catalog** (F): `<vault>/plan-catalog.md` is auto-regenerated
  on plan mutation and every phase-prompt render. Machine-owned. A
  ships it tracked in git for Obsidian visibility but init leaves it
  empty — the daemon writes it on first start.
- **Expert declarations** (F, sub-N): `<vault>/experts/<expert-id>.md`
  holds per-expert persona + knowledge scope + retrieval strategy. A
  ships `experts/` tracked in git but init creates it empty. Default
  expert declaration files are scaffolded by sub-N's brainstorm, not A.
- **Obsidian-native format discipline** (B): Dataview-friendly
  kebab-case YAML frontmatter, wikilinks, tags, a Mnemosyne-shipped
  `.obsidian/` template with Dataview required and Templater optional.
- **Self-containment via embedded prompts** (B, Session 12): the
  daemon ships with `fixed-memory/`, `phases/`, and `create-plan.md`
  embedded via Elixir's `@external_resource` + `File.read!/1` compile-
  time read pattern. The same self-containment discipline applies to
  A's `.obsidian/` template files — they are embedded into the daemon
  release at compile time from `priv/obsidian/`, and materialised on
  init. **No runtime dependency on `LLM_CONTEXT/`**.
- **Pipes-only `erlexec` adapter** (C, Session 11): internal reasoning
  sessions spawned by the daemon (fact extraction, Level 2 routing,
  ingestion) use C's `Mnemosyne.HarnessAdapter.ClaudeCode` GenServer.
  Those sessions inherit the parent daemon's resolved vault and do not
  re-resolve — sub-A's discovery chain runs exactly once at daemon boot.
- **Tier 1 and Tier 2 roots independently addressable as daemon
  startup config** (E): both must be resolvable at daemon boot and
  overridable for tests pointing at fixture directories via env vars.
- **Hard errors by default** (project-wide): unexpected conditions,
  missing files, schema mismatches all fail loudly with a clear
  diagnostic naming the offending path. No silent fallbacks. Any
  exception requires explicit written rationale.
- **D's scope collapsed by F** (Session 9): per-plan advisory locks are
  replaced by OTP mailbox serialization inside PlanActor GenServers.
  D's remaining scope is a **system-wide daemon singleton lock** at
  `<runtime_dir>/mnemosyne/daemon.lock` via `flock(2)` (not in the
  vault — invisible to Obsidian), plus advisory coordination for
  external-tool (Obsidian, git) concurrent writes.
- **Cross-cutting brainstorms own their own sibling adoption stubs**
  (M): A's sibling implementation plan must include observability
  adoption stubs at the boundaries A introduces, re-cast from Rust
  `tracing` + enum events to Elixir `:telemetry` + typed
  `Mnemosyne.Event.*` structs.

## Architectural decisions

### A1. Vault discovery is explicit; no walk-up; no implicit dev-root; resolved once at daemon boot

The `mnemosyne` binary is **one command with subcommands**. The
subcommand `mnemosyne daemon` starts the persistent Elixir/OTP
application; the subcommands `mnemosyne init`, `mnemosyne init --from`,
`mnemosyne config use-vault`, and `mnemosyne adopt-project` are
short-lived bootstrap utilities that run outside the daemon.

Vault discovery runs in two distinct contexts:

1. **Daemon boot** (`mnemosyne daemon`): the application supervisor's
   `init/1` callback resolves the active vault via the precedence
   chain, verifies it, and stashes the resolved absolute path in a
   `Mnemosyne.Vault` GenServer. Every actor (PlanActor, ExpertActor,
   harness adapter, routing engine, ingestion pipeline) reads the vault
   path from this single source. **No actor re-runs the chain.**

2. **Bootstrap subcommands** (`init`, `adopt-project`, `config
   use-vault`, `init --from`): these are `Mnemosyne.CLI` escript-style
   entry points that run without starting the full application tree.
   They resolve the chain (or create rather than consume, in the case
   of `init`) and exit. When a user runs `mnemosyne adopt-project
   <path>` while a daemon is already running, the bootstrap subcommand
   also sends a `:rescan` message to the running daemon over
   `<vault>/runtime/daemon.sock` so the live actor tree picks up the
   new symlink without restart.

The precedence chain is unchanged from the brainstorm:

1. `--vault <path>` CLI flag — wins always; per-invocation override.
2. `MNEMOSYNE_VAULT` env var — set by shell init or test harness.
3. User config file:
   - Linux: `~/.config/mnemosyne/config.toml`
   - macOS: `~/Library/Application Support/mnemosyne/config.toml`
   - Windows: `%APPDATA%\mnemosyne\config.toml`
   - resolved via Elixir's `:filename.basedir(:user_config, "mnemosyne")`
     (the BEAM-idiomatic equivalent of the Rust `dirs::config_dir()`
     the brainstorm originally used).
4. None of the above → hard error with actionable message.

There is no walk-up search from `cwd`. There is no implicit dev-root
concept. The single-vault-per-machine model remains a deliberate
simplification, and the rationale only strengthens under the daemon
framing:

- **Composes cleanly with Mnemosyne-as-LLM-client** (C's internal
  session spawns): internal harness sessions inherit the daemon's
  resolved vault via the GenServer registry; they do not perform their
  own `cwd` dance or re-resolve from env.
- **Test isolation is trivial**: `MNEMOSYNE_VAULT=$tmpdir mix test` for
  ExUnit integration tests, same knob the brainstorm committed to.
- **Consistent with B's "no implicit ambient state" posture**.
- **Bypasses the ancestor `Mnemosyne-vault/` footgun** that walk-up
  discovery would introduce.
- **Plays with BEAM distribution** (reserved for sub-P, v2+): a peer
  daemon addresses a remote vault through its qualified peer prefix,
  not by inheriting the local discovery chain. A's single-vault-per-
  machine constraint has no negative interaction with future
  multi-daemon transport.

A user who genuinely wants multiple vaults on one machine switches via
env var or `--vault` per bootstrap subcommand, or runs a second daemon
pointed at a different vault via `MNEMOSYNE_VAULT=<other> mnemosyne
daemon` (two daemons cannot run against the same vault because D's
singleton lock forbids it).

### A2. User config schema (top-level flat)

```toml
# ~/.config/mnemosyne/config.toml
vault = "/Users/antony/Mnemosyne-vault"
```

Single key for v1. Future user-level preferences would land under
`[preferences]` or similar; the top-level `vault` key is the stable
contract. Schema is unchanged by the BEAM pivot — the file is TOML
regardless of the daemon's implementation language.

### A3. Vault identity = `<vault>/mnemosyne.toml` schema-versioned marker

A directory is a Mnemosyne vault if and only if it contains a
`mnemosyne.toml` file at its root with a parseable `[vault]` table
containing `schema_version`. The marker file is git-tracked and
doubles as the host for optional vault-level configuration overrides.

```toml
# Mnemosyne vault — created by `mnemosyne init`
[vault]
schema_version = 1
created = "2026-04-13T14:22:00Z"
created_by_version = "0.2.0"

# Optional: override embedded defaults. Omit any section to keep defaults.
# [language_profiles.rust]
# dependency_file = "Cargo.toml"
# dependency_parser = "cargo"
```

Verification rules (each is a hard error, never a silent fallback):

- File missing → `not a Mnemosyne vault (missing mnemosyne.toml): <path>`
- File parse error → `invalid mnemosyne.toml: <path>: <parse error>`
- `schema_version` higher than the daemon's
  `@vault_schema_version` module attribute →
  `vault schema_version=N but this daemon supports M; upgrade Mnemosyne`

The marker table is parsed by Elixir's `Toml` library (hex dep
`{:toml, "~> 0.7"}`). Parse failures surface as
`%Mnemosyne.Vault.MarkerError{}` structs and log via the sub-M
`:telemetry` event pipeline before the daemon exits with a non-zero
status.

The marker serves three audiences:

1. The **Mnemosyne daemon** (identity check at boot and on `rescan`).
2. **Obsidian users** browsing the vault in their file tree ("what is
   this directory?").
3. **Forward-compat tooling**: `schema_version` is the gate for future
   vault-format migrations.

The marker and the override config share one file because their key
spaces never overlap. The "marker + config in one TOML file" pattern
mirrors `Cargo.toml`'s "project marker + config", which happens to also
echo `mix.exs`'s "project manifest + config" — users coming from either
ecosystem find the shape familiar.

**`daemon.toml` is a separate file** (per sub-F). It holds daemon-level
runtime knobs (`[harnesses.*]` for sub-O, `[peers]` for sub-P, fact
extraction model config, log levels). The two-file split is deliberate:
`mnemosyne.toml` is the **identity + vault-format override** host;
`daemon.toml` is the **runtime configuration** host. Swapping a daemon
version or reconfiguring `[harnesses.*]` must not touch the vault's
identity marker.

### A4. Vault directory layout

```
<vault>/
├── mnemosyne.toml                    # marker + optional vault-format overrides — tracked
├── daemon.toml                       # sub-F daemon config (harnesses, peers, log levels) — tracked
├── routing.ex                        # sub-F user-editable routing rules — tracked
├── plan-catalog.md                   # sub-F auto-regenerated vault catalog — tracked (machine-owned)
├── .git/                             # vault's own git
├── .gitignore                        # tracked
├── .obsidian/                        # curated subset tracked
│   ├── community-plugins.json        # tracked (Dataview required, Templater optional)
│   ├── core-plugins.json             # tracked
│   ├── app.json                      # tracked (canonical settings)
│   ├── snippets/                     # tracked (CSS for Mnemosyne views)
│   ├── plugins/dataview/data.json    # tracked
│   ├── workspace.json                # gitignored (per-machine pane state)
│   ├── workspace-mobile.json         # gitignored
│   └── cache                         # gitignored
├── knowledge/                        # Tier 2 global knowledge — tracked
│   └── (axes empty at init; accreted by future curation)
├── experts/                          # sub-F expert declarations — tracked
│   └── (empty at init; scaffolded by sub-N default expert set)
├── archive/                          # tracked (auditable history)
├── runtime/                          # all gitignored, machine-local, ephemeral
│   ├── daemon.sock                   # sub-F Unix socket for TUI/Obsidian/web clients
│   ├── staging/<plan-path>/          # sub-B phase staging dirs (incl. materialised prompts/)
│   ├── mailboxes/<qualified-id>.jsonl # sub-F actor mailboxes
│   ├── interrupts/<qualified-id>/    # pending interrupt commands for live sessions
│   └── events/                       # sub-E ingestion event queue
├── cache/                            # gitignored, derived data (indexes, parsed entries)
└── projects/                         # gitignored, machine-local symlink targets
    ├── apianyware-macos -> /Users/antony/Development/APIAnyware-MacOS/mnemosyne/
    ├── guivisionvmdriver -> /Users/antony/Development/GUIVisionVMDriver/mnemosyne/
    └── mnemosyne -> /Users/antony/Development/Mnemosyne/mnemosyne/
```

Changes from the Session-7 layout (all driven by sub-F or sub-C/D/E
commitments after Session 9):

- **Added at vault root (tracked)**: `daemon.toml`, `routing.ex`,
  `plan-catalog.md`, `experts/`. All four are owned by sub-F but A
  scaffolds them at init time so a fresh vault is immediately
  openable in Obsidian with the expected tree shape.
- **`runtime/daemon.sock` added**: the Unix socket sub-F's Rust TUI
  client (and any future Obsidian/web client) attaches to. Bound by
  the daemon at boot; the inode lives under `runtime/` so it composes
  with the vault's gitignored-runtime discipline.
- **`runtime/daemon.lock` and `runtime/daemon.pid` removed**:
  sub-D's singleton lock moved to a **system-wide** location at
  `<runtime_dir>/mnemosyne/daemon.lock` (outside the vault) so it is
  invisible to Obsidian. The lock file contains both the daemon's PID
  and the vault path it serves, eliminating the need for a separate
  `daemon.pid`. Bootstrap subcommands read the system-wide lock file
  to determine whether a daemon is running for their target vault.
  See sub-D's design doc for details.
- **`runtime/mailboxes/<qualified-id>.jsonl` added**: sub-F's actor
  mailboxes. One JSONL file per plan or expert, keyed by qualified ID
  (which is a filesystem-path-derived value per F). Cursor files for
  mailbox reads live alongside. Per-actor mailbox concurrency is
  serialized by the owning GenServer — no file-lock needed.
- **`runtime/interrupts/<qualified-id>/` rekeyed**: the per-plan
  interrupt directory is now keyed on the sub-F qualified ID
  (path-derived) rather than a separate plan-id field. Consistent with
  "filesystem-derivable data is never cached in metadata."
- **`<project>/mnemosyne/plans/` collapses to `<project>/mnemosyne/project-root/`**:
  no change to A's symlink structure (which still points at the
  `mnemosyne/` container), but a change to what lives inside each
  mount target. The symlink rule is unchanged; the target contents
  are F's domain.

Notes:

- **Prompts do not live in the vault.** Per sub-B (rewritten Session 12),
  prompts are compile-time-embedded into the daemon release via
  `@external_resource` + `File.read!/1` from `priv/prompts/` and
  materialised at `<staging>/prompts/` per phase render. Same
  self-containment story applies to A's `.obsidian/` template files
  (embedded from `priv/obsidian/`).
- **`init` scaffolds an empty `knowledge/`, an empty `experts/`, and a
  minimal `routing.ex` stub.** No Tier 2 axes pre-committed. No default
  expert declarations (sub-N's scope). The minimal `routing.ex` stub
  is:
  ```elixir
  defmodule Mnemosyne.UserRouting do
    @moduledoc "User-editable routing rules. Safe to edit; hot-reloaded."
    def route(_kind, _facts), do: :no_route
  end
  ```
  Compiles cleanly, hot-reloads cleanly, fires zero rules — the
  Level 2 fallback handles everything until the user adds a clause.
- **`plan-catalog.md` is scaffolded with a machine-owned header** but
  zero entries. The daemon writes the full catalog on first boot after
  it has an actor tree to enumerate. A's init leaves just:
  ```markdown
  # Mnemosyne Vault Catalog

  > Machine-owned. Regenerated automatically on plan mutation and
  > every phase-prompt render. Manual edits are overwritten.
  ```
- **There is no `<vault>/config.yml`.** v0.1.0's YAML config file is
  deleted; A's `mnemosyne.toml` + sub-F's `daemon.toml` (both TOML)
  replace it. v0.1.0's `Config::load(dir)` site in `src/config.rs`
  becomes dead code along with the rest of the Rust CLI; deletion
  tracked in sub-G's migration scope.

### A5. Git-tracking policy

| Subtree | Tracked | Rationale |
|---|---|---|
| `mnemosyne.toml` | yes | Vault identity + optional vault-format overrides |
| `daemon.toml` | yes | Daemon runtime config is user-authored, machine-portable |
| `routing.ex` | yes | User-authored routing rules — the whole point of user-visible routing |
| `plan-catalog.md` | yes | Machine-owned but committed so Obsidian shows it in file tree + git history tracks catalog evolution |
| `experts/*.md` | yes | Expert declarations are authored config and shared across machines |
| `.gitignore` | yes | Bootstrapping |
| `.obsidian/` curated subset | yes | The shared template is the whole point of Obsidian-native discipline |
| `.obsidian/workspace*.json`, `.obsidian/cache` | no | Per-machine pane state, churns constantly |
| `knowledge/` | yes | Tier 2 global knowledge — the asset users want shared across machines |
| `archive/` | yes | Auditable history per v0.1.0 design |
| `runtime/daemon.sock` | no | Unix socket inode; machine-local |
| `runtime/staging/` | no | Phase-boundary scratch |
| `runtime/mailboxes/` | no | Actor mailboxes; daemon-owned; rebuildable from archive |
| `runtime/interrupts/` | no | Pending interrupts; ephemeral |
| `runtime/events/` | no | Ingestion event queue |
| `cache/` | no | Derived data, rebuildable |
| `projects/` symlinks | no | Targets are machine-local absolute paths; including in git is a footgun |

Rationale for making `plan-catalog.md` tracked even though it is
machine-owned: Obsidian's file tree and backlinks index read from git
state, and Dataview queries against the catalog are far more useful
when the history of catalog entries is visible across commits. The
machine-owned header tells humans not to edit; the regeneration
discipline is hard-errors-by-default (any manual edit between
regenerations is silently overwritten on the next write, and that is
documented in the catalog header).

`mailboxes/` is **deliberately gitignored** even though it contains
durable state — it is rebuildable from `archive/` on daemon boot, and
including it in git would generate a churn storm. Sub-F's recovery
discipline is "replay archive into fresh mailboxes on start"; A just
makes sure the directory exists.

This policy has the property that the vault remains a clean
push/pullable git repo from the user's perspective: `git push` ships
knowledge, archive, expert declarations, routing rules, daemon config,
catalog, vault-format identity, and the shared Obsidian template. It
does not ship runtime state, the socket, the lock, machine-local
symlink targets, or workspace pane state. The gitignore file at vault
root encodes the policy.

### A6. `mnemosyne init <path>` — fresh vault

Short-lived bootstrap subcommand; runs as an escript-style entry point
outside the daemon's OTP application tree. Path argument is
**optional**, defaulting to `~/Mnemosyne-vault/`. The UX gain of a
default outweighs the deliberate-placement argument; users who want a
different location pass it explicitly.

Step-by-step:

1. **Resolve target.** If `<path>` omitted, default to
   `Path.join(System.user_home!(), "Mnemosyne-vault")`. Otherwise
   make `<path>` absolute via `Path.expand/1`. Hard error if
   `System.user_home/0` returns `nil`.
2. **Pre-flight checks**, each hard-erroring on failure with the
   offending path in the message:
   - `<path>` does not already exist (file or directory) — refuse to
     overwrite anything.
   - Parent directory of `<path>` exists — refuse to silently create
     ancestors.
   - `git` is on PATH (`System.find_executable("git")` returns a
     string) — bootstrap must work or fail loudly.
3. **Create directory tree** with `File.mkdir_p!/1`:
   ```
   <path>/
   <path>/knowledge/
   <path>/experts/
   <path>/archive/
   <path>/runtime/staging/
   <path>/runtime/mailboxes/
   <path>/runtime/interrupts/
   <path>/runtime/events/
   <path>/cache/
   <path>/projects/
   <path>/.obsidian/
   <path>/.obsidian/snippets/
   <path>/.obsidian/plugins/dataview/
   ```
   `runtime/` itself is created but `daemon.sock` is **not** — the
   running daemon creates it on boot. Init creates only the containing
   directories. (The singleton lock is system-wide, not in the vault.)
4. **Materialise the embedded `.obsidian/` template.** Each template
   file is embedded into the daemon release at compile time via
   `@external_resource` + `File.read!/1` from `priv/obsidian/`. Init
   writes each verbatim to its target. Same self-containment pattern
   sub-B uses for phase prompts and `fixed-memory/` after the
   Session-12 rewrite.
5. **Write `mnemosyne.toml`** with the marker table:
   ```toml
   [vault]
   schema_version = 1
   created = "<RFC3339 UTC via DateTime.utc_now() |> DateTime.to_iso8601()>"
   created_by_version = "<Mix.Project.config()[:version] embedded at compile time>"
   ```
6. **Write minimal `daemon.toml`** with the stable-schema stub:
   ```toml
   [daemon]
   log_level = "info"

   [harnesses.claude_code]
   # v1 default adapter; sub-O adds more in v1.5+
   enabled = true
   ```
   Sub-F owns this file's full schema. A ships just enough to make
   the daemon boot cleanly against a fresh vault.
7. **Write minimal `routing.ex`** with the no-route stub shown in A4.
8. **Write empty `plan-catalog.md`** with the machine-owned header
   shown in A4.
9. **Write `.gitignore`** with the tracked-vs-gitignored policy from
   A5:
   ```
   /runtime/
   /cache/
   /projects/
   /.obsidian/workspace.json
   /.obsidian/workspace-mobile.json
   /.obsidian/cache
   ```
10. **Initialise git, stage, commit.** Three sequential `git` calls
    via `System.cmd/3`: `init`, `add .` (safe here because the vault
    is brand-new and contains only files we just created), `commit -m
    "Initialize Mnemosyne vault"`. Each call hard-errors if it fails
    — no v0.1.0-style best-effort silent ignore.
11. **Update user config.** Atomic write of
    `~/.config/mnemosyne/config.toml` (or platform equivalent) with
    `vault = "<absolute-path>"`. If a user config already exists
    pointing elsewhere, do **not** overwrite — print a notice instead:
    > Vault created at `<path>`. Your user config still points at
    > `<other-vault>`. Run `mnemosyne config use-vault <path>` to
    > switch, or set `MNEMOSYNE_VAULT=<path>`.
12. **Print success** with next-step hints:
    ```
    ✓ Mnemosyne vault initialised at /Users/antony/Mnemosyne-vault
    ✓ User config updated to point at the new vault
    Next steps:
      mnemosyne daemon                                  # start the persistent daemon
      mnemosyne adopt-project <path-to-project>         # add a project to the vault
      open /Users/antony/Mnemosyne-vault                # open the vault in Obsidian
    ```

On any step-N failure after step 3 (partially populated vault), the
directory is **left in place** for the user to inspect. No destructive
cleanup on our own errors. The vault is unusable until init completes
successfully — the missing `mnemosyne.toml` ensures the daemon's
`verify_vault` call at boot will refuse to recognise it.

### A7. `mnemosyne init --from <git-url> <path>` — clone existing vault

Bootstrap subcommand for cloning a vault from a remote. Path argument
is optional, defaulting to `~/Mnemosyne-vault/` (overriding `git
clone`'s URL-basename habit for consistency).

Step-by-step:

1. **Resolve target** as in A6.
2. **Pre-flight checks**: `<path>` doesn't exist; parent exists; `git`
   on PATH; URL is non-empty.
3. **`git clone <url> <path>`**. Failure is a hard error with git's
   stderr in the message.
4. **Verify the clone is a Mnemosyne vault** by calling
   `verify_vault(<path>)`. Three failure modes: marker missing
   ("cloned repository is not a Mnemosyne vault"), parse error,
   schema_version higher than this daemon supports. **All three
   trigger a cleanup** — `File.rm_rf!(<path>)` — because the clone
   created the directory and we don't want to leave garbage behind.
   (Distinct from fresh init's "leave it for inspection" because in
   the clone case the directory was created entirely by us, not
   partially populated by us.)
5. **Create the gitignored runtime tree**:
   `runtime/{staging,mailboxes,interrupts,events}/`, `cache/`,
   `projects/`. These are not in the cloned repo (they're gitignored)
   but the rest of Mnemosyne expects them to exist. Same
   `File.mkdir_p!` block as fresh init step 3 minus the directories
   that came in via the clone.
6. **Do not create `daemon.sock`** — it belongs to the running daemon,
   not the clone. (The singleton lock is system-wide, not in the vault.)
7. **Update user config** as in A6 step 11.
8. **Print success** with the same next-step hints, including a note
   that the user must re-run `mnemosyne adopt-project` once per
   project on this machine (the symlinks are gitignored and live in
   `projects/`).

### A8. `mnemosyne config use-vault <path>` — switch vault

Writes/updates the user config file to point at a different vault.
Verifies the target is a valid vault first (calls `verify_vault`),
creates the parent directory if missing, writes the TOML atomically
(write-temp + rename). Single-key replacement; preserves any future
`[preferences]` sections under it.

If a daemon is already running against the **old** vault at the time
`config use-vault` is invoked, the switch **does not affect the live
daemon** — the old daemon continues to serve the old vault until
stopped and restarted. `config use-vault` only changes what the **next**
daemon start resolves.

### A9. `mnemosyne adopt-project <project-path>` — mount a project

Per-machine setup that compensates for the gitignored `projects/`
directory.

1. Verify `<project-path>/mnemosyne/` exists. Hard error otherwise:
   "not a Mnemosyne-managed project; the project must have a
   `mnemosyne/` subdirectory before it can be adopted."
2. Verify `<project-path>/mnemosyne/project-root/plan-state.md`
   exists. Hard error otherwise: "project has a `mnemosyne/` directory
   but no root plan; run `mnemosyne daemon --init-project
   <project-path>` first." (Sub-F owns the `--init-project` subcommand
   that creates the initial `project-root/` with `plan-state.md`. A
   just checks the invariant.)
3. Compute the symlink name as the lowercase basename of the project
   directory (e.g., `APIAnyware-MacOS` → `apianyware-macos`).
4. Resolve the active vault via the precedence chain.
5. Create symlink `<vault>/projects/<lowercase-name>/ -> <abs-project-path>/mnemosyne/`.
   Hard error if a symlink (or any file) with that name already
   exists.
6. If a daemon is running against the active vault (detected by reading
   the system-wide lock file at `<runtime_dir>/mnemosyne/daemon.lock`
   and checking whether the vault path matches), send a `:rescan`
   NDJSON message over `runtime/daemon.sock`. If no daemon is serving
   this vault, skip the notification.
7. Print confirmation.

Symlink targets are **absolute paths** so the link survives the user
moving cwd, but breaks if they move the project. That is acceptable: a
moved project should be re-adopted explicitly.

On a fresh clone of a vault from another machine, the user re-runs
`adopt-project` once per project. Sub-G's migration story produces a
one-shot script that loops over a list, but that is G's scope.

### A10. Tier 1 / Tier 2 root resolution

Defaults are derived from the resolved vault path and the active plan
path:

| Root | Default | Source |
|---|---|---|
| **Tier 2** (global) | `<vault>/knowledge/` | derived from resolved vault |
| **Tier 1** (per-project) | `<active-project>/mnemosyne/knowledge/` | walked up from active plan path |

Test override knobs (env vars only — not exposed as CLI flags to keep
the surface tight):

```
MNEMOSYNE_TIER1_ROOT=/tmp/fixture/tier1
MNEMOSYNE_TIER2_ROOT=/tmp/fixture/tier2
```

When set and non-empty, each variable bypasses the corresponding
default derivation. Set independently. The vault discovery chain is
still honoured for everything else (`runtime/`, `archive/`,
`.obsidian/`, `routing.ex`, `experts/`); only the two knowledge roots
are decoupled.

Resolution function (Elixir):

```elixir
defmodule Mnemosyne.Vault.Roots do
  @enforce_keys [:vault, :tier2]
  defstruct [:vault, :tier2, :tier1]

  @type t :: %__MODULE__{
          vault: Path.t(),
          tier2: Path.t(),
          tier1: Path.t() | nil
        }

  @spec resolve(Path.t(), Path.t() | nil) :: {:ok, t()} | {:error, term()}
  def resolve(vault, active_plan) do
    tier2 =
      env_override("MNEMOSYNE_TIER2_ROOT") ||
        Path.join(vault, "knowledge")

    tier1 =
      cond do
        override = env_override("MNEMOSYNE_TIER1_ROOT") ->
          override

        is_binary(active_plan) ->
          derive_tier1_from_plan(active_plan)

        true ->
          nil
      end

    with :ok <- verify_directory_exists(tier2, "Tier 2 knowledge root"),
         :ok <- maybe_verify(tier1, "Tier 1 knowledge root") do
      {:ok, %__MODULE__{vault: vault, tier2: tier2, tier1: tier1}}
    end
  end

  defp env_override(key) do
    case System.get_env(key) do
      nil -> nil
      "" -> nil
      value -> value
    end
  end

  defp maybe_verify(nil, _label), do: :ok
  defp maybe_verify(path, label), do: verify_directory_exists(path, label)
end
```

`derive_tier1_from_plan/1` walks up the plan path looking for the
topmost `mnemosyne/` ancestor that contains a `project-root/` sibling
(i.e., the ancestor sits at `<project>/mnemosyne/project-root/...`),
then returns `<that-mnemosyne-dir>/knowledge/`. If the walk fails
(plan path malformed or not under a `mnemosyne/project-root/`
ancestor), hard error with the offending path in the message.

**Note the rename from the Session-7 walk-up rule**: the target
sibling is now `project-root/`, not `plans/`. Sub-F's invariant #3 is
the basis ("Every adopted project has exactly one `project-root/`
directory as a direct child of `<project>/mnemosyne/`") and invariant
#4 is the safeguard ("No plan directory at any depth is named
`project-root` except the single reserved root"). Together they
guarantee the walk-up terminates at the correct ancestor and never
false-matches a nested plan named `project-root`.

Optionality semantics:

- **Tier 2 is required** for any daemon action that touches global
  knowledge (expert queries, ingestion Stage 5 dispatch, Level 2
  routing agent, curation). Hard error if `<vault>/knowledge/` doesn't
  exist at daemon boot.
- **Tier 1 is optional.** `tier1: nil` means "no active plan path in
  scope, no per-project knowledge to serve." PlanActor phase runners
  resolve Tier 1 at phase-boundary time (they know their own plan
  path); global daemon actions (Tier 2 curation, routing rule
  compilation, catalog regeneration) do not need Tier 1.

A's contract stops at "here's how to resolve the two roots given a
vault and an optional active plan." The *content* of Tier 1 / Tier 2
(axes, schema, frontmatter conventions) is not A's scope — it is owned
by E (ingestion routing rules) and a future knowledge-format
brainstorm.

## Reference: vault-resolution algorithm

```elixir
defmodule Mnemosyne.Vault do
  @vault_schema_version 1

  @spec resolve(Path.t() | nil) :: {:ok, Path.t()} | {:error, term()}
  def resolve(flag) do
    raw =
      cond do
        is_binary(flag) ->
          flag

        env = non_empty_env("MNEMOSYNE_VAULT") ->
          env

        true ->
          case read_user_config() do
            {:ok, %Mnemosyne.Vault.UserConfig{vault: path}} ->
              path

            {:ok, nil} ->
              throw_not_configured()

            {:error, reason} ->
              throw_not_configured(reason)
          end
      end

    verify(raw)
  end

  @spec verify(Path.t()) :: {:ok, Path.t()} | {:error, term()}
  def verify(path) do
    abs =
      case File.stat(path) do
        {:ok, _} -> Path.expand(path)
        {:error, reason} -> raise "vault path does not exist: #{path} (#{inspect(reason)})"
      end

    marker_path = Path.join(abs, "mnemosyne.toml")

    raw =
      case File.read(marker_path) do
        {:ok, contents} ->
          contents

        {:error, :enoent} ->
          raise "not a Mnemosyne vault (missing mnemosyne.toml): #{abs}"

        {:error, reason} ->
          raise "cannot read mnemosyne.toml at #{abs}: #{inspect(reason)}"
      end

    case Toml.decode(raw) do
      {:ok, %{"vault" => %{"schema_version" => v}}}
      when is_integer(v) and v <= @vault_schema_version ->
        {:ok, abs}

      {:ok, %{"vault" => %{"schema_version" => v}}}
      when is_integer(v) ->
        raise "vault schema_version=#{v} but this daemon supports #{@vault_schema_version}; upgrade Mnemosyne"

      {:ok, _} ->
        raise "invalid mnemosyne.toml: #{abs}: missing [vault].schema_version"

      {:error, parse_err} ->
        raise "invalid mnemosyne.toml: #{abs}: #{inspect(parse_err)}"
    end
  end

  defp non_empty_env(key) do
    case System.get_env(key) do
      nil -> nil
      "" -> nil
      v -> v
    end
  end
end
```

Key points:

- **`Path.expand/1` replaces the earlier Rust `canonicalize()`**. The
  distinction matters: `Path.expand/1` resolves `..`, `~`, and
  relative segments but does not follow symlinks. Symlink resolution
  happens inside `File.stat/1` (which follows by default). For the
  vault root itself A requires that the resolved path be a **real
  directory**, not a symlink, because `<vault>/projects/<name>/`
  symlinks are created relative to the vault root and a symlinked root
  would make those targets ambiguous. The daemon boot sequence adds an
  explicit `File.lstat!/1` check to reject a symlinked vault root with
  a hard error.
- All four error cases hard-fail with an actionable diagnostic naming
  the offending path.
- `init` bypasses `Mnemosyne.Vault.resolve/1` entirely — it creates
  rather than finds. The bootstrap subcommand dispatcher routes
  `mnemosyne init` through a separate code path.
- The full daemon boot sequence is:
  1. Resolve vault (this function).
  2. Verify vault (this function).
  3. Acquire system-wide singleton lock at
     `<runtime_dir>/mnemosyne/daemon.lock` (sub-D). The lock file
     contains the daemon's PID and the vault path.
  4. Start the OTP supervision tree (sub-F).
  5. Bind the Unix socket at `<vault>/runtime/daemon.sock` (sub-F).
  6. Enumerate `<vault>/projects/*/mnemosyne/project-root/` subtrees
     and start PlanActors (sub-F).
  7. Enumerate `<vault>/experts/*.md` and start ExpertActors (sub-N).
  8. Load and compile `<vault>/routing.ex` (sub-F).
  9. Regenerate `<vault>/plan-catalog.md` (sub-F).
  10. Mark the daemon "ready" via sub-M `:telemetry` event.
  Any failure in 1–3 is a hard error before the supervision tree
  starts (clean exit, no garbage). Failures in 4–10 escalate through
  OTP supervision per sub-F's restart strategy.

## Cross-sub-project contracts

The contracts A locks for the rest of the orchestrator merge:

| Contract | Owner consuming it |
|---|---|
| Vault discovery = flag → env → user config → hard error; resolved **once at daemon boot** | F (daemon boot), all bootstrap subcommands |
| Vault marker = `<vault>/mnemosyne.toml` with `[vault].schema_version` | F, D, G |
| Vault layout per A4 (including `daemon.toml`, `routing.ex`, `plan-catalog.md`, `experts/`, `runtime/{daemon.sock,mailboxes,interrupts,events,staging}`) | B (`runtime/staging/`), C (spawn CWD), D (system-wide lock, not in vault), E (`runtime/events/`, `<vault>/knowledge/`), F (`routing.ex`, `plan-catalog.md`, `experts/`, `daemon.sock`, `mailboxes/`), I (`.obsidian/` template), G (per-machine `adopt-project` reruns, rename migration), N (`experts/` declaration format) |
| `<project>/mnemosyne/project-root/` as F's reserved root-plan directory (replaces the earlier `plans/` container) | F (owner), G (migration), B (staging paths) |
| Symlink target = `<vault>/projects/<lowercase-name>/ -> <project>/mnemosyne/` | F, G, I |
| Tier 1 / Tier 2 split = derived from vault + active plan, env-var overridable; walk-up looks for `mnemosyne/project-root/` ancestor | E, B, F |
| Gitignore policy per A5 (adds `daemon.sock`, `mailboxes/` to gitignored side; singleton lock is system-wide, not in vault) | D, E, F, G |
| `init` scaffolds empty `knowledge/`, empty `experts/`, minimal `routing.ex` stub, minimal `daemon.toml` stub, empty `plan-catalog.md` with machine-owned header | E, F, N, future knowledge-format brainstorm |
| `adopt-project` is the per-machine project mounting step; on-live-daemon variant reads system-wide lock file then sends `:rescan` over `runtime/daemon.sock` | F, G |
| System-wide daemon singleton lock at `<runtime_dir>/mnemosyne/daemon.lock` prevents a second daemon on the machine | D, F |

## Memory.md updates this amendment produces

The orchestrator memory.md already reflects the A5 scope expansion
(`routing.ex`, `daemon.toml`, `plan-catalog.md`, `experts/`,
`daemon.sock`, `mailboxes/`) via the Session-9 F commitment and the
Session-11/12 sub-C/sub-B rewrites. No further orchestrator memory
edits are required by this amendment beyond flipping the sub-A
brainstorm row from "**F amendment pending**" to "**F amendment
absorbed (Session 14, inline rewrite)**" and recording the absorption
in the brainstorm history at the top of the doc.

The sub-A sibling plan's memory.md needs the following in-place
changes:

1. **Strike the "BEAM pivot amendment (unblocked)" section** and
   replace with "BEAM pivot absorbed (Session 14, inline rewrite) —
   see the rewritten design doc."
2. **Strike the "Critical-path note (2026-04-15)"** — the note is now
   stale; sub-F's Task 0 gate sees the absorption via the updated
   orchestrator memory table row.
3. **Update "Dependencies on sibling sub-projects"** to cite F's
   actual commitments (project-root, routing module, plan catalog,
   expert declarations) rather than the earlier "F may change the
   descent invariant" hedge.

The sub-A sibling plan's backlog.md needs:

1. **Strike the "BEAM Pivot Notice" block** and replace with "Design
   doc fully rewritten inline for Elixir/BEAM + sub-F commitments
   (Session 14). Task descriptions remain as intent specifications."
2. **Strike the "still needs an inline rewrite" paragraph** — done.
3. **Retain Tasks 1–15 as intent specifications.** Every task
   translates Rust idioms (cargo, serde, include_str!, clap, chrono)
   to their Elixir equivalents per the "executing session translates"
   rule already in the backlog header. The task list itself does not
   need a rewrite; the design doc it cites is now authoritative.

## Non-goals (explicit)

- **No migration from `~/.mnemosyne/`.** No existing usage to migrate.
  v0.1.0 paths are deleted in sub-G's migration, not transitioned.
- **No walk-up dev-root discovery.** Considered and rejected. The
  single-vault-per-machine model is a deliberate simplification,
  validated by two rounds of brainstorming and the BEAM pivot.
- **No multi-vault management.** A user can have multiple vaults on
  disk and switch between them via env var or `--vault`, but Mnemosyne
  does not maintain a registry, does not have a `mnemosyne vault list`
  command, and does not "know about" more than one at a time.
- **No automatic multi-machine sync.** The vault is a git repo; the
  user pushes/pulls. Conflict resolution is git's job, not
  Mnemosyne's. Team mode (sub-P, v2+) revisits this.
- **No `<vault>/mnemosyne.toml` overrides scaffolded at init.** The
  optional override sections inside `mnemosyne.toml` are present only
  if the user later opts in.
- **No default expert declarations scaffolded by A.** Sub-N's
  brainstorm owns the default expert set; A just creates an empty
  `experts/` directory.
- **No Tier 2 axis pre-commitment.** `init` scaffolds an empty
  `knowledge/` directory; the axis structure is for a future
  knowledge-format brainstorm, possibly post-v1.
- **No prompts-as-vault-data in v1.** Sub-B's compile-time-embedded
  prompts decision (Session-12 rewrite) stands. Prompts-as-data is a
  forward direction, not a v1 feature.
- **No remote setup at init.** The vault has its own `.git` but no
  remote; the user adds one via plain `git remote add`.
- **A does not own the contents of `routing.ex`, `plan-catalog.md`,
  or `experts/*.md`.** A ships scaffolded stubs at init; sub-F and
  sub-N own the content evolution.
- **A does not own the daemon lifecycle, supervision, or actor
  wiring.** Sub-F owns the daemon shape; A just guarantees vault
  resolution and the layout the daemon expects to find at boot.

## Open questions for the implementation phase

1. **What does the embedded `.obsidian/` template actually contain?**
   The A4 list is illustrative — the implementation needs concrete
   file contents. Probably belongs to a small "design the default
   Obsidian template" subtask that produces the JSON files, possibly
   informed by sub-I's "which Obsidian features cover which Mnemosyne
   data surfaces" brainstorm if I lands first. If I has not landed
   yet, A's implementation ships a minimal template (Dataview enabled,
   sensible `app.json` defaults, no opinionated CSS) and I's
   brainstorm later refines it. Unchanged by the BEAM pivot.
2. **Should `init` validate that the parent of the target path is
   writable before doing anything?** Currently the design says "let
   `File.mkdir_p!` fail naturally" — that produces a less-friendly
   error message but avoids TOCTOU races. Decide during implementation
   whether the friendlier message is worth the extra check.
3. **Does `adopt-project` belong inside `init`'s scope or is it a
   separate feature?** A9 documents it for completeness, but it's
   arguably its own task. Implementation phase decides whether to
   ship it as part of the vault-discovery PR or as a follow-up.
4. **Schema migration mechanism for future `schema_version` bumps.** A
   pins `schema_version = 1` and verification refuses higher values.
   There is no mechanism to migrate a v1 vault to v2. Fine for v1 but
   should be a deliberate future-design item — not A's job to design
   now, but A's job to flag.
5. **Cross-cutting observability adoption.** Per the M brainstorm
   discipline, A's sibling implementation plan must include stub
   tasks for emitting `Mnemosyne.Event.*` structs at vault discovery,
   init, and `adopt-project` boundaries. Post-M-amendment, the stubs
   are typed Elixir structs emitted via `:telemetry.execute/3` (not
   Rust `tracing` calls). Candidate event types:
   `Mnemosyne.Event.VaultResolved`,
   `Mnemosyne.Event.VaultVerificationFailed`,
   `Mnemosyne.Event.VaultInitialised`,
   `Mnemosyne.Event.ProjectAdopted`.
6. **What does the minimal `daemon.toml` contain?** A6 ships a
   two-section stub (`[daemon].log_level`,
   `[harnesses.claude_code]`). Sub-F owns the full schema. If sub-F
   lands a more complete daemon.toml spec before A's implementation
   executes, A's init picks up the richer stub. If not, A ships the
   two-section minimum and F's implementation phase extends it in
   place.
7. **Team-mode use is unanswered.** Mnemosyne is currently a
   solo-developer tool by design and assumption. Every A decision
   reflects this: single vault per machine, single user pushes/pulls
   the vault git, sequential curation, no concurrent-edit conflict
   handling, no per-user attribution in the marker file, no identity
   in the user config. Sub-P's scope (v2+) revisits this. Does not
   block v1 implementation — the gitignore policy already separates
   "shared asset" from "machine-local state," which is exactly the
   split a team workflow would need.

## Appendix A — Decision Trail

Clarifying-question-style decision history for sub-project A. Q1–Q5
were locked during the original Session-7 brainstorm (2026-04-13).
Q6 is the BEAM daemon pivot (Session 9, 2026-04-14). Q7 is the
sub-F architectural commitments that affect the vault layout
(Session 9). Corrections are inline at the relevant question, not
in a separate amendment layer, per the "rewrite specs inline" project
discipline.

### Q1 — Discovery model: walk-up, env-and-config, or registry?

**Locked**: env-and-config precedence chain (flag → env var → user
config → hard error). No walk-up. No registry.

**Rationale**: walk-up introduces the "forgotten ancestor
`Mnemosyne-vault/`" footgun. A registry duplicates state that the
filesystem already expresses. The precedence chain is trivially
testable with a single env-var override and composes cleanly with
future daemon spawns of internal harness sessions.

**Session-14 correction**: unchanged. The BEAM pivot only changes
*how often* the chain runs (once per daemon boot vs. once per CLI
invocation), not *what* it resolves.

### Q2 — Init flow: single command with `--from` flag, or two subcommands?

**Locked**: single `mnemosyne init` command with an optional `--from
<git-url>` flag that switches between fresh scaffold and clone
semantics.

**Rationale**: keeps the user-facing surface small and mirrors `git
init` / `git clone` naming conventions. The flag switch is cleaner
than two nearly-identical subcommands.

**Session-14 correction**: unchanged. `init` is now a *bootstrap
subcommand* (runs outside the daemon supervision tree) rather than a
standalone binary, but the flag-based clone semantics are identical.

### Q3 — Gitignore policy: opt-in or opt-out; what's tracked by default?

**Locked**: tracked-by-default subset is `mnemosyne.toml`,
`.gitignore`, curated `.obsidian/`, `knowledge/`, `archive/`.
Everything under `runtime/`, `cache/`, `projects/` is gitignored.

**Rationale**: tracks the "shared asset" surface (global knowledge,
vault identity, Obsidian template); excludes the "machine-local
state" surface (runtime, cache, symlinks). Users get a `git
push`-able vault from day one.

**Session-14 correction**: expanded the tracked side with four new
surfaces (`daemon.toml`, `routing.ex`, `plan-catalog.md`,
`experts/*.md`) committed by sub-F's Session-9 brainstorm. Expanded
the gitignored side with three new runtime surfaces (`daemon.sock`,
`mailboxes/`, `interrupts/`). The singleton lock is system-wide (not
in vault) per sub-D's Session-18 design. The
opt-out principle is unchanged; the set of files the principle
applies to grew.

### Q4 — Migration scope: transition `~/.mnemosyne/` or delete outright?

**Locked**: delete outright. v0.1.0 has no real users; migration
code is bureaucratic overhead with no benefit. Sub-G owns the
deletion site.

**Rationale**: "no real users" is a one-shot license to skip
migration scaffolding. Every `~/.mnemosyne/` reference becomes
dead code at G's cutover.

**Session-14 correction**: the scope of "v0.1.0 code to delete"
grew from "hardcoded `~/.mnemosyne/` paths in `main.rs` and
`Config::load` in `src/config.rs`" to "the entire Rust CLI binary"
when the BEAM pivot landed. Sub-A's amendment does not change this
— G's cutover deletes the Rust codebase wholesale.

### Q5 — Identity verification: dotfile marker or parseable config?

**Locked**: parseable TOML `mnemosyne.toml` with a schema-versioned
`[vault]` table. Human-readable, forward-compat, shares a file with
optional overrides.

**Rationale**: dotfiles hide from Obsidian's file tree, which is
load-bearing for the "humans can explore everything" discipline. A
TOML marker is visible, parseable, and mirrors `Cargo.toml`'s
"project marker + config" shape users already know.

**Session-14 correction**: unchanged. Swapping `serde+toml` for
Elixir's `Toml` library does not change the schema or the shape of
the parseable marker. `mix.exs` happens to be Elixir's "project
marker + config" file, reinforcing the ecosystem analogy.

### Q6 — BEAM daemon pivot: how does vault discovery survive the runtime swap?

**Added Session 14** (absorbing Session 9's F commitment).

**Decision**: vault discovery remains a **pure function from CLI
args + environment + user config to an absolute path**, independent
of runtime language. The only changes are:

- It runs **once per daemon boot**, not once per CLI invocation. All
  internal actors read from a single `Mnemosyne.Vault` GenServer.
- The implementation swaps Rust's `dirs::config_dir()` for Elixir's
  `:filename.basedir(:user_config, "mnemosyne")`; swaps
  `serde::Deserialize` for the `Toml` library's decode; swaps
  `PathBuf` for `Path.t()`; swaps `anyhow::Result` for `{:ok, _} |
  {:error, _}` tuples or `raise` for truly exceptional cases.
- `Path.expand/1` replaces `canonicalize()` with an added
  `File.lstat!/1` symlink-rejection check at the vault root (see
  "Reference: vault-resolution algorithm").
- All bootstrap subcommands (`init`, `adopt-project`, `config
  use-vault`, `init --from`) are short-lived escript-style entry
  points that do not start the OTP supervision tree.
- A second `mnemosyne daemon` is blocked by sub-D's system-wide
  singleton lock at `<runtime_dir>/mnemosyne/daemon.lock`.

**Rationale**: every Session-7 decision survives the language swap
because none of them were language-specific. The "discovery is one
pure function" framing is stable across any runtime; the "identity
via TOML marker" framing is stable across any TOML parser. The pivot
strengthens the "single source of truth per daemon" model because
all actors now read from one GenServer rather than re-resolving per
CLI invocation.

### Q7 — Sub-F commitments: what vault-layout additions and renames does A absorb?

**Added Session 14** (absorbing Session 9's F commitment and
Session 11 sub-C + Session 12 sub-B rewrites).

**Decision**: A's vault layout picks up the following changes,
all tracked in A4:

**New at vault root (tracked in git):**
- `daemon.toml` — F's daemon config file. A scaffolds a two-section
  stub at init.
- `routing.ex` — F's user-editable routing module. A scaffolds a
  no-route stub at init.
- `plan-catalog.md` — F's auto-regenerated vault catalog. A scaffolds
  a machine-owned header with zero entries at init; the daemon writes
  the full catalog on first boot.
- `experts/` — F's expert declarations directory. A scaffolds empty;
  sub-N's brainstorm provides the default expert declaration files.

**New under `runtime/` (gitignored):**
- `daemon.sock` — F's Unix socket for TUI/Obsidian/web clients. Bound
  by the daemon at boot, not at init.
- `mailboxes/<qualified-id>.jsonl` — F's actor mailboxes. Directory
  created at init; individual mailbox files created by actors.

**Moved out of vault (system-wide):**
- D's singleton lock moved to `<runtime_dir>/mnemosyne/daemon.lock`
  (system-wide, not in vault). Contains PID + vault path. Invisible to
  Obsidian. Bootstrap subcommands read this file to detect whether a
  daemon is serving their target vault.
- `daemon.pid` eliminated — redundant with the lock file content.

**Renamed/collapsed:**
- `<project>/mnemosyne/plans/` → `<project>/mnemosyne/project-root/`
  per F's "project-root as reserved plan directory" commitment. A's
  symlink target is still `<project>/mnemosyne/` (the container), so
  the rename does not change A's symlink shape. The change only
  affects A's `derive_tier1_from_plan/1` walk-up logic, which now
  searches for `mnemosyne/project-root/` instead of `mnemosyne/plans/`.
- `runtime/locks/<plan-id>.lock` deleted (not renamed). Sub-D's scope
  collapsed because OTP mailbox serialization inside PlanActor
  GenServers handles per-plan ordering; the singleton lock is
  system-wide.

**Rationale**: every addition follows the "A owns the vault-layout
stable contract; sibling sub-projects own the contents" discipline.
A does not design routing rules, catalog regeneration, expert
declaration formats, or mailbox wire formats — it just guarantees
the directory exists at the expected path and is tracked or
gitignored per A5. F's commitments compose cleanly with A's
Session-7 decisions; no Q1–Q5 answer had to change.

## Appendix B — `mix.exs` deps projection

The deps sub-A introduces to the daemon's `mix.exs`:

```elixir
{:toml, "~> 0.7"}          # TOML parser for mnemosyne.toml + daemon.toml
```

Everything else sub-A needs is in the Elixir/OTP standard library or
already pulled in by other sub-projects:

- `File.*`, `Path.*`, `System.*`, `DateTime.*` — stdlib
- `:filename.basedir/2` — stdlib (`:filename` module)
- `:telemetry.execute/3` — sub-M dep, already declared
- `Mnemosyne.Event.*` structs — sub-M module namespace

No Rust-side crates. The earlier brainstorm's Cargo.toml projection
(`dirs`, `serde`, `toml`, `chrono`, `scopeguard`) is retired.

## Appendix C — Glossary (post-amendment)

- **Vault** — the user-chosen directory holding `mnemosyne.toml` and
  the layout in A4. Resolved once per daemon boot.
- **Vault marker** — `<vault>/mnemosyne.toml` with a `[vault]` table.
- **Vault root symlink rule** — the vault root itself must be a real
  directory, not a symlink. `<vault>/projects/*/` symlinks are fine
  (and expected).
- **`project-root/`** — sub-F's reserved plan-directory name for the
  single root plan of each adopted project.
- **`mnemosyne.toml` vs `daemon.toml`** — identity + vault-format
  overrides vs. daemon runtime knobs. Two separate files.
- **Bootstrap subcommand** — `init`, `init --from`, `config
  use-vault`, `adopt-project`. Short-lived escript-style entry points
  that do not start the OTP supervision tree.
- **`Mnemosyne.Vault` GenServer** — the single source of truth for
  the resolved vault path while the daemon is running. Every actor
  reads from it; no actor re-runs the discovery chain.
- **`rescan` message** — the NDJSON message a bootstrap subcommand
  sends to a running daemon over `<vault>/runtime/daemon.sock` after
  mutating the vault (e.g., after `adopt-project` created a new
  symlink).
- **Singleton daemon lock** — sub-D's system-wide lock file at
  `<runtime_dir>/mnemosyne/daemon.lock`. One daemon per machine,
  enforced by `flock(2)`. Contains PID + vault path.

## Origin

Brainstormed 2026-04-13 as the work-phase task for sub-project A in
the Mnemosyne orchestrator plan. Followed the
`superpowers:brainstorming` skill's clarifying-question /
approach-proposal / section-by-section flow. Five forking decisions
(Q1–Q5) were locked through clarifying questions (discovery model,
init flow shape, gitignore policy, migration scope, identity
verification), then five design sections were presented for
section-by-section approval (layout, discovery, init, Tier 1 / Tier 2
addressability, cross-sub-project contracts). One mid-design correction
(symlink target) was applied and then walked back when the existing
memory.md framing was re-checked. The final Session-7 design landed
cleanly with no contradictions to B / E / M's prior decisions.

**Amended Session 14 (2026-04-15)** to absorb the BEAM daemon pivot
(Session 9) and sub-F's architectural commitments (Session 9:
`project-root/`, `routing.ex`, `plan-catalog.md`, `experts/`, Unix
socket client protocol, path-based qualified IDs) plus the consequent
scope collapse of sub-D. The amendment is an inline rewrite — §A1–§A10
are re-cast in place, the reference algorithm is rewritten in Elixir,
the cross-sub-project contracts table is updated, and the decision
trail is preserved in Appendix A with Q6 (BEAM pivot) and Q7 (sub-F
commitments) recording the amendment substance. No Q1–Q5 answer had
to change: the discovery model, init flow, gitignore principle,
migration scope, and identity verification all survive the runtime
swap unchanged in spirit. **No supersede-amendment layer; no stale
Rust content under a disclaimer.** The rewrite follows the precedent
set by sub-C (Session 11, 1186 lines) and sub-B (Session 12, 2296
lines).
