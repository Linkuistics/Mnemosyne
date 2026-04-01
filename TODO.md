# Mnemosyne — Next Steps

## 1. Implement the horizon-scanning mode

The spec and implementation plan are written and committed:

- **Spec:** `docs/superpowers/specs/2026-04-01-horizon-scanning-design.md`
- **Plan:** `docs/superpowers/plans/2026-04-01-horizon-scanning.md`

The plan has 5 tasks (skill rewrite, eval corpus entries, rubric dimension, harness
code changes, coverage matrix update). No Rust core changes — all work is in the
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
