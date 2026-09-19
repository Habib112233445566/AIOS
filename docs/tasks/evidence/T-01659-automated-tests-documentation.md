# T-01659: Automated Tests Documentation

## Sub-Epic
Kernel Module Management / Automated Tests

## Objective
Document the complete automated testing architecture, test batteries (KM1..KM8), invariants (AT-KM1..AT-KM5), test execution procedures, and cross-surface parity verification in `docs/kernel_module_management.md`.

## Deliverables
1. **Section 9 of `docs/kernel_module_management.md`**:
   - Detailed specification of all 8 test batteries:
     - KM1: `test_kernel_module_data_model.rs` (Invariants KM1..KM5: module syntax, parameter validation, install command safety, preset completeness)
     - KM2: `test_kernel_module_service.rs` (Runtime service: procfs fallback, pre-commit conflict detection, atomic persistence, idempotent mutations)
     - KM3: `test_kernel_module_config.rs` (Subsystem configuration: modprobe.d and modules-load.d line parsers, store ingestion, CFG-KM1..CFG-KM5)
     - KM4: `test_kernel_module_automated.rs` (In-tree integration: compound state transitions, scale limits, 10 MiB document ceiling, corrupt store recovery)
     - KM5: `test_kernel_module_cli_smoke.py` (Operator CLI: subcommands, exit codes, terminal sanitization)
     - KM6: `test_kernel_module_mcp_smoke.py` (Agent MCP: JSON-RPC tools `aios.kernel_module.*`, schema advertising, full lifecycle, cross-surface parity)
     - KM7: `test_kernel_module_config_smoke.py` (Configuration integration: `aiosh mod import` and `export`, roundtrip fidelity, conflict rejection)
     - KM8: `test_kernel_module_automated_cases.py` (Compound lifecycle sequences, boundary values, concurrent store isolation)
   - Invariants AT-KM1 through AT-KM5 formal descriptions.
   - Test orchestration instructions via `tools/test_kernel_module_suites.py`.
   - Evidence cross-references for T-01651 through T-01658.

## Verification
- Documentation verified in `docs/kernel_module_management.md`.
- Evidence recorded in `docs/tasks/evidence/T-01659-automated-tests-documentation.md` and `T-01659-documentation.md`.
