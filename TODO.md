# Mnemosyne -- Next Steps

Actionable tasks with Claude Code prompts. Each task is self-contained.

---

## 1. Implement the horizon-scanning mode in the explore-knowledge skill

The spec and implementation plan are written and committed. This adds web-search-driven discovery of new developments to the `/explore-knowledge` Claude Code skill. No Rust core changes -- all work is in the skill prompt and the Python eval harness.

- **Spec:** `docs/superpowers/specs/2026-04-01-horizon-scanning-design.md`
- **Plan:** `docs/superpowers/plans/2026-04-01-horizon-scanning.md`
- 5 tasks: skill rewrite, eval corpus entries, rubric dimension, harness code changes, coverage matrix update

### Prompt

```
Read the implementation plan at docs/superpowers/plans/2026-04-01-horizon-scanning.md
and execute it task by task. The plan has 5 tasks with detailed steps, code blocks,
and commit messages. Use subagent-driven development (the plan header specifies this).

The spec is at docs/superpowers/specs/2026-04-01-horizon-scanning-design.md if you
need to check design intent.
```

---

## 2. Smoke-test the horizon-scanning skill

After implementing task 1, verify the skill works end-to-end.

### Prompt

```
I just implemented the horizon-scanning mode in the explore-knowledge skill. Run
/explore-knowledge and select horizon scanning mode to smoke test the pipeline.
Pick 1-2 scanning areas, verify that web search, novelty filtering, batch
presentation, and entry generation all work. Report any issues with the skill
prompt that need iteration.
```

---

## 3. Evaluate generated entries with the quality harness

After smoke testing, score entries created during the smoke test.

### Prompt

```
Run the eval quality harness against my live knowledge store to score the
prospective entries created by horizon scanning:

  cd eval/quality
  PYTHONPATH=../.. python3 -m eval.quality.src.__main__ --store ~/.mnemosyne/knowledge/ --single-pass --verbose

Check that the new horizon_relevance dimension appears for prospective entries
and that scores are reasonable (3+ on all dimensions). If any entries score
below 3, suggest specific improvements to the skill prompt in
adapters/claude-code/skills/explore-knowledge.md.
```

---

## 4. Add a `--tags` flag to `mnemosyne query`

The CLI currently supports term-based search and `--context` auto-detection, but has no way to query by explicit tags from the command line. The horizon-scanning skill spec references `mnemosyne query --tags <area_tags> --format json` which does not exist yet. Adding this would also be useful for scripting and other adapters.

### Prompt

```
Add a --tags flag to the mnemosyne query command. It should accept a comma-separated
list of tags and use them for tag-based retrieval (same as the context path but with
explicitly provided tags instead of auto-detected ones). --tags and --context should
be mutually exclusive. Update the CLI Reference doc at docs/reference.md to document
the new flag.

The query infrastructure already supports tag-based search via QueryOptions.tags --
this is just wiring the CLI flag through to it.
```

---

## 5. Evaluation Phase 3: Multi-session simulation

Validate that knowledge accumulates correctly and transfers across projects over a simulated multi-session workflow. Requires designing a simulation that initialises a fresh store, runs 3-5 sessions across mock projects, and measures knowledge base state at each boundary.

See the Phase 3 intent section in the [evaluation framework spec](docs/superpowers/specs/2026-04-01-evaluation-framework-design.md).

### Prompt

```
Design and implement the Phase 3 multi-session simulation for the Mnemosyne
evaluation framework. The intent is described in Section 5.1 of
docs/superpowers/specs/2026-04-01-evaluation-framework-design.md.

Start by writing a spec (docs/superpowers/specs/) that resolves the open design
questions: scripted fixtures vs LLM-driven observation generation, black-box CLI
testing vs white-box library calls, and how to define expected state at each
boundary without making the test tautological.

Then write an implementation plan and execute it.
```

---

## 6. Evaluation Phase 4: Controlled impact experiments

A/B experimental harness demonstrating that Mnemosyne measurably improves AI assistant outcomes on coding tasks. Requires task design, sample size planning, and cost management for multi-run API experiments.

See the Phase 4 intent section in the same spec.

### Prompt

```
Design the Phase 4 controlled impact experiment for the Mnemosyne evaluation
framework. The intent is described in Section 5.2 of
docs/superpowers/specs/2026-04-01-evaluation-framework-design.md.

Start by writing a spec that defines 5-10 coding tasks with known pitfalls,
the A/B harness architecture, sample size requirements, and cost estimates.
This is a design task -- implementation comes later.
```

---

## 7. Expand default context mappings

The built-in `context_mappings` in `config.yml` only covers 4 Cargo dependencies (tokio, sqlx, axum, serde) and no pyproject dependencies. Expanding this would improve out-of-the-box context detection for common Rust and Python ecosystems.

### Prompt

```
Expand the default context_mappings in src/config.rs to cover the most common
Cargo and pyproject dependencies. Add mappings for at least:

Cargo: tonic, reqwest, tracing, clap, anyhow, thiserror, diesel, redis, rayon,
       actix-web, warp, hyper

Pyproject: fastapi, sqlalchemy, pydantic, pytest, httpx, django, flask, celery

Follow the existing pattern. Each dependency maps to the conceptual tags it
represents (e.g., tonic -> [grpc, rpc, protobuf, networking]). Update the
Configuration doc at docs/configuration.md to list the new defaults.

Also update the defaults/config.yml file to match, and add tests in
tests/config_test.rs for the new mappings.
```

---

## 8. Additional language detection profiles

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

---

## 9. Add `--version` flag to the CLI

The CLI defines `--version` implicitly via clap but does not include a version string in the `#[command]` attribute.

### Prompt

```
Add version output to the mnemosyne CLI. Update the #[command] attribute in
src/main.rs to include version = env!("CARGO_PKG_VERSION"). Verify that
`cargo run -- --version` prints "mnemosyne 0.1.0". Update the user guide
prerequisites section if needed.
```

---

## 10. Publish to crates.io

Once the core is stable and horizon scanning is implemented.

### Prompt

```
Prepare Mnemosyne for publishing to crates.io:
1. Review Cargo.toml metadata (description, repository, homepage, keywords, categories)
2. Ensure all public API items have doc comments
3. Add a lib-level doc comment with examples
4. Run `cargo publish --dry-run` and fix any issues
5. Verify the README renders correctly on crates.io (check for broken links)
```
