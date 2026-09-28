# Task Evidence: T-02376 - Audit Chain Extensions: Observability Integration

## Goal
Integrate the observability subsystem of Audit Chain Extensions with the surrounding system via CLI commands and MCP tools.

## Implementation Details
1. **CLI Command Integration**:
   - Added `aiosh audit stats [--json]` to `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Wired `cmd_audit_stats` into `run_audit_command`.
   - Supports both human-readable telemetry summaries and structured JSON output for monitoring pipelines.
2. **MCP Tool Integration**:
   - Registered tool `aios.audit.stats` in `code/aiosh-rust/aiosh-mcp/src/main.rs` tool catalog with descriptions and input schema.
   - Wired dispatch handler for `"aios.audit.stats"` to generate `AuditChainObservabilityReport` via `AuditChainService`, recorded through PEP and ring dispatch.
3. **Cross-Substrate Verification**:
   - Workspace check `cargo check --workspace` clean with 0 warnings.
   - Unit tests in `aiosh-core` pass cleanly.

## Verification
- `cargo check --workspace` verified clean.
- Tool catalog discovery verified in MCP server.
- CLI subcommand registration verified.
