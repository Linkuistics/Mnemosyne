# Configuration Reference

> **Status:** Mnemosyne is in architectural design phase. This document describes the **intended v1 configuration surface** based on the committed architecture. Field names and defaults may adjust during implementation.

Mnemosyne's configuration lives in three files inside the vault:

| File | Purpose |
|---|---|
| `<vault>/mnemosyne.toml` | Vault identity marker and vault-level overrides (owned by sub-A) |
| `<vault>/daemon.toml` | Daemon runtime config: harnesses, routing, logging, extensibility hooks (owned by sub-F) |
| `<vault>/routing.ex` | User-editable declarative routing rules (owned by sub-F) |

This document specifies the first two. For `routing.ex`, see [`user-guide.md#editing-routing-rules`](user-guide.md#editing-routing-rules).

## Contents

1. [mnemosyne.toml — vault identity](#mnemosynetoml--vault-identity)
2. [daemon.toml — daemon runtime](#daemontoml--daemon-runtime)
3. [Environment variables](#environment-variables)
4. [Vault discovery precedence](#vault-discovery-precedence)
5. [Default values](#default-values)

## mnemosyne.toml — vault identity

Lives at `<vault>/mnemosyne.toml`. Its **presence plus a parseable `[vault]` table with a known `schema_version`** is what makes a directory a valid Mnemosyne vault. The daemon checks this on every startup via `verify_vault`.

### Example

```toml
[vault]
schema_version = 1
created_at = "2026-04-14T11:48:52Z"
name = "Mnemosyne development vault"

# Optional: override language profiles (otherwise uses binary-embedded defaults)
[languages.rust]
extensions = ["rs", "toml"]
tags = ["rust", "systems", "memory-safe"]

[languages.elixir]
extensions = ["ex", "exs"]
tags = ["elixir", "beam", "otp", "functional"]

# Optional: override context mappings (binary-embedded defaults are usually fine)
[context.rust_async]
patterns = ["async", "await", "tokio"]
tags = ["rust", "async-patterns"]
```

### Required fields

- `[vault]` table must exist
- `[vault].schema_version` must be an integer matching a version this daemon knows (currently `1`)

Missing file, missing `[vault]` table, missing or unknown schema_version → hard error with actionable diagnostic.

### Optional fields

- `[vault].created_at` — ISO 8601 timestamp of vault creation
- `[vault].name` — human-readable vault label
- `[languages.*]` — language profiles that override the binary-embedded defaults
- `[context.*]` — context detection mappings that override the binary-embedded defaults

Language profiles and context mappings are **ephemeral overrides** — if the daemon encounters an unknown `[languages.foo]` section, it uses `foo` as an additional profile alongside the defaults.

## daemon.toml — daemon runtime

Lives at `<vault>/daemon.toml`. Defaults are reasonable; most users don't need to edit this file. Created at vault init with commented examples.

### Full example

```toml
# ============================================================
# Mnemosyne daemon configuration
# ============================================================

[daemon]
# Unix socket path for client attachment. Defaults to <vault>/runtime/daemon.sock.
socket_path = "/Users/you/Mnemosyne-vault/runtime/daemon.sock"

# Log file path. "stderr" means log to stderr (foreground mode only).
log_path = "/Users/you/Mnemosyne-vault/runtime/daemon.log"
log_level = "info"  # debug | info | warn | error

# Number of tokio-style worker threads for actor message processing.
# Default is to use all available cores; set lower if sharing a machine.
# worker_threads = 4

# Idle timeout before an Active actor transitions back to Dormant.
# Expressed as a duration string.
actor_idle_timeout = "5m"

# Maximum concurrent active actors. Exceeding this means the daemon
# prioritises actors with attached clients or recent activity.
max_active_actors = 20


# ============================================================
# Harness adapters — LLM session spawning backends
# ============================================================
# V1 ships the Claude Code adapter. Sub-O (mixture of models) adds
# more adapters post-v1. The [harnesses.*] section shape is reserved
# and ready; v1 only honors claude-code.

[harnesses.claude-code]
# Path to the claude-code binary. If unset, uses $PATH.
# binary = "/usr/local/bin/claude"

# Tool profile for internal reasoning sessions (fact extraction,
# routing, expert consultation). Restricts what the session can do.
internal_session_tools = ["read", "grep", "glob"]

# Default model for plan sessions (overridable per-actor via the
# model: field in actor declarations — sub-O feature, v1 ignores)
default_model = "claude-opus-4-6"


# ============================================================
# Fact extraction for declarative routing
# ============================================================
# Small cheap LLM pass that extracts concern keywords from message
# bodies before routing rules evaluate them.

[fact_extraction]
harness = "claude-code"
model = "claude-haiku-4-5"
max_topics = 5
timeout_ms = 5000
prompt_template_override = ""  # empty means use embedded default


# ============================================================
# Level 2 routing agent
# ============================================================
# Fresh-context routing agent spawned when declarative rules don't
# decide a cross-project dispatch.

[level2_routing]
harness = "claude-code"
model = "claude-opus-4-6"
timeout_ms = 300000  # 5 minutes
max_retargets = 3    # how many re-dispatches before escalating to human


# ============================================================
# Ingestion pipeline (sub-E)
# ============================================================

[ingestion]
# Whether to auto-fire ingestion after every reflect phase.
auto_fire = true

# Model used for candidate extraction and classification.
model = "claude-sonnet-4-6"

# Threshold for promoting Tier 1 → Tier 2.
promotion_score_threshold = 0.7


# ============================================================
# Observability (sub-M)
# ============================================================

[observability]
# Enable Prometheus metrics endpoint.
prometheus_enabled = false
# prometheus_port = 9090

# Enable telemetry events forwarded to tokio-console-like tooling.
telemetry_console_enabled = false


# ============================================================
# RESERVED sections for post-v1 sub-projects
# ============================================================

# [peers] — reserved for sub-P (team mode, v2+).
# Must be empty in v1. Non-empty entries are a hard error.
# peers = []

# [experts] — reserved for sub-N (domain experts, v1.5+).
# Individual experts are declared as files in <vault>/experts/,
# not in this config. This section holds global expert settings.
# default_retrieval_strategy = "keyword"
```

### Section reference

#### `[daemon]`

| Field | Type | Default | Purpose |
|---|---|---|---|
| `socket_path` | string | `<vault>/runtime/daemon.sock` | Unix socket for client attachment |
| `log_path` | string | `<vault>/runtime/daemon.log` | Log file; `stderr` for foreground mode |
| `log_level` | string | `info` | One of `debug`, `info`, `warn`, `error` |
| `worker_threads` | int | all cores | Tokio-style worker thread count |
| `actor_idle_timeout` | duration | `5m` | Active → Dormant transition after this idle time |
| `max_active_actors` | int | `20` | Soft cap on concurrent active actors |

#### `[harnesses.<name>]`

One subsection per adapter. V1 ships `[harnesses.claude-code]`. Sub-O extends with more.

| Field | Type | Default | Purpose |
|---|---|---|---|
| `binary` | string | (search `$PATH`) | Path to the harness binary |
| `internal_session_tools` | array | `["read", "grep", "glob"]` | Tool profile for internal reasoning sessions |
| `default_model` | string | adapter-specific | Model used when no per-actor override |

**Reserved for sub-O**: additional adapters (`[harnesses.codex]`, `[harnesses.ollama]`, `[harnesses.openai]`, etc.). v1 parses unknown adapter sections and logs a warning.

#### `[fact_extraction]`

Controls the small LLM pass that extracts concern keywords before routing rules fire.

| Field | Type | Default | Purpose |
|---|---|---|---|
| `harness` | string | `claude-code` | Which harness adapter to use |
| `model` | string | `claude-haiku-4-5` | Which model; use cheapest capable |
| `max_topics` | int | `5` | Maximum concerns extracted per message |
| `timeout_ms` | int | `5000` | Hard timeout; exceeding means "no facts" and Level 2 fallback |
| `prompt_template_override` | string | (embedded default) | Override for the extraction prompt |

#### `[level2_routing]`

Controls the cross-project routing agent that fires when declarative rules don't decide.

| Field | Type | Default | Purpose |
|---|---|---|---|
| `harness` | string | `claude-code` | Which harness adapter to use |
| `model` | string | `claude-opus-4-6` | Which model; typically a capable one |
| `timeout_ms` | int | `300000` | Hard timeout; exceeding = treated as rejection |
| `max_retargets` | int | `3` | Re-dispatch budget before escalating to human |

#### `[ingestion]`

Sub-E ingestion pipeline settings.

| Field | Type | Default | Purpose |
|---|---|---|---|
| `auto_fire` | bool | `true` | Run ingestion automatically after reflect phases |
| `model` | string | `claude-sonnet-4-6` | Model for candidate extraction/classification |
| `promotion_score_threshold` | float | `0.7` | Score ≥ threshold promotes Tier 1 → Tier 2 |

#### `[observability]`

Sub-M observability settings.

| Field | Type | Default | Purpose |
|---|---|---|---|
| `prometheus_enabled` | bool | `false` | Enable Prometheus metrics endpoint |
| `prometheus_port` | int | `9090` | Metrics endpoint port |
| `telemetry_console_enabled` | bool | `false` | Enable structured telemetry output |

#### `[peers]` (reserved for sub-P, v2+)

Reserved for team mode. V1 hard-errors if this section contains any non-empty entries with a message directing the user to wait for v2.

#### `[experts]` (reserved for sub-N, v1.5+)

Reserved for expert-related global settings. Individual expert declarations live as files in `<vault>/experts/`, not in this config. Sub-N's brainstorm will decide what goes here.

## Environment variables

Most configuration lives in `daemon.toml`. A small number of environment variables affect startup.

| Variable | Purpose |
|---|---|
| `MNEMOSYNE_VAULT` | Override vault discovery; takes precedence over config-file lookup |
| `MNEMOSYNE_TIER1_ROOT` | Test fixture override for per-project knowledge root |
| `MNEMOSYNE_TIER2_ROOT` | Test fixture override for global knowledge root |
| `MNEMOSYNE_LOG_LEVEL` | Override `daemon.log_level` (useful for debugging startup) |
| `MNEMOSYNE_NO_COLOR` | Disable ANSI colors in logs (detected automatically in non-TTY) |

## Vault discovery precedence

When the daemon starts, it resolves the vault via this precedence chain (sub-A's discipline):

1. **`--vault <path>` command-line flag** — highest precedence
2. **`MNEMOSYNE_VAULT` environment variable**
3. **User config at `<config-dir>/mnemosyne/config.toml`** (standard platform config dir: `~/.config/mnemosyne/config.toml` on Linux/macOS, `%APPDATA%\mnemosyne\config.toml` on Windows)
4. **Hard error** — no default vault discovery, no walk-up, no implicit dev-root

The config file form is trivial:

```toml
[vault]
path = "/Users/you/Mnemosyne-vault"
```

This explicit-chain design is deliberate: no "find my vault" magic that can guess wrong. The user always knows where their vault is.

## Default values

All default values are **embedded in the binary** and used when the corresponding config field is missing. Language profiles, context mappings, prompt templates — everything has an embedded default. Configuration files only need to specify overrides.

This means:

- A minimal `mnemosyne.toml` is just `[vault]\nschema_version = 1`
- A minimal `daemon.toml` can be empty
- A vault with neither file fails vault discovery with an actionable error

Users only edit these files when they want to deviate from defaults.

---

## Further reading

- [Architecture overview](architecture.md) — how configuration fits into the larger picture
- [User guide](user-guide.md) — daily workflow referencing these config files
- [Sub-A design doc](superpowers/specs/2026-04-13-sub-A-global-store-design.md) — vault discovery, identity marker, config precedence details
- [Sub-F design doc](superpowers/specs/2026-04-14-sub-F-hierarchy-design.md) — daemon.toml schema and reserved extensibility sections
