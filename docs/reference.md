# Mnemosyne CLI Reference

Complete reference for all `mnemosyne` commands, flags, and expected output.

## Contents

- [mnemosyne init](#mnemosyne-init)
- [mnemosyne query](#mnemosyne-query)
- [mnemosyne promote](#mnemosyne-promote)
- [mnemosyne status](#mnemosyne-status)
- [mnemosyne curate](#mnemosyne-curate)
- [mnemosyne explore](#mnemosyne-explore)
- [mnemosyne install](#mnemosyne-install)

---

## mnemosyne init

Create `~/.mnemosyne/` with the default knowledge store structure.

### Synopsis

```
mnemosyne init [--from <url>]
```

### Description

`init` creates the knowledge store at `~/.mnemosyne/`. When called without arguments, it creates a fresh store with default configuration, the five axis directories, an `archive/` directory, a `cache/` directory, and a `.gitignore` excluding `cache/`. It then initialises a Git repository and makes an initial commit.

When `--from` is provided, it clones the specified Git repository instead of creating a fresh store. This is the recommended path when setting up Mnemosyne on a new machine where a knowledge base already exists.

`init` fails if `~/.mnemosyne/` already exists.

### Flags

| Flag | Description |
|------|-------------|
| `--from <url>` | Clone an existing knowledge repository instead of creating a fresh store. Accepts any URL valid for `git clone`. |

### Examples

```bash
# Create a fresh knowledge store
mnemosyne init

# Clone an existing knowledge base from GitHub
mnemosyne init --from git@github.com:you/mnemosyne-knowledge.git

# Clone from a local path (useful for migrations or testing)
mnemosyne init --from /path/to/existing/mnemosyne
```

### Expected Output

```
Mnemosyne initialized at /Users/you/.mnemosyne
```

On failure (store already exists):

```
Error: Directory already exists: /Users/you/.mnemosyne. Use a different path or remove it first.
```

---

## mnemosyne query

Search the global knowledge base.

### Synopsis

```
mnemosyne query [<terms>...] [--context] [--format <format>] [--max-tokens <n>]
```

### Description

`query` searches the knowledge base and returns formatted results. Two modes are supported:

**Term mode** — explicitly provide search terms. The CLI matches terms against entry titles and body text.

**Context mode** (`--context`) — infer search terms from the current directory. The CLI detects language markers, dependency files, and project name, maps them to tags via `context_mappings` in `config.yml`, and retrieves entries matching those tags. This is the primary mode used by the Claude Code plugin.

`--context` and positional terms are mutually exclusive. If both are provided, `--context` takes precedence and the terms are ignored.

Results are returned in descending relevance order. The `--max-tokens` flag limits the number of results by approximating 500 tokens per entry.

### Arguments

| Argument | Description |
|----------|-------------|
| `<terms>...` | One or more search terms. Multiple terms are joined and matched against title and body. Optional if `--context` is used. |

### Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--context` | off | Infer search tags from the current project directory. |
| `--format <format>` | `markdown` | Output format: `markdown`, `json`, or `plain`. |
| `--max-tokens <n>` | — | Limit output by approximate token count. Each result is estimated at 500 tokens. If omitted, returns up to 10 results. |

### Output Formats

**`markdown`** — Suitable for loading into LLM context. Each result is an H2 heading followed by tags and body:

```markdown
## Rust Async Channel Patterns (high)
Tags: rust, async, tokio, channels

## Bounded channels prevent backpressure

**2026-01-15:** Always use bounded channels in tokio. Unbounded channels
caused memory exhaustion under sustained load in the api-server project.

---
```

**`json`** — Structured array, suitable for programmatic processing:

```json
[
  {
    "title": "Rust Async Channel Patterns",
    "tags": ["rust", "async", "tokio", "channels"],
    "confidence": "high",
    "body": "## Bounded channels prevent backpressure\n\n..."
  }
]
```

**`plain`** — One line per result, suitable for quick scanning or shell scripting:

```
title: Rust Async Channel Patterns | confidence: high | tags: rust, async, tokio, channels
```

### Examples

```bash
# Search by keyword
mnemosyne query async error handling

# Infer context from current project
mnemosyne query --context

# Context query with token budget for injection into a 4K context window
mnemosyne query --context --max-tokens 4000

# JSON output for programmatic processing
mnemosyne query --context --format json

# Plain output for shell scripting
mnemosyne query rust --format plain

# Combine context with token limit
mnemosyne query --context --format markdown --max-tokens 8000
```

### Expected Output (no results)

```
No matching knowledge found.
```

---

## mnemosyne promote

Promote a learning to global knowledge.

### Synopsis

```
mnemosyne promote [--tags <tags>] [--origin <project>]
```

### Description

`promote` is an interactive session that guides you through creating a new global knowledge entry. It prompts for the entry's title, tags, origin project, and body content. After collection, it runs contradiction detection against existing entries with overlapping tags before saving.

The CLI infers the appropriate axis from the entry's tags and generates a filename from the title.

If tags or origin project are provided as flags, those steps are skipped in the interactive session.

### Flags

| Flag | Description |
|------|-------------|
| `--tags <tags>` | Comma-separated tags for the entry. If omitted, prompted interactively. |
| `--origin <project>` | Name of the origin project. If omitted, prompted interactively. |

### Interactive Session

```
Mnemosyne — Promote to Global Knowledge

Title for this knowledge entry:
> Rust async channel patterns

Tags (comma-separated):
> rust, async, tokio, channels

Origin project:
> api-server

Knowledge content (end with empty line):
> Always use bounded channels in tokio. Unbounded channels can cause
> memory exhaustion under sustained load. The bound should be sized
> to the expected burst, not set arbitrarily large.
>
```

If contradictions are detected:

```
⚠ Potential contradictions detected:
  Tokio concurrency patterns (overlap: 62%)
  Async channel best practices (overlap: 58%)

[s]upersede  [c]oexist  [d]iscard  [r]efine
> s
```

### Contradiction Resolution Options

| Option | Behaviour |
|--------|-----------|
| `s` — Supersede | Saves the new entry and moves the old entry's body to a `## Superseded` section inline. Old content is preserved with date range. |
| `c` — Coexist | Saves the new entry without modifying the existing one. Both are valid. |
| `d` — Discard | Cancels promotion. Nothing is saved. |
| `r` — Refine | Cancels promotion so you can revise your entry before trying again. |

### Expected Output (success)

```
✓ Promoted to knowledge/languages/rust-async-channel-patterns.md
```

### Axis Inference Rules

The CLI chooses an axis based on tag content in this priority order:

1. Any recognised language tag (rust, python, swift, haskell, etc.) → `languages/`
2. Any recognised tool tag (cargo, git, docker, npm, etc.) → `tools/`
3. Any recognised domain tag (macos, web, database, networking, etc.) → `domains/`
4. Anything else → `techniques/`

---

## mnemosyne status

Display a summary of the knowledge base.

### Synopsis

```
mnemosyne status
```

### Description

`status` loads all entries from the knowledge store and displays summary statistics: total entries, breakdown by axis, breakdown by confidence level, and the unique set of origin projects.

### Expected Output

```
Mnemosyne Knowledge Base

Location: /Users/you/.mnemosyne
Total entries: 47

Entries by axis:
  domains: 8
  languages: 19
  techniques: 14
  tools: 6

Entries by confidence:
  high: 23
  medium: 17
  low: 5
  prospective: 2

Origin projects:
  api-server
  cli-tool
  mnemosyne
  web-app
```

If no entries exist yet:

```
Mnemosyne Knowledge Base

Location: /Users/you/.mnemosyne
Total entries: 0

Entries by axis:

Entries by confidence:

Origin projects:
```

---

## mnemosyne curate

Run a reflective curation session.

### Synopsis

```
mnemosyne curate
```

### Description

`curate` is an interactive session for reviewing and maintaining the quality of the knowledge base. It does not accept flags; the session is entirely interactive.

The session opens with divergence detection (recent entries from new projects that overlap with global entries), displays active knowledge areas, then walks through relevant entries one at a time.

See the [User Guide — Curation Sessions](user-guide.md#curation-sessions) for a full description of the session flow.

### Interactive Choices Per Entry

| Key | Action |
|-----|--------|
| `v` | Validate — updates `last_validated` to today |
| `s` | Supersede — records old content in a `## Superseded` section |
| `r` | Refine — mark for editing (currently logs intent) |
| `p` | Prune — archives entry with reason |
| `n` | Next — skips entry |

### Expected Output (session end)

```
Curation session complete.
```

---

## mnemosyne explore

Run an interactive knowledge exploration session.

### Synopsis

```
mnemosyne explore
```

### Description

`explore` is an interactive session for growing the knowledge base. It performs gap analysis (thin or missing areas), surfaces open questions and prospective entries, identifies tag clusters that may benefit from synthesis, then invites you to explore any area and optionally save new knowledge.

See the [User Guide — Exploration Sessions](user-guide.md#exploration-sessions) for a full description of the session flow.

### Gap Detection Heuristics

The CLI uses three heuristics to identify gaps:

1. **Sparse language coverage** — a detected language has fewer than 3 entries
2. **Singleton tags** — more than 3 tags appear in only one entry
3. **Low entry density** — entry count is less than twice the number of origin projects

### Expected Output (session end)

```
Exploration session complete.
```

---

## mnemosyne install

Install an adapter plugin.

### Synopsis

```
mnemosyne install <adapter>
```

### Description

`install` copies the named adapter's plugin files to the appropriate location for the target editor or tool. Currently the only supported adapter is `claude-code`.

### Arguments

| Argument | Description |
|----------|-------------|
| `<adapter>` | Adapter to install. Currently only `claude-code` is supported. |

### Adapters

**`claude-code`**

Installs the Claude Code observational-memory plugin to `~/.claude/plugins/observational-memory/`. This includes skills (slash commands) for `/begin-work`, `/reflect`, `/promote-global`, `/curate-global`, `/explore-knowledge`, `/create-plan`, and `/setup-knowledge`, plus reference documents for global knowledge, observational memory, plan format, and coding conventions.

The install command must be run from the Mnemosyne repository directory or from a location where the adapter files are available alongside the binary.

### Examples

```bash
# Install the Claude Code plugin
mnemosyne install claude-code
```

### Expected Output

```
✓ Claude Code plugin installed to /Users/you/.claude/plugins/observational-memory
```

On failure (unknown adapter):

```
Unknown adapter: neovim. Available: claude-code
```

On failure (adapter files not found):

```
Error: Could not find adapter files. Run from the Mnemosyne repo directory.
```
