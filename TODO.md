# Mnemosyne -- Next Steps

## 1. Implement the horizon-scanning mode

The spec and implementation plan are written and committed:

- **Spec:** `docs/superpowers/specs/2026-04-01-horizon-scanning-design.md`
- **Plan:** `docs/superpowers/plans/2026-04-01-horizon-scanning.md`

The plan has 5 tasks (skill rewrite, eval corpus entries, rubric dimension, harness
code changes, coverage matrix update). No Rust core changes -- all work is in the
skill prompt and the Python eval harness.

### Prompt

```
Read the implementation plan at docs/superpowers/plans/2026-04-01-horizon-scanning.md
and execute it task by task. The plan has 5 tasks with detailed steps, code blocks,
and commit messages. Use subagent-driven development (the plan header specifies this).

The spec is at docs/superpowers/specs/2026-04-01-horizon-scanning-design.md if you
need to check design intent.
```

## 2. Manual smoke test of the horizon-scanning skill

After implementation, verify the skill works end-to-end by running an explore session.

### Prompt

```
I just implemented the horizon-scanning mode in the explore-knowledge skill. Run
/explore-knowledge and select horizon scanning mode to smoke test the pipeline.
Pick 1-2 scanning areas, verify that web search, novelty filtering, batch
presentation, and entry generation all work. Report any issues with the skill
prompt that need iteration.
```

## 3. Evaluate generated entries with the quality harness

After smoke testing, run the eval harness against any entries created during
the smoke test to check quality scores.

### Prompt

```
Run the eval quality harness against my live knowledge store to score the
prospective entries created by horizon scanning:

  python -m eval.quality --store ~/.mnemosyne/knowledge/ --verbose

Check that the new horizon_relevance dimension appears for prospective entries
and that scores are reasonable (3+ on all dimensions). If any entries score
below 3, suggest specific improvements to the skill prompt in
adapters/claude-code/skills/explore-knowledge.md.
```

## 4. Evaluation Phase 3: Multi-session simulation

Validate that knowledge accumulates correctly and transfers across projects over a
simulated multi-session workflow. Requires designing a simulation that initialises a
fresh store, runs 3-5 sessions across mock projects, and measures knowledge base state
at each boundary.

### Prompt

```
Read the Phase 3 intent section in docs/superpowers/specs/2026-04-01-evaluation-framework-design.md.
Design and implement a multi-session simulation harness that:
1. Initialises a fresh ~/.mnemosyne/ store
2. Simulates 3-5 development sessions across 2-3 mock projects
3. Uses the CLI to promote knowledge entries at session boundaries
4. Measures: entry count growth, tag coverage expansion, contradiction detection
   triggers, cross-project knowledge reuse
5. Reports pass/fail criteria for each metric

Place the harness in eval/simulation/ as a Rust binary that orchestrates the CLI.
Write a spec first, get approval, then implement.
```

## 5. Evaluation Phase 4: Controlled impact experiments

A/B experimental harness demonstrating that Mnemosyne measurably improves AI assistant
outcomes on coding tasks. Requires task design, sample size planning, and cost management
for multi-run API experiments.

### Prompt

```
Read the Phase 4 intent section in docs/superpowers/specs/2026-04-01-evaluation-framework-design.md.
Design an A/B experimental framework that:
1. Defines 5-10 coding tasks spanning multiple languages and domains
2. Runs each task twice: once with Mnemosyne context, once without
3. Uses Claude API to execute tasks programmatically
4. Measures: task completion rate, code quality (via LLM judge), time-to-solution,
   error avoidance
5. Includes sample size calculations and statistical significance testing

Write the spec first. This will require API costs -- include a cost estimate.
Place in eval/impact/.
```

## 6. Publish to crates.io

Once the core is stable and horizon scanning is implemented:

### Prompt

```
Prepare Mnemosyne for publishing to crates.io:
1. Review Cargo.toml metadata (description, repository, homepage, keywords, categories)
2. Ensure all public API items have doc comments
3. Add a lib-level doc comment with examples
4. Run `cargo publish --dry-run` and fix any issues
5. Verify the README renders correctly on crates.io (check for broken links)
```

## 7. Additional language detection profiles

The config currently covers 12 languages. Expand coverage for commonly used ecosystems.

### Prompt

```
Add language detection profiles to src/config.rs for:
- Go (go.mod, .go files)
- Java (pom.xml, build.gradle, .java files)
- TypeScript/JavaScript (package.json, tsconfig.json, .ts/.js files)
- C/C++ (CMakeLists.txt, Makefile, .c/.cpp/.h files)
- Ruby (Gemfile, .rb files)
- Elixir (mix.exs, .ex/.exs files)

Also add context_mappings for npm/yarn dependencies (react, express, etc.)
and Go modules. Update the defaults/config.yml to match. Add tests for each
new profile in tests/.
```
