# Task Evidence: T-01858 - Network Bootstrap / automated tests: Hardening

## 1. Overview
- **Task ID**: `T-01858`
- **Sub-Epic**: 6 (Network Bootstrap Automated Tests)
- **Goal**: Harden automated test harnesses, assert RAII cleanup, prevent resource leaks, and verify explicit error messages on negative paths.

---

## 2. Hardening Measures Implemented
1. **Deterministic Cleanup Assertion**:
   - Added `test_automated_tempdir_cleanup_on_drop` verifying that temporary directories created by `MockNetworkEnv` are purged from disk immediately upon drop.
   - Wrapped Python temporary directory and environment overrides in `try/finally` blocks to guarantee cleanup even if test assertions fail.
2. **Negative Name Validation**:
   - Added `test_automated_invalid_interface_names_rejected` asserting that empty names, whitespace, path components (`/`), parent traversal (`..`), null characters (`\0`), and names longer than 15 characters fail with explicit error codes.
3. **Fault Injection Isolation**:
   - Verified that corrupt routing tables and empty DNS configurations do not crash the service, produce unhandled panics, or leak descriptors.

---

## 3. Verification
- `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_network_automated`: 8/8 passed in 0.11s.
- `python code/aiosh-cli/tests/test_network_e2e_smoke.py`: 5/5 passed in 0.14s.
