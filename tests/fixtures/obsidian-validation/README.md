# Obsidian Symlink Validation Spike

Reproducible fixture for validating that Obsidian's Dataview, graph view, and
backlink tracking work correctly when a subtree of the vault is reached via a
filesystem symlink rather than being a real directory.

This is the load-bearing pre-implementation blocker for the
`dedicated Mnemosyne-vault with symlinked per-project directories`
architectural decision (see `LLM_STATE/mnemosyne-orchestrator/memory.md`).

## Why

The orchestrator design places each project's Mnemosyne content at
`<project>/mnemosyne/` inside the project repo, and hosts a dedicated
`<dev-root>/Mnemosyne-vault/` alongside the project repos. Per-project
content appears under the vault via one symlink per project at
`<vault>/projects/<project-name>` targeting `<project>/mnemosyne/`.

If Obsidian's core features do not work across a symlink boundary on
**both macOS and Linux**, this layout must fall back to a hard-copy staging
model with two-way sync before any implementation proceeds.

## Layout

```
tests/fixtures/obsidian-validation/
├── README.md           this file
├── VERSIONS.md         pinned versions of Obsidian and Dataview used per run
├── checks.md           the six acceptance checks with pass criteria
├── setup-vault.sh      materializes a runnable vault from the templates
├── vault-template/     vault-side contents; symlink is created at runtime
│   ├── .obsidian/      minimal Obsidian config with Dataview listed as community plugin
│   ├── README.md       vault-level index with cross-boundary wikilink
│   ├── knowledge/      vault-native notes
│   │   ├── obsidian-spike.md    target of cross-boundary wikilink
│   │   └── async-patterns.md    filler so graph view has more than one node
│   └── queries/
│       └── cross-boundary-query.md   Dataview LIST scanning projects/
├── project-template/   project-side contents; becomes <scratch>/example-project/mnemosyne/
│   ├── README.md
│   ├── boundary-note.md          cross-boundary wikilink into knowledge/
│   └── another-note.md           second project-side note for multi-row Dataview output
└── results/            captured per-run evidence; created on demand
    ├── macos/
    └── linux/
```

## How to run (host-side materialization)

```sh
./setup-vault.sh /tmp/obsidian-spike
```

This produces:

```
/tmp/obsidian-spike/
├── vault/                             (copy of vault-template/)
│   └── projects/example -> ../../example-project/mnemosyne   (relative symlink)
└── example-project/
    └── mnemosyne/                     (copy of project-template/)
```

After materialization, install Dataview into
`/tmp/obsidian-spike/vault/.obsidian/plugins/dataview/` (the setup script does
this automatically if `curl` and `unzip` are available), then open the vault
in Obsidian and work through `checks.md`.

## How to run (VM-driven via GUIVisionVMDriver)

See `../../../LLM_STATE/mnemosyne-orchestrator/session-log.md` for the
specific session run that produced the results in `results/macos/` and
`results/linux/`. The run invokes `guivision` commands from the host to
upload this fixture into each golden image, install Obsidian, run
`setup-vault.sh` inside the guest, launch Obsidian, and capture each of the
six checks via a mix of `guivision agent snapshot` (accessibility), `guivision
screenshot` (framebuffer), and `guivision find-text` (OCR against the
screenshot).

Evidence captured per platform:

- Full screenshots of each check being verified (PNG)
- Dataview query output as OCR'd text or accessibility snapshot JSON
- Obsidian's console log (downloaded via `guivision download`)
- A short `result.md` summarizing pass/fail per check with references to
  the evidence files

## Pass/fail criteria

Each of the six checks in `checks.md` has a binary pass criterion. The spike
as a whole passes on a platform iff all six checks pass on that platform.
The spike passes overall iff it passes on **both** macOS and Linux.

If the spike fails on either platform, the architectural decision falls back
to a hard-copy staging model and sub-project A's brainstorm must absorb the
fallback end-to-end before any implementation proceeds (see memory.md).
