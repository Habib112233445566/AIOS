# T-02451: Sandbox Enforcement Automated Tests Research

## 1. Research Objectives
Establish the testing methodology, formal test vectors, invariant assertions, and stress frameworks for automated testing of the Sandbox Enforcement subsystem (`test_sandbox_automated.rs`).

## 2. Prior Art & Subsystem Analysis
1. **Existing Automated Test Suites**:
   - `test_audit_chain_automated.rs`: Vector-based stress testing (`AUTOAUDIT1` through `AUTOAUDIT8`).
   - `test_pep_grant_automated.rs`: Stress and concurrency verification across policy components.
2. **Key Invariants to Assert in Automated Suite**:
   - **AUTOSANDBOX1**: Profile Lifecycle & Boundary Clamp Validation
   - **AUTOSANDBOX2**: Filesystem Policy Overlap, Conflict, and Traversal (`..`) Rejection
   - **AUTOSANDBOX3**: Syscall & Network Isolation Modes Enforcement
   - **AUTOSANDBOX4**: Resource Limits Boundary Extremes (0, bounds exceeded, 64 GiB max)
   - **AUTOSANDBOX5**: Supervised Execution Lifecycle & Exit Code Propagation
   - **AUTOSANDBOX6**: Output Capture Hard Clamping & Truncation
   - **AUTOSANDBOX7**: PEP Capability Grant Authorization Gate Fail-Closed Verification
   - **AUTOSANDBOX8**: Multi-Threaded Concurrent Execution & Audit Logging Safety

## 3. Facts vs Assumptions
- **Fact**: Stress tests must run quickly (< 5s total) and not depend on external network connectivity or root privileges.
- **Fact**: Concurrency tests must prove absence of SQLite WAL deadlocks across concurrent execution threads.
- **Assumption**: Python executables (`python`/`python3`) or core tools can serve as reproducible child targets.

## 4. Design Decisions for Specification (T-02452)
- Define 8 formal test vectors (`AUTOSANDBOX1`..`AUTOSANDBOX8`) in `code/aiosh-rust/aiosh-core/tests/test_sandbox_automated.rs`.
- Include a companion Python smoke suite `test_sandbox_automated_smoke.py`.
