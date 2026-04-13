# Pinned Versions

The spike must run with identical Obsidian and Dataview versions on both
platforms, and the versions must be recorded alongside results so future
re-runs can reproduce (or deliberately refresh) the pin.

## Default pins (updated when the spike is re-run)

| Component | Version | Source |
|-----------|---------|--------|
| Obsidian  | 1.12.7  | macOS: pre-installed in `guivision-golden-macos-tahoe` (brew cask); Linux: `Obsidian-1.12.7-arm64.AppImage` from the GitHub release, extracted via `--appimage-extract`, launched with `--disable-gpu --no-sandbox` |
| Dataview  | 0.5.67  | <https://github.com/blacksmithgu/obsidian-dataview/releases/tag/0.5.67> |

`setup-vault.sh` fetches Dataview from the pinned GitHub release tag. To
override, pass `DATAVIEW_VERSION=x.y.z` in the environment.

Obsidian is **not** installed by `setup-vault.sh`. Each platform's install
procedure is part of the VM-driver script that opens Obsidian.

## Per-run record

Each run appends a row below with the actual versions observed, the date, and
the evidence commit hash.

| Run date | Platform | Obsidian | Dataview | Evidence | Result |
|----------|----------|----------|----------|----------|--------|
| 2026-04-13 | macOS Tahoe (Apple Silicon, `guivision-golden-macos-tahoe`) | 1.12.7 | 0.5.67 | `results/macos/` | PASS 6/6 |
| 2026-04-13 | Ubuntu 24.04 ARM64 (`guivision-golden-linux-24.04`), GNOME/Xorg, Electron software rendering | 1.12.7 | 0.5.67 | `results/linux/` | PASS 6/6 |
