# T-01730: Hardware Detection — CLI Surface Verification & Evidence

## Metadata
- **Task ID**: `T-01730`
- **Sub-Epic**: Sub-Epic 3: Hardware Detection CLI Surface (Closure)
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Verification Overview
This task represents the formal verification and closure of **Sub-Epic 3: Hardware Detection CLI Surface** (`T-01721` through `T-01730`).

All subcommands (`scan`, `list`, `show`, `summary`, `verify`), options (`--class`, `--no-attrs`, `--sysfs`, `--procfs`, `--file`, `--json`), input validation, terminal escape sanitization, and audit logging were comprehensively exercised across Rust and Python test matrices.

---

## 2. Test Execution & Results

### 2.1 Rust In-Tree Unit & Integration Suite
- **Command**: `cargo test -p aiosh-cli --bin aiosh -- test_hardware_cli_coverage`
- **Results**:
  ```text
  running 1 test
  test task_cli_tests::test_hardware_cli_coverage ... ok

  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.63s
  ```
- **Assertions Covered**:
  1. CLI help flag returns exit code 0 and usage documentation.
  2. Unknown subcommand returns exit code 2 with `UNKNOWN_SUBCOMMAND`.
  3. Sysfs path > 1024 chars rejected with exit code 2 and `PATH_TOO_LONG`.
  4. Procfs path containing control characters rejected with exit code 2 and `PATH_CONTAINS_CONTROL_CHAR`.
  5. Invalid device class name rejected with exit code 2 and `INVALID_DEVICE_CLASS`.
  6. Missing device ID on `show` returns exit code 2 with `MISSING_DEVICE_ID`.
  7. Whitespace device ID on `show` returns exit code 2 with `MISSING_DEVICE_ID`.
  8. Control character device ID on `show` returns exit code 2 with `DEVICE_ID_CONTAINS_CONTROL_CHAR`.
  9. Device ID > 256 chars returns exit code 2 with `DEVICE_ID_TOO_LONG`.
  10. Mock sysfs scan succeeds (code 0) and discovers mock PCI GPU device.
  11. Mock sysfs list succeeds (code 0) and outputs device list.
  12. Mock sysfs summary succeeds (code 0) and outputs aggregated counts.
  13. `show` for existing device succeeds (code 0).
  14. `show` for nonexistent device returns code 1 with `DEVICE_NOT_FOUND`.
  15. `verify` against live mock sysfs passes invariants HD1..HD5 (code 0).
  16. Human non-JSON stdout paths for `scan`, `list`, `summary`, `show`, and `verify` execute cleanly through terminal sanitization.
  17. Offline file verification (`--file`) parses valid JSON inventory and passes (code 0).
  18. Corrupted inventory file returns code 1 with `VERIFICATION_FAILED`.

### 2.2 Python External Process Smoke Suite
- **Command**: `python code/aiosh-cli/tests/test_hardware_cli_smoke.py`
- **Results**:
  ```text
  Running Hardware Detection CLI smoke suite with binary: target/debug/aiosh.exe
  PASS: aiosh hw --help and unknown subcommand
  PASS: aiosh hw path hygiene and validation
  PASS: aiosh hw mock subsystems
  ALL TESTS PASSED: aiosh hw CLI smoke test suite.
  ```

---

## 3. Sub-Epic 3 Traceability Matrix
| Task | Title | Status | Artifact Reference |
| :--- | :--- | :--- | :--- |
| `T-01721` | CLI surface: Research | COMPLETED | `T-01721-cli-surface-research.md` |
| `T-01722` | CLI surface: Specification | COMPLETED | `T-01722-cli-surface-specification.md` |
| `T-01723` | CLI surface: Scaffold | COMPLETED | `T-01723-cli-surface-scaffold.md` |
| `T-01724` | CLI surface: Implementation | COMPLETED | `T-01724-cli-surface-implementation.md` |
| `T-01725` | CLI surface: Unit Test | COMPLETED | `T-01725-cli-surface-unit-test.md` |
| `T-01726` | CLI surface: Integration | COMPLETED | `T-01726-cli-surface-integration.md` |
| `T-01727` | CLI surface: Security Review | COMPLETED | `T-01727-cli-surface-security-review.md` |
| `T-01728` | CLI surface: Hardening | COMPLETED | `T-01728-cli-surface-hardening.md` |
| `T-01729` | CLI surface: Documentation | COMPLETED | `T-01729-cli-surface-documentation.md` |
| `T-01730` | CLI surface: Verification & Evidence | COMPLETED | `T-01730-cli-surface-verification-evidenc.md` |
