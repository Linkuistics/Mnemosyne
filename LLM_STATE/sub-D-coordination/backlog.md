# Backlog — Sub-project D: Daemon Coordination

Implementation backlog for sub-project D. All tasks derive from §1–§7
of the design doc at
`{{PROJECT}}/docs/superpowers/specs/2026-04-16-sub-D-coordination-design.md`.
Consult the spec before starting any task.

**Sibling plan scaffolded** in Session 18 of the orchestrator plan
(2026-04-16) immediately after the sub-D brainstorm. The spec is
frozen; this plan executes against it.

**Implementation runway:** sub-D has **no gate task**. All four modules
are pure Elixir with no dependency on sub-F's supervisor integration.
`SingletonLock` and `VaultGit` are independent. `FreshnessTracker`
must land before `FileIO`. Tests follow their respective modules.

**Adoption by siblings:** sub-B (`copy_back/2`), sub-E (`SafeFileWriter`),
sub-F (boot sequence, plan-catalog), and sub-N (expert absorb writes)
all integrate with `FileIO.write_safe/3`. Those adoption tasks live in
their respective sibling backlogs, not here. Sub-D delivers the modules;
siblings consume them.

## Task Backlog

### Task 1 — `Mnemosyne.Coordination.SingletonLock` `[impl]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Implement the singleton lock module per spec §2.
  `acquire/1` takes a vault path, resolves the system-wide lock directory
  (`$XDG_RUNTIME_DIR` or `System.tmp_dir!/0`), creates the directory if
  absent, opens `daemon.lock`, acquires `flock(LOCK_EX | LOCK_NB)` via
  `:erlang.open_port/2` with `flock -xn`, writes PID + vault path into
  the file. On contention, reads incumbent PID + vault path and returns
  `{:error, {:daemon_running, pid, vault_path}}`. On success, returns
  `{:ok, port}` — the caller holds the port for the daemon's lifetime.
  Emit `%Diagnostic{target: "mnemosyne.lock"}` events per spec §2.6.
  ~40 lines.
- **Results:** _pending_

### Task 2 — SingletonLock tests `[test]`
- **Status:** not_started
- **Dependencies:** Task 1
- **Description:** Per spec §7.1:
  - Two daemons, same vault: second hard-errors with correct PID + path.
  - Stale lock after crash: `kill -9` first daemon, second acquires ok.
  - Graceful shutdown: clean exit, lock released, second starts ok.
  Pure Elixir + filesystem fixtures.
- **Results:** _pending_

### Task 3 — `Mnemosyne.Coordination.FreshnessTracker` `[impl]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Implement the freshness tracker per spec §3.2.
  GenServer owning a `:named_table` + `:public` ETS table keyed by
  absolute path. `record/2` stores `{path, :crypto.hash(:sha256, content)}`.
  `check/1` re-reads file from disk, computes SHA-256, compares against
  stored hash. Returns `:unchanged` or `{:changed, current_hash}`.
  Started early in the supervision tree (before actors). ~50 lines.
- **Results:** _pending_

### Task 4 — `Mnemosyne.FileIO` `[impl]`
- **Status:** not_started
- **Dependencies:** Task 3
- **Description:** Implement the mandatory file I/O layer per spec §3.3.
  `read_tracked/1` reads file + records hash via FreshnessTracker.
  `write_safe/3` implements the full write discipline: category-based
  freshness check, atomic temp-then-rename (same-directory temp file),
  per-category conflict strategies (`:exclusive` skips check,
  `:phase_owned` writes anyway + warn, `:human_editable` discards + info),
  hash update, telemetry emission. Handles file-deleted-externally case
  per spec §3.3 step 6. ~80 lines.
- **Results:** _pending_

### Task 5 — FreshnessTracker + FileIO tests `[test]`
- **Status:** not_started
- **Dependencies:** Task 4
- **Description:** Per spec §7.2 and §7.3:
  - Phase-owned external modification: write succeeds, warn emitted.
  - Human-editable external modification: write discarded, info emitted.
  - Exclusive external modification: write proceeds, no diagnostic.
  - No external modification: write proceeds, no diagnostic.
  - File deleted externally (phase-owned): recreated, warn emitted.
  - File deleted externally (human-editable): skipped, warn emitted.
  - Atomic write safety: kill mid-write, no partial file at target path.
  Pure Elixir + filesystem fixtures.
- **Results:** _pending_

### Task 6 — `Mnemosyne.VaultGit` `[impl]`
- **Status:** not_started
- **Dependencies:** none
- **Description:** Implement the vault git interface per spec §4.2.
  `commit/3` takes vault path, list of file paths, commit message.
  Runs `git add` + `git commit` via `System.cmd/3`. On `index.lock`
  contention, emits `%Diagnostic{level: :error}` and returns
  `{:error, :index_lock_held}`. No retry. ~30 lines.
- **Results:** _pending_

### Task 7 — VaultGit tests `[test]`
- **Status:** not_started
- **Dependencies:** Task 6
- **Description:** Per spec §7.4:
  - Index.lock contention: hold lock, trigger commit, assert error + no retry.
  - Successful commit: trigger commit, assert in `git log`.
  Pure Elixir + filesystem fixtures (temp git repos).
- **Results:** _pending_
