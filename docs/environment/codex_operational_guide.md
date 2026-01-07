# Codex Operational Guide — blankstop

Last verified: 2026-01-07

## How we run Codex
- Interactive (recommended): `cd /path/to/blankstop && codex`
- Non-interactive: `codex exec "<task>"`

## Repo workflow (WSL + Windows)
- Edit in WSL; run the app in Windows at `C:\code\blankstop`.
- Use `scripts/sync_to_windows.sh` before running on Windows.
- Prefer pnpm for frontend commands.

## Useful commands
- `pnpm tauri dev` (Windows)
- `pnpm dev` (frontend only)
- `cargo test` (src-tauri)

## VS Code tasks
- Rust: check/build/test/clippy
- Sync: Windows mirror + watch
- Tauri: dev (Windows)
- Zip: bundles (app + src-tauri + scripts; docs optional)

## Bundles (for external review)
- Use `node scripts/zip/zip_blankstop_bundles.mjs` to generate zip bundles.
- Output lives in `scripts/zip/output/`.
- Optional docs bundle: add `--docs`.
