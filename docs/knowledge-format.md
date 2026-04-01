# Knowledge File Format

This document specifies the format for knowledge entries stored in `~/.mnemosyne/knowledge/`.

## Contents

- [Overview](#overview)
- [YAML Frontmatter](#yaml-frontmatter)
- [Body Format](#body-format)
- [Priority Codes](#priority-codes)
- [Superseded Section](#superseded-section)
- [Prospective Entries](#prospective-entries)
- [Axis Conventions](#axis-conventions)
- [Complete Examples](#complete-examples)

---

## Overview

Each knowledge entry is a Markdown file with YAML frontmatter. The structure is:

```
---
<frontmatter>
---

<body>
```

Files are stored under `~/.mnemosyne/knowledge/<axis>/<filename>.md`. Filenames are derived from titles by lowercasing, replacing non-alphanumeric characters with hyphens, and collapsing consecutive hyphens.

The format is designed to be:
- **Human-readable and editable** — a plain text editor is sufficient for all operations
- **Machine-parseable** — the CLI parses and serialises entries deterministically
- **Diff-friendly** — entries evolve by appending, not in-place editing of body content

---

## YAML Frontmatter

All fields are required unless marked optional.

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Human-readable name for the entry. Used in query output headings. |
| `tags` | list of strings | Cross-referencing tags. Used for retrieval, contradiction detection, and axis inference. |
| `created` | date (YYYY-MM-DD) | Date the entry was first created. Set automatically on promotion; do not change. |
| `last_validated` | date (YYYY-MM-DD) | Date the entry was last confirmed as still accurate. Updated by `curate` → validate. |
| `confidence` | enum | Epistemic confidence level. See [Confidence Levels](#confidence-levels) below. |
| `source` | string (optional) | Source or context description. For manually created entries, a short note on provenance. |
| `origins` | list of objects (optional) | Provenance trail: which projects contributed to this entry. See [Origins](#origins) below. |
| `supersedes` | list of strings (optional) | Filenames of entries this one supersedes. Informational; the superseded content is also recorded inline. |

### Confidence Levels

The `confidence` field accepts one of four values:

| Value | Meaning |
|-------|---------|
| `high` | Validated across multiple projects or in a high-stakes context. You would stake a production deployment on it. |
| `medium` | Validated in one project, very likely broadly applicable. The default confidence for a successfully promoted learning. |
| `low` | Observed once, tentative. Recorded because it may matter, not because it is certain. |
| `prospective` | Awareness of a possibility, not yet validated by hands-on experience. Typically from horizon scanning or secondhand reports. Clearly distinguished from experience-validated knowledge. |

### Origins

Each entry in the `origins` list has three fields:

| Field | Type | Description |
|-------|------|-------------|
| `project` | string | Name of the project where the learning was observed. |
| `date` | date (YYYY-MM-DD) | Date of the observation. |
| `context` | string | Brief description of the context in which it was observed. |

Multiple origins accumulate over time as the same learning is confirmed in additional projects. This is the provenance trail.

### Frontmatter Example

```yaml
---
title: Rust Async Channel Patterns
tags: [rust, async, tokio, channels, concurrency]
created: 2026-01-15
last_validated: 2026-03-20
confidence: high
source: hands-on experience
origins:
  - project: api-server
    date: 2026-01-15
    context: "Async bridge implementation — memory exhaustion incident"
  - project: cli-tool
    date: 2026-03-20
    context: "Background job queue implementation"
supersedes: []
---
```

---

## Body Format

The body is freeform Markdown following the frontmatter. Conventions help the CLI and human readers navigate entries consistently.

### Structure

Body content is organised into H2 (`##`) sections. Each section represents a distinct insight or observation under the entry's topic. Entries accumulate new sections over time; existing sections are not edited in place (append-only evolution preserves diff clarity).

### Date-Stamped Entries

Each substantive observation within a section is preceded by a bold date:

```markdown
**2026-01-15:** <observation text>
```

This date corresponds to when the observation was recorded, not necessarily the date in `created`. When multiple observations are added over time, they form a chronological record:

```markdown
## Bounded channels prevent backpressure

**2026-01-15:** Always use bounded channels in tokio. Unbounded channels
caused memory exhaustion under sustained load in the api-server project.

**2026-03-20:** Confirmed in cli-tool's background job queue. Bound should
be sized to expected burst load; arbitrarily large bounds defeat the purpose.
```

---

## Priority Codes

Priority codes follow the date stamp in observations, carried from the per-project observational-memory system:

| Code | Priority | Meaning |
|------|----------|---------|
| `🔴` | Critical | Failure mode or hard constraint. Must be respected. Ignoring this causes failures. |
| `🟡` | Useful | Significant pattern or best practice. Should be followed. |
| `🟢` | Informational | Context, background, or nice-to-know. Worth recording. |
| `⚪` | Neutral | Fleeting note. Low promotion priority. |

Priority codes appear after the bold date:

```markdown
**2026-01-15:** 🔴 Always use bounded channels in tokio. Unbounded channels
caused memory exhaustion under sustained load.

**2026-01-20:** 🟡 Prefer `tokio::sync::mpsc` over `std::sync::mpsc` in async
contexts — the tokio variant is task-aware and does not block the executor.

**2026-01-25:** 🟢 The bound size should match expected burst load; a good
starting point is 2x the maximum observed throughput per second.
```

---

## Superseded Section

When an entry is superseded — either by a new entry or during a curation session — the old content is preserved inline in a `## Superseded` section appended to the body. Nothing is deleted.

### Format

```markdown
## Superseded

### <old content summary> (<start date> → <supersede date>)
> <old content summary>

**Reason superseded:** <reason text>
```

### Example

```markdown
## Bounded channels prevent backpressure

**2026-01-15:** 🔴 Always use bounded channels in tokio. Unbounded channels
caused memory exhaustion under sustained load.

**2027-02-01:** 🟡 tokio 2.0 introduced automatic back-pressure for unbounded
channels under memory pressure. Unbounded channels are now safer than before,
but bounded channels remain the preferred pattern for predictable latency.

## Superseded

### Use bounded channels unconditionally (2026-01-15 → 2027-02-01)
> Use bounded channels unconditionally

**Reason superseded:** tokio 2.0 changed back-pressure semantics; updated
guidance now qualifies when bounded channels are required.
```

Multiple supersession records accumulate in the same `## Superseded` section. Each is a separate H3 subsection.

---

## Prospective Entries

Prospective entries represent awareness of possibilities, techniques, or approaches that have not yet been validated by hands-on experience. They are clearly distinguished from experience-validated knowledge.

### Frontmatter for Prospective Entries

```yaml
---
title: Zig as a C Interoperability Layer
tags: [zig, c, ffi, interoperability]
created: 2026-02-10
last_validated: 2026-02-10
confidence: prospective
source: horizon-scan
origins: []
supersedes: []
---
```

The `source: horizon-scan` convention identifies entries created during exploration sessions rather than from direct project experience.

### Body Convention for Prospective Entries

Prospective entries typically use a `## Why This Matters` section and an `## Open Questions` section:

```markdown
## Why This Matters

Zig's explicit control over memory layout and its C ABI compatibility make it
a candidate for writing high-performance interoperability layers without the
complexity of C++ or the overhead of Rust's FFI ergonomics.

## Open Questions

- Does Zig's safety story at the FFI boundary hold up in practice?
- How does the toolchain interact with existing C build systems (CMake, Meson)?
- What is the runtime overhead of Zig's optionals at C function boundaries?
```

Prospective entries are revisited during exploration sessions. When hands-on experience validates or contradicts them, confidence is updated and observations are added to the body.

---

## Axis Conventions

The five top-level axes provide browsable, human-navigable organisation. Tags handle cross-cutting queries regardless of which axis an entry lives in.

### `languages/`

Entries about programming language semantics, idioms, patterns, and pitfalls.

**File naming convention:** `<language>-<topic>.md`

Examples:
- `rust-ownership-patterns.md`
- `python-asyncio-pitfalls.md`
- `swift-concurrency-model.md`
- `haskell-monad-transformer-stacks.md`

**Tag convention:** Always include the language name as a tag. Add tags for relevant sub-topics (e.g., `async`, `type-system`, `testing`).

### `domains/`

Entries about application domains, platforms, and problem spaces that cut across languages.

**File naming convention:** `<domain>-<topic>.md`

Examples:
- `macos-appkit-delegate-patterns.md`
- `web-api-pagination.md`
- `database-connection-pooling.md`
- `networking-tls-certificate-rotation.md`

**Tag convention:** Include the domain or platform name as a tag. Add sub-domain and technology tags.

### `tools/`

Entries about development tools, build systems, package managers, and deployment infrastructure.

**File naming convention:** `<tool>-<topic>.md`

Examples:
- `cargo-workspace-organisation.md`
- `git-rebase-workflow.md`
- `docker-layer-caching.md`
- `xcode-signing-configuration.md`

**Tag convention:** Include the tool name as a tag. Add operation and context tags.

### `techniques/`

Entries about development techniques, design patterns, testing approaches, and cross-cutting practices that are language- and tool-agnostic.

**File naming convention:** `<technique>.md` or `<technique>-<aspect>.md`

Examples:
- `tdd-outside-in.md`
- `error-handling-domain-types.md`
- `async-patterns.md`
- `code-review-checklist.md`

**Tag convention:** Descriptive topic tags. Optionally include language tags if the technique is language-specific in practice.

### `projects/`

Entries about specific codebases: architectural decisions, project-specific idioms, and cross-references to other knowledge.

**File naming convention:** `<project-name>.md`

Examples:
- `api-server.md`
- `cli-tool.md`
- `mnemosyne.md`

**Tag convention:** Include the project name and the primary language(s) and domain(s) as tags.

---

## Complete Examples

### A High-Confidence Language Entry

```markdown
---
title: Rust Ownership and the Borrow Checker
tags: [rust, ownership, borrowing, lifetimes, memory]
created: 2025-06-01
last_validated: 2026-01-10
confidence: high
source: hands-on experience
origins:
  - project: api-server
    date: 2025-06-01
    context: "Initial Rust adoption — learning curve"
  - project: cli-tool
    date: 2025-11-15
    context: "Async refactor — lifetime issues with futures"
  - project: mnemosyne
    date: 2026-01-10
    context: "Entry parsing — lifetime-annotated references in parse functions"
supersedes: []
---

## Shared references require explicit lifetime annotation in structs

**2025-06-01:** 🔴 If a struct holds a reference, it requires a lifetime
parameter. The compiler enforces this. Forgetting lifetime parameters is one
of the most common early Rust errors.

**2025-11-15:** 🟡 Lifetimes in async contexts interact with `Send` bounds.
A reference that is `!Send` makes the future `!Send`, preventing it from being
spawned on a multi-threaded executor.

## Ownership transfers are moves, not copies

**2025-06-15:** 🟡 Assignment moves ownership for non-`Copy` types. After
`let b = a;`, `a` is invalid. Use `.clone()` to keep both, or pass references
to avoid moves.

**2026-01-10:** 🟢 This is different from C++ move semantics — Rust's moves
are zero-cost and enforced at compile time, not convention.
```

### A Prospective Entry from Horizon Scanning

```markdown
---
title: Effect Systems as a Replacement for Monad Transformers
tags: [haskell, effects, algebraic-effects, type-system]
created: 2026-03-15
last_validated: 2026-03-15
confidence: prospective
source: horizon-scan
origins: []
supersedes: []
---

## Why This Matters

Effect system libraries (Effectful, Cleff, Polysemy) promise to replace
monad transformer stacks with a cleaner programming model that avoids
the `n*m` instance problem and provides better error messages.

## Open Questions

- Does Effectful's performance match a hand-rolled `ReaderT IO` stack?
- What is the migration path from existing transformer-based code?
- Do effect systems compose well with streaming libraries like Conduit?
- Has the ecosystem (testing, logging, etc.) caught up with Effectful?
```

### An Entry with a Superseded Section

```markdown
---
title: Python Async HTTP Clients
tags: [python, async, http, aiohttp, httpx]
created: 2024-08-01
last_validated: 2026-02-20
confidence: high
source: hands-on experience
origins:
  - project: data-pipeline
    date: 2024-08-01
    context: "External API integration"
  - project: web-scraper
    date: 2026-02-20
    context: "Migration from aiohttp to httpx"
supersedes: [python-aiohttp-patterns.md]
---

## httpx is the preferred async HTTP client for new projects

**2026-02-20:** 🟡 httpx provides both sync and async interfaces with a
compatible API, making testing easier. The client is fully type-annotated
and the connection pool management is simpler than aiohttp.

## Superseded

### Use aiohttp for async HTTP requests (2024-08-01 → 2026-02-20)
> Use aiohttp for async HTTP requests

**Reason superseded:** httpx has matured and provides a better developer
experience with type annotations and dual sync/async API. aiohttp remains
viable for high-performance use cases but is no longer the default recommendation.
```
