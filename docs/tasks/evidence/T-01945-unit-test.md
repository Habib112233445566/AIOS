# Task Evidence: T-01945 - System Update / Configuration: Unit Test

- **Task**: `T-01945`
- **Sub-Epic**: `Sub-Epic 5: Configuration & Policy`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Unit Testing
Authored and executed comprehensive unit tests for `SystemUpdateConfig` in `code/aiosh-rust/aiosh-core/tests/test_system_update_config.rs`:

1. **`test_default_config_valid`**:
   - Asserts that `SystemUpdateConfig::default()` validates cleanly.
   - Validates default paths (`/var/lib/aiosh/updates`, `/var/lib/aiosh/updates/staging`).
   - Validates default flags (`allow_auto_apply: false`, `auto_rollback_on_failure: true`).
   - Verifies seamless mapping to `SystemUpdateServiceConfig` via `to_service_config()`.

2. **`test_path_hygiene_and_traversal`**:
   - Validates rejection of empty paths.
   - Validates rejection of paths exceeding 1024 characters.
   - Validates rejection of control characters in paths.
   - Validates rejection of `..` parent directory traversal in both `state_dir` and `staging_dir`.

3. **`test_bounds_validation`**:
   - Verifies rejection of `check_interval_secs` outside $[60, 2592000]$.
   - Verifies rejection of `max_payload_bytes` outside $[1\text{MB}, 10\text{GB}]$.
   - Verifies rejection of `min_free_space_bytes` exceeding 100GB.
   - Verifies rejection of `trusted_keys` with $>32$ entries, $>256$ chars, or control characters.

4. **`test_from_env_overrides`**:
   - Verifies ingestion of `AIOSH_UPDATE_STATE_DIR`, `AIOSH_UPDATE_STAGING_DIR`, `AIOSH_UPDATE_CHANNEL`, `AIOSH_UPDATE_CHECK_INTERVAL_SECS`, `AIOSH_UPDATE_AUTO_APPLY`, `AIOSH_UPDATE_AUTO_ROLLBACK`, and `AIOSH_UPDATE_MAX_PAYLOAD_BYTES`.

5. **`test_file_persistence_and_loading`**:
   - Verifies atomic file writing via `save_to_file()`.
   - Verifies deserialization and validation via `from_file()`.
   - Verifies rejection of missing files and files exceeding 1MB.

## Verification
- Executed `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_system_update_config`.
- Result: 5/5 unit tests passed with 0 failures.
