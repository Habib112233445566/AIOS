# T-01745: Hardware Detection — Configuration Unit Test

## Metadata
- **Task ID**: `T-01745`
- **Sub-Epic**: Sub-Epic 5: Hardware Detection Configuration
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Test Coverage Overview
Implemented unit tests in `code/aiosh-rust/aiosh-core/tests/test_hardware_config.rs` covering all invariants defined in `T-01742` (HCFG1..HCFG5):

1. `test_hardware_config_default_valid`: Verifies default configuration fields and valid validation status.
2. `test_hcfg1_path_hygiene_empty`: Verifies rejection of empty or whitespace paths for store, sysfs, and procfs.
3. `test_hcfg1_path_hygiene_control_chars`: Verifies rejection of control characters (`\n`, `\0`, `\t`) in paths.
4. `test_hcfg1_path_hygiene_max_length`: Verifies rejection of path strings exceeding 1024 characters.
5. `test_hcfg2_class_filtering_valid`: Verifies acceptance of valid non-duplicate `DeviceClass` lists.
6. `test_hcfg2_class_filtering_duplicate`: Verifies rejection of duplicate device classes in filter list.
7. `test_hcfg2_class_filtering_max_count`: Verifies rejection when class list exceeds maximum allowed count (9).
8. `test_hcfg3_resource_bounds_max_devices`: Verifies bounds checking for `max_devices` ($1 \le n \le 50,000$).
9. `test_hcfg3_resource_bounds_max_payload`: Verifies bounds checking for `max_payload_bytes` ($1024 \le n \le 104,857,600$).
10. `test_hcfg4_timeout_bounds`: Verifies bounds checking for `scan_timeout_secs` ($1 \le n \le 300$).
11. `test_hcfg5_json_roundtrip`: Verifies lossless serialization and deserialization of `HardwareConfig`.
12. `test_hcfg5_load_from_path_missing_file`: Verifies fallback to `HardwareConfig::default()` when config file does not exist.
13. `test_hcfg5_save_and_load_roundtrip`: Verifies atomic directory creation, file persistence, and reload fidelity.
14. `test_hardware_config_from_env`: Verifies environment variable override parsing for all supported configuration options.
