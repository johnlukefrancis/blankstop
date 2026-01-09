# System Overview (Blankstop)

This document describes Blankstop's current architecture and runtime data flow.
It reflects the present end-state (tray-only app with Win32 clipboard listener,
allowlist gating, sanitizer with JS validation + fallback, toast + settings UI).

## Goals
- Event-driven clipboard sanitation for terminal copy/paste on Windows.
- Minimize UI surface (tray-first; settings window only when opened).
- Respect user intent via allowlist gating and explicit enable/disable.
- Provide fast feedback via toast notifications.
- Keep the architecture modular and inspectable by module ownership.

## Non-goals
- Cross-platform clipboard support (Windows is the active target).
- Background polling or periodic scanning of clipboard contents.
- Heavy UI framework or persistent main window.
- Sanitizing non-text clipboard formats.

## High-level component map

Rust backend (Tauri):
- `src-tauri/src/lib/mod.rs`: app setup, tray init, global shortcut, window setup.
- `src-tauri/src/lib/commands/`: Tauri invoke entry points for settings.
- `src-tauri/src/lib/state/`: config load/save + runtime shared state.
- `src-tauri/src/lib/win/clipboard_listener/`: Win32 message-only window and handler.
- `src-tauri/src/lib/win/process.rs`: resolve clipboard owner / foreground exe.
- `src-tauri/src/lib/sanitize/`: text cleanup + JS reflow/parse validation.
- `src-tauri/src/lib/tray/`: tray menu, settings window toggle, sanitize-now entry.
- `src-tauri/src/lib/toast/`: toast window creation + display.

Frontend windows (webview):
- Settings window (label `main`) is hidden on launch; opened via tray menu.
- Toast window (label `toast`) is transparent, always-on-top, and hidden by default.
- Both are served by `app/src/` and share the same bundle, with runtime mode selection.

## Clipboard event flow

1) Win32 event
- A message-only window listens to `WM_CLIPBOARDUPDATE` (no polling).

2) Pre-checks
- Guardrails: enabled flag, pause state, and self-write cooldown gate.
- Read current clipboard text; if not text, stop.
- If clipboard hash matches last written hash, stop.

3) Source exe resolution
- Resolve clipboard owner exe (strict) and optional foreground exe fallback.
- Source exe is used for gating and user-facing context in toasts/logs.

4) Allowlist gate
- If `only_allowlisted` is true and owner exe is missing or not allowlisted, stop.

5) Sanitize
- Normalize + clean text, then:
  - JS path: attempt JS reflow, validate by parsing; only rewrite if parse succeeds.
  - Fallback path: text-mode unwrapping and cleanup.
- Large input guard: oversized payloads skip JS parse and text unwrapping; only
  clean + trailing whitespace trim runs.
- If sanitized output equals normalized input, stop (no rewrite, no toast).

6) Conditional rewrite
- Rewrite clipboard with CRLF line endings (Windows-friendly).
- Update runtime state (hash, self-write window, last source exe, log entry).

7) Toast
- If toast is enabled, show a brief toast (includes source exe when available).

## State and config flow

Persistence:
- Config is loaded on startup from app config dir: `config.json`.
- Config is normalized (allowlist trimmed, lowercased, de-duped).
- Config is saved on any settings change and when tray toggles update it.

Runtime state (non-persistent):
- SharedState (mutex) tracks:
  - `config` (current settings)
  - `paused_until`
  - `last_written_hash` and `self_write_until` (self-write loop guard)
  - `last_source_exe`
  - `log` (recent sanitize events)
  - `debug_last_clipboard` + `debug_last_summary` (truncated; raw clipboard is
    dev-only and never captured/emitted in release builds)

UI sync:
- Settings window loads initial state via `get_ui_state`.
- Updates go through `update_config`, which normalizes, persists, and emits `ui-state`.
- Toast window listens to `toast-message` events and animates itself.

## Invariants
- No polling: clipboard updates are event-driven (`WM_CLIPBOARDUPDATE`).
- No infinite loop on self-write: self-write cooldown and hash gate prevent re-entry.
- Allowlist default behavior: `only_allowlisted` true; owner exe must match allowlist.
- JS path only rewrites if parse validates; otherwise fallback is the normalized text.
- No rewrite if output matches normalized input; no toast for no-op sanitize.
- Tray-only on launch: settings window hidden until explicitly opened.

## Where to look in code

Core orchestration:
- `src-tauri/src/lib/mod.rs` (app setup, state wiring, shortcut registration)

Clipboard pipeline:
- `src-tauri/src/lib/win/clipboard_listener/window.rs` (Win32 listener)
- `src-tauri/src/lib/win/clipboard_listener/handler.rs` (event handling pipeline)
- `src-tauri/src/lib/win/clipboard_listener/clipboard.rs` (read/write text)
- `src-tauri/src/lib/win/process.rs` (owner/foreground exe resolution)

Sanitizer:
- `src-tauri/src/lib/sanitize/mod.rs` (entry point, JS vs text routing)
- `src-tauri/src/lib/sanitize/js/mod.rs` (JS heuristic + reflow)
- `src-tauri/src/lib/sanitize/js/parse.rs` (JS parse validation)
- `src-tauri/src/lib/sanitize/clean.rs` (cleanup pass)
- `src-tauri/src/lib/sanitize/unwarp.rs` (unwrap logic)

State + config:
- `src-tauri/src/lib/state/io.rs` (load/save config.json)
- `src-tauri/src/lib/state/config.rs` (config schema + normalize)
- `src-tauri/src/lib/state/runtime.rs` (SharedState + UI emit)
- `src-tauri/src/lib/state/models.rs` (UiState + LogEntry)

Tray + UI:
- `src-tauri/src/lib/tray/mod.rs` (tray menu + settings window)
- `src-tauri/src/lib/toast/mod.rs` (toast window + show)
- `app/src/config/settings.ts` (settings UI wiring)
- `app/src/ui/toast.ts` (toast UI wiring)
