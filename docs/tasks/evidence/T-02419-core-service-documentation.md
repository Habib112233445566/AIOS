# T-02419: Sandbox Enforcement Core Service Documentation

## 1. Overview & Architecture
The **Sandbox Enforcement Core Service** (`SandboxService`) orchestrates containment profiles, execution supervision, PEP authorization gating, and cryptographic audit emission for AIOS userspace processes.

Key architectural responsibilities:
- **Capability Probing**: Discovers kernel Landlock LSM version, Seccomp-BPF filters, and process capabilities via `HostSandboxCapabilities::probe()`.
- **Profile Catalog**: In-memory registry storing `standard`, `strict`, `permissive`, and custom `SandboxProfile` instances with capacity bounds (`MAX_PROFILES_IN_SERVICE = 256`).
- **Supervised Dispatch**: Executes commands with working directory resolution, environment variable allowlisting and injection, output stream truncation (`max_output_capture_bytes`), and execution time measurement.
- **Audit Integration**: Writes an immutable cryptographic record to the SQLite WAL `audit_ring` table with `tool="sandbox"` and attaches the row hash to the result envelope.

---

## 2. API & Usage Examples

### A. Rust Core Service API
```rust
use aiosh_core::audit::AuditRing;
use aiosh_core::sandbox_data_model::{SandboxExecutionRequest, SandboxProfile};
use aiosh_core::sandbox_service::{SandboxConfig, SandboxService};

// 1. Initialize service with SQLite WAL audit ring
let ring = AuditRing::open_in_memory()?;
let mut service = SandboxService::with_default_profiles(Some(ring));

// 2. Prepare execution request
let request = SandboxExecutionRequest {
    command: "/bin/ls".into(),
    args: vec!["-l".into(), "/tmp".into()],
    cwd: Some("/tmp".into()),
    profile: SandboxProfile::standard(),
    session_id: Some("session-101".into()),
    pep_grant_id: Some("pep-grant-xyz".into()),
    stdin_data: None,
};

// 3. Execute under containment
let result = service.execute(&request)?;
println!("Exit code: {}", result.exit_code);
println!("Duration: {} ms", result.duration_ms);
println!("Audit hash: {:?}", result.audit_hash);
```

### B. MCP Tool Call Examples
```json
// Query registered profiles
{
  "name": "aios.sandbox.profiles",
  "arguments": {}
}

// Probe host kernel capabilities
{
  "name": "aios.sandbox.probe",
  "arguments": {}
}
```

---

## 3. Constraints & Operational Limitations
1. **Host Kernel Prerequisites**:
   - Landlock LSM requires Linux kernel version 5.13 or newer. On non-Linux systems or containers lacking `CAP_SYS_ADMIN`, the runtime honestly reports Landlock and Seccomp as `unavailable` while executing with process-level fallback controls.
2. **Buffer Limits**:
   - Captured stdout and stderr streams are capped at `max_output_capture_bytes` (default: 10 MiB) to avoid out-of-memory errors.
3. **Protected Profiles**:
   - Factory profiles (`standard`, `strict`, `permissive`) cannot be deleted.
