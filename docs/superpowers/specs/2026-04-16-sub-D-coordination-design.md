# Sub-D: Daemon Coordination — Design Spec

> Daemon singleton enforcement, external-tool file coordination, and vault
> git concurrency constraints for Mnemosyne v1.

**Status:** design complete
**Date:** 2026-04-16
**Depends on:** sub-F (daemon architecture), sub-A (vault layout)
**Consumed by:** sub-B (copy_back), sub-E (SafeFileWriter), sub-F (boot
sequence, plan-catalog, routing rule commits), sub-N (expert absorb writes),
sub-M (diagnostic events)

---

## §1 Scope and non-goals

### §1.1 Scope

Three coordination concerns, each at a different scale:

1. **D1 — Daemon singleton lock.** Prevents two `mnemosyne daemon`
   processes from running against the same vault simultaneously.
2. **D2 — External-tool coordination.** Handles the case where Obsidian,
   a text editor, or another external tool modifies a file the daemon also
   reads and writes. Detection via content-hash freshness checking;
   resolution strategy varies by file category.
3. **D3 — Vault git concurrency.** Documents the v1 constraint that the
   daemon never performs autonomous git commits. All daemon-triggered git
   operations are human-mediated.

### §1.2 Non-goals

- **Per-plan advisory locks.** OTP mailbox serialization inside PlanActor
  GenServers handles intra-daemon per-plan ordering. F's daemon is a
  singleton, so inter-process per-plan contention does not exist.
- **Intra-daemon write coordination.** Same — GenServer serialization.
- **Multi-user concurrent access.** Sub-P (v2+) owns this.
- **Autonomous daemon git commits.** V1 constraint; all git operations are
  human-mediated. Sub-P revisits.
- **File-system watcher for external-modification detection.** Sub-N
  already watches `experts/` via `:file_system`. D uses read-before-write
  content hashing, not a watcher — proportionate to the rare-edge-case
  frequency.

### §1.3 Key principle

D's mechanisms are **edge-case safety nets**, not high-frequency
coordination primitives. The typical v1 workflow has the daemon as sole
writer during phase cycles, with humans browsing in Obsidian and
occasionally editing between cycles. The co-equal-actors principle is a
fallback guarantee — humans primarily interact via the work phase,
respond to agent decisions, and browse memories. Direct file editing
concurrent with daemon writes is rare.

---

## §2 Daemon singleton lock (D1)

### §2.1 Mechanism

`flock(2)` exclusive lock on a **system-wide** lock file, not inside the
vault. This prevents the lock from being visible in Obsidian's file tree
and enforces the v1 single-daemon-per-machine constraint.

**Lock file location** (per-user, cleared on reboot):
- **Linux:** `$XDG_RUNTIME_DIR/mnemosyne/daemon.lock`
  (typically `/run/user/<uid>/mnemosyne/daemon.lock`)
- **macOS:** `$TMPDIR/mnemosyne/daemon.lock`
  (per-user temp dir, cleared on reboot)

Resolved at daemon startup via:

```elixir
defmodule Mnemosyne.Coordination.SingletonLock do
  @spec lock_dir() :: Path.t()
  defp lock_dir do
    base = System.get_env("XDG_RUNTIME_DIR") || System.tmp_dir!()
    Path.join(base, "mnemosyne")
  end
end
```

The directory is created if absent (single `File.mkdir_p!/1`). The lock file
is not inside any vault, so Obsidian never sees it.

### §2.2 Lock file content

The lock file contains the daemon's OS PID **and** the vault path it
serves, newline-separated:

```
<pid>
<vault-absolute-path>
```

This allows the contention error message to include both: `"daemon already
running (pid <N>) serving vault <other-path>"`.

### §2.3 Acquisition sequence

Boot sequence step 3 (after vault resolve + verify, per sub-A §A6):

1. Ensure the lock directory exists (`File.mkdir_p!/1`).
2. Open `<lock_dir>/daemon.lock` with `File.open(..., [:write, :read])`,
   creating if absent.
3. Attempt `flock(LOCK_EX | LOCK_NB)` (non-blocking exclusive lock) on the
   file descriptor.
4. **On success:** write the daemon's OS PID and vault path into the file
   (overwriting any stale content), flush, hold the fd open for the lifetime
   of the daemon process. The kernel releases the lock automatically on
   process exit (clean shutdown or crash).
5. **On failure** (lock held by another process): read the file content for
   the incumbent PID and vault path, hard-error with
   `"daemon already running (pid <N>) serving vault <path>"`.

### §2.4 Crash safety

`flock(2)` is kernel-enforced. The lock is released when the file descriptor
is closed, which happens automatically on process death regardless of exit
reason (SIGKILL, OOM, panic, clean shutdown). No stale-lock-file cleanup is
needed. This is the primary reason for choosing `flock` over
`O_CREAT | O_EXCL` (which requires PID-liveness checks on stale files).

### §2.5 Elixir implementation

Erlang's `:file` module does not expose `flock(2)`. Implementation uses
`:erlang.open_port/2` with the OS `flock` command:

```elixir
lock_path = Path.join(lock_dir(), "daemon.lock")
port = Port.open(
  {:spawn, ~s(flock -xn #{lock_path} -c "sleep infinity")},
  [:binary, :exit_status]
)
```

The port process holds the lock. Linking the port to the daemon's
supervision tree ensures cleanup on crash — if the daemon dies, the port
dies, the kernel releases the flock. On contention, `flock -xn` exits
immediately with status 1, which the daemon detects via `:exit_status`.

Alternative: a small C port program calling `flock(2)` directly. More
control but introduces a compiled dependency. The `flock` CLI approach is
preferred for v1 — it requires no compilation and works on macOS and Linux.

### §2.6 Observability

Emit `%Diagnostic{target: "mnemosyne.lock"}` events per sub-M's adoption
matrix:

- `lock_acquired` — daemon startup, includes vault path
- `lock_released` — daemon shutdown (graceful only; crash releases silently)
- `lock_contended` — second daemon attempted, includes incumbent PID

---

## §3 External-tool coordination (D2)

### §3.1 File categories

The daemon's file universe splits into three categories by coordination
strategy:

| Category | Files | Strategy | Rationale |
|---|---|---|---|
| **Daemon-exclusive** (`:exclusive`) | `plan-catalog.md`, `runtime/*` (staging, mailboxes, events, interrupts) | No freshness check. Atomic write, overwrite unconditionally. | Machine-generated, overwritten on every regeneration. External edits are meaningless. Documented in file headers. |
| **Phase-cycle-owned** (`:phase_owned`) | `plan-state.md`, `memory.md`, `backlog.md`, `session-log.md` (during an active phase) | Daemon-authoritative detect-and-warn. | Daemon is mid-phase; re-reading would break session invariants. External edit is overwritten, diagnostic emitted. |
| **Human-editable** (`:human_editable`) | `routing.ex`, `knowledge/**/*.md`, `experts/*.md`, `daemon.toml`, `mnemosyne.toml` | Human-authoritative detect-and-discard. | Human edit is authoritative. Daemon re-reads, discards its in-flight version, emits diagnostic. |

Note on `experts/*.md`: sub-N's `DeclarationLoader` already watches this
directory via `:file_system` and hot-reloads on change. D's content-hash
mechanism is a complementary safety net covering the write path. In v1 the
daemon does not write to expert declarations, but the pattern is uniform
across all human-editable files.

### §3.2 Content-hash freshness check

Every daemon file read records a **freshness hash** — SHA-256 of the raw
file bytes — stored in an ETS table keyed by absolute path.

```elixir
defmodule Mnemosyne.Coordination.FreshnessTracker do
  @moduledoc false
  @table :mnemosyne_freshness

  @spec record(path :: Path.t(), content :: binary()) :: :ok
  @spec check(path :: Path.t()) :: :unchanged | {:changed, current_hash :: binary()}
end
```

- `record/2` — called after every daemon file read. Stores
  `{path, :crypto.hash(:sha256, content)}` in the ETS table.
- `check/1` — called before every daemon file write. Re-reads the file from
  disk, computes SHA-256 of current content, compares against the stored
  hash. Returns `:unchanged` or `{:changed, current_hash}`.

The ETS table is owned by a dedicated `GenServer` started early in the
supervision tree (before any actor). `:named_table` + `:public` read access
so `FileIO` can read without message-passing overhead. Writes go through the
owning GenServer for crash-safety (table survives individual actor crashes
but not daemon crashes — appropriate since daemon restart re-reads
everything from disk).

### §3.3 Write path: `Mnemosyne.FileIO`

All daemon file writes go through a shared module that enforces the
read-before-write + hash-check + atomic-write discipline:

```elixir
defmodule Mnemosyne.FileIO do
  @spec read_tracked(path :: Path.t()) :: {:ok, binary()} | {:error, File.posix()}
  @spec write_safe(
          path :: Path.t(),
          content :: binary(),
          category :: :exclusive | :phase_owned | :human_editable
        ) :: :ok | {:conflict, :external_modification}
end
```

**`read_tracked/1`:** Reads the file, records the content hash via
`FreshnessTracker.record/2`, returns the content. Every daemon file read
must use this function (not raw `File.read/1`) to ensure the freshness table
stays current.

**`write_safe/3`:** Encapsulates the full write discipline:

1. If `category == :exclusive` → skip freshness check, atomic write, done.
2. Otherwise → call `FreshnessTracker.check/1`:
   - `:unchanged` → atomic write (temp-then-rename), update stored hash to
     new content's hash, return `:ok`.
   - `{:changed, _}` → external modification detected:
     - `:phase_owned` → write proceeds anyway (daemon-authoritative). Emit
       `%Diagnostic{level: :warn, target: "mnemosyne.coordination", message: "external modification to <path> overwritten by phase cycle"}`.
       Update stored hash. Return `:ok`.
     - `:human_editable` → write is discarded. Update stored hash to the
       external version's hash. Emit
       `%Diagnostic{level: :info, target: "mnemosyne.coordination", message: "external modification to <path> detected; daemon write discarded"}`.
       Return `{:conflict, :external_modification}`.
3. If file doesn't exist (deleted externally):
   - `:phase_owned` → recreate the file, emit `%Diagnostic{level: :warn}`.
   - `:human_editable` → emit `%Diagnostic{level: :warn, message: "file <path> deleted externally; daemon write skipped"}`, return
     `{:conflict, :external_modification}`.

**Atomic writes:** All writes use temp-then-rename:
`File.write!(tmp_path, content)` then `File.rename!(tmp_path, target_path)`.
The tmp file is created in the same directory as the target (same filesystem
mount) to ensure `rename(2)` is atomic.

### §3.4 Integration points

| Consumer | Integration |
|---|---|
| **Sub-B** (`copy_back/2`) | Calls `FileIO.write_safe(path, content, :phase_owned)` for phase-cycle file writes. |
| **Sub-E** (`SafeFileWriter`) | Delegates to `FileIO.write_safe(path, content, :human_editable)` for Tier 1 orphan knowledge writes. |
| **Sub-F** (plan-catalog) | Calls `FileIO.write_safe(path, content, :exclusive)` — no freshness check, just atomic rename. |
| **Sub-N** (expert absorb) | Each `ExpertActor` calls `FileIO.write_safe(path, content, :human_editable)` for Tier 2 knowledge writes. |
| **Sub-M** (observability) | Subscribes to `%Diagnostic{target: "mnemosyne.coordination"}` events via `:telemetry`. |

---

## §4 Vault git concurrency (D3)

### §4.1 V1 constraint

The daemon never performs autonomous git commits. All daemon-triggered git
operations are human-mediated — the user explicitly approves the action
(e.g., accepting a routing rule suggestion in the TUI) before any
`git add` / `git commit` executes.

**Consequence:** vault git concurrency is a non-issue in v1. The user
naturally serializes their own git operations — they will not `git pull`
while simultaneously accepting a rule suggestion.

### §4.2 Git interface

Daemon git operations (when triggered by user approval) use a sequential
executor:

```elixir
defmodule Mnemosyne.VaultGit do
  @spec commit(
          vault_path :: Path.t(),
          paths :: [Path.t()],
          message :: String.t()
        ) :: :ok | {:error, term()}
end
```

No locking beyond git's own `index.lock`. If git's own lock contention
fails the operation (another git process running), the daemon surfaces
`%Diagnostic{level: :error, message: "git operation failed: index.lock held"}`
and does not retry. The user can retry via the TUI.

### §4.3 Reserved hooks for v2+

- `daemon.toml` reserves a `[git]` section (currently empty/undocumented)
  for sub-P's eventual multi-user git sync strategy.
- The `VaultGit` module boundary is the integration point — sub-P replaces
  the sequential executor with a coordination-aware one.

### §4.4 What's explicitly not in v1

- Auto-commit of plan-catalog.md regenerations
- Auto-commit of knowledge file writes
- Background git push/pull sync
- Conflict resolution for concurrent git operations
- Per-commit attribution beyond the default git user

---

## §5 Cross-sub-project contracts

D locks the following contracts for sibling consumption:

| Contract | Consumers |
|---|---|
| System-wide daemon singleton lock at `<runtime_dir>/mnemosyne/daemon.lock` via `flock(2)`, acquired at boot step 3, released on process death. Contains PID + vault path. Not inside the vault — invisible to Obsidian. | F (boot sequence), A (removes `daemon.lock` and `daemon.pid` from vault layout) |
| `Mnemosyne.FileIO.read_tracked/1` and `write_safe/3` as the mandatory file I/O layer for all daemon reads and writes to plan and knowledge files | B (`copy_back/2`), E (`SafeFileWriter`), F (plan-catalog), N (expert absorb writes) |
| `Mnemosyne.Coordination.FreshnessTracker` ETS table keyed by absolute path, recording SHA-256 content hashes | FileIO (internal), M (diagnostic events) |
| Three file categories (`:exclusive`, `:phase_owned`, `:human_editable`) with defined conflict strategies per §3.1 | B, E, F, N |
| `Mnemosyne.VaultGit.commit/3` as the sole daemon git interface, human-mediated only in v1 | F (routing rule suggestions) |
| `%Diagnostic{target: "mnemosyne.coordination"}` events for lock acquire/release/contend and external-modification detect | M (telemetry subscription) |

---

## §6 Cross-sub-project requirements

### On sub-B (phase cycle)

- `copy_back/2` delegates to `Mnemosyne.FileIO.write_safe/3` with category
  `:phase_owned`. For this category, `write_safe` always returns `:ok` —
  the write proceeds even on external modification (daemon-authoritative).
  `copy_back` does not need conflict handling; the diagnostic event emitted
  by `FileIO` is sufficient.
- B consumes the daemon singleton lock as a startup precondition — B does
  not acquire it directly.

### On sub-E (knowledge ingestion)

- `SafeFileWriter` delegates to `Mnemosyne.FileIO.write_safe/3` with
  category `:human_editable`. On `{:conflict, :external_modification}`,
  sub-E treats the candidate as "not written" and emits
  `%Ingestion.CandidateSkipped{reason: :external_modification}`.
- Sub-E's existing atomic temp-then-rename is subsumed by `FileIO` — remove
  the duplicate implementation.

### On sub-A (global store)

- **Applied:** `runtime/daemon.lock` and `runtime/daemon.pid` removed from
  vault layout (§A4). Boot sequence collapsed from 11 to 10 steps. All
  references updated to point to system-wide lock location.

### On sub-F (hierarchy, router, daemon)

- Boot sequence step 3 calls `Mnemosyne.Coordination.SingletonLock.acquire/1`.
- `plan-catalog.md` writes use `FileIO.write_safe/3` with `:exclusive`.
- Routing rule suggestion commits use `VaultGit.commit/3`.
- Bootstrap subcommands read the system-wide lock file (not a vault-local
  pid file) to determine whether to send `:rescan`.

### On sub-N (domain experts)

- Expert absorb writes call `FileIO.write_safe/3` with `:human_editable`.
  On `{:conflict, :external_modification}`, the expert emits
  `%Expert.AbsorbSkipped{reason: :external_modification}` and the candidate
  is not written. The conflict is non-fatal — it means a human edited the
  target path while ingestion was running.

### On sub-M (observability)

- Subscribe to `[:mnemosyne, :coordination, :*]` telemetry events.
- `%Diagnostic{target: "mnemosyne.lock"}` for singleton lock lifecycle.
- `%Diagnostic{target: "mnemosyne.coordination"}` for freshness-check
  conflicts (warn for phase-owned overwrites, info for human-editable
  discards).

---

## §7 Testing strategy

### §7.1 Singleton lock tests

- **Two daemons, same vault:** start two daemon processes against the same
  vault. Assert the second hard-errors with
  `"daemon already running (pid <N>) against vault <path>"`.
- **Stale lock after crash:** start a daemon, `kill -9` it, start a second.
  Assert the second acquires the lock successfully (kernel released `flock`
  on death).
- **Graceful shutdown:** start a daemon, shut it down cleanly. Assert the
  lock file exists but is unlocked (second daemon can start).

### §7.2 Freshness detection tests

- **Phase-owned, external modification:** daemon reads `memory.md`, external
  process modifies it, daemon writes. Assert write succeeds and
  `%Diagnostic{level: :warn}` emitted.
- **Human-editable, external modification:** daemon reads a knowledge file,
  external process modifies it, daemon attempts write. Assert write is
  discarded, `{:conflict, :external_modification}` returned, and
  `%Diagnostic{level: :info}` emitted.
- **Exclusive, external modification:** daemon writes `plan-catalog.md`
  after external modification. Assert write proceeds with no diagnostic (no
  freshness check for exclusive files).
- **No external modification:** daemon reads then writes. Assert
  `:unchanged` detected, write proceeds, no diagnostic.
- **File deleted externally (phase-owned):** daemon reads, file deleted,
  daemon writes. Assert file recreated, `%Diagnostic{level: :warn}` emitted.
- **File deleted externally (human-editable):** daemon reads, file deleted,
  daemon writes. Assert write skipped, `%Diagnostic{level: :warn}` emitted.

### §7.3 Atomic write tests

- **Kill mid-write:** kill the daemon during temp file creation. Assert no
  partial file at the target path (temp file may remain; cleanup is
  best-effort).

### §7.4 VaultGit tests

- **Index.lock contention:** hold `.git/index.lock`, trigger a daemon git
  commit via `VaultGit.commit/3`. Assert `%Diagnostic{level: :error}` and
  no retry.
- **Successful commit:** trigger a commit with no contention. Assert commit
  appears in `git log`.

All tests are pure Elixir + filesystem fixtures. No LLM calls, no external
services.

---

## §8 Implementation size estimate

D is small. The implementation is:

- `Mnemosyne.Coordination.SingletonLock` — ~40 lines (port open, flock,
  PID write, error handling)
- `Mnemosyne.Coordination.FreshnessTracker` — ~50 lines (ETS table, record,
  check, GenServer owner)
- `Mnemosyne.FileIO` — ~80 lines (read_tracked, write_safe with category
  dispatch, atomic temp-then-rename)
- `Mnemosyne.VaultGit` — ~30 lines (commit wrapper, index.lock error
  handling)
- Tests — ~150 lines across the four test modules

Total: ~350 lines of Elixir. A sibling plan is warranted — the
implementation has clear task boundaries and integration points with four
other sub-projects.

---

## Appendix A — Decision Trail

### Q1. flock vs O_CREAT|O_EXCL for singleton lock?

`flock(2)` chosen. `O_CREAT | O_EXCL` (Elixir's `:exclusive` flag) checks
for file existence, not lock state — stale lock files after crashes require
PID-liveness checks and introduce edge cases (PID reuse, race between check
and re-create). `flock` is kernel-enforced and automatically released on
process death regardless of exit reason. No stale-lock cleanup needed.

### Q1a. System-wide lock vs per-vault lock?

System-wide chosen. The lock file lives at `<runtime_dir>/mnemosyne/daemon.lock`
(outside the vault) rather than `<vault>/runtime/daemon.lock`. Two reasons:
(1) a lock file inside the vault is visible in Obsidian's file tree, which
is undesirable; (2) v1 enforces single-daemon-per-machine via sub-A's
single-vault discovery, so a system-wide lock is the correct scope. The lock
file contains both the PID and the vault path being served, so contention
errors are fully informative.

### Q1b. Separate daemon.pid file?

Not needed. The system-wide lock file contains both the PID and the vault
path. Bootstrap subcommands read the lock file to determine whether a daemon
is serving their target vault — PID and vault path in one read.

### Q2. File-system watcher vs content-hash for external-modification detection?

Content-hash (read-before-write) chosen. `:file_system` is already a
dependency from sub-N, but adding a daemon-wide watcher for all
daemon-interesting files is disproportionate to the rare-edge-case frequency
of external-tool conflicts. Content hashing is cheap (`SHA-256` of file
bytes), requires no new dependencies, and catches exactly the case that
matters: "has this file changed since I last read it?"

### Q3. mtime vs content hash for freshness detection?

Content hash chosen per user input. mtime has known pitfalls: coarse
filesystem granularity (some filesystems have 1-second resolution), editors
that preserve mtime, `touch` without content change causing false positives.
SHA-256 detects actual content changes, not metadata changes. Combined with
always-read-before-write, this gives a clean invariant: every daemon write
is preceded by a freshness check, no exceptions.

### Q4. Daemon-authoritative vs human-authoritative for phase-cycle files?

Daemon-authoritative for phase-cycle files, human-authoritative for
non-phase files. The daemon is mid-phase during active cycles; re-reading
externally modified files would break session invariants. For non-phase
files (knowledge, routing rules, expert declarations), the human edit is
the intentional action and the daemon's in-flight write is the one to
discard. This aligns with the practical workflow: humans primarily interact
via the work phase and browse in Obsidian, not edit files concurrently with
daemon writes.

### Q5. Autonomous daemon git commits?

Not in v1. All daemon-triggered git operations are human-mediated. This
makes vault git concurrency a non-issue — the user naturally serializes
their own git operations. Sub-P (v2+) is the place where real git
concurrency lands, likely with CRDT-based sync or a coordination-aware
executor.
