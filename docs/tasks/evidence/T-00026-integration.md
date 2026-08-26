# T-00026 — Task Ledger Control core service: Integration

**Date:** 2026-08-22
**Type:** integration wiring
**Depends on:** T-00025 unit tests

## What shipped

1. **CI registration point** — `ci/run_all_smokes.sh` gains
   `task_service_smoke` (`code/aiosh-mcp/tests/test_task_service_smoke.py`),
   placed after `mcp_smoke` so the Rust binaries are guaranteed built
   (rust_smoke runs first). The core service is now exercised on every
   baseline run through its production surface.

2. **Cross-substrate parity extension** — `rust_smoke.sh` ledger-parity
   step grew from 2 flows to 4, all against a scratch copy at any
   pointer position:
   - Rust `done` → Python reads pointer+seq (existing);
   - Python `block` → Rust reads blocked list (existing);
   - **Python `unblock`+`skip` → Rust `rebuild` → replay keeps pointer
     past the skip, `skipped[]`/`blocked[]` exact** (new, D4);
   - **Rust `skip` → Python reads pointer past it** (new, D4).
   This fulfills spec T-00022 §6's promised parity extension and closes
   the flow-coverage half of SPEC-TASK-LEDGER limitation L5.

3. Production reachability was already live from T-00024
   (`aios.task` in the MCP manifest; verified again inside
   task_service_smoke W1/W2 over the real binary).

## Verification

```
$ bash ci/run_all_smokes.sh
PASS: rust_smoke            # now contains 4-direction ledger parity
PASS: classifier_smoke
PASS: mcp_smoke
PASS: task_service_smoke    # NEW in CI
PASS: pentest_smoke
PASS: sandbox_smoke
PASS: retention_smoke
PASS: demo_smoke
PASS: cli_bash_smoke
PASS: task_ledger_unit
PASS: task_ledger_scaffold
== ALL 11 SMOKE SUITES PASS ==
```

New rust_smoke parity lines observed during the run:

```
parity ok: rust rebuilt python-written events (skip replayed, next_task=27)
parity ok: python read rust-written skip (next_task=28 skipped_tail=27)
```

## Acceptance check

- [x] Feature reachable through its production surface (MCP tool,
      asserted end-to-end in CI by task_service_smoke).
- [x] Cross-substrate parity confirmed where shared state is touched
      (4-direction ledger flows incl. rebuild replay).
- [x] Registration/discoverability updated (CI runner + manifest from
      T-00024; CLI help already listed `aiosh task` since T-00016).
- [x] Closest existing smokes for the integrated path pass
      (task_service_smoke + rust_smoke + task_ledger_unit).
