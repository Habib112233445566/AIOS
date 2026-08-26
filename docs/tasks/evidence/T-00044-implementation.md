# T-00044 — Task Ledger Control MCP/API surface: Implementation

**Date:** 2026-08-22
**Type:** implementation (Python reference server)
**Depends on:** T-00043 scaffold; spec T-00042 (D1–D5)

## What shipped

`code/aiosh-mcp/aiosh_mcp/server.py`:

- **`aios_task` implemented** — full 7-action mirror of the Rust tool:
  - validation BEFORE the gate (structural + caps + conditional
    presence; `{ok:false}` envelope, no gate interaction), then
    `_dispatch.dispatch(tool="aios.task", command="task.<action>",
    require_grant=per-action)` — the SAME gate string as Rust, so one
    `--tools "aios.task"` grant authorizes both substrates (D2);
  - ledger ops delegated to `tools/task_ledger.py` (imported once;
    paths bind from `AIOSH_TASKS_DIR`), preserving no-skip, atomic
    writes, fsync'd events, bounded lock, D4 replay;
  - envelope parity (D4): bare payloads wrapped
    `{ok,action,data}`; mutations `{**raw, action}`; every outcome
    commits exactly one honest audit row (`ok` / gate-refused rows
    written inside `dispatch` / business `error` via commit).
- `_load_task_ledger()` + `_run_task_action()` helpers;
  `_TASK_READ_ONLY` policy set; caps consts (4096/16).

## Verification (live probes, scratch sandbox, real grant)

```
[✓] status envelope + audit row
[✓] done: pointer+event+evidence stub
[✓] structural/semantic refusals pre-gate (unknown action, empty note)
[✓] no-grant mutation refused at gate ("pep"), audited
[✓] NO-SKIP refusal, byte-identical state
[✓] rebuild replay through the tool: skip survived (next_task=3)
$ python3 tests/test_smoke.py → PASS (baseline unaffected)
```

## Acceptance check
- [x] Targeted probes pass (parity with Rust semantics).
- [x] No regression: mcp_smoke green; no new dependencies.
- [x] Audit invariants: consequential actions → exactly one row each.
