# Memory — Sub-project D: Daemon Coordination

This plan implements sub-project D of the Mnemosyne orchestrator merge.
D's brainstorm is complete; this plan is the implementation work, not
a design phase. The design is fully specified in the spec referenced
below.

## Primary reference

**`{{PROJECT}}/docs/superpowers/specs/2026-04-16-sub-D-coordination-design.md`**
is the authoritative design document. Every task in this plan's backlog
derives from the spec's §1–§7.

## Parent plan

The orchestrator-level plan lives at
`{{PROJECT}}/LLM_STATE/mnemosyne-orchestrator/` (currently — will move
to `{{PROJECT}}/mnemosyne/project-root/` after sub-project G's
migration lands). It coordinates this sub-plan with its siblings. The
parent plan's `memory.md` holds cross-sub-project architectural state.
This file holds only sub-project-D-specific implementation state.

## Key architectural anchors

### D-1: System-wide singleton lock, not per-vault
Lock file at `<runtime_dir>/mnemosyne/daemon.lock` via `flock(2)`.
Contains PID + vault path. Not inside the vault — invisible to Obsidian.
Enforces v1 single-daemon-per-machine constraint.

### D-2: Content-hash freshness, not mtime
SHA-256 of raw file bytes via `:crypto.hash/2`. Always read-before-write.
Detects actual content changes, not metadata changes.

### D-3: Three file categories with distinct strategies
`:exclusive` (no check, overwrite), `:phase_owned` (daemon-authoritative,
warn on conflict), `:human_editable` (human-authoritative, discard on
conflict). Categories assigned by the caller via `FileIO.write_safe/3`.

### D-4: No autonomous daemon git commits in v1
All git operations are human-mediated. Vault git concurrency is a
non-issue by design constraint.

### D-5: Edge-case safety nets, not high-frequency coordination
External-tool conflicts are rare. The typical workflow has the daemon as
sole writer during phase cycles. Co-equal-actors is a fallback guarantee.

## Sibling integration points

- Sub-B: `copy_back/2` → `FileIO.write_safe(path, content, :phase_owned)`
- Sub-E: `SafeFileWriter` → `FileIO.write_safe(path, content, :human_editable)`
- Sub-F: boot step 3 → `SingletonLock.acquire/1`; plan-catalog → `FileIO.write_safe(path, content, :exclusive)`; routing rules → `VaultGit.commit/3`
- Sub-N: expert absorb → `FileIO.write_safe(path, content, :human_editable)`
- Sub-M: subscribes to `%Diagnostic{target: "mnemosyne.coordination"}` and `%Diagnostic{target: "mnemosyne.lock"}` events
