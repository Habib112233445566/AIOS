# T-02402: Sandbox Enforcement Data Model Specification

## 1. Specification Overview
This specification defines the formal data structures, interfaces, invariants, and validation semantics for the **Sandbox Enforcement Data Model** in `aiosh-core`. It establishes a comprehensive, type-safe representation of execution sandboxing, security profiles, resource quotas, filesystem permissions, syscall filtering, and execution results.

---

## 2. Core Types & Schema Definitions

### A. Profiles & Isolation Levels
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxProfileType {
    Strict,
    Standard,
    Permissive,
    IsolatedDev,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationLevel {
    ProcessOnly,
    RestrictedNamespaces,
    FullLandlockSeccomp,
}
```

### B. Resource Limits
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: u64,
    pub max_cpu_time_ms: u64,
    pub max_wall_time_ms: u64,
    pub max_processes: u32,
    pub max_open_files: u32,
}
```
- **Invariants**:
  - `max_memory_bytes`: Minimum 1MB ($10^6$ bytes), maximum 64GB ($64 \times 10^9$ bytes). Default: 512MB.
  - `max_wall_time_ms`: Minimum 10ms, maximum 3,600,000ms (1 hour). Default: 30,000ms (30s).
  - `max_processes`: Range 1..4096. Default: 32.
  - `max_open_files`: Range 16..65536. Default: 256.

### C. Filesystem Policy
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FilesystemPolicy {
    pub paths_ro: Vec<String>,
    pub paths_rw: Vec<String>,
    pub paths_execute: Vec<String>,
    pub paths_denied: Vec<String>,
    pub allow_cwd_rw: bool,
    pub allow_tmp_rw: bool,
}
```
- **Invariants**:
  - Every path must be canonical or absolute; relative paths with `..` are strictly rejected.
  - Conflicts: A path cannot appear simultaneously in `paths_ro` and `paths_rw` or `paths_denied`.

### D. Network Policy
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkIsolationMode {
    Disabled,
    LoopbackOnly,
    FilteredEgress { allowed_ports: Vec<u16> },
    Unrestricted,
}
```

### E. Syscall Policy
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyscallAction {
    KillProcess,
    ReturnErrno(i32),
    Log,
    Allow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyscallPolicy {
    pub no_new_privs: bool,
    pub default_action: SyscallAction,
    pub denylist: Vec<String>,
    pub allowlist: Vec<String>,
}
```

### F. Environment Policy
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EnvironmentPolicy {
    pub clean_env: bool,
    pub allow_vars: Vec<String>,
    pub injected_vars: std::collections::BTreeMap<String, String>,
}
```

### G. Unified Sandbox Profile
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxProfile {
    pub name: String,
    pub profile_type: SandboxProfileType,
    pub isolation_level: IsolationLevel,
    pub resources: ResourceLimits,
    pub filesystem: FilesystemPolicy,
    pub network: NetworkIsolationMode,
    pub syscall: SyscallPolicy,
    pub environment: EnvironmentPolicy,
}
```

### H. Execution Request & Result
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxExecutionRequest {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub profile: SandboxProfile,
    pub session_id: Option<String>,
    pub pep_grant_id: Option<String>,
    pub stdin_data: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxComponentStatus {
    pub component: String,
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxExecutionStatus {
    Success,
    TimedOut,
    Signaled(i32),
    Violation(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxExecutionResult {
    pub exit_code: i32,
    pub status: SandboxExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub components_applied: Vec<SandboxComponentStatus>,
    pub audit_hash: Option<String>,
}
```

---

## 3. Backward Compatibility & Conversion Interface
To guarantee seamless interoperability with Sprint 2's `SandboxPolicy` in `sandbox.rs`:
- Implement `From<&SandboxProfile> for SandboxPolicy`.
- `SandboxProfile::from_legacy_policy(legacy: &SandboxPolicy) -> SandboxProfile`.

---

## 4. Error Handling & Validation
Standardized error codes:
- `ERR_SANDBOX_INVALID_PATH`: Path contains traversal (`..`) or invalid characters.
- `ERR_SANDBOX_INVALID_LIMIT`: Resource bounds outside permitted ranges.
- `ERR_SANDBOX_POLICY_CONFLICT`: Incompatible filesystem or network rules.
- `ERR_SANDBOX_EMPTY_COMMAND`: Missing executable target.
