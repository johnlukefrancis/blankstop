# ADR-007: Architecture-First Execution (No Band-Aids)

Status: Accepted
Date: 2026-01-07
Owners: JL · Coding agents
Scope: all blankstop changes (frontend + Rust)

---

## Context
We lose time when we add stopgaps instead of fixing the underlying system.
Blankstop should stay small, correct, and coherent.

---

## Decision
1) **No temporary fixes**
- Do not add “just for now” hacks.
- Every change must move the system toward the intended final design.

2) **Fix root causes, not symptoms**
- Describe the broken contract.
- Identify the single owning module.
- Fix the contract at the owner, not with parallel state or extra gates.

3) **Rewrite structure when necessary**
- If the current structure blocks correctness, change the structure.
- Do not preserve bad boundaries for convenience.

4) **Assume agentic throughput**
- We prefer correct architecture over minimal diffs.
- Split large refactors into small, clear modules per ADR‑000.

---

## Band‑aid detection (non‑compliant)
A change is non‑compliant if it:
- Adds duplicate state or shadow config.
- Adds timing-based luck (“delay/retry until it works”).
- Adds special cases without fixing the root owner.

---

## Consequences
- Fewer recurring bugs.
- A clearer system for future agents and humans.
