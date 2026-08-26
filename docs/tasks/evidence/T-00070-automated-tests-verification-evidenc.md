# T-00070 — Task Ledger Control automated tests: Verification & Evidence

**Date:** 2026-08-22
**Type:** verification & evidence (no code changed)
**Depends on:** T-00069 documentation
**Artifact note:** instruction path `T-00070-verify.md`; ledger row's
declared artifact (`…-verification-evidenc.md`) mirrored byte-for-byte.

## 1. Epic-specific suites — all PASS

```
$ cargo test (workspace)                      → 79 passed; 0 failed (0 warnings)
$ python3 tools/test_task_ledger.py           → PASS (U1..U16)
$ python3 tools/test_task_ledger_scaffold.py  → PASS
$ python3 …/test_task_service_smoke.py        → PASS (W1..W8)
$ python3 …/test_task_mcp_smoke.py            → PASS (P1..P8)
$ python3 …/test_ledger_matrix_smoke.py       → PASS (M1..M8)
$ python3 code/aiosh-cli/tests/test_task_cli_smoke.py   → PASS (C1..C9)
$ python3 code/aiosh-cli/tests/test_task_config_smoke.py→ PASS (K1..K5)
```

## 2. Full baseline smoke set — 15/15 PASS

```
$ bash ci/run_all_smokes.sh
PASS: rust_smoke / classifier_smoke / mcp_smoke
PASS: task_service_smoke / task_mcp_smoke / task_cli_smoke
PASS: task_config_smoke / task_matrix_smoke    # NEW this sub-epic
PASS: pentest_smoke / sandbox_smoke / retention_smoke / demo_smoke
PASS: cli_bash_smoke / task_ledger_unit / task_ledger_scaffold
== ALL 15 SMOKE SUITES PASS ==
```

## 3. State-file verification

```
pre-completion pointer : next_task = 70, completed = 1..69, seq = 69
post-completion pointer: next_task = 71, completed = 1..70, seq = 70
aiosh task check → {"ok":true,"total_tasks":10000}
```

## 4. Milestone — automated tests sub-epic CLOSED (T-00061..T-00070)

The ledger now carries a permanent cross-surface regression matrix
(M1–M8) alongside the four per-surface suites, wired into CI (15
suites). Suite hardening bounded all subprocess waits and made the
lock-holder leak-proof. Security review of the suites themselves: no
leaks, no bypass surface. Task Ledger Control: **7/10 components
closed**; security-policy component begins at T-00071.
`task_plan.md`/`progress.md` updated.
