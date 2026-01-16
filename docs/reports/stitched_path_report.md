What you’re seeing isn’t “Ubuntu doing a bit of fun formatting.” It’s the classic **visual line wrap being copied as real newlines**, plus some “helpful” indentation on the continuation line. Then Bash does exactly what it’s supposed to do: newline ends the command, so your path gets chopped in half and the second half gets executed as a brand‑new command. Humans love this kind of UX. 

### What actually happened in your broken run

Your clipboard (after copy) effectively became:

* argument 1: `/home/johnf/code/textureportal/assets/` (a directory; Python tries to open it and dies)
* then a *second command*: `organized/procedural/seamless_procedural_abyssal_void_d184052a_3.png` (which of course “doesn’t exist” as a command)

So the error makes perfect sense, unfortunately.

### Why Blankstop currently misses this

Blankstop’s sanitizer has three main modes: **shell-like**, **JS-like**, and **text-mode fallback**. 

* In **shell-like mode**, current code joins:

  * explicit continuations (`\` for Bash, backtick for PowerShell, caret for cmd)
  * wrapped content *inside quotes* (string literal wraps)
* But it **does not** currently join **unquoted token wraps** like:

  * `.../assets/` + newline + `  organized/...`
  * `ml_basecolor_manual-` + newline + `  176858...`

Meanwhile, your second example (`ls -l ... manual-\n  176...`) often won’t even be detected as shell-like (no prompts, no `\`, etc.), so it falls into **text-mode**, where the join heuristics are conservative and can’t infer a wrap width from just two lines. Result: it refuses to join. 

So: totally predictable failure mode.

## Goal

Make Blankstop reliably “stitch” **broken path/filename tokens** across a newline when the newline is obviously a wrap artifact.

### Behavior table

| Situation (clipboard text)                        | Expected behavior                                                                     |
| ------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `/home/.../assets/\n  organized/procedural/x.png` | Join into `/home/.../assets/organized/procedural/x.png` (no space)                    |
| `ml_basecolor_manual-\n  1768589129403-1.png`     | Join into `ml_basecolor_manual-1768589129403-1.png` (no space)                        |
| Normal multi-line code / structured blocks        | Do **not** “flatten” real blocks; only join when the boundary looks like a token wrap |

### Invariants

* Only “token-stitch” when:

  * next line is **indented** (leading whitespace > 0), and
  * previous line ends with a **strong token-joiner** (at minimum: `/`, `\\`, `-`), and
  * next line starts with a **token-ish** char (alnum, `_`, `-`, `.`, `/`, `\\`)
* Join must be **no-space** (concatenate) and must **drop the indentation** on the continuation line.
* Never stitch across blank lines.

## Two viable designs

### Design A: Minimal “token stitch” rule in both shell reflow + text-mode join decision

**What changes**

1. Add a small heuristic like `is_punctuated_token_wrap(prev, next, next_leading_ws)` (join/no-space).
2. Use it in:

   * `src-tauri/src/lib/sanitize/shell/reflow.rs` (so shell-like input gets fixed)
   * `src-tauri/src/lib/sanitize/boundaries.rs` (so text-mode fallback also fixes it)
3. Add tests that reproduce your exact failures.

**Pros**

* Surgical, low risk.
* Fixes the “assets/\n  organized/…” and “manual-\n  176…” cases immediately.
* Works regardless of whether the snippet is detected as shell-like.

**Cons**

* Doesn’t try to fix every possible wrap style (like wraps at whitespace boundaries without punctuation). That’s probably fine for now.

### Design B: New “CLI unwrap mode” that aggressively reconstructs one-liner commands

**What changes**

* Add a dedicated CLI unwrapping pass that:

  * joins indented continuation lines even when the split is at whitespace
  * uses stronger “looks like a single command” detection (few lines, no blank lines, long first line, etc.)

**Pros**

* Fixes more “wrapped command” cases (including your line break after `...SLICING=off` if that’s also a wrap artifact).
* Better for terminal copy/paste overall.

**Cons**

* Higher risk of flattening real shell scripts (indentation is meaningful in humans’ brains, even if shell doesn’t care).
* More heuristics, more surface area.

**Recommendation:** start with **Design A**. It hits your pain exactly and doesn’t turn Blankstop into a shell-script reformatter.

## Concrete code touch points

* Shell join logic: @src-tauri/src/lib/sanitize/shell/reflow.rs
* Text-mode join decision: @src-tauri/src/lib/sanitize/boundaries.rs
* Shared heuristics location (small): @src-tauri/src/lib/sanitize/heuristics.rs
* Tests:

  * @src-tauri/src/lib/sanitize/shell/tests.rs (add the `assets/` example)
  * @src-tauri/src/lib/sanitize/tests.rs (add the `ml_basecolor_manual-` example)

This aligns with Blankstop’s pipeline structure (shell/js/text routing) and keeps changes in the sanitizer layer, which is the correct owner. 

## Tradeoffs you’re accepting

* You might occasionally join something like `foo-\n  bar` in prose into `foo-bar`. That’s not a tragedy; it’s arguably correct.
* You will *not* join arbitrary “word wrap” splits unless they look like tokens. That’s intentional: fewer surprise rewrites.