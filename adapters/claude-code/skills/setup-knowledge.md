---
name: setup-knowledge
description: Scaffold the observational memory knowledge system for a new project. Usage: /setup-knowledge
---

You are scaffolding the observational memory knowledge system for this project.

## 1. Understand the project

Ask the user: "What are the main organisational axes for this project?" Provide examples:
- "For a web app: services, features, infrastructure"
- "For a multi-language project: targets, apps, pipeline"
- "For a monorepo: packages, shared-libs, deployment"

Get from the user:
- **Primary axis** name and example entries
- **Secondary axis** name and example entries (optional)
- **Matrix axis** name (default: "matrix" — intersection of primary x secondary)
- **Extra axes** — any additional knowledge categories (e.g., pipeline, testing, deployment)
- **Review cadence** — steps between code review sessions (default: 5)
- **Knowledge directory** — where to create it (default: `knowledge/`)
- **Plans directory** — where plans live (default: `LLM_STATE/plans/`)

## 2. Create directory structure

Create the following directories:
- `{knowledge_dir}/`
- `{knowledge_dir}/{primary_axis}/`
- `{knowledge_dir}/{secondary_axis}/` (if specified)
- `{knowledge_dir}/{matrix_axis}/` (if both primary and secondary exist)
- `{knowledge_dir}/{extra}/` for each extra axis
- `{plans_dir}/`

## 3. Create knowledge CLAUDE.md

Write `{knowledge_dir}/CLAUDE.md` with:

```markdown
# Knowledge Base

This directory contains all project learnings organised by axis. It is the single source of
truth for discoveries made during implementation.

## Axes

| Axis | Path | What goes here |
|------|------|----------------|
| {primary} | `{primary}/` | {description} |
| {secondary} | `{secondary}/` | {description} |
| {matrix} | `{matrix}/{secondary_entry}/{primary_entry}.md` | Intersection discoveries |
{extra rows for each extra axis}

## Rules

- **Never duplicate** — if a learning applies to a broader axis, put it there
- **Date entries** — prefix with `**YYYY-MM-DD:**` so staleness is visible
- **Link don't copy** — CLAUDE.md files point here; knowledge is not inlined
- **Promote eagerly** — if a narrow learning turns out to be broadly applicable, move it up
- **Priority codes** — 🔴 (critical), 🟡 (useful), 🟢 (informational)
```

## 4. Create configuration file

Write `.observational-memory.yml` in the project root:

```yaml
knowledge_dir: {knowledge_dir}
plans_dir: {plans_dir}
axes:
  primary: {primary_axis}
  secondary: {secondary_axis}
  matrix: {matrix_axis}
  extras:
{extra axes as list}
review_cadence: {cadence}
```

## 5. Update root CLAUDE.md

Add to the project's root CLAUDE.md (at the top, after any existing header):

```markdown
## Knowledge System

This project uses observational memory for structured knowledge capture.

- `{knowledge_dir}/` — all learnings, organised by axis (see `{knowledge_dir}/CLAUDE.md`)
- `{plans_dir}/` — active multi-session plans with Do/Verify/Observe steps

**Skills:**
- `/begin-work <{primary_axis}> [{secondary_axis}]` — start or continue work with full context
- `/reflect` — promote observations to the knowledge base during code review
```

## 6. Initialise global knowledge store (if not already set up)

Check whether `mnemosyne` is installed and `~/.mnemosyne/` does not already exist:

```bash
which mnemosyne && [ ! -d ~/.mnemosyne ]
```

If both conditions are met, offer: "Mnemosyne global knowledge is available but not yet initialised. Run `mnemosyne init` to set up the global store? [y/N]"

If the user accepts, run:

```bash
mnemosyne init
```

If `mnemosyne` is not installed or `~/.mnemosyne/` already exists, skip this step silently.

## 7. Offer to scaffold existing directories

If the project already has directories that map to the axes (e.g., `src/services/auth/`), offer to create CLAUDE.md routing files in them that point to the relevant knowledge files.

## 8. Summary

Report what was created and suggest next steps:
- "Knowledge system scaffolded. Next: populate initial knowledge files, or start work with /begin-work."
