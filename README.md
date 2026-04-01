# Mnemosyne

Mnemosyne is a global developer knowledge system for LLM-driven development. It supplements per-project observational memory with a CLI-managed knowledge store at `~/.mnemosyne/` that accumulates transferable insights across every project you work on.

In Greek mythology, Mnemosyne is the Titaness of memory and remembrance — the keeper of knowledge that shaped civilisation and the mother of the nine Muses. The name was chosen deliberately. Senior developers carry a form of tacit memory that makes them effective across unfamiliar codebases: patterns they've seen fail, idioms that hold up under pressure, tools that deliver on their promises. Mnemosyne externalises that memory so it can be shared with an LLM on every session, in every project.

The problem Mnemosyne solves is context loss. Each LLM conversation starts fresh. Per-project knowledge helps within a project, but the insight that "unbounded channels cause memory exhaustion under sustained load" is valuable in every async Rust project — not just the one where you first learned it. Mnemosyne captures those cross-project insights, organises them by axis (language, domain, tool, technique, project), and surfaces the relevant ones automatically when you begin work.

## Quick Start

```bash
# Build and install
cargo install --git https://github.com/your-org/mnemosyne

# Initialise global knowledge store
mnemosyne init

# Install the Claude Code plugin
mnemosyne install claude-code

# Query global knowledge (auto-detects current project context)
mnemosyne query --context

# Promote a per-project learning to global
mnemosyne promote --tags rust,async --origin my-project
```

To continue on a new machine with an existing knowledge base:

```bash
mnemosyne init --from git@github.com:you/mnemosyne-knowledge.git
```

## Features

| Feature | Description |
|---------|-------------|
| **Two-tier knowledge** | Per-project knowledge (Tier 1) and global knowledge (Tier 2) work independently; global is purely additive |
| **Axis organisation** | Five top-level axes: `languages/`, `domains/`, `tools/`, `techniques/`, `projects/` |
| **Tag-based retrieval** | Cross-cutting queries via frontmatter tags — find everything tagged `async` regardless of axis |
| **Context-inferred queries** | Detects project languages and dependencies, maps them to tags, retrieves relevant global knowledge |
| **Contradiction detection** | Jaccard tag-overlap scoring flags potential contradictions before promotion; you resolve interactively |
| **Evidence-based evolution** | No time-based expiry; knowledge is valid until contradicted by evidence |
| **Supersession records** | Old content preserved inline in a `## Superseded` section with date ranges and reasons |
| **Divergence detection** | Flags when multiple projects depart from global knowledge, suggesting the global record needs updating |
| **Reflective curation** | Interactive `curate` command reviews active areas and surfaces divergence |
| **Socratic exploration** | `explore` command performs gap analysis and horizon scanning |
| **Git-backed store** | `~/.mnemosyne/` is a standard Git repo; sync between machines with push/pull |
| **Claude Code plugin** | `mnemosyne install claude-code` installs skills and references for the observational-memory plugin |
| **Graceful degradation** | The Claude Code plugin works without Mnemosyne installed — global features are simply omitted |

## Philosophy

Knowledge in Mnemosyne is treated as living beliefs, not permanent records.

**No time-based expiry.** A two-year-old insight about how garbage collection interacts with thread scheduling may be as valid today as when it was written — possibly more so, as accumulated evidence has validated it across many contexts. Time alone tells you nothing about whether knowledge is still correct. Evidence does. Knowledge expires when it is contradicted by observation, not when a clock ticks.

**Evidence-based evolution.** Every entry carries provenance: which projects it was observed in, when, under what context. When a contradiction arises, the system surfaces it and guides you through a resolution — supersede, coexist, discard, or refine. The history is preserved, not discarded.

**Confidence as epistemic honesty.** Entries carry one of four confidence levels: `high` (validated across multiple contexts), `medium` (validated in one project, likely transferable), `low` (observed once, tentative), and `prospective` (awareness of a possibility, not yet validated by hands-on experience). Prospective entries are clearly distinguished from experience-validated knowledge.

**Curation as deliberate practice.** The `curate` command is not automated maintenance — it is a reflective session. You review your own knowledge, confirm what still holds, update what has shifted, and prune what no longer applies. This mirrors how expert practitioners consolidate expertise.

## Documentation

- [User Guide](docs/user-guide.md) — installation, daily workflow, full walkthrough
- [CLI Reference](docs/reference.md) — every command, flag, and expected output
- [Knowledge Format](docs/knowledge-format.md) — file format spec, frontmatter fields, body conventions
- [Evolution Guide](docs/evolution-guide.md) — philosophy and mechanics of knowledge evolution
- [Configuration](docs/configuration.md) — `config.yml` reference, language profiles, context mappings
- [Plugin Development](docs/plugin-development.md) — how to build adapters for other editors/tools
- [Research Sources](docs/research-sources.md) — annotated bibliography of the cognitive science underpinning the design

## Status

Version 0.1.0 — initial implementation. The CLI, knowledge format, and Claude Code plugin are functional. The `explore` command's horizon-scanning mode (web search integration) is planned for a future release.

## License

MIT
