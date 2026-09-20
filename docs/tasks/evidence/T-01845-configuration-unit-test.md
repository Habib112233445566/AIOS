# Task Evidence: T-01845 - Network Bootstrap / configuration: Unit Test

## Summary
Implemented and executed comprehensive unit tests for `NetworkConfig` in `code/aiosh-rust/aiosh-core/tests/test_network_config.rs`.

## Unit Test Coverage
1. **Default State**:
   - `test_network_config_default_valid`: Verifies default parameters match specification (`default_store_path`, `sysfs_net_path`, `procfs_path`, `resolv_conf_path`, `max_interfaces=1024`, `max_routes=4096`, `max_dns_servers=32`, `max_payload_bytes=10MB`, `scan_timeout_secs=30`, `fallback_dns_servers=["1.1.1.1", "8.8.8.8"]`).
2. **`NCONF1` (Path Hygiene)**:
   - `test_nconf1_path_hygiene_empty`: Rejects empty or whitespace-only paths.
   - `test_nconf1_path_hygiene_control_chars`: Rejects control characters (`\n`, `\0`, `\t`, `\r`).
   - `test_nconf1_path_hygiene_max_length`: Rejects paths exceeding 1024 characters.
   - `test_nconf1_path_hygiene_traversal`: Rejects parent directory traversal (`..`).
3. **`NCONF2` (Capacity Limits)**:
   - `test_nconf2_capacity_limits_max_interfaces`: Validates `1..=10,000` bounds.
   - `test_nconf2_capacity_limits_max_routes`: Validates `1..=50,000` bounds.
   - `test_nconf2_capacity_limits_max_dns_servers`: Validates `1..=64` bounds.
4. **`NCONF3` (Resource & Timeout Bounds)**:
   - `test_nconf3_resource_bounds_max_payload`: Validates `1024..=104_857_600` bounds.
   - `test_nconf3_timeout_bounds`: Validates `1..=300` seconds bounds.
5. **`NCONF4` (Fallback DNS Validation)**:
   - `test_nconf4_fallback_dns_valid`: Accepts valid IPv4 and IPv6 strings.
   - `test_nconf4_fallback_dns_invalid_ip`: Rejects non-IP strings.
   - `test_nconf4_fallback_dns_empty`: Rejects empty DNS strings.
   - `test_nconf4_fallback_dns_exceeds_max`: Rejects fallback count exceeding `max_dns_servers`.
6. **`NCONF5` (Environment Variable Ingestion)**:
   - `test_nconf5_from_env_valid`: Tests overrides for all paths, limits, and timeouts.
   - `test_nconf5_from_env_invalid_fallback`: Tests safe fallback to defaults when invalid environment values are supplied.
7. **`NCONF6` (Persistence & File Bounds)**:
   - `test_nconf6_json_roundtrip`: Verifies serde serialization/deserialization.
   - `test_nconf6_load_from_path_missing_file`: Verifies non-existent path loads default config.
   - `test_nconf6_save_and_load_roundtrip`: Tests atomic write and reload via `load_from_path` and `from_file`.
   - `test_nconf6_oversized_file_rejected`: Rejects files exceeding `MAX_CONFIG_FILE_BYTES` (1 MB).

## Verification Results
- Test Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_network_config`
- Result: 19 passed; 0 failed; 0 ignored.
