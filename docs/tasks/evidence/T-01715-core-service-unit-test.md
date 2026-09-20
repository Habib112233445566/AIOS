# T-01715: Hardware Detection — Core Service Unit Test

## Metadata
- **Task ID**: `T-01715`
- **Sub-Epic**: Hardware Detection / Core Service
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Test Architecture & Coverage
Authored comprehensive unit tests in `code/aiosh-rust/aiosh-core/tests/test_hardware_service.rs` validating `HardwareService` discovery and filtering:

1. **`test_hardware_service_mock_sysfs_scan`**:
   - Synthesizes an isolated temporary sysfs & procfs structure containing:
     - PCI device (`0000:00:02.0` GPU) with vendor `0x8086`, device `0x9bc4`, class `0x030000`, driver `i915`.
     - USB device (`1-1`) with vendor `0x046d`, product `0xc52b`, manufacturer `Logitech`.
     - Block device (`sda`) with rotational `0`, removable `0`, size `1000215216`, model `Samsung SSD 970`.
     - Net device (`eth0`) with address `52:54:00:12:34:56`, operstate `up`, speed `1000`.
     - DMI system (`dmi/id`) with sys_vendor `Framework`, product_name `Laptop 13`, bios_version `03.04`.
     - CPU `/proc/cpuinfo` with model name `Intel(R) Core(TM) i7-1165G7`, 8 processors.
   - Asserts discovery of all 6 device classes.
   - Asserts all invariants HD1..HD5 hold on generated `HardwareInventory`.
   - Validates driver resolution and attribute mappings.

2. **`test_hardware_service_class_filtering`**:
   - Tests `HardwareScanOptions` class filtering requesting only `DeviceClass::Gpu` and `DeviceClass::Network`.
   - Asserts inventory strictly contains only GPU and Network devices, excluding USB, Block, CPU, and System.

3. **`test_hardware_service_attribute_stripping`**:
   - Tests `HardwareScanOptions` with `include_attributes: false`.
   - Asserts all discovered devices have empty `attributes` maps while retaining essential metadata (ID, class, vendor, model, driver).

4. **`test_hardware_service_empty_sysfs_resilience`**:
   - Executes `HardwareService` against an empty root directory without sysfs or procfs hierarchies.
   - Asserts graceful execution returning an empty inventory without crashing, panicking, or returning errors.

---

## 2. Test Execution Verification
Command:
```bash
cargo test -p aiosh-core --test test_hardware_service
```

Output:
```
running 4 tests
test test_hardware_service_empty_sysfs_resilience ... ok
test test_hardware_service_attribute_stripping ... ok
test test_hardware_service_class_filtering ... ok
test test_hardware_service_mock_sysfs_scan ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

Full `aiosh-core` regression test:
```bash
cargo test -p aiosh-core
```
Output: All 80+ test cases across storage, kernel, service, session, and hardware suites PASSED (0 failures).

---

## 3. Acceptance Criteria Checklist
- [x] Hermetic mock sysfs test verifying end-to-end multi-class hardware detection.
- [x] Class filtering options verified.
- [x] Attribute suppression options verified.
- [x] Empty directory resilience verified.
- [x] 100% test pass rate with 0 regressions.
