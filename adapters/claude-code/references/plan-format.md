# Integrated Plan Format

Plans track progress across multiple work sessions with built-in observational memory.

## File Structure

### 1. Task Summary
2-3 sentences describing the overall goal.

### 2. Session Continuation Prompt
A copyable prompt that invokes /begin-work with the right context:

```
Run /begin-work {context} {sub-context} to load knowledge and find your place,
then continue from the next incomplete step. Complete each step's
Do → Verify → Observe cycle.
```

### 3. Progress Checklist

Steps with the three-phase structure:

```markdown
- [ ] Step 1: {description}
  - **Do**: {what to implement}
  - **Verify**: {how to confirm it works}
  - **Observe**: _(filled after completion)_
- [ ] Step 2: ...
- [ ] Code Review Session 1
  - Review steps 1-N, run /reflect, assess progress
```

Step granularity: each step completable in 1-2 hours, clear completion criteria.

Code review sessions interleaved every 3-5 implementation steps.

### 4. Observations
Accumulated from step-level observations, grouped by date:

```markdown
**YYYY-MM-DD:**
- 🔴 {critical discovery} (Step N)
- 🟡 {useful discovery} (Step N)
```

### 5. Promoted
Tracks observations promoted to the knowledge base:

```markdown
- 🔴 {observation} → knowledge/{axis}/{file}.md (YYYY-MM-DD)
```

### 6. Learnings
Significant in-flight discoveries not yet ready for promotion. Reviewed during
code review sessions.

## Lifecycle

1. Create the plan at the start of a multi-session task
2. After each step: fill in Observe, copy to Observations section
3. Every 3-5 steps: code review session with /reflect
4. Start each session with /begin-work
5. When complete: final /reflect to promote remaining observations
