# Task Evidence: T-02396 - Audit Chain Extensions: Recovery & Validation Integration

## Goal
Integrate the recovery & validation subsystem of Audit Chain Extensions into the CLI and MCP server surfaces.

## Integration Details
1. **CLI Commands**:
   - `aiosh audit validate [--json]`: Executes `AuditChainRecoveryManager::validate`, reporting total events, healthy events, overall validity, and detailed diagnostic issues.
   - `aiosh audit repair [--backup-dir <dir>] [--json]`: Creates an atomic pre-flight database file snapshot and appends a forward repair anchor event, restoring chain integrity without rewriting historical ledger rows.
2. **MCP Tools**:
   - `aios.audit.validate`: Model Context Protocol tool inspecting and verifying structural and cryptographic invariants.
   - `aios.audit.repair`: Model Context Protocol tool executing forward recovery with optional backup directory specification.
   - Both tools execute through `dispatch::recorded_call`, enforcing policy checks and logging the recovery actions themselves.
3. **Workspace Check**:
   - `cargo check --workspace` clean with 0 warnings and 0 errors across all 4 crates.
