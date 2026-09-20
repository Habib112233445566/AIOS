# Task Evidence: T-01957 (System Update / automated tests: Security Review)

## 1. Security Review & Threat Modeling
Sub-Epic 6 Automated Tests Security Review evaluates potential vulnerabilities and abuse scenarios associated with the test harnesses (`test_system_update_e2e.rs` and `test_system_update_e2e_smoke.py`).

## 2. Threat Modeling Matrix (THREAT-UTEST-01 - THREAT-UTEST-06)

| Threat ID | Threat Scenario | Severity | Impact | Mitigation Strategy |
|---|---|---|---|---|
| `THREAT-UTEST-01` | Insecure Temp Directory & Predictable Names | MEDIUM | Local unprivileged users tampering with test staging directories or snooping payloads. | Use `std::process::id()` and UUID/nanosecond suffixes; ensure strict directory permissions (`0700` where supported). |
| `THREAT-UTEST-02` | Test Panic / Resource Leakage | MEDIUM | Test aborts leave gigabytes of simulated payload in `/tmp` causing disk exhaustion. | RAII cleanup guards (`Drop` implementation) for test staging directories ensuring cleanup even on panic/failure. |
| `THREAT-UTEST-03` | Test Fixture Path Traversal | HIGH | Malicious test fixture with `../` attempting deletion or overwrite of host filesystem during teardown. | Canonicalize paths and verify destination directory is within `std::env::temp_dir()` before deleting recursively. |
| `THREAT-UTEST-04` | Host System Mutation (Privilege Escalation) | CRITICAL | Tests attempting raw block device access (`/dev/sda`) or real bootloader modification. | Strictly isolate tests to file-based mocks and in-memory simulated slot controllers; prohibit direct block device access in user-space tests. |
| `THREAT-UTEST-05` | Concurrent Test Collision | LOW | Parallel test runners (`cargo test -- --test-threads > 1`) using shared temp paths causing race conditions. | Unique temp directory paths per test function including function name and PID. |
| `THREAT-UTEST-06` | Unverified Audit Trail Emission | MEDIUM | Tests pass state transitions without asserting PEP authorization and audit event logging. | Explicitly assert audit row generation and grant attribution in integration tests. |

## 3. Findings & Required Hardening Actions (for T-01958)
1. Implement a RAII `TestTempDir` struct in `test_system_update_e2e.rs` that automatically deletes the directory when dropped, guaranteeing zero leftover artifacts on panic.
2. Ensure path canonicalization and verification that temp directories are children of `std::env::temp_dir()` before executing `fs::remove_dir_all`.
3. Add explicit test assertions verifying that state-changing actions produce structured errors and clean states on failure.
