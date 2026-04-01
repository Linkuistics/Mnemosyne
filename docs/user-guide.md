# Mnemosyne User Guide

This guide walks through everything you need to use Mnemosyne effectively, from installation through daily workflow to multi-machine sync.

## Contents

- [Installation](#installation)
- [First-Time Setup](#first-time-setup)
- [Daily Workflow](#daily-workflow)
- [Promoting Learnings to Global](#promoting-learnings-to-global)
- [Querying Global Knowledge](#querying-global-knowledge)
- [Status Overview](#status-overview)
- [Curation Sessions](#curation-sessions)
- [Exploration Sessions](#exploration-sessions)
- [Multi-Machine Sync](#multi-machine-sync)
- [Claude Code Plugin Setup](#claude-code-plugin-setup)

---

## Installation

Mnemosyne is a Rust CLI. Build and install it from source:

```bash
git clone https://github.com/Linkuistics/Mnemosyne
cd mnemosyne
cargo install --path .
```

Or install directly from the repository:

```bash
cargo install --git https://github.com/Linkuistics/Mnemosyne
```

Verify the installation:

```bash
mnemosyne --version
# mnemosyne 0.1.0
```

### Prerequisites

- Rust toolchain (stable, 1.75 or later) — install via [rustup.rs](https://rustup.rs)
- Git — for the version-controlled knowledge store and multi-machine sync

---

## First-Time Setup

### Create the Knowledge Store

```bash
mnemosyne init
```

This creates `~/.mnemosyne/` with the following structure:

```
~/.mnemosyne/
├── config.yml          # Language profiles and context mappings
├── .gitignore          # Excludes cache/
├── knowledge/
│   ├── languages/      # Rust, Python, Swift, etc.
│   ├── domains/        # macOS, web, databases, etc.
│   ├── tools/          # Cargo, Git, Docker, etc.
│   ├── techniques/     # TDD, async patterns, error handling, etc.
│   └── projects/       # Per-project summaries and cross-references
├── archive/            # Pruned entries (preserved, not deleted)
└── cache/              # Derived data (gitignored, rebuilt automatically)
```

`mnemosyne init` also initialises a Git repository in `~/.mnemosyne/` and makes an initial commit. This enables version history and multi-machine sync.

### Clone an Existing Knowledge Base

If you already have a Mnemosyne knowledge base hosted as a Git repository (for example, from another machine), clone it instead:

```bash
mnemosyne init --from git@github.com:you/mnemosyne-knowledge.git
```

This replaces the fresh-init with a `git clone`. Your existing knowledge, config, and history are preserved.

---

## Daily Workflow

Mnemosyne integrates with the observational-memory Claude Code plugin. The core cycle is:

1. **Begin work** — `/begin-work <context>` loads per-project knowledge and queries global knowledge
2. **Implement** — follow the Do → Verify → Observe cycle; record observations in your plan
3. **Reflect** — `/reflect` promotes per-project observations to per-project knowledge, and optionally to global

### Beginning a Session

When you run `/begin-work`, the plugin:

1. Loads per-project knowledge files
2. Checks whether `mnemosyne` is installed (`which mnemosyne`)
3. If installed, runs `mnemosyne query --context --format markdown`
4. Displays global knowledge entries relevant to the current project alongside per-project knowledge

The `--context` flag instructs the CLI to infer tags from the current directory — detecting languages, dependency files, and project name — rather than requiring you to specify search terms. This makes global knowledge surface automatically without any extra effort.

If `mnemosyne` is not installed, the plugin proceeds without it. No errors, no warnings — global knowledge is simply absent from the session.

### Recording Observations

During implementation, record observations in your plan file using the priority codes:

- `🔴` — critical: must be promoted; indicates something that will cause failures if ignored
- `🟡` — useful: should be promoted; useful patterns and anti-patterns
- `🟢` — informational: consider promoting; context and background
- `⚪` — neutral: fleeting notes, probably not worth promoting

### Reflecting at Review Points

Run `/reflect` after completing a block of work:

```
/reflect
```

The plugin will:
1. Identify un-promoted observations in the current plan
2. Suggest which per-project knowledge file each belongs in
3. Promote observations to per-project knowledge on your confirmation
4. Offer to promote each learning further to global knowledge via `mnemosyne promote`

---

## Promoting Learnings to Global

Promotion is the pathway from raw observation to durable global knowledge.

### Via `/reflect` (Recommended)

After promoting an observation to per-project knowledge, the plugin asks:

```
This learning may apply beyond this project. Promote to global? [y/N]
```

Answering `y` runs `mnemosyne promote` interactively.

### Via the CLI Directly

Run promotion at any time from any project directory:

```bash
mnemosyne promote --tags rust,async,tokio --origin my-project
```

The CLI prompts you for:

1. **Title** — a concise, descriptive title for the knowledge entry
2. **Tags** — if not provided via `--tags`, prompted interactively (comma-separated)
3. **Origin project** — if not provided via `--origin`, prompted interactively
4. **Content** — the knowledge body; type your text and end with a blank line

### Contradiction Detection

Before saving, the CLI checks existing entries for potential contradictions using Jaccard tag-overlap scoring. An entry with 50% or more tag overlap with your new entry is flagged:

```
⚠ Potential contradictions detected:
  Rust async channel patterns (overlap: 62%)

[s]upersede  [c]oexist  [d]iscard  [r]efine
```

Your options:

- **`s` — Supersede**: Your new knowledge replaces the old. The old content is moved to a `## Superseded` section in the file with a reason and date range. The file's history is preserved; nothing is deleted.
- **`c` — Coexist**: Both entries are valid in different contexts. The promotion proceeds; add scope notes to distinguish them.
- **`d` — Discard**: The new learning is project-specific or incorrect. Promotion is cancelled.
- **`r` — Refine**: Pause to refine your wording before continuing.

### Where Entries Are Saved

The CLI infers the appropriate axis from the entry's tags:

| Tags include | Saved under |
|-------------|-------------|
| A language name (rust, python, swift, …) | `knowledge/languages/` |
| A tool name (cargo, git, docker, …) | `knowledge/tools/` |
| A domain name (macos, web, database, …) | `knowledge/domains/` |
| Anything else | `knowledge/techniques/` |

The filename is derived from the title by lowercasing, replacing non-alphanumeric characters with hyphens, and appending `.md`.

---

## Querying Global Knowledge

### Basic Query

Search by keyword:

```bash
mnemosyne query async channels
```

Multiple terms are joined with AND-like matching against title and body text.

### Context-Inferred Query

Let the CLI infer what's relevant from the current project directory:

```bash
mnemosyne query --context
```

The CLI detects:
- Language markers (e.g., `Cargo.toml` → `rust`)
- Dependency files (e.g., `tokio` in `Cargo.toml` → tags `async`, `tokio`, `concurrency`)
- Project name (from git remote or directory name)

It maps detected signals to tags via the `context_mappings` configuration and retrieves all matching entries.

### Output Formats

```bash
# Default: markdown (suitable for LLM context loading)
mnemosyne query async --format markdown

# Structured JSON (suitable for programmatic processing)
mnemosyne query async --format json

# Single-line summary per entry (suitable for quick scanning)
mnemosyne query async --format plain
```

### Token Budget

Limit the output to fit within a token budget (the CLI approximates 500 tokens per result):

```bash
mnemosyne query --context --max-tokens 4000
```

This returns at most 8 results when using the 500-token-per-result approximation. Useful when injecting results into an LLM context window with a known limit.

---

## Status Overview

Get a summary of your knowledge base:

```bash
mnemosyne status
```

Example output:

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

Use `mnemosyne status` to get a sense of where your knowledge is concentrated and where gaps may exist.

---

## Curation Sessions

Curation is a reflective review practice, not automated maintenance. Run it after completing a major project, when you sense your understanding has shifted, or whenever you want to deliberately consolidate what you've learned.

```bash
mnemosyne curate
```

The session proceeds in three phases:

### Phase 1: Divergence Review

The CLI identifies entries where recent project activity (last 90 days) may contradict global knowledge. These are entries with high tag overlap between global entries and recent per-project observations from projects not listed in the entry's origins.

```
Entries with potential divergence:
  Rust async channel patterns — 2 diverging projects: api-server, cli-tool
```

Divergence is an implicit signal that global knowledge may be outdated or too narrow. Reviewing these first focuses curation energy where it matters most.

### Phase 2: Active Area Summary

The CLI identifies your most active knowledge areas based on recently validated entries:

```
Areas of recent activity: rust (12), async (8), tokio (7), error-handling (5), testing (4)
```

### Phase 3: Interactive Review

For each entry selected for review (divergent entries first; otherwise entries in active areas), you see:

```
1. Rust async channel patterns [high] tags: rust, async, tokio, channels
   Last validated: 2025-11-03

   [v]alidate  [s]upersede  [r]efine  [p]rune  [n]ext
   >
```

Your choices:

- **`v` — Validate**: The entry still holds. Updates `last_validated` to today.
- **`s` — Supersede**: Your understanding has changed. The old content moves to a `## Superseded` section inline.
- **`r` — Refine**: Edit the entry to sharpen the wording or add nuance.
- **`p` — Prune**: The entry is no longer applicable. It moves to `archive/` with your reason. It is preserved, not deleted.
- **`n` — Next**: Skip this entry for now.

After curation, consider committing the changes to keep your Git history clean:

```bash
cd ~/.mnemosyne && git add -A && git commit -m "Curation session $(date +%Y-%m-%d)"
```

---

## Exploration Sessions

Exploration actively grows the knowledge base. Where curation reviews what you have, exploration identifies what's missing and generates new knowledge.

```bash
mnemosyne explore
```

### Gap Analysis

The session opens with a gap analysis:

```
Gap Analysis

  • You have knowledge tagged 'swift' but only 2 entries — could be expanded
  • 8 tags appear in only 1 entry — consider expanding or consolidating
  • Knowledge entries are sparse relative to the number of projects — consider promoting more learnings
```

Gaps are detected by examining tag distribution across entries: sparse language coverage, singleton tags, and low entry-to-project ratios are all flagged.

### Open Questions

Next, the session surfaces entries with `low` or `prospective` confidence:

```
Open Questions / Prospective Knowledge

  • Swift concurrency model [prospective]
  • OCaml multicore runtime [low confidence]
```

These are areas where you have awareness but not validated experience — candidates for deliberate study or experimentation.

### Tag Clusters

Finally, the session identifies tag pairs that appear together frequently, suggesting they might benefit from synthesis into a unified entry:

```
Tag Clusters (may benefit from synthesis)

  • rust + async — 7 entries
  • testing + error-handling — 4 entries
```

### Interactive Exploration

After the analysis, you can explore any topic interactively:

```
Would you like to explore any of these areas? (Enter a topic, or 'q' to quit)
> swift concurrency

Tell me about your experience with 'swift concurrency':
(Type your thoughts, end with an empty line)

> Structured concurrency with async/await in Swift 5.5+ dramatically reduces
> callback pyramid of doom. Task groups let you express parallel work declarably.
> Actor isolation catches data races at compile time — far better than runtime.
>

Suggested tags for this knowledge:
  swift, concurrency

Save as [h]igh, [m]edium, [l]ow, or [p]rospective confidence? (or [d]iscard)
> h

✓ Saved to knowledge/languages/swift-concurrency.md
```

---

## Multi-Machine Sync

`~/.mnemosyne/` is a standard Git repository. Sync between machines using standard Git operations.

### Setting Up a Remote

Create a private repository on GitHub, GitLab, or any Git host, then add it as a remote:

```bash
cd ~/.mnemosyne
git remote add origin git@github.com:you/mnemosyne-knowledge.git
git push -u origin main
```

### Syncing

On machine A after a curation or exploration session:

```bash
cd ~/.mnemosyne && git add -A && git commit -m "Add learnings from project X" && git push
```

On machine B before starting work:

```bash
cd ~/.mnemosyne && git pull
```

### Conflict Resolution

If you work on two machines without syncing and both make commits, you may encounter merge conflicts. Knowledge files use a YAML + Markdown format that is human-readable — conflicts can be resolved manually in any text editor.

The `cache/` directory is gitignored and rebuilt automatically, so it never causes conflicts.

---

## Claude Code Plugin Setup

The Claude Code plugin integrates Mnemosyne into your per-project observational-memory workflow.

### Installation

```bash
mnemosyne install claude-code
```

This copies the plugin files to `~/.claude/plugins/observational-memory/`. The plugin provides:

**Skills (slash commands):**

| Command | Description |
|---------|-------------|
| `/begin-work <context>` | Start or continue work with full knowledge context including global |
| `/reflect` | Promote plan observations to knowledge, with global promotion offer |
| `/promote-global` | Ad-hoc promotion of a specific learning to global |
| `/curate-global` | Run a curation session from within Claude Code |
| `/explore-knowledge` | Run an exploration session from within Claude Code |
| `/create-plan` | Create a new implementation plan |
| `/setup-knowledge` | Initialise the per-project knowledge structure |

**References (context documents):**

| Document | Purpose |
|----------|---------|
| `global-knowledge-guide.md` | Overview of the two-tier model and global knowledge format |
| `observational-memory-guide.md` | Per-project observational memory workflow |
| `plan-format.md` | Plan file format specification |
| `coding-conventions.md` | Project-level coding conventions template |

### Verifying the Installation

After installation, open a Claude Code session in any project and run:

```
/begin-work test
```

If Mnemosyne is installed and initialised, you will see a "Global knowledge loaded" section in the output. If Mnemosyne is not installed or the knowledge base is empty, the section is omitted — the plugin degrades gracefully.
