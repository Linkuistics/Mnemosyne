# Horizon-Scanning Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the placeholder horizon-scanning section in the `explore-knowledge` Claude Code skill with a fully defined five-stage pipeline (scanning area selection, web search, novelty filtering, batch presentation, entry generation), and add evaluation support for the new entry type.

**Architecture:** All logic lives in the skill prompt — no Rust core changes. The skill instructs Claude Code to use web search, query the knowledge base via `mnemosyne query`, and write prospective entries as markdown files directly to `~/.mnemosyne/knowledge/<axis>/`. Evaluation additions are: 4 new corpus entries demonstrating the expected output format, one new rubric dimension (`horizon_relevance`) with conditional application, and a small code change to `rubric.py` and `__main__.py` to support the `applies_to` field.

**Tech Stack:** Markdown (skill prompt), YAML (rubric, corpus frontmatter), Python (eval harness modifications)

---

## File Structure

### Skill (single file rewrite)

- Modify: `adapters/claude-code/skills/explore-knowledge.md` — Replace Section 4 with the full horizon-scanning pipeline

### Evaluation Additions

- Create: `eval/corpus/entries/techniques-effect-systems.md` — [New]-type prospective entry
- Create: `eval/corpus/entries/tools-rust-analyzer-updates.md` — [Update]-type prospective entry
- Create: `eval/corpus/entries/domains-wasm-server-side.md` — [New]-type prospective entry
- Create: `eval/corpus/entries/languages-swift-typed-throws.md` — [New]-type prospective entry
- Modify: `eval/quality/rubrics/entry_quality.yaml` — Add `horizon_relevance` dimension with `applies_to: prospective`
- Modify: `eval/quality/src/rubric.py` — Support `applies_to` field for conditional dimension filtering
- Modify: `eval/quality/src/__main__.py` — Pass entry confidence to rubric formatting so conditional dimensions work

---

## Task 1: Rewrite the explore-knowledge skill

**Files:**
- Modify: `adapters/claude-code/skills/explore-knowledge.md`

This is the core deliverable. The existing skill has 6 sections; Section 4 (horizon scanning) is replaced with the full pipeline from the spec. Sections 1-3 and 5 are unchanged. Section 6 (summary) gets updated metrics.

- [ ] **Step 1: Replace Section 4 with the full horizon-scanning pipeline**

Replace lines 53-71 of `adapters/claude-code/skills/explore-knowledge.md` (the current Section 4) with the following:

```markdown
## 4. Horizon scanning

Horizon scanning discovers new developments in the user's active ecosystems and creates
prospective knowledge entries. It runs a five-stage pipeline.

### Stage 1: Suggest scanning areas

Build a ranked list of candidate scanning areas from two sources:

**Gap-driven areas (priority 1):** If gap analysis ran in Stage 3, use its results. Areas
flagged as thin or missing are high-value scan targets. Example: "You have 5 entries tagged
`tokio` but nothing on `error-handling` in async contexts" → suggest "async error handling".

**Active-domain areas (priority 2):** Query the knowledge base:

```bash
mnemosyne query --format json
```

Count tag occurrences across all entries. Identify:
- The top 5-8 tags by frequency (the user's core ecosystems)
- Co-occurring tag pairs that appear together in 3+ entries (compound topics worth scanning
  as a unit, e.g. `rust + async`)

If horizon scanning is running standalone (without prior gap analysis), only active-domain
areas are available.

Present suggestions grouped by source:

> Based on your knowledge base, here are suggested scanning areas:
>
> From gaps:
>   1. Async error handling (gap: active in async but no error handling entries)
>   2. Observability tooling (gap: 1 entry, mostly prospective)
>
> From active domains:
>   3. Rust ecosystem (12 entries)
>   4. Tokio / async runtime (8 entries)
>   5. Swift concurrency (4 entries)
>
> Or describe your own area to scan.
>
> Which areas? (comma-separated numbers, or type a custom topic)

The user selects by number, enters a custom topic, or both.

### Stage 2: Web search

For each selected area, formulate and execute targeted web search queries using Claude Code's
native web search. The query count depends on the area type:

**Active-domain areas (2 queries):** The user has established knowledge; focus on what's new.
- `"{area} new releases {current_year}"`
- `"{area} emerging patterns best practices {current_year}"`

**Gap-driven areas (3 queries):** The user has sparse coverage; cast a wider net.
- `"{area} current best practices"`
- `"{area} libraries comparison {current_year}"`
- `"{area} breaking changes deprecations migration"`

Execute all queries for all selected areas before proceeding to filtering.

### Stage 3: Novelty filtering

Filter search results against the existing knowledge base to surface only genuinely novel
findings.

Load existing entries with overlapping tags:

```bash
mnemosyne query --tags <area_tags> --format json
```

For each candidate finding from web search, evaluate:

- **Title/topic overlap** — Does an existing entry already cover this topic?
- **Substance overlap** — Does the finding's core insight already appear in an existing
  entry's body, even under a different title?
- **Temporal novelty** — Is this a development after the existing entry's `last_validated`
  date? If so, flag as `[Update]` rather than `[New]`.

Discard findings that duplicate existing knowledge. Keep findings that are:
- **[New]** — no existing entry covers this topic
- **[Update]** — relates to an existing entry but describes a newer development

### Stage 4: Batch presentation and selection

Present filtered findings as a numbered list grouped by scanning area:

> Horizon scan found N novel developments in your selected areas:
>
> Rust async ecosystem:
>   1. tokio 2.0 — automatic back-pressure for unbounded channels [Update]
>   2. async-std officially archived, community migrating to smol [New]
>
> Async error handling (gap):
>   3. error-stack 3.0 — structured error context with span traces [New]
>
> Select entries to create (comma-separated numbers, 'all', or 'none'):

If the user asks for more detail on a specific number before deciding, elaborate from the
search results in context. `none` is always valid — scanning may surface nothing worth keeping.

### Stage 5: Generate prospective entries

For each selected finding, write a prospective knowledge entry directly to
`~/.mnemosyne/knowledge/<axis>/`.

**Frontmatter:**

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

**Body — two sections:**

```markdown
## Why This Matters

<2-3 sentences synthesised from search results, contextualised to the user's existing
knowledge. Explains relevance, not a copy of the search snippet.>

## Open Questions

- <question requiring hands-on experience to answer>
- <question about applicability to their specific context>
- <question about trade-offs or migration path>
```

**For [Update] findings**, add a cross-reference in the body:

> *Relates to existing entry: "<entry title>". This development may warrant updating
> that entry during your next curation session.*

**Axis placement** — infer from tags:
- Language tags → `languages/`
- Tool tags → `tools/`
- Domain tags → `domains/`
- Technique tags → `techniques/`

**Filename** — kebab-case from the title, `.md` extension. Example: "Tokio 2.0 Back-Pressure
Semantics" → `tokio-2-0-back-pressure-semantics.md`.

If `~/.mnemosyne/knowledge/` does not exist, warn the user and suggest `mnemosyne init`.

After writing all entries, report each path: "Created knowledge/{axis}/{filename}".
```

- [ ] **Step 2: Update Section 6 (summary) to include horizon-scanning metrics**

Replace the current Section 6:

```markdown
## 6. Summary

After the session, report:
- Gaps identified: N
- Candidate entries created: M
- Prospective entries added: K
- Open questions resolved: J

Suggest scheduling the next exploration session: "Consider running /explore-knowledge again after your next major project to capture new patterns."
```

With:

```markdown
## 6. Summary

After the session, report:
- Gaps identified: N
- Candidate entries created (from gap analysis): M
- Horizon scan: searched K areas, found F developments, created P prospective entries
- Open questions resolved: J

Suggest scheduling the next exploration session: "Consider running /explore-knowledge again after your next major project to capture new patterns."
```

- [ ] **Step 3: Verify the complete skill file reads correctly**

Read `adapters/claude-code/skills/explore-knowledge.md` end to end. Verify:
- Section numbering is consistent (1-6)
- No orphaned references to `mnemosyne promote` in Section 4 (replaced with direct file writes)
- The frontmatter `name` and `description` are unchanged
- Sections 1, 2, 3, 5 are unmodified from the original

- [ ] **Step 4: Commit**

```bash
git add adapters/claude-code/skills/explore-knowledge.md
git commit -m "feat(skill): implement horizon-scanning pipeline in explore-knowledge

Replace the placeholder Section 4 with a five-stage pipeline: scanning area
selection, web search, novelty filtering, batch presentation, and prospective
entry generation. Update summary metrics in Section 6."
```

---

## Task 2: Add evaluation corpus entries

**Files:**
- Create: `eval/corpus/entries/techniques-effect-systems.md`
- Create: `eval/corpus/entries/tools-rust-analyzer-updates.md`
- Create: `eval/corpus/entries/domains-wasm-server-side.md`
- Create: `eval/corpus/entries/languages-swift-typed-throws.md`

These 4 entries demonstrate the expected output format for horizon-scan entries. They cover both `[New]` and `[Update]` types, multiple axes, and varying tag densities. They follow the "Why This Matters" + "Open Questions" body structure from the spec.

- [ ] **Step 1: Create techniques-effect-systems.md ([New] type)**

```markdown
---
title: Effect Systems as Replacement for Monad Transformers
tags: [haskell, effect-systems, functional, architecture]
created: 2026-03-20
last_validated: 2026-03-20
confidence: prospective
source: horizon-scan
origins: []
supersedes: []
---

## Why This Matters

Effect system libraries (Effectful, Cleff, Polysemy) promise to replace monad transformer
stacks with a cleaner programming model that avoids the `n*m` instance problem and provides
better error messages. Effectful in particular claims performance on par with `ReaderT IO`
while supporting modular effect composition.

This is relevant to any Haskell codebase currently using `mtl` or `transformers` stacks,
especially those where adding a new effect requires touching many layers.

## Open Questions

- Does Effectful's performance hold under real workloads with 5+ effects in the stack?
- What is the migration path from an existing `mtl`-based codebase?
- Do effect systems compose well with streaming libraries like Conduit or Pipes?
- Has the ecosystem (testing, logging, database access) caught up with Effectful?
```

- [ ] **Step 2: Create tools-rust-analyzer-updates.md ([Update] type)**

```markdown
---
title: Rust-Analyzer Procedural Macro Expansion Improvements
tags: [rust, tooling, rust-analyzer, ide]
created: 2026-03-25
last_validated: 2026-03-25
confidence: prospective
source: horizon-scan
origins: []
supersedes: []
---

## Why This Matters

Recent rust-analyzer releases have significantly improved procedural macro expansion,
reducing the frequency of "proc macro not expanded" errors that disrupted IDE workflows.
The improvement covers derive macros, attribute macros, and function-like macros used by
frameworks like `serde`, `clap`, and `sqlx`.

*Relates to existing entry: "Cargo Workspaces". This development may warrant updating
that entry during your next curation session, particularly regarding IDE experience
in multi-crate workspaces.*

## Open Questions

- Does the improved expansion handle workspace-level proc macros without per-crate
  configuration?
- What is the memory overhead of expanded macros in large codebases?
- Are `sqlx` compile-time checked queries now fully supported in the IDE?
```

- [ ] **Step 3: Create domains-wasm-server-side.md ([New] type)**

```markdown
---
title: WebAssembly for Server-Side Sandboxing
tags: [wasm, security, sandboxing, server]
created: 2026-03-18
last_validated: 2026-03-18
confidence: prospective
source: horizon-scan
origins: []
supersedes: []
---

## Why This Matters

The WebAssembly Component Model and WASI preview 2 are enabling server-side use of Wasm
as a lightweight sandboxing mechanism. Runtimes like Wasmtime and WasmEdge allow executing
untrusted code with fine-grained capability control (filesystem, network, clock access),
offering an alternative to containers for plugin systems and multi-tenant workloads.

This is relevant to any service that runs user-provided or third-party code and needs
isolation without the overhead of full container orchestration.

## Open Questions

- What is the cold-start latency for Wasm modules compared to container startup?
- How mature is the Component Model for composing modules from different languages?
- Can Wasm sandboxes enforce memory limits reliably enough for production multi-tenancy?
- What is the debugging experience for Wasm guests — are source maps and stack traces usable?
```

- [ ] **Step 4: Create languages-swift-typed-throws.md ([New] type)**

```markdown
---
title: Swift Typed Throws
tags: [swift, error-handling, type-system]
created: 2026-03-22
last_validated: 2026-03-22
confidence: prospective
source: horizon-scan
origins: []
supersedes: []
---

## Why This Matters

Swift 6 introduces typed throws (`throws(SomeError)`), allowing functions to declare the
specific error type they throw rather than the opaque `any Error`. This enables exhaustive
`catch` blocks and eliminates the need for runtime type checking in error handling paths.

This changes the error handling ergonomics for any Swift codebase that currently relies on
`do/catch` with `as?` casts to distinguish error types — a pattern that is fragile and
produces runtime failures when new error cases are added.

## Open Questions

- Does typed throws compose well across module boundaries, or does it create coupling?
- What is the migration path for existing code that throws `any Error`?
- How does typed throws interact with async/await and structured concurrency error propagation?
```

- [ ] **Step 5: Verify all 4 entries pass structural validation**

```bash
cd eval/quality && python -c "
from eval.quality.src.structural import check_entry
import os

entries_dir = os.path.join(os.path.dirname(os.path.abspath('.')), 'corpus', 'entries')
new_files = [
    'techniques-effect-systems.md',
    'tools-rust-analyzer-updates.md',
    'domains-wasm-server-side.md',
    'languages-swift-typed-throws.md',
]
for f in new_files:
    path = os.path.join(entries_dir, f)
    with open(path) as fh:
        result = check_entry(fh.read(), f)
    status = 'PASS' if result.valid else f'FAIL: {result.errors}'
    print(f'{f}: {status}')
"
```

Expected: all 4 entries show PASS.

- [ ] **Step 6: Commit**

```bash
git add eval/corpus/entries/techniques-effect-systems.md \
        eval/corpus/entries/tools-rust-analyzer-updates.md \
        eval/corpus/entries/domains-wasm-server-side.md \
        eval/corpus/entries/languages-swift-typed-throws.md
git commit -m "feat(eval): add 4 horizon-scan corpus entries for quality benchmarking

Add prospective entries covering effect systems, rust-analyzer updates,
Wasm server-side sandboxing, and Swift typed throws. These demonstrate
the [New] and [Update] entry formats from the horizon-scanning spec."
```

---

## Task 3: Add horizon_relevance rubric dimension

**Files:**
- Modify: `eval/quality/rubrics/entry_quality.yaml`

- [ ] **Step 1: Add the horizon_relevance dimension to the rubric**

Append to `eval/quality/rubrics/entry_quality.yaml` after the `confidence_fit` dimension:

```yaml

  horizon_relevance:
    description: "Does the entry identify a development genuinely useful to a practitioner in the tagged domains?"
    applies_to: prospective
    anchors:
      5: "Identifies a specific, recent development with clear implications for existing practice. A practitioner would want to know this."
      4: "Identifies a relevant development but implications require some interpretation."
      3: "Identifies something real but relevance to the tagged domains is moderate."
      2: "Vaguely relevant trend without actionable signal."
      1: "Generic or outdated information with no practical relevance."
```

The `applies_to: prospective` field is new — it tells the harness to only score this dimension for entries with `confidence: prospective`. The next task implements support for this field.

- [ ] **Step 2: Commit**

```bash
git add eval/quality/rubrics/entry_quality.yaml
git commit -m "feat(eval): add horizon_relevance rubric dimension for prospective entries

New dimension with applies_to: prospective field. Scored only for
prospective entries to evaluate whether horizon-scan findings identify
genuinely useful developments."
```

---

## Task 4: Support conditional rubric dimensions in the eval harness

**Files:**
- Modify: `eval/quality/src/rubric.py:15-39`
- Modify: `eval/quality/src/__main__.py:40-66`
- Create: `eval/quality/tests/test_rubric.py`

The `applies_to` field means the harness needs to know the entry's confidence level when formatting the rubric prompt. Dimensions with `applies_to` are only included when the entry matches.

- [ ] **Step 1: Write failing test for conditional dimension filtering**

Create `eval/quality/tests/test_rubric.py`:

```python
from eval.quality.src.rubric import format_rubric_prompt


def test_format_rubric_prompt_excludes_conditional_dimensions_by_default():
    rubric = {
        "dimensions": {
            "specificity": {
                "description": "How specific?",
                "anchors": {5: "Very", 1: "Not"},
            },
            "horizon_relevance": {
                "description": "Relevant development?",
                "applies_to": "prospective",
                "anchors": {5: "Very", 1: "Not"},
            },
        }
    }
    prompt = format_rubric_prompt(rubric)
    assert "specificity" in prompt
    assert "horizon_relevance" not in prompt


def test_format_rubric_prompt_includes_conditional_when_matching():
    rubric = {
        "dimensions": {
            "specificity": {
                "description": "How specific?",
                "anchors": {5: "Very", 1: "Not"},
            },
            "horizon_relevance": {
                "description": "Relevant development?",
                "applies_to": "prospective",
                "anchors": {5: "Very", 1: "Not"},
            },
        }
    }
    prompt = format_rubric_prompt(rubric, entry_confidence="prospective")
    assert "specificity" in prompt
    assert "horizon_relevance" in prompt


def test_format_rubric_prompt_excludes_conditional_when_not_matching():
    rubric = {
        "dimensions": {
            "specificity": {
                "description": "How specific?",
                "anchors": {5: "Very", 1: "Not"},
            },
            "horizon_relevance": {
                "description": "Relevant development?",
                "applies_to": "prospective",
                "anchors": {5: "Very", 1: "Not"},
            },
        }
    }
    prompt = format_rubric_prompt(rubric, entry_confidence="high")
    assert "specificity" in prompt
    assert "horizon_relevance" not in prompt
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd eval/quality && python -m pytest tests/test_rubric.py -v
```

Expected: FAIL — `format_rubric_prompt` does not accept `entry_confidence` parameter.

- [ ] **Step 3: Update format_rubric_prompt to support applies_to filtering**

In `eval/quality/src/rubric.py`, replace the `format_rubric_prompt` function:

```python
def format_rubric_prompt(
    rubric: dict[str, Any],
    shuffle: bool = False,
    entry_confidence: str | None = None,
) -> str:
    """Format a rubric into a prompt string for LLM evaluation.

    If shuffle=True, randomize dimension order for variance reduction.
    If entry_confidence is provided, only include dimensions whose applies_to
    field matches or dimensions without an applies_to field.
    """
    dimensions = list(rubric.get("dimensions", {}).items())

    # Filter conditional dimensions
    filtered = []
    for dim_name, dim_spec in dimensions:
        applies_to = dim_spec.get("applies_to")
        if applies_to is None or applies_to == entry_confidence:
            filtered.append((dim_name, dim_spec))
    dimensions = filtered

    if shuffle:
        random.shuffle(dimensions)

    lines = [
        f"Evaluate this knowledge entry on the following dimensions.",
        f"For each dimension, provide a score (1-5) and a one-sentence justification.",
        f"Return your response as a YAML list with fields: dimension, score, justification.",
        "",
    ]

    for dim_name, dim_spec in dimensions:
        lines.append(f"## {dim_name}")
        lines.append(dim_spec.get("description", ""))
        lines.append("")
        for score, anchor in sorted(dim_spec.get("anchors", {}).items(), reverse=True):
            lines.append(f"  {score}: {anchor}")
        lines.append("")

    return "\n".join(lines)
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd eval/quality && python -m pytest tests/test_rubric.py -v
```

Expected: all 3 tests PASS.

- [ ] **Step 5: Update __main__.py to extract entry confidence and pass it to rubric formatting**

In `eval/quality/src/__main__.py`, modify the `evaluate_entries` function. Replace the loop body (lines 41-65) that processes each entry:

```python
    entry_reports = []
    for filename in sorted(os.listdir(entries_dir)):
        if not filename.endswith(".md"):
            continue
        filepath = os.path.join(entries_dir, filename)
        with open(filepath) as f:
            content = f.read()

        if verbose:
            print(f"  Evaluating {filename}...", file=sys.stderr)

        # Extract confidence from frontmatter for conditional dimensions
        entry_confidence = _extract_confidence(content)

        # Pass 1: standard dimension order
        prompt1 = format_rubric_prompt(rubric, shuffle=False, entry_confidence=entry_confidence)
        scores1 = judge.evaluate(content, prompt1)

        if single_pass:
            entry_reports.append(EntryReport(filename=filename, scores=scores1))
            continue

        # Pass 2: shuffled dimension order (variance reduction)
        prompt2 = format_rubric_prompt(rubric, shuffle=True, entry_confidence=entry_confidence)
        scores2 = judge.evaluate(content, prompt2)

        # Average scores across passes, keep justification from pass 1
        averaged = _average_scores(scores1, scores2)
        entry_reports.append(EntryReport(filename=filename, scores=averaged))
```

Add the `_extract_confidence` helper function before `_average_scores`:

```python
def _extract_confidence(content: str) -> str | None:
    """Extract confidence value from entry frontmatter."""
    content = content.strip()
    if not content.startswith("---"):
        return None
    parts = content.split("---", 2)
    if len(parts) < 3:
        return None
    try:
        import yaml
        fm = yaml.safe_load(parts[1])
        if isinstance(fm, dict):
            return str(fm.get("confidence", "")).lower() or None
    except Exception:
        return None
    return None
```

- [ ] **Step 6: Run the existing structural tests to verify no regressions**

```bash
cd eval/quality && python -m pytest tests/ -v
```

Expected: all tests PASS (existing `test_structural.py` + new `test_rubric.py`).

- [ ] **Step 7: Commit**

```bash
git add eval/quality/src/rubric.py \
        eval/quality/src/__main__.py \
        eval/quality/tests/test_rubric.py
git commit -m "feat(eval): support conditional rubric dimensions via applies_to field

Dimensions with applies_to only appear in the rubric prompt when the
entry's confidence level matches. This enables the horizon_relevance
dimension to score only prospective entries."
```

---

## Task 5: Update the evaluation framework plan's coverage matrix

**Files:**
- Modify: `docs/superpowers/plans/2026-04-01-evaluation-framework.md:57-99`

The coverage matrix lists 39 entries. With 4 new entries, update to 43.

- [ ] **Step 1: Add the 4 new entries to the coverage matrix**

Insert after row 39 (projects-microservices-auth.md) in the coverage matrix table:

```markdown
| 40 | techniques-effect-systems.md | techniques | haskell, effect-systems, functional, architecture | prospective | 0 (source: horizon-scan) |
| 41 | tools-rust-analyzer-updates.md | tools | rust, tooling, rust-analyzer, ide | prospective | 0 (source: horizon-scan) |
| 42 | domains-wasm-server-side.md | domains | wasm, security, sandboxing, server | prospective | 0 (source: horizon-scan) |
| 43 | languages-swift-typed-throws.md | languages | swift, error-handling, type-system | prospective | 0 (source: horizon-scan) |
```

- [ ] **Step 2: Update the distribution summary**

Replace:

```markdown
**Distribution:** high=11, medium=13, low=8, prospective=7. Tag density: 1–6.
```

With:

```markdown
**Distribution:** high=11, medium=13, low=8, prospective=10. Tag density: 1–6.
```

Also update the "39 entries" reference in the header area to "43 entries".

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/plans/2026-04-01-evaluation-framework.md
git commit -m "docs: update eval coverage matrix with 4 new horizon-scan corpus entries

Adds techniques-effect-systems, tools-rust-analyzer-updates,
domains-wasm-server-side, and languages-swift-typed-throws. Corpus
grows from 39 to 43 entries, prospective count from 7 to 10."
```
