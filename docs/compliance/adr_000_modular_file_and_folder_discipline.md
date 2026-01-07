# ADR-000: Modular File and Folder Discipline

Status: Accepted
Date: 2026-01-07
Owners: JL · Coding agents
Scope: blankstop runtime code (app/src, src-tauri/src)

---

## Decision
We enforce small, single‑purpose modules and a predictable folder layout.

1) **No monoliths**
- New behavior goes into new modules in the correct subfolder.
- “Just add it here” edits that grow large files are disallowed.

2) **Size limits**
- Target ≤200 LOC (including blanks/comments).
- Hard cap 250 LOC. If you add logic to a file over the cap, you must split it.

3) **Naming**
- Files and folders are **snake_case**.
- Rust allows `mod.rs`, `lib.rs`, `main.rs` as standard exceptions.
- Barrel/index files should only re‑export (no logic).

4) **Structure reflects responsibility**
- Orchestrators stay thin. Logic lives in focused helper modules.
- When a folder grows past ~10–12 siblings, introduce subfolders by concern.

---

## Invariants
- I0.1: Every module has one clear responsibility.
- I0.2: Orchestrators wire; they do not own heavy logic.
- I0.3: Large files must be split when touched.
- I0.4: New files/folders are snake_case.

---

## Examples
- If `src-tauri/src/lib/win/clipboard_listener/mod.rs` grows large, move pieces into
  `clipboard_listener/{window.rs,clipboard.rs,handler.rs}`.
- If `app/src/config/settings.ts` grows large, split into
  `app/src/config/{settings.ts,events.ts,render.ts}`.

---

## Consequences
- Changes are easier to review and reason about.
- The repo stays navigable as features grow.
