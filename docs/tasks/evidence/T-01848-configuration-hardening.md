# Task Evidence: T-01848 - Network Bootstrap / configuration: Hardening

## 1. Overview
- **Task ID**: `T-01848`
- **Sub-Epic**: 5 (Network Bootstrap Configuration)
- **Goal**: Harden `NetworkConfig` persistence, validation, and error-handling pathways.

---

## 2. Hardening Measures Implemented
1. **Explicit Error Classification**:
   - Prefixed all error messages with structured failure codes:
     - `NCONF_VALIDATION_ERROR`: Invariant violation during data structure validation or boundary violation.
     - `NCONF_IO_ERROR`: Filesystem read, write, metadata inspection, or directory creation failures.
     - `NCONF_PARSE_ERROR`: Malformed JSON deserialization failures.
     - `NCONF_SERIALIZATION_ERROR`: JSON serialization failures.
2. **Zero Temporary File Leakage**:
   - In `save_to_path()`, added immediate cleanup (`let _ = fs::remove_file(&tmp_path);`) on both `fs::write()` failure and `fs::rename()` failure. No partial or orphan `.tmp.*` files remain on disk under any error condition.
3. **Atomic Permissions on Unix**:
   - On Unix platforms (`#[cfg(unix)]`), intermediate temporary files are explicitly set to mode `0600` (`fs::Permissions::from_mode(0o600)`) before rename, preventing concurrent unauthorized read access during write transitions.
4. **File Size Limit Defense**:
   - `load_from_path()` strictly verifies file length against `MAX_CONFIG_FILE_BYTES` (1 MB) via `fs::metadata()` before reading content into memory, neutralizing memory exhaustion DoS attacks.

---

## 3. Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core` passed cleanly.
- Unit tests verified all error messages and roundtrip operations.
