* The join decision still has a **hard gate**: it will not join unless it sees “wrap evidence” from **(a)** trailing padding spaces or **(b)** “near inferred wrap width” — and if neither is present, it only joins true identifier splits. That’s literally encoded in @src-tauri/src/lib/sanitize/boundaries.rs (`if !wrap_evidence && !identifier_split { join: false }`).
* Your wrap profile inference is also brittle in the real world: it only samples “long lines” at `raw_len_chars >= 40` and requires 2+ samples. If your terminal pane is narrow or the wrapped segments are <40 chars (which *can happen* with indentation + narrow splits), `wrap_width` becomes `None`, so wrap evidence collapses to “trailing_ws_count > 0”, which you don’t have. See @src-tauri/src/lib/sanitize/profile.rs.

So your current classifier is *behaving exactly as designed* — and that design can’t win if the clipboard text is unpadded and wrap width isn’t confidently inferred.

## The fix that actually moves the needle

### Stop treating “padding/width” as the required evidence.

Instead, treat **syntactic continuation context** as the robust evidence that survives real terminals.

In other words: **unwrap only when the newline is inside an “unfinished expression context”**, plus a small set of safe, high-confidence keyword cases (`return`, `throw`) that are notorious for becoming broken by wraps (and are also the ones you explicitly called out: `return\n  fallback`).

This is not “language parsing”; it’s a deterministic, lightweight **delimiter + string state scanner** that lets you know whether a newline happens while `(` / `[` are still open (or while you’re inside a normal quote string). That signal exists even when padding is stripped.

### Why this works for your concrete failures

* `pick(screen,\n  ["width"...`
  At the newline, you are inside an unclosed `(` from `pick(` → **join** (with a single space) even if there’s no padding and no wrap-width inference.
* `return\n  fallback`
  Newline after `return` is almost never intentional in JS (ASI hazard), and it’s a huge “wrap broke this” signal → **join** even though delimiter depth may be 0.

### Why this does *not* collapse real multi-line blocks

Because we add **hard structural guards** that are stable:

* never join after `{` when the next line is more indented (block opening)
* never join before `}` / `]` / `)` closers
* never join at depth==0 when the next line looks like a new statement or an object property line (e.g. `userAgent:`) at the same indentation level

This preserves the block/statement newlines while still repairing wraps **inside** expressions.

---

# Concrete refactor plan (single direction)

## 1) Add a context scanner module

Create @src-tauri/src/lib/sanitize/context.rs

It should scan the text in order (line by line) and produce `BoundaryContext` for each newline boundary:

* `paren_depth` for `(` `)`
* `bracket_depth` for `[` `]`
* `brace_depth` for `{` `}` (mostly for guards)
* string state:

  * in single quote `'...'`
  * in double quote `"..."`
  * in backtick `` `...` `` (optional; you can treat backticks conservatively)
* comment state:

  * `//` ignore rest of line
  * `/* ... */` across lines (optional but easy)

Important: delimiters inside strings/comments must **not** affect depth.

This gives you a robust signal: **“is this newline inside an unfinished expression?”**

## 2) Replace the “wrap_evidence must exist” gate with a score + hard guards

Rewrite @src-tauri/src/lib/sanitize/boundaries.rs so joining is decided by:

### Hard NO-join guards (run first)

* blank/empty line on either side → no join
* block open guard (you already have one): `prev ends_with '{' && next.indent > prev.indent` → no join
* next is a close-only line: next trimmed starts with `}` or `]` or `)` → no join
* statement end at depth 0: `prev ends_with ';'` and next indent is not greater → no join
* object-property guard at depth 0:

  * if `prev ends_with ','`
  * and `next` starts with a property key (`foo:` or `"foo":` or `'foo':`)
  * and `next.indent == prev.indent`
    → no join (prevents flattening object literal property lists)

### High-confidence YES-join (before scoring)

* strict identifier split (your existing `.jo\nin` case) → join, no space
* **keyword break**:

  * if `prev` ends with `return` or `throw`
  * and next starts with an “expression-ish” token (`[({"'` + identifier + digit)
    → join, **insert one space**
    (This directly fixes `return\n  fallback`)

### Scored join for everything else

Compute a score from signals like:

* `+3` if boundary is inside `paren_depth>0` or `bracket_depth>0`
* `+2` if next starts with continuation punctuation (`[ ( . , ) ] }`) or prev ends with continuation punctuation (`+ - * / % = ? : & | , .`)
* `+2` if indentation looks like a wrap continuation:

  * `next.indent < prev.indent` (and next isn’t a closing brace line)
  * OR next.indent is “weird” relative to the snippet’s common indent (optional)
* `+1` if prev line length is in the top tail (p90-ish) **or** near a computed wrap width (but *never required*)

Join if score >= threshold (start with 4; tune with tests + debug).

**Key change:** width/padding becomes a *bonus*, not a requirement.

## 3) Fix wrap profile to work on unpadded and narrow terminals

Update @src-tauri/src/lib/sanitize/profile.rs:

* remove the hard `raw_len_chars >= 40` cutoff
* compute distribution stats over **trimmed_end lengths**:

  * `max_len`
  * `p90_len` (or p85)
* wrap width guess should be derived from the **top tail**:

  * consider lengths >= `max_len - 8` (or dynamic)
  * if you have 2+ such lines, use their median as wrap width
  * otherwise wrap width can be `None` and the system still works (because we no longer require it)

## 4) Wire context into unwarp

Update @src-tauri/src/lib/sanitize/unwarp.rs to precompute contexts once and pass the appropriate `BoundaryContext` into `decide_join(...)`.

(If `decide_join`’s signature needs to change, that’s fine — it’s the owning layer.)

## 5) Replace tests with **unpadded** real-world reproductions

Your tests currently succeed because they are artificially padded and/or width-anchored. That’s the mismatch.

Update @src-tauri/src/lib/sanitize/tests.rs to include at least:

* an **unpadded** `pick(screen,\n  ["width"...` case in a realistic object-literal context
* an **unpadded** `return\n  fallback` case (no padding, no anchor lines)
* a guard test that ensures object property lines do not join:

  * `href: ...,` followed by `userAgent: ...` stays two lines

These tests should pass **without padding and without wrap_width**.

## 6) Add one real-world debug hook (so we stop guessing)

Because “tests pass but reality doesn’t” is fundamentally an *observability* gap:

* Store `last_raw_clipboard_text` (bounded, e.g. first 8–16 KB) and `last_join_debug` (bounded) in runtime state after sanitation.
* Expose it via an existing state event or a command.
* This lets you paste the exact real clipboard into a test and permanently end this loop.

This is not a band-aid; it’s the minimum defense against future heuristic drift.

---

## The punchline

Right now the sanitizer is “correct” per its current contract — it’s just the **wrong contract** for real clipboard data. The fix is to change the contract so that **unfinished-expression context + safe guards** is the primary signal, and padding/width is only a helpful bonus when present.

If you run Prompt 1 + 2, your two concrete failures (`return\n  fallback` and `pick(screen,\n[`) should start joining even on unpadded clipboard text, while the object/statement newlines stay intact.
