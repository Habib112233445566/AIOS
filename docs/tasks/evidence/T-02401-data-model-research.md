# T-02401: Sandbox Enforcement Data Model Research

## 1. Executive Summary & Objective
This research document establishes the empirical facts, architectural constraints, prior art, and core data model requirements for **Sandbox Enforcement** within the AIOS Security Kernel & Policy Enforcement Point (PEP) Fabric (Phase 2). It provides the formal foundation for structured sandbox profiles, process isolation parameters, filesystem access lists, syscall filtering policies, environment sanitization specifications, and audit execution records.

---

## 2. Analysis of Existing Codebase & Prior Art

### A. Current Sandbox Implementation (`code/aiosh-rust/aiosh-core/src/sandbox.rs`, `code/aiosh-rust/aiosh-sandbox/src/main.rs`)
1. **Existing Primitives**:
   - `PathRule`: Basic struct with `path: String`, `read: bool`, `write: bool`, `execute: bool`.
   - `SandboxPolicy`: Holds `paths_ro: Vec<String>`, `paths_rw: Vec<String>`, `paths_execute: Vec<String>`, `no_new_privs: bool`, `seccomp_denylist: Vec<String>`, `inherit_defaults: bool`.
   - `DEFAULT_DENYLIST`: 17 dangerous x86_64 syscalls (`ptrace`, `mount`, `reboot`, `init_module`, `setuid`, etc.).
   - `apply_in_child` / `sandbox_exec`: Fork/execve sequence with stderr event emission (`{"event": "sandbox_applied", ...}`).
2. **Current Limitations & Gaps**:
   - **No Profile Abstraction**: All callers must manually construct path vectors and denylists or accept hardcoded defaults; there are no typed tiers (e.g., `Strict`, `Standard`, `Permissive`, `IsolatedDev`).
   - **Absence of Resource Constraints**: No data structures defining memory limits (bytes/cgroups), CPU time quotas, max thread/process count (`pids_max`), or execution timeouts.
   - **Lacks Network Isolation Modeling**: The existing policy only models filesystem paths and syscalls; there is no representation for network sandboxing (e.g., `None`, `LoopbackOnly`, `AllowedEndpoints`).
   - **Missing Environment Sanitization Rules**: Subprocesses inherit host environments unless manually stripped; no model exists for environment variable allowlists/denylists.
   - **Unstructured Execution Results**: Execution output is raw exit codes and parsed stderr JSON strings, lacking typed telemetry, resource usage metrics, and direct linkage to PEP grants (`pep_grant_id`) and audit hashes.

### B. Upstream Standards & Authoritative References
1. **Linux Kernel Landlock LSM** (ABI v1-v4; Documentation/security/landlock.rst):
   - Access rights: `LANDLOCK_ACCESS_FS_EXECUTE`, `LANDLOCK_ACCESS_FS_WRITE_FILE`, `LANDLOCK_ACCESS_FS_READ_FILE`, `LANDLOCK_ACCESS_FS_READ_DIR`, `LANDLOCK_ACCESS_FS_REMOVE_DIR`, `LANDLOCK_ACCESS_FS_REMOVE_FILE`, `LANDLOCK_ACCESS_FS_MAKE_DIR`, etc.
   - Hierarchy: Rulesets are additive; once restricted, a process cannot elevate rights.
2. **Seccomp-BPF** (RFC / Documentation/userspace-api/seccomp_filter.rst, `prctl(PR_SET_NO_NEW_PRIVS)`):
   - Filter programs return actions: `SECCOMP_RET_KILL_PROCESS`, `SECCOMP_RET_TRAP`, `SECCOMP_RET_ERRNO`, `SECCOMP_RET_LOG`, `SECCOMP_RET_ALLOW`.
   - Architecture checks (`SECCOMP_DATA_ARCH_OFFSET`) prevent multi-arch syscall bypasses.
3. **OCI (Open Container Initiative) Runtime Specification**:
   - Standardizes process limits (`rlimits`), namespaces (`pid`, `net`, `ipc`, `uts`, `mount`), and read-only/masked paths.
4. **Bubblewrap (`bwrap`) & Systemd Sandbox Directives**:
   - `ProtectSystem=strict`, `ProtectHome=read-only`, `PrivateTmp=yes`, `SystemCallFilter=~@privileged`.

---

## 3. Fact vs. Assumption Separation

| Topic | Verified Fact | Working Assumption |
|---|---|---|
| **Landlock Availability** | Landlock is supported in Linux kernels $\ge$ 5.13; non-Linux hosts return `ENOSYS` / `EOPNOTSUPP`. | AIOS must provide an "honest position" fallback on non-Linux hosts: report `FAIL: landlock unsupported on non-Linux` in audit telemetry without failing closed unless strict mode is demanded. |
| **Seccomp Filter Inheritance** | Seccomp filters persist across `execve` when `PR_SET_NO_NEW_PRIVS` is set. | Denylists must include dangerous process inspection (`ptrace`) and namespace modification syscalls. |
| **Resource Limits** | POSIX `setrlimit` / Linux cgroups can constrain CPU and memory limits. | Memory and time bounds can be represented in platform-agnostic units (megabytes, milliseconds) in the data model. |
| **PEP Integration** | PEP capability tokens (`pep_grant_id`) govern tool invocation authority. | Every sandboxed execution request must carry optional `pep_grant_id` and `session_id` to link directly with the PEP fabric. |

---

## 4. Architectural Data Model Decisions

1. **Profile Hierarchy**:
   - `SandboxProfileType`: Enumeration covering `Strict` (zero network, minimal RO paths, strict seccomp), `Standard` (cwd RW, temp RW, default denylist), `Permissive` (relaxed paths, warn-only), and `Custom`.
2. **Modular Policy Structure**:
   - Separate concerns into `FilesystemPolicy`, `SyscallPolicy`, `NetworkPolicy`, `ResourceLimits`, and `EnvironmentPolicy`.
3. **Execution Request & Result Envelopes**:
   - `SandboxExecutionRequest`: Command, arguments, working directory, environment, profile, timeouts, and PEP provenance IDs.
   - `SandboxExecutionResult`: Exit status, execution duration, peak memory, applied security components, violations, and audit record hash.
4. **Validation Invariants**:
   - All filesystem paths must reject path traversal (`..`).
   - Timeout and resource bounds must have deterministic upper ceilings.
   - Canonical JSON serialization for all model structs to guarantee deterministic hashing.

---

## 5. Unknowns & Resolutions

- **Q1: Should the data model live in `aiosh-core` alongside `sandbox.rs` or in a separate module?**
  - *Resolution*: Create `sandbox_data_model.rs` (re-exported via `aiosh-core::sandbox_data_model` and `aiosh-core::sandbox`) to maintain clean separation between domain models and OS-specific syscall invocations.
- **Q2: How should network policy be modeled before full network namespace support is ready?**
  - *Resolution*: Model `NetworkIsolationMode` as `Disabled`, `LoopbackOnly`, `FilteredEgress`, and `Unrestricted`. On platforms without netns support, `Disabled` blocks network syscalls via seccomp filter (`socket`, `connect`).
