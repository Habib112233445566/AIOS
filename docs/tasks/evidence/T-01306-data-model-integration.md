# T-01306: Init & Service Supervision - Data Model: Integration

## Metadata
- **Task ID:** `T-01306`
- **Subsystem:** `code/aiosh-rust/aiosh-core::service`
- **Component:** Init & Service Supervision Data Model Integration
- **Status:** Complete

## 1. Integrated Surfaces

### Operator CLI (`aiosh service`)
- Integrated `cmd_service` in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - `aiosh service validate --name <name> [--json]`: Validates service name syntax against SS1 rules (1..128 chars, alphanumeric, dots, dashes, underscores, valid extensions `.service`, `.socket`, `.target`, `.timer`, no control chars or path traversal).
  - `aiosh service validate --spec <file_or_inline_json> [--json]`: Parses and deeply audits full `ServiceSpec` against SS1..SS5 invariants.
  - Structured output formatting: human-readable status for terminal operators and standard JSON envelope with `code`, `data`, and `error` for automated systems.
  - Automatic audit row emission into SQLite WAL ring (`audit.db`) with classified rule flags via `classify_and_emit`.

### Autonomous Agent MCP Tool (`aios.service.validate`)
- Registered `aios.service.validate` tool schema in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - Arguments: `name?: string`, `spec?: object`, `grant_id?: string`.
  - Dispatched via `recorded_call` ensuring PEP authorization checks and immutable audit logging.
  - Returns standard response envelope with `valid: bool`, details, and any invariant violation error lists.

### Subsystem Master Test Runner Matrix (`tools/test_service_suites.py`)
- Created dedicated test runner `tools/test_service_suites.py`:
  - Criterion `SS1`: Service data model integrity & invariants (`test_service_data_model`).
  - Criterion `SS2`: Service CLI surface commands & options (`test_cmd_service_flow`).
  - Criterion `SS3`: Service MCP tool surface (`test_mcp_service_tools`).

## 2. Verification
- `test_cmd_service_flow` in `code/aiosh-rust/aiosh-cli/src/main.rs`: Exercises `--help`, unknown subcommands, missing args, valid/invalid names, valid/invalid inline specs, and JSON output envelopes.
- `test_mcp_service_tools` in `code/aiosh-rust/aiosh-mcp/src/main.rs`: Exercises discovery in `tool_manifest()`, valid/invalid names, control characters, valid/invalid specs, and missing arguments.
- Master test runner `tools/test_service_suites.py` passes all criteria `SS1..SS3`.
