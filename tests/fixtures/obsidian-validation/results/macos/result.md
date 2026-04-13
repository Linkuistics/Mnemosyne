# macOS run — 2026-04-13

**Platform:** macOS Tahoe (Apple Silicon, `guivision-golden-macos-tahoe`)
**Obsidian:** 1.12.7
**Dataview:** 0.5.67
**Driven by:** GUIVisionVMDriver CLI (`guivision exec`, `guivision upload`, `guivision input`, `guivision screenshot`, `guivision find-text`, `guivision agent windows|window-focus`)
**Vault scratch path (in VM):** `/Users/admin/obsidian-spike-scratch/vault`

## Result: PASS (6/6)

| Check | Pass? | Evidence file | Notes |
|-------|-------|---------------|-------|
| 1 — Dataview LIST across boundary | Y | `check-1-dataview-rendered.png` | Both project-side notes visible as list items (`• another-note`, `• boundary-note`). OCR confirmed. |
| 2 — Graph view cross-boundary edges | Y | `check-2-graph-view.png` | All six nodes rendered (cross-boundary-query, README, obsidian-spike, boundary-note, async-patterns, another-note); visible edge between `obsidian-spike` and `boundary-note`. |
| 3 — Backlinks panel cross-boundary | Y | `check-3-backlinks.png` | Backlinks panel lists `projects/example/boundary-note` under Linked mentions; footer reads "2 backlinks" (README.md + boundary-note). |
| 4 — File tree into symlinked subtree | Y | `check-4-file-tree.png` | `v projects > v example` expanded; all three notes (`another-note`, `boundary-note`, `README`) visible and openable; `boundary-note` opened and rendered the full markdown body via the symlink path. |
| 5 — File watcher through symlink | Y | `check-5-watcher-before.png`, `check-5-watcher-after.png`, `check-5-search.png` | External append via `guivision exec "echo ... >> .../example-project/mnemosyne/boundary-note.md"` picked up by Obsidian within ~3s (elapsed end-to-end 4s including snapshot call). The rendered note's DOM was updated — `externally appended at 14:06:11` found via both Cmd+F in-file search and OCR. Caveat: the appended line was scrolled below the initial viewport fold; earlier scroll attempts via `guivision input scroll` landed on the wrong pane and did not advance the content area. In-file search was a reliable workaround for evidence capture. |
| 6 — Safety checks do not block vault | Y | `check-6-first-open.png`, `check-6-trust-modal.png` | Obsidian opens the vault without any symlink-specific warning. It does show the **standard community-plugin trust modal** on first launch of any vault that ships with community plugins enabled — this is plugin-trust UX, not symlink-related, and accepting it proceeds into a normal vault with Dataview enabled. |

## Observations / caveats

- **Plugin-trust modal on first launch** (Check 6). The `vault-template/.obsidian/community-plugins.json` lists `dataview` as an installed community plugin, so the first launch of the vault shows Obsidian's standard "Do you trust the author of this vault?" modal. This is NOT a symlink check — the same modal would appear for any vault with community plugins regardless of whether it is on native or symlinked storage. Accepting ("Trust author and enable plugins") proceeds cleanly; Dataview then renders Check 1's LIST. The fixture could suppress this modal in future by omitting community-plugins.json and having the run script install Dataview via agent-side setup after the vault is first opened, but the current behaviour is considered standard for v1 fixtures.
- **`guivision input scroll` + Notification Center overlay.** macOS Tahoe's desktop-resident Notification Center widgets (Featured / Forecast / Month, plus a "What's new in macOS Tahoe" Discover popover) render on top of the Obsidian window area and appear to intercept mouse-wheel scroll events in some cases — `guivision input scroll --dy 5` at `(800, 500)` did not scroll the reading-view content when the Notification Center popover was visible, and byte-identical screenshots confirmed no viewport change. Workarounds that worked reliably: (a) in-file search via Cmd+F, which scrolls the rendered view to the match, and (b) capturing OCR text from the rendered DOM regardless of viewport position. Not a symlink issue; noted here for future spike runs.
- **`guivision find-text` returns coordinates in VNC display pixels**, not window-relative points. All clicks used screen-absolute coords.
- **`guivision` CLI has no `--vnc-password` flag** — the VNC password is supplied exclusively via the `--connect <spec.json>` option, where the JSON spec file is `{"vnc": {"host": ..., "port": ..., "password": "..."}, "agent": {...}, "platform": "..."}`. The password came from the `GUIVISION_VNC_PASSWORD` env var exported by `vm-start.sh`.

## Command trace

Sequence (abbreviated) for reproducibility:

1. `source scripts/macos/vm-start.sh --platform macos --name mnemosyne-spike-macos --display 1920x1080`
2. Fixture tarred locally, `guivision upload` to `/tmp/mnemo-fixture.tar`, extracted in-VM via `guivision exec "tar xf ..."`.
3. `guivision exec "bash .../setup-vault.sh /Users/admin/obsidian-spike-scratch"` — vault materialized, Dataview 0.5.67 fetched from the pinned GitHub release.
4. Obsidian config written locally (`vaults.mnemospike00001.path = <vault>`, `open = true`) and uploaded to `~/Library/Application Support/obsidian/obsidian.json` via `guivision upload`.
5. `guivision exec "open -a Obsidian"` — Obsidian auto-opened the registered vault on launch.
6. Plugin-trust modal accepted via `guivision find-text "Trust author"` + `guivision input click 441 479`.
7. For each of Checks 1–5: navigated via Cmd+O quick switcher or Cmd+P command palette; captured screenshots via `guivision screenshot --connect spec.json`; verified rendered content via `guivision find-text --connect spec.json`.

## Conclusion

macOS (Apple Silicon, Tahoe) passes the Obsidian symlink validation on all six checks. Dataview, graph view, backlinks, file tree, file watcher, and safety checks all work correctly across the `<vault>/projects/example → ../../example-project/mnemosyne` relative symlink.

This is half of the spike outcome. The architectural decision (dedicated Mnemosyne-vault with symlinked per-project directories) holds on macOS. The Linux run via `guivision-golden-linux-24.04` is captured in a sibling `result.md` alongside.
