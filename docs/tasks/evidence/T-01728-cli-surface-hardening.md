# T-01728: Hardware Detection — CLI Surface Hardening

## Metadata
- **Task ID**: `T-01728`
- **Sub-Epic**: Sub-Epic 3: Hardware Detection CLI Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Hardening Actions Implemented

### 1.1 Device ID Hygiene and Validation (Remediating CS-2)
- Added strict trimming and non-empty check for `target_id`:
  - `Some(id) if !id.trim().is_empty() => id.trim()`
  - Whitespace-only or empty device ID returns exit code 2 with error code `MISSING_DEVICE_ID`.
- Added maximum length check (`target_id.len() <= 256`):
  - Returns exit code 2 with `DEVICE_ID_TOO_LONG`.
- Added control character validation (`target_id.chars().any(|c| c.is_control())`):
  - Returns exit code 2 with `DEVICE_ID_CONTAINS_CONTROL_CHAR`.
- Full audit event emission through `classify_and_emit` on all early-exit branches.

### 1.2 Terminal Injection Sanitization (Remediating CS-1)
- Wrapped all human stdout outputs in `sanitize_terminal(...)`:
  - `aiosh hw scan`: summary category labels sanitized.
  - `aiosh hw list`: device ID, driver name, and device description sanitized.
  - `aiosh hw show`: device name, ID, vendor ID, device ID, vendor name, driver, sysfs path, dev path, and all attribute keys and values sanitized.
  - `aiosh hw summary`: category labels sanitized.

### 1.3 Test Suite Expansion
- Updated `test_hardware_cli_coverage` in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - Asserted exit code 2 for whitespace-only device ID (`show "   "`).
  - Asserted exit code 2 for control characters in device ID (`show "bad\x07dev"`).
  - Asserted exit code 2 for device ID exceeding 256 characters (`show x*257`).
  - Exercised human non-JSON stdout paths for `scan`, `list`, `summary`, `show`, and `verify`.
