# T-00050 — Task Ledger Control MCP/API surface: Verification & Evidence

**Date:** 2026-08-22
**Type:** verification & evidence (no code changed)
**Depends on:** T-00049 documentation
**Artifact note:** instruction path `T-00050-verify.md`; ledger row's
declared artifact (`…-verification-evidenc.md`) mirrored byte-for-byte.

## 1. Epic-specific suites — all PASS

```
$ cargo test (workspace)                    → 77 passed; 0 failed (0 warnings)
$ python3 tools/test_task_ledger.py         → PASS (U1..U16)
$ python3 tools/test_task_ledger_scaffold.py→ PASS
$ python3 code/aiosh-mcp/tests/test_task_service_smoke.py → PASS (W1..W8)
$ python3 code/aiosh-mcp/tests/test_task_mcp_smoke.py     → PASS (P1..P8)
$ python3 code/aiosh-cli/tests/test_task_cli_smoke.py     → PASS (C1..C9)
```

## 2. Full baseline smoke set — 13/13 PASS

```
$ bash ci/run_all_smokes.sh
PASS: rust_smoke / classifier_smoke / mcp_smoke
PASS: task_service_smoke / task_mcp_smoke      # Python MCP surface NEW
PASS: pentest_smoke / sandbox_smoke / retention_smoke / demo_smoke
PASS: cli_bash_smoke / task_cli_smoke
PASS: task_ledger_unit / task_ledger_scaffold
== ALL 13 SMOKE SUITES PASS ==
```

## 3. State-file verification

```
pre-completion pointer : next_task = 50, completed = 1..49, seq = 49
post-completion pointer: next_task = 51, completed = 1..50, seq = 50
aiosh task check → {"ok":true,"total_tasks":10000}
```

## 4. Milestone — MCP/API surface sub-epic CLOSED (T-00041..T-00050)

Both substrates now expose the Task Ledger Control surface over MCP
behind the identical classifier→PEP→audit gate, with ONE grant valid
across both (Rust-minted grant spent on the Python server, proven in
CI). Cross-substrate drift is now guarded by three dedicated suites
(W/C/P). Notable catches this sub-epic: the `rebuild` gating hole on
the Python port (P6, fixed) and the bool-task_id coercion edge
(T-00048). SPEC §7 L5 RESOLVED; §8.1/§8.2 operator references live.
Task Ledger Control: **5/10 components closed**; configuration begins
at T-00051. `task_plan.md`/`progress.md` updated.
