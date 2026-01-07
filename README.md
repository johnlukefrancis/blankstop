# Blankstop

Blankstop is a Windows-first tray app that watches clipboard updates from terminal apps and cleans up soft-wrapped text before you paste.

## What it does
- Runs in the system tray with no main window on launch.
- Listens to clipboard changes using Win32 events (no polling).
- Only sanitizes text copied from allowlisted terminal processes by default.
- Shows a brief glass-style toast when it modifies the clipboard.
- Global shortcut: Ctrl+Alt+Shift+V toggles Enabled.

## Development

Install dependencies:

```bash
pnpm install
```

Run the app:

```bash
pnpm tauri dev
```

Build the app:

```bash
pnpm tauri build
```

## Allowlist

Edit the allowlist in Settings (one executable per line). Defaults include:
- windowsterminal.exe
- wt.exe
- code.exe
- conhost.exe
- pwsh.exe
- powershell.exe
- cmd.exe
- bash.exe
- wsl.exe

## Security note

Blankstop only modifies clipboard text when the source process is allowlisted (unless you disable that toggle in Settings). It does not collect telemetry or send data anywhere.
