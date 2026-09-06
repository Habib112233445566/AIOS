# T-01362: Init & Service Supervision - Security Policy: Specification

## Metadata
- **Task ID:** `T-01362`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Security Policy
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Scope & Objective
Specify the data structures, validation invariants, and evaluation contracts for the Init & Service Supervision Security Policy subsystem (`code/aiosh-rust/aiosh-core/src/service_policy.rs`).
The subsystem validates service specifications (`ServiceSpec`) and service store contents (`ServiceStore`) against mandatory security criteria before services are registered, enabled, or executed by the supervisor.

---

## 2. Data Types & Interfaces

### Policy Modes
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServicePolicyMode {
    Enforcing,
    Audit,
    Permissive,
}
```

### Security Policy Specification
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceSecurityPolicy {
    pub mode: ServicePolicyMode,
    pub prohibited_services: Vec<String>,
    pub prohibited_exec_paths: Vec<String>,
    pub disallow_root: bool,
    pub allowed_root_services: Vec<String>,
    pub require_service_user: bool,
    pub disallow_env_vars: Vec<String>,
    pub allowed_service_types: Vec<ServiceType>,
    pub max_env_vars: usize,
    pub max_timeout_secs: u64,
}
```

### Violations & Verdicts
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServicePolicyViolation {
    pub rule_id: String,
    pub service_name: String,
    pub description: String,
    pub fatal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServicePolicyVerdict {
    pub service_name: String,
    pub allowed: bool,
    pub mode: ServicePolicyMode,
    pub violations: Vec<ServicePolicyViolation>,
    pub evaluated_at: String,
}
```

---

## 3. Invariants (SP1..SP6)

### SP1: Policy Configuration Bounds
- `prohibited_services`: Maximum 1,024 entries. Each entry must follow valid service name syntax and contain no control characters or whitespace.
- `prohibited_exec_paths`: Maximum 128 entries. Each path must be non-empty and absolute.
- `disallow_env_vars`: Maximum 128 entries. Must contain valid environment variable identifier characters.
- `allowed_service_types`: Must not be empty.
- `max_env_vars`: Range $[1 \dots 1024]$.
- `max_timeout_secs`: Range $[1 \dots 86400]$.

### SP2: Prohibited Service Blocking
- Evaluates `spec.name` against `prohibited_services`.
- Matches both exact name and base name stripping `.service` suffix.
- Default prohibited list includes insecure legacy network daemons:
  - `telnet.service`, `rsh.service`, `rlogin.service`, `rexec.service`, `tftp.service`, `xinetd.service`, `ypserv.service`, `ypbind.service`.
- Generates violation `SP2-PROHIBITED-SERVICE` (fatal = true).

### SP3: Executable Path & Working Directory Hygiene
- Evaluates `exec_start`, `exec_stop`, and `exec_reload`:
  - Binary executable token (first whitespace-delimited argument) must be an absolute path.
  - Binary path must not reside within any `prohibited_exec_paths` prefix (default: `/tmp/`, `/var/tmp/`, `/dev/shm/`, `/run/user/`).
  - Neither binary path nor arguments nor `working_dir` may contain directory traversal sequences (`..`).
- Violations:
  - `SP3-PROHIBITED-PATH` (fatal = true)
  - `SP3-RELATIVE-PATH` (fatal = true)
  - `SP3-PATH-TRAVERSAL` (fatal = true)

### SP4: User Privilege & Root Restriction
- When `require_service_user == true`:
  - `spec.user` must not be `None` or empty, unless `spec.name` is in `allowed_root_services`.
  - Violation: `SP4-UNPRIVILEGED-USER-REQUIRED` (fatal = true).
- When `disallow_root == true`:
  - `spec.user` must not be `"root"` or UID 0 unless `spec.name` is explicitly listed in `allowed_root_services`.
  - Violation: `SP4-ROOT-DISALLOWED` (fatal = true).

### SP5: Environment & Parameter Sanitization
- Evaluates `spec.environment`:
  - Disallows dangerous environment variables listed in `disallow_env_vars` (defaults: `LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS`).
  - Violation: `SP5-DANGEROUS-ENV-VAR` (fatal = true).
  - Environment variable count must not exceed `max_env_vars`.
  - Violation: `SP5-ENV-COUNT-EXCEEDED` (fatal = false).
- Evaluates timeouts:
  - `spec.timeout_start_secs` and `spec.timeout_stop_secs` must not exceed `max_timeout_secs`.
  - Violation: `SP5-TIMEOUT-EXCEEDED` (fatal = false).
- Evaluates service type:
  - `spec.service_type` must be present in `allowed_service_types`.
  - Violation: `SP5-DISALLOWED-TYPE` (fatal = true).

### SP6: Tri-State Mode Semantics & Audit Effects
- `Enforcing`: If any fatal violation exists, `verdict.allowed = false`.
- `Audit`: `verdict.allowed = true` regardless of violations; all violations are captured and reported.
- `Permissive`: Non-fatal violations are ignored; `SP2-PROHIBITED-SERVICE` violations remain fatal (`allowed = false`).
- All policy evaluations emit cryptographic audit rows to SQLite WAL ring (`aiosh-core::audit`).

---

## 4. Reused vs. New Interfaces

### Reused Interfaces
- `ServiceSpec`, `ServiceType`, `ServiceState`, `ServiceStartupMode`, `ServiceRestartPolicy` (`code/aiosh-rust/aiosh-core/src/service.rs`).
- `ServiceStore` (`code/aiosh-rust/aiosh-core/src/service_service.rs`).
- `classify_and_emit` / `audit` ring (`code/aiosh-rust/aiosh-core/src/audit.rs`).

### New Interfaces
- Module `code/aiosh-rust/aiosh-core/src/service_policy.rs`.
- Structs: `ServiceSecurityPolicy`, `ServicePolicyViolation`, `ServicePolicyVerdict`, `ServicePolicyMode`.
- Methods:
  - `ServiceSecurityPolicy::validate(&self) -> Result<(), String>`
  - `ServiceSecurityPolicy::evaluate_spec(&self, spec: &ServiceSpec) -> ServicePolicyVerdict`
  - `ServiceSecurityPolicy::evaluate_store(&self, store: &ServiceStore) -> Vec<ServicePolicyVerdict>`
  - `ServiceSecurityPolicy::from_file<P: AsRef<Path>>(path: P) -> Result<Self, String>`
  - `ServiceSecurityPolicy::from_env() -> Result<Self, String>`
  - `ServiceSecurityPolicy::resolve(custom_path: Option<&str>) -> Result<Self, String>`
- CLI command: `aiosh service policy [--check] [--json] [--config <path>]`.
- MCP tool: `aios.service.policy`.
