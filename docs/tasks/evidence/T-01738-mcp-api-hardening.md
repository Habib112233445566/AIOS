# T-01738: Hardware Detection — MCP/API Surface Hardening

## Metadata
- **Task ID**: `T-01738`
- **Sub-Epic**: Sub-Epic 4: Hardware Detection MCP/API Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Hardening Actions Implemented

### 1.1 In-Closure Parameter Validation & Audit Logging Guarantee (Remediating MS-1)
- In `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - Moved all validation for `device_id` in `aios.hardware.get` and `file_path` in `aios.hardware.verify` into the closure passed to `dispatch::recorded_call`.
  - Guaranteed that invalid parameter attempts (missing ID, control character injection, oversized paths) are routed through the Policy Enforcement Point and recorded as failure events in the SQLite WAL audit ring rather than exiting silently prior to audit emission.

### 1.2 Target Parameter Attribution (Remediating MS-4)
- Updated `aios.hardware.get` to pass `target_name.as_deref()` to `dispatch::recorded_call`.
- Allows audit queries to filter hardware device queries by specific device ID targets.

### 1.3 Strict File Type Screening (Remediating MS-3)
- In `aios.hardware.verify`, verified that target path exists and is a regular file (`path.is_file()`), strictly refusing directories, named FIFOs, or character devices.
- Enforced a hard 10MB memory buffer ceiling (`meta.len() <= 10 * 1024 * 1024`).

### 1.4 Verification
- `cargo test -p aiosh-mcp --bin aiosh-mcp -- test_hardware_mcp_surface` PASSED.
