# T-00045 — Task Ledger Control MCP/API surface: Unit Test

**Date:** 2026-08-22
**Type:** unit/wire tests (one real port-bug fix)
**Depends on:** T-00044 implementation

## What shipped

`code/aiosh-mcp/tests/test_task_mcp_smoke.py` — standalone suite
driving the registered `aios_task` tool in a fully isolated sandbox:

| Case | Class | Asserts |
|---|---|---|
| P1 | valid | status envelope + audit row |
| P2 | valid | done: pointer + event + evidence stub |
| P3 | invalid | unknown action refused pre-gate |
| P4 | invalid | empty note refused pre-gate |
| P5 | boundary | note >4096 refused |
| P6 | gate | rebuild WITHOUT grant → pep refusal, audited, state untouched |
| P7 | failure mode | NO-SKIP: byte-identical state, no events |
| P8 | parity | skip → rebuild through the SAME surface: pointer past skip |

## Failing-test-first catch (the headline)

P6 failed on first run: the Python port had `rebuild` inside
`_TASK_READ_ONLY`, while Rust's `requires_grant()` treats it as
consequential — an ungrantable-state-rewrite hole on the reference
substrate. Fixed (`_TASK_READ_ONLY = {"status","check"}`), P6 green.
This is precisely the cross-substrate drift the component exists to
prevent; the test suite is now its permanent guard.

Also fixed en route: this VM lacked the T-2 baseline editable install
(`pip install -e code/aiosh-mcp`) — restored rather than shimming
sys.path, matching the established environment contract.

## Broken-feature check

Sabotaged copy (assertion string replaced) → `[✗] P3 …`, exit=1;
real suite immediately after: `PASS: task mcp wire smoke (P1..P8)`.

## Verification

```
$ python3 code/aiosh-mcp/tests/test_task_mcp_smoke.py  → PASS (P1..P8)
$ python3 tests/test_smoke.py                          → PASS (baseline)
```

## Acceptance check
- [x] Standalone pass. [x] Negative cases asserted (P3/P4/P5/P6/P7).
