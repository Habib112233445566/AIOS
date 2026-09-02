# T-00655 — Repository Health / configuration: Unit Test

## 1. Unit Test Scope
This task tests `RepoHealthConfig` across schema validation, default creation, deserialization, and path reading.

## 2. Test Execution & Coverage
1. **`test_repo_health_config_default_and_roundtrip`**:
   - Asserts default struct properties.
   - Asserts JSON roundtrip preserves exact values.
2. **`test_repo_health_config_validation_errors`**:
   - Asserts empty versions are rejected.
   - Asserts invalid `max_file_bytes` (< 1024) are rejected.
   - Asserts empty `ignored_dirs` and directory paths with `..` or path separators are rejected.
   - Asserts `security_policy_path` with `..` is rejected.
   - Asserts `min_security_policy_bytes == 0` is rejected.
3. **`test_repo_health_config_from_path`**:
   - Asserts file reading from custom temporary JSON file.

## 3. Test Verification Output
```text
PASS: cargo test repo_health_config::tests (3/3 tests passed)
PASS: config schema definition & safety assertions

ALL REPO HEALTH CONFIG SMOKE TESTS PASSED!
```
