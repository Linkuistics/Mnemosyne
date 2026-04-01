---
name: create-plan
description: Create a multi-session plan with built-in observational memory. Usage: /create-plan <name> [context] [sub-context]
---

You are creating a multi-session implementation plan with observational memory built in.

## 1. Parse arguments

Extract:
- `name` — the plan name (used for the filename)
- `context` — optional primary context (if provided, load knowledge via /begin-work)
- `sub-context` — optional secondary context

## 2. Load context (if provided)

If context is provided, run the begin-work sequence to load relevant knowledge. This informs the plan content.

## 3. Gather plan details

Ask the user for:
- **Task summary**: 2-3 sentences describing what this plan accomplishes
- **Steps**: Either provide them, or ask the user to describe the work and help break it down

## 4. Load configuration

Read `.observational-memory.yml` to get `plans_dir` and `review_cadence` (default: 5).

## 5. Generate the plan

Write the plan to `{plans_dir}/{context}/{name}.md` (or `{plans_dir}/{name}.md` if no context).

Use this structure:

```markdown
# {name}

## Task Summary
{user-provided summary}

## Session Continuation Prompt

\```
Run /begin-work {context} {sub-context} to load knowledge and find your place,
then continue from the next incomplete step. Complete each step's
Do → Verify → Observe cycle.
\```

## Progress

- [ ] Step 1: {description}
  - **Do**: {what to implement}
  - **Verify**: {how to confirm it works}
  - **Observe**: _(pending)_
- [ ] Step 2: {description}
  - **Do**: {what to implement}
  - **Verify**: {how to confirm it works}
  - **Observe**: _(pending)_
{... more steps ...}
- [ ] Code Review Session 1
  - Review steps 1-{review_cadence}, run /reflect, assess progress
{... more steps and review sessions interleaved every review_cadence steps ...}

## Observations

_(Accumulated from step-level observations during work)_

## Promoted

_(Tracks observations promoted to the knowledge base)_

## Learnings

_(Significant in-flight discoveries not yet ready for promotion)_
```

## 6. Confirm

Show the user the generated plan and ask for any adjustments before finalising.
