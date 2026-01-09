# Blankstop

Blankstop is a Windows-first tray app that watches clipboard updates from terminal
apps and cleans up soft-wrapped text before you paste.

## Install
- GitHub Releases (placeholder until releases exist).
- For local installers and packaging notes, see
  `docs/release/windows_installer.md`.

## What it does
- Runs in the system tray with no main window on launch.
- Listens to clipboard changes using Win32 events (no polling).
- Only sanitizes text copied from allowlisted processes by default.
- Sanitizer uses a JS parse "oracle": JS-like text is only rewritten if the
  reflowed result parses as valid JS; otherwise the clipboard stays unchanged.
- Shows a brief glass-style toast when it modifies the clipboard.
- Global shortcut: Ctrl+Alt+Shift+V toggles Enabled.

## When it will NOT change your clipboard
- Blankstop is disabled (tray toggle or settings).
- Pause is active (tray menu).
- `only_allowlisted` is on and the clipboard owner exe is not allowlisted (or
  owner exe can’t be resolved).
- JS-like text failed parse validation (oracle rejects, so clipboard unchanged).
- Sanitizer produced no material changes (output matches normalized input).

## Settings
Settings are in the tray menu (quick toggles) and the settings window.

Toggles & fields:
- Enabled
- Toast enabled
- Status toast enabled
- Only allowlisted
- Allowlist (one exe per line)
- Run on startup

Debug surfaces:
- Settings UI shows last source exe and last action time.
- Deeper debug fields (`debug_last_clipboard`, `debug_last_summary`) exist in
  `UiState` but are not rendered in the UI; inspect via devtools by invoking
  `get_ui_state`.

## Development
Install dependencies:

```bash
pnpm install
```

WSL -> Windows workflow (required for running the app on Windows):
- Sync to Windows:
  - One-shot: `scripts/sync_to_windows.sh`
  - Watch mode: `scripts/watch_sync_to_windows.sh`
- Optional: set `BLANKSTOP_WIN_PATH` to override the Windows path
  (default: `/mnt/c/code/blankstop`).

Run the app on Windows (from the mirrored repo):

```bash
pnpm tauri dev
```

Build the app:

```bash
pnpm tauri build
```

## Troubleshooting (quick)
See `docs/runbook/troubleshooting.md` for symptom → cause → where to inspect.

## Docs
- `docs/architecture/system_overview.md`
- `docs/architecture/sanitizer_pipeline.md`
- `docs/architecture/windows_integration.md`
- `docs/release/windows_installer.md`
- `docs/runbook/troubleshooting.md`

## Security & privacy
- Local-only: no telemetry, no network calls for clipboard data.
- Allowlist is on by default; only allowlisted executables can trigger changes.
- Clipboard is only rewritten when sanitization makes a validated change.
