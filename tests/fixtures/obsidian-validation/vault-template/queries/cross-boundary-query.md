---
title: Cross-boundary Dataview query
tags: [query]
---

# Projects tagged `spike`

The Dataview LIST below scans `FROM "projects"` — i.e. everything under the
vault's `projects/` directory, which on disk is a symlink to
`<scratch>/example-project/mnemosyne/`. If Dataview's file walker follows the
symlink, the two project-side notes tagged `spike` appear as list entries.

```dataview
LIST
FROM "projects"
WHERE contains(file.tags, "#spike")
SORT file.name ASC
```

## Expected output

Two rows:

- `projects/example/another-note`
- `projects/example/boundary-note`

Zero rows or an error banner is a **fail** for Check 1.
