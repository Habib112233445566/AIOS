# T-01216: Package Management - Core Service: Integration

## Metadata
- **Task ID:** `T-01216`
- **Subsystem:** `code/aiosh-rust/aiosh-cli` & `code/aiosh-rust/aiosh-mcp`
- **Component:** Package Management Core Service Integration
- **Status:** Complete

## 1. Production Surfaces Integrated

### CLI Surface (`aiosh package`)
Extended `cmd_package` in [main.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-cli/src/main.rs) with:
- `aiosh package list [--format <deb|apk|flatpak|tarball>] [--state <state>] [--pattern <pattern>] [--limit <n>] [--store <path>] [--json]`
- `aiosh package show <name> [--store <path>] [--json]`
- `aiosh package plan --actions <json_or_file> [--dry-run] [--store <path>] [--json]`
- Audited with `classify_and_emit` into the append-only SQLite ring.

### MCP Surface (`aios.package.*`)
Extended `Server` in [main.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-mcp/src/main.rs) with:
- `aios.package.list`: Queries packages matching optional filters (format, state, pattern, limit, store_path).
- `aios.package.get`: Retrieves specific package specification by name.
- `aios.package.plan`: Plans package transaction validating closure and delta size arithmetic.
- Routed through `dispatch::recorded_call` ensuring PEP authorization checks and canonical JSON audit recording.

## 2. Test Verification
All CLI and MCP integration test suites passed:
- `aiosh-cli`: 21 tests passed (`test_cmd_package_flow` exercising validate, list, show, plan happy and error paths).
- `aiosh-mcp`: 9 tests passed (`test_mcp_package_tools` exercising validate, list, get, plan happy and error paths).
