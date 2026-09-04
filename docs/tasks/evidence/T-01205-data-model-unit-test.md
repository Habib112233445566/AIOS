# T-01205: Package Management - Data Model: Unit Test

## Metadata
- **Task ID:** `T-01205`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package`
- **Component:** Package Management Data Model Unit & Integration Tests
- **Status:** Complete

## 1. Test Suite Deliverable
Authored standalone test suite in `code/aiosh-rust/aiosh-core/tests/test_package_data_model.rs` covering all core invariants PM1..PM5, boundary conditions, and failure modes.

### Test Cases & Invariant Assertions:
1. `test_pm1_package_name_boundary_and_syntax`:
   - Valid names: `bash`, `coreutils`, `g++`, `libssl3`, `python3.11`, `zlib1g`, `apk-tools`, `alpine-baselayout`.
   - Length boundary checks: min length 1 char (`a`, `1`), max length 128 chars.
   - Negative rejection assertions: oversized names (>128), uppercase characters, leading symbols (`-`, `+`, `.`), invalid characters (`@`, `!`, `/`, `\`), whitespace, control characters, null bytes.
2. `test_pm2_bounds_and_lengths`:
   - Version bounds: empty string rejection, 64-char boundary acceptance, 65-char rejection, control character rejection.
   - Architecture bounds: non-empty, graphic ASCII checks.
   - Description bounds: 4096-byte acceptance, 4097-byte rejection.
   - Sizing bounds: 100 GiB ceiling acceptance, 100 GiB + 1 byte rejection.
   - Dependencies count bounds: 256 items acceptance, 257 items rejection.
3. `test_pm3_dependency_hygiene`:
   - Self-dependency rejection (`dep.name == spec.name`).
   - Duplicate dependency rejection.
   - Invalid dependency name syntax rejection.
4. `test_pm4_checksum_and_provenance`:
   - 64-character hexadecimal SHA-256 validation.
   - Non-hex characters and non-64 lengths rejection.
   - Repository URL: HTTPS protocol enforcement, loopback HTTP (`127.0.0.1`, `localhost`) allowed, unencrypted public HTTP rejected.
5. `test_pm5_state_consistency`:
   - Installed packages must have `installed_size_bytes > 0`.
   - Available packages permitted with 0 installed size.
6. `test_package_transaction_invariants`:
   - Valid multi-action batch transactions.
   - Conflicting actions on identical package names rejected.
   - Empty action lists rejected.
   - Malformed timestamps and non-graphic transaction IDs rejected.
7. `test_serde_json_roundtrip`:
   - Full serde serialize/deserialize roundtrip parity with snake_case tag verification.
