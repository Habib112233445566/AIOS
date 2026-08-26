# T-00066 — Task Ledger Control automated tests: Integration

**Date:** 2026-08-22
**Type:** integration wiring
**Depends on:** T-00065 unit tests
**Honesty note:** this file was written AFTER `task done 66` (a cwd
slip sent the original write to a non-existent relative path). Content
reflects exactly what was verified before completion; nothing retrofitted.

## What shipped

- **CI registration** — `ci/run_all_smokes.sh` gains `task_matrix_smoke`
  (`code/aiosh-mcp/tests/test_ledger_matrix_smoke.py`), after
  `task_config_smoke`. The cross-surface matrix (wildcard/narrow grants
  on both substrates, concurrent-writer lock-busy, config propagation,
  grant expiry, block/unblock) runs on every baseline.
- No production changes; no other suites affected.

## Verification

```
$ bash ci/run_all_smokes.sh → == ALL 15 SMOKE SUITES PASS ==
PASS: task_matrix_smoke     # NEW
```

## Acceptance check
- [x] Matrix runs end-to-end in CI. [x] Registration complete.
- [x] Integrated-path smokes green.
