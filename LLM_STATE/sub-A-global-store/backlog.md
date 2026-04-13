# Backlog — Sub-project A: Vault Location, Discovery, and Bootstrap

Implementation backlog for sub-project A. All tasks derive from the
design doc at
`{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-A-global-store-design.md`.
Consult the spec before starting any task.

Tasks are listed in approximately recommended order. The ordering
follows the dependency chain: core types → I/O primitives → resolution
functions → template authoring → commands → v0.1.0 deletion →
observability stub → tests → docs. The work phase picks the best next
task with input from the user.

## Task Backlog

### Task 1 — Core types + `VAULT_SCHEMA_VERSION` constant `[types]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Author the core types that the rest of sub-project A
  depends on. Land them first so sibling sub-projects (B, D, E, F) can
  consume them as real types rather than stubs.

  Types to define (per spec §A2, §A3, §A10):

  - `VaultMarker` — serde-deserialisable TOML shape, one nested
    `[vault]` table containing `schema_version: u32`, `created:
    chrono::DateTime<Utc>`, `created_by_version: String`. Additional
    optional override sections (`[language_profiles.*]`,
    `[context_mappings.*]`) deserialise into existing
    `config::LanguageProfile` / `config::Config` shapes; absent
    sections fall through to binary defaults.
  - `UserConfig` — serde-deserialisable TOML shape, single top-level
    `vault: PathBuf`. Room to extend later without breaking existing
    deserialisation.
  - `ResolvedRoots` — plain struct with `vault: PathBuf`,
    `tier2: PathBuf`, `tier1: Option<PathBuf>`. No serde; this is an
    in-memory type only.
  - `VAULT_SCHEMA_VERSION: u32 = 1` — pub const in the vault module.

  Unit tests (TDD): round-trip TOML parse/serialize for `VaultMarker`
  with and without override sections; `UserConfig` parse from minimal
  `vault = "..."` content; `VaultMarker` rejects missing `[vault]`
  section and missing `schema_version` field.
- **Results:** _pending_

### Task 2 — `mnemosyne.toml` marker file I/O `[io]`
- **Status:** not_started
- **Dependencies:** Task 1
- **Description:** Read/write functions for the vault marker file
  (`<vault>/mnemosyne.toml`). Per spec §A3 and the reference
  algorithm in the design doc.

  API to author:

  - `read_marker(vault_root: &Path) -> Result<VaultMarker>` — reads
    `<vault>/mnemosyne.toml`, parses it, returns the marker. Hard
    errors with the specific message strings from spec §A3:
    - "not a Mnemosyne vault (missing mnemosyne.toml): \<path>"
    - "invalid mnemosyne.toml: \<path>: \<parse error>"
  - `write_marker(vault_root: &Path, marker: &VaultMarker) -> Result<()>`
    — atomic write-temp + rename.
  - `check_schema_compatibility(marker: &VaultMarker) -> Result<()>`
    — returns Err with the spec's exact message string if
    `marker.vault.schema_version > VAULT_SCHEMA_VERSION`.

  Unit tests: missing file → expected error; malformed TOML → expected
  error; newer schema_version → expected error; valid marker
  round-trips through write/read.
- **Results:** _pending_

### Task 3 — User config file I/O `[io]`
- **Status:** not_started
- **Dependencies:** Task 1
- **Description:** Read/write the user-level config file at the
  platform-specific location via `dirs::config_dir().join("mnemosyne/config.toml")`.
  Per spec §A2.

  API to author:

  - `user_config_path() -> Result<PathBuf>` — returns the platform
    path or hard-errors if `dirs::config_dir()` returns `None`.
  - `read_user_config() -> Result<Option<UserConfig>>` — returns
    `Ok(None)` when the file doesn't exist, `Ok(Some(cfg))` when it
    parses, `Err` on parse error. Note: missing file is the normal
    "no vault configured yet" state, not an error.
  - `write_user_config(cfg: &UserConfig) -> Result<()>` — creates the
    parent directory if missing, writes atomically (write-temp +
    rename).

  Unit tests: round-trip; missing file → `Ok(None)`; malformed TOML
  → `Err`; atomic write leaves no temp file on success; atomic write
  leaves original file intact on temp-write failure. Use a
  per-test tmpdir with `MNEMOSYNE_CONFIG_HOME` override or a
  dependency-injected path for isolation.
- **Results:** _pending_

### Task 4 — `resolve_vault` + `verify_vault` `[resolution]`
- **Status:** not_started
- **Dependencies:** Task 2, Task 3
- **Description:** Implement the vault discovery precedence chain from
  spec §A1 and the reference algorithm at the end of the design doc.

  API to author:

  - `resolve_vault(flag: Option<&Path>) -> Result<PathBuf>` — applies
    the precedence chain: `--vault` flag → `MNEMOSYNE_VAULT` env var
    (non-empty) → user config → hard error. Returns an absolute
    canonicalised path.
  - `verify_vault(path: &Path) -> Result<PathBuf>` — canonicalises the
    path, reads the marker, calls `check_schema_compatibility`,
    returns the absolute path. Hard-errors per spec §A1 / §A3.

  Unit tests: each precedence level (flag > env > user config >
  error); empty env var falls through; missing user config falls
  through to error; verification rejects non-vault directories;
  canonicalisation resolves symlinks. Tests use tmpdirs for all
  vault locations and manipulate env vars via
  `scopeguard`-protected set/unset.
- **Results:** _pending_

### Task 5 — `resolve_roots` + `derive_tier1_from_plan` `[resolution]`
- **Status:** not_started
- **Dependencies:** Task 4
- **Description:** Tier 1 / Tier 2 resolution per spec §A10.

  API to author:

  - `resolve_roots(vault: PathBuf, active_plan: Option<&Path>) -> Result<ResolvedRoots>`
    — derives Tier 2 from vault, Tier 1 from the walk-up logic or the
    env var override. Verifies both resolved roots exist as
    directories.
  - `derive_tier1_from_plan(plan_path: &Path) -> Result<PathBuf>` —
    walks up from the plan path looking for the topmost `mnemosyne/`
    ancestor that has a `plans/` sibling (i.e., the ancestor sits at
    `<project>/mnemosyne/plans/...`), returns
    `<that-mnemosyne-dir>/knowledge/`.

  Unit tests: default derivation (Tier 2 = `<vault>/knowledge`, Tier 1
  walked from plan); env var overrides (`MNEMOSYNE_TIER1_ROOT`,
  `MNEMOSYNE_TIER2_ROOT`) bypass defaults; walk-up succeeds on
  well-formed plan path; walk-up hard-errors on malformed path
  lacking a `mnemosyne/plans/` ancestor; missing Tier 2 directory
  hard-errors; `active_plan: None` yields `tier1: None`.
- **Results:** _pending_

### Task 6 — Author the embedded `.obsidian/` template `[template]`
- **Status:** not_started
- **Dependencies:** none (parallelisable with Tasks 1–5)
- **Description:** Create the `templates/obsidian/` directory in the
  Mnemosyne source tree and author the minimal template files that
  `mnemosyne init` materialises per spec §A4 and §A6 step 4.

  Files to author:

  - `templates/obsidian/community-plugins.json` — lists Dataview as
    enabled.
  - `templates/obsidian/core-plugins.json` — sensible defaults
    (file-explorer, search, graph, backlinks, outline, tag-pane,
    page-preview enabled).
  - `templates/obsidian/app.json` — canonical settings:
    `alwaysUpdateLinks = true`, `useMarkdownLinks = false` (wikilinks
    preferred), `newLinkFormat = "shortest"`, other sensible
    defaults.
  - `templates/obsidian/plugins/dataview/data.json` — default
    Dataview config (refresh-on-file-change, inline queries enabled).
  - `templates/obsidian/snippets/mnemosyne.css` — empty file as a
    placeholder for future CSS styling; created so the directory
    exists and Obsidian-ingested templates can use it.

  Wire the files into the binary via `include_str!` at use-site in
  task 7. No code changes in this task — just the template files.

  **Deferred refinement:** if sub-project I's Obsidian coverage
  brainstorm has landed before this task is executed, consult I's
  design doc and incorporate recommendations into the template. If
  I has not landed, ship the minimal template listed above; I's
  later brainstorm can refine in place.

  Unit tests: not applicable (template authoring task); task 15's
  integration tests exercise materialisation end-to-end.
- **Results:** _pending_

### Task 7 — `mnemosyne init <path>` fresh vault command `[command]`
- **Status:** not_started
- **Dependencies:** Task 2, Task 3, Task 6
- **Description:** Implement the fresh init flow per spec §A6. Replaces
  the v0.1.0 `run_init` function in `src/commands/init.rs`.

  Step-by-step per spec §A6:

  1. Resolve target (default `~/Mnemosyne-vault/` when omitted).
  2. Pre-flight checks: target doesn't exist, parent exists, `git` on
     PATH.
  3. Create directory tree.
  4. Materialise embedded `.obsidian/` template via `include_str!`.
  5. Write `mnemosyne.toml` with marker table.
  6. Write `.gitignore` with tracked-vs-gitignored policy.
  7. `git init`, `git add .`, `git commit` — each hard-errors on
     failure.
  8. Update user config (with "different vault exists" check).
  9. Print success with next-step hints.

  Integration tests (in `tests/init_test.rs` — replaces the v0.1.0
  test file): fresh init in tmpdir succeeds; pre-existing target
  hard-errors; missing parent hard-errors; missing git hard-errors
  (via PATH manipulation); user config update both writes fresh and
  preserves different-vault state; `verify_vault` passes against
  the created vault.
- **Results:** _pending_

### Task 8 — `mnemosyne init --from <git-url>` clone vault command `[command]`
- **Status:** not_started
- **Dependencies:** Task 2, Task 3, Task 7
- **Description:** Implement the clone init flow per spec §A7.

  Step-by-step per spec §A7:

  1. Resolve target (default `~/Mnemosyne-vault/` when omitted).
  2. Pre-flight checks: target doesn't exist, parent exists, `git` on
     PATH, URL non-empty.
  3. `git clone <url> <path>` — hard error with git stderr on
     failure.
  4. `verify_vault(<path>)` — three failure modes all trigger
     `fs::remove_dir_all` cleanup.
  5. Create the gitignored runtime tree (`runtime/`, `cache/`,
     `projects/` and nested dirs).
  6. Update user config.
  7. Print success.

  Integration tests: clone from a local bare-repo fixture succeeds;
  clone of a non-vault repo triggers cleanup and hard-errors; schema
  version mismatch triggers cleanup and hard-errors.
- **Results:** _pending_

### Task 9 — `mnemosyne config use-vault <path>` command `[command]`
- **Status:** not_started
- **Dependencies:** Task 3, Task 4
- **Description:** Implement the vault-switching command per spec §A8.

  Steps:

  1. Parse the path argument, make absolute.
  2. Call `verify_vault` on the target — hard error if invalid.
  3. Create user config parent directory if missing.
  4. Write `UserConfig { vault }` atomically, preserving any future
     `[preferences]` section.
  5. Print confirmation.

  Integration tests: switch from vault A to vault B; refuse to switch
  to a non-vault target; refuse to switch to a newer-schema vault;
  verify the user config file contents after switch.
- **Results:** _pending_

### Task 10 — `mnemosyne adopt-project <project-path>` command `[command]`
- **Status:** not_started
- **Dependencies:** Task 4
- **Description:** Implement the per-machine project mounting command
  per spec §A9.

  Steps:

  1. Verify `<project-path>/mnemosyne/` exists — hard error otherwise.
  2. Compute symlink name as lowercase basename of the project
     directory.
  3. Resolve the active vault via the precedence chain.
  4. Create symlink `<vault>/projects/<lowercase-name> ->
     <abs-project-path>/mnemosyne/`. Hard error if a symlink or file
     with that name already exists at the destination.
  5. Print confirmation.

  Integration tests: successful adoption creates the symlink with
  the expected absolute target; project without `mnemosyne/` subdir
  hard-errors; duplicate adoption hard-errors; the resulting
  symlink is readable and points at the expected path.
- **Results:** _pending_

### Task 11 — Delete v0.1.0 `~/.mnemosyne/` hardcoded paths from `src/main.rs` `[cleanup]`
- **Status:** not_started
- **Dependencies:** Task 4, Task 5, Task 7, Task 8, Task 9, Task 10
- **Description:** v0.1.0's `src/main.rs` builds the knowledge store
  path via `dirs::home_dir().join(".mnemosyne")` at eight sites — one
  per Commands variant. Replace all of them with calls to
  `resolve_vault` (with a `--vault` flag plumbed through the `Cli`
  struct) and thread `ResolvedRoots` down into each command handler.

  Concretely:

  - Add `#[arg(long, global = true)] vault: Option<PathBuf>` to the
    `Cli` struct.
  - At the top of `main`, call `resolve_vault(cli.vault.as_deref())`
    for every command except `Init` and `Config` (which create or
    update rather than consume).
  - Update each command handler (`run_query`, `run_promote`,
    `run_curate`, `run_explore`, `run_install`, `run_status`) to
    accept a `ResolvedRoots` parameter rather than a
    `mnemosyne_dir` `PathBuf`.
  - Delete the `dirs::home_dir().join(".mnemosyne")` occurrences.
  - Delete `tests/init_test.rs` content that depends on the v0.1.0
    directory layout and replace with Task 7's integration tests.

  Verification: `rg '\.mnemosyne' src/ tests/` returns zero matches
  (except in deleted-code commits and in documentation strings that
  are updated in Task 15); `cargo test` and `cargo clippy` pass.
- **Results:** _pending_

### Task 12 — Delete v0.1.0 `Config::load(dir)` call site; move language profiles to embedded defaults `[cleanup]`
- **Status:** not_started
- **Dependencies:** Task 11
- **Description:** v0.1.0's `src/config.rs` exposes
  `Config::load(dir: &Path)` which reads `<dir>/config.yml` into a
  `Config` struct. With v0.1.0 paths deleted, this call site is dead.
  Refactor so that:

  - `Config::default()` (already exists) is the source of
    language profiles and context mappings.
  - Optional overrides come from the `[language_profiles.*]` and
    `[context_mappings.*]` sections of `<vault>/mnemosyne.toml`,
    loaded via `read_marker` from Task 2 and merged over the
    defaults.
  - Delete `Config::load(dir)` and `Config::save(dir)` — no code
    path uses YAML files anymore.
  - Delete any `config.yml` fixtures under `tests/fixtures/` and
    update `tests/config_test.rs` to test the merge semantics
    against TOML input.

  Verification: `rg 'config\.yml|Config::load|Config::save' src/
  tests/` returns zero matches; `cargo test` passes.
- **Results:** _pending_

### Task 13 — Observability adoption stub for sub-project M `[observability]`
- **Status:** not_started
- **Dependencies:** Task 4, Task 7, Task 10
- **Description:** Per sub-project M's "cross-cutting brainstorms own
  their own sibling adoption stubs" discipline, emit
  `mnemosyne_event!` calls at the boundaries sub-project A
  introduces. Stub format follows M's spec at
  `{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-M-observability-design.md`
  and M's implementation plan at
  `{{PROJECT}}/LLM_STATE/sub-M-observability/`.

  Boundaries to instrument:

  - **Vault discovery**: emit `VaultResolved { source, path }` after
    `resolve_vault` succeeds, where `source` is one of `Flag`,
    `EnvVar`, `UserConfig`. Helps diagnose "which config layer won"
    during incidents.
  - **Verification failure**: emit typed variants for
    `VaultMarkerMissing { path }`, `VaultMarkerInvalid { path, error
    }`, `VaultSchemaTooNew { path, found, supported }`.
  - **Init success**: emit `VaultInitialised { path,
    schema_version, fresh_or_clone }`.
  - **Adopt-project success**: emit `ProjectAdopted { vault_path,
    project_path, symlink_name }`.

  If M's framework hasn't landed yet at task time, author the event
  variants as plain `tracing::info!` / `tracing::error!` calls with
  the same structured fields. The typed-event macro swap is
  mechanical once M's implementation lands; M's migration discipline
  (parallel-emit + verification window) applies.

  Verification: each boundary site has an observability emission
  point; event payloads match M's spec shape; at minimum a
  `cargo check` passes.
- **Results:** _pending_

### Task 14 — Integration tests for init/clone/resolve `[tests]`
- **Status:** not_started
- **Dependencies:** Task 7, Task 8, Task 9, Task 10
- **Description:** End-to-end integration tests that exercise the
  full discovery → init → adopt flow against fixture directories
  under tmpdirs. Complements the per-task unit tests with
  cross-boundary scenarios.

  Scenarios to cover:

  - **Fresh init flow**: `init $tmpdir/vault` → `verify_vault` passes
    → `MNEMOSYNE_VAULT=$tmpdir/vault mnemosyne status` succeeds.
  - **Clone flow**: create a bare git fixture containing a minimal
    vault, `init --from $fixture_url $tmpdir/vault` → same
    downstream.
  - **Discovery precedence**: with all three layers set to distinct
    valid vaults, confirm `--vault` wins; with `--vault` omitted,
    env var wins; with both omitted, user config wins.
  - **Adopt-project round trip**: `adopt-project $tmpdir/fake-proj` →
    symlink exists at `$tmpdir/vault/projects/fake-proj` → readable.
  - **Test override env vars**: `MNEMOSYNE_TIER1_ROOT=$tmpdir/t1
    MNEMOSYNE_TIER2_ROOT=$tmpdir/t2` bypass the default derivation.

  All tests use per-test tmpdirs and scopeguard-protected env var
  manipulation to avoid test cross-contamination.
- **Results:** _pending_

### Task 15 — Update user-facing docs `[docs]`
- **Status:** not_started
- **Dependencies:** Task 11, Task 12
- **Description:** Update the documentation files that reference
  v0.1.0's `~/.mnemosyne/` location. Per user feedback memory
  (`feedback_documentation.md`), documentation is a first-class
  deliverable.

  Files to update:

  - `{{PROJECT}}/README.md` — replace any `~/.mnemosyne/` references
    with the new discovery model. Update the quick-start to show
    `mnemosyne init ~/Mnemosyne-vault`.
  - `{{PROJECT}}/docs/reference.md` — the `init` command section
    must describe the new flow (fresh + clone subcommands, user
    config update, the marker file, default path). Update the
    `status` and `query` sections to drop `~/.mnemosyne/`
    references.
  - `{{PROJECT}}/docs/user-guide.md` — replace the "Getting
    started" walkthrough with the new discovery / init flow.
  - `{{PROJECT}}/docs/configuration.md` — describe the user config
    file location (Linux/macOS/Windows), the override sections
    inside `mnemosyne.toml`, and the env vars
    (`MNEMOSYNE_VAULT`, `MNEMOSYNE_TIER1_ROOT`,
    `MNEMOSYNE_TIER2_ROOT`).

  Verification: `rg '\.mnemosyne' docs/` returns zero matches
  (except historical research notes under
  `docs/research-sources.md` if any); `rg '\.mnemosyne' README.md`
  returns zero matches.
- **Results:** _pending_
