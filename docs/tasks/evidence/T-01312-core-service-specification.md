# T-01312: Init & Service Supervision - Core Service: Specification

## Metadata
- **Task ID:** `T-01312`
- **Subsystem:** `code/aiosh-rust/aiosh-core::service_service`
- **Component:** Init & Service Supervision Core Service Specification
- **Status:** Complete

## 1. Scope & Core Contracts
This specification defines the registry, query engine, lifecycle state machine, topological dependency resolution, and atomic persistence mechanisms for the AIOS Init & Service Supervision Core Service (`ServiceStore`).

### Reused Interfaces
- Types from `aiosh_core::service`: `ServiceSpec`, `ServiceStatus`, `ServiceHealth`, `ServiceType`, `ServiceState`, `ServiceRestartPolicy`, `ServiceStartupMode`, `ServiceDependencyType`, `ServiceDependency`, `ServiceAction`, `ServiceQuery`, and validators `validate_service_spec`, `validate_service_status`, `validate_service_name`.
- Standard library: `std::collections::BTreeMap`, `std::path::Path`, `chrono::Utc`.

### New Data Structures (`code/aiosh-rust/aiosh-core/src/service_service.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceActionReport {
    pub service_name: String,
    pub action: ServiceAction,
    pub previous_state: ServiceState,
    pub new_state: ServiceState,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStore {
    pub services: BTreeMap<String, ServiceSpec>,
    pub statuses: BTreeMap<String, ServiceStatus>,
}
```

### Method Contracts on `ServiceStore`:
1. `new() -> Self`: Initializes store seeded with canonical reference services for AIOS (`aios-securityd.service`, `auditd.service`, `dbus.service`, `systemd-journald.service`, `network-manager.service`, `ssh.service`).
2. `empty() -> Self`: Initializes an empty store without pre-seeded entries.
3. `list_services(&self) -> Vec<&ServiceSpec>`: Returns all service specifications sorted by name.
4. `get_service(&self, name: &str) -> Option<&ServiceSpec>`: Exact lookup by service name.
5. `get_status(&self, name: &str) -> Option<&ServiceStatus>`: Exact lookup of runtime status by service name.
6. `register_service(&mut self, spec: ServiceSpec) -> Result<(), String>`:
   - Validates `spec` using `validate_service_spec`.
   - Rejects registration if a service with the same name already exists (`CS1`).
   - Automatically initializes a corresponding `ServiceStatus` (defaults to `Inactive`, healthy).
7. `unregister_service(&mut self, name: &str) -> Result<ServiceSpec, String>`:
   - Removes and returns existing service specification and associated status, or returns error if not found.
8. `query(&self, query: &ServiceQuery) -> Vec<&ServiceSpec>`:
   - Filters services by name pattern (case-insensitive substring), service type, runtime state, and startup mode.
   - Applies optional `limit` boundary ($[1 \dots 10,000]$).
9. `execute_action(&mut self, service_name: &str, action: ServiceAction) -> Result<ServiceActionReport, String>`:
   - Verifies target service exists in store.
   - Enforces lifecycle state transitions (`CS2`):
     - `Start`: Rejects if `Masked`. Transitions from `Inactive | Failed` to `Active`. Sets `healthy = true` and clears `last_error`.
     - `Stop`: Transitions from `Active` to `Inactive`.
     - `Restart`: Transitions to `Active`, increments `restarts` counter, sets `healthy = true`.
     - `Reload`: Valid only if service is currently `Active`.
     - `Enable`: Transitions `startup_mode` to `Enabled`.
     - `Disable`: Transitions `startup_mode` to `Disabled`.
     - `Mask`: Rejects if service is currently `Active` (must stop first). Transitions `startup_mode` to `Masked`.
     - `Unmask`: Transitions `startup_mode` from `Masked` to `Disabled`.
   - Emits structured `ServiceActionReport`.
10. `plan_service_order(&self, target_service: &str) -> Result<Vec<String>, String>`:
    - Resolves dependency closure for `target_service` across all directed dependencies (`Requires`, `After`).
    - Executes Kahn's topological sort algorithm (`CS3`).
    - Detects and rejects cyclic dependency graphs with clear diagnostic messages.
11. `save_to_path(&self, path: &Path) -> Result<(), String>`:
    - Atomically persists store to disk via temporary file rename (`<path>.tmp`) with `0o644` permissions (`CS5`).
12. `load_from_path(path: &Path) -> Result<ServiceStore, String>`:
    - Enforces 10 MiB (`10,485,760` bytes) file size ceiling and max 10,000 entity count ceiling (`CS5`).
    - Deep-validates all loaded specs and statuses against invariants.

## 2. Invariants (CS1..CS5)
- **`CS1` (Registry Uniqueness & Integrity)**: Every entry in `services` is uniquely keyed by `spec.name` conforming to SS1 syntax.
- **`CS2` (Lifecycle State Transitions)**: Discrete state transitions follow the strict finite state machine; illegal transitions (e.g. masking an active service) fail closed.
- **`CS3` (Topological Dependency Ordering)**: Dependency schedules ensure that required dependencies are ordered before targets with verified acyclicity.
- **`CS4` (Telemetry Consistency)**: Runtime `ServiceStatus` reports consistent health, PID, restart counters, and RFC-3339 timestamps.
- **`CS5` (Persistence Atomicity & Resource Ceilings)**: All file writes are atomic; file loads strictly reject streams > 10 MiB or > 10,000 entities.

## 3. Failure Envelopes & Audit Effects
- Operational failures return explicit, structured error messages (`Result<T, String>`).
- State mutations through CLI and MCP surfaces emit immutable SHA-256 hash-chained audit events into the SQLite WAL ring (`audit.db`).
