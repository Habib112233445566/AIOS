# T-02296: Grant Lifecycle Recovery & Validation Integration

## Overview
This task integrates the PEP Grant Store Recovery and Invariant Validation engine with the `aiosh-mcp` JSON-RPC server surface. Two operational tools have been registered in `tools/list` and mapped to dispatch handlers protected by the PEP audit ring buffer.

## Integrated MCP Tools

### 1. `aios.pep.grant.validate_store`
- **Tool Description**: Validates structural and hierarchical invariants across the entire PEP capability grant store.
- **Parameters**:
  - `store_path` (optional, string): Canonical path to the JSON grant store. If omitted, uses the configured default from `PepGrantConfig`.
- **Response**: Emits `PepGrantValidationReport` containing total grants, healthy count, full issue list (with severity and diagnostic code), `is_valid` flag, and `can_auto_repair` capability.

### 2. `aios.pep.grant.recover`
- **Tool Description**: Executes non-destructive quarantine and auto-repair on a corrupted, desynchronized, or orphaned PEP grant store.
- **Parameters**:
  - `store_path` (optional, string): Canonical path to the JSON grant store.
  - `dry_run` (optional, boolean): If `true`, computes planned recovery actions and post-validation results without altering files on disk.
- **Response**: Emits `PepGrantRecoveryResult` containing `ok`, `backup_path`, `quarantine_path`, array of applied repair actions, `repaired_count`, and the post-repair validation report.

## Verification
- MCP tool catalog verified in `list_tools()` with strict JSON schema definitions.
- Dispatch arms wired to `dispatch::recorded_call` guaranteeing immutable SHA-256 audit ring records for every store inspection and repair execution.
- Workspace compilation verified cleanly: `cargo check -p aiosh-mcp` finished with 0 errors and 0 warnings.
