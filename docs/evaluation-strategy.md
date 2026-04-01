# Evaluation Strategy

This document describes how to evaluate whether Mnemosyne works — not just whether the code runs correctly, but whether the system achieves its purpose: enabling AI assistants to accumulate expertise that transfers across projects and improves over time.

## Contents

- [The Evaluation Problem](#the-evaluation-problem)
- [Intrinsic vs Extrinsic Evaluation](#intrinsic-vs-extrinsic-evaluation)
- [Three Tiers of Evaluation](#three-tiers-of-evaluation)
  - [Tier 1: Algorithmic Correctness](#tier-1-algorithmic-correctness)
  - [Tier 2: Knowledge Quality](#tier-2-knowledge-quality)
  - [Tier 3: Downstream Impact](#tier-3-downstream-impact)
- [Metrics Reference](#metrics-reference)
  - [Retrieval Metrics](#retrieval-metrics)
  - [Evolution Metrics](#evolution-metrics)
  - [Quality Metrics](#quality-metrics)
  - [Impact Metrics](#impact-metrics)
- [Evaluation Techniques](#evaluation-techniques)
  - [Benchmark Corpus Design](#benchmark-corpus-design)
  - [LLM-as-Judge](#llm-as-judge)
  - [Multi-Session Simulation](#multi-session-simulation)
  - [A/B Experimental Design](#ab-experimental-design)
- [Connecting Evaluation to Theory](#connecting-evaluation-to-theory)
- [Implementation Roadmap](#implementation-roadmap)
- [Research Context](#research-context)

---

## The Evaluation Problem

Mnemosyne occupies an unusual position for evaluation. It is not a compiler (where correctness is binary), not a web service (where latency and throughput provide clear signals), and not an ML model (where a loss function drives optimisation). It is a knowledge management system whose value is measured in how well an AI assistant performs *over time* and *across projects* — a deeply qualitative, longitudinal outcome.

This creates a fundamental tension. The system's purpose is to replicate how senior developers accumulate expertise: through contradiction resolution, belief revision, and reflective practice. But the outcome of expertise — better judgement, fewer mistakes, richer contextual awareness — resists simple quantification. A developer who avoids a concurrency pitfall because they recall a lesson from a previous project has benefited from their knowledge system, but that benefit is visible only as the absence of an error.

The evaluation strategy must therefore work at multiple levels of abstraction, from "does the search function return the right entries?" to "does the system make an AI assistant measurably more effective?" — with the understanding that the higher levels are harder to measure but more valuable.

---

## Intrinsic vs Extrinsic Evaluation

Information retrieval research draws a foundational distinction between two evaluation approaches:

**Intrinsic evaluation** measures the system's internal properties: does the search return relevant results? Does contradiction detection identify true contradictions? Are entries well-formed? These properties can be measured independently of any downstream task.

**Extrinsic evaluation** measures whether users (or AI agents) accomplish their goals more effectively with the system than without it. This is what ultimately matters, but it is expensive, noisy, and confounded by many variables.

Most retrieval systems settle for intrinsic evaluation because extrinsic evaluation is prohibitively expensive. Mnemosyne's entire value proposition, however, is extrinsic: the system succeeds only if the knowledge it manages actually helps. The evaluation strategy must therefore include both — intrinsic evaluation for rapid, rigorous feedback on components, and extrinsic evaluation for periodic validation that the system delivers on its promise.

---

## Three Tiers of Evaluation

### Tier 1: Algorithmic Correctness

**What it validates:** The mechanical properties of the system — parsing, storage, search, matching, and detection work as specified.

**Current state:** 49 integration tests across 14 test modules cover this tier. Tests use real file I/O with `tempfile` for isolation, fixture-based test data, and deterministic assertions.

**What to expand:**

The existing tests validate *behaviour* (does this function do what the docstring says?) but not *quality* (does this function do its job well enough?). The gap is most visible in the search and scoring functions, which produce correct output but whose output quality has not been benchmarked.

Specific areas for expansion:

- **Search ranking quality**: The `score_entry` function in `src/knowledge/index.rs` uses hardcoded weights — tag overlap multiplied by 10.0, title match adds 5.0, body match adds 2.0, confidence multipliers of 1.0/0.8/0.6/0.4. These weights are reasonable defaults but have never been validated against a benchmark.

- **Contradiction detection calibration**: The 0.5 Jaccard similarity threshold for flagging potential contradictions in `find_contradictions` is a design choice, not an empirically validated threshold. Too low produces noise (false positives that waste the user's time during promotion). Too high misses real contradictions.

- **Context detection coverage**: The language and dependency detection heuristics in `src/context/detect.rs` use file-marker-based rules. Coverage across real-world project layouts has not been systematically measured.

---

### Tier 2: Knowledge Quality

**What it validates:** The quality of the knowledge that flows through the system — whether entries are well-formed, whether promotion produces useful generalisations, whether curation improves the knowledge base over time.

**Why this matters:** A knowledge management system can be mechanically correct but still useless if the knowledge it manages is vague, redundant, or poorly organised. The distinction between "the search works" and "the search returns *useful* entries" depends entirely on the quality of the entries themselves.

**Key questions this tier answers:**

- Are promoted entries specific enough to be actionable?
- Do entries preserve enough provenance (origins, context) to be trustworthy?
- Does the supersession mechanism preserve historical context without cluttering active entries?
- Does the tag vocabulary evolve coherently, or does it fragment into synonyms and near-duplicates?
- Does curation actually improve the knowledge base, or does it merely confirm what already exists?

This tier requires human judgement or LLM-based proxy judgement (see [LLM-as-Judge](#llm-as-judge) below), because entry quality is not reducible to a formula.

---

### Tier 3: Downstream Impact

**What it validates:** The system's actual purpose — whether AI assistants that use Mnemosyne produce better outcomes than those that don't.

**Why this is hardest:** The benefit of accumulated knowledge is diffuse and longitudinal. It manifests as:

- Avoiding pitfalls encountered in previous projects (error prevention)
- Applying patterns discovered elsewhere (positive transfer)
- Asking better questions during curation (deeper reflection)
- Producing more idiomatic code in unfamiliar languages (cross-domain expertise)

Each of these is real but hard to isolate from confounding variables (the quality of the LLM, the difficulty of the task, the structure of the prompt).

**Key questions this tier answers:**

- Does a Claude Code session *with* Mnemosyne produce measurably better outcomes than *without*?
- Does knowledge accumulate correctly over multiple simulated sessions?
- Does cross-project knowledge transfer actually help on new tasks?
- Does the system's value compound over time (more knowledge → better outcomes → more valuable knowledge)?

---

## Metrics Reference

### Retrieval Metrics

These measure how well the search and query system surfaces relevant knowledge.

| Metric | Definition | Measurement |
|--------|-----------|-------------|
| **Precision@k** | Of the top-k results returned, what fraction are relevant? | Requires a benchmark corpus with labelled relevant entries per query |
| **Recall@k** | Of all relevant entries in the corpus, what fraction appear in the top-k results? | Same benchmark corpus |
| **Mean Reciprocal Rank (MRR)** | The average of 1/rank for the first relevant result across all queries. Higher is better. | MRR = 1 means the best result is always ranked first |
| **Normalised Discounted Cumulative Gain (nDCG)** | Measures ranking quality accounting for graded relevance (not just binary). Rewards relevant results appearing higher. | Useful when entries have varying degrees of relevance to a query |
| **Context precision** | When using `--context` auto-detection, what fraction of returned entries are relevant to the detected project? | Run from known project directories, compare to expert expectations |

**Which to use:** Start with **MRR** — it is simple, interpretable, and directly measures "does the best answer appear near the top?" Expand to nDCG later if you need to distinguish between "somewhat relevant" and "highly relevant" results.

### Evolution Metrics

These measure whether the knowledge evolution mechanisms (contradiction detection, supersession, divergence detection) function correctly and usefully.

| Metric | Definition | Measurement |
|--------|-----------|-------------|
| **Contradiction precision** | Of entries flagged as potential contradictions, what fraction are genuine contradictions? | Requires labelled pairs (contradicting / non-contradicting) |
| **Contradiction recall** | Of genuine contradictions in the corpus, what fraction are flagged? | Same labelled pairs |
| **Contradiction F1** | Harmonic mean of precision and recall. Balances false positives against missed contradictions. | F1 = 2 × (precision × recall) / (precision + recall) |
| **Supersession chain integrity** | After N supersessions, can the full history be reconstructed from the entry? | Parse supersession records programmatically and verify ordering and content preservation |
| **Tag vocabulary coherence** | Ratio of semantically distinct tags to total tag count. Lower fragmentation is better. | Cluster tags by semantic similarity; measure cluster count vs total tags |

**Which to use:** Start with **contradiction F1** — it directly measures the most critical evolution mechanism. The 0.5 Jaccard threshold is the primary tuning lever.

### Quality Metrics

These measure the quality of knowledge entries themselves, independent of how they are retrieved or evolved.

| Metric | Definition | Measurement |
|--------|-----------|-------------|
| **Structural completeness** | Does the entry have all required frontmatter fields, a non-empty body, and valid values? | Automated validation against the knowledge format spec |
| **Specificity** | Is the entry specific enough to be actionable, or is it a vague generalisation? | LLM-as-judge on a 1-5 rubric |
| **Actionability** | Could someone apply this knowledge to a concrete task? | LLM-as-judge on a 1-5 rubric |
| **Provenance quality** | Are origins present, dated, and contextualised? | Automated check for origin fields + LLM assessment of context richness |
| **Appropriate confidence** | Does the confidence level match the evidence? (e.g., a single-project observation should not be "high") | Cross-reference confidence with origin count and validation dates |

**Which to use:** Start with **structural completeness** (fully automatable) and **specificity** (the most common failure mode in knowledge entries).

### Impact Metrics

These measure whether the system delivers on its promise of accumulated expertise.

| Metric | Definition | Measurement |
|--------|-----------|-------------|
| **Pitfall avoidance rate** | Given seeded knowledge about a known pitfall, does the agent avoid it in a new project? | Controlled experiment: seed pitfall knowledge, present task containing the pitfall, measure whether the agent avoids it |
| **Positive transfer rate** | Given knowledge from Project A, does the agent produce better results on a related task in Project B? | A/B experiment: same task with/without relevant prior knowledge |
| **Knowledge accumulation curve** | Does the knowledge base grow in quality and coverage over simulated sessions? | Measure quality metrics (above) at intervals across a multi-session simulation |
| **Session preparation quality** | Does the `begin-work` skill surface genuinely useful context? | LLM-as-judge or human review of the context injected at session start |

**Which to use:** Start with **pitfall avoidance rate** — it is the most concrete and controllable of the impact metrics. If you can demonstrate that seeded knowledge prevents a known error, the system's value becomes tangible.

---

## Evaluation Techniques

### Benchmark Corpus Design

A benchmark corpus is the foundation for all quantitative evaluation. Without ground truth, metrics are unmeasurable.

**Structure:**

```
eval/
  corpus/
    entries/           # 50-100 synthetic knowledge entries
      rust-lifetimes.md
      python-async-pitfalls.md
      docker-layer-caching.md
      ...
    queries/           # 20-30 queries with labelled relevance
      queries.yaml     # query text, tags, expected relevant entries
    contradictions/    # 10-15 entry pairs with labels
      pairs.yaml       # entry_a, entry_b, is_contradiction: true/false
    projects/          # Mock project directories for context detection
      rust-web-api/
      python-ml-pipeline/
      ...
  harness/
    run_retrieval.rs   # Compute MRR, precision@k, recall@k
    run_contradiction.rs  # Compute F1 for contradiction detection
    run_context.rs     # Compute accuracy for context detection
```

**Design principles:**

- **Entries should span all five axes** (languages, domains, tools, techniques, projects) to exercise cross-axis retrieval.
- **Queries should include both tag-based and text-based searches**, and mixes of both.
- **Relevance labels should be graded** (highly relevant, somewhat relevant, not relevant) to enable nDCG computation later.
- **Contradiction pairs should include near-misses**: entries with high tag overlap but no semantic contradiction, to test whether the system over-flags.
- **Mock projects should cover the languages in the config** (Rust, Python, TypeScript, Haskell, etc.) with realistic file structures.

**Size guidance:** Start small (30 entries, 10 queries, 5 contradiction pairs) and expand as the harness proves useful. A too-large corpus creates maintenance burden without proportional benefit.

### LLM-as-Judge

For metrics that require judgement (specificity, actionability, quality), use an LLM to evaluate entries against a rubric. This is a well-established technique in LLM evaluation research (see [Research Context](#research-context)).

**Rubric design:**

```yaml
specificity:
  5: "Identifies a specific technology, version, or scenario. A practitioner
      could apply this without further research."
  4: "Identifies a specific area but requires some contextual knowledge
      to apply."
  3: "Reasonably specific but could apply to multiple situations."
  2: "Vague. Could apply to many technologies or scenarios."
  1: "Completely generic. No actionable specificity."

actionability:
  5: "Contains a concrete recommendation, pattern, or constraint that
      directly guides implementation."
  4: "Contains useful guidance but requires interpretation."
  3: "Informative but not directly actionable."
  2: "Mostly context or background. Limited practical application."
  1: "No actionable content."
```

**Implementation approach:**

1. Design the rubric with clear anchor descriptions for each score level.
2. Evaluate each entry by presenting the LLM with the entry content and the rubric.
3. Request both a score and a brief justification (the justification improves score reliability).
4. Use multiple evaluation passes and average scores to reduce variance.
5. Periodically calibrate against human judgement — score 10-20 entries yourself and compare.

**Pitfalls:**

- LLMs tend toward the middle of scales (3/5). Use clear anchor descriptions and examples to spread the distribution.
- Self-evaluation bias: do not use the same model that generated entries to judge them. Use a different model or family.
- Rubric drift: revisit the rubric when the knowledge domain evolves.

### Multi-Session Simulation

To evaluate whether knowledge accumulates correctly over time, simulate the multi-session workflow that Mnemosyne is designed to support.

**Simulation structure:**

```
Session 1: Work on Project A
  → Discover 3 observations
  → Promote 2 to per-project knowledge
  → Promote 1 to global knowledge

Session 2: Work on Project B (related domain)
  → begin-work retrieves global knowledge from Session 1
  → Measure: was the promoted entry retrieved? Was it relevant?
  → Discover 2 new observations, one contradicting Session 1 knowledge
  → Promote contradiction; verify contradiction detection fires

Session 3: Curation
  → Run curate command
  → Measure: are flagged entries the right ones?
  → Resolve contradiction; verify supersession chain
  → Measure: did quality metrics improve post-curation?
```

**What to measure at each step:**

- Knowledge base size and quality metrics before and after each session
- Retrieval relevance of context-injected knowledge at session start
- Whether contradiction detection fires when expected (and doesn't when not expected)
- Whether the supersession chain is structurally intact after resolution

**Automation level:** This can be partially automated using scripted CLI interactions, but the "discovery" and "promotion" steps require either human input or LLM-driven simulation of the developer workflow. A fully automated simulation requires an LLM to play the role of the developer making observations — which introduces its own evaluation challenges.

### A/B Experimental Design

The gold standard for Tier 3 evaluation: does Mnemosyne actually help?

**Design:**

1. Define a set of coding tasks that exercise cross-project knowledge (e.g., "set up a connection pool in a new Rust project" when the knowledge base contains entries about connection pool pitfalls from a Python project).
2. Run each task twice: once with Mnemosyne providing context (treatment), once without (control).
3. Evaluate outcomes on:
   - Task completion (did the agent finish?)
   - Error presence (did the agent make the known pitfall error?)
   - Code quality (LLM-as-judge on the output)
   - Time to completion (number of iterations/tool calls)

**Controls:**

- Same LLM, same temperature, same system prompt (except Mnemosyne context injection)
- Multiple runs per condition to account for LLM non-determinism
- Task difficulty should be calibrated so that the control condition sometimes fails — if both conditions always succeed, you cannot detect a difference

**Statistical considerations:**

- LLM output variance is high. Plan for 10+ runs per condition per task to achieve statistical power.
- Use paired comparisons (same task, with/without) rather than between-task comparisons.
- Report effect sizes, not just p-values — a statistically significant but tiny improvement is not practically meaningful.

---

## Connecting Evaluation to Theory

Mnemosyne's design is grounded in cognitive science (see `research-sources.md`). Each theoretical foundation implies specific testable properties:

### Tulving's Episodic-Semantic Distinction

**Theory:** Expert knowledge transitions from episodic (specific incidents) to semantic (general principles) through consolidation.

**Testable property:** After promotion from per-project to global knowledge, entries should be more general (semantic) while preserving enough provenance (episodic) to remain trustworthy. Measure by comparing specificity scores of per-project observations vs their promoted global counterparts — global entries should score high on both generality *and* provenance quality.

### AGM Belief Revision

**Theory:** Rational belief revision minimises change: when incorporating contradictory information, retain as much existing knowledge as possible.

**Testable property:** Supersession should preserve the old content inline, not delete it. After contradiction resolution, the total knowledge (entry + supersession record) should contain strictly more information than the original entry alone. Measure by comparing information content (word count is a rough proxy; LLM-as-judge is better) before and after supersession.

### Ericsson's Deliberate Practice

**Theory:** Expertise develops through focused effort on challenging material with structured reflection and feedback.

**Testable property:** Curation should produce observable improvements in the knowledge base — not just confirm what exists. Measure the delta in quality metrics before and after a curation session. If curation consistently produces zero changes, either the knowledge base is perfect (unlikely) or the curation prompts are not challenging enough.

### Chase & Simon's Expert Chunking

**Theory:** Experts organise knowledge into meaningful chunks indexed by recognition patterns.

**Testable property:** The tag system should enable efficient retrieval by pattern (the "recognition" mechanism). Measure retrieval performance when querying by tags alone vs full-text search — tag-based retrieval should achieve higher precision because tags represent deliberate chunking.

### Cognitive Load Theory

**Theory:** Effective learning minimises extraneous load while maximising germane load.

**Testable property:** The `--max-tokens` constraint on query output should improve task performance by preventing context overload. Measure task outcomes at varying token limits — there should be a sweet spot where enough context helps but too much hurts.

---

## Implementation Roadmap

### Phase 1: Benchmark Corpus — COMPLETE

**Goal:** Create the ground truth that enables quantitative evaluation.

**Deliverables:**
- 39 synthetic knowledge entries spanning all five axes (languages, domains, tools, techniques, projects), four confidence levels, and tag densities from 1 to 6
- 20 queries with graded relevance labels (relevance 1 or 2), including 3 context-based queries
- 8 contradiction pairs with labels (3 true contradictions, 5 non-contradictions including near-misses)
- 4 mock project directories for context detection (Rust, Python, Haskell, mixed Rust+Python)
- A Rust evaluation harness (`eval/`) computing MRR, Precision@k, Recall@k, nDCG@k, contradiction F1 with threshold sweep, and context detection accuracy

**Baseline results:**

| Metric | Value |
|--------|-------|
| MRR | 0.975 |
| Precision@5 | 0.400 |
| Recall@5 | 0.942 |
| nDCG@5 | 0.945 |
| Contradiction F1 (threshold 0.50) | 0.667 (P=0.500, R=1.000) |
| Language detection accuracy | 1.000 |
| Dependency detection accuracy | 1.000 |
| Tag mapping accuracy | 1.000 |

The contradiction F1 of 0.667 at the default threshold of 0.50 matches the designed expectation (3 true positives, 3 false positives from tag-overlapping non-contradictions, 0 false negatives). The threshold sweep shows F1 peaks at 0.30–0.50 and precision reaches 1.0 at threshold 0.65 at the cost of recall (0.33).

**Known limitation:** The `context` field on queries (used by q06, q10, q18) is parsed but not passed to the search function, because the `Query` struct in the mnemosyne library does not yet support context-filtered search. This means the benchmark does not currently test context-aware retrieval. When the library adds a context field to `Query`, the harness will need to be updated.

**Usage:**

```bash
cd eval && cargo run -- --verbose --sweep    # human-readable with threshold sweep
cd eval && cargo run -- --json               # machine-readable JSON
cd eval && cargo test                        # 14 unit tests
```

### Phase 2: Quality Rubric and Automated Audit — COMPLETE

**Goal:** Establish a repeatable process for measuring entry quality.

**Deliverables:**
- A four-dimension quality rubric (`eval/quality/rubrics/entry_quality.yaml`) with behaviourally anchored 1–5 scales for specificity, actionability, provenance, and confidence fit
- An LLM-as-judge harness (`eval/quality/`) using the Anthropic SDK (Claude) with two-pass variance reduction via dimension order shuffling
- Automated structural completeness checks that validate all 39 corpus entries without requiring API calls
- A provider-agnostic Judge protocol enabling future providers beyond Claude

**Structural completeness baseline:** 39/39 corpus entries pass all structural checks (required fields, valid confidence values, origins present for high/medium confidence entries).

**Usage:**

```bash
cd eval/quality

# Structural checks only (no API key required)
PYTHONPATH=../.. python3 -c "
from eval.quality.src.structural import check_directory
results = check_directory('../corpus/entries')
print(f'{sum(r.valid for r in results)}/{len(results)} valid')
"

# Full LLM-as-judge evaluation (requires ANTHROPIC_API_KEY)
PYTHONPATH=../.. python3 -m eval.quality.src.__main__ --single-pass --verbose

# Run tests
PYTHONPATH=../.. python3 -m pytest tests/ -v    # 5 tests
```

### Phase 3: Multi-Session Simulation — TODO

**Goal:** Validate the knowledge accumulation and transfer pathway end to end.

**Deliverables:**
- A scripted multi-session simulation (3-5 sessions across 2-3 mock projects)
- Instrumentation to measure knowledge base state at each session boundary
- A report format that shows accumulation curve, retrieval relevance, and evolution events

**Open design questions:**
- How to simulate the "developer discovers something" step — scripted fixtures or LLM-driven observation generation?
- Whether to test via CLI invocations (black-box) or library calls (white-box)
- How to define expected state at each boundary without making the test tautological

**Success criteria:**
- Cross-project knowledge is retrievable from a project that didn't originate it
- Contradiction detection fires when a later session contradicts an earlier one
- Quality metrics do not degrade across sessions

**Why third:** This requires both the benchmark corpus (to validate retrieval within sessions) and the quality rubric (to assess knowledge growth), so it builds on Phases 1 and 2.

### Phase 4: Controlled Impact Experiments — TODO

**Goal:** Demonstrate that Mnemosyne measurably improves AI assistant outcomes.

**Deliverables:**
- A set of 5-10 coding tasks with known pitfalls that cross-project knowledge could prevent
- An A/B harness that runs tasks with and without Mnemosyne context
- Statistical analysis of effect sizes across conditions
- A report documenting the conditions under which Mnemosyne helps (and doesn't)

**Open design questions:**
- Task design — what pitfalls are concrete enough to test but general enough to matter?
- Sample size — how many runs per condition for statistical power given LLM variance?
- Cost management — multiple API calls per run; how to keep experiments affordable?
- Whether to use Claude Code as the agent or a simpler agent loop

**Success criteria:**
- Statistically meaningful difference in pitfall avoidance rate between conditions
- Effect demonstrated across at least 3 distinct tasks
- Results reproducible across runs

**Why last:** This is the most expensive and most informative evaluation. It answers the question the system exists to answer, but requires all the preceding infrastructure.

---

## Research Context

The evaluation techniques described here draw on established practices in information retrieval evaluation and LLM-based assessment:

### Information Retrieval Evaluation

The precision, recall, MRR, and nDCG metrics used in the retrieval evaluation are standard in the information retrieval literature. Voorhees (2002) provides a comprehensive overview of evaluation methodology in the TREC tradition, including the importance of relevance judgement quality and the statistical properties of different metrics.

**Voorhees, E.M. (2002).** The philosophy of information retrieval evaluation. In *Evaluation of Cross-Language Information Retrieval Systems* (CLEF 2001). Springer.

### LLM-as-Judge

Using LLMs to evaluate text quality is a technique validated by Zheng et al. (2023) in the MT-Bench and Chatbot Arena work. Key findings: LLM judges achieve high agreement with human judges when given clear rubrics, position bias can be mitigated by swapping answer order, and chain-of-thought justifications improve rating quality.

**Zheng, L., et al. (2023).** Judging LLM-as-a-Judge with MT-Bench and Chatbot Arena. *NeurIPS 2023*.

### Evals as a Discipline

OpenAI's work on evals (2023-2024) and Anthropic's approach to model evaluation both emphasise that evaluation design is at least as important as model development — poorly designed evals give false confidence, while well-designed evals reveal genuine capabilities and limitations. The principle applies equally to evaluating a knowledge management system: the evaluation is only as good as the benchmark and the metrics.

---

*This document is a living artefact. As the evaluation framework is implemented and findings emerge, update this document with results, revised metrics, and lessons learned about what works and what doesn't.*
