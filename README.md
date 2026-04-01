# Mnemosyne

> Long-term, cross-project knowledge for AI-assisted development.

A [Linkuistics](https://github.com/Linkuistics) project.

---

Mnemosyne builds long-term, hierarchically organised, cross-linked global memory for LLM-driven development. It simulates how senior developers accumulate expertise across projects, tools, languages, and domains -- and makes that knowledge available to AI agents working on any project.

Named after the Greek Titaness of memory and mother of the nine Muses, Mnemosyne represents the preservation of knowledge across time and context.

## Why Mnemosyne Exists

AI coding assistants start every session from scratch. They have no memory of what they learned yesterday -- which patterns worked, which approaches failed, what the team prefers. Senior developers are effective precisely because they carry this cross-project, cross-language knowledge. Mnemosyne gives AI agents the same advantage.

The problem is scale: a developer works on dozens of projects over years. You can't load all accumulated knowledge into a single context window. Mnemosyne solves this with careful indexing, cross-referencing, and context-aware retrieval -- surfacing only the knowledge relevant to the current task.

## Architecture

A two-tier knowledge system:

### Tier 1 -- Per-Project Knowledge

Each project has a `knowledge/` directory containing observations specific to that codebase -- patterns discovered, decisions made, techniques that worked. This is managed by a Claude Code plugin (based on Mastra's observational memory concepts).

### Tier 2 -- Global Knowledge

Cross-project knowledge lives in `~/.mnemosyne/`, a Git-backed store organised along multiple axes:

```
~/.mnemosyne/
├── config.yml
├── .gitignore
├── knowledge/
│   ├── languages/         # Rust, Swift, Racket, Haskell, Prolog, etc.
│   ├── domains/           # macOS/AppKit, web, databases, concurrency, etc.
│   ├── tools/             # Cargo, Git, Docker, etc.
│   ├── techniques/        # TDD, async patterns, error handling, etc.
│   └── projects/          # Per-project summaries and cross-references
├── archive/               # Pruned entries (preserved, not deleted)
└── cache/                 # Derived data (gitignored)
```

Knowledge flows upward: project-level observations are promoted to global knowledge when they prove useful across projects. The CLI provides commands for querying, promoting, curating, and evolving knowledge entries.

### Knowledge Files

Each entry is Markdown with YAML frontmatter tracking provenance and confidence:

```yaml
---
title: "Chez Scheme FFI requires explicit gc-protect for callbacks"
tags: [chez-scheme, ffi, memory-management]
created: 2026-02-15
last_validated: 2026-03-20
confidence: high
origins:
  - project: apianyware-macos
    date: 2026-02-15
    context: "FFI callback integration"
supersedes: []
---
```

### Evolution

Mnemosyne tracks how knowledge evolves -- entries can be superseded, contradicted, or validated over time. Stale knowledge is archived rather than deleted, preserving the history of understanding.

## Components

- **`mnemosyne` CLI** (Rust) -- query, promote, curate, explore, and manage the global knowledge store
- **Claude Code plugin** -- provides 7 skills (`/begin-work`, `/reflect`, `/setup-knowledge`, `/create-plan`, `/curate-global`, `/promote-global`, `/explore-knowledge`) that integrate per-project observational memory with the global knowledge base
- **`~/.mnemosyne/` Git repo** -- the global store itself, versioned for history and portability
- **Evaluation framework** -- Rust harness for retrieval/contradiction/context metrics plus a Python LLM-as-judge harness for entry quality scoring

## Quick Start

```bash
# Build and install
cargo install --path .

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

## CLI Commands

| Command | Description |
|---------|-------------|
| `mnemosyne init [--from <repo>]` | Create `~/.mnemosyne/` with default structure, or clone an existing knowledge repo |
| `mnemosyne query <terms> [--context] [--format markdown\|json\|plain] [--max-tokens N]` | Search global knowledge by terms or auto-detected project context |
| `mnemosyne promote [--tags T] [--origin P]` | Interactively promote a learning to the global store with contradiction checking |
| `mnemosyne curate` | Interactive reflective curation session -- review, validate, supersede, or prune entries |
| `mnemosyne explore` | Interactive knowledge exploration -- gap analysis, tag clusters, open questions |
| `mnemosyne install claude-code` | Install the Claude Code adapter plugin to `~/.claude/plugins/observational-memory` |
| `mnemosyne status` | Knowledge base summary: entry counts by axis and confidence, origin projects |

## Features

| Feature | Description |
|---------|-------------|
| **Two-tier knowledge** | Per-project knowledge (Tier 1) and global knowledge (Tier 2) work independently; global is purely additive |
| **Axis organisation** | Five top-level axes: `languages/`, `domains/`, `tools/`, `techniques/`, `projects/` |
| **Tag-based retrieval** | Cross-cutting queries via frontmatter tags -- find everything tagged `async` regardless of axis |
| **Context-inferred queries** | Detects project languages and dependencies, maps them to tags, retrieves relevant global knowledge |
| **Contradiction detection** | Jaccard tag-overlap scoring flags potential contradictions before promotion; you resolve interactively |
| **Evidence-based evolution** | No time-based expiry; knowledge is valid until contradicted by evidence |
| **Supersession records** | Old content preserved inline in a `## Superseded` section with date ranges and reasons |
| **Divergence detection** | Flags when multiple projects depart from global knowledge, suggesting the global record needs updating |
| **Reflective curation** | Interactive `curate` command reviews active areas and surfaces divergence |
| **Socratic exploration** | `explore` command performs gap analysis and surfaces tag clusters for synthesis |
| **Git-backed store** | `~/.mnemosyne/` is a standard Git repo; sync between machines with push/pull |
| **Claude Code plugin** | 7 skills for observational memory, planning, reflection, curation, promotion, and exploration |
| **Graceful degradation** | The Claude Code plugin works without Mnemosyne installed -- global features are simply omitted |
| **Language detection** | Built-in profiles for 12 languages: Rust, Python, Haskell, OCaml, Prolog, Mercury, Scheme, Racket, Common Lisp, Smalltalk, Idris, Swift |
| **Dependency parsing** | Extracts dependencies from Cargo.toml and pyproject.toml to infer relevant knowledge tags |
| **Multiple output formats** | Query results in Markdown, JSON, or plain text; token-budget-aware output limiting |

## Evaluation

Mnemosyne includes a quantitative evaluation framework (`eval/`) for measuring retrieval quality, contradiction detection accuracy, context detection coverage, and entry quality.

### Rust Harness (`eval/`)

A standalone binary that loads a benchmark corpus of synthetic knowledge entries, graded-relevance queries, annotated contradiction pairs, and mock projects, then reports:

- **Retrieval metrics** -- MRR, Precision@k, Recall@k, nDCG@k
- **Contradiction detection** -- Precision, Recall, F1 with configurable threshold sweep
- **Context detection** -- Language, dependency, and tag mapping accuracy

```bash
cd eval && cargo run -- --verbose --sweep    # human-readable with threshold sweep
cd eval && cargo run -- --json               # machine-readable JSON
```

### Python Quality Harness (`eval/quality/`)

An LLM-as-judge harness that scores entries against a four-dimension rubric (specificity, actionability, provenance, confidence fit) using the Anthropic SDK, with two-pass variance reduction:

```bash
cd eval/quality
PYTHONPATH=../.. python -m eval.quality.src.__main__ --single-pass --verbose
```

Includes automated structural completeness checks that require no API key.

See [Evaluation Strategy](docs/evaluation-strategy.md) for the full methodology and research context.

## Philosophy

Knowledge in Mnemosyne is treated as living beliefs, not permanent records.

**No time-based expiry.** A two-year-old insight about how garbage collection interacts with thread scheduling may be as valid today as when it was written -- possibly more so, as accumulated evidence has validated it across many contexts. Time alone tells you nothing about whether knowledge is still correct. Evidence does. Knowledge expires when it is contradicted by observation, not when a clock ticks.

**Evidence-based evolution.** Every entry carries provenance: which projects it was observed in, when, under what context. When a contradiction arises, the system surfaces it and guides you through a resolution -- supersede, coexist, discard, or refine. The history is preserved, not discarded.

**Confidence as epistemic honesty.** Entries carry one of four confidence levels: `high` (validated across multiple contexts), `medium` (validated in one project, likely transferable), `low` (observed once, tentative), and `prospective` (awareness of a possibility, not yet validated by hands-on experience). Prospective entries are clearly distinguished from experience-validated knowledge.

**Curation as deliberate practice.** The `curate` command is not automated maintenance -- it is a reflective session. You review your own knowledge, confirm what still holds, update what has shifted, and prune what no longer applies. This mirrors how expert practitioners consolidate expertise.

## Documentation

- [User Guide](docs/user-guide.md) -- installation, daily workflow, full walkthrough
- [CLI Reference](docs/reference.md) -- every command, flag, and expected output
- [Knowledge Format](docs/knowledge-format.md) -- file format spec, frontmatter fields, body conventions
- [Evolution Guide](docs/evolution-guide.md) -- philosophy and mechanics of knowledge evolution
- [Configuration](docs/configuration.md) -- `config.yml` reference, language profiles, context mappings
- [Plugin Development](docs/plugin-development.md) -- how to build adapters for other editors/tools
- [Research Sources](docs/research-sources.md) -- annotated bibliography of the cognitive science underpinning the design
- [Evaluation Strategy](docs/evaluation-strategy.md) -- metrics, techniques, and framework for evaluating whether the system works

## Status

Version 0.1.0 -- initial implementation. The CLI, knowledge format, and Claude Code plugin are functional. The evaluation framework (intrinsic retrieval/contradiction/context metrics and LLM-as-judge quality scoring) is implemented.

**Not yet implemented:**
- Horizon-scanning mode in the `explore` command (web search integration for discovering new developments). Spec and implementation plan are written.
- Multi-session simulation evaluation (Phase 3) -- validating knowledge accumulation across simulated project sessions.
- Controlled impact experiments (Phase 4) -- A/B experimental harness measuring whether Mnemosyne improves AI assistant outcomes.

## Related Projects

Mnemosyne accumulates knowledge from working on all Linkuistics projects and makes it available across them:

- **[APIAnyware-MacOS](https://github.com/Linkuistics/APIAnyware-MacOS)** -- knowledge about FFI patterns, platform APIs, code generation
- **[TestAnyware](https://github.com/Linkuistics/TestAnyware)** -- knowledge about VM management, GUI testing strategies
- **[*Pro IDEs](https://github.com/Linkuistics)** -- knowledge about language-specific idioms and tooling

## License

[Apache-2.0](LICENSE) -- Copyright 2026 Linkuistics
