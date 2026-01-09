# Troubleshooting Runbook

Symptom → likely cause → where to inspect.

## Toast appears but clipboard unchanged
Likely causes:
- Sanitizer produced no material changes (output equals normalized input).
- JS path parse failed; sanitizer returned normalized input.

Where to inspect:
- `src-tauri/src/lib/win/clipboard_listener/handler.rs` (rewrite gate).
- `src-tauri/src/lib/sanitize/mod.rs` (JS vs fallback decision).
- `src-tauri/src/lib/sanitize/js/parse.rs` (oracle parse).

## Clipboard unchanged because not allowlisted
Likely causes:
- `only_allowlisted` is on and clipboard owner exe is not in the allowlist.
- Clipboard owner exe could not be resolved.

Where to inspect:
- Settings UI: allowlist field + only_allowlisted toggle.
- Tray menu: enabled/paused state.
- `src-tauri/src/lib/win/process.rs` (exe resolution).
- `src-tauri/src/lib/state/config.rs` (allowlist normalization).

## Clipboard unchanged because JS parse failed
Likely causes:
- JS-like detection passed, reflow ran, but parse failed.
- Sanitizer returned normalized input, so no rewrite occurred.

Where to inspect:
- `src-tauri/src/lib/sanitize/js/reflow.rs` (reflow rules).
- `src-tauri/src/lib/sanitize/js/parse.rs` (module/script parse).
- `src-tauri/src/lib/sanitize/mod.rs` (oracle gate).

## App feels disabled / pause state confusion
Likely causes:
- Enabled toggle is off (tray or settings).
- Pause is active from tray menu.
- Global shortcut toggled enabled without user noticing.

Where to inspect:
- Tray menu (Enabled checkmark / Pause / Resume).
- Settings window (Enabled toggle).
- `src-tauri/src/lib/tray/mod.rs` (toggle + pause logic).

## Where to look (quick map)
- Tray state + toggles: `src-tauri/src/lib/tray/mod.rs`
- Toast pipeline: `src-tauri/src/lib/toast/mod.rs`
- Clipboard handler: `src-tauri/src/lib/win/clipboard_listener/handler.rs`
- Process resolution: `src-tauri/src/lib/win/process.rs`
- Sanitizer pipeline: `src-tauri/src/lib/sanitize/`

Notes:
- The settings UI currently surfaces last source exe and last action time only.
- Debug clipboard fields exist in `UiState` but are not rendered in the UI.
