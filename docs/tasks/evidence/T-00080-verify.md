# T-00080 — Task Ledger Control security policy: Verification & Evidence

**Date:** 2026-08-22
**Type:** verification & evidence (no code changed)
**Depends on:** T-00079 documentation
**Artifact note:** instruction path `T-00080-verify.md`; ledger row's
declared artifact (`…-verification-evidenc.md`) mirrored byte-for-byte.

## 1. Epic-specific suites — all PASS

```
$ cargo test (workspace)                    → 79 passed; 0 failed (0 warnings)
$ python3 tools/test_task_ledger.py         → PASS (U1..U16)
$ python3 tools/check_security_policy.py    → PASS: security policy criteria (S1..S5)
$ W/P/C/K/M suites                          → all PASS
```

## 2. Full baseline smoke set — 16/16 PASS

```
$ bash ci/run_all_smokes.sh
PASS: rust_smoke / classifier_smoke / mcp_smoke
PASS: task_service_smoke / task_mcp_smoke / task_cli_smoke
PASS: task_config_smoke / task_matrix_smoke
PASS: pentest_smoke / sandbox_smoke / retention_smoke / demo_smoke
PASS: cli_bash_smoke
PASS: security_policy                        # NEW this sub-epic
PASS: task_ledger_unit / task_ledger_scaffold
== ALL 16 SMOKE SUITES PASS ==
```

## 3. State-file verification

```
pre-completion pointer : next_task = 80, completed = 1..79, seq = 79
post-completion pointer: next_task = 81, completed = 1..80, seq = 80
aiosh task check → {"ok":true,"total_tasks":10000}
```

## 4. Milestone — security policy sub-epic CLOSED (T-00071..T-00080)

Root `SECURITY.md` shipped meeting OpenSSF Scorecard Security-Policy
criteria (verified by the permanent `security_policy` CI suite):
owner-provided private advisory channel, vulnerability scope derived
from six component reviews, supported surfaces, 7d/90d commitments,
rule-pack governance, and a linked knowledge index. Policy-artifact
review found no fabrications/secrets; enforcement prevents rot.
Task Ledger Control: **8/10 components closed**; observability begins
at T-00081. `task_plan.md`/`progress.md` updated.
