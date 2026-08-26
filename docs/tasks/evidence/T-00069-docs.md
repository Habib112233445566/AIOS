# T-00069 — Task Ledger Control automated tests: Documentation

**Date:** 2026-08-22
**Type:** documentation (no code changed)
**Depends on:** T-00068 hardening

## What shipped

1. **`docs/SPEC-TASK-LEDGER.md` §8.4 (new)** — the six-suite test map
   for this component (surface × case-count table), matrix-suite
   purpose statement, conventions for new cases, and the encoded
   design fact that `rebuild` is intentionally lock-free.
2. **§9 evidence index** extended with rows T-00040..T-00069.

## Acceptance check
- [x] Spec updated with what exists + where.
- [x] Conventions documented (isolated sandboxes, observable
      assertions, broken-checks, timeouts).
- [x] Evidence files linked.
