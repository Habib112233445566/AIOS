# T-00106 — Task Ledger Control / recovery & validation: Integration

Date: 2026-08-23 · Status: INTEGRATION COMPLETE — feature live on all three
production surfaces.

## Wiring changes

| Surface | Change |
|---|---|
| Python MCP (`code/aiosh-mcp/aiosh_mcp/server.py`) | `"validate"` added to `_TASK_READ_ONLY` + `_TASK_ACTIONS`; no-task_id set; `_run_task_action` routes to `mod.validate_state()`; tool docstring updated |
| Rust core (`aiosh-core/src/task_service.rs`) | `TaskAction::Validate` variant (parse/as_str, grant-free, no task_id) → `ledger::validate_state(p)` |
| Rust CLI (`aiosh-cli/src/main.rs`) | parse exclusion + usage text + dedicated help page for `aiosh task validate` |
| Rust MCP (`aiosh-mcp/src/main.rs`) | `validate` added to published inputSchema enum + description |
| Python reference CLI (`tools/task_ledger.py`) | already wired at T-00103 scaffold |

Gate semantics: read-only ⇒ **no PEP grant**, same class as
status/check/metrics; every invocation still writes exactly one honest
audit row via the shared dispatch gate (verified by probe: audit_id
present on wire result).

## Cross-substrate parity (drifted sandbox, seeded next_task tamper)

Rust MCP wire vs Python in-process MCP vs Rust CLI vs Python CLI —
**findings payload byte-equal modulo audit_id** (`consistent:false`,
identical checks/detail/replay/live on all three).

## Pre-existing asymmetry discovered and fixed

Probe exposed that the Python generic action path omitted
`classifier_policy_revision` from success envelopes while Rust
`dispatch::recorded_call` always attaches it. Python now attaches the same
key (additive, per the T-00082 additive-only convention). Affects all
aios.task actions symmetrically.

## Harness repairs required by the suite run (honest log)

1. Environment: `pip install -e code/aiosh-mcp` was missing in this fresh
   container (documented T-00002 baseline step).
2. `test_observability_smoke.py::b1_empty_ring_boundary` — `mkdtemp()` +
   unconditional `.mkdir()` → `FileExistsError`; `exist_ok=True` added.
3. `test_observability_smoke.py::o1_cli_happy` — compared the printed
   metrics snapshot (pre-call ring by design) against a POST-hoc SQL count,
   which always differs by the CLI's own trailing audit row. Rewritten to
   pin the real contract: `rows == count_before` AND
   `count_after == count_before + 1` (exactly-one-row proof). Also made
   `Sandbox.audit_count()` return 0 for a not-yet-created DB.

## Verification (all green)

- `cargo build` zero warnings; `cargo test` 82 pass (13+69).
- Suites: task_mcp_smoke (P), task_service_smoke (W), task_cli_smoke (C),
  ledger_matrix_smoke (M), metrics_smoke, observability_smoke (O),
  `rust_smoke.sh` — **ALL PASS**.
- Live probes: Rust MCP wire `aios.task validate` (audited), Rust CLI
  `aiosh task validate`, Python CLI `task_ledger.py validate` — agree.
