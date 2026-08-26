# T-00049 — Task Ledger Control MCP/API surface: Documentation

**Date:** 2026-08-22
**Type:** documentation (no code changed)
**Depends on:** T-00048 hardening

## What shipped

1. **`docs/SPEC-TASK-LEDGER.md` §8.2 (new)** — Python reference server
   parity reference: naming convention note, one-grant-both-servers
   gate string, caps/envelopes, hardening notes, and a live-verified
   copy-pasteable `aios_task(action="check")` example (output captured
   verbatim: `{"ok": true, "total_tasks": 10000, "action": "check",
   "audit_id": 215}`).
2. **§7 L5 marked RESOLVED** — both substrates expose the ledger over
   MCP; residual wording replaced with the proof summary.
3. **§9 evidence index** extended with rows T-00040..T-00049.

## Acceptance check
- [x] Spec updated with what shipped + invocation.
- [x] Copy-pasteable example (live-verified).
- [x] Constraints stated (naming divergence, pre-gate envelope form).
- [x] Evidence files linked.
