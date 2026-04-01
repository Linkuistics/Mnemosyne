---
name: begin-work
description: Start or continue implementation work with full knowledge context. Usage: /begin-work <context> [sub-context]
---

You are starting or continuing implementation work. Follow these steps exactly.

## 1. Parse arguments

The user invoked `/begin-work <context> [sub-context]`. Extract:
- `context` — the primary organisational unit (e.g., a target, service, or module)
- `sub-context` — optional secondary unit (e.g., an app, feature, or endpoint)

## 2. Load configuration

Look for `.observational-memory.yml` in the project root. If it exists, read it to get:
- `knowledge_dir` (default: `knowledge`)
- `plans_dir` (default: `LLM_STATE/plans`)
- `axes.primary`, `axes.secondary`, `axes.matrix`, `axes.extras`
- `review_cadence` (default: 5)

If no config file exists, use defaults and infer axes from the `knowledge/` directory structure.

## 3. Load knowledge

Read the following files (skip any that don't exist):

1. **Knowledge index**: `{knowledge_dir}/CLAUDE.md`
2. **Primary axis**: `{knowledge_dir}/{axes.primary}/{context}.md`
3. If sub-context provided:
   - **Secondary axis spec**: `{knowledge_dir}/{axes.secondary}/{sub-context}/spec.md`
   - **Secondary axis learnings**: `{knowledge_dir}/{axes.secondary}/{sub-context}/learnings.md`
   - **Secondary axis test strategy**: `{knowledge_dir}/{axes.secondary}/{sub-context}/test-strategy.md`
   - **Matrix**: `{knowledge_dir}/{axes.matrix}/{sub-context}/{context}.md`
4. **Extra axes**: For each axis in `axes.extras`, check if `{knowledge_dir}/{axis}/` has content relevant to the context

## 4. Check for active plan

Look for a plan file:
- With sub-context: `{plans_dir}/{context}/{sub-context}.md`
- Without sub-context: `{plans_dir}/{context}/plan.md`

If a plan exists:
1. Count completed vs total steps (lines matching `- [x]` vs `- [ ]`)
2. Find the next incomplete step (first `- [ ]` that is not a Code Review Session)
3. Count un-promoted observations (entries in `## Observations` not in `## Promoted`)
4. Check if code review is overdue: count implementation steps since last completed Code Review Session. If > review_cadence, flag it.
5. Show any 🔴 (critical) un-promoted observations prominently

If no plan exists, mention that and offer to create one via `/create-plan`.

## 5. Load global knowledge (if available)

Check whether `mnemosyne` is installed:

```bash
which mnemosyne
```

If found, run:

```bash
mnemosyne query --context --format markdown
```

This queries the global knowledge base for entries relevant to the current project context. If the command fails or produces no output, skip silently — do not report errors to the user.

Include the output in the summary under "### Global knowledge loaded". If not found, omit that section entirely.

## 6. Display summary

Present a clear summary:

```
## Working on: {context} {sub-context}

### Knowledge loaded
- [list each file that was successfully read]

### Global knowledge loaded
- [list knowledge entries from mnemosyne query, if available]

### Plan status
- Progress: N/M steps complete
- Next step: Step K — {description}
- Un-promoted observations: X (Y critical)
- Code review: {overdue / due in N steps / up to date}

### Critical observations (un-promoted 🔴)
- {list any}
```

Omit "Global knowledge loaded" if `mnemosyne` is not installed.

## 7. Proceed

After displaying the summary, proceed to work on the next incomplete step. Follow the Do → Verify → Observe cycle for each step.
