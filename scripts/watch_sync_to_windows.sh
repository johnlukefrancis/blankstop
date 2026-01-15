#!/usr/bin/env bash
set -euo pipefail

SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST_DIR="${BLANKSTOP_WIN_PATH:-/mnt/d/code/blankstop}"

if ! command -v rsync >/dev/null 2>&1; then
  echo "rsync not found. Install it with: sudo apt install -y rsync" >&2
  exit 1
fi

if ! command -v inotifywait >/dev/null 2>&1; then
  echo "inotifywait not found. Install it with: sudo apt install -y inotify-tools" >&2
  exit 1
fi

mkdir -p "$DEST_DIR"

sync_once() {
  rsync -a --delete \
    --exclude ".git" \
    --exclude "node_modules" \
    --exclude "dist" \
    --exclude "dist-ssr" \
    --exclude "src-tauri/target" \
    "$SOURCE_DIR/" "$DEST_DIR/"
}

sync_once

echo "Watching $SOURCE_DIR -> $DEST_DIR"
while inotifywait -r -e modify,create,delete,move \
  --exclude "(\\.git|node_modules|dist|dist-ssr|src-tauri/target)" \
  "$SOURCE_DIR" >/dev/null 2>&1; do
  sync_once
  echo "Synced at $(date +%H:%M:%S)"
  sleep 0.1
done
