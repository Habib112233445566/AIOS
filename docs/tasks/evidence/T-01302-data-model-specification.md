# T-01302: Init & Service Supervision - Data Model: Specification

## Metadata
- **Task ID:** `T-01302`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision (`code/aiosh-rust/aiosh-core::service`)
- **Status:** Complete

---

## 1. Scope & System Interfaces
This specification establishes the concrete data structures, lifecycle state enumerations, validation functions, and persistence contracts for the AIOS Init & Service Supervision data model.

### Reused Interfaces
- `serde::{Serialize, Deserialize}`: Standard JSON serialization and deserialization.
- `std::collections::BTreeMap`: Deterministic key-value ordering for environment variables.
- Standard ISO 8601 timestamps (`chrono::Utc`) for service activation tracking and audit records.
- Fail-closed error envelopes: `{"code": "<ERROR_CODE>", "data": ..., "error": "<MESSAGE>"}`.
- Non-repudiation audit ring buffer (`code/aiosh-rust/aiosh-core/src/audit.rs`) for state-changing operations.

### Upstream Alignment & AIOS Specifics
- **Upstream Alignment**: Modeled on standard `systemd.service(5)` unit directives (Type, Restart, ExecStart, TimeoutStartSec, Requires, Wants, After) and OpenRC runscript lifecycle semantics (need, use, start, stop, status).
- **AIOS Specifics**: Marked with `#[serde(rename_all = "snake_case")]` for uniform JSON-RPC / MCP representations; strict entity bounds and size limits enforced at deserialize/validation time.

---

## 2. Core Data Structures (`code/aiosh-rust/aiosh-core/src/service.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Service execution architecture type (aligned with systemd Service Type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    Simple,
    Exec,
    Forking,
    Oneshot,
    Notify,
    Idle,
}

/// Runtime lifecycle state of a managed service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    Active,
    Inactive,
    Activating,
    Deactivating,
    Failed,
    Reloading,
    Unknown,
}

/// Restart policy for the service process upon termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceRestartPolicy {
    No,
    Always,
    OnSuccess,
    OnFailure,
    OnAbnormal,
    OnWatchdog,
    OnAbort,
}

/// Startup enablement mode (aligned with systemd unit enablement).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStartupMode {
    Enabled,
    Disabled,
    Masked,
    Static,
}

/// Relationship type between services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceDependencyType {
    Requires,
    Wants,
    After,
    Before,
    Conflicts,
}

/// Directed dependency on another service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceDependency {
    pub name: String,
    pub dependency_type: ServiceDependencyType,
    pub optional: bool,
}

/// Health and process accounting telemetry for a service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub healthy: bool,
    pub exit_code: Option<i32>,
    pub pid: Option<u32>,
    pub uptime_seconds: Option<u64>,
    pub restarts: u32,
    pub last_error: Option<String>,
}

/// Canonical specification of a managed service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub name: String,
    pub description: String,
    pub exec_start: String,
    pub exec_stop: Option<String>,
    pub exec_reload: Option<String>,
    pub service_type: ServiceType,
    pub restart_policy: ServiceRestartPolicy,
    pub startup_mode: ServiceStartupMode,
    pub user: Option<String>,
    pub group: Option<String>,
    pub working_dir: Option<String>,
    pub environment: BTreeMap<String, String>,
    pub dependencies: Vec<ServiceDependency>,
    pub timeout_start_secs: u64,
    pub timeout_stop_secs: u64,
}

/// Runtime status snapshot of a service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub state: ServiceState,
    pub startup_mode: ServiceStartupMode,
    pub pid: Option<u32>,
    pub health: ServiceHealth,
    pub started_at: Option<String>,
}

/// Administrative action performed on a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
    Reload,
    Enable,
    Disable,
    Mask,
    Unmask,
}

/// Query filter for listing and discovering services.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceQuery {
    pub name_pattern: Option<String>,
    pub state: Option<ServiceState>,
    pub startup_mode: Option<ServiceStartupMode>,
    pub limit: Option<usize>,
}
```

---

## 3. Invariants & Formal Validation Rules (SS1..SS5)

### `validate_service_name(name: &str) -> Result<(), String>`
- **`SS1.1` (Non-Empty)**: `name` must not be empty.
- **`SS1.2` (Length Bound)**: `name.len() <= 128`.
- **`SS1.3` (Initial Character)**: Must start with ASCII alphanumeric `[a-zA-Z0-9]`.
- **`SS1.4` (Allowed Alphabet)**: All subsequent characters must be `[a-zA-Z0-9_.-]`.
- **`SS1.5` (Disallowed Patterns)**: Prohibits spaces, forward/backward slashes (`/`, `\`), null bytes (`\0`), and shell metacharacters (`;`, `&`, `|`, `>`, `<`, `$`, `` ` ``, `"`).

### `validate_service_spec(spec: &ServiceSpec) -> Result<(), String>`
- **`SS1` (Identifier Validity)**: `validate_service_name(&spec.name)` must succeed.
- **`SS2` (Command & Path Validation)**:
  - `spec.exec_start` must not be empty, length $\le 4096$ characters.
  - If `spec.exec_stop` is present, length $\le 4096$ characters.
  - If `spec.exec_reload` is present, length $\le 4096$ characters.
  - If `spec.working_dir` is present:
    - Length $\le 1024$ characters.
    - Must be an absolute path (begins with `/` or Windows drive letter `[A-Z]:\`).
    - Must not contain path traversal tokens (`..`).
- **`SS3` (Dependency Hygiene & Acyclicity)**:
  - Count of dependencies $\le 128$.
  - No self-dependency: for every dependency `dep`, `dep.name != spec.name`.
  - No duplicate dependencies: all `dep.name` must be unique.
  - Every `dep.name` must be a valid service name per `validate_service_name`.
- **`SS4` (Resource & Field Limits)**:
  - `spec.description.len() <= 4096`.
  - `spec.timeout_start_secs` must be in range `[1, 86400]` (default 30s).
  - `spec.timeout_stop_secs` must be in range `[1, 86400]` (default 30s).
  - Environment variables: count $\le 256$, keys non-empty and $\le 256$ chars, values $\le 4096$ chars. Keys must not contain `=` or null bytes.
  - If `user` is present, length $\le 64$ chars, valid Unix username characters `[a-z_][a-z0-9_-]*`.
  - If `group` is present, length $\le 64$ chars, valid Unix group name characters `[a-z_][a-z0-9_-]*`.
- **`SS5` (State & Mode Consistency)**:
  - A service with `startup_mode == ServiceStartupMode::Masked` cannot be in `Enabled` state.
  - `validate_service_status(status: &ServiceStatus)`:
    - If `status.state == ServiceState::Failed`, `status.health.healthy` must be `false`.
    - If `status.state == ServiceState::Active`, `status.health.healthy` must be `true` (unless an explicit failing health probe is active).

---

## 4. Happy Path, Failure Paths, and Audit Effects

### Happy Path
1. Operator or autonomous agent creates a `ServiceSpec` (e.g. for `aios-securityd`).
2. `validate_service_spec(&spec)` validates `SS1..SS5` and returns `Ok(())`.
3. The spec is serialized to bounded JSON ($\le 10$ MiB) and saved to disk or registered in the in-memory service registry.

### Failure Paths
1. **Invalid Naming**: Name contains slashes, spaces, or exceeds 128 characters $\to$ fails `SS1` with `Err("service name contains forbidden characters...")`.
2. **Path Traversal in Working Directory**: `working_dir = Some("/var/lib/../etc")` $\to$ fails `SS2` with `Err("working_dir contains path traversal sequence '..'")`.
3. **Self-Dependency**: Service `foo` lists `foo` in `dependencies` $\to$ fails `SS3` with `Err("service cannot depend on itself")`.
4. **Extreme Timeouts**: `timeout_start_secs = 0` or `100000` $\to$ fails `SS4` with `Err("timeout_start_secs out of range [1, 86400]")`.

### Audit Effects (ADR-0035 / ADR-0036)
- Read-only data model validations do not alter persistent state and do not emit audit events.
- Consequential state transitions (when integrated into `service_service.rs` in `T-01311`) will emit structured records to SQLite WAL:
  - `action`: `service.start` | `service.stop` | `service.restart` | `service.register`
  - `target`: `service_name`
  - `actor`: `operator` | `agent_id`
  - `result`: `success` | `denied` | `failed`
