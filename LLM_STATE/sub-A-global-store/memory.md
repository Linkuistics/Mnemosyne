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
- **Sub-project F**: A's `derive_tier1_from_plan` walks up from a plan
  path to find the project root. F owns plan hierarchy semantics and
  must preserve the invariant that every plan path has a
  `mnemosyne/plans/` ancestor. If F's hierarchy design changes the
  marker-file rule or the descent invariant, the walk-up logic must be
  revisited.
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
