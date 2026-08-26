# T-00090 — observability: Verification & Evidence (epic closure)

**Date:** 2026-08-22
**Verdict:** Task Ledger Control **observability component VERIFIED &
CLOSED** (T-00081..T-00090). Full baseline green.

## Full baseline (captured 2026-08-22, this session)

`bash ci/run_all_smokes.sh` → **ALL 17 SMOKE SUITES PASS**, including the
newly wired permanent `metrics_smoke` suite:

```
PASS: rust_smoke          PASS: retention_smoke
PASS: classifier_smoke    PASS: demo_smoke
PASS: mcp_smoke           PASS: metrics_smoke   ← new, permanent in CI
PASS: task_service_smoke  PASS: cli_bash_smoke
PASS: task_mcp_smoke      PASS: task_cli_smoke
PASS: pentest_smoke       PASS: task_config_smoke
PASS: sandbox_smoke       PASS: task_matrix_smoke
                          PASS: security_policy
                          PASS: task_ledger_unit
                          PASS: task_ledger_scaffold
```

Rust unit tests: `cargo test` → 5/5 binaries ok, **79 tests, zero
warnings**. Metrics unit smoke standalone: **12/12 checks pass**.

## What the observability component shipped (summary)

- `aios.task {action:"metrics"}` + `aiosh task metrics` +
  Python `_task_metrics`: stable additive-only snapshot
  `{tasks, audit, config}`; counts-only disclosure; read-only,
  grant-free; one honest audit row per call.
- Defects caught & fixed by tests-first discipline (T-00085): wire
  accepted `task_id` on metrics; CLI silently swallowed stray operands.
- Discoverability (T-00086): `"metrics"` added to the published
  inputSchema enum; descriptions updated on both substrates.
- Hardening (T-00088): O(1) COUNT(*) replaces full-table materialization;
  caps/depth/env bounds verified empirically.
- Docs (T-00089): SPEC-TASK-LEDGER §8.6 with limitations L-O1..L-O3.

## Milestone bookkeeping

- Task Ledger Control: **9/10 components closed**
  (data model, core service, CLI, MCP/API, configuration, automated
  tests, security policy, observability ✓). Remaining:
  **documentation** component starts at T-00091; recovery & validation
  follows.
- `task_plan.md` and `progress.md` updated this session with the
  T-00085..T-00090 entries.
- Pointer advanced strictly one per completion:
  84 → 85 → 86 → 87 → 88 → 89 → **90 → next_task = 91**.
