# T-01258: Package Management - Automated Tests: Hardening

## Metadata
- **Task ID:** `T-01258`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Package Management / Automated Tests
- **Status:** Complete
- **Date:** 2026-09-04

---

## 1. Hardening Controls & Defenses

### 1. Subprocess Execution Timeouts
- `tools/test_package_suites.py` wraps every `cargo test` invocation in a strict 120-second timeout (`timeout=120`).
- If an automated test experiences a deadlock, infinite loop, or resource contention, the test runner terminates the process immediately, logs an explicit timeout error `[-] PM* timed out after 120s`, and exits with code 1 rather than hanging indefinitely.

### 2. RAII Temp Directory & Resource Hygiene
- In `code/aiosh-rust/aiosh-core/tests/test_package_automated.rs`, temporary stores are created via `tempfile::tempdir()`.
- Guaranteed cleanup: Even if an assertion fails or a test panics, Rust's `Drop` implementation automatically deletes the temporary directory and all files inside, preventing leftover temp files or disk leaks.

### 3. Action Array & String Upper Bounds
- The automated test suite explicitly tests and hardens the 256-action transaction ceiling (`actions.len() <= 256`) and the package name / pattern length caps.
- Attempting to plan transactions with $> 256$ entries returns an explicit, structured error string `exceeds 256 entries`, rejecting uncontrolled allocation.

### 4. Zero Silent Failures
- All failure modes in the test runner return non-zero exit codes (`sys.exit(1)`).
- Error messages from stdout and stderr are printed explicitly to `sys.stderr` when a cargo test fails, ensuring fail-fast observability (ADR-0036).
