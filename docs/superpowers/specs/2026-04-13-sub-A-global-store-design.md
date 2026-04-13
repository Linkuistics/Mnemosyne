# Sub-project A — Vault location, discovery, and bootstrap

**Status:** design complete, awaiting implementation
**Brainstorm date:** 2026-04-13
**Parent plan:** `LLM_STATE/mnemosyne-orchestrator/`
**Sibling implementation plan:** `LLM_STATE/sub-A-global-store/`

## Purpose

Replace Mnemosyne v0.1.0's hardcoded `~/.mnemosyne/` knowledge store with an
explicit, user-chosen, git-tracked **vault** that hosts Tier 2 global
knowledge, runtime state, the shipped Obsidian template, and per-project
mount points. Define how the Mnemosyne binary discovers the vault, how it
verifies a directory really is a vault, how it bootstraps a fresh one
(or clones an existing one), how Tier 1 and Tier 2 knowledge roots are
exposed for production and tests, and which contracts the rest of the
orchestrator's sub-projects can rely on.

## Scope

In scope:

- Vault directory layout (the full tree, what is git-tracked vs gitignored)
- Vault discovery mechanism (precedence chain, user config schema)
- Vault identity verification (the `mnemosyne.toml` marker)
- The `mnemosyne init` flow (fresh and clone variants)
- The `mnemosyne adopt-project` follow-on
- Tier 1 / Tier 2 root addressability via env-var overrides
- The cross-cutting contracts other sub-projects consume

Out of scope (explicit non-goals listed in detail at the end):

- Migration from `~/.mnemosyne/` (no existing usage)
- Multi-vault registry / `mnemosyne vault list`
- Automatic multi-machine sync
- Tier 2 axis structure (deferred to a future knowledge-format brainstorm)
- Team-mode multi-developer use (open question)

## Constraints absorbed from earlier brainstorms

These are non-negotiable inputs from sub-projects B, E, and M:

- **Vault layout is dedicated** (B): `<dev-root>/Mnemosyne-vault/` was B's
  framing, but A discards the implicit dev-root walk-up — the vault is just
  a user-chosen absolute path, default `~/Mnemosyne-vault/`.
- **Per-project content via symlinks** (B): the vault accesses each adopted
  project via `<vault>/projects/<project-name>/` symlinking to
  `<project>/mnemosyne/`. The `<project>/mnemosyne/` directory is the
  per-project state container holding `plans/` and `knowledge/`; it is
  *not* a single plan.
- **Symlink validation passed 6/6** (B, Session 5) on macOS Tahoe and
  Ubuntu 24.04. The vault-as-view-over-symlinks framing stands; the
  hard-copy fallback is not needed.
- **Obsidian-native format discipline** (B): Dataview-friendly
  kebab-case YAML frontmatter, wikilinks, tags, a Mnemosyne-shipped
  `.obsidian/` template with Dataview required and Templater optional.
- **Self-containment via embedded prompts** (B): Mnemosyne ships its
  prompts vendored into the binary via `include_str!`. The same pattern
  applies to the shipped `.obsidian/` template files.
- **Tier 1 and Tier 2 roots independently addressable as startup config**
  (E): both must be resolvable at process startup and overridable for
  tests pointing at fixture directories.
- **Hard errors by default** (project-wide): unexpected conditions,
  missing files, schema mismatches all fail loudly with a clear
  diagnostic naming the offending path. No silent fallbacks.
- **Cross-cutting brainstorms own their own sibling adoption stubs** (M):
  A's sibling implementation plan must include observability adoption
  stubs at the boundaries A introduces.

## Architectural decisions

### A1. Vault discovery is explicit; no walk-up; no implicit dev-root

The Mnemosyne binary resolves the active vault path through a strict
precedence chain on every invocation:

1. `--vault <path>` CLI flag — wins always; per-invocation override
2. `MNEMOSYNE_VAULT` env var — set by shell init or test harness
3. User config file:
   - Linux: `~/.config/mnemosyne/config.toml`
   - macOS: `~/Library/Application Support/mnemosyne/config.toml`
   - Windows: `%APPDATA%\mnemosyne\config.toml`
   - resolved via `dirs::config_dir()`
4. None of the above → hard error with actionable message

There is no walk-up search from `cwd`. There is no implicit dev-root
concept. The single-vault-per-machine model is a deliberate
simplification:

- It composes cleanly with Mnemosyne-as-LLM-client (internal reasoning
  sessions inherit the parent's env var with no `cwd` dance)
- Test isolation is trivial (`MNEMOSYNE_VAULT=$tmpdir` or `--vault $tmpdir`)
- It is consistent with B's "self-containment, no implicit ambient state"
  posture
- It bypasses the "what if a forgotten `Mnemosyne-vault/` lives in some
  ancestor directory" footgun that walk-up discovery would introduce

A user who genuinely wants multiple vaults on one machine switches via env
var or `--vault` per invocation. Mnemosyne does not maintain a registry of
known vaults.

### A2. User config schema (top-level flat)

```toml
# ~/.config/mnemosyne/config.toml
vault = "/Users/antony/Mnemosyne-vault"
```

Single key for v1. Future user-level preferences would land under
`[preferences]` or similar; the top-level `vault` key is the stable
contract.

### A3. Vault identity = `<vault>/mnemosyne.toml` schema-versioned marker

A directory is a Mnemosyne vault if and only if it contains a
`mnemosyne.toml` file at its root with a parseable `[vault]` table
containing `schema_version`. The same file doubles as the host for
optional vault-level configuration overrides.

```toml
# Mnemosyne vault — created by `mnemosyne init`
# Do not delete the [vault] section. Other sections are optional overrides.

[vault]
schema_version = 1
created = "2026-04-13T14:22:00Z"
created_by_version = "0.2.0"

# Optional: override embedded defaults. Omit any section to keep defaults.

# [language_profiles.rust]
# markers = ["Cargo.toml"]
# extensions = [".rs"]
# dependency_file = "Cargo.toml"
# dependency_parser = "cargo"

# [context_mappings.cargo_dependencies]
# tokio = ["async", "tokio", "concurrency"]
```

Verification rules (each is a hard error, never a silent fallback):

- File missing → `not a Mnemosyne vault (missing mnemosyne.toml): <path>`
- File parse error → `invalid mnemosyne.toml: <path>: <parse error>`
- `schema_version` higher than the binary's `VAULT_SCHEMA_VERSION` →
  `vault schema_version=N but this binary supports M; upgrade Mnemosyne`

The file is git-tracked. It serves three audiences: the Mnemosyne binary
(identity check), Obsidian users browsing the vault (human-readable
"what is this directory"), and forward-compat tooling (schema_version is
the gate for future vault format migrations).

The marker and the override config share one file because their key
spaces never overlap, eliminating a dotfile-without-extension and
mirroring the `Cargo.toml` "project marker + config" pattern Rust users
already know.

### A4. Vault directory layout

```
<vault>/
├── mnemosyne.toml                    # marker + optional overrides — tracked
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
├── archive/                          # tracked (auditable history)
├── runtime/                          # all gitignored, machine-local, ephemeral
│   ├── staging/<plan-id>/            # B's phase staging dirs (incl. materialised prompts/)
│   ├── locks/<plan-id>.lock          # D's per-plan advisory locks
│   ├── locks/store.lock              # D's store-level write lock for E ingestion
│   ├── interrupts/<plan-id>/         # pending interrupt commands for live sessions
│   └── events/                       # E's ingestion event queue
├── cache/                            # gitignored, derived data (indexes, parsed entries)
└── projects/                         # gitignored, machine-local symlink targets
    ├── apianyware-macos -> /Users/antony/Development/APIAnyware-MacOS/mnemosyne/
    ├── guivisionvmdriver -> /Users/antony/Development/GUIVisionVMDriver/mnemosyne/
    └── mnemosyne -> /Users/antony/Development/Mnemosyne/mnemosyne/
```

Notes:

- **Prompts do not live in the vault.** Per B, prompts are
  `include_str!`'d into the binary and materialised at
  `<staging>/prompts/` per phase render.
- **`init` scaffolds an empty `knowledge/` directory.** No axes are
  pre-committed; the Tier 2 axis structure is for a future
  knowledge-format brainstorm. (The v0.1.0 `projects/` axis is
  explicitly retired — project-specific knowledge is Tier 1 by
  definition, hosted per-project at `<project>/mnemosyne/knowledge/`.)
- **There is no `<vault>/config.yml`.** v0.1.0's YAML config file is
  deleted; A's `mnemosyne.toml` (TOML) replaces it. v0.1.0's
  `Config::load(dir)` site in `src/config.rs` becomes dead code in B's
  implementation phase.

### A5. Git-tracking policy

| Subtree | Tracked | Rationale |
|---|---|---|
| `mnemosyne.toml` | yes | Vault identity + optional overrides |
| `.gitignore` | yes | Bootstrapping |
| `.obsidian/` curated subset | yes | The shared template is the whole point of Obsidian-native discipline |
| `.obsidian/workspace*.json`, `.obsidian/cache` | no | Per-machine pane state, churns constantly |
| `knowledge/` | yes | Tier 2 global knowledge — the asset users want shared across machines |
| `archive/` | yes | Auditable history per v0.1.0 design |
| `runtime/` | no | Machine-local, ephemeral, all rebuildable |
| `cache/` | no | Derived data, rebuildable |
| `projects/` symlinks | no | Targets are machine-local absolute paths; including in git is a footgun |

This policy has the property that the vault is a clean push/pullable
git repo from the user's perspective: `git push` ships knowledge,
archive, and the shared Obsidian template; it does not ship runtime
state, machine-local symlink targets, or workspace pane state. The
gitignore file at vault root encodes the policy.

### A6. `mnemosyne init <path>` — fresh vault

Path argument is **optional**, defaulting to `~/Mnemosyne-vault/`. The UX
gain of a default outweighs the deliberate-placement argument; users
who want a different location pass it explicitly.

Step-by-step:

1. **Resolve target.** If `<path>` omitted, default to
   `dirs::home_dir().join("Mnemosyne-vault")`. Otherwise make `<path>`
   absolute via `std::path::absolute` (not `canonicalize`, since the
   path doesn't exist yet). Hard error if `dirs::home_dir()` returns
   `None`.
2. **Pre-flight checks**, each hard-erroring on failure with the
   offending path in the message:
   - `<path>` does not already exist (file or directory) — refuse to
     overwrite anything
   - Parent directory of `<path>` exists — refuse to silently create
     ancestors
   - `git --version` succeeds — bootstrap must work or fail loudly
3. **Create directory tree** with `fs::create_dir_all`:
   ```
   <path>/
   <path>/knowledge/
   <path>/archive/
   <path>/runtime/staging/
   <path>/runtime/locks/
   <path>/runtime/interrupts/
   <path>/runtime/events/
   <path>/cache/
   <path>/projects/
   <path>/.obsidian/
   <path>/.obsidian/snippets/
   <path>/.obsidian/plugins/
   ```
4. **Materialise the embedded `.obsidian/` template.** Each template
   file lives in the Mnemosyne source tree at `templates/obsidian/` and
   is embedded into the binary via `include_str!("../templates/obsidian/<file>")`.
   `init` writes each verbatim to its target. Same self-containment
   pattern B uses for prompts.
5. **Write `mnemosyne.toml`** with the marker table:
   ```toml
   [vault]
   schema_version = 1
   created = "<RFC3339 UTC timestamp>"
   created_by_version = "<env!(\"CARGO_PKG_VERSION\")>"
   ```
6. **Write `.gitignore`** with the tracked-vs-gitignored policy from A5:
   ```
   /runtime/
   /cache/
   /projects/
   /.obsidian/workspace.json
   /.obsidian/workspace-mobile.json
   /.obsidian/cache
   ```
7. **Initialise git, stage, commit.** Three sequential `git` calls:
   `init`, `add .` (safe here because the vault is brand-new and
   contains only files we just created), `commit -m "Initialize
   Mnemosyne vault"`. Each call hard-errors if it fails — no v0.1.0-style
   best-effort silent ignore.
8. **Update user config.** Atomic write of
   `~/.config/mnemosyne/config.toml` (or platform equivalent) with
   `vault = "<absolute-path>"`. If a user config already exists pointing
   elsewhere, do **not** overwrite — print a notice instead:
   > Vault created at `<path>`. Your user config still points at
   > `<other-vault>`. Run `mnemosyne config use-vault <path>` to switch,
   > or set `MNEMOSYNE_VAULT=<path>`.
9. **Print success** with next-step hints:
   ```
   ✓ Mnemosyne vault initialised at /Users/antony/Mnemosyne-vault
   ✓ User config updated to point at the new vault

   Next steps:
     mnemosyne adopt-project <path-to-project>    # add a project to the vault
     open /Users/antony/Mnemosyne-vault           # open the vault in Obsidian
   ```

**Atomicity:** if any step 3–7 fails, the partially-created vault
directory is **left in place** for the user to inspect. No destructive
cleanup on our own errors. The vault is unusable until init completes
successfully — the missing `mnemosyne.toml` ensures `verify_vault` will
refuse to recognise it.

### A7. `mnemosyne init --from <git-url> [<path>]` — clone existing vault

Path argument is optional, defaulting to `~/Mnemosyne-vault/` (overriding
`git clone`'s URL-basename habit for consistency).

Step-by-step:

1. **Resolve target** as in A6.
2. **Pre-flight checks**: `<path>` doesn't exist; parent exists; `git`
   on PATH; URL is non-empty.
3. **`git clone <url> <path>`**. Failure is a hard error with git's
   stderr in the message.
4. **Verify the clone is a Mnemosyne vault** by calling `verify_vault(<path>)`.
   Three failure modes: marker missing ("cloned repository is not a
   Mnemosyne vault"), parse error, schema_version higher than this
   binary supports. **All three trigger a cleanup** —
   `fs::remove_dir_all(<path>)` — because the clone created the
   directory and we don't want to leave garbage behind. (Distinct from
   fresh init's "leave it for inspection" because in the clone case
   the directory was created entirely by us, not partially populated by
   us.)
5. **Create the gitignored runtime tree**:
   `runtime/{staging,locks,interrupts,events}/`, `cache/`, `projects/`.
   These are not in the cloned repo (they're gitignored) but the rest
   of Mnemosyne expects them to exist. Same `fs::create_dir_all` block
   as fresh init step 3 minus the directories that came in via the
   clone.
6. **Update user config** as in A6 step 8.
7. **Print success** with the same next-step hints.

### A8. `mnemosyne config use-vault <path>` — switch vault

Writes/updates the user config file to point at a different vault.
Verifies the target is a valid vault first (calls `verify_vault`),
creates the parent directory if missing, writes the TOML atomically
(write-temp + rename). Single-key replacement; preserves any future
`[preferences]` sections under it.

### A9. `mnemosyne adopt-project <project-path>` — mount a project

Per-machine setup that compensates for the gitignored `projects/`
directory.

1. Verify `<project-path>/mnemosyne/` exists. Hard error otherwise:
   "not a Mnemosyne-managed project; the project must have a
   `mnemosyne/` subdirectory before it can be adopted."
2. Compute the symlink name as the lowercase basename of the project
   directory (e.g., `APIAnyware-MacOS` → `apianyware-macos`).
3. Resolve the active vault via the precedence chain.
4. Create symlink `<vault>/projects/<lowercase-name> -> <abs-project-path>/mnemosyne/`.
   Hard error if a symlink (or any file) with that name already exists.
5. Print confirmation.

Symlink targets are **absolute paths** so the link survives the user
moving cwd, but breaks if they move the project. That is acceptable: a
moved project should be re-adopted explicitly.

On a fresh clone of a vault from another machine, the user re-runs
`adopt-project` once per project. Sub-G's migration story may produce a
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
still honoured for everything else (runtime/, archive/, .obsidian/);
only the two knowledge roots are decoupled.

Resolution function:

```rust
pub struct ResolvedRoots {
    pub vault: PathBuf,
    pub tier2: PathBuf,
    pub tier1: Option<PathBuf>,
}

pub fn resolve_roots(vault: PathBuf, active_plan: Option<&Path>) -> Result<ResolvedRoots> {
    let tier2 = env_override("MNEMOSYNE_TIER2_ROOT")
        .unwrap_or_else(|| vault.join("knowledge"));

    let tier1 = if let Some(t1) = env_override("MNEMOSYNE_TIER1_ROOT") {
        Some(t1)
    } else if let Some(plan) = active_plan {
        Some(derive_tier1_from_plan(plan)?)
    } else {
        None
    };

    verify_directory_exists(&tier2, "Tier 2 knowledge root")?;
    if let Some(ref t1) = tier1 {
        verify_directory_exists(t1, "Tier 1 knowledge root")?;
    }

    Ok(ResolvedRoots { vault, tier2, tier1 })
}

fn env_override(key: &str) -> Option<PathBuf> {
    std::env::var(key).ok().filter(|s| !s.is_empty()).map(PathBuf::from)
}
```

`derive_tier1_from_plan` walks up the plan path looking for the topmost
`mnemosyne/` ancestor that contains a `plans/` sibling, then returns
`<that-mnemosyne-dir>/knowledge/`. If the walk fails (plan path
malformed or not under a `mnemosyne/plans/` ancestor), hard error with
the offending path in the message.

Optionality semantics:

- **Tier 2 is required** for any command that touches global knowledge
  (`query`, `promote`, `curate`, `status`). Hard error if
  `<vault>/knowledge/` doesn't exist.
- **Tier 1 is optional.** `tier1: None` means "no active plan, no
  per-project knowledge to serve." Commands that need it (E's
  ingestion pipeline, B's phase prompts that load Tier 1 entries into
  staging) hard-error when `tier1.is_none()`. Commands that don't need
  it (a global `query` from cwd `~`) proceed.

A's contract stops at "here's how to resolve the two roots given a
vault and an optional active plan." The *content* of Tier 1 / Tier 2
(axes, schema, frontmatter conventions) is not A's scope — it is owned
by E (ingestion routing rules) and a future knowledge-format
brainstorm.

## Reference: vault-resolution algorithm

```rust
pub const VAULT_SCHEMA_VERSION: u32 = 1;

pub fn resolve_vault(flag: Option<&Path>) -> Result<PathBuf> {
    let raw = if let Some(p) = flag {
        p.to_path_buf()
    } else if let Ok(s) = std::env::var("MNEMOSYNE_VAULT") {
        if s.is_empty() {
            try_user_config()?
        } else {
            PathBuf::from(s)
        }
    } else {
        try_user_config()?
    };
    verify_vault(&raw)
}

fn try_user_config() -> Result<PathBuf> {
    let cfg = read_user_config()?
        .ok_or_else(|| anyhow!(
            "no vault configured. Run 'mnemosyne init <path>' to create one, \
             set MNEMOSYNE_VAULT, or run 'mnemosyne config use-vault <path>'."
        ))?;
    Ok(cfg.vault)
}

fn verify_vault(path: &Path) -> Result<PathBuf> {
    let abs = path.canonicalize()
        .with_context(|| format!("vault path does not exist: {}", path.display()))?;
    let marker_path = abs.join("mnemosyne.toml");
    let raw = fs::read_to_string(&marker_path)
        .with_context(|| format!(
            "not a Mnemosyne vault (missing mnemosyne.toml): {}", abs.display()))?;
    let marker: VaultMarker = toml::from_str(&raw)
        .with_context(|| format!("invalid mnemosyne.toml: {}", abs.display()))?;
    if marker.vault.schema_version > VAULT_SCHEMA_VERSION {
        bail!(
            "vault schema_version={} but this binary supports {}; upgrade Mnemosyne",
            marker.vault.schema_version, VAULT_SCHEMA_VERSION
        );
    }
    Ok(abs)
}
```

Key points:

- `canonicalize()` is mandatory. It resolves symlinks and produces an
  absolute, real path. This ensures every downstream sub-project
  receives a consistent absolute path.
- All four error cases hard-fail with an actionable diagnostic naming
  the offending path.
- `init` bypasses `resolve_vault` entirely — it creates rather than
  finds. The CLI dispatch routes `Commands::Init { … }` through a
  separate code path.

## Cross-sub-project contracts

The contracts A locks for the rest of the orchestrator merge:

| Contract | Owner consuming it |
|---|---|
| Vault discovery = env var → user config → flag override; hard error on miss | B (executor startup), all CLI commands |
| Vault marker = `<vault>/mnemosyne.toml` with `[vault].schema_version` | B, D |
| Vault layout per A4 | B (`runtime/staging/`, `runtime/locks/`), D (`runtime/locks/`), E (`runtime/events/`, `<vault>/knowledge/`), F (`projects/<name>/mnemosyne/plans/`), I (`.obsidian/` template), G (per-machine `adopt-project` reruns) |
| Symlink target = `<vault>/projects/<lowercase-name>/ -> <project>/mnemosyne/` | F, G, I |
| Tier 1 / Tier 2 split = derived from vault + active plan, env-var overridable | E, B |
| Gitignore policy per A5 | D (locks must not be in git), E (events queue not in git), G |
| `init` scaffolds an empty `knowledge/` (no axis pre-commitment) | E, future knowledge-format brainstorm |
| `adopt-project` is the per-machine project mounting step | G |

## Memory.md updates this brainstorm produces

The following entries are appended/updated in
`LLM_STATE/mnemosyne-orchestrator/memory.md`:

1. **New "Stable architectural decisions" entry:** "Vault discovery is
   explicit: env var → user config → flag override; no walk-up; no
   implicit dev-root concept." Names hard-errors-by-default, test
   isolation, and Mnemosyne-as-LLM-client composition as rationale.
2. **New "Stable architectural decisions" entry:** "Vault identity is
   verified by `mnemosyne.toml` schema_version marker; missing or
   wrong-version is a hard error."
3. **Update the "Global knowledge store moves from `~/.mnemosyne/` to a
   visible location under DEV_ROOT" entry:** strike the DEV_ROOT
   framing, replace with "to a user-specified absolute path resolved
   via the discovery chain (default `~/Mnemosyne-vault/`)."
4. **New "Stable architectural decisions" entry:** "v0.1.0 has no real
   users; legacy `~/.mnemosyne/` paths are deletable, not
   transitionable. Sub-A produced no `migrate` command; B's
   implementation phase deletes the v0.1.0 hardcoded paths in
   `main.rs` and the `Config::load` call site outright."
5. **Update the "Sub-projects" table row for A:** mark brainstorm as
   **done 2026-04-13**, link this design doc and the sibling plan path.
6. **New "Open questions" entry:** "Team-mode usage of Mnemosyne" — see
   open question 6 below for the full text.

## Non-goals (explicit)

- **No migration from `~/.mnemosyne/`.** No existing usage to migrate.
  v0.1.0 paths are deleted in B's implementation, not transitioned.
- **No walk-up dev-root discovery.** Considered and rejected. The
  single-vault-per-machine model is a deliberate simplification.
- **No multi-vault management.** A user can have multiple vaults on
  disk and switch between them via env var or `--vault`, but
  Mnemosyne does not maintain a registry, does not have a `mnemosyne
  vault list` command, and does not "know about" more than one at a
  time.
- **No automatic multi-machine sync.** The vault is a git repo; the
  user pushes/pulls. Conflict resolution is git's job, not
  Mnemosyne's.
- **No `<vault>/config.toml` overrides scaffolded at init.** The
  optional override sections inside `mnemosyne.toml` are present only
  if the user later opts in.
- **No Tier 2 axis pre-commitment.** `init` scaffolds an empty
  `knowledge/` directory; the axis structure is for a future
  knowledge-format brainstorm, possibly post-v1.
- **No prompts-as-vault-data in v1.** B's embedded-prompts decision
  stands. Prompts-as-data is a forward direction, not a v1 feature.
- **No remote setup at init.** The vault has its own `.git` but no
  remote; the user adds one via plain `git remote add`.

## Open questions for the implementation phase

1. **What does the embedded `.obsidian/` template actually contain?** The
   A4 list is illustrative — the implementation needs concrete file
   contents. Probably belongs to a small "design the default Obsidian
   template" subtask that produces the JSON files, possibly informed by
   sub-I's "which Obsidian features cover which Mnemosyne data
   surfaces" brainstorm if I lands first. If I has not landed yet, A's
   implementation ships a minimal template (Dataview enabled, sensible
   `app.json` defaults, no opinionated CSS) and I's brainstorm later
   refines it.
2. **Should `init` validate that the parent of the target path is
   writable before doing anything?** Currently the design says "let
   `fs::create_dir_all` fail naturally" — that produces a less-friendly
   error message but avoids TOCTOU races. Decide during implementation
   whether the friendlier message is worth the extra check.
3. **Does `adopt-project` belong inside `init`'s scope or is it a
   separate feature?** A9 documents it for completeness, but it's
   arguably its own task. Implementation phase decides whether to ship
   it as part of the vault-discovery PR or as a follow-up.
4. **Schema migration mechanism for future `schema_version` bumps.** A
   pins `schema_version = 1` and verification refuses higher values.
   There is no mechanism to migrate a v1 vault to v2. This is fine for
   v1 but should be a deliberate future-design item — not A's job to
   design now, but A's job to flag.
5. **Cross-cutting observability adoption.** Per the M brainstorm
   discipline, A's sibling implementation plan must include a stub
   task for emitting `mnemosyne_event!` calls at vault discovery,
   init, and `adopt-project` boundaries (resolved-vault,
   schema-version-mismatch, marker-missing,
   adopt-symlink-created).
6. **Team-mode use is unanswered.** Mnemosyne is currently a
   solo-developer tool by design and assumption. Every A decision
   reflects this: single vault per machine, single user pushes/pulls
   the vault git, sequential curation, no concurrent-edit conflict
   handling, no per-user attribution in the marker file, no identity
   in the user config. Sub-D's locking model is also single-machine.
   What changes when two or more developers want to share a vault?
   Likely affects D (distributed locking semantics or pessimistic-lock
   transitions), E (ingestion conflict handling on shared knowledge
   writes), the curation workflow (parallel curation discipline), and
   possibly the marker file (per-user identity attribution). Does not
   obviously force A to revise the discovery chain or the layout — the
   gitignore policy already separates "shared asset" from
   "machine-local state," which is exactly the split a team workflow
   would need. Owner: TBD; likely a future cross-cutting brainstorm of
   its own once a team use case is concrete.

## Origin

Brainstormed 2026-04-13 as the work-phase task for sub-project A in the
Mnemosyne orchestrator plan. Followed the `superpowers:brainstorming`
skill's clarifying-question / approach-proposal / section-by-section
flow. Five forking decisions were locked through clarifying questions
(discovery model, init flow shape, gitignore policy, migration scope,
identity verification), then five design sections were presented for
section-by-section approval (layout, discovery, init, Tier 1 / Tier 2
addressability, cross-sub-project contracts). One mid-design correction
(symlink target) was applied and then walked back when the existing
memory.md framing was re-checked. The final design lands cleanly with
no contradictions to B / E / M's prior decisions.
