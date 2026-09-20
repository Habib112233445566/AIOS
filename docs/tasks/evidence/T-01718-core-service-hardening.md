# T-01718: Hardware Detection — Core Service Hardening

## Metadata
- **Task ID**: `T-01718`
- **Sub-Epic**: Hardware Detection / Core Service
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Hardening Architecture & Implemented Controls

Following the findings of `T-01717`, the following security hardening controls were implemented in `code/aiosh-rust/aiosh-core/src/hardware_service.rs`:

1. **Bounded I/O with `std::io::Read::take(1024)`**:
   - `read_trimmed_file` now uses `file.take(1024).read_to_end(&mut buffer)` directly at the OS byte stream level.
   - Prevents virtual sysfs stream exhaustion, memory allocation loops, and pseudo-file denial-of-service.

2. **Control Character & Null Byte Sanitization**:
   - Strips non-printable ASCII control characters (`!c.is_ascii_control() || c == '\t' || c == '\n' || c == '\r'`).
   - Prevents null-byte injection (`\0`), terminal escape injection (`\x1b`), and BELL (`\x07`) corruption.

3. **Driver Identifier Sanitization & Whitelisting**:
   - `resolve_driver_name` validates that resolved driver names are non-empty, capped at 128 characters, and strictly composed of ASCII alphanumeric, underscore, hyphen, or dot (`[a-zA-Z0-9_.-]+`).
   - Traversal attempts (such as `../../evil_driver`) or paths containing illegal characters are rejected, returning `None`.

---

## 2. Test Verification
Implemented `test_hardware_service_hardening_bounds` in `tests/test_hardware_service.rs`:
- Tested oversized file padding (>2048 bytes), verifying truncation and successful parsing.
- Tested embedded null bytes and BEL control characters, verifying clean stripping.
- Tested path traversal attempts in driver symlinks, verifying rejection.

Command:
```bash
cargo test -p aiosh-core --test test_hardware_service
```
Result:
```
running 5 tests
test test_hardware_service_empty_sysfs_resilience ... ok
test test_hardware_service_attribute_stripping ... ok
test test_hardware_service_class_filtering ... ok
test test_hardware_service_hardening_bounds ... ok
test test_hardware_service_mock_sysfs_scan ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```
Status: **PASS (100%)**.
