# T-01838: Network Bootstrap / MCP/API Surface: Hardening

## 1. Overview
- **Task ID**: `T-01838`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Goal**: Harden the MCP/API surface of Network Bootstrap against failure, resource leaks, and misuse.

---

## 2. Hardening Measures Implemented

### 1. Strict Size Bounds & Input Sanitation
- **Path Length Caps**: `sysfs_path`, `procfs_path`, `resolv_path` parameters are validated in `resolve_network_service`:
  - Length capped at $\le 1024$ characters.
  - Control characters rejected via `c.is_control()`.
- **Interface Name Bounds**: `validate_interface_name` enforces $\le 15$ characters and restricts character set to `^[a-zA-Z0-9_.-]+$`.
- **Read Bounds in Core Service**: Wrapped reads enforce size ceilings:
  - `MAX_SYSFS_FILE_BYTES` = 64 KB
  - `MAX_ROUTE_FILE_BYTES` = 1 MB
  - `MAX_RESOLV_FILE_BYTES` = 64 KB

### 2. Explicit Error Propagation & Standard Envelopes
- All tool handlers encapsulate logic within closures passed to `dispatch::recorded_call`.
- Failures return `{ "ok": false, "error": "..." }` or JSON-RPC error responses; never silent failures.
- Missing required fields (e.g. `interface` in `show`, `up`, `down`) return explicit error messages.

### 3. Resource Hygiene & Zero Leak Guarantee
- All service operations are stateless and bounded.
- Database connections in `AuditRing` and `PepStore` are managed via connection pools and transaction-safe statements.
- Zero temporary files or background processes created during tool execution.

### 4. Honest Audit Logging on Failure
- Every failed invocation (invalid argument, path too long, control character, interface not found) emits a failure audit row in the audit ring per ADR-0035 §F-2.
