# T-00064 — Task Ledger Control automated tests: Implementation

**Date:** 2026-08-22
**Type:** implementation (test suite; two harness-level discoveries)
**Depends on:** T-00063 scaffold

## What shipped

All six matrix cases implemented in
`code/aiosh-mcp/tests/test_ledger_matrix_smoke.py`:

| Case | Proves |
|---|---|
| M1 | wildcard `aios.*` grant authorizes `done` on Python MCP (+ no-grant refusal first) |
| M2 | the SAME wildcard grant object authorizes the Rust MCP over real stdio wire |
| M3 | exact-string grant `aios.task.done` matches nothing → pep-gate refusal on BOTH surfaces |
| M4 | evidence-list cap (>16) refused pre-gate on Python MCP |
| M5 | holder-then-mutate: `done` fails loudly (`ledger lock busy`, 1 s budget) while lock held; succeeds after release |
| M6 | `AIOSH_LEDGER_MAX_TEXT=64` propagates to the Python MCP surface via a fresh interpreter |

## Two discoveries made by building this suite (recorded honestly)

1. **In-process env binding.** Direct-fn Python calls read the SUITE
   process's `os.environ` (audit paths bind at import) — a sandbox env
   dict passed only to subprocesses silently points Python tools at the
   real `~/.aios` DB. Fixed in the harness (sandbox applied to
   `os.environ`), matching P-suite's established pattern.
2. **`rebuild` is lock-free BY DESIGN** (recovery tool; spec
   T-00012 §4). M5 originally used `rebuild` and passed trivially —
   rewritten to a lock-taking mutation (`done`) so the case actually
   exercises bounded-wait semantics.

## Verification

```
$ python3 tests/test_ledger_matrix_smoke.py
[✓] M1..M6 → PASS: task ledger matrix smoke (M1..M6)
```

## Acceptance check
- [x] Targeted suite passes (all six cross-surface interactions).
- [x] No production changes required — both discoveries were harness/
      design-documentation level, and the lock-free-rebuild fact is now
      encoded in the suite comment + evidence.
