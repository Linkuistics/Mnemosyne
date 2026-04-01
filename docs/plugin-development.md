# Plugin Development Guide

This guide explains how to build a Mnemosyne adapter for a new editor, IDE, or AI tool. The Claude Code adapter is the reference implementation.

## Contents

- [What Adapters Do](#what-adapters-do)
- [What the CLI Provides](#what-the-cli-provides)
- [The Graceful Degradation Pattern](#the-graceful-degradation-pattern)
- [Shelling Out to the CLI](#shelling-out-to-the-cli)
- [Context Loading Integration](#context-loading-integration)
- [Output Format Parsing](#output-format-parsing)
- [Installing an Adapter](#installing-an-adapter)
- [Building a Minimal Adapter](#building-a-minimal-adapter)
- [Adapter File Structure](#adapter-file-structure)

---

## What Adapters Do

An adapter integrates Mnemosyne's global knowledge capabilities into a specific tool's plugin or extension system. The adapter's responsibilities are:

1. **Check availability** — determine whether `mnemosyne` is installed before invoking it
2. **Query on context load** — run `mnemosyne query --context` when the user begins a session, injecting results into the tool's context
3. **Offer promotion pathways** — when the user records a significant learning, offer to run `mnemosyne promote` to promote it globally
4. **Surface curation and exploration** — provide user-facing commands that delegate to `mnemosyne curate` and `mnemosyne explore`
5. **Degrade gracefully** — if `mnemosyne` is not installed, continue working without it; no errors, no warnings to the user

What adapters do NOT do:
- Manage the knowledge store directly (that is the CLI's job)
- Implement their own knowledge format parsers
- Call the Mnemosyne library (Rust crate) directly — only the CLI binary

All adapter-to-Mnemosyne communication is through the CLI's stdout.

---

## What the CLI Provides

The CLI is the sole interface between adapters and the knowledge store. Relevant commands for adapters:

| Command | Output | Use case |
|---------|--------|---------|
| `mnemosyne query [terms] [--context] [--format F] [--max-tokens N]` | Formatted knowledge | Context loading at session start |
| `mnemosyne promote [--tags T] [--origin P]` | Interactive session | Global promotion after per-project promotion |
| `mnemosyne status` | Summary text | Dashboard display or health check |
| `mnemosyne curate` | Interactive session | User-initiated curation |
| `mnemosyne explore` | Interactive session | User-initiated exploration |

The CLI writes results to stdout. Interactive sessions (`promote`, `curate`, `explore`) read from stdin and write prompts to stdout — they require a connected terminal.

---

## The Graceful Degradation Pattern

Every adapter must implement graceful degradation: all Mnemosyne-dependent features must be silently skipped if the CLI is not installed.

### Availability Check

Check for the CLI binary before any invocation:

**Shell (bash/zsh):**
```bash
if command -v mnemosyne > /dev/null 2>&1; then
    # mnemosyne is available
fi
```

**In a Claude Code skill (Markdown):**
```markdown
Check whether `mnemosyne` is installed:

```bash
which mnemosyne
```

If found, run:
```bash
mnemosyne query --context --format markdown
```

If the command fails or produces no output, skip silently.
```

**In a JavaScript/TypeScript plugin:**
```typescript
import { execSync } from 'child_process';

function isMnemosyneAvailable(): boolean {
  try {
    execSync('which mnemosyne', { stdio: 'pipe' });
    return true;
  } catch {
    return false;
  }
}
```

**In a Python plugin:**
```python
import shutil

def is_mnemosyne_available() -> bool:
    return shutil.which('mnemosyne') is not None
```

### What "Graceful" Means

- Do not show error messages, warnings, or "Mnemosyne not found" notices to the user
- Do not change the layout of your plugin's output based on Mnemosyne's absence
- Simply omit the sections that Mnemosyne would have populated
- Do not retry, poll, or cache the availability check for the session (re-check on each invocation — the user may install Mnemosyne mid-session)

The rationale: users who have not installed Mnemosyne should get a seamless experience with the per-project features, not constant reminders about a tool they may not want.

---

## Shelling Out to the CLI

### Working Directory

The CLI uses the working directory for context detection (`query --context`). Always invoke the CLI with the working directory set to the project root.

**Shell:**
```bash
cd /path/to/project && mnemosyne query --context --format markdown
```

**Node.js:**
```typescript
const result = execSync('mnemosyne query --context --format markdown', {
  cwd: projectRoot,
  encoding: 'utf8'
});
```

**Python:**
```python
import subprocess
result = subprocess.run(
    ['mnemosyne', 'query', '--context', '--format', 'markdown'],
    cwd=project_root,
    capture_output=True,
    text=True
)
```

### Error Handling

CLI invocations may fail for various reasons (misconfigured store, empty knowledge base, parse errors). Handle failures by silently discarding the output and continuing without global knowledge:

```typescript
function queryGlobalKnowledge(projectRoot: string): string | null {
  if (!isMnemosyneAvailable()) return null;
  try {
    const output = execSync(
      'mnemosyne query --context --format markdown',
      { cwd: projectRoot, encoding: 'utf8', timeout: 5000 }
    );
    return output.trim() || null;
  } catch {
    return null;  // silently degrade
  }
}
```

### Timeout

Apply a reasonable timeout (5 seconds is a good default) to prevent CLI invocations from blocking the editor or tool. The CLI scans the filesystem during context detection and may be slow on very large projects or network-mounted filesystems.

---

## Context Loading Integration

Context loading is the primary use case for adapters: injecting relevant global knowledge into the LLM's context window at the start of a work session.

### When to Load Context

Load global knowledge:
- When the user begins a session or opens a project (`/begin-work` equivalent)
- When the user explicitly requests it
- Not on every message (too expensive in tokens and latency)

### Token Budget Management

Use `--max-tokens` to keep the injection size within the LLM's context window budget:

```bash
mnemosyne query --context --format markdown --max-tokens 4000
```

The CLI approximates 500 tokens per result and limits accordingly. Adjust the budget based on:
- The LLM's context window size
- How much other context you're injecting (per-project knowledge, conversation history)
- The user's preferences

For a 32K context window with significant per-project knowledge, `--max-tokens 4000` is a reasonable starting point.

### Presenting the Results

When injecting into an LLM context, wrap the output in a clear section heading so the model understands its provenance:

```markdown
### Global knowledge loaded

<mnemosyne query output here>
```

If the query returns no results or fails, omit the section entirely.

---

## Output Format Parsing

### Markdown

The default format. Use it when injecting into LLM context. Each result is an H2 section:

```markdown
## <title> (<confidence>)
Tags: <comma-separated tags>

<body>

---
```

No parsing required — inject directly into context.

### JSON

Use JSON when you need to process results programmatically (display in a UI, filter, deduplicate against per-project knowledge):

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

Parse with any JSON library. The `body` field contains the full Markdown body of the entry.

### Plain

One line per result. Use for quick scanning in terminal contexts:

```
title: Rust Async Channel Patterns | confidence: high | tags: rust, async, tokio, channels
```

---

## Installing an Adapter

When a user runs `mnemosyne install <adapter-name>`, the CLI copies the adapter's files from the repository's `adapters/<adapter-name>/` directory to the tool-specific location.

### Registering Your Adapter

To register a new adapter with the CLI's `install` command, add a match arm in `src/main.rs`:

```rust
"your-tool" => {
    let plugin_target = dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".your-tool/plugins/mnemosyne");

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let source_candidates = [
        exe_dir.as_ref().map(|d| d.join("../adapters/your-tool")),
        Some(std::path::PathBuf::from("adapters/your-tool")),
    ];

    let source = source_candidates
        .iter()
        .flatten()
        .find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!(
            "Could not find adapter files. Run from the Mnemosyne repo directory."
        ))?;

    commands::install::run_install_claude_code(source, &plugin_target)?;
    println!("✓ Mnemosyne adapter installed to {}", plugin_target.display());
}
```

The `run_install_claude_code` function performs a recursive directory copy; it works for any adapter.

---

## Building a Minimal Adapter

Here is a complete example of a minimal adapter for a hypothetical tool that supports Markdown-based skills.

### Skill: `mnemosyne-context.md`

This skill queries global knowledge and injects it into the session:

```markdown
---
name: mnemosyne-context
description: Load relevant global knowledge from Mnemosyne. Usage: /mnemosyne-context
---

Load global knowledge from Mnemosyne if available.

## Check availability

```bash
which mnemosyne
```

If mnemosyne is not installed, skip this skill entirely — do not report any errors.

## Query global knowledge

Run in the current project directory:

```bash
mnemosyne query --context --format markdown --max-tokens 4000
```

## Present results

If the command succeeds and produces output, display it under the heading:

### Global knowledge loaded

<output from mnemosyne query>

If the command fails or produces no output, omit the section entirely.
```

### Skill: `mnemosyne-promote.md`

This skill promotes a learning to global:

```markdown
---
name: mnemosyne-promote
description: Promote a learning to Mnemosyne global knowledge. Usage: /mnemosyne-promote
---

Promote the most recent significant learning to Mnemosyne global knowledge.

## Check availability

```bash
which mnemosyne
```

If mnemosyne is not installed, inform the user that global promotion requires
the Mnemosyne CLI and stop.

## Prepare for promotion

Ask the user:
1. What is the learning? (1-2 sentences, transferable to other projects)
2. What tags best describe it? (comma-separated)

## Run promotion

```bash
mnemosyne promote --tags <tags> --origin <project-name>
```

This starts an interactive session. The CLI will prompt for the title and
content. Contradiction detection runs automatically before saving.
```

---

## Adapter File Structure

The reference structure for an adapter in the `adapters/` directory:

```
adapters/
└── your-tool/
    ├── plugin.json           # Adapter metadata (name, version, description)
    ├── skills/               # Slash commands or equivalent
    │   ├── mnemosyne-context.md
    │   ├── mnemosyne-promote.md
    │   ├── mnemosyne-curate.md
    │   └── mnemosyne-explore.md
    └── references/           # Context documents
        └── mnemosyne-guide.md
```

### `plugin.json`

```json
{
  "name": "mnemosyne-your-tool",
  "version": "0.1.0",
  "description": "Mnemosyne global knowledge integration for your-tool",
  "author": "Your Name"
}
```

### `references/mnemosyne-guide.md`

A brief guide explaining the two-tier model, the knowledge format, and how to use the skills. This is injected as a reference document (read once per session or on demand, not on every message). See `adapters/claude-code/references/global-knowledge-guide.md` for the reference implementation.
