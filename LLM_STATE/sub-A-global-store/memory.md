# Memory — Sub-project A: Vault location, discovery, and bootstrap

Implementation-level state for sub-project A of the Mnemosyne orchestrator
merge. The authoritative design lives at
`{{PROJECT}}/docs/superpowers/specs/2026-04-13-sub-A-global-store-design.md`
and the parent orchestrator plan at
`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/`. This file holds only
implementation-level discoveries, cross-sub-project dependency state, and
open questions surfaced during the build. Design-level contradictions
with the spec must be escalated to the parent plan rather than silently
absorbed here.

## Implementation strategy

Sub-project A introduces a new vault primitive and a new resolution
layer. Tasks in this plan build up in dependency order:

1. **Core types** (`VaultMarker`, `UserConfig`, `ResolvedRoots`) and the
   `VAULT_SCHEMA_VERSION` constant. Drive with unit tests for TOML
   parse/serialize round-trips.
2. **Marker file I/O** — read, write, schema version check.
3. **User config file I/O** — platform-specific path via
   `dirs::config_dir()`, atomic write-temp + rename, read with missing
   file returning `Ok(None)`.
4. **`resolve_vault` + `verify_vault`** — the full precedence chain and
   hard-error semantics from spec §A1 and the reference algorithm.
5. **`resolve_roots` + `derive_tier1_from_plan`** — Tier 1 / Tier 2
   resolution with env-var overrides from spec §A10.
6. **Embedded `.obsidian/` template** — author minimal template files
   under `templates/obsidian/`, wire them into the binary via
   `include_str!`, materialise at init time.
7. **`mnemosyne init <path>`** — fresh vault command, integrates 1-6.
8. **`mnemosyne init --from <git-url>`** — clone command.
9. **`mnemosyne config use-vault <path>`** — switch vault command.
10. **`mnemosyne adopt-project <project-path>`** — mount a project.
11. **Delete v0.1.0 hardcoded `~/.mnemosyne/` paths** from `src/main.rs`.
    Thread `ResolvedRoots` through every command that touches the store.
12. **Delete v0.1.0 `Config::load(dir)` call site.** Move language
    profiles and context mappings to binary-embedded defaults with
    optional overrides loaded from `mnemosyne.toml`.
13. **Observability adoption stub** — emit `mnemosyne_event!` calls at
    vault discovery, init, and adopt-project boundaries per sub-project
    M's cross-cutting discipline.
14. **Integration tests** — exercise init/clone/resolve via fixture
    directories using `MNEMOSYNE_VAULT` env var.
15. **Update user-facing docs** — `README.md`, `docs/reference.md`,
    `docs/user-guide.md` no longer describe `~/.mnemosyne/` as the
    knowledge store location.

Each task produces a commit with tests. Follow TDD discipline per
project coding-style guides — the test suite drives implementation.

## Dependencies on sibling sub-projects

- **Sub-project B**: consumes `ResolvedRoots` at executor startup. A's
  implementation lands before B's executor wiring so B can depend on
  real types rather than a stub. B's `PhaseRunner::run_phase` receives
  `ResolvedRoots` as a parameter.
- **Sub-project D**: consumes `<vault>/runtime/locks/` as the lock
  directory. A's init creates this directory and gitignores it; D's
  locking primitive lands after and writes lock files inside.
- **Sub-project E**: consumes `<vault>/runtime/events/` as the
  ingestion event queue and consumes both Tier 1 / Tier 2 roots for
  routing. Same dependency shape as D — A scaffolds the directories, E
  lands after.
- **Sub-project F** (Session 9, locked): F committed `project-root/`
  as the reserved root-plan directory under `<project>/mnemosyne/`,
  replacing the earlier `plans/` container. A's
  `derive_tier1_from_plan/1` walks up looking for the topmost
  `mnemosyne/project-root/` ancestor and returns
  `<that-mnemosyne-dir>/knowledge/`. F's invariants #3 ("every
  adopted project has exactly one `project-root/` directly under
  `<project>/mnemosyne/`") and #4 ("no plan at any depth is named
  `project-root` except the reserved root") guarantee termination
  and non-false-match. F also adds four tracked files at the vault
  root that A scaffolds at init (`daemon.toml`, `routing.ex`,
  `plan-catalog.md`, empty `experts/`), plus the gitignored
  `runtime/{daemon.sock,daemon.lock,daemon.pid,mailboxes/}` set.
  The vault root must remain a real directory, not a symlink — A's
  `verify_vault` adds an `lstat` check at boot.
- **Sub-project G**: consumes `mnemosyne adopt-project` as the
  per-machine project mounting step. G's migration scripts loop over a
  project list and invoke this command once per project.
- **Sub-project I**: owns the `.obsidian/` template content. A ships a
  minimal template (Dataview enabled, sensible `app.json` defaults, no
  opinionated CSS) if I has not landed; I's brainstorm later refines
  the template in place.
- **Sub-project M**: consumes `mnemosyne_event!` adoption stubs at
  vault boundaries. A's task 13 emits the stub calls; M's framework
  lands after and picks them up. Per the "cross-cutting brainstorms
  own their own sibling adoption stubs" discipline, the stubs are
  authored now rather than deferred to M's adoption wave.

## BEAM pivot + sub-F commitments absorbed (Session 14, inline rewrite)

The orchestrator's Session 9 committed Mnemosyne to a persistent BEAM
daemon (Elixir/OTP) and sub-F locked the architectural commitments
(`project-root/`, `routing.ex`, `plan-catalog.md`, `experts/`, Unix
socket client protocol, path-based qualified IDs, daemon singleton
lock collapsing sub-D's per-plan-lock scope). Sub-A's design doc was
rewritten inline in Session 14 (2026-04-15) to absorb both pivots
following the sub-C/sub-B precedent — see the rewritten
`specs/2026-04-13-sub-A-global-store-design.md`. Q1–Q5 are preserved
in Appendix A with correction notes; Q6 (BEAM pivot) and Q7 (sub-F
commitments) record the amendment substance.

**Net change to A4 vault layout:**

- New tracked at vault root: `daemon.toml`, `routing.ex`,
  `plan-catalog.md`, `experts/`.
- New gitignored under `runtime/`: `daemon.sock`, `daemon.lock`,
  `daemon.pid`, `mailboxes/<qualified-id>.jsonl`.
- `<project>/mnemosyne/plans/` → `<project>/mnemosyne/project-root/`
  (no change to A's symlink target — still
  `<vault>/projects/<name>/ -> <project>/mnemosyne/`).
- `runtime/locks/<plan-id>.lock` → singleton `runtime/daemon.lock`.

**Net change to A6 init flow:** init scaffolds the new tracked files
with minimal stubs (no-route `routing.ex`, two-section `daemon.toml`,
machine-owned-header `plan-catalog.md`, empty `experts/`). The
running daemon creates `daemon.sock`, `daemon.lock`, `daemon.pid` at
boot, not at init.

**Net change to A10 walk-up:** `derive_tier1_from_plan/1` searches
for the topmost `mnemosyne/project-root/` ancestor, not
`mnemosyne/plans/`. F's invariants #3 and #4 guarantee termination
and non-false-match.

**Implementation language:** all new code is Elixir/OTP. Backlog
tasks remain as intent specifications — `include_str!` becomes
`@external_resource` + `File.read!/1`; `serde` becomes the `Toml`
hex package; `dirs::config_dir()` becomes
`:filename.basedir(:user_config, "mnemosyne")`; `PathBuf` becomes
`Path.t()`; `anyhow::Result` becomes `{:ok, _} | {:error, _}` or
`raise` for hard errors. Tasks 11 and 12 reduce to "verify the
Elixir daemon does not carry over the old hardcoded paths" rather
than Rust refactoring work. The full Rust CLI is retired by sub-G's
migration scope.

## Open questions

### What does the embedded `.obsidian/` template actually contain?

Spec §A4 lists illustrative template files (`community-plugins.json`,
`core-plugins.json`, `app.json`, `snippets/`, `plugins/dataview/data.json`)
but does not pin their contents. Possible authoring strategies:

- **Minimal:** enable Dataview, set sensible `app.json` defaults
  (alwaysUpdateLinks=true, useMarkdownLinks=false to prefer wikilinks),
  no custom CSS. Implementation ships this unless sub-project I has
  landed first.
- **Informed by I:** if sub-project I's brainstorm has produced the
  Obsidian coverage design, the template incorporates I's
  recommendations. Requires A task ordering to depend on I's brainstorm.

Resolved at task 6 time based on I's status.

### Parent-directory writability pre-check?

Spec §A6 says "let `fs::create_dir_all` fail naturally." That produces
a less-friendly error but avoids TOCTOU races. Decide during
implementation whether the friendlier message is worth the extra
check.

### `adopt-project` as part of init's PR or a follow-up?

Spec §A9 documents it for completeness. Implementation phase decides
whether to ship it together with the vault-discovery work (task 10) or
as a follow-up PR. Recommendation: ship together — it is the natural
user-facing counterpart to `init`.

### Schema migration mechanism for future `schema_version` bumps

A pins `schema_version = 1` and verification refuses higher values.
There is no mechanism to migrate a v1 vault to v2. Fine for v1 but
should be a deliberate future-design item. Not A's job to design now;
A's job to flag. Flagged in spec §A3 and in the orchestrator plan.

### Team-mode use

Mnemosyne is currently a solo-developer tool by design and assumption.
Surfaced 2026-04-13 during A's brainstorm. See the parent orchestrator
plan's "Open questions" section for the full discussion. Does not
block A's v1 implementation — the gitignore policy already separates
shared assets from machine-local state, which is the split a team
workflow would eventually need.

## Risk watch list

### Risk: `.obsidian/` template contents are under-specified at authoring time

Mitigation: ship a minimal template and document the exact files in
the spec; sub-project I refines the template after I's brainstorm.
Materialised as an open question above.

### Risk: `derive_tier1_from_plan` walk-up logic breaks under sub-project F's hierarchy design

Mitigation: document the invariant F must preserve (every plan path
has a `mnemosyne/plans/` ancestor, the topmost `mnemosyne/` ancestor
is the project root). If F's brainstorm surfaces a contradiction,
escalate to the parent plan.

### Risk: user-config file location differs on macOS (`~/Library/Application Support/`) and trips up shell-script users

Mitigation: `dirs::config_dir()` is the platform-standard location.
Users who prefer XDG on macOS can set `MNEMOSYNE_VAULT` directly and
bypass the user config file. Documented in A1 and in the error
message when no vault is configured.

### Risk: `mnemosyne.toml` at the vault root collides with a project that already has a `mnemosyne.toml` elsewhere

Mitigation: the vault is at a dedicated top-level path (default
`~/Mnemosyne-vault/`), not inside a project repo. Collision is
impossible unless a user explicitly points `MNEMOSYNE_VAULT` at a
project repo that happens to have the same filename — in which case
the schema check ensures it is recognised as a valid vault or hard
errors on parse.

### Risk: `git clone` vault from a remote may lack the gitignored runtime tree, breaking first-run expectations

Mitigation: step 5 of spec §A7 creates the gitignored runtime tree
after the clone succeeds. The fresh clone is always a valid, usable
vault.
