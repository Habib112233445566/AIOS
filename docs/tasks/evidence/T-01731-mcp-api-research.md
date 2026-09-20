# T-01731: Hardware Detection — MCP/API Surface Research

## Metadata
- **Task ID**: `T-01731`
- **Sub-Epic**: Sub-Epic 4: Hardware Detection MCP/API Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Architectural Research & Context

### 1.1 MCP Protocol & AIOS Daemon Design
`aiosh-mcp` is the Model Context Protocol (MCP) server for AIOS, exposing userspace subsystem capabilities to AI agents, orchestrators, and automated reasoning tools over JSON-RPC 2.0 stdio.

The server operates via:
- `tools/list`: Advertises available tool contracts with JSON Schema specifications for inputs.
- `tools/call`: Dispatches incoming calls to domain logic, enforcing Policy Enforcement Point (PEP) authorizations and appending cryptographic audit records into the SQLite WAL audit ring.

### 1.2 Subsystem Tool Pattern
Examining neighboring subsystems (`aios.fs_layout.*`, `aios.kernel_module.*`, `aios.package.*`), the standardized MCP pattern entails:
1. Five primary tools corresponding to discovery, listing, item detail lookup, statistical aggregation, and invariant validation:
   - `aios.hardware.scan`
   - `aios.hardware.list`
   - `aios.hardware.get`
   - `aios.hardware.summary`
   - `aios.hardware.verify`
2. Invocation via `dispatch::recorded_call`:
   - Enforces optional/mandatory PEP authorization grants.
   - Automatically writes audit log records.
   - Standardizes response envelopes: `{"ok": true, "tool": "...", "data": ...}` or `{"ok": false, "error": "..."}`.
3. Path and parameter hygiene:
   - Allow mock path injection (`sysfs_path`, `procfs_path`) for hermetic testing in CI and containerized environments.
   - Strict length and control character validation for string arguments (`device_id`, `sysfs_path`, `procfs_path`, `file_path`).

---

## 2. Technical Findings for Implementation
1. **Core Service Reusability**: `aiosh_core::HardwareService`, `HardwareScanOptions`, `DeviceClass`, `HardwareInventory`, and `validate_hardware_inventory` provide complete domain coverage.
2. **Schema Alignment**: Parameter types must match JSON Schema specifications (`string`, `array`, `boolean`, `integer`).
3. **No Unsafe Code**: All parameter parsing and JSON deserialization will remain completely safe Rust.
