# T-01676: Observability Integration

## Sub-Epic
Kernel Module Management / Observability

## Objective
Integrate the Kernel Module Management Observability subsystem (`KernelModuleObservabilityReport`) into the operational CLI (`aiosh mod observability`) and agent MCP surface (`aios.kernel_module.observability`).

## Implementation Summary
1. **CLI Surface Integration (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
   - Added `observability` and `status` subcommands under `aiosh mod`.
   - Supports `--store <path>`, `--proc-modules <path>`, `--policy <path>`, and `--json`.
   - Resolves live procfs and configuration store, evaluates compliance against security policy, and emits structured audit row via `classify_and_emit`.
2. **MCP Surface Integration (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - Registered tool `aios.kernel_module.observability` in `list_tools` with full schema.
   - Handled `aios.kernel_module.observability` in `call_tool` dispatched via `dispatch::recorded_call`.
3. **Integration Smoke Test (`code/aiosh-cli/tests/test_kernel_module_observability_smoke.py`)**:
   - Validates default CLI observability report.
   - Validates CLI report with synthetic mock `/proc/modules` and custom store.
   - Validates JSON-RPC invocation of `aios.kernel_module.observability`.

## Verification
- Verified end-to-end via `python code/aiosh-cli/tests/test_kernel_module_observability_smoke.py`.
- Artifacts: `docs/tasks/evidence/T-01676-observability-integration.md` and `T-01676-integration.md`.
