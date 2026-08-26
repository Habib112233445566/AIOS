# T-00088 — observability: Hardening (evidence)

**Date:** 2026-08-22
**Result:** all hardening criteria verified empirically; one real gap fixed;
build + suites green.

## Hardening inventory (metrics surfaces)

| Criterion | Status | Evidence |
|---|---|---|
| Size caps on file reads | Already enforced (`read_capped`): 10 MiB state → loud `too large … > cap 4194304` | T-00087 S2 |
| Parser depth bound | serde_json recursion limit → `recursion limit exceeded` on 300-level nesting, exit 1, no panic | T-00087 S2b |
| Env-knob bounds + loud errors | overflow knob → `invalid AIOSH_LEDGER_MAX_TEXT='…': not a decimal integer`; floor/ceiling per ledger_config | T-00085 O7c, T-00087 S4 |
| Bounded lock wait | flock poll ≤ AIOSH_LEDGER_LOCK_TIMEOUT_SECS (ceiling 86400) — pre-existing (T-00028/T-00058) | ledger.rs |
| Wire request cap | 1 MiB line cap with framing-preserving drain (T-00028) — unchanged | mcp main.rs |
| Standard envelope, never silent | corrupt state → ok:false both substrates; refusal paths carry reason | O8a/O8b |
| Honest audit row incl. failure | exactly ONE row per call (ok AND refused) re-proven post-fixes | T-00087 S6 probe |
| Resource cleanup | Python: audit conn closed in `finally` (verified in current source); Rust: RAII connections | server.py:366-372 |
| **Memory-bounded row count** | **FIXED this task**: CLI + MCP used `tail(i64::MAX)` (materializes every live row to count). Replaced with O(1) `COUNT(*)` via `AuditRing::count()` in both `aiosh-cli/src/main.rs` and `aiosh-mcp/src/main.rs`. | this task |

## Note on concurrent baseline repair

Mid-session, the Python `_task_metrics` gained a dispatch-gate route +
try/except + conn-close (docstring cites matrix case M10, 2026-08-22,
from a concurrent session's baseline repair). This task VERIFIED that
hardening empirically rather than duplicating it; no Python change needed.

## Verification

```
cargo build                    → zero warnings
cargo test                     → 5/5 binaries ok (79 tests)
tests/test_metrics_smoke.py    → 12/12 checks pass
test_ledger_matrix_smoke.py    → PASS (M1..M8)
test_task_mcp_smoke.py         → PASS (P1..P8)
test_task_service_smoke.py     → PASS (W1..W8)
```
