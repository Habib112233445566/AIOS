# T-01666: Security Policy Integration

## Sub-Epic
Kernel Module Management / Security Policy

## Objective
Integrate the Kernel Module Management Security Policy (`KernelModuleSecurityPolicy`) into the operational CLI surface (`aiosh mod policy`) and agent MCP surface (`aios.kernel_module.policy`).

## Implementation Summary
1. **CLI Surface Integration (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
   - Added `policy` subcommand under `aiosh mod`.
   - Flags supported:
     - `--policy <path>` / `--config <path>`: Custom policy file with 64 KiB cap.
     - `--evaluate-store`: Evaluates all directives and autoload entries in the store.
     - `--module <name>` / `aiosh mod policy <name>`: Evaluates policy against a specific module.
     - Default: Outputs active policy mode, prohibited/protected module counts, allowed install commands, disallowed parameter keys, and bounds.
   - Audit integration: Every policy evaluation or inspection emits an audit row via `classify_and_emit(&mut ctx, "kernel_module", "policy", ...)`.
   - Exit codes: 0 for allowed/clean, 1 for policy violations or resolution errors, 2 for invalid arguments.
2. **MCP Surface Integration (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - Registered tool `aios.kernel_module.policy` in `list_tools` with full JSON schema.
   - Handled `aios.kernel_module.policy` in `call_tool`, supporting store evaluation, single module evaluation, and policy inspection.
   - Dispatched through `dispatch::recorded_call` with audit row recording.
3. **Automated Integration Smoke Test (`code/aiosh-cli/tests/test_kernel_module_policy_smoke.py`)**:
   - Tests CLI policy inspection (`aiosh mod policy --json`).
   - Tests CLI module evaluation for permitted and prohibited modules.
   - Tests CLI store evaluation for clean and violating stores.
   - Tests MCP JSON-RPC tool invocation end-to-end.

## Verification
- Verified end-to-end with `python code/aiosh-cli/tests/test_kernel_module_policy_smoke.py`.
