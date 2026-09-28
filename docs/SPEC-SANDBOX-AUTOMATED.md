# Specification: Sandbox Enforcement Automated Testing Subsystem

## 1. Overview
The Sandbox Enforcement Automated Testing Subsystem provides comprehensive, regression-resistant verification of the AIOS sandbox isolation kernel, covering profile lifecycles, filesystem traversal constraints, syscall/network isolation, resource limits, process execution lifecycles, and concurrency safety.

---

## 2. Test Architecture & Vector Matrix

### Formal Test Vectors (`test_sandbox_automated.rs`)

| Vector ID | Test Name | Invariant Asserted |
|---|---|---|
| `AUTOSANDBOX1` | `test_autosandbox1_profile_lifecycle_and_invariants` | Profile capacity (max 256), duplicate name rejection, protected profile immutability (`standard`, `strict`, `permissive`). |
| `AUTOSANDBOX2` | `test_autosandbox2_filesystem_policy_conflict_and_traversal` | Path conflict detection (`paths_ro` vs `paths_rw`), directory traversal rejection (`..`), path length limits (4096). |
| `AUTOSANDBOX3` | `test_autosandbox3_syscall_and_network_modes` | Network isolation modes (`Disabled`, `LoopbackOnly`, `Unrestricted`), seccomp syscall action mapping (`ReturnErrno`). |
| `AUTOSANDBOX4` | `test_autosandbox4_resource_limits_boundary_conditions` | Zero memory rejection, excessive memory rejection (>64 GiB), zero execution wall-clock timeout rejection. |
| `AUTOSANDBOX5` | `test_autosandbox5_supervised_execution_lifecycle` | Child process spawning, exit code propagation (0, 42, 127 for missing binaries), stdout/stderr capture. |
| `AUTOSANDBOX6` | `test_autosandbox6_output_capture_truncation` | Output capture buffer clamping to `max_output_capture_bytes` preventing memory exhaustion. |
| `AUTOSANDBOX7` | `test_autosandbox7_pep_grant_authorization_gating` | PEP capability grant authorization; fail-closed execution rejection without valid grant when enabled. |
| `AUTOSANDBOX8` | `test_autosandbox8_thread_safe_concurrent_invocations` | Multi-threaded concurrent execution across 4 worker threads writing to SQLite WAL ring buffer. |

---

## 3. Invocation Commands

### Rust Integration Test Suite
```bash
cargo test -p aiosh-core --test test_sandbox_automated
```

### Python End-to-End Smoke Test Suite
```bash
python -m pytest code/aiosh-mcp/tests/test_sandbox_automated_smoke.py
```

### Subprocess Integration Script
```bash
python scripts/test_sandbox_integration_smoke.py
```

---

## 4. Known Constraints & Platform Specifics
- On Windows systems, Linux Landlock and Seccomp BPF syscall isolation features are gracefully marked as unavailable (`status: "unavailable"` in probe diagnostics), with fallback to process-level isolation and standard stream supervision.
- Subprocess timeouts default to profile limits (`max_wall_time_ms`) with strict clamp ranges $[10\text{ ms}, 3600\text{ s}]$.
