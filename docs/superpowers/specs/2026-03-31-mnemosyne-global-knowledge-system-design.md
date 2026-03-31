# Mnemosyne — Global Developer Knowledge System

## 1. System Overview

Mnemosyne is a two-tier knowledge system for LLM-driven development that simulates
how senior developers accumulate expertise over time across projects, tools, languages,
and domains.

### Tier 1: Per-Project Knowledge (existing)

The observational-memory plugin, unchanged. Knowledge lives in each project's `knowledge/`
directory, organized by project-specific axes. Plans use the Do → Verify → Observe cycle.
`/reflect` promotes observations to per-project knowledge.

### Tier 2: Global Knowledge (new)

A global layer managed by the Mnemosyne CLI. Knowledge lives in `~/.mnemosyne/`, organized
by hybrid axes with tag-based cross-referencing. Global knowledge is populated by promoting
per-project learnings upward when they have broad applicability.

### Relationship Between Tiers

Tier 1 works independently — the plugin is useful without Mnemosyne installed. Tier 2 is
additive: it enhances Tier 1 by making cross-project knowledge available during `/begin-work`
and by detecting contradictions between new observations and existing global knowledge.

### Components

- **`~/.mnemosyne/`** — a Git repo containing global knowledge, config, and cache
- **`mnemosyne` CLI** — a Rust binary for querying, promoting, curating, and managing
  global knowledge
- **Claude Code plugin** — updated observational-memory plugin that shells out to the CLI
  for global operations; installed via `mnemosyne install claude-code`

---

## 2. Global Knowledge Store (`~/.mnemosyne/`)

### Directory Structure

```
~/.mnemosyne/
├── config.yml                    # Global settings, language profiles, context mappings
├── knowledge/
│   ├── languages/                # Rust, Swift, Racket, Python, Prolog, etc.
│   │   └── rust.md
│   ├── domains/                  # macOS/AppKit, web, databases, etc.
│   │   └── macos-appkit.md
│   ├── tools/                    # Cargo, Git, Docker, etc.
│   │   └── cargo.md
│   ├── techniques/               # TDD, async patterns, error handling, etc.
│   │   └── async-patterns.md
│   └── projects/                 # Per-project summaries and cross-references
│       └── apianyware-macos.md
├── archive/                      # Pruned entries (preserved, not deleted)
├── cache/                        # Derived index, not committed to Git
│   └── tag-index.json
├── docs/                         # User guide, reference, examples
│   ├── guide.md
│   ├── reference.md
│   └── examples/
└── .gitignore                    # Ignores cache/
```

The **fixed top-level axes** (languages, domains, tools, techniques, projects) provide
browsable structure. **Tags** in file frontmatter handle cross-cutting queries — finding
everything tagged `async` regardless of which directory it lives in.

### Knowledge File Format

```markdown
---
title: Rust Async Patterns
tags: [rust, async, tokio, concurrency]
created: 2026-03-31
last_validated: 2026-03-31
confidence: high
origins:
  - project: apianyware-macos
    date: 2026-03-31
    context: "Racket FFI async bridge work"
supersedes: []
---

## Bounded channels prevent backpressure bugs

**2026-03-31:** 🔴 Always use bounded channels in tokio. Unbounded channels
caused memory exhaustion under sustained load in the Racket async bridge.

## Task cancellation requires explicit cleanup

**2026-03-31:** 🟡 When a tokio task is cancelled, drop guards don't run
in the expected order. Explicit cleanup via cancellation tokens is safer.
```

### Frontmatter Fields

| Field | Purpose |
|-------|---------|
| `title` | Human-readable name |
| `tags` | Cross-referencing mechanism; used for retrieval and contradiction detection |
| `created` | When the entry was first created |
| `last_validated` | Updated during curation when the developer confirms "still holds" |
| `confidence` | `high` / `medium` / `low` / `prospective`; set by the developer during promotion. `prospective` indicates awareness rather than validated experience (from exploration sessions) |
| `origins` | Provenance trail: which project, when, what context. Multiple origins if reinforced across projects |
| `supersedes` | Links to entries this one replaced, preserving evolution history |

---

## 3. Knowledge Evolution

Knowledge is treated as **living beliefs**, not permanent records. There is no time-based
expiry. Knowledge is valid until evidence suggests otherwise — a 5-year-old entry about
fundamental design principles is as valid as yesterday's, unless something challenges it.

### 3.1 Contradiction Detection at Promotion Time

When promoting from per-project to global, the CLI searches existing global entries with
overlapping tags. If a potential contradiction is found, the developer resolves it
interactively:

- **Supersede** — replace existing with new understanding. Old content moves to a
  `## Superseded` section within the file with a reason and date, preserving why the
  developer changed their mind.
- **Coexist** — both are valid in different contexts. Add scope/context to disambiguate.
- **Discard** — the new observation was wrong.
- **Refine** — edit both entries to capture the nuance.

### Supersession Format

```markdown
## Superseded

### Unbounded channels for logging (2026-01-15 → 2026-03-31)
> Prefer unbounded channels for fire-and-forget logging

**Reason superseded:** Caused memory exhaustion under sustained load.
Bounded channels with backpressure are safer in all cases.
```

### 3.2 Reflective Curation

A deliberate practice via `mnemosyne curate` (or `/curate-global` in Claude Code). Not
automated pruning — a developer-driven reflection session analogous to how humans
periodically reconsider their assumptions. The system:

- Identifies **areas of recent activity** across projects and presents related global
  knowledge for review ("You've done significant Rust async work across 3 projects
  recently. Here's what your global knowledge says. Still hold?")
- Surfaces **entries with implicit divergence** (see below)
- For each entry, the developer can: **validate** (confirm still holds, bump
  `last_validated`), **supersede** (provide updated understanding), **refine**
  (adjust scope/nuance), or **prune** (archive — moved to `~/.mnemosyne/archive/`,
  not deleted)

### 3.3 Implicit Divergence Detection

Over time, if project-local observations across multiple projects consistently diverge
from a global entry — even without explicit contradiction at promotion time — Mnemosyne
flags this during curation. This is the closest analog to how humans gradually shift
positions through accumulated experience rather than a single contradicting event.

**Mechanism:** Each global promotion records its origin project. During `mnemosyne curate`,
the system compares recent project-local promotions against existing global entries with
overlapping tags. If 2+ recent promotions from different projects express a position that
diverges from an existing global entry, the entry is flagged for reflective review. The
exact divergence threshold and matching heuristics will be refined during implementation
as real usage data accumulates.

### 3.4 Knowledge Exploration Sessions (`/explore-knowledge`)

An interactive, Socratic dialogue between the LLM, the knowledge base, and the developer.
Unlike curation (which reviews existing knowledge for validity), exploration actively
seeks to **grow** the knowledge base by identifying gaps, researching new developments,
and engaging the developer's expertise.

**Three modes of exploration:**

#### Gap Analysis

The system analyzes the knowledge base for thin or missing areas relative to the
developer's active domains:

- "You have deep knowledge about Rust async patterns but nothing about error handling
  strategies in async contexts. What's your approach?"
- "Your macOS/AppKit knowledge covers UI controls but has no entries about accessibility.
  Is that intentional or a gap worth filling?"
- "You've worked with 4 Scheme-family languages but your knowledge base has no entries
  comparing their FFI approaches. Want to capture your perspective?"

The developer responds conversationally. The system distills their responses into
candidate knowledge entries with appropriate tags, confidence levels, and axis placement.

#### Horizon Scanning

The system performs web searches in areas relevant to the developer's knowledge base,
looking for:

- New libraries, frameworks, or tools in domains the developer works in
- Architectural approaches or patterns gaining traction in relevant communities
- Breaking changes or deprecations affecting tools/languages in the knowledge base
- Research papers or conference talks related to the developer's technique areas

Findings are presented for discussion: "There's a new Rust error handling crate called
`error-stack` that takes a different approach to the context pattern you've documented.
Want to explore it?" The developer's assessment gets recorded as **prospective** knowledge
— entries with `confidence: prospective` indicating they represent awareness of
possibilities rather than validated experience.

#### Open Questions

The system identifies unresolved tensions or open questions within the knowledge base:

- Entries where the developer chose "coexist" during contradiction resolution — are those
  contexts still distinct, or has experience clarified which approach is better?
- Entries with `confidence: low` or `confidence: prospective` — has the developer gained
  enough experience to upgrade or discard them?
- Clusters of related entries that might benefit from a synthesized overview

**Prospective knowledge format:**

```markdown
---
title: Error-Stack Crate for Structured Error Context
tags: [rust, error-handling, libraries]
created: 2026-04-01
confidence: prospective
source: horizon-scan
origins:
  - project: global
    date: 2026-04-01
    context: "Discovered during /explore-knowledge session"
---

## Assessment

**2026-04-01:** 🟢 error-stack provides frame-based error context similar to
backtraces but structured. Potentially superior to the anyhow + thiserror pattern
for complex error chains. Not yet evaluated in a real project.

**Status:** Prospective — to be validated through hands-on use.
```

Prospective entries are clearly distinguished from experience-validated knowledge.
They participate in contradiction detection (a prospective entry might be superseded
by real experience) and are surfaced during future exploration sessions for follow-up.

**CLI and plugin integration:**

- CLI: `mnemosyne explore` — runs the exploration session interactively in the terminal
- Claude Code: `/explore-knowledge` skill — runs the session within the agent, with
  access to web search and the current project context for richer discussion

---

## 4. The Rust CLI (`mnemosyne`)

### Commands

| Command | Purpose |
|---------|---------|
| `mnemosyne init` | Create `~/.mnemosyne/` with default structure |
| `mnemosyne init --from <repo>` | Clone an existing knowledge repo into `~/.mnemosyne/` |
| `mnemosyne query <terms>` | Search global knowledge by text and/or tags |
| `mnemosyne query --context` | Infer context from current project, return relevant knowledge |
| `mnemosyne query --format <fmt>` | Output format: `markdown` (default), `json`, `plain` |
| `mnemosyne query --max-tokens <n>` | Limit output to fit within a context budget |
| `mnemosyne promote --tags <tags> --origin <project>` | Interactive promotion with contradiction detection |
| `mnemosyne curate` | Reflective curation session |
| `mnemosyne explore` | Interactive knowledge exploration session (gap analysis, horizon scanning) |
| `mnemosyne install claude-code` | Install/update the Claude Code plugin |
| `mnemosyne status` | Summary: entry counts by axis, recent activity, flagged contradictions |

### Internal Architecture

```
mnemosyne/
├── src/
│   ├── main.rs                    # CLI entry point (clap)
│   ├── commands/                  # One module per command
│   │   ├── init.rs
│   │   ├── query.rs
│   │   ├── promote.rs
│   │   ├── curate.rs
│   │   ├── explore.rs
│   │   ├── install.rs
│   │   └── status.rs
│   ├── knowledge/                 # Core knowledge management
│   │   ├── store.rs               # Read/write knowledge files
│   │   ├── entry.rs               # Parse/serialize entries (frontmatter + content)
│   │   ├── index.rs               # KnowledgeIndex trait + file-native implementation
│   │   └── tags.rs                # Tag matching, overlap detection
│   ├── evolution/                 # Knowledge lifecycle
│   │   ├── contradiction.rs       # Detect contradictions via tag overlap + content analysis
│   │   ├── supersede.rs           # Handle supersession (move old content, update metadata)
│   │   └── divergence.rs          # Implicit divergence detection across projects
│   ├── context/                   # Project context inference
│   │   ├── detect.rs              # Read Cargo.toml, package.json, etc.
│   │   └── mapping.rs             # Map detected signals to tags
│   └── plugin/                    # Plugin generation/installation
│       └── claude_code.rs         # Generate and install Claude Code plugin files
├── tests/
├── docs/
└── Cargo.toml
```

### The KnowledgeIndex Trait

The key abstraction for future extensibility:

```rust
trait KnowledgeIndex {
    fn search(&self, query: &Query) -> Vec<SearchResult>;
    fn find_contradictions(&self, entry: &Entry) -> Vec<Contradiction>;
    fn find_by_tags(&self, tags: &[Tag]) -> Vec<&Entry>;
    fn rebuild(&mut self) -> Result<()>;
}
```

v1 implements this with in-memory file scanning + cached tag index. v2 can add a
`VectorIndex` implementation using LanceDB + local embeddings behind the same trait,
without changing any command logic.

### Dependencies (minimal)

- `clap` — CLI argument parsing
- `serde` / `serde_yaml` — frontmatter parsing
- `gray_matter` or similar — markdown frontmatter extraction
- `ignore` — fast directory walking (same lib ripgrep uses)
- `colored` — terminal output formatting

---

## 5. Claude Code Plugin Integration

The existing observational-memory plugin evolves to become Mnemosyne's Claude Code adapter.
It gains global knowledge capabilities while remaining backward-compatible — works without
the CLI installed, just without global features.

### Plugin Structure

Distributed from the Mnemosyne repo at `adapters/claude-code/`:

```
adapters/claude-code/
├── plugin.json
├── skills/
│   ├── begin-work.md              # Updated: also loads global knowledge
│   ├── reflect.md                 # Updated: offers global promotion
│   ├── create-plan.md             # Unchanged
│   ├── setup-knowledge.md         # Updated: also runs mnemosyne init if needed
│   ├── curate-global.md           # New: reflective global curation
│   ├── promote-global.md          # New: ad-hoc global promotion
│   └── explore-knowledge.md       # New: interactive knowledge exploration
├── references/
│   ├── observational-memory-guide.md
│   ├── plan-format.md
│   ├── coding-conventions.md
│   └── global-knowledge-guide.md  # New: how global knowledge works
```

### Changes to Existing Skills

**`/begin-work`** gains a new step after loading project knowledge:

> Shell out to `mnemosyne query --context --format markdown`. Include results in
> the summary under "Global knowledge loaded". If the CLI is not installed, skip silently.

**`/reflect`** gains global promotion after per-project promotion:

> For each observation promoted to per-project knowledge, ask: "This learning may apply
> beyond this project. Promote to global?" If yes, shell out to `mnemosyne promote`.
> The CLI handles contradiction detection. If the CLI is not installed, skip silently.

### Graceful Degradation

Every global feature is gated on `which mnemosyne` succeeding. If the CLI isn't installed,
the plugin behaves exactly as the current observational-memory plugin — no errors, no
warnings, just per-project functionality.

### Installation

```bash
mnemosyne install claude-code
# Copies adapters/claude-code/ to ~/.claude/plugins/observational-memory/
# Preserves any project-specific skills the user has added
# Reports what was updated
```

---

## 6. Context-Inferred Retrieval

When `mnemosyne query --context` runs from within a project directory, it builds a
relevance profile by reading project signals.

### Signal Sources

| Signal | What it provides |
|--------|-----------------|
| `Cargo.toml` | Language: Rust, plus crate names as tool hints |
| `package.json` | Language: JS/TS, plus framework detection |
| `pyproject.toml` / `requirements.txt` | Language: Python, plus library detection |
| `*.cabal` / `stack.yaml` | Language: Haskell |
| `dune-project` / `*.opam` | Language: OCaml |
| `*.ipkg` | Language: Idris |
| `info.rkt` | Language: Racket |
| `pack.pl` | Language: Prolog |
| `Mercury.options` | Language: Mercury |
| `*.asd` | Language: Common Lisp |
| `*.swift` / `Package.swift` | Language: Swift |
| `.observational-memory.yml` | Project axes, knowledge structure |
| `CLAUDE.md` / `LLM_CONTEXT/` | Project-specific context, domain keywords |
| `.git/config` | Project name from remote URL |
| File extensions scan | Language detection fallback |

### Extensible Language Profiles

Detection is driven by a registry of language profiles in `config.yml`, not hardcoded
logic. Ships with defaults for common languages; users add profiles for any language:

```yaml
language_profiles:
  rust:
    markers: ["Cargo.toml"]
    extensions: [".rs"]
    dependency_file: "Cargo.toml"
    dependency_parser: cargo
  python:
    markers: ["pyproject.toml", "setup.py", "requirements.txt"]
    extensions: [".py"]
    dependency_file: "pyproject.toml"
    dependency_parser: pyproject
  prolog:
    markers: ["pack.pl"]
    extensions: [".pl", ".pro"]
  mercury:
    extensions: [".m", ".mh"]
    markers: ["Mercury.options"]
  ocaml:
    markers: ["dune-project", "*.opam"]
    extensions: [".ml", ".mli"]
    dependency_parser: opam
  haskell:
    markers: ["*.cabal", "stack.yaml", "cabal.project"]
    extensions: [".hs"]
    dependency_parser: cabal
  scheme:
    extensions: [".scm", ".ss", ".sld"]
  racket:
    markers: ["info.rkt"]
    extensions: [".rkt"]
  common-lisp:
    markers: ["*.asd"]
    extensions: [".lisp", ".cl", ".lsp"]
  smalltalk:
    extensions: [".st"]
    markers: [".smalltalk.ston"]
  idris:
    extensions: [".idr"]
    markers: ["*.ipkg"]
```

Dependency parsers are pluggable — v1 ships with parsers for the most common formats.
Languages without a parser gracefully degrade to extension-based detection.

### Retrieval Pipeline

1. **Detect** — scan project root for signal files, extract raw signals
2. **Map** — convert signals to tags via configurable mappings
3. **Query** — search global index for entries matching those tags
4. **Rank** — order by relevance (matching tags, confidence, validation recency)
5. **Limit** — return top N entries (configurable) to fit within context budgets

Context budget awareness: the CLI accepts a `--max-tokens` flag. Results are truncated
to fit, prioritizing higher-confidence and more-relevant entries.

### Context Mappings

Configurable in `config.yml`:

```yaml
context_mappings:
  cargo_dependencies:
    tokio: [async, tokio, concurrency]
    sqlx: [database, sql, async]
    axum: [web, http, api]
  file_patterns:
    "*.swift": [swift, apple]
    "*.rkt": [racket, scheme, lisp]
```

---

## 7. Documentation

Documentation is a first-class deliverable, built alongside implementation.

### Audiences

| Audience | What they need | Where it lives |
|----------|---------------|----------------|
| End users | Install, configure, daily use | `~/.mnemosyne/docs/` + README |
| Plugin users | Claude Code integration, skill reference | Plugin `references/` |
| Contributors | Architecture, extending the system | Mnemosyne repo `docs/` |

### Deliverables

- **README.md** — project overview, quick start, philosophy
- **User Guide** (`docs/user-guide.md`) — complete walkthrough: setup, daily workflow,
  curation sessions, knowledge evolution, multi-machine sync
- **CLI Reference** (`docs/reference.md`) — every command, flag, and option with examples
- **Knowledge Format Spec** (`docs/knowledge-format.md`) — file format, frontmatter fields,
  tagging conventions, supersession format
- **Evolution Guide** (`docs/evolution-guide.md`) — contradiction detection, effective
  curation sessions, the philosophy behind evidence-based evolution
- **Configuration Reference** (`docs/configuration.md`) — config.yml format, language
  profiles, context mappings, customization
- **Plugin Development Guide** (`docs/plugin-development.md`) — how to build adapters
  for harnesses beyond Claude Code
- **Research Sources** (`docs/research-sources.md`) — annotated bibliography of research
  informing Mnemosyne's design (see below)

Each doc is written when its corresponding feature is implemented.

### Research Sources

Mnemosyne is a living, research-informed project. The research sources document is an
annotated bibliography tracking the cognitive science, memory research, and knowledge
management theory that informs the system's design. Initial areas:

- **Mastra's Observational Memory** — direct inspiration; structured knowledge capture
  for AI agents
- **Human memory models** — long-term memory organization (semantic networks, schemas,
  spreading activation)
- **Belief revision** — how humans update, contradict, and abandon prior beliefs
  (AGM theory, coherentism)
- **Expertise accumulation** — novice-to-expert progression; chunking, deliberate practice,
  pattern recognition
- **Spaced retrieval / testing effect** — active recall strengthens memory (relevant to
  curation/reflection)
- **Knowledge management systems** — Zettelkasten, personal knowledge bases,
  organizational memory
- **Cognitive load theory** — context-window-aware retrieval; limits of working memory

Each entry notes how it influences or could influence Mnemosyne's design, serving as a
roadmap for future enhancements grounded in evidence.

---

## 8. Distribution and Multi-Machine Management

### Installation

```bash
# Install from source (v1)
cargo install --git <repo-url> mnemosyne

# First-time setup
mnemosyne init

# Clone existing knowledge base on a new machine
mnemosyne init --from git@github.com:user/mnemosyne-knowledge.git

# Install Claude Code adapter
mnemosyne install claude-code
```

### Multi-Machine Sync

`~/.mnemosyne/` is a standard Git repo. Syncing between machines:

```bash
cd ~/.mnemosyne && git push    # on machine A
cd ~/.mnemosyne && git pull    # on machine B
```

### Committed vs. Cached

| Committed (portable) | Cached (derived, gitignored) |
|---------------------|------------------------------|
| `knowledge/` — all entries | `cache/tag-index.json` |
| `archive/` — pruned entries | `cache/` — future vector index |
| `config.yml` | |
| `docs/` | |

The cache is rebuilt automatically on first query after a clone or pull.

---

## 9. Scope

### v1

- `~/.mnemosyne/` Git repo with hybrid axes + tag-based cross-referencing
- Knowledge file format with full frontmatter (tags, confidence, origins, supersedes,
  last_validated)
- Rust CLI: `init`, `query`, `query --context`, `promote`, `curate`, `explore`,
  `install claude-code`, `status`
- Evidence-based knowledge evolution: contradiction detection, reflective curation,
  implicit divergence detection, supersession with history
- Context-inferred retrieval with extensible language profiles
- Updated Claude Code plugin with graceful degradation
- `KnowledgeIndex` trait designed to accept future vector search backend
- Full documentation suite including research sources
- Multi-machine sync via Git

### v2+ (deferred)

- Vector search via LanceDB + local embeddings (behind existing `KnowledgeIndex` trait)
- Adapter plugins for non-Claude-Code harnesses (Cursor, Copilot, etc.)
- Team/shared knowledge bases (multi-developer)
- Web UI for browsing/managing knowledge
- Additional dependency parsers for less common language ecosystems
