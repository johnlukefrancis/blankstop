## Why it merged `screen: ...` with `trianglerain_globals: {` (root cause)

In your current owner, @src-tauri/src/lib/sanitize/boundaries.rs:

1. The **property-start guard** only fires when indentation matches:

* Current code: it blocks only if
  `next.leading_ws_count == prev.leading_ws_count && looks_like_property_start(next_trim_start)`.

So if the next property is **dedented** (common when you exit a nested object and start a new top-level property), the guard does **not** trigger.

2. The **score is biased toward joining almost any indented code line**:

These three lines are the killer combination:

* `if next.leading_ws_count > 0 { score += 1; }`
* `if next.leading_ws_count < prev.leading_ws_count { score += 1; }`
* `if continuation_punct(prev_trim, next_trim_start) { score += 2; }`
  (and `continuation_punct` treats a trailing `,` as “continuation”)

At a boundary like:

* prev ends with `,` (very common in object literals) → `+2`
* next is indented at all → `+1`
* next is dedented relative to prev → `+1`

That’s **score = 4**, which meets your `score < 4 => no join` threshold, so it joins.
That exactly explains merges like:

```
screen: safe(() => pick(screen,
trianglerain_globals: {
```

even when there’s **no paren/bracket continuation context** at that boundary.

So: the corruption is not “mysterious”; it’s a deterministic consequence of the current scoring weights + the indent-gated property guard.

---

# The next step (single plan): make property/statement boundaries *hard*, and stop using indentation as join evidence

### Goal in user behavior

* Keep fixing true soft-wraps (inside `(` / `[` and the `return\n  expr` ASI hazard)
* Stop *ever* joining across:

  * object property boundaries (at any indent)
  * statement boundaries (depth 0, no continuation context)

### Behavior table

| Situation                    | Input                                   | Expected                          |
| ---------------------------- | --------------------------------------- | --------------------------------- |
| Soft wrap inside call args   | `pick(screen,\n  ["width"...])`         | join newline                      |
| ASI hazard                   | `return\n  fallback;`                   | join newline (`return fallback;`) |
| Property boundary (dedented) | `screen: ...,\ntrianglerain_globals: {` | **do not join**                   |

### Invariants

1. **Indentation is not join evidence**. (Code is always indented; using it as a join signal guarantees false positives.)
2. **Property-start lines are a hard boundary**: if next line begins with a property key (`foo:` / `"foo":` / `'foo':`), do not join into it (unless we’re inside a string/block-comment continuation, where property syntax is just characters).
3. Outside `(` / `[` (and outside string/comment continuation), joining requires **strong evidence** (identifier split, keyword ASI fix, or wrap-width line-fill for non-code text).

---

## Concrete changes to make in the owner (@src-tauri/src/lib/sanitize/boundaries.rs)

### 1) Make the property-start guard unconditional (remove indent equality)

Change:

* from:

  * “no join if property start AND indent matches”
* to:

  * “no join if property start, regardless of indent”
* but **do not apply** this guard when `ctx.in_string` or `ctx.in_block_comment` is true (otherwise you can’t repair a wrapped string/comment that happens to contain `foo:` text).

So effectively:

* If `!ctx.in_string && !ctx.in_block_comment && looks_like_property_start(next_trim_start)` → `join = false`

This alone stops the `screen:` → `trianglerain_globals:` merge class.

### 2) Remove the two indentation-based score boosts (they’re fundamentally unsound)

Delete these from scoring:

* `if next.leading_ws_count > 0 { score += 1; }`
* `if next.leading_ws_count < prev.leading_ws_count { score += 1; }`

Those are what let “comma + any indent + any dedent” hit your threshold.

### 3) Remove the brace-depth indent bonus

This one is also unsafe for object-literal structure:

* `if ctx.brace_depth > 0 && next.leading_ws_count > prev.leading_ws_count { score += 1; }`

Brace depth is *where property lists live*. Adding positive score here makes property merges more likely, not less.

### 4) Treat `,` as weak punctuation unless we’re in paren/bracket context

Right now comma gives `+2` via `continuation_punct`. But commas are **normal line terminators** in object and array formatting.

You can keep `continuation_punct` but split it conceptually:

* “strong continuation punctuation” (`.` `(` `[` operators) can remain evidence
* “weak continuation punctuation” (`,` `:`) should **not** contribute at depth 0

The smallest change is:

* If `ctx.paren_depth == 0 && ctx.bracket_depth == 0` and the only reason `continuation_punct` is true is `prev_last == ','`, then don’t add the `+2`.

(Alternatively: remove `,` from `continuation_punct` entirely; you won’t lose the important fixes because your paren/bracket context already carries the join.)

---

# Tests to add (these will prevent regressions)

You already have `does_not_join_object_properties_at_depth0`, but it doesn’t cover the **dedent boundary** that bit you.

Add a regression test that mirrors the failure:

### `does_not_join_dedented_property_start`

Input should include:

* a wrapped call inside a nested object property (so we still test unwrap works)
* then a **dedented** next property start (`trianglerain_globals:`)

Expected:

* unwrap inside the call (`pick(screen,\n  [ ... ])` joins)
* but keep newline before `trianglerain_globals: {`

This is the exact “screen merged with trianglerain_globals” case you reported.

Also add:

* `does_not_join_property_start_even_if_indent_differs`
  Minimal 2-line case: prev ends with `,` and next is `foo:` with different indent. Ensure no join.

---

## Direct answer to your question (“tighten property guard vs statement boundary heuristic?”)

Do **both**, but in the owning layer, as one coherent rule set:

* **Yes**: tighten the property-start guard to apply **regardless of indent** (with the exception of `ctx.in_string` / `ctx.in_block_comment` continuations).
* And more importantly: **stop using indentation/dedent as positive join evidence**. That’s what’s letting “comma + indent” fake “soft wrap” and corrupt structure.

That combination stops property-line merges *without* sacrificing the soft-wrap fixes you actually care about (paren/bracket continuations + ASI keyword joins).