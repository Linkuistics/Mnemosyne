#!/usr/bin/env bash
set -euo pipefail

# Materialize the Obsidian symlink validation vault into a scratch directory.
#
# Produces:
#   <scratch>/vault/                        (copy of vault-template/)
#   <scratch>/example-project/mnemosyne/    (copy of project-template/)
#   <scratch>/vault/projects/example        -> ../../example-project/mnemosyne  (symlink)
#
# The symlink is RELATIVE so the vault survives being uploaded between hosts.
#
# Environment:
#   DATAVIEW_VERSION   Dataview release tag (default: 0.5.67)
#   SKIP_DATAVIEW      set to 1 to skip Dataview download (manual install instead)

DATAVIEW_VERSION="${DATAVIEW_VERSION:-0.5.67}"
SKIP_DATAVIEW="${SKIP_DATAVIEW:-0}"

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <scratch-dir>" >&2
  exit 1
fi

SCRATCH="$1"
HERE="$(cd "$(dirname "$0")" && pwd)"

mkdir -p "$SCRATCH"
rm -rf "$SCRATCH/vault" "$SCRATCH/example-project"

cp -R "$HERE/vault-template" "$SCRATCH/vault"
mkdir -p "$SCRATCH/example-project"
cp -R "$HERE/project-template" "$SCRATCH/example-project/mnemosyne"

mkdir -p "$SCRATCH/vault/projects"
( cd "$SCRATCH/vault/projects" && ln -sfn ../../example-project/mnemosyne example )

if [[ "$SKIP_DATAVIEW" != "1" ]]; then
  DATAVIEW_DIR="$SCRATCH/vault/.obsidian/plugins/dataview"
  mkdir -p "$DATAVIEW_DIR"
  BASE="https://github.com/blacksmithgu/obsidian-dataview/releases/download/${DATAVIEW_VERSION}"
  for f in main.js manifest.json styles.css; do
    curl -fsSL "$BASE/$f" -o "$DATAVIEW_DIR/$f"
  done
  echo "Installed Dataview ${DATAVIEW_VERSION} into $DATAVIEW_DIR"
fi

echo ""
echo "Vault materialized at: $SCRATCH/vault"
echo "Symlink:               $SCRATCH/vault/projects/example -> ../../example-project/mnemosyne"
echo ""
echo "Next: open the vault in Obsidian and work through checks.md"
