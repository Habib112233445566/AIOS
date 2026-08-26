# T-00065 — Task Ledger Control automated tests: Unit Test

**Date:** 2026-08-22
**Type:** unit tests (suite extension; one wrong assumption corrected)
**Depends on:** T-00064 implementation

## What shipped

Matrix suite extended M1..M6 → **M1..M8**:

| Case | Asserts |
|---|---|
| M7 | TTL-expired grant: refused fail-closed on BOTH substrates — even for read-only `status` when explicitly presented (semantic discovered during the case: initial assumption "expiry irrelevant for read-only" was WRONG and corrected against observed behavior on Python, then parity-confirmed over the Rust wire) |
| M8 | block holds pointer (`blocked:[2]`, next unchanged) → unblock restores retry (`blocked:[]`, next returns); uses fresh ledger via new `Sandbox.reset_ledger()` helper (shared-sandbox exhaustion fix — M5 completes the last task) |

## Verification

```
$ python3 tests/test_ledger_matrix_smoke.py
[✓] M1..M8 → PASS: task ledger matrix smoke (M1..M8)
```

## Acceptance check
- [x] New cases run standalone within the suite and pass.
- [x] Negative cases asserted (expired-grant refusals both substrates;
      blocked-state file contents).
