# T-01734: Hardware Detection — MCP/API Surface Implementation

## Metadata
- **Task ID**: `T-01734`
- **Sub-Epic**: Sub-Epic 4: Hardware Detection MCP/API Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Implementation Details

Implemented the full business logic for all 5 hardware detection MCP tools in `code/aiosh-rust/aiosh-mcp/src/main.rs`:

1. **`resolve_hardware_service` & `parse_hardware_classes`**:
   - Path length validation ($\le 1024$ chars) and control character rejection on `sysfs_path` and `procfs_path`.
   - Dynamic instantiation of `HardwareService` with custom sysfs/procfs roots or host defaults.
   - Parsing of `classes` array argument into strongly-typed `DeviceClass` vectors with graceful rejection of unknown classes.
2. **`aios.hardware.scan`**:
   - Executes `service.scan(&options)`.
   - Returns complete `HardwareInventory` JSON payload in standard envelope.
3. **`aios.hardware.list`**:
   - Executes `service.scan(&options)` with optional class filtering.
   - Returns `{"devices": [...], "count": total}`.
4. **`aios.hardware.get`**:
   - Validates `device_id` presence, whitespace trimming, length ($\le 256$), and control characters.
   - Searches inventory for matching `id` and returns `{"device": <HardwareDevice>}` or errors with device not found.
5. **`aios.hardware.summary`**:
   - Discovers inventory and extracts `{"summary": inv.summary, "total": total}`.
6. **`aios.hardware.verify`**:
   - Supports live scan or loading from serialized `--file_path` (with strict 10MB ceiling, existence checks, and non-directory validation).
   - Validates inventory invariants HD1..HD5 via `aiosh_core::validate_hardware_inventory`.
   - Returns `{"valid": true, "device_count": count}` or error with failure diagnostics.

All tools route through `dispatch::recorded_call` ensuring SQLite WAL audit ring persistence.
