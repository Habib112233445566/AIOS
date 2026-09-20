# T-01696: Kernel Module Management Recovery & Validation Integration

## Sub-Epic
Kernel Module Management / Recovery & Validation (T-01696)

## Objective
Integrate the recovery & validation subsystem of Kernel Module Management across both the CLI (`aiosh mod check [--auto-recover]`) and MCP (`aios.kernel_module.check`) production surfaces with full audit trail logging and cross-surface parity.

## Integrated Surfaces
1. **Operator CLI Surface (`aiosh mod check`)**:
   - Wired in `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Supports `--store <path>`, `--auto-recover` (or `--recover`), and `--json`.
   - Non-destructive self-healing with timestamped quarantine backup (`<store>.corrupt.<timestamp>.bak`).
   - PEP / audit integration: records event in audit ring with status and health metrics.
   - Comprehensive unit test coverage in `test_cmd_kernel_module_flow`.

2. **MCP Tool Surface (`aios.kernel_module.check`)**:
   - Advertised in `tool_manifest` with JSON schema validation.
   - Implemented in `call_tool` dispatching to `recover_store_file` or `check_store_file`.
   - Security bounds on path length (<= 1024 bytes) and control character rejection.
   - Verified via 4 new test assertions in `test_mcp_kernel_module_tools`.

3. **End-to-End Integration Smoke Suite**:
   - Created `code/aiosh-cli/tests/test_kernel_module_recovery_smoke.py`.
   - Verified 5 test scenarios:
     - `test_cli_check_healthy`
     - `test_cli_check_corrupted_and_auto_recover`
     - `test_cli_check_human_output`
     - `test_mcp_check_tool`
     - `test_cross_surface_parity`
   - All tests passed 100%.
