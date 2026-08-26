# T-00086 — observability: Integration (evidence)

**Date:** 2026-08-22
**Result:** metrics reachable through ALL THREE production surfaces;
discoverable via `tools/list`; integration smokes green.

## Registration-point updates (discoverability)

1. **Rust MCP manifest** (`aiosh-rust/aiosh-mcp/src/main.rs`):
   `aios.task` inputSchema `action.enum` now includes `"metrics"`;
   description updated to "Read-only: status, check, metrics".
   Verified live post-build:
   ```
   enum: ['status','check','done','block','unblock','skip','rebuild','metrics']
   desc mentions metrics: True ; call ok: True keys ['audit','config','tasks']
   ```
2. **Python reference server** (`aiosh-mcp/aiosh_mcp/server.py`): tool
   docstring now lists `metrics` in the read-only set (FastMCP derives
   the client-facing description from this).

## Production call paths exercised

| Surface | Path | Proof |
|---|---|---|
| Rust MCP wire | `tools/call aios.task {action:"metrics"}` | O1/O7a in `test_metrics_smoke.py` |
| CLI | `aiosh task metrics` | O3/O7b/O8b |
| Python reference | `aios_task(action="metrics")` | O2 |

## Cross-substrate parity

Key set {tasks, audit, config} asserted EQUAL across Rust wire and Python
(O2b). Ledger file-level parity re-proven by `rust_smoke.sh`
(Rust↔Python read/write/rebuild both directions) after the changes:

```
parity ok: python read rust-written state …
parity ok: rust rebuilt python-written events (skip replayed) …
== RUST SMOKE SUITE PASS ==
```

## Integration suites run (all green)

```
bash code/aiosh-rust/ci/rust_smoke.sh        → RUST SMOKE SUITE PASS
python3 tests/test_metrics_smoke.py          → 12/12 checks pass
python3 tests/test_task_mcp_smoke.py         → PASS (P1..P8)
python3 tests/test_task_service_smoke.py     → PASS (W1..W8)
python3 tests/test_ledger_matrix_smoke.py    → PASS (M1..M8)
```

No legacy-suite behavior changed; schema enum is additive.
