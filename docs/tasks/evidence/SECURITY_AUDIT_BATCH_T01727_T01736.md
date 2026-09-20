# Security Audit: Batch T-01727 to T-01736

## Metadata
- **Batch Range**: `T-01727` .. `T-01736` (10 tasks)
- **Sub-Epics Covered**:
  1. Sub-Epic 3: Hardware Detection CLI Surface (`T-01727`..`T-01730`) — **SUB-EPIC 3 CLOSURE**
  2. Sub-Epic 4: Hardware Detection MCP/API Surface (`T-01731`..`T-01736`) — Implementation & Integration
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor**: Lead Security & Systems Auditor
- **Status**: **PASSED (Zero Open Vulnerabilities)**

---

## 1. Executive Summary
This batch completes and formally seals **Sub-Epic 3 (Hardware Detection CLI Surface)** through security review, hardening against terminal escape injection, documentation, and verification (`T-01727` through `T-01730`). Subsequently, it researches, specifies, scaffolds, implements, unit tests, and cross-surface integrates **Sub-Epic 4 (Hardware Detection MCP/API Surface)** across tasks `T-01731` through `T-01736`.

### Key Security Controls & Hardening Delivered:
1. **Terminal Injection Neutralization (`sanitize_terminal`)**:
   - Passed all human-facing stdout/stderr text across CLI subcommands (`scan`, `list`, `show`, `summary`) through `sanitize_terminal`. Neutralizes ANSI escape sequences that could otherwise manipulate terminal state or spoof success banners.
2. **Device ID Hygiene & Rejection Bounds**:
   - Both CLI and MCP surfaces strictly validate `device_id`:
     - Whitespace-only or empty IDs rejected with `MISSING_DEVICE_ID`.
     - Length capped at 256 characters (`DEVICE_ID_TOO_LONG`).
     - ASCII control characters rejected (`DEVICE_ID_CONTAINS_CONTROL_CHAR`).
3. **MCP Strict Schema Conformity (HM1)**:
   - All 5 tools (`aios.hardware.scan`, `aios.hardware.list`, `aios.hardware.get`, `aios.hardware.summary`, `aios.hardware.verify`) enforce `additionalProperties: false` in their JSON Schema declarations.
4. **Uniform MCP Error & Envelope Standardization (HM2)**:
   - Success responses consistently packaged as `{"ok": true, "tool": "<name>", "data": ...}`.
   - Failures consistently packaged as `{"ok": false, "error": "<msg>"}`.
5. **Cryptographic Audit Ring Integration (HM3, CS4)**:
   - All MCP tool calls route through `dispatch::recorded_call`, persisting structured invocation parameters, actor attribution, and execution outcomes to the SQLite WAL audit ring with SHA-256 hash chaining.
6. **File Buffer & Resource DoS Prevention (CS3)**:
   - File-based offline verification (`--file` / `file_path`) enforces a strict 10MB file ceiling and verifies `is_file()` before in-memory buffering to prevent memory exhaustion or FIFO stalls.
7. **Hermetic Testability & Path Hygiene (HM4, HM5)**:
   - Custom `sysfs_path` and `procfs_path` arguments are bounded to 1,024 characters, tested for control character presence, and allow safe, unprivileged testing on non-Linux or simulated environments.

---

## 2. Task-by-Task Security Analysis

### Task T-01727: Hardware Detection / CLI Surface: Security Review
- **Threat Modeling**: Evaluated attack vectors CS-1 (terminal escape injection), CS-2 (whitespace/empty device ID bypass), CS-3 (unbounded file buffering), and CS-4 (audit ring coverage on early returns).
- **Remediation Plan**: Defined explicit device ID validation, terminal sanitization wrappers, and comprehensive early-exit audit emission.

### Task T-01728: Hardware Detection / CLI Surface: Hardening
- **Implementation**:
  - Added `target_id.trim().is_empty()` check returning code 2 (`MISSING_DEVICE_ID`).
  - Added length check ($\le 256$) and control character check for `target_id`.
  - Applied `sanitize_terminal` to all human output lines across `scan`, `list`, `show`, and `summary`.
- **Verification**: `test_hardware_cli_coverage` expanded and verified passing (18 assertions pass).

### Task T-01729: Hardware Detection / CLI Surface: Documentation
- **Surface Documented**: Added Section 9 to `docs/hardware_detection.md`.
- **Coverage**: Documented CLI syntax, subcommands, options, exit codes, security invariants, and Sub-Epic 3 evidence cross-references.

### Task T-01730: Hardware Detection / CLI Surface: Verification & Evidence (Sub-Epic 3 Closure)
- **Closure Verification**:
  - `cargo test -p aiosh-cli --bin aiosh -- test_hardware_cli_coverage` PASSED (0.63s).
  - `python code/aiosh-cli/tests/test_hardware_cli_smoke.py` PASSED across all test suites.
  - Sub-Epic 3 formally closed.

### Task T-01731: Hardware Detection / MCP/API Surface: Research
- **Research Findings**:
  - Analyzed MCP JSON-RPC 2.0 stdio server architecture in `aiosh-mcp`.
  - Established requirements for 5 hardware tools matching existing core service capabilities.
  - Formulated parameter sanitization and audit emission plan using `dispatch::recorded_call`.

### Task T-01732: Hardware Detection / MCP/API Surface: Specification
- **Formal Specifications**:
  - Formalized invariants HM1..HM5.
  - Defined input schemas, required properties, and output envelopes for `aios.hardware.{scan,list,get,summary,verify}`.

### Task T-01733: Hardware Detection / MCP/API Surface: Scaffold
- **Scaffolding**:
  - Registered the 5 hardware tools in `tools/list` with strict `additionalProperties: false`.
  - Added scaffold match arms in `call_tool`.
  - Confirmed compilation via `cargo check -p aiosh-mcp`.

### Task T-01734: Hardware Detection / MCP/API Surface: Implementation
- **Implementation**:
  - Implemented `resolve_hardware_service` and `parse_hardware_classes` with path and class validation.
  - Implemented business logic for `aios.hardware.scan`, `aios.hardware.list`, `aios.hardware.get`, `aios.hardware.summary`, and `aios.hardware.verify`.
  - Routed all tool calls through `dispatch::recorded_call`.
  - Verified `cargo check -p aiosh-mcp` succeeds.

### Task T-01735: Hardware Detection / MCP/API Surface: Unit Test
- **Test Implementation**:
  - Implemented `test_hardware_mcp_surface` in `aiosh-mcp/src/main.rs`.
  - Covered schema conformity, mock device scan, class filtering, item get, non-existent get, summary parity, live verify, file verify, corrupted file rejection, and path hygiene.
- **Verification**: `cargo test -p aiosh-mcp --bin aiosh-mcp -- test_hardware_mcp_surface` PASSED.

### Task T-01736: Hardware Detection / MCP/API Surface: Integration
- **Integration Test**:
  - Authored `code/aiosh-mcp/tests/test_hardware_mcp_smoke.py`.
  - Verified JSON-RPC stdio protocol execution across advertising, full lifecycle, cross-surface CLI vs MCP parity, and security bounds.
- **Verification**: `python code/aiosh-mcp/tests/test_hardware_mcp_smoke.py` PASSED.

---

## 3. Vulnerability & Invariant Matrix

| Invariant / Control | Target Subsystem | Enforced In | Verification Test | Status |
| :--- | :--- | :--- | :--- | :--- |
| **CS1: Terminal Sanitization** | CLI Surface | `aiosh-cli/src/main.rs` (`sanitize_terminal`) | `test_hardware_cli_coverage` | **PASS** |
| **CS2: Device ID Hygiene** | CLI Surface | `aiosh-cli/src/main.rs` (`cmd_hardware`) | `test_hardware_cli_smoke.py` | **PASS** |
| **CS3: 10MB File Ceiling** | CLI & MCP | CLI `cmd_hardware`, MCP `aios.hardware.verify` | `test_hardware_mcp_surface` | **PASS** |
| **CS4: Audit Emission** | CLI & MCP | `classify_and_emit` / `recorded_call` | `test_hardware_mcp_surface` | **PASS** |
| **HM1: Schema Conformity** | MCP Surface | `tools/list` schema definitions | `test_tools_list_advertising` | **PASS** |
| **HM2: Envelope Uniformity** | MCP Surface | `call_tool` standard envelopes | `test_mcp_hardware_lifecycle` | **PASS** |
| **HM3: Audit Trail** | MCP Surface | `dispatch::recorded_call` | `test_hardware_mcp_smoke.py` | **PASS** |
| **HM4: Parameter Hygiene** | MCP Surface | `resolve_hardware_service`, `get` | `test_security_bounds` | **PASS** |
| **HM5: Hermetic Testability**| MCP Surface | custom `sysfs_path` / `procfs_path` | `test_cross_surface_parity` | **PASS** |

---

## 4. Conclusion & Production Readiness
All 10 tasks in batch `T-01727` through `T-01736` have been rigorously implemented, hardened, verified, and audited. Sub-Epic 3 is formally closed, and Sub-Epic 4 is fully implemented, unit tested, and cross-surface integrated. Zero security vulnerabilities remain. The codebase is fully verified and ready for atomic commit and push to GitHub `origin main`.
