# T-00060 — Task Ledger Control configuration: Verification & Evidence

**Date:** 2026-08-22
**Type:** verification & evidence (no code changed)
**Depends on:** T-00059 documentation
**Artifact note:** instruction path `T-00060-verify.md`; ledger row's
declared artifact (`…-verification-evidenc.md`) mirrored byte-for-byte.

## 1. Epic-specific suites — all PASS

```
$ cargo test (workspace)                     → 79 passed; 0 failed (0 warnings)
$ python3 tools/test_task_ledger.py          → PASS (U1..U16)
$ python3 tools/test_task_ledger_scaffold.py → PASS
$ python3 …/test_task_service_smoke.py       → PASS (W1..W8)
$ python3 …/test_task_mcp_smoke.py           → PASS (P1..P8)
$ python3 …/test_task_cli_smoke.py           → PASS (C1..C9)
$ python3 …/test_task_config_smoke.py        → PASS (K1..K5)
```

## 2. Full baseline smoke set — 14/14 PASS

```
$ bash ci/run_all_smokes.sh
PASS: rust_smoke / classifier_smoke / mcp_smoke
PASS: task_service_smoke / task_mcp_smoke / task_cli_smoke
PASS: task_config_smoke                      # NEW
PASS: pentest_smoke / sandbox_smoke / retention_smoke / demo_smoke
PASS: cli_bash_smoke / task_ledger_unit / task_ledger_scaffold
== ALL 14 SMOKE SUITES PASS ==
```

## 3. State-file verification

```
pre-completion pointer : next_task = 60, completed = 1..59, seq = 59
post-completion pointer: next_task = 61, completed = 1..60, seq = 60
aiosh task check → {"ok":true,"total_tasks":10000}
```

## 4. Milestone — configuration sub-epic CLOSED (T-00051..T-00060)

Research grounded in Twelve-Factor III (fetched live) → spec (D1–D6:
env vars, no files, no MCP exposure) → scaffold → implementation
(`LedgerConfig::from_env`, consumers in ledger/task_service/CLI,
python mirror) → K-suite → integration (CI 14/14) → security review
(operator-only boundary; u64 extremes probed) → hardening (86400 s
ceiling, both substrates) → documentation (§8.3 + index) → this
verification. Task Ledger Control: **6/10 components closed**;
automated-tests component begins at T-00061.
`task_plan.md`/`progress.md` updated.
