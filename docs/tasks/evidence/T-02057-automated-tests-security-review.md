# Task Evidence: T-02057 (Capability Model / automated tests: Security Review)

## Task Information
- **Task ID**: T-02057
- **Title**: Capability Model / automated tests: Security Review
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 6: Automated Tests
- **Status**: Completed
- **Date**: 2026-09-20

## Threat Model: Automated Test Harness (`THREAT-CAPTEST-01..06`)

| Threat ID | Threat Vector | CWE | Impact | Mitigation Strategy |
|---|---|---|---|---|
| `THREAT-CAPTEST-01` | Host State Pollution & Residual Fixtures | CWE-379 | Flaky tests, disk exhaustion, data contamination | `MockCapabilityEnv` uses RAII `tempfile::TempDir` with deterministic cleanup on drop. |
| `THREAT-CAPTEST-02` | Clock Skew & Temporal Race Conditions | CWE-362 | Flaky expiration tests in CI environments | Tests inject deterministic `simulated_now: DateTime<Utc>` to evaluate expiry and pruning. |
| `THREAT-CAPTEST-03` | Zombie Subprocesses in Python E2E Harness | CWE-400 | Orphaned `aiosh-mcp` processes causing CI deadlocks | Bounded `timeout_s` (30s) with explicit `p.kill()` and `p.wait()` in exception handlers. |
| `THREAT-CAPTEST-04` | Unchecked Recursion in Multi-Tier Traversal | CWE-674 | Stack overflow during deep attenuation or revocation | Cycle detection in `revoke_capability` using `visited: HashSet<String>` and acyclic hierarchy. |
| `THREAT-CAPTEST-05` | Path Traversal Vulnerabilities in Test Fixtures | CWE-22 | Accidental mutation or deletion of files outside temp dir | Explicit assertions verifying rejection of `..` path components and non-json extensions. |
| `THREAT-CAPTEST-06` | Incomplete Invariant Assertions (False Positives) | CWE-398 | Undetected security regressions | Dual-assertion pattern: verify positive authorization (Read granted) AND negative denial (Write denied). |

## Review Conclusion
The automated test harnesses (`test_capability_automated.rs` and `test_capability_automated_smoke.py`) are fundamentally sound. Hardening targets for `T-02058`:
1. Verify explicit `Drop` implementation or cleanup verification on `MockCapabilityEnv`.
2. Add explicit signal handling / process termination guarantee in `test_capability_automated_smoke.py`.
3. Add stress test verifying that large capability hierarchies (e.g., 50+ nodes) do not cause stack overflow.
