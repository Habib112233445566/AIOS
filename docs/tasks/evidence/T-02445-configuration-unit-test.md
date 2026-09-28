# T-02445: Sandbox Enforcement Configuration Unit Test

## 1. Overview
This document records verification evidence for task T-02445: Sandbox Enforcement Configuration Unit Test.
Focused automated tests were authored in `code/aiosh-rust/aiosh-core/tests/test_sandbox_config.rs`.

## 2. Test Cases Executed
1. `test_config_default_and_validation`: Verifies factory defaults, version constraints, and validation pass.
2. `test_config_json_roundtrip`: Verifies lossless serde serialization and deserialization.
3. `test_config_bounds_enforcement`: Tests rejection of invalid versions, blank profile names, output capture bounds (1 KiB..64 MiB), timeouts (1s..86400s), profile capacity (1..1024), and path traversal (`..`) in `custom_profiles_dir`.
4. `test_config_file_persistence`: Verifies atomic directory creation and roundtrip disk persistence.
5. `test_config_file_size_limit`: Asserts fail-closed rejection of files exceeding `MAX_CONFIG_FILE_BYTES` (64 KiB).
6. `test_config_env_overrides`: Validates environment variable overrides (`AIOS_SANDBOX_*`).

## 3. Results
```text
running 6 tests
test test_config_env_overrides ... ok
test test_config_default_and_validation ... ok
test test_config_bounds_enforcement ... ok
test test_config_file_persistence ... ok
test test_config_file_size_limit ... ok
test test_config_json_roundtrip ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```
