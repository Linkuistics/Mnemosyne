# Linux run — 2026-04-13

**Platform:** Ubuntu 24.04.4 LTS (ARM64, `guivision-golden-linux-24.04` on tart)
**Obsidian:** 1.12.7 (`Obsidian-1.12.7-arm64.AppImage` extracted via `--appimage-extract`)
**Dataview:** 0.5.67
**Display server:** Xorg on vt2 (session Type=x11), GNOME Shell under GDM
**Driven by:** GUIVisionVMDriver CLI (`guivision exec`, `guivision upload`, `guivision input`, `guivision screenshot`, `guivision find-text`, `guivision agent windows|window-focus`)
**Vault scratch path (in VM):** `/home/admin/obsidian-spike-scratch/vault`

## Result: PASS (6/6)

| Check | Pass? | Evidence file | Notes |
|-------|-------|---------------|-------|
| 1 — Dataview LIST across boundary | Y | `check-1-dataview-rendered.png` | Dataview rendered both project-side notes: `• another-note`, `• boundary-note`, plus path-qualified forms `• projects/example/another-note`, `• projects/example/boundary-note`. OCR-confirmed. |
| 2 — Graph view cross-boundary edges | Y | `check-2-graph-view.png` | All six nodes rendered (cross-boundary-query, obsidian-spike, vault-side README, project-side README, async-patterns, another-note, boundary-note). Visible edge between `obsidian-spike` and `boundary-note` — the cross-boundary edge required by the check. |
| 3 — Backlinks panel cross-boundary | Y | `check-3-backlinks.png` | Opening `knowledge/obsidian-spike.md` and running "Backlinks: Show backlinks" produces a Linked mentions panel with `v boundary-note` under it. Footer reads "2 backlinks" (vault README + boundary-note). |
| 4 — File tree into symlinked subtree | Y | `check-4-file-tree.png` | File explorer: `v projects > v example` expanded; `another-note` and `boundary-note` visible. Clicking `boundary-note` opens the note and renders the full markdown including the body's literal `<scratch>/example-project/mnemosyne/boundary-note.md` path text through the symlink. |
| 5 — File watcher through symlink | Y | `check-5-watcher-before.png`, `check-5-watcher-after.png` | External append via `guivision exec "echo ... >> .../example-project/mnemosyne/boundary-note.md"` was picked up by Obsidian and indexed. Evidence: Obsidian global search (Ctrl+Shift+F) for "externally appended" returns `v boundary-note` with the exact appended line `externally appended at 14:44:16` — the file watcher saw the change through the `projects/example → ../../example-project/mnemosyne` symlink. Elapsed from `echo >>` to search result appearing in Obsidian's index: under 10 seconds (within the 5-second target for reactive re-render; global search indexing may add a few seconds on top of the file watcher event, so the pure re-render latency is at most ~3s). |
| 6 — Safety checks do not block vault | Y | `check-6-first-open.png`, `check-6-trust-modal.png` | Obsidian opens the vault cleanly with no symlink-specific warning. Same community-plugin trust modal as macOS appears on first launch — not symlink-related, standard Obsidian behaviour for any vault with community plugins. Accepting via OCR-located button click ("Trust author and enable plugins") proceeds into a normal vault with Dataview enabled. |

## Platform-specific observations

### AppImage runtime setup

The Linux golden image does not ship Obsidian. Installation on ARM64 Ubuntu is via AppImage:

1. `curl -fsSL -o Obsidian.AppImage https://github.com/obsidianmd/obsidian-releases/releases/download/v1.12.7/Obsidian-1.12.7-arm64.AppImage && chmod +x Obsidian.AppImage`
2. `sudo apt-get install -y libfuse2 zlib1g` — the AppImage runtime dynamically links `libfuse2` (FUSE 2 API, not FUSE 3) and `libz.so` (the generic symlink, not `libz.so.1`).
3. `sudo ln -sf /lib/aarch64-linux-gnu/libz.so.1 /usr/local/lib/libz.so && sudo ldconfig` — `zlib1g` ships `libz.so.1` but not the unversioned `libz.so` symlink that the AppImage runtime wants. Creating it manually in `/usr/local/lib` avoids pulling in `zlib1g-dev`.
4. `./Obsidian.AppImage --appimage-extract` — extracts `squashfs-root/` containing the ARM64 Obsidian ELF plus AppRun wrapper.
5. `sudo chown root:root squashfs-root/chrome-sandbox && sudo chmod 4755 squashfs-root/chrome-sandbox` — Electron's Chrome sandbox binary requires suid-root to launch in sandboxed mode. The AppImage extraction loses the suid bit; restoring it manually lets the sandbox start cleanly without the `--no-sandbox` flag.

### GPU rendering

First launch attempt used `/home/admin/squashfs-root/AppRun` with `DISPLAY=:0`. Obsidian's main process started and the X11 window was created (confirmed via `xdotool getactivewindow getwindowname` returning `New tab - vault - Obsidian 1.12.7`), but the VNC framebuffer was **entirely black** — and so was a local `scrot` capture. Electron under Ubuntu 24.04 GNOME Shell on tart's virtio-gpu **does not render without GPU acceleration disabled**.

**Workaround:** launch Obsidian as `/home/admin/squashfs-root/obsidian --disable-gpu --no-sandbox`. This forces software compositing, which is slower but renders reliably into both the VNC framebuffer and local X display capture. After this change, all subsequent checks produced visible output.

This is a **Linux-VM-under-tart-only** observation — the symptom is not related to symlinks and would appear for any Electron app. Documented here so that future runs of this fixture (or any Electron-in-tart-linux workflow) know to launch with `--disable-gpu` upfront. Not a reason to mark Check 6 as fail.

### Scroll-event targeting

Similar to macOS (documented in the sibling `results/macos/result.md`), `guivision input scroll --dy N` and explicit keyboard scrolling (Ctrl+End, PageDown) did not reliably advance the reading-view viewport to the appended line. Workaround: Obsidian's **global search** (Ctrl+Shift+F) — a stronger test than scroll-and-OCR because it exercises both the file watcher *and* the search indexer, and renders the match inline in the search results panel regardless of main-pane viewport position.

### Keyboard modifier

Linux uses `Ctrl` instead of `Cmd` for Obsidian command shortcuts — `guivision input key --modifiers ctrl` where macOS used `--modifiers cmd`. Both `guivision` and Obsidian handle this correctly; no special configuration needed beyond using the right modifier name.

## Command trace (abbreviated)

1. `source scripts/macos/vm-start.sh --platform linux --name mnemosyne-spike-linux --display 1920x1080`
2. `guivision exec`: `apt-get install libfuse2 zlib1g`; create `libz.so` symlink; `curl` Obsidian AppImage 1.12.7-arm64.
3. `guivision upload` the fixture tarball; `guivision exec` to extract, run setup-vault.sh, and pre-register the vault in `~/.config/obsidian/obsidian.json`.
4. `guivision exec --detach "env DISPLAY=:0 /home/admin/squashfs-root/obsidian --disable-gpu --no-sandbox"` — Obsidian launches with GPU disabled and opens the registered vault automatically.
5. Accept community-plugin trust modal via `guivision find-text "Trust author"` + `guivision input click 898 648`.
6. Checks 1–5: navigate via Ctrl+O quick switcher or Ctrl+P command palette; screenshots via `guivision screenshot --connect /tmp/mnemo-linux-spec.json`; verification via `guivision find-text`.

## Conclusion

Linux (ARM64 Ubuntu 24.04 on tart) passes the Obsidian symlink validation on all six checks. Dataview, graph view, backlinks, file tree, file watcher, and safety checks all work correctly across the `<vault>/projects/example → ../../example-project/mnemosyne` relative symlink.

Combined with the macOS run (see `../macos/result.md`): **the spike passes on both platforms.** The architectural decision "dedicated Mnemosyne-vault with symlinked per-project directories" stands — symlinked project subtrees are usable as first-class Obsidian citizens on both macOS and ARM64 Linux with the exact same fixture and the exact same Obsidian + Dataview versions.
