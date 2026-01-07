## Files I actually inspected (from the provided bundles)

* @src-tauri/tauri.conf.json
* @src-tauri/src/lib/mod.rs
* @src-tauri/src/lib/tray/mod.rs
* @src-tauri/src/lib/toast/mod.rs
* @src-tauri/src/lib/win/clipboard_listener/window.rs
* @src-tauri/src/lib/win/clipboard_listener/handler.rs
* @src-tauri/src/lib/win/clipboard_listener/clipboard.rs
* @src-tauri/src/lib/win/process.rs
* @src-tauri/src/lib/sanitize/mod.rs
* @src-tauri/src/lib/sanitize/normalize.rs
* @src-tauri/src/lib/sanitize/wrap.rs
* @src-tauri/src/lib/sanitize/tests.rs
* @src-tauri/src/lib/state/config.rs + @src-tauri/src/lib/state/runtime.rs
* @src-tauri/capabilities/default.json
* @app/index.html
* @app/src/config/settings.ts
* @app/src/ui/toast.ts
* @app/css/app.css + @app/css/toast.css

(And I read the bundle index first: blankstop_bundles_index.md.)

---

# 1) Implementation vs design intent

### Tray-only app (no main window on launch)

**Matches intent, mostly.**

* The main window is declared `visible: false` in @src-tauri/tauri.conf.json, so it shouldn’t appear on launch.
* In setup, it also explicitly hides the main window if it exists (@src-tauri/src/lib/mod.rs).

Potential improvement (not required, but relevant): the “main” window exists and is just hidden; that’s fine for “tray-only”, but if you ever see a “flash” on startup, you’ll want to ensure the window never shows even briefly (usually a config + window-creation ordering issue).

---

### Win32 clipboard listener (event-driven)

**Matches intent, but there’s a serious robustness bug.**

* You’re using `AddClipboardFormatListener` + `WM_CLIPBOARDUPDATE` on a message-only window and a Win32 message loop thread. That’s the correct event-driven approach (@src-tauri/src/lib/win/clipboard_listener/window.rs + @src-tauri/src/lib/win/clipboard_listener/mod.rs).

**However:** `read_clipboard_text()` can return early without `CloseClipboard()`, leaving the clipboard locked (details in section 2). That will absolutely make the app “not reliable”.

---

### Allowlist filtering by source exe

**Matches intent.**

* Source detection uses `GetClipboardOwner()` and falls back to `GetForegroundWindow()` if needed, then `GetWindowThreadProcessId` → `QueryFullProcessImageNameW` → basename lowercased (@src-tauri/src/lib/win/process.rs).
* Filtering is applied in the clipboard handler when `only_allowlisted` is true (@src-tauri/src/lib/win/clipboard_listener/handler.rs), with allowlist normalization in @src-tauri/src/lib/state/config.rs.

One practical note: if Windows Terminal ever reports `applicationframehost.exe` as the clipboard owner in your setup, it won’t match the default allowlist. Right now the UI logs can help confirm what exe you’re actually seeing (the handler writes `last_source_exe` and log entries).

---

### Sanitizer behavior (soft-wrap fixes, no over-flattening)

**Intent is present, but current heuristics can both under-fix and over-flatten.**

* You normalize line endings, trim trailing whitespace per line, and trim blank edges (@src-tauri/src/lib/sanitize/mod.rs + @src-tauri/src/lib/sanitize/normalize.rs).
* You attempt to unwrap:

  * “Rule A” for 2–4 lines (join into one line) via `should_rule_a()` (@src-tauri/src/lib/sanitize/wrap.rs).
  * Multi-line wrap join based on inferred wrap width via `infer_wrap_width()` + `join_wrapped_multiline()` (@src-tauri/src/lib/sanitize/wrap.rs).

But:

* `should_rule_a()` is too permissive and will flatten real multi-line snippets (especially small code blocks).
* `infer_wrap_width()` refuses to infer in exactly the scenario you *care* about: **a big snippet where only one long line got wrapped** (details below).

---

### Toast behavior (glass, bottom-right, short duration, non-sticky)

**The “intended shape” is there, but the plumbing is fragile and likely the direct cause of your toast bug.**

* You create a transparent, always-on-top, borderless “toast” window, positioned bottom-right, and attempt to hide it shortly after showing (@src-tauri/src/lib/toast/mod.rs).
* The CSS is glassy (backdrop blur etc) and the toast is hidden unless `.show` is applied (@app/css/toast.css + @app/css/app.css).
* But the toast window URL is `index.html?toast=1` and toast-mode is determined only via query params in the frontend (@src-tauri/src/lib/toast/mod.rs + @app/src/config/settings.ts). This is a fragile coupling point.

---

# 2) Debug the two issues

## Issue A — Toast shows blank content and doesn’t auto-hide

### What the code is doing now

* Rust creates the toast window at startup loading:
  `WebviewUrl::App("index.html?toast=1".into())` (@src-tauri/src/lib/toast/mod.rs).
* Frontend decides “toast mode” only by checking `window.location.search` for `toast=1` and then conditionally importing the toast listener code (@app/src/config/settings.ts).
* Toast UI is **display:none** unless the body has `.toast-mode` and the toast root has `.show` (@app/css/app.css + @app/css/toast.css).
* Rust `show_toast()` tries **two mechanisms**:

  1. `window.emit("toast-message", message)` (expects frontend listener)
  2. `window.eval(...)` that directly sets text + adds `.show` and removes `.show` after 1200ms
     Then `window.show()`, then schedules a `window.hide()` after 1400ms (@src-tauri/src/lib/toast/mod.rs).

### Likely failure modes (ranked)

1. **Toast window is not actually in toast mode** → `#toast-root` stays `display:none` → you see an “empty/blank” window.
   This can happen if the query string isn’t propagated the way you expect in the built app, or if the page fails to load using that URL form. This coupling is:

   * URL: @src-tauri/src/lib/toast/mod.rs
   * Mode detection: @app/src/config/settings.ts
   * Hide/display rules: @app/css/app.css

2. **The toast window isn’t included in capabilities** (windows list contains only `"main"`) → frontend Tauri APIs / event listening may be denied or behave differently in that window.
   That can break `listen("toast-message")`, and depending on how Tauri boots the JS bridge, could even interfere with the “toast-mode class” script running.
   See @src-tauri/capabilities/default.json.

3. **Race: you emit/eval before the toast webview is ready**, and you ignore errors (`let _ = ...`).
   If both `emit` and `eval` happen too early and fail, `.show` never gets applied → toast looks blank.
   (Also: since you don’t have a “toast-ready handshake”, this can be intermittent.)
   See @src-tauri/src/lib/toast/mod.rs.

4. **The window never hides because the hide call isn’t executing reliably from this call path** (clipboard listener thread → tauri async runtime).
   This is less likely (Tauri *should* handle it), but if it’s happening, the fix is to move show/hide onto the main thread (or let the toast window JS call `hide()` itself).

### Concrete fix direction (architecture-first)

To make this robust (and stop accumulating “toast weirdness” debt), pick a single, deterministic mechanism:

* **Stop relying on `index.html?toast=1`** as the only mode signal.
* **Give the toast window an explicit mode flag via an initialization script** (or a dedicated toast HTML entrypoint).
* **Add `"toast"` to capabilities windows list**, or create a minimal capability for toast.
* **Use one show path**: either:

  * Rust shows/hides window, frontend only renders; **or**
  * Frontend shows/hides itself after receiving an event (Rust just emits).

Given your bundle constraints and minimal change risk, the most direct is:

* Keep `index.html`, but set `window.__blankstopToast = true` via window builder init script.
* Update @app/src/config/settings.ts to treat that as toast mode.
* Add `"toast"` to @src-tauri/capabilities/default.json.
* Remove redundant `eval_toast` OR make it the only mechanism (but not both).

---

## Issue B — Sanitizer doesn’t remove terminal soft-wrap newlines; pasted content gets truncated/merged

This one is explainable directly from the current heuristics.

### 1) Over-flattening: Rule A will flatten short code blocks

`should_rule_a()` returns true for **any** 2–4 line text where a continuation line starts with whitespace (even if it’s *real indentation*), because of:

```rust
if is_whitespace_continuation(line) {
  return true;
}
```

(@src-tauri/src/lib/sanitize/wrap.rs)

So a real code block like:

```
if (foo) {
  bar();
}
```

…gets turned into a **single line**, which is exactly the “merged mid-block” failure mode.

This contradicts your “no over-flattening” intent.

### 2) Under-unwrapping: wrap width inference bails when there’s only one long line in a big snippet

`infer_wrap_width()` only returns a wrap width in the `total == 1` case if the entire snippet has `lines.len() <= 3`:

```rust
} else if lines.len() <= 3 {
  Some(mode)
} else {
  None
}
```

(@src-tauri/src/lib/sanitize/wrap.rs)

So if you copy a big multi-line snippet where **only one long line got terminal-wrapped** (very common), you can have `total == 1` long line but `lines.len() = 30`, and the function returns `None`.
That means **no unwrapping happens at all**, and your wrapped line stays broken → “truncated” behavior when pasted.

### 3) Mis-inference risk: mode-of-long-lines can pick the wrong “wrap width”

The current “mode” heuristic can accidentally pick a smaller, common line length from normal code/JSON lines instead of the actual terminal wrap column. Then `join_wrapped_multiline()` may join boundaries it shouldn’t, causing “merged” behavior.

---

## Bonus: a critical clipboard bug that can make everything look flaky

In @src-tauri/src/lib/win/clipboard_listener/clipboard.rs:

```rust
let handle = unsafe { GetClipboardData(CF_UNICODETEXT.0 as u32) }.ok()?;
```

If that `.ok()?` returns `None` (e.g., non-text clipboard), you return early **without calling `CloseClipboard()`**. That can lock the clipboard and cause cascading weirdness. This should be fixed first.

---

# 3) What the “correct” behavior should be (explicit contract)

| Situation                                                 | Input                            | Expected output                                           |
| --------------------------------------------------------- | -------------------------------- | --------------------------------------------------------- |
| Copy a wrapped commit subject from terminal               | `"...restore\n  requirement\n"`  | Join to one line (single space), keep words intact        |
| Copy a short multi-line code block                        | `if (...) {\n  x();\n}\n`        | **Preserve newlines** (only trim trailing whitespace)     |
| Copy a large snippet where only one long line was wrapped | many lines, one `...jo\nin(...)` | Only unwrap the wrapped boundary; keep all other newlines |

Invariants to enforce in code:

* Never “collapse to one line” purely because a continuation line begins with whitespace.
* In multi-line content, only join a newline when you have strong evidence it’s a wrap artifact (token split, wrap-column evidence, continuation signal).
* Clipboard must never be left open (always close on all paths).

---
