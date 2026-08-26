# T-00055 — Task Ledger Control configuration: Unit Test

**Date:** 2026-08-22
**Type:** unit/wire tests
**Depends on:** T-00054 implementation

## What shipped

`code/aiosh-cli/tests/test_task_config_smoke.py` — standalone suite
driving the REAL `aiosh` binary (env-isolated):

| Case | Asserts |
|---|---|
| K1 | `task config`: six knobs, source=default |
| K2 | env override flips value AND source to "env" (8192) |
| K3 | override APPLIED end-to-end: 5000-char note refused at default, accepted at 8192, stored verbatim in event |
| K4 | invalid value → loud error NAMING the variable, exit 1, honest audit row (outcome=refused) |
| K5 | below-floor value refused naming the floor ("must be >= 64") |

Plus the injected-source Rust tests from T-00054 (`from_source_
precedence_and_loud_errors`, defaults/scaffold proofs).

## Broken-feature check

Sabotaged floor assertion → `[✗] K5`, exit=1; real suite PASS after.

## Verification
```
$ python3 code/aiosh-cli/tests/test_task_config_smoke.py → PASS (K1..K5)
$ cargo test → 79 passed; 0 failed
```

## Acceptance check
- [x] Standalone pass. [x] Negative cases asserted (K4/K5 + K3's
      refused-default half).
