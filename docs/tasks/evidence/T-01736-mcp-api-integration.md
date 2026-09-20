# T-01736: Hardware Detection — MCP/API Surface Integration

## Metadata
- **Task ID**: `T-01736`
- **Sub-Epic**: Sub-Epic 4: Hardware Detection MCP/API Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Cross-Surface Integration Suite
Authored and executed `code/aiosh-mcp/tests/test_hardware_mcp_smoke.py`, proving full end-to-end integration of the 5 Hardware Detection MCP tools over JSON-RPC stdio.

### 1.1 Tool Advertising Verification
- Dispatched `tools/list` request to `aiosh-mcp`.
- Verified advertising of:
  - `aios.hardware.scan`
  - `aios.hardware.list`
  - `aios.hardware.get`
  - `aios.hardware.summary`
  - `aios.hardware.verify`
- Confirmed JSON Schema definitions with `additionalProperties: false`.

### 1.2 Lifecycle Test
- Created hermetic mock sysfs environment with mock PCI GPU (`0000_01_00.0`, vendor `0x10de`, device `0x2684`, class `0x030000`).
- Tested `aios.hardware.scan`: discovered device and verified `gpu` category count.
- Tested `aios.hardware.list`: verified unfiltered and class-filtered queries (`gpu` count 1, `block` count 0).
- Tested `aios.hardware.get`: verified item lookup for `pci:0000:01:00.0` and error on non-existent device.
- Tested `aios.hardware.summary`: verified summary parity.
- Tested `aios.hardware.verify`: verified live scan passes invariants HD1..HD5; verified valid JSON file passes; verified corrupted file is refused.

### 1.3 Cross-Surface Parity
- Compared operator CLI (`aiosh hw scan` / `aiosh hw show`) with MCP tools (`aios.hardware.scan` / `aios.hardware.get`) over the same mock environment.
- Verified byte and field parity across both substrates.

### 1.4 Security Boundary Enforcement
- Tested rejection of paths exceeding 1024 bytes.
- Tested rejection of paths with ASCII control characters (`\x07`).
- Tested rejection of empty/whitespace-only `device_id`.
- Tested rejection of `device_id` containing control characters or exceeding 256 characters.
- Tested rejection of unrecognized device classes.

## 2. Test Execution Output
```text
Running Hardware Detection MCP smoke suite with binary: code/aiosh-rust/target/debug/aiosh-mcp.exe
PASS: test_tools_list_advertising (all 5 tools advertised)
PASS: test_mcp_hardware_lifecycle (all 5 tools operational)
PASS: test_cross_surface_parity (CLI and MCP return identical state)
PASS: test_security_bounds (all boundary violations refused)
ALL TESTS PASSED: aios.hardware MCP smoke test suite.
```
