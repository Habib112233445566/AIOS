# Task T-02496 Evidence: Sandbox Recovery & Validation Integration

## Goal
Integrate Sandbox Enforcement Recovery & Validation across the CLI (`aiosh sandbox validate`, `aiosh sandbox recover`) and MCP surfaces (`aios.sandbox.validate`, `aios.sandbox.recover`).

## Integrated Surfaces & Features
1. **CLI Commands (`aiosh-cli/src/main.rs`)**:
   - `aiosh sandbox validate [--dir <PATH>] [--json]`: Audits presence of baseline factory profiles (`standard`, `strict`, `permissive`), checks in-memory profile limits, and verifies custom profile JSON files in specified directory.
   - `aiosh sandbox recover [--strategy <defaults|quarantine|dry_run>] [--dir <PATH>] [--json]`: Restores factory profiles and quarantines corrupt custom manifests into timestamped directories (`.quarantine_<timestamp>`).
2. **MCP Tool Handlers (`aiosh-mcp/src/main.rs`)**:
   - Registered `aios.sandbox.validate` with optional `custom_dir`.
   - Registered `aios.sandbox.recover` with `strategy` and `custom_dir`.
   - Handled calls through `dispatch::recorded_call`, creating immutable audit trail records.
3. **Integration Verification**:
   - CLI: `aiosh sandbox validate --json` successfully executed with code 0 and clean health report.
   - CLI: `aiosh sandbox recover --strategy dry_run --json` successfully executed with code 0.
   - MCP Test Suite: `pytest code/aiosh-mcp/tests/test_sandbox_mcp.py` passes 14/14 tests.
