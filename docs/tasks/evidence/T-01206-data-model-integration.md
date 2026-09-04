# T-01206: Package Management - Data Model: Integration

## Metadata
- **Task ID:** `T-01206`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package`
- **Component:** Package Management Data Model Integration
- **Status:** Complete

## 1. Integrated Surfaces

### Operator CLI (`aiosh package`)
- Integrated `cmd_package` in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - `aiosh package validate --name <name> [--json]`: Validates package name syntax against PM1 rules.
  - `aiosh package validate --spec <file_or_inline_json> [--json]`: Parses and deeply audits full `PackageSpec` against PM1..PM5.
  - Structured output formatting: tabular summary for human operators and standard JSON envelope with `code`, `data`, and `error` for machines.
  - Automatic audit row emission into SQLite WAL ring (`audit.db`) with classified rule flags.

### Autonomous Agent MCP Tool (`aios.package.validate`)
- Registered `aios.package.validate` tool schema in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - Arguments: `name?: string`, `spec?: object`, `grant_id?: string`.
  - Dispatched via `recorded_call` ensuring PEP authorization checks and immutable audit logging.
  - Returns standard response envelope with `valid: bool`, details, and any violation error lists.

## 2. Verification
- `test_cmd_package_flow` in `code/aiosh-rust/aiosh-cli/src/main.rs` tests valid names, invalid names, valid specs, invalid specs, help flag, and error branches.
- `test_mcp_package_tools` in `code/aiosh-rust/aiosh-mcp/src/main.rs` tests valid name, invalid name, control character injection, valid spec, invalid spec (self-dependency), and missing argument handling.
