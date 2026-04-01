# Mnemosyne — Evaluation Framework

## 1. Purpose

This spec defines the infrastructure for evaluating whether Mnemosyne works — not just
whether the code runs correctly, but whether retrieval is accurate, knowledge quality is
high, and the system delivers measurable value to AI assistants over time.

The evaluation framework has two concrete deliverables (Phases 1-2) and two intent-level
designs (Phases 3-4) that will be specified in detail once the foundation is proven.

### Relationship to Existing Tests

The 48 integration tests in `tests/` validate *correctness*: does the code do what the
docstrings say? The evaluation framework validates *quality*: does the code do its job
well enough? The two are complementary and independent. The existing test suite is not
modified by this work.

---

## 2. Corpus (`eval/corpus/`)

The benchmark corpus is the shared ground truth that both harnesses consume. All corpus
files are checked into git and versioned alongside the code.

### 2.1 Knowledge Entries (`eval/corpus/entries/`)

30-50 synthetic knowledge entries in standard Mnemosyne format (YAML frontmatter +
Markdown body).

**Coverage requirements:**
- All five axes represented (languages, domains, tools, techniques, projects)
- All four confidence levels represented (high, medium, low, prospective)
- Tag density ranges from 1 tag to 6+ tags per entry
- Body lengths range from 2-3 sentences to multi-section entries
- At least 3 entries per axis to enable meaningful retrieval benchmarks
- Origins fields vary: some single-project, some multi-project, some absent (prospective)

**Naming convention:** `{axis}-{topic}.md`, e.g. `languages-rust-lifetimes.md`,
`techniques-connection-pooling.md`.

**Quality standard:** These entries must be realistic — the kind of knowledge a developer
would actually promote. They should not be lorem ipsum or trivially generated. Hand-curate
them, drawing on real patterns from the project's domain.

### 2.2 Queries with Relevance Judgements (`eval/corpus/queries.yaml`)

10-20 queries, each with graded relevance labels against the corpus entries.

**Schema:**

```yaml
queries:
  - id: q01
    text: "async error handling"
    tags: [async, error-handling]
    context: null                    # or a project type, e.g. "rust-web-api"
    relevant:
      - entry: techniques-async-error-handling.md
        relevance: 2                 # 2 = highly relevant
      - entry: languages-rust-async.md
        relevance: 1                 # 1 = somewhat relevant
      # entries not listed are relevance 0 (not relevant)
```

**Coverage requirements:**
- Mix of tag-only queries, text-only queries, and combined queries
- At least 3 queries that use context-based filtering (simulated project type)
- Each corpus entry should be relevant to at least one query (no orphan entries)
- Include at least 2 "hard" queries where the relevant entry has low tag overlap
  but high semantic relevance (tests whether the scoring function handles this)

### 2.3 Contradiction Pairs (`eval/corpus/contradictions.yaml`)

5-10 pairs of entry filenames with boolean labels.

**Schema:**

```yaml
pairs:
  - entry_a: languages-rust-async.md
    entry_b: languages-rust-async-updated.md
    is_contradiction: true
    note: "Updated entry supersedes the original on cancellation semantics"

  - entry_a: techniques-connection-pooling.md
    entry_b: domains-database-connections.md
    is_contradiction: false
    note: "High tag overlap but complementary content — pooling strategy vs connection lifecycle"
```

**Coverage requirements:**
- At least 3 true contradictions (entries that genuinely conflict)
- At least 3 false positives (high tag overlap but no semantic contradiction)
- At least 1 near-miss: entries that share 2+ tags but discuss different aspects of the
  same topic
- The corpus entries directory must contain both members of each pair

### 2.4 Mock Projects (`eval/corpus/projects/`)

3-5 minimal directory trees with file markers for context detection testing.

**Structure per mock project:**

```
eval/corpus/projects/
  rust-web-api/
    Cargo.toml         # minimal, with relevant dependencies
    src/
      main.rs          # can be empty — detection uses file presence, not content
    expected.yaml      # expected detection results
  python-ml-pipeline/
    requirements.txt
    setup.py
    src/
      __init__.py
    expected.yaml
  typescript-react/
    package.json       # with react dependency
    tsconfig.json
    src/
      index.tsx
    expected.yaml
```

**`expected.yaml` schema:**

```yaml
languages: [rust]
dependencies: [actix-web, serde, tokio]
expected_tags: [rust, web, async]
```

**Coverage requirements:**
- At least one project per major language in the config (Rust, Python, TypeScript)
- At least one multi-language project (e.g. Python + TypeScript)
- File markers should match what `detect.rs` actually looks for

---

## 3. Rust Evaluation Harness (`eval/src/`)

A separate Rust binary crate that depends on the `mnemosyne` library crate. It loads
the corpus, runs the library's search/detection/contradiction functions, and reports
quantitative metrics.

### 3.1 Crate Structure

```
eval/
  Cargo.toml            # [package] name = "mnemosyne-eval"
  src/
    main.rs             # CLI entry point (clap)
    corpus.rs           # Load and parse corpus files, queries.yaml, contradictions.yaml
    retrieval.rs        # MRR, precision@k, recall@k, nDCG computation
    contradiction.rs    # F1, precision, recall for contradiction detection
    context.rs          # Accuracy for context detection
    report.rs           # Output formatting (human-readable and JSON)
```

**Dependency on mnemosyne:** The `Cargo.toml` references the main crate via path:

```toml
[dependencies]
mnemosyne = { path = ".." }
```

### 3.2 Retrieval Metrics

The harness loads all corpus entries into a `FileIndex`, executes each query from
`queries.yaml`, and compares the ranked results against the relevance judgements.

**Metrics computed:**

| Metric | Definition | Formula |
|--------|-----------|---------|
| MRR | Mean Reciprocal Rank — average of 1/rank for the first relevant result | MRR = (1/N) * sum(1/rank_i) |
| Precision@k | Fraction of top-k results that are relevant | P@k = \|relevant in top-k\| / k |
| Recall@k | Fraction of all relevant entries that appear in top-k | R@k = \|relevant in top-k\| / \|all relevant\| |
| nDCG@k | Normalised Discounted Cumulative Gain — rewards relevant results ranked higher, accounts for graded relevance | Standard nDCG formula using relevance grades 0/1/2 |

**Default k = 5.** Configurable via `--k` flag.

**Per-query breakdown:** With `--verbose`, report metrics per query to identify which
queries perform poorly. This guides corpus refinement and scoring function improvements.

### 3.3 Contradiction Detection Metrics

For each contradiction pair from `contradictions.yaml`, the harness loads `entry_a`
into a `FileIndex` and calls `find_contradictions` with `entry_b` as the candidate.
If `entry_a` appears in the results, the pair is flagged. The flag is compared against
the label.

**Metrics computed:**

| Metric | Definition |
|--------|-----------|
| Precision | Of pairs flagged as contradictions, what fraction truly are? |
| Recall | Of true contradictions, what fraction were flagged? |
| F1 | Harmonic mean of precision and recall |

**Threshold sweep:** With `--sweep`, run contradiction detection across thresholds
from 0.3 to 0.8 in steps of 0.05. Report F1 at each threshold. Output as a table:

```
Contradiction Threshold Sweep:
  0.30  P=0.45  R=0.95  F1=0.61
  0.35  P=0.52  R=0.90  F1=0.66
  ...
  0.50  P=0.80  R=0.75  F1=0.77  <-- current default
  ...
  0.80  P=0.95  R=0.40  F1=0.57
```

This enables empirical calibration of the threshold.

### 3.4 Context Detection Accuracy

The harness runs `detect` against each mock project directory and compares the detected
languages and dependencies against the `expected.yaml` manifest.

**Metrics computed:**

| Metric | Definition |
|--------|-----------|
| Language accuracy | Fraction of mock projects where all expected languages were correctly detected |
| Dependency accuracy | Fraction of expected dependencies correctly detected across all projects |
| Tag mapping accuracy | Fraction of expected tags produced by the context mapping |

### 3.5 CLI Interface

```
mnemosyne-eval [OPTIONS]

Options:
  --corpus <PATH>     Path to corpus directory (default: eval/corpus)
  --k <N>             Top-k for retrieval metrics (default: 5)
  --sweep             Run contradiction threshold sweep
  --verbose           Per-query and per-pair breakdown
  --json              Output in JSON format
```

**Exit code:** 0 always (metrics are informational, not pass/fail). The harness reports
numbers; humans decide whether they are good enough.

### 3.6 JSON Output Schema

```json
{
  "retrieval": {
    "mrr": 0.82,
    "precision_at_k": 0.74,
    "recall_at_k": 0.68,
    "ndcg_at_k": 0.79,
    "k": 5,
    "query_count": 15,
    "per_query": [...]
  },
  "contradiction": {
    "threshold": 0.5,
    "precision": 0.80,
    "recall": 0.75,
    "f1": 0.77,
    "pair_count": 8,
    "sweep": [...]
  },
  "context_detection": {
    "language_accuracy": 0.93,
    "dependency_accuracy": 0.87,
    "tag_mapping_accuracy": 0.90,
    "project_count": 4
  }
}
```

---

## 4. Python Quality Harness (`eval/quality/`)

A standalone Python package that evaluates knowledge entry quality using LLM-as-judge.

### 4.1 Package Structure

```
eval/quality/
  pyproject.toml          # package metadata, dependencies
  rubrics/
    entry_quality.yaml    # specificity, actionability, provenance, confidence fit
  src/
    __init__.py
    __main__.py           # CLI entry point
    judge.py              # Provider-agnostic Judge protocol
    providers/
      __init__.py
      claude.py           # Anthropic SDK implementation
    rubric.py             # Load and format rubric YAML into prompts
    structural.py         # Automated structural completeness checks
    report.py             # Aggregate scores, format output
    config.py             # Provider selection, API key handling
```

### 4.2 Judge Protocol

```python
from typing import Protocol

class JudgeScore:
    dimension: str       # e.g. "specificity"
    score: int           # 1-5
    justification: str   # one-sentence explanation

class Judge(Protocol):
    def evaluate(self, entry_content: str, rubric: dict) -> list[JudgeScore]:
        """Score a knowledge entry against a rubric."""
        ...
```

The `providers/claude.py` module implements this protocol using the Anthropic SDK.
New providers (OpenAI, local models) implement the same protocol.

**Provider selection:** Via `--provider` flag or `MNEMOSYNE_EVAL_PROVIDER` environment
variable. Default: `claude`.

**Model selection:** Via `--model` flag or `MNEMOSYNE_EVAL_MODEL` environment variable.
Default: `claude-haiku-4-5-20251001` (fast and cheap for rubric scoring).

### 4.3 Rubric Format

Rubrics are stored as YAML in `eval/quality/rubrics/`. Each rubric defines dimensions
with anchor descriptions for each score level.

```yaml
name: entry_quality
description: Evaluates the quality of a Mnemosyne knowledge entry
dimensions:
  specificity:
    description: "How specific and targeted is this entry?"
    anchors:
      5: "Identifies a specific technology, version, or scenario. A practitioner
          could apply this without further research."
      4: "Identifies a specific area but requires some contextual knowledge."
      3: "Reasonably specific but could apply to multiple situations."
      2: "Vague. Could apply to many technologies or scenarios."
      1: "Completely generic. No actionable specificity."

  actionability:
    description: "Could someone act on this knowledge?"
    anchors:
      5: "Contains a concrete recommendation, pattern, or constraint that
          directly guides implementation."
      4: "Contains useful guidance but requires interpretation."
      3: "Informative but not directly actionable."
      2: "Mostly context or background."
      1: "No actionable content."

  provenance:
    description: "Is the origin and evidence base clear?"
    anchors:
      5: "Multiple dated origins with specific project context. Clear evidence trail."
      4: "At least one origin with context. Provenance is traceable."
      3: "Origin present but context is thin."
      2: "Origin field exists but is minimal or generic."
      1: "No provenance information."

  confidence_fit:
    description: "Does the confidence level match the evidence?"
    anchors:
      5: "Confidence perfectly matches evidence depth (e.g. high confidence
          with multi-project validation)."
      4: "Confidence is reasonable given the evidence."
      3: "Confidence is slightly optimistic or conservative."
      2: "Confidence is mismatched — e.g. high confidence from a single observation."
      1: "Confidence is clearly wrong given the evidence."
```

### 4.4 Structural Completeness

Automated checks that require no LLM. Run as a fast pre-filter.

**Checks:**
- All required frontmatter fields present (title, tags, created, confidence)
- Confidence is a valid value (high/medium/low/prospective)
- Tags is a non-empty list
- Body is non-empty
- Dates are valid ISO 8601 format
- If confidence is high or medium, origins field should be present

**Output:** Count of valid/invalid entries, with per-entry error details in verbose mode.

### 4.5 Variance Reduction

Each entry is evaluated twice per run. Between evaluations, the order of rubric dimensions
is shuffled in the prompt to mitigate position bias. Scores are averaged across the two
passes. The justification from the first pass is retained.

This doubles API cost but significantly improves score reliability. A `--single-pass`
flag skips the second evaluation for quick-and-dirty assessments.

### 4.6 Entry Sources

The harness can evaluate entries from two sources:

- **Corpus entries** (`--corpus eval/corpus/entries/`): the benchmark — useful for
  validating that the corpus itself is high quality
- **Live entries** (`--store ~/.mnemosyne/knowledge/`): the real knowledge base —
  useful for auditing the actual knowledge you've accumulated

Default: corpus entries. The `--store` flag switches to live evaluation.

### 4.7 CLI Interface

```
python -m eval.quality [OPTIONS]

Options:
  --corpus <PATH>       Path to entries directory (default: eval/corpus/entries)
  --store <PATH>        Evaluate live knowledge store instead of corpus
  --rubric <PATH>       Path to rubric YAML (default: eval/quality/rubrics/entry_quality.yaml)
  --provider <NAME>     LLM provider (default: claude)
  --model <NAME>        Model ID (default: claude-haiku-4-5-20251001)
  --single-pass         Skip variance reduction (one eval per entry)
  --json                Output in JSON format
  --verbose             Per-entry score breakdown
```

### 4.8 Output Format

**Human-readable (default):**

```
Entry Quality (N=42 entries):
  Specificity:     mean=3.8  median=4  std=0.9
  Actionability:   mean=3.5  median=4  std=1.1
  Provenance:      mean=4.1  median=4  std=0.7
  Confidence fit:  mean=4.0  median=4  std=0.8

  Structural completeness: 40/42 valid (2 issues)
    - techniques-async-patterns.md: missing origins (confidence=medium)
    - tools-docker-layers.md: invalid date format in created field

  Lowest scoring entries:
    - domains-web-caching.md: specificity=1, actionability=2
    - projects-legacy-migration.md: confidence_fit=1
```

**JSON:** Per-entry scores plus aggregates, same structure as human-readable but machine-parseable.

---

## 5. Phases 3-4: Design Intent

These phases build on the infrastructure from Phases 1-2. They are described at intent
level and will receive their own detailed specs.

### 5.1 Phase 3: Multi-Session Simulation

**Goal:** Validate that knowledge accumulates correctly and transfers across projects
over a simulated multi-session workflow.

**Shape:** A Python-based simulation (extending the Phase 2 package) that:

1. Initialises a fresh Mnemosyne store in a temporary directory
2. Simulates 3-5 sessions across 2-3 mock projects
3. Each session: queries for context, injects synthetic observations via `promote`,
   triggers `curate` where appropriate
4. Measures knowledge base state at each session boundary using the Phase 2 quality
   metrics and Phase 1 retrieval metrics

**Open design questions:**
- How to simulate the "developer discovers something" step — scripted fixtures or
  LLM-driven observation generation?
- Whether to test via CLI invocations (black-box) or library calls (white-box)
- How to define expected state at each boundary without making the test tautological

**Success criteria:**
- Cross-project knowledge is retrievable from a project that didn't originate it
- Contradiction detection fires when a later session contradicts an earlier one
- Quality metrics do not degrade across sessions (knowledge base health is maintained)

### 5.2 Phase 4: Controlled Impact Experiments

**Goal:** Demonstrate measurably that Mnemosyne improves AI assistant outcomes on
coding tasks.

**Shape:** An A/B experimental harness that:

1. Defines 5-10 coding tasks, each containing a known pitfall that cross-project
   knowledge could prevent
2. Seeds relevant knowledge (treatment) or provides no prior knowledge (control)
3. Runs each task N times per condition via the Claude API
4. Evaluates outcomes: task completion, pitfall avoidance, code quality (LLM-as-judge)
5. Reports effect sizes with confidence intervals

**Open design questions:**
- Task design — what pitfalls are concrete enough to test but general enough to matter?
- Sample size — how many runs per condition for statistical power given LLM variance?
- Cost management — multiple API calls per run; how to keep experiments affordable?
- Whether to use Claude Code as the agent or a simpler agent loop

**Success criteria:**
- Statistically meaningful difference in pitfall avoidance rate between conditions
- Effect demonstrated across at least 3 distinct tasks
- Results reproducible across runs

---

## 6. Directory Layout Summary

```
eval/
  Cargo.toml                          # Rust harness crate
  src/
    main.rs
    corpus.rs
    retrieval.rs
    contradiction.rs
    context.rs
    report.rs
  quality/                            # Python harness package
    pyproject.toml
    rubrics/
      entry_quality.yaml
    src/
      __init__.py
      __main__.py
      judge.py
      providers/
        __init__.py
        claude.py
      rubric.py
      structural.py
      report.py
      config.py
  corpus/                             # Shared benchmark data
    entries/
      languages-rust-lifetimes.md
      techniques-connection-pooling.md
      ...
    queries.yaml
    contradictions.yaml
    projects/
      rust-web-api/
        Cargo.toml
        src/main.rs
        expected.yaml
      python-ml-pipeline/
        requirements.txt
        setup.py
        src/__init__.py
        expected.yaml
      typescript-react/
        package.json
        tsconfig.json
        src/index.tsx
        expected.yaml
```

---

## 7. Dependencies and Requirements

### Rust Harness
- Rust 1.75+ (same as the main crate)
- `mnemosyne` library crate (path dependency)
- `clap` for CLI parsing
- `serde` + `serde_yaml` + `serde_json` for corpus loading and JSON output

### Python Harness
- Python 3.10+
- `anthropic` SDK (for Claude provider)
- `pyyaml` for rubric and corpus loading
- No other required dependencies — keep it minimal

### Environment
- `ANTHROPIC_API_KEY` environment variable for the Python harness
- No API key required for the Rust harness (purely local computation)

---

## 8. What This Spec Does Not Cover

- Changes to the existing `tests/` integration test suite
- Changes to the `mnemosyne` CLI or library code (the harness evaluates the current
  code; improvements to scoring or thresholds are separate work informed by evaluation
  results)
- Implementation details of Phases 3-4 (to be specified separately)
- Automated CI integration for the Python harness (the Rust harness can run in CI;
  the Python harness requires API keys and is run on-demand)
