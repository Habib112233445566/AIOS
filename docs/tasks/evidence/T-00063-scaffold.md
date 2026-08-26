# T-00063 — Task Ledger Control automated tests: Scaffold

**Date:** 2026-08-22
**Type:** scaffold (case registry + sandbox harness; bodies fail loudly)
**Depends on:** T-00062 spec

## What shipped

`code/aiosh-mcp/tests/test_ledger_matrix_smoke.py` — the cross-surface
matrix suite skeleton per spec T-00062: Sandbox class (isolated tasks
dir + audit home + fresh 3-task ledger + grant minting helper), six
registered case stubs raising `NotImplementedError("T-00064")`, runner
that reports each case `[✗] … (scaffold: …)` and exits 1.

## Verification
```
$ python3 tests/test_ledger_matrix_smoke.py
[✗] M1..M6 (scaffold) → exit 1   # expected scaffold state
```
Not wired into CI until T-00066, so baseline unaffected.

## Acceptance check
- [x] File imports/runs cleanly; all six cases present and failing
      loudly; harness (sandbox/registry/reporting) fully working.
