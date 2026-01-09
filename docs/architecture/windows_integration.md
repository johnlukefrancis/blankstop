# Windows Integration

This document describes Blankstop's Win32 clipboard integration and the related
runtime gates. It is intentionally scoped to Windows-specific behavior.

## Clipboard listener (Win32 message-only window)
Owner: `src-tauri/src/lib/win/clipboard_listener/window.rs`

How it works:
- A dedicated thread creates a message-only window (`HWND_MESSAGE`).
- The window registers with `AddClipboardFormatListener`.
- Windows sends `WM_CLIPBOARDUPDATE` on clipboard changes.
- The window proc forwards events to `handle_clipboard_update`.

Why this design:
- Event-driven: avoids polling and minimizes latency.
- Message-only window stays invisible and does not appear in the taskbar.

## Clipboard owner exe resolution
Owner: `src-tauri/src/lib/win/process.rs`

Resolution chain:
- `GetClipboardOwner` yields the HWND of the clipboard owner.
- `GetWindowThreadProcessId` resolves the process ID.
- `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` opens the process.
- `QueryFullProcessImageNameW` returns the full path.
- The exe name is extracted and lowercased.

Fallback:
- If the clipboard owner is unavailable, a foreground window exe lookup is
  attempted (for user-facing context only).

Notes:
- Allowlist gating uses the strict clipboard owner exe (not the foreground exe).
- Foreground exe is used for toast/log context if owner is missing.

## Allowlist behavior + config toggles
Owners: `src-tauri/src/lib/state/config.rs`, `src-tauri/src/lib/tray/mod.rs`,
`app/src/config/settings.ts`

Key behavior:
- `only_allowlisted` defaults to true.
- If enabled, the owner exe must be present and in the allowlist.
- If the owner exe cannot be resolved, sanitation is skipped.
- Allowlist entries are normalized (trimmed, lowercased, de-duped) on save.

User controls:
- Settings UI toggles enabled/only_allowlisted/toast behavior.
- Tray menu allows quick enable/disable and pause.

## Self-write loop prevention
Owner: `src-tauri/src/lib/win/clipboard_listener/handler.rs`

Strategy:
- Store a hash of the last written clipboard payload.
- Maintain a short self-write cooldown window (`self_write_until`).
- On `WM_CLIPBOARDUPDATE`, bail out if either guard is active.

This prevents the app from re-processing its own clipboard writes while still
allowing fast response to legitimate external changes.
