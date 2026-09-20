# T-01735: Hardware Detection — MCP/API Surface Unit Test

## Metadata
- **Task ID**: `T-01735`
- **Sub-Epic**: Sub-Epic 4: Hardware Detection MCP/API Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Test Suite Implementation
Implemented `test_hardware_mcp_surface` in `code/aiosh-rust/aiosh-mcp/src/main.rs`:

1. **Tool Schema Conformity (HM1)**:
   - Verified that `aios.hardware.scan`, `aios.hardware.list`, `aios.hardware.get`, `aios.hardware.summary`, and `aios.hardware.verify` are returned by `tool_manifest()`.
   - Verified that every hardware tool sets `additionalProperties: false` on its `inputSchema`.
2. **`aios.hardware.scan` Execution**:
   - Tested discovery against mock sysfs root containing a mock PCI GPU (`vendor: 0x10de`, `device: 0x2684`, `class: 0x030000`).
   - Verified `data.devices[0].id == "pci:0000:01:00.0"` and `data.summary.gpu == 1`.
3. **`aios.hardware.list` Execution & Filtering**:
   - Verified unfiltered device count.
   - Verified class filtering: `classes: ["gpu"]` returns 1 device; `classes: ["block"]` returns 0 devices.
4. **`aios.hardware.get` Execution**:
   - Verified lookup of existing device (`pci:0000:01:00.0`).
   - Verified error return for non-existent device (`pci:nonexistent`).
   - Verified rejection of whitespace-only device ID (`"   "`).
5. **`aios.hardware.summary` Execution**:
   - Verified summary counts per class (`summary.gpu == 1`, `total == 1`).
6. **`aios.hardware.verify` Execution**:
   - Live scan validation passes HD1..HD5 (`data.valid == true`).
   - File-based validation with valid JSON passes.
   - File-based validation with corrupted JSON fails (`ok == false`).
7. **Parameter & Path Hygiene (HM4)**:
   - Sysfs path > 1024 characters rejected.
   - Procfs path containing ASCII control characters rejected.
   - Unrecognized device class rejected.

## 2. Test Execution Output
```text
running 1 test
test tests::test_hardware_mcp_surface ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 1.75s
```
