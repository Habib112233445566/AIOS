# T-02453: Sandbox Enforcement Automated Tests Scaffold

## 1. Scaffold Overview
This task establishes the automated integration test scaffolding for the Sandbox Enforcement subsystem in `code/aiosh-rust/aiosh-core/tests/test_sandbox_automated.rs`.

---

## 2. Test Harness Structure

The integration test suite imports the following core components:
- `aiosh_core::audit::{AuditRing, OpenOptions}`: Structured SQLite WAL audit logging ring buffer.
- `aiosh_core::sandbox_config::*`: Sandbox configuration and limits (`SandboxConfig`, constants).
- `aiosh_core::sandbox_data_model::*`: Domain models (`SandboxProfile`, `ResourceLimits`, `FilesystemPolicy`, `SyscallPolicy`, `NetworkIsolationMode`, `IsolationLevel`).
- `aiosh_core::sandbox_service::*`: Core engine (`SandboxService`, `SandboxExecutionRequest`, `SandboxExecutionResponse`).

---

## 3. Test Vector Scaffold Matrix

The scaffold defines the following test cases corresponding to specification vectors:
1. `test_autosandbox1_profile_lifecycle_and_invariants`:
   - Validates registration, immutable protected factory profiles (`standard`, `strict`, `permissive`), duplicate detection, and deletion lifecycle.
2. `test_autosandbox2_filesystem_policy_conflict_and_traversal`:
   - Scaffolds path conflict checking (`paths_ro` vs `paths_rw`), path traversal detection (`..`), and path length bound checking (`MAX_PATH_LEN`).
3. `test_autosandbox3_syscall_and_network_modes`:
   - Tests isolation level settings, network isolation modes (`Disabled`, `LoopbackOnly`, `Unrestricted`), and syscall action mappings (`ReturnErrno`).
4. `test_autosandbox4_resource_limits_boundary_conditions`:
   - Validates resource constraint boundaries (zero memory, excessive memory, zero wall timeout).
5. `test_autosandbox5_supervised_execution_lifecycle`:
   - Tests process spawning, exit code propagation (0, 42, 127 for non-existent binaries), and stdout/stderr capture.
6. `test_autosandbox6_output_capture_truncation`:
   - Verifies capture clamping to `max_output_capture_bytes` to prevent buffer bloat attacks.
7. `test_autosandbox7_pep_grant_authorization_gating`:
   - Verifies fail-closed enforcement when `enforce_pep_grants` is set.
8. `test_autosandbox8_thread_safe_concurrent_invocations`:
   - Verifies multithreaded concurrent execution across 4 worker threads writing to an `AuditRing` backed by SQLite WAL.

---

## 4. Verification
The test harness compiles cleanly and all 8 vectors are wired to valid assertions.
```
cargo test -p aiosh-core --test test_sandbox_automated
```
