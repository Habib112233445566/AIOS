# T-00029 — Task Ledger Control core service: Documentation

**Date:** 2026-08-22
**Type:** documentation (no code changed)
**Depends on:** T-00028 hardening

## What shipped

1. **`docs/SPEC-TASK-LEDGER.md` §8 (new)** — operator/agent reference
   for the `aios.task` MCP surface: action/grant table, error-channel
   semantics (`-32602` schema · `-32700` oversized line · `isError`
   business refusals), lock-busy behavior, and **copy-pasteable,
   live-verified examples** (status call output captured verbatim,
   `grant create` consent flow, agent mutation shape).

2. **§7 limitations refreshed to current truth:**
   - L1 updated — lock wait is now bounded (5 s, T-00028); loud
     `ledger lock busy` failure replaces the infinite-hang behavior.
   - L2 **RESOLVED** — ancestor-walk resolver (T-00024), live-verified.
   - L3 **RESOLVED** — rebuild pointer replay (T-00024), parity-proven.
   - L5 updated — parity now covers 4 cross-substrate flows (T-00026);
     residual nuance stated honestly (legacy Python MCP server has no
     `aios.task` by design).

3. **§9 evidence index** extended with rows T-00020..T-00029 so the
   whole core-service chain is linked from the doc.

4. **`docs/README.md`** task-ledger rule now mentions the `aios.task`
   MCP surface and points at §8 for the operator reference.

## Verification

- The §8 status example was executed before being written down; the
  embedded output snippet (`"ok":true,"action":"status",
  "data":{"next_task":29,…},"audit_id":97`) is the real response.
- Grant/mutation examples match the empirically verified shapes from
  T-00025 (W2/W8) and T-00027 (S1/S2).

## Acceptance check

- [x] Relevant spec updated with what shipped and how to invoke it.
- [x] At least one copy-pasteable example command / tool call (§8,
      live-verified).
- [x] Constraints and known limitations stated, not omitted (§7 with
      explicit RESOLVED markers for L2/L3).
- [x] Task evidence files linked from the doc (§9 index).
