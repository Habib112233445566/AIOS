# T-01716: Hardware Detection — Core Service Integration

## Metadata
- **Task ID**: `T-01716`
- **Sub-Epic**: Hardware Detection / Core Service
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Integration Scope & Changes
1. **Public Re-Exports**:
   - Re-exported core hardware detection types and services from crate root in `code/aiosh-rust/aiosh-core/src/lib.rs`:
     - `HardwareService`, `HardwareScanOptions`
     - `HardwareDevice`, `HardwareInventory`, `DeviceClass`, `validate_hardware_inventory`
     - Hardening constants: `MAX_DEVICES`, `MAX_ATTRIBUTES_PER_DEVICE`, `MAX_ATTRIBUTE_KEY_LEN`, `MAX_ATTRIBUTE_VAL_LEN`, `MAX_DEVICE_ID_LEN`, `MAX_DEVICE_NAME_LEN`, `MAX_PATH_LEN`, `MAX_JSON_PAYLOAD_SIZE`.
   - Added `validate_hardware_inventory(inv: &HardwareInventory) -> Result<(), String>` in `hardware.rs` for crate-wide validation parity.

2. **Integration Test Suite**:
   - Implemented `code/aiosh-rust/aiosh-core/tests/test_hardware_integration.rs`:
     - `test_hardware_service_crate_root_integration`: Validates integration of all 6 device classes (PCI, USB, Block, Net, CPU, System) through crate root API, asserting invariants HS2 (Class Accuracy), HS3 (Deterministic Ordering), HS4 (Hex Normalization), and HS5 (Inventory Validity).
     - `test_hardware_service_hs1_missing_roots_fallback`: Validates HS1 (Graceful Degradation) when targeting non-existent sysfs/procfs roots.
     - `test_hardware_service_hs2_class_isolation`: Validates filtering by specific `DeviceClass` (Network only).
   - Implemented `code/aiosh-cli/tests/test_hardware_service_smoke.py`:
     - Cross-platform Python smoke test validating HS1 (Fallback), HS2 (PCI Class mapping), HS3 (Sorting), HS4 (Hex normalization & path safety), and HS5 (JSON serialization roundtrip).

---

## 2. Test Execution Verification

### Rust Integration Tests
```bash
cargo test -p aiosh-core --test test_hardware_integration
```
Output:
```
running 3 tests
test test_hardware_service_hs1_missing_roots_fallback ... ok
test test_hardware_service_hs2_class_isolation ... ok
test test_hardware_service_crate_root_integration ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

### Python Integration Smoke Suite
```bash
python code/aiosh-cli/tests/test_hardware_service_smoke.py
```
Output:
```
Running Hardware Detection Core Service integration smoke suite...
PASS: test_hs1_missing_sysfs_resilience
PASS: test_hs2_class_mapping_and_resolution
PASS: test_hs3_deterministic_sorting
PASS: test_hs4_sanitization_and_normalization
PASS: test_hs5_inventory_validation_roundtrip
ALL HARDWARE DETECTION CORE SERVICE INTEGRATION TESTS PASSED.
```

---

## 3. Invariants Verification Matrix
| Invariant | Description | Verification Method | Status |
|---|---|---|---|
| **HS1** | Graceful fallback on missing roots | `test_hardware_service_hs1_missing_roots_fallback` | **PASS** |
| **HS2** | Accurate class identification | `test_hardware_service_hs2_class_isolation` | **PASS** |
| **HS3** | Deterministic device ID sorting | `test_hardware_service_crate_root_integration` | **PASS** |
| **HS4** | Hex normalization & path safety | `test_hs4_sanitization_and_normalization` | **PASS** |
| **HS5** | Inventory validity (HD1..HD5) | `validate_hardware_inventory` assertion | **PASS** |

---

## 4. Acceptance Criteria Checklist
- [x] Public API re-exports in `aiosh_core` root.
- [x] Full integration test suite in Rust (`test_hardware_integration.rs`).
- [x] Python integration smoke test (`test_hardware_service_smoke.py`).
- [x] Invariants HS1..HS5 fully verified.
- [x] 100% test pass rate across all suites.
