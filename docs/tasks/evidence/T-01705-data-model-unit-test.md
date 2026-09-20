# T-01705: Hardware Detection — Data Model Unit Test

## Metadata
- **Task ID**: `T-01705`
- **Sub-Epic**: Hardware Detection / Data Model
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Test Engineer**: Antigravity Autonomous Agent

---

## 1. Test Suite Implementation
Implemented comprehensive unit test suite in `code/aiosh-rust/aiosh-core/tests/test_hardware.rs` covering:
1. **String Mapping & Parsing**:
   - `test_device_class_and_bus_string_mapping`: Tests bidirectional conversion for `DeviceClass` and `DeviceBus` including fuzzy/case-insensitive `from_str_loose()`.
2. **Device Validation & Builders**:
   - `test_hardware_device_validation_valid`: Verifies happy path for fully configured device builder with vendor, device, driver, and sysfs attributes.
3. **Negative Input Cases**:
   - `test_hardware_device_validation_invalid_id`: Asserts rejection of empty IDs, whitespace IDs, and control characters in IDs.
   - `test_hardware_device_validation_invalid_hex`: Asserts rejection of malformed vendor/device hex IDs (short, long, non-hex).
   - `test_hardware_device_validation_invalid_paths`: Asserts rejection of traversal paths (`..`) and control characters.
4. **Inventory State Operations**:
   - `test_hardware_inventory_operations`: Asserts addition, retrieval, classification filtering, bus filtering, summary computation, and device removal.
5. **Invariant Enforcement**:
   - `test_hd1_duplicate_device_id_rejection`: Asserts rejection of colliding device IDs.
   - `test_hd3_summary_parity`: Asserts error when device class counts diverge from summary.
   - `test_hd5_json_roundtrip_and_deterministic_order`: Asserts lossless JSON serialization and deserialization.

---

## 2. Test Execution Output
Executed `cargo test -p aiosh-core --test test_hardware`:
```
running 9 tests
test test_hardware_device_validation_invalid_hex ... ok
test test_device_class_and_bus_string_mapping ... ok
test test_hardware_device_validation_invalid_paths ... ok
test test_hardware_device_validation_invalid_id ... ok
test test_hardware_device_validation_valid ... ok
test test_hd1_duplicate_device_id_rejection ... ok
test test_hd3_summary_parity ... ok
test test_hd5_json_roundtrip_and_deterministic_order ... ok
test test_hardware_inventory_operations ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## 3. Acceptance Criteria Checklist
- [x] Dedicated unit test file `tests/test_hardware.rs` created.
- [x] Negative, boundary, and invariant cases asserted.
- [x] Standalone test runs green (9/9 passing).
- [x] Invariants HD1 through HD5 fully covered.
