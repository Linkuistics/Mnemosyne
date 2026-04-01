# Mnemosyne — Horizon-Scanning Mode

## 1. Purpose

This spec defines the horizon-scanning mode within the `explore-knowledge` Claude Code
skill. Horizon scanning discovers new developments in the user's active ecosystems —
library updates, emerging patterns, new tools, breaking changes — and creates `prospective`
knowledge entries from them.

Unlike gap analysis (which examines what's thin) and open questions (which revisits what's
uncertain), horizon scanning looks outward: what has the world produced that this developer
doesn't yet know about?

The feature lives entirely in the Claude Code skill layer. No changes to the Rust core
library or CLI are required. The skill orchestrates web search, novelty filtering, and
entry generation using Claude Code's native capabilities, persisting results via the
existing `mnemosyne promote` command or direct file writes.

### Relationship to Existing Work

The `explore-knowledge` skill already defines three exploration modes (gap analysis,
horizon scanning, open questions) and Section 4 contains a placeholder description of
horizon scanning. This spec replaces that placeholder with a fully defined pipeline.

The knowledge format already supports horizon-scan output: `confidence: prospective` and
`source: horizon-scan` are established conventions (see `docs/knowledge-format.md` lines
198-237). The eval corpus contains 7 prospective entries demonstrating the target format.

The Rust CLI provides `mnemosyne query`, `mnemosyne status`, and `mnemosyne promote` —
all the data operations the skill needs. No new CLI subcommands are introduced.

---

## 2. Pipeline

The horizon-scanning pipeline has five stages, executed sequentially within a single
`explore-knowledge` session.

```
┌──────────────────────────────────────────────────────────┐
│  1. Scanning Area Selection                              │
│     Tag frequency + gap analysis → ranked suggestions    │
│     User picks areas to scan                             │
├──────────────────────────────────────────────────────────┤
│  2. Web Search                                           │
│     Formulate 2-3 queries per area → execute searches    │
├──────────────────────────────────────────────────────────┤
│  3. Novelty Filtering                                    │
│     Load existing entries → filter duplicates            │
├──────────────────────────────────────────────────────────┤
│  4. Batch Presentation & Selection                       │
│     Numbered list → user picks which to keep             │
├──────────────────────────────────────────────────────────┤
│  5. Entry Generation                                     │
│     Create prospective entries for selected findings     │
└──────────────────────────────────────────────────────────┘
```

### Session context

When the user runs all three exploration modes, gap analysis runs first. Its outputs —
identified gaps and tag frequency data — feed directly into Stage 1 of horizon scanning.
Open questions runs after horizon scanning, so newly created prospective entries may
surface for discussion.

When the user runs horizon scanning standalone, the skill loads tag frequency and
co-occurrence data from `mnemosyne status` and `mnemosyne query --format json` to
inform Stage 1.

---

## 3. Stage 1: Scanning Area Selection

The skill builds a ranked list of candidate scanning areas from two signal sources.

### 3.1 Gap-Driven Areas (Priority 1)

If gap analysis ran earlier in the session, areas flagged as thin or missing are
high-value scan targets. These are domains where the developer is active but
knowledge is sparse — exactly where horizon scanning adds the most value.

Example: "You have 5 entries tagged `tokio` but nothing on `error-handling` in async
contexts" becomes a suggested scanning area: "async error handling".

### 3.2 Active-Domain Areas (Priority 2)

The skill queries the knowledge base for tag frequency:

```bash
mnemosyne query --format json
```

From the response, it counts tag occurrences across all entries and identifies:

- **High-frequency tags** (top 5-8 by count): the user's core ecosystems
- **Co-occurring tag pairs** (tags frequently appearing together in entries):
  compound topics worth scanning as a unit (e.g., `rust + async`)

### 3.3 Presentation

The skill presents suggestions grouped by source:

```
Based on your knowledge base, here are suggested scanning areas:

From gaps:
  1. Async error handling (gap: active in async but no error handling entries)
  2. Observability tooling (gap: 1 entry, mostly prospective)

From active domains:
  3. Rust ecosystem (12 entries)
  4. Tokio / async runtime (8 entries)
  5. Swift concurrency (4 entries)

Or describe your own area to scan.

Which areas? (comma-separated numbers, or type a custom topic)
```

The user selects by number, enters a custom topic, or both. Custom topics are not
restricted to the suggestions.

---

## 4. Stage 2: Web Search

For each selected scanning area, the skill formulates targeted web search queries
and executes them via Claude Code's native web search. The number and type of
queries depends on whether the area was gap-driven or active-domain-driven.

### 4.1 Active-Domain Queries (2 per area)

For areas where the user has established knowledge, queries focus on what's new:

| Category | Template | Example |
|----------|----------|---------|
| Ecosystem updates | `"{area} new releases {current_year}"` | "rust async new releases 2026" |
| Emerging patterns | `"{area} emerging patterns best practices {current_year}"` | "tokio emerging patterns best practices 2026" |

### 4.2 Gap-Driven Queries (3 per area)

For areas where the user has sparse coverage, queries cast a wider net:

| Category | Template | Example |
|----------|----------|---------|
| State of the art | `"{area} current best practices"` | "async error handling rust current best practices" |
| Key libraries | `"{area} libraries comparison {current_year}"` | "observability tooling rust libraries comparison 2026" |
| Breaking changes | `"{area} breaking changes deprecations migration"` | "observability tooling breaking changes deprecations migration" |

### 4.3 Execution

Queries execute via Claude Code's native web search capability. No external API keys
or rate limiting configuration is needed — search is covered by the user's Claude Max
subscription.

---

## 5. Stage 3: Novelty Filtering

After web searches return results, the skill filters candidates against the existing
knowledge base to surface only genuinely novel findings.

### 5.1 Load Existing Knowledge

For each scanning area, query the knowledge base for entries with overlapping tags:

```bash
mnemosyne query --tags <area_tags> --format json
```

This returns structured entry data including titles, tags, body content, confidence
levels, and `last_validated` dates. The skill holds this in conversation context.

### 5.2 Filter Criteria

For each candidate finding from web search, the skill evaluates three dimensions:

**Title/topic overlap** — Does an existing entry already cover this specific topic?
A search result describing bounded channels in tokio is a duplicate if there is
already an entry titled "Bounded channels prevent backpressure".

**Substance overlap** — Even with different titles, does the finding's core insight
already appear in an existing entry's body? A result about "tokio channel sizing"
may be covered by an existing entry that discusses channel bounds.

**Temporal novelty** — Is the finding a development after the existing entry's
`last_validated` date? A release from 2026-03 is novel relative to an entry last
validated in 2025-12, even if the topic overlaps. These are flagged as `[Update]`
rather than `[New]`.

Claude performs this evaluation using its reasoning — comparing search results against
knowledge entries held in context. There is no fuzzy matching algorithm; the LLM is
the relevance filter.

### 5.3 Output

A filtered list of findings, each tagged as:

- **[New]** — No existing entry covers this topic
- **[Update]** — Relates to an existing entry but describes a newer development

---

## 6. Stage 4: Batch Presentation & Selection

Filtered findings are presented as a numbered batch grouped by scanning area.

### 6.1 Format

```
Horizon scan found 7 novel developments in your selected areas:

Rust async ecosystem:
  1. tokio 2.0 — automatic back-pressure for unbounded channels [Update]
  2. async-std officially archived, community migrating to smol [New]
  3. Rust 2026 edition stabilises async iterators [New]

Async error handling (gap):
  4. error-stack 3.0 — structured error context with span traces [New]
  5. Emerging pattern: typed error channels in actor frameworks [New]

Observability tooling (gap):
  6. OpenTelemetry Rust SDK reaches 1.0 stable [Update]
  7. tracing-forest — hierarchical span visualisation gaining adoption [New]

Select entries to create (comma-separated numbers, 'all', or 'none'):
```

### 6.2 Conventions

- Grouped by scanning area for context
- Each finding gets a one-line summary
- `[New]` vs `[Update]` tag indicates novelty type
- `none` is always a valid response — scanning may surface nothing worth keeping
- The user may ask for more detail on specific numbers before deciding; the skill
  elaborates from search results in context

---

## 7. Stage 5: Entry Generation

For each selected finding, the skill generates a prospective knowledge entry following
the established format conventions.

### 7.1 Frontmatter

```yaml
---
title: <descriptive title>
tags: [<scanning area base tags>, <finding-specific tags>]
created: <today's date>
last_validated: <today's date>
confidence: prospective
source: horizon-scan
origins: []
supersedes: []
---
```

All horizon-scan entries use:
- `confidence: prospective` — not yet validated by hands-on experience
- `source: horizon-scan` — identifies provenance as exploration, not project work
- `origins: []` — empty until validated in an actual project

### 7.2 Body Structure

```markdown
## Why This Matters

<2-3 sentences synthesised from search results, contextualised to the user's
existing knowledge. Explains relevance to the user's ecosystem and practice,
not a copy of the search snippet.>

## Open Questions

- <question the user would need to answer through hands-on experience>
- <question about applicability to their specific context>
- <question about trade-offs or migration path>
```

### 7.3 Tag Derivation

Tags combine:
- The scanning area's base tags (e.g., `rust`, `async`)
- Topic-specific tags from the finding (e.g., `tokio`, `back-pressure`, `channels`)

### 7.4 Axis Placement

Inferred from tags using the same logic as `mnemosyne promote`:
- Language tags → `languages/`
- Tool tags → `tools/`
- Domain tags → `domains/`
- Technique tags → `techniques/`

### 7.5 [Update] Entries

Findings tagged `[Update]` include a cross-reference in the body:

```markdown
## Why This Matters

<synthesis>

*Relates to existing entry: "Bounded channels prevent backpressure". This
development may warrant updating that entry during your next curation session.*

## Open Questions

- <questions>
```

This connects new developments to existing knowledge and flags curation opportunities.

### 7.6 Persistence

The skill writes entries as markdown files directly to `~/.mnemosyne/knowledge/<axis>/`.
This bypasses `mnemosyne promote` intentionally: promote's interactive flow is designed
for single entries with project origins and contradiction detection, neither of which
applies to batch-generated prospective entries with `origins: []`.

The skill:
1. Determines the axis directory from the tags (Section 7.4)
2. Generates a filename from the title (kebab-case, `.md` extension)
3. Writes the complete markdown file (frontmatter + body)
4. Reports the path: "Created knowledge/{axis}/{filename}"

If `~/.mnemosyne/knowledge/` does not exist, the skill warns the user and suggests
running `mnemosyne init` first.

---

## 8. Integration with Explore Session

### 8.1 Skill Structure

The redesigned `explore-knowledge` skill retains its existing section structure:

| Section | Content | Changed? |
|---------|---------|----------|
| 1. Prerequisites | Check mnemosyne, load status | No |
| 2. Mode selection | Present three modes | No |
| 3. Gap analysis | Identify thin/missing areas | No |
| 4. Horizon scanning | Full pipeline (this spec) | **Yes — replaces placeholder** |
| 5. Open questions | Review low/prospective entries | No |
| 6. Summary | Session metrics | Updated to include horizon metrics |

### 8.2 Data Flow Between Modes

When running all three modes in sequence:

- Gap analysis produces a list of identified gaps and computes tag frequency data
- Horizon scanning receives these as input for scanning area suggestions (Stage 1)
- Horizon scanning creates prospective entries that become part of the knowledge base
- Open questions may surface these new entries for immediate discussion

When running horizon scanning standalone:

- The skill loads tag frequency and co-occurrence data from the CLI
- Gap analysis results are not available, so suggestions come from active domains only

### 8.3 Summary Update

The session summary (Section 6) adds horizon-scanning metrics:

```
Exploration session complete:
  Gaps identified: N
  Horizon scan: searched K areas, found M developments, created P entries
  Open questions resolved: J
```

---

## 9. Evaluation

### 9.1 Existing Harness Compatibility

The eval/quality harness evaluates prospective entries without modification. The
four rubric dimensions (specificity, actionability, provenance, confidence fit)
apply to horizon-scan entries — the 7 existing eval corpus entries with
`source: horizon-scan` already demonstrate this.

### 9.2 Additional Eval Corpus Entries

Add 3-5 prospective entries to the eval corpus that represent typical horizon-scan
output, covering both `[New]` and `[Update]` finding types. These complement the
existing 7 prospective entries and broaden the sample for quality scoring.

### 9.3 Horizon Relevance Rubric Dimension

Add one dimension to the entry quality rubric specific to prospective entries:

```yaml
horizon_relevance:
  description: "Does the entry identify a development genuinely useful to a
    practitioner in the tagged domains?"
  applies_to: prospective  # only scored for prospective entries
  anchors:
    5: "Identifies a specific, recent development with clear implications
        for existing practice. A practitioner would want to know this."
    4: "Identifies a relevant development but implications require some
        interpretation."
    3: "Identifies something real but relevance to the tagged domains is
        moderate."
    2: "Vaguely relevant trend without actionable signal."
    1: "Generic or outdated information with no practical relevance."
```

### 9.4 Manual Spot-Checking Protocol

The full pipeline involves live web search and is non-deterministic. Automated
end-to-end regression testing is not practical. Instead:

1. Run `/explore-knowledge` in horizon scanning mode
2. Select all generated entries
3. Run the eval/quality harness against the live knowledge store:
   ```bash
   python -m eval.quality --store ~/.mnemosyne/knowledge/ --verbose
   ```
4. Review per-entry scores for the new prospective entries
5. Entries scoring below 3 on any dimension warrant revision of the skill prompt

This protocol fits naturally into the existing evaluation workflow and requires no
new tooling.

---

## 10. Session Statefulness

Horizon scanning is stateless across sessions. Each scan is independent — there is
no log of what was scanned or when.

Redundancy is handled by the novelty filter: previous scanning sessions created
prospective entries that now exist in the knowledge base. When the same area is
scanned again, those entries are loaded in Stage 3 and findings that overlap with
them are filtered out.

This means:
- No new persistence mechanisms (no scan log files or metadata)
- Natural deduplication through the knowledge base itself
- The user can re-scan any area at any time without coordination

---

## 11. What This Spec Does Not Cover

- Changes to the Rust core library or CLI (no new subcommands, dependencies, or code)
- Changes to other Claude Code skills (promote-global, curate-global, begin-work)
- Web search provider selection or API key management (uses Claude Code's native search)
- Rate limiting or cost management (covered by the user's Claude Max subscription)
- Automated end-to-end testing of the live pipeline (non-deterministic by nature)
- Session history or scan logging (explicitly excluded per design decision)
