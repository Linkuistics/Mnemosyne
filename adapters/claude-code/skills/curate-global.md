---
name: curate-global
description: Reflective curation of global knowledge — validate, supersede, or prune entries. Usage: /curate-global
---

You are running a reflective curation session for the Mnemosyne global knowledge base. Curation is not automated pruning — it is developer-driven reflection, analogous to how humans periodically reconsider their assumptions.

## 1. Check prerequisites

Verify `mnemosyne` is installed:

```bash
which mnemosyne
```

If not found, inform the user: "Mnemosyne is not installed. Install it with `cargo install mnemosyne` to use global knowledge features." Then stop.

## 2. Explain what curation does

Before starting, briefly explain:

> Curation reviews your global knowledge for continued validity. The system will surface:
> - Entries related to recent project activity for your review
> - Entries with implicit divergence (recent project learnings that may contradict global knowledge)
> - Low-confidence entries that may be ready for upgrading or discarding
>
> For each entry you can: **validate** (confirm it still holds), **supersede** (replace with updated understanding), **refine** (adjust scope or nuance), or **prune** (archive it — not deleted, just removed from active rotation).

## 3. Run the curation session

Launch the interactive CLI session:

```bash
mnemosyne curate
```

Stay present during the session to help the user think through decisions. The CLI will present entries and ask for choices — your role is to help interpret the content, surface relevant context from the current conversation, and suggest what action makes sense.

## 4. Guide curation decisions

For each entry presented:

- **Validate**: If the entry still reflects the developer's understanding, confirm it. The CLI updates `last_validated`.
- **Supersede**: If the developer's understanding has changed, help articulate the new understanding. The old content moves to a `## Superseded` section with a reason and date.
- **Refine**: If the entry is mostly right but needs nuance, help edit scope, context, or caveats.
- **Prune**: If the entry is no longer applicable or was wrong, archive it. The CLI moves it to `~/.mnemosyne/archive/` — it is not deleted.

## 5. Summary

After the session completes, summarise: "Curated N entries. Validated: X, Superseded: Y, Refined: Z, Pruned: W."
