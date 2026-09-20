# Task Evidence: T-01850 - Network Bootstrap / configuration: Verification & Evidence

## 1. Overview
- **Task ID**: `T-01850`
- **Sub-Epic**: 5 (Network Bootstrap Configuration Subsystem — Closure)
- **Goal**: Formally verify all configuration subsystem requirements and close Sub-Epic 5.

---

## 2. Verification Suites Executed

### A. Rust Unit Test Suite (`aiosh-core`)
- **Command**: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_network_config`
- **Output**:
  ```text
  running 20 tests
  test test_nconf1_path_hygiene_max_length ... ok
  test test_nconf1_path_hygiene_control_chars ... ok
  test test_nconf1_path_hygiene_traversal ... ok
  test test_nconf1_path_hygiene_empty ... ok
  test test_nconf2_capacity_limits_max_dns_servers ... ok
  test test_nconf2_capacity_limits_max_interfaces ... ok
  test test_nconf2_capacity_limits_max_routes ... ok
  test test_nconf3_resource_bounds_max_payload ... ok
  test test_nconf3_timeout_bounds ... ok
  test test_nconf4_fallback_dns_empty ... ok
  test test_nconf4_fallback_dns_exceeds_max ... ok
  test test_nconf4_fallback_dns_invalid_ip ... ok
  test test_nconf4_fallback_dns_valid ... ok
  test test_nconf5_from_env_invalid_fallback ... ok
  test test_nconf5_from_env_valid ... ok
  test test_nconf6_load_from_path_missing_file ... ok
  test test_nconf6_json_roundtrip ... ok
  test test_network_config_default_valid ... ok
  test test_nconf6_oversized_file_rejected ... ok
  test test_nconf6_save_and_load_roundtrip ... ok

  test result: ok. 20 passed; 0 failed; 0 ignored; finished in 0.03s
  ```

### B. Python Cross-Surface Integration Smoke Suite (`aiosh-cli`)
- **Command**: `python code/aiosh-cli/tests/test_network_config_smoke.py`
- **Output**:
  ```text
  Running Network Bootstrap Configuration Smoke Tests (T-01846)...
  PASS: test_nconf1_path_hygiene
  PASS: test_nconf2_capacity_limits
  PASS: test_nconf3_resource_bounds
  PASS: test_nconf4_fallback_dns
  PASS: test_nconf5_environment_ingestion
  PASS: test_nconf6_persistence
  ALL NETWORK CONFIG SMOKE TESTS PASSED.
  ```

---

## 3. Sub-Epic 5 Closure
- All tasks in Sub-Epic 5 (`T-01841` through `T-01850`) are verified, hardened, documented, and fully tested.
- Invariants `NCONF1..NCONF6` are strictly enforced across both Rust and Python surfaces.
- Sub-Epic 5 is formally **CLOSED**.
