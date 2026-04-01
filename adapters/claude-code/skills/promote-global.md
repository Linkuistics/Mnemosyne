---
name: promote-global
description: Promote a learning to the global Mnemosyne knowledge base. Usage: /promote-global
---

You are promoting a learning from this project to the Mnemosyne global knowledge base. Global promotion is for learnings that apply across projects — not project-specific facts, but transferable understanding.

## 1. Check prerequisites

Verify `mnemosyne` is installed:

```bash
which mnemosyne
```

If not found, inform the user: "Mnemosyne is not installed. Install it with `cargo install mnemosyne` to use global knowledge features." Then stop.

## 2. Identify the learning

If the user has not already specified what to promote, ask:

"What learning would you like to promote to global knowledge? Describe it in a sentence or two."

Help the user articulate:
- The core insight (not what they did, but what they learned)
- Why it applies beyond this project
- A tentative axis: language, domain, tool, or technique

## 3. Suggest tags

Based on the learning, suggest appropriate tags. Tags enable cross-project retrieval and contradiction detection. Good tags are:
- Technology-specific: `rust`, `swift`, `async`, `tokio`
- Domain-specific: `macos`, `databases`, `networking`
- Technique-specific: `error-handling`, `concurrency`, `testing`

Ask the user to confirm or adjust.

## 4. Determine origin

Identify the current project name for the `origin` field. Infer from:

```bash
git remote get-url origin 2>/dev/null | sed 's/.*\///' | sed 's/\.git//'
```

Or use the current directory name as fallback. Confirm with the user.

## 5. Run the promotion

Launch the interactive CLI session:

```bash
mnemosyne promote --tags <tags> --origin <project>
```

The CLI will:
1. Prompt for the entry title and body
2. Check for contradictions with existing global entries (entries with overlapping tags)
3. If contradictions are found, present resolution options: supersede, coexist, discard, refine
4. Write the new entry to the appropriate axis in `~/.mnemosyne/knowledge/`

Stay present to help the user navigate contradiction resolution. If a conflict is detected, help them decide: does the new learning replace the old, or do both hold in different contexts?

## 6. Confirm

After the CLI completes, report: "Promoted to global knowledge/{axis}/{filename}."

If the promotion was discarded (user chose 'discard' in contradiction resolution), report that outcome without treating it as an error.
