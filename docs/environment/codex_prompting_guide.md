# Codex Prompting Guide — blankstop

Updated: 2026-01-07
Owners: JL
Depends on: docs/compliance/adr_000_modular_file_and_folder_discipline.md, docs/compliance/adr_005_typescript_native_contracts_and_rails.md, docs/compliance/adr_007_architecture_first_agentic_execution_no_band_aids.md

---

## 1) Core philosophy
- Prompts stay short and surgical.
- Point to docs and code paths; do not restate repo internals you cannot see.
- Do not ask for explicit plans or “think step by step.” Codex already plans when needed.

Every prompt is just:
- **Task** — what must happen.
- **Context** — essential scope and boundaries.
- **Refs** — docs to read first.
- **Deliver** — what Codex should produce.
- **Constraints** — hard rails only.

---

## 2) Prompt shape

```text
Task: <imperative, one sentence>
Context: <scope + constraints in 1–3 lines>
Refs: { @docs/... }
Deliver: <what to change or produce>
Constraints: <non‑negotiable rails only>
```

Notes:
- Keep the prompt under ~6–8 lines unless more refs are critical.
- Use @paths for docs or files; do not wrap @paths in backticks.

---

## 3) Constraints to include (when relevant)
- ADR-000: small modules; snake_case; no monolith growth.
- ADR-005: do not weaken types; use local TODO(ts-precision) for dynamic seams.
- ADR-007: no stopgaps; fix architecture, not symptoms.

---

## 4) Examples

### Feature change (code)
```text
Task: Add toast “Test notification” tray item to verify UI without touching clipboard.
Context: Tauri v2 tray app; toast window is a separate webview.
Refs: { @src-tauri/src/lib/tray/mod.rs, @src-tauri/src/lib/toast/mod.rs, @docs/compliance/adr_007_architecture_first_agentic_execution_no_band_aids.md }
Deliver: Implement the tray item and ensure it triggers a toast with text and auto-hide.
Constraints: Follow ADR-000 (small modules) and ADR-007 (no stopgaps).
```

### Debugging
```text
Task: Diagnose why toast messages are blank and persist on Windows.
Context: App runs in Windows; frontend in app/; toast is a webview window.
Refs: { @app/src/ui/toast.ts, @src-tauri/src/lib/toast/mod.rs }
Deliver: Identify root cause + implement a fix + summarize verification steps.
Constraints: No band‑aids; fix the owning layer.
```

---

## 5) Anti‑patterns to avoid
- Pasting large code blocks instead of pointing to @paths.
- Asking for verbose or step‑by‑step reasoning.
- “Quick fix” alternatives when ADR‑007 applies.
- Vague constraints like “follow best practices.”
