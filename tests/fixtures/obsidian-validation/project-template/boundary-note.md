---
title: Boundary Note
tags: [spike, cross-boundary]
---

This note lives on the project side of the symlink boundary. It exists at
`<scratch>/example-project/mnemosyne/boundary-note.md` on disk and is
reached by Obsidian via `<vault>/projects/example/boundary-note.md` through
the symlink.

## Cross-boundary wikilink

It links back into the vault's knowledge tree: [[obsidian-spike]].

If Obsidian's link resolver follows the symlink, clicking this wikilink opens
`knowledge/obsidian-spike.md` in the vault-native subtree.

## What this note exercises

- **Check 1** (Dataview): Dataview's `FROM "projects"` scan must see this
  file and include it in the LIST rendered by
  `queries/cross-boundary-query.md`.
- **Check 2** (Graph): the wikilink above must render as an edge in the
  global graph view, connecting this node to `obsidian-spike`.
- **Check 3** (Backlinks): opening `knowledge/obsidian-spike.md` and viewing
  its Backlinks panel must list this note as a linked mention.
