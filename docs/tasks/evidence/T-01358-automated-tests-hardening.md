# T-01358: Init & Service Supervision - Automated Tests: Hardening

## Metadata
- **Task ID:** `T-01358`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Automated Tests Hardening
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Hardening Controls & Defenses

### 1. Subprocess Execution Timeouts
- `tools/test_service_suites.py` wraps every `cargo test` invocation in a strict 120-second timeout (`timeout=120`).
- If an automated test encounters a deadlock, infinite loop, or resource contention, the test runner terminates the process immediately, logs an explicit timeout error `[-] SS* timed out after 120s`, and exits with code 1 rather than hanging indefinitely.

### 2. Resource Cleanup & File Hygiene
- In `code/aiosh-rust/aiosh-core/tests/test_service_automated.rs`, temporary stores are isolated using per-process unique filenames (`aios_st3_auto_test_{pid}.json`).
- All created test store files are removed via `std::fs::remove_file`, precluding file descriptor or disk leaks.

### 3. Cycle Detection & Graph Upper Bounds
- The automated integration test suite explicitly exercises Kahn's algorithm cycle detection (`test_st2_dependency_dag_order_and_cycle_detection`).
- Graphs with cycles abort in $O(V + E)$ time with an explicit error, avoiding infinite loops or call-stack overflow.

### 4. Zero Silent Failures
- All failure modes in the test runner return non-zero exit codes (`sys.exit(1)`).
- Error messages from stdout and stderr are printed explicitly to `sys.stderr` when a cargo test fails, ensuring fail-fast observability (ADR-0036).
- Action execution errors and invalid inputs produce structured error envelopes (never silent failure).
