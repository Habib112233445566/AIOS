# T-02452: Sandbox Enforcement Automated Tests Specification

## 1. Specification Overview
This specification details the formal test vectors, invariant assertions, concurrency conditions, and expected outcomes for the automated test suite of Sandbox Enforcement (`test_sandbox_automated.rs`).

---

## 2. Test Vector Matrix

### AUTOSANDBOX1: Profile Lifecycle & Bounds Validation
- **Goal**: Validate profile registration capacity, collision handling, and factory profile immutability.
- **Assertion**:
  - `MAX_PROFILES_IN_SERVICE` (256) capacity limit enforced.
  - Duplicate profile names rejected (`ERR_SANDBOX_PROFILE_EXISTS`).
  - Protected profiles (`standard`, `strict`, `permissive`) cannot be deleted (`ERR_SANDBOX_CANNOT_DELETE_DEFAULT`).

### AUTOSANDBOX2: Filesystem Policy Conflict & Traversal Stress
- **Goal**: Verify path conflict detection and strict directory traversal prevention.
- **Assertion**:
  - Mutually exclusive paths (read-only vs write-only) reject with `ERR_SANDBOX_POLICY_CONFLICT`.
  - Path traversal sequences (`..`) in read, write, or execute paths reject with `ERR_SANDBOX_INVALID_PATH`.
  - Paths exceeding `MAX_PATH_LEN` (4096) reject with `ERR_SANDBOX_BOUNDS_EXCEEDED`.

### AUTOSANDBOX3: Syscall & Network Isolation Modes
- **Goal**: Verify isolation level combinations and network policy enforcement.
- **Assertion**:
  - `NetworkIsolationMode::None` vs `LoopbackOnly` vs `Full`.
  - Bounded syscall ruleset definitions.

### AUTOSANDBOX4: Resource Limits Boundary Conditions
- **Goal**: Verify boundary limits on memory and CPU time.
- **Assertion**:
  - Zero memory or memory exceeding 64 GiB rejected with `ERR_SANDBOX_INVALID_LIMIT`.
  - Zero execution timeout rejected with `ERR_SANDBOX_INVALID_LIMIT`.

### AUTOSANDBOX5: Supervised Execution Lifecycle & Exit Propagation
- **Goal**: Verify child process execution, standard stream capture, and exit code propagation.
- **Assertion**:
  - Exit code 0 propagation on successful execution.
  - Non-zero exit code propagation on failed commands.
  - Return code 127 on nonexistent binaries.

### AUTOSANDBOX6: Output Capture Truncation & Memory Bounding
- **Goal**: Verify standard output and standard error capture is strictly clamped to `max_output_capture_bytes`.
- **Assertion**:
  - Emitted stream exceeding capture buffer is cleanly truncated without process panic.

### AUTOSANDBOX7: PEP Grant Authorization Gating
- **Goal**: Verify PEP capability grant authorization under enforcing configuration.
- **Assertion**:
  - When `enforce_pep_grants` is true, missing grant fails closed with `ERR_SANDBOX_PEP_UNAUTHORIZED`.
  - When grant is present, execution proceeds.

### AUTOSANDBOX8: Thread-Safe Concurrent Invocations
- **Goal**: Verify multiple execution threads simultaneously running sandboxed tasks and writing to `AuditRing`.
- **Assertion**:
  - Zero SQLite WAL lock deadlocks across 4 concurrent execution threads.
  - Consistent total audit row count in `audit_ring`.
