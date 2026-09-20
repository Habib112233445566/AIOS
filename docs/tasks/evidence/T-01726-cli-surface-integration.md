# T-01726: Hardware Detection — CLI Surface Integration

## Metadata
- **Task ID**: `T-01726`
- **Sub-Epic**: Sub-Epic 3: Hardware Detection CLI Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Integration Scope & Changes
Integrated and verified the Hardware Detection CLI operator surface end-to-end:

1. **Binary Wiring & Subcommand Dispatch**:
   - `aiosh hw <scan|list|show|summary|verify>` wired to `cmd_hardware` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Re-exports from `aiosh_core` utilized across all subcommands.
   - Verified that both `aiosh hw` and `aiosh hardware` dispatch accurately.

2. **Integration Test Suite**:
   - Implemented `code/aiosh-cli/tests/test_hardware_cli_smoke.py` verifying real binary execution against:
     - `aiosh hw --help` (exit code 0, all tokens verified).
     - Unknown subcommand handling (exit code 2, envelope error `UNKNOWN_SUBCOMMAND`).
     - Path hygiene enforcement: `--sysfs` oversized length cap (>1024 chars), control-character rejection (`PATH_CONTAINS_CONTROL_CHAR`).
     - Class filtering validation: rejection of unrecognized classes (`INVALID_DEVICE_CLASS`).
     - Subcommand parameter validation: missing device ID on `show` (`MISSING_DEVICE_ID`).
     - Mock sysfs/procfs environment integration:
       - `scan`: verifies full scan, `--class gpu` filtering, and `--no-attrs` attribute suppression.
       - `list`: verifies device table / array formatting.
       - `summary`: verifies categorical count breakdown.
       - `show`: verifies lookup of existing device and rejection of nonexistent device (`DEVICE_NOT_FOUND`).
       - `verify`: verifies live scan validation, valid file validation, and corrupted file rejection (`VERIFICATION_FAILED`).

---

## 2. Test Execution Verification

### Python Integration Smoke Suite
```bash
python code/aiosh-cli/tests/test_hardware_cli_smoke.py
```
Output:
```
Running Hardware Detection CLI smoke suite with binary: ...\aiosh.exe
PASS: aiosh hw --help and unknown subcommand
PASS: aiosh hw path hygiene and validation
PASS: aiosh hw mock subsystems
ALL TESTS PASSED: aiosh hw CLI smoke test suite.
```

### Full Hardware Detection Test Matrix
- `test_hardware_model_smoke.py`: PASS
- `test_hardware_service_smoke.py`: PASS
- `test_hardware_cli_smoke.py`: PASS

---

## 3. Invariants Verification Matrix (HC1..HC5)

| Invariant | Name | Guarantee & Enforcement | Verification Test | Status |
| :--- | :--- | :--- | :--- | :--- |
| **HC1** | JSON Envelope | Standard `{code, data, error}` format | `test_hardware_cli_smoke.py` | **PASS** |
| **HC2** | Audit Emission | Every invocation recorded to WAL ring | Internal SQLite test assertions | **PASS** |
| **HC3** | Exit Code Discipline | 0 = Success, 1 = Domain Error, 2 = Usage Error | `test_hardware_cli_smoke.py` | **PASS** |
| **HC4** | Filter & Attr Control | Accurate `--class` filter & `--no-attrs` stripping | `test_hw_mock_subsystems` | **PASS** |
| **HC5** | Path Sanitization | Reject oversized paths and control chars | `test_hw_path_hygiene_and_validation` | **PASS** |

---

## 4. Acceptance Criteria Checklist
- [x] Full binary integration verified.
- [x] End-to-end smoke test passing (100%).
- [x] Invariants HC1..HC5 verified.
- [x] Zero regressions across other CLI subcommands.
