# Global Knowledge Guide

## The Two-Tier Model

Mnemosyne is a two-tier knowledge system:

| Tier | Where | Scope | Managed by |
|------|-------|-------|------------|
| Per-project | `knowledge/` in each project | Project-specific learnings | Observational memory plugin |
| Global | `~/.mnemosyne/` | Cross-project transferable knowledge | Mnemosyne CLI + plugin |

Tier 1 works without Tier 2. The global layer is additive: it enhances `/begin-work` with
cross-project context and adds promotion pathways from per-project to global during `/reflect`.

## Global Knowledge Structure

```
~/.mnemosyne/
├── config.yml
├── knowledge/
│   ├── languages/      # Rust, Swift, Python, etc.
│   ├── domains/        # macOS, web, databases, etc.
│   ├── tools/          # Cargo, Git, Docker, etc.
│   ├── techniques/     # TDD, async patterns, error handling, etc.
│   └── projects/       # Per-project summaries and cross-references
└── archive/            # Pruned entries (preserved, not deleted)
```

Fixed top-level axes provide browsable structure. Tags in frontmatter handle cross-cutting
queries — find everything tagged `async` regardless of which directory it lives in.

## Knowledge File Format

```markdown
---
title: Rust Async Patterns
tags: [rust, async, tokio, concurrency]
created: 2026-03-31
last_validated: 2026-03-31
confidence: high
origins:
  - project: my-project
    date: 2026-03-31
    context: "Async bridge work"
supersedes: []
---

## Bounded channels prevent backpressure bugs

**2026-03-31:** 🔴 Always use bounded channels in tokio. Unbounded channels
caused memory exhaustion under sustained load.
```

### Frontmatter Fields

| Field | Purpose |
|-------|---------|
| `title` | Human-readable name |
| `tags` | Cross-referencing; used for retrieval and contradiction detection |
| `created` | When first created |
| `last_validated` | Updated when developer confirms "still holds" during curation |
| `confidence` | `high` / `medium` / `low` / `prospective` |
| `origins` | Provenance trail: which project, when, context |
| `supersedes` | Links to entries this one replaced |

**Confidence levels:**
- `high` — validated across multiple projects or high-stakes context
- `medium` — validated in one project, likely broadly applicable
- `low` — observed once, tentative
- `prospective` — awareness of a possibility, not yet validated by hands-on experience

## Promoting to Global

### Via /reflect (recommended)

After a code review session, `/reflect` offers to promote per-project observations to global
for each observation it processes. Accept when the learning has clear cross-project applicability.

### Via /promote-global (ad-hoc)

Run `/promote-global` at any time to promote a specific learning. The skill guides you through:
1. Articulating the learning as a transferable insight
2. Selecting appropriate tags
3. Running `mnemosyne promote` which handles contradiction detection

### Contradiction Detection

When promoting, the CLI checks existing global entries with overlapping tags. If a potential
contradiction is found, you resolve it interactively:

- **Supersede** — your new understanding replaces the old. Old content moves to a
  `## Superseded` section in the file with a reason and date.
- **Coexist** — both are valid in different contexts. Add scope/context to disambiguate.
- **Discard** — the new observation was wrong or project-specific.
- **Refine** — edit both entries to capture the nuance.

## Curation

`/curate-global` runs a reflective review session. The system surfaces:
- Entries related to recent project activity
- Entries with implicit divergence (recent learnings that may contradict global knowledge)
- Low-confidence entries ready for review

For each entry: **validate** (bump `last_validated`), **supersede**, **refine**, or **prune**
(move to `archive/` — preserved, not deleted).

Curation is deliberate practice, not automation. Run it after completing a major project or
when you sense your understanding in an area has shifted.

## Knowledge Exploration

`/explore-knowledge` actively grows the knowledge base through three modes:

- **Gap analysis** — identify thin or missing areas relative to your active domains
- **Horizon scanning** — research new developments in your ecosystems (uses web search)
- **Open questions** — review unresolved tensions and `confidence: low` / `prospective` entries

Entries created during horizon scanning use `confidence: prospective` — they represent
awareness, not validated experience. Prospective entries are clearly distinguished from
experience-validated knowledge and are revisited in future exploration sessions.

## Knowledge Evolution

Knowledge is treated as living beliefs, not permanent records. There is no time-based expiry.
Knowledge is valid until evidence suggests otherwise — a multi-year-old entry about fundamental
design principles can be as valid as yesterday's, unless challenged.

The evolution path: raw observation → per-project learning → global learning → superseded
(if contradicted) → archived (if no longer applicable).

## Multi-Machine Sync

`~/.mnemosyne/` is a standard Git repo. Sync between machines:

```bash
cd ~/.mnemosyne && git push    # on machine A
cd ~/.mnemosyne && git pull    # on machine B
```

The `cache/` directory is gitignored — it is rebuilt automatically after clone or pull.

## Installation

```bash
cargo install --git <repo-url> mnemosyne
mnemosyne init                          # first-time setup
mnemosyne install claude-code           # install Claude Code plugin
```

To continue on a new machine with an existing knowledge base:

```bash
mnemosyne init --from git@github.com:user/mnemosyne-knowledge.git
```
