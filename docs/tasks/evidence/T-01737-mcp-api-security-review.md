# T-01737: Hardware Detection — MCP/API Surface Security Review

## Metadata
- **Task ID**: `T-01737`
- **Sub-Epic**: Sub-Epic 4: Hardware Detection MCP/API Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Threat Modeling & Surface Analysis
Audited the 5 MCP tools (`aios.hardware.scan`, `aios.hardware.list`, `aios.hardware.get`, `aios.hardware.summary`, `aios.hardware.verify`) and helper functions in `code/aiosh-rust/aiosh-mcp/src/main.rs`:

### Scenario MS-1: Audit Log Bypass on Invalid Arguments
- **Hazard**: Pre-dispatch validation returning `return json!({ "ok": false, "error": ... });` outside of `dispatch::recorded_call` would prevent failed or malicious requests from being logged to the SQLite WAL audit ring.
- **Finding**: In `aios.hardware.get` and `aios.hardware.verify`, initial parameter length and control character checks occurred prior to entering `recorded_call`.
- **Remediation**: Move all parameter validation inside closure `f`, allowing `dispatch::recorded_call` to record the attempt and return the standard error envelope.

### Scenario MS-2: Strict JSON Schema Typing for Class Filtering
- **Hazard**: Providing non-string elements inside the `classes` array could cause deserialization panics if not strictly validated against string types.
- **Remediation**: `parse_hardware_classes` cleanly checks array elements and returns descriptive `Err` for non-string items or unrecognized classes.

### Scenario MS-3: FIFO / Pseudo-Terminal Denial of Service on File Verification
- **Hazard**: An attacker passing a FIFO (e.g. `/dev/stdin` or named pipe) or special block device to `aios.hardware.verify` could cause single-threaded MCP server hang.
- **Remediation**: In `aios.hardware.verify`, explicitly enforce `path.is_file()` and cap length to 10MB via `metadata()`.

### Scenario MS-4: Target Parameter Traceability
- **Hazard**: Audit events for `aios.hardware.get` should record the requested device ID in `target` field of the audit record.
- **Remediation**: Pass `Some(&dev_id)` as the target parameter to `dispatch::recorded_call`.

---

## 2. Hardening Plan for T-01738
1. Move `device_id` and `file_path` validation inside closure `f` in `call_tool`.
2. Ensure `dispatch::recorded_call` is always called so all queries and rejections are audited.
3. Update `test_hardware_mcp_surface` and smoke tests to verify audit logging of rejections.
