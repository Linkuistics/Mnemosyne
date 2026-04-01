---
name: reflect
description: Promote plan observations to the knowledge base during code review sessions. Usage: /reflect
---

You are conducting a code review reflection session. This promotes raw observations from the current plan into durable knowledge.

## 1. Identify the current plan

Look for the most recently referenced plan in this conversation. If unclear, ask the user which plan to reflect on.

Read the plan file.

## 2. Find un-promoted observations

Read the `## Observations` section. Cross-reference with the `## Promoted` section. An observation is un-promoted if it appears in Observations but not in Promoted.

Separate un-promoted observations by priority: 🔴 (critical), 🟡 (useful), 🟢 (informational).

## 3. Load project configuration

Read `.observational-memory.yml` (or use defaults) to understand the knowledge axes.

## 4. Process 🔴 and 🟡 observations

For each un-promoted 🔴 or 🟡 observation:

1. **Analyse scope**: Does this apply to...
   - Only this specific task? → matrix axis (`{knowledge_dir}/{matrix}/{sub-context}/{context}.md`)
   - All implementations of this sub-context? → secondary axis (`{knowledge_dir}/{secondary}/{sub-context}/learnings.md`)
   - All work in this context? → primary axis (`{knowledge_dir}/{primary}/{context}.md`)
   - The whole project? → an extras axis (e.g., pipeline, infrastructure)

2. **Present suggestion**: "This observation looks like a {axis} learning. Promote to `{target_file}`?"

3. **On confirmation**:
   - Read the target knowledge file
   - Append the observation as a dated entry: `**YYYY-MM-DD:** {priority_emoji} {observation text}`
   - Add an entry to the plan's `## Promoted` section: `- {priority_emoji} {summary} → {target_file} (YYYY-MM-DD)`

## 5. Process 🟢 observations

Summarise all un-promoted 🟢 observations. Ask: "These informational observations remain. Promote any? (default: skip)"

Only promote if the user specifically selects items.

## 6. Pattern detection

After processing individual observations, look for patterns:

- **Clustering**: If 2+ observations relate to the same topic (e.g., GC issues, delegate patterns), suggest consolidating into a single knowledge entry at an appropriate axis.
- **Overlap**: If a matrix-level observation duplicates something already in a broader axis, flag it and suggest removing the duplicate.
- **Promotion candidates**: If a narrow-axis learning seems broadly applicable (e.g., a matrix finding that would help all targets), suggest promoting to a wider axis.

## 7. Offer global promotion (if available)

Check whether `mnemosyne` is installed:

```bash
which mnemosyne
```

If found, for each observation promoted to per-project knowledge in step 4, ask:

"This learning may apply beyond this project. Promote to global? [y/N]"

If the user says yes, run:

```bash
mnemosyne promote --tags <inferred_tags> --origin <project_name>
```

The CLI will guide the interactive promotion session, including contradiction detection. The project name can be inferred from the git remote URL or the project directory name.

If `mnemosyne` is not installed, skip this step entirely — no warnings, no errors.

## 8. Summary

Report: "Promoted N observations. M remain un-promoted (🟢 informational). K patterns detected. G promoted to global."

Omit the global count if `mnemosyne` was not available.
