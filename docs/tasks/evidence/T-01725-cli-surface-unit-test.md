# T-01725: Hardware Detection — CLI Surface Unit Test

## Metadata
- **Task ID**: `T-01725`
- **Sub-Epic**: Sub-Epic 3: Hardware Detection CLI Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Test Architecture & Coverage
Authored comprehensive unit tests in `code/aiosh-rust/aiosh-cli/src/main.rs` (`test_hardware_cli_coverage`):

1. **Usage & Subcommand Syntax**:
   - `aiosh hw --help` returns exit code 0.
   - Unknown subcommand `bogus_cmd` returns exit code 2 and standard error envelope (`UNKNOWN_SUBCOMMAND`).

2. **Path Hygiene Enforcement**:
   - Oversized path (>1024 characters) rejected with exit code 2 and `PATH_TOO_LONG`.
   - Control character injection (e.g. `\x00` in `--procfs`) rejected with exit code 2 and `PATH_CONTAINS_CONTROL_CHAR`.

3. **Classification & Validation Constraints**:
   - Unrecognized device class (e.g. `--class badclass`) rejected with exit code 2 and `INVALID_DEVICE_CLASS`.
   - Missing required positional device ID on `show` returns exit code 2 and `MISSING_DEVICE_ID`.

4. **Hermetic Mock Scanning & Subcommands**:
   - `aiosh hw scan --sysfs <mock> --procfs <mock> --json`: returns exit code 0 and valid inventory.
   - `aiosh hw list --sysfs <mock> --procfs <mock> --json`: returns exit code 0 and device array.
   - `aiosh hw summary --sysfs <mock> --procfs <mock> --json`: returns exit code 0 and count summary.
   - `aiosh hw show <id>` on existing device: returns exit code 0 and device details.
   - `aiosh hw show <id>` on nonexistent device: returns exit code 1 and `DEVICE_NOT_FOUND`.
   - `aiosh hw verify`: returns exit code 0 on valid live inventory.
   - `aiosh hw verify --file <valid>`: returns exit code 0.
   - `aiosh hw verify --file <corrupted>`: returns exit code 1 and `VALIDATION_FAILED`.

---

## 2. Test Execution Verification
Command:
```bash
cargo test -p aiosh-cli --bin aiosh -- test_hardware_cli_coverage
```
Output:
```
running 1 test
test task_cli_tests::test_hardware_cli_coverage ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.75s
```
Status: **PASS (100%)**.
