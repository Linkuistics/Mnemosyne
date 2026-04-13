# Six Acceptance Checks

Each check has a binary pass criterion. Capture evidence into
`results/<platform>/check-<n>-<short-name>.{png,md,json}` as appropriate.

All checks run against a materialized vault produced by `setup-vault.sh`, with
Dataview installed and enabled, Obsidian opened on the vault root.

---

## Check 1 — Dataview LIST spans the symlink boundary

**What:** Open `queries/cross-boundary-query.md`. It contains a Dataview LIST
that scans `FROM "projects"` with a tag filter. The entries under
`projects/example/` are reached via the symlink.

**Pass:** The rendered LIST includes at least the two project-side notes
(`projects/example/boundary-note` and `projects/example/another-note`) as
clickable entries. Zero rows or an error message = fail.

**Evidence:** Screenshot of the rendered preview plus, if possible, an
accessibility snapshot of the Dataview output region.

---

## Check 2 — Graph view renders cross-boundary edges

**What:** Open the global graph view (command palette → "Graph view: Open
graph view"). The vault contains a wikilink
`[[obsidian-spike]]` inside `projects/example/boundary-note.md`, which targets
`knowledge/obsidian-spike.md` on the vault-native side. This link crosses the
symlink boundary.

**Pass:** The graph renders the edge between the two nodes. Fail modes: one
or both nodes missing; no edge between them.

**Evidence:** Screenshot of the graph view zoomed so both nodes and the edge
are visible.

---

## Check 3 — Backlinks panel sees cross-boundary references

**What:** Open `knowledge/obsidian-spike.md`. Open the Backlinks panel
(command palette → "Backlinks: Show backlinks").

**Pass:** The Backlinks panel lists `projects/example/boundary-note.md` as a
linked mention. Empty list or missing entry = fail.

**Evidence:** Screenshot of the Backlinks panel with the entry visible.

---

## Check 4 — File tree pane navigates into the symlinked subtree

**What:** In the file explorer pane, expand `projects/example/` and click each
of the notes inside.

**Pass:** The three files (`README.md`, `boundary-note.md`, `another-note.md`)
appear in the tree and open normally when clicked, rendering their markdown
content. Fail modes: the subtree is empty in the file tree; clicking notes
fails; notes open as unknown file type.

**Evidence:** Screenshot of the expanded file tree plus a screenshot of
`boundary-note.md` open in the main pane.

---

## Check 5 — File watcher reflects external edits to symlinked notes

**What:** With `projects/example/boundary-note.md` open in Obsidian, append a
line to the same file from the shell (e.g.
`echo "externally appended" >> .../example-project/mnemosyne/boundary-note.md`).

**Pass:** Within 5 seconds, Obsidian re-renders the note to show the appended
line without a manual reload. Fail modes: stale content shown indefinitely;
Obsidian shows a stale-file warning; user must close and reopen.

**Evidence:** Screenshot before the edit, the shell command run, screenshot
after (showing the new line). Record the elapsed time informally.

---

## Check 6 — Obsidian safety checks do not block the symlinked vault

**What:** Open the vault fresh (Obsidian → "Open folder as vault" →
`<scratch>/vault/`). Observe any warnings, prompts, or trust dialogs.

**Pass:** Obsidian opens the vault without blocking. Informational messages
about symlinks are acceptable (note them). Fail modes: Obsidian refuses to
open; a modal dialog blocks vault load; the symlinked subtree is quietly
hidden.

**Evidence:** Screenshot of Obsidian's first-open state. Note any warnings.

---

## Aggregating the result per platform

Append a `results/<platform>/result.md` with a one-row summary per check:

```
| Check | Pass? | Evidence file | Notes |
|-------|-------|---------------|-------|
| 1 - dataview across boundary | Y/N | check-1-dataview.png | ... |
| 2 - graph view               | Y/N | check-2-graph.png    | ... |
| 3 - backlinks                | Y/N | check-3-backlinks.png | ... |
| 4 - file tree                | Y/N | check-4-tree.png     | ... |
| 5 - file watcher             | Y/N | check-5-watcher-{before,after}.png | elapsed: Ns |
| 6 - safety checks            | Y/N | check-6-first-open.png | ... |
```

Platform passes iff all six rows are Y. Overall spike passes iff macOS and
Linux both pass.
