# T-02411: Sandbox Enforcement Core Service Research

## 1. Executive Summary & Objective
This research document establishes the architectural requirements, design patterns, upstream constraints, and prior art for the **Sandbox Enforcement Core Service** (`SandboxService`) in `aiosh-core`. The core service acts as the central execution broker that bridges high-level policy evaluation, profile management, process spawning with containment controls, execution timeout supervision, audit trail emission, and PEP grant verification.

---

## 2. Analysis of Existing Codebase & Subsystems

### A. Current Process Execution & Audit Flow
1. **`sandbox_exec` (`sandbox.rs`)**:
   - Spawns child process via `libc::fork()` + `libc::execv()` on Linux, applying `no_new_privs`, `seccomp-bpf`, and `landlock` in the child before exec.
   - On non-Linux hosts, runs `std::process::Command` directly with honest stderr warning emission.
2. **Audit Logging Flow (`audit.rs`, `audit_chain_ext.rs`)**:
   - `AuditRing` records events into SQLite WAL table `audit_ring`.
   - Consequential actions must write an audit row carrying `tool="sandbox"`, `command`, `args`, `outcome`, `c_flags`, and extended causal links.
3. **PEP Grant Verification (`pep_grant.rs`, `pep_grant_service.rs`)**:
   - Grants have `grant_id`, `actor_id`, `capability`, `constraints`, and expiration.
   - When `pep_grant_id` is supplied in an execution request, the sandbox core service must verify that the grant is valid, active, and authorizes the target command.

---

## 3. Facts vs. Assumptions

| Domain | Verified Fact | Working Assumption |
|---|---|---|
| **Process Supervision** | Standard OS process execution can hang indefinitely if child blocks on IO or deadlocks. | The core service must wrap process execution in an active watchdog timer bounded by `max_wall_time_ms`. |
| **Output Buffering** | Uncontrolled stdout/stderr capture can exhaust host memory (OOM). | Output streams must be capped (e.g., max 10MB per stream) with truncation markers. |
| **Host Capability Probing** | Not all hosts support Landlock ABI v1-v4 or Seccomp BPF (e.g. containers without `CAP_SYS_ADMIN` or WSL/macOS/Windows). | The service must dynamically probe host containment capabilities and report them honestly in execution telemetry. |
| **Audit Atomicity** | Consequential executions must be auditable even when the sandboxed command fails or is terminated. | The core service must ensure an audit row is recorded on both success and termination paths. |

---

## 4. Core Architectural Decisions for `SandboxService`

1. **Service Struct & Responsibilities**:
   - `SandboxService`: Manages profile catalog (`standard`, `strict`, `permissive`, user-registered custom profiles), provides capability probing, executes commands with timeout enforcement, and links to `AuditRing`.
2. **Profile Catalog & Registry**:
   - In-memory thread-safe registry holding predefined and dynamically registered `SandboxProfile` instances.
3. **Execution Pipeline**:
   - Step 1: Validate request against bounds and profile constraints.
   - Step 2: (Optional) Validate PEP grant token if `pep_grant_id` is provided.
   - Step 3: Probe host containment status and assemble execution parameters.
   - Step 4: Execute command with bounded timeout and capped stdout/stderr capture.
   - Step 5: Record execution event into `AuditRing` with duration, exit code, and applied components.
   - Step 6: Return structured `SandboxExecutionResult`.

---

## 5. Unknowns & Resolutions
- **Q1: Should execution be synchronous or asynchronous?**
  - *Resolution*: Implement synchronous execution with bounded timeout thread/channel supervision, matching the existing `aiosh-core` service patterns (e.g. `AuditChainService`, `DistroStore`, `FsLayoutService`), while allowing future async extensions.
- **Q2: How should host capability discovery be represented?**
  - *Resolution*: Model `HostSandboxCapabilities` struct with boolean flags: `landlock_supported`, `seccomp_bpf_supported`, `no_new_privs_supported`, `cgroups_v2_supported`, and detected `landlock_abi_version`.
