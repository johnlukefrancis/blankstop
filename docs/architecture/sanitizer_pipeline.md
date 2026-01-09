# Sanitizer Pipeline

This document defines the sanitizer pipeline and its correctness gates.
It describes the actual end-state behavior (no speculation).

## Overview
Entry point: `sanitize_text` in `src-tauri/src/lib/sanitize/mod.rs`.
The pipeline is:
1) Pre-clean stage (normalize + strip).
1.5) Large input guard (skip JS + unwrap when oversized).
2) JS-like detection gate (conservative).
3) JS reflow + parse validation (oracle).
4) Fallback text-mode sanitizer (non-JS).
5) Caller-level rewrite gate (no-op if output matches normalized input).

## Pre-clean stage (normalize + strip)
Owner: `src-tauri/src/lib/sanitize/clean.rs`

What it does:
- Line endings: normalize all `\r\n` and `\r` to `\n`.
- U+2028 / U+2029: replace with `\n` (JS line/paragraph separators).
- Invisible format/control chars: remove, except `\n`, `\r`, and `\t`.
- Leading bullet prefix: remove a single leading "• " on the first non-empty line
  when the line looks code-ish (to undo clipboard bullets from some apps).

Why:
- Normalized line endings make downstream heuristics stable across sources.
- U+2028/U+2029 are valid in JS literals but often break parsing or display; treat
  them as line breaks to align with pasted text expectations.
- Invisible format characters (soft hyphens, bidi controls, etc.) can corrupt
  parsing and text comparison; they are stripped up front.
- The bullet strip is a targeted fix for copy-from-bullet UI when the payload is
  actually code, not a list item.

Outputs:
- `clean.text`: normalized + cleaned + bullet-stripped.
- `clean.normalized`: line-ending normalized only (used for JS fallback).

## Large input guard (skip heavy paths)
Owner: `src-tauri/src/lib/sanitize/limits.rs`

Behavior:
- If cleaned text exceeds `MAX_CLIPBOARD_CHARS` (currently 250,000 chars),
  the sanitizer skips JS detection/reflow/parse and skips text-mode unwrapping.
- Only the pre-clean stage + per-line trailing whitespace trimming runs.
- `js_validated` remains false for this path.

## JS-like detection (conservative gate)
Owner: `src-tauri/src/lib/sanitize/js/mod.rs`

A string is JS-like only if ALL of the following hold:
- Non-empty after trimming.
- Contains at least one newline.
- Has a JS "signal":
  - `=>`, OR
  - `;`, OR
  - a JS keyword boundary (const/let/var/function/return/class/import/export/...)
  - OR structural tokens plus a soft-wrap seam.
- Has a soft-wrap seam (identifier split or continuation punctuation across
  adjacent lines).

Why conservative:
- The JS reflow path is more aggressive; false positives would rewrite
  non-code prose. The gate is intentionally narrow to avoid that.

## JS reflow rules
Owner: `src-tauri/src/lib/sanitize/js/reflow.rs`

Behavior summary:
- Operates in a small state machine (normal / quotes / template / comments).
- Joins lines in normal mode when `should_join` allows it:
  - Avoids joining after terminators (`;`, `{`, `}`, `)`, `]`).
  - Avoids joining before closing tokens (`}`, `]`, `)`).
  - Avoids joining if the next line looks like an object property start.
  - Avoids joining into line comments.
- When joining, trims trailing inline whitespace and inserts a single space
  unless the join is an identifier split (to keep `foo` + `Bar` -> `fooBar`).
- Blank lines are preserved.

String literal newline handling (single/double quotes):
- Newlines inside string literals are removed.
- Indentation following a newline inside a string is dropped; if a single
  leading space is present (and not followed by more whitespace), that space
  is preserved to keep intentional spacing.
- Escapes (`\\`) are preserved and passed through.

Template literals:
- Template literal content is preserved verbatim (including newlines).

## JS validation (parse-only oracle)
Owner: `src-tauri/src/lib/sanitize/js/parse.rs`

Contract:
- The reflowed JS is accepted only if it parses as a module or script.
- Parsing is the oracle; no other signal can force a rewrite.
- If parse fails, the sanitizer returns the *normalized input* (line endings
  only), which means the clipboard should remain unchanged.

Why:
- This prevents corrupting JS when the heuristics misfire.
- A parse failure is treated as a hard stop, not a "best effort" rewrite.

## Fallback text-mode sanitizer
Owner: `src-tauri/src/lib/sanitize/mod.rs` + `lines.rs` + `profile.rs` + `boundaries.rs`

When used:
- Used when text is not JS-like.
- Also used when JS-like detection is false (regardless of input).

What it does:
- Builds line metadata (leading/trailing whitespace, raw length).
- Estimates wrap profile for the block.
- Unwraps soft-wrapped lines while respecting block structure.

Key invariants (do not flatten real blocks):
- Do not join across blank lines.
- Do not join when the next line starts with a closing token (`}`, `]`, `)`).
- Do not join after a statement terminator when indentation does not increase.
- Do not join into property starts (object literals are preserved).
- In block comments or obvious code blocks, joining is conservative.

## Debugging: "why did it skip?"

Important: the sanitizer can run and still produce *no clipboard rewrite* if the
output equals the normalized input. Common reasons:

1) JS-like path rejected by the oracle
- Reflow happened, but parse failed.
- Output becomes the normalized input, so the caller treats it as no-op.

2) Input not JS-like
- The gate is conservative: missing signals or no soft-wrap seam.
- The fallback path may decide no changes are safe.

3) No material changes after cleanup
- Line endings normalized but text is otherwise identical.
- Cleaned output equals normalized input.

Inspecting debug state:
- `UiState` includes `debug_last_clipboard` and `debug_last_summary` (truncated to
  4096 chars) set by the clipboard handler.
- `debug_last_clipboard` is dev-only; in release builds it is never captured and
  is always `None` in `UiState`.
- These fields are not rendered in the settings UI; inspect via devtools by
  invoking `get_ui_state` and applying `JSON.stringify` to reveal escapes.

Escaped view tips:
- Use JSON stringification when inspecting the debug clipboard to reveal
  `\n`, `\t`, and invisible characters.
- If `debug_last_clipboard` shows normalized newlines but output is unchanged,
  the rewrite gate likely short-circuited.

## Owning modules
- `src-tauri/src/lib/sanitize/mod.rs` (pipeline entry + JS/fallback switch)
- `src-tauri/src/lib/sanitize/clean.rs` (pre-clean + bullet strip)
- `src-tauri/src/lib/sanitize/js/mod.rs` (JS gate + oracle call)
- `src-tauri/src/lib/sanitize/js/reflow.rs` (JS join rules)
- `src-tauri/src/lib/sanitize/js/parse.rs` (parse-only validation)
- `src-tauri/src/lib/sanitize/boundaries.rs` (text-mode join gates)
- `src-tauri/src/lib/sanitize/context.rs` (paren/string/comment context)
- `src-tauri/src/lib/sanitize/lines.rs` (line metadata)
