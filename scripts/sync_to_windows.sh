#!/usr/bin/env bash
set -euo pipefail

SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST_DIR="${BLANKSTOP_WIN_PATH:-/mnt/c/code/blankstop}"

if ! command -v rsync >/dev/null 2>&1; then
  echo "rsync not found. Install it with: sudo apt install -y rsync" >&2
  exit 1
fi

mkdir -p "$DEST_DIR"

rsync -a --delete \
  --exclude ".git" \
  --exclude "node_modules" \
  --exclude "dist" \
  --exclude "dist-ssr" \
  --exclude "src-tauri/target" \
  "$SOURCE_DIR/" "$DEST_DIR/"

echo "Synced to $DEST_DIR"
