You’re not crazy — the sanitizer is *very* close, but there’s one structural mistake in the pipeline that explains **exactly** the symptom you’re seeing (“toast says unwrapped 2 lines, but lots of soft-wrap breaks remain”), and then a couple smaller heuristic gaps.

## First: the toast “box inside a box” is **not Rust**

Rust is only creating a transparent window and positioning it (`transparent(true)`, `decorations(false)`, etc.) in @src-tauri/src/lib/toast/mod.rs.

The “outer box” you’re noticing is coming from **CSS/background on the root element**, while the inner box is `#toast-root`’s own glass panel. In your CSS, `:root` sets a background color (`background-color: var(--bg);`) in @app/css/app.css, and toast-mode only sets `body` to transparent. Result: the `html` element can still render a background, so you see a second layer. That’s a frontend fix later (e.g., make `html` background transparent in toast mode). For now: ✅ your mental model is right — the “look” is TS/CSS, Rust is only window flags + lifecycle.

---

# The sanitizer issue: why it’s failing right now

### Root cause: wrap-width estimation is being poisoned by an earlier join pass

In @src-tauri/src/lib/sanitize/mod.rs you do:

1. `join_identifier_splits(&working, …)`
2. then `estimate_wrap_width(&working)`
3. then `join_wrapped_multiline(&working, wrap_width, …)`

That ordering is the killer.

If the identifier-split pass joins even **one** wrap boundary (e.g. `jo\nin`), it produces a **longer line than any original “visual row”**. Then `estimate_wrap_width` (in @src-tauri/src/lib/sanitize/heuristics.rs) looks at the longest lines and effectively treats *that new, longer line* as the “wrap width”. After that, your multiline join pass refuses to join the remaining boundaries because their previous-line lengths are no longer “close enough” to the (inflated) wrap width.

That perfectly matches your report:

* it “unwrapped 2 lines” (identifier-split joins)
* but leaves many soft wraps like `pick(screen,\n[` and `return\n fallback` because the multiline wrap join never triggers.

### Secondary issue: identifier-split join is too broad (and also not used the way you think)

`join_identifier_splits` uses `should_join_identifier`, which currently only checks “prev ends with ident char and next starts with ident char” (plus some length gating), but it does **not** require that this boundary is actually a *split identifier* (like `.jo\nin`). That’s risky.

You already have `is_identifier_split(prev, next)` (also in @src-tauri/src/lib/sanitize/heuristics.rs), but `join_identifier_splits` doesn’t use it. So it’s easy for this pass to “join things that are not true token splits”, which is exactly the class of bug that leads to “merged/truncated mid-block” behavior.

### Third issue: continuation whitespace trimming is too conservative

In @src-tauri/src/lib/sanitize/wrap.rs, `join_wrapped_multiline` only trims leading whitespace on continuation lines if `is_wrap_indent(line)` returns true, which currently means **two spaces or a tab**.

But your real-world breaks include:

* next line starts with **one** space (very common), or
* next line starts with `[` or other punctuation (no whitespace), which is fine,
  but the important part is: when we *do* decide to join, we should basically always treat leading whitespace on the continuation as wrap artifact and trim it.

---

# What we actually want (behavior + invariants)

## Goal in user-visible behavior

When copying from terminal apps (especially Windows Terminal running Ubuntu/WSL), Blankstop should:

* remove **soft-wrap inserted newlines** (visual wraps)
* remove **padding/trailing whitespace**
* keep **real, intentional newlines** (multi-line blocks / separate statements)
* never “flatten everything” unless it’s clearly a 2–4 line wrap case.

## Behavior table

| Situation                           | Input                       | Expected behavior                                         |
| ----------------------------------- | --------------------------- | --------------------------------------------------------- |
| Long one-liner wraps mid-expression | `pick(screen,\n["width"...` | Join into `pick(screen, ["width"...` (newline removed)    |
| ASI hazard in JS (`return` wraps)   | `return\n  fallback`        | Join into `return fallback` (newline removed + one space) |
| Real multi-line block               | `if (foo) {\n  bar();\n}`   | Preserve newlines; only trim trailing whitespace          |

## Invariants

1. **Wrap-width must be inferred from the original visual rows**, not after any joining (no self-poisoning).
2. Never collapse into one line unless the “Rule A” gate is satisfied (your current design intent).
3. In multiline mode, only remove a newline if we have evidence it’s a wrap boundary:

   * prev line is “full” (near wrap width), or
   * we have a strong token-split marker (like `.jo\nin`)
4. When joining a wrap boundary, treat leading whitespace on the continuation as an artifact (trim it) and insert at most **one** space when needed.

---

# Two designs that solve this safely (pick one)

## Design A — Minimal change, fits current structure

Keep your current modules and flow, but change two things:

1. **Compute wrap width before any joining**

* In @src-tauri/src/lib/sanitize/mod.rs:

  * compute `wrap_width = estimate_wrap_width(&lines)` using the original trimmed lines.
  * then do joins; do *not* recompute wrap width after joining.

2. **Make identifier-split join strict**

* In @src-tauri/src/lib/sanitize/wrap.rs / heuristics:

  * `join_identifier_splits` should only join when `is_identifier_split(prev_trim, next_trim)` is true (or an equally strict predicate).
  * Otherwise it should not join at all.

3. **When joining in multiline, always `trim_start()` the continuation line**

* In @src-tauri/src/lib/sanitize/wrap.rs:

  * remove the “two spaces or tab” gate (`is_wrap_indent`)
  * just trim continuation start on join

Why this is “safe/predictable”:

* You’re only joining where the original “full row” evidence exists (wrap width) or where token split evidence exists.
* You’re not inventing language parsing.
* You fix the root pipeline bug (ADR‑007 compliant).

## Design B — Boundary classifier (bigger change, more robust long-term)

Introduce a new module like:

* `sanitize/boundaries.rs` that computes a per-newline “join score” from multiple signals:

  * full-row evidence (near wrap width)
  * token split evidence
  * continuation punctuation (like `[` after `,`)
  * indentation anomalies
  * “code-block” likelihood

Then the unwrap pass joins only when score ≥ threshold and records “reasons” (optional) for debugging.

Pros:

* Easier to extend without turning into special-case soup.
* Debuggable: you can log why a join did/didn’t happen.

Cons:

* More code + more moving parts now.

**My recommendation:** do **Design A now** (fast, fixes the core bug), then evolve toward Design B only if you still see edge cases.

---

# The simplest “this will probably fix your exact sample” change

**Move wrap width estimation earlier.** That alone will usually cause `join_wrapped_multiline` to start joining the remaining wrap boundaries, including cases where the next line starts with whitespace or `[` (because `join_wrapped_multiline` doesn’t care what the next char is — it was just never being activated due to a bad width).