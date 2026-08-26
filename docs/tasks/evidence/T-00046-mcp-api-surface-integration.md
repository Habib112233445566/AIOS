# T-00046 — Task Ledger Control MCP/API surface: Integration

**Date:** 2026-08-22
**Type:** integration wiring
**Depends on:** T-00045 unit tests

## What shipped

1. **CI registration** — `ci/run_all_smokes.sh` gains `task_mcp_smoke`
   (`code/aiosh-mcp/tests/test_task_mcp_smoke.py`), placed beside
   `task_service_smoke`. The Python reference substrate's ledger tool
   is now exercised on every baseline run.
2. **Discoverability** — `tests/test_smoke.py` expected-set now
   includes `aios_task` (registered-tools assertion).
3. **Cross-substrate parity confirmed**: P-suite mints its grant with
   the RUST CLI and spends it on the PYTHON server — one
   `--tools "aios.task"` grant authorizes both substrates (D2 proven
   end-to-end); file-level parity remains covered by rust_smoke's
   4-flow step in the same CI run.

## Harness fixes during wiring (honesty)

P8 initially crashed under CI: mutation envelopes carry top-level
fields (`next_task`), not the wrapped `data` form — my assertion read
the wrong level; fixed to match the actual contract pinned by Rust.

## Verification

```
$ bash ci/run_all_smokes.sh → == ALL 13 SMOKE SUITES PASS ==
PASS: task_mcp_smoke        # NEW
(all prior suites unchanged-green)
```

## Acceptance check
- [x] Feature reachable through production surface, end-to-end in CI.
- [x] Cross-substrate parity exercised (grant + files).
- [x] Registration/discoverability updated.
- [x] Integrated-path smokes pass.
