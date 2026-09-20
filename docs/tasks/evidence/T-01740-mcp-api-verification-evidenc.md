# T-01740: Hardware Detection — MCP/API Surface Verification & Evidence

## Metadata
- **Task ID**: `T-01740`
- **Sub-Epic**: Sub-Epic 4: Hardware Detection MCP/API Surface (Closure)
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Verification Overview
This task represents the formal verification and closure of **Sub-Epic 4: Hardware Detection MCP/API Surface** (`T-01731` through `T-01740`).

All 5 MCP tools (`aios.hardware.scan`, `aios.hardware.list`, `aios.hardware.get`, `aios.hardware.summary`, `aios.hardware.verify`), parameter parsing, schema conformity (`additionalProperties: false`), standard envelopes, in-closure audit emission, and security sanitization were comprehensively verified across Rust and Python test matrices.

---

## 2. Test Execution & Results

### 2.1 Rust In-Tree Unit Test Suite
- **Command**: `cargo test -p aiosh-mcp --bin aiosh-mcp -- test_hardware_mcp_surface`
- **Results**:
  ```text
  running 1 test
  test tests::test_hardware_mcp_surface ... ok

  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.13s
  ```
- **Assertions Verified**:
  1. `tools/list` returns all 5 hardware tools, each with `additionalProperties: false` on `inputSchema`.
  2. `aios.hardware.scan` against mock sysfs discovers mock PCI device (`0000_01_00.0` -> `pci:0000:01:00.0`) and calculates correct summary counts.
  3. `aios.hardware.list` returns full device list and count.
  4. `aios.hardware.list` with `classes: ["gpu"]` filters to 1 device; `classes: ["block"]` filters to 0 devices.
  5. `aios.hardware.get` retrieves details for `pci:0000:01:00.0`.
  6. `aios.hardware.get` refuses non-existent device ID (`pci:nonexistent`) and whitespace-only device ID.
  7. `aios.hardware.summary` returns aggregate counts matching inventory.
  8. `aios.hardware.verify` live scan passes invariants HD1..HD5.
  9. `aios.hardware.verify` with valid JSON file passes; corrupted file is rejected.
  10. Path hygiene: `sysfs_path` > 1024 chars rejected, `procfs_path` with control chars rejected, invalid class rejected.

### 2.2 Python External Process Smoke Suite
- **Command**: `python code/aiosh-mcp/tests/test_hardware_mcp_smoke.py`
- **Results**:
  ```text
  Running Hardware Detection MCP smoke suite with binary: code/aiosh-rust/target/debug/aiosh-mcp.exe
  PASS: test_tools_list_advertising (all 5 tools advertised)
  PASS: test_mcp_hardware_lifecycle (all 5 tools operational)
  PASS: test_cross_surface_parity (CLI and MCP return identical state)
  PASS: test_security_bounds (all boundary violations refused)
  ALL TESTS PASSED: aios.hardware MCP smoke test suite.
  ```

---

## 3. Sub-Epic 4 Traceability Matrix
| Task | Title | Status | Artifact Reference |
| :--- | :--- | :--- | :--- |
| `T-01731` | MCP/API surface: Research | COMPLETED | `T-01731-mcp-api-research.md` |
| `T-01732` | MCP/API surface: Specification | COMPLETED | `T-01732-mcp-api-specification.md` |
| `T-01733` | MCP/API surface: Scaffold | COMPLETED | `T-01733-mcp-api-scaffold.md` |
| `T-01734` | MCP/API surface: Implementation | COMPLETED | `T-01734-mcp-api-implementation.md` |
| `T-01735` | MCP/API surface: Unit Test | COMPLETED | `T-01735-mcp-api-unit-test.md` |
| `T-01736` | MCP/API surface: Integration | COMPLETED | `T-01736-mcp-api-integration.md` |
| `T-01737` | MCP/API surface: Security Review | COMPLETED | `T-01737-mcp-api-security-review.md` |
| `T-01738` | MCP/API surface: Hardening | COMPLETED | `T-01738-mcp-api-hardening.md` |
| `T-01739` | MCP/API surface: Documentation | COMPLETED | `T-01739-mcp-api-documentation.md` |
| `T-01740` | MCP/API surface: Verification & Evidence | COMPLETED | `T-01740-mcp-api-verification-evidenc.md` |
