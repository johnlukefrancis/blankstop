# CLAUDE.md — blankstop (repo)

*Merges with ~/.claude/CLAUDE.md (nearest wins). Keep this file short; detailed rails live in docs/compliance/*.md.*

## Scope
- This repo is **blankstop** only. Ignore Triangle Rain skills/rails.
- Docs in `docs/compliance/` are the primary architectural contracts.

## Repo rails (high signal)
- **snake_case** for files/folders (Rust `mod.rs`/`lib.rs`/`main.rs` are allowed).
- Prefer **tiny, single‑purpose modules**; target ≤200 LOC, hard cap 250 LOC.
- No monolith growth: split when adding new responsibilities.

## Workflow (Windows + WSL)
- Edit in WSL, run the app in Windows (D:\code\blankstop).
- Use `scripts/sync_to_windows.sh` before Windows runs.
- Use pnpm for frontend commands.

## TypeScript/Rust contracts
- Do **not** weaken types to satisfy the compiler. Fix code or tighten types.
- Use `any`/`unknown` only at real dynamic seams with a TODO tag.

## End of turn contract
- List files touched (+ why).
- List acceptance checks actually run.
- Suggest a commit message `emoji type(scope): summary`.
