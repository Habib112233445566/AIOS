# T-01708: Hardware Detection — Data Model Hardening

## Metadata
- **Task ID**: `T-01708`
- **Sub-Epic**: Hardware Detection / Data Model
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Hardening Engineer**: Antigravity Autonomous Agent

---

## 1. Summary of Hardening Controls Implemented
In response to the security review in `T-01707`, strict numerical limits, size caps, and validation boundaries were added to `code/aiosh-rust/aiosh-core/src/hardware.rs`:

1. **Size Caps & Limits**:
   - `MAX_DEVICES = 10_000`: Caps maximum devices per inventory to defeat inventory flood DoS.
   - `MAX_DEVICE_ID_LEN = 128`: Prevents oversized device identifiers.
   - `MAX_DEVICE_NAME_LEN = 256`: Prevents unbounded device name strings.
   - `MAX_ATTRIBUTES_PER_DEVICE = 128`: Limits attributes map to mitigate heap exhaustion attacks.
   - `MAX_ATTRIBUTE_KEY_LEN = 64`: Enforces attribute key length bounds.
   - `MAX_ATTRIBUTE_VAL_LEN = 1024`: Enforces attribute value length bounds.
   - `MAX_PATH_LEN = 512`: Rejects excessively long or malformed filesystem paths.
   - `MAX_JSON_PAYLOAD_SIZE = 10 * 1024 * 1024` (10 MB): Rejects oversized JSON payloads before parsing.

2. **Validation Envelope & Failure Reporting**:
   - All validation methods return standard `Result<(), String>` with descriptive, actionable error strings (never silent failure).
   - Control character filtering across IDs, names, paths, attribute keys, and attribute values.

---

## 2. Test Verification
The unit test suite `code/aiosh-rust/aiosh-core/tests/test_hardware.rs` was expanded with `test_hardening_bounds_and_caps`:
```
running 10 tests
test test_device_class_and_bus_string_mapping ... ok
test test_hardware_device_validation_invalid_hex ... ok
test test_hardware_device_validation_invalid_id ... ok
test test_hardening_bounds_and_caps ... ok
test test_hardware_device_validation_invalid_paths ... ok
test test_hd1_duplicate_device_id_rejection ... ok
test test_hardware_inventory_operations ... ok
test test_hardware_device_validation_valid ... ok
test test_hd3_summary_parity ... ok
test test_hd5_json_roundtrip_and_deterministic_order ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
```

---

## 3. Acceptance Criteria Checklist
- [x] Hardening limits and size caps added across data model structures.
- [x] Negative bounds tests authored and passing standalone.
- [x] Errors reported cleanly in standard result envelope.
- [x] No temp/connection leaks or unbounded memory paths.
