# T-02412: Sandbox Enforcement Core Service Specification

## 1. Scope & Objective
This specification defines the exact interface, behavior, operational lifecycle, failure handling, and audit trail effects for the **Sandbox Enforcement Core Service** (`SandboxService`) in `aiosh-core`.

---

## 2. Types & Interface Contract

### A. Host Capabilities Probe
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostSandboxCapabilities {
    pub landlock_supported: bool,
    pub landlock_abi_version: Option<u32>,
    pub seccomp_bpf_supported: bool,
    pub no_new_privs_supported: bool,
    pub platform: String,
}
```

### B. Core Service Configuration
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub default_profile_name: String,
    pub max_output_capture_bytes: usize,
    pub enforce_pep_grants: bool,
    pub audit_enabled: bool,
}
```
- **Defaults**:
  - `default_profile_name`: "standard"
  - `max_output_capture_bytes`: 10,485,760 (10 MB)
  - `enforce_pep_grants`: false (enabled in PEP enforcement modes)
  - `audit_enabled`: true

### C. `SandboxService` API
```rust
pub struct SandboxService {
    ring: Option<AuditRing>,
    config: SandboxConfig,
    profiles: HashMap<String, SandboxProfile>,
}

impl SandboxService {
    pub fn new(ring: Option<AuditRing>, config: SandboxConfig) -> Self;
    pub fn with_default_profiles(ring: Option<AuditRing>) -> Self;

    pub fn register_profile(&mut self, profile: SandboxProfile) -> Result<(), String>;
    pub fn get_profile(&self, name: &str) -> Option<SandboxProfile>;
    pub fn list_profiles(&self) -> Vec<SandboxProfile>;
    pub fn remove_profile(&mut self, name: &str) -> Result<bool, String>;

    pub fn probe_host_capabilities() -> HostSandboxCapabilities;

    pub fn execute(&mut self, request: &SandboxExecutionRequest) -> Result<SandboxExecutionResult, String>;
}
```

---

## 3. Execution Pipeline & Invariants

1. **Validation & Resolution**:
   - Validate `request.validate()`. If invalid, return `Err(reason)`.
   - If profile is not provided in request, resolve `config.default_profile_name`.
2. **PEP Grant Gating**:
   - If `config.enforce_pep_grants` is true and `request.pep_grant_id` is missing or invalid, return `Err("ERR_PEP_GRANT_REQUIRED")`.
3. **Execution & Supervision**:
   - Spawn execution with environment filtering according to `request.profile.environment`.
   - Apply timeout watchdog bounded by `request.profile.resources.max_wall_time_ms`.
   - If the process exceeds wall time, terminate process with `SIGKILL` and record `SandboxExecutionStatus::TimedOut`.
   - Cap captured stdout and stderr at `config.max_output_capture_bytes`.
4. **Audit Trail Recording**:
   - If `config.audit_enabled` and `ring` is `Some`:
     - Construct `AuditRowInput` with:
       - `tool`: `"sandbox"`
       - `command`: `request.command.clone()`
       - `args`: serialized JSON carrying args, profile name, and duration
       - `outcome`: `"ok"` if success, `"error"` on failure/timeout
       - `c_flags`: security classification flags
     - Write row to SQLite WAL audit ring.
     - Attach resulting row hash to `SandboxExecutionResult.audit_hash`.
5. **Return**:
   - Return populated `SandboxExecutionResult`.
