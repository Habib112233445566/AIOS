# T-01303: Init & Service Supervision - Data Model: Scaffold

## Metadata
- **Task ID:** `T-01303`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision Data Model Scaffold (`code/aiosh-rust/aiosh-core::service`)
- **Status:** Complete

---

## 1. Scaffold Deliverables
Created module skeleton `code/aiosh-rust/aiosh-core/src/service.rs` and wired it into `aiosh-core` (`code/aiosh-rust/aiosh-core/src/lib.rs`).

### Defined Typed Interfaces & Data Structures:
- `ServiceType`: Enum (`Simple`, `Exec`, `Forking`, `Oneshot`, `Notify`, `Idle`) with snake_case Serde bindings.
- `ServiceState`: Enum (`Active`, `Inactive`, `Activating`, `Deactivating`, `Failed`, `Reloading`, `Unknown`).
- `ServiceRestartPolicy`: Enum (`No`, `Always`, `OnSuccess`, `OnFailure`, `OnAbnormal`, `OnWatchdog`, `OnAbort`).
- `ServiceStartupMode`: Enum (`Enabled`, `Disabled`, `Masked`, `Static`).
- `ServiceDependencyType`: Enum (`Requires`, `Wants`, `After`, `Before`, `Conflicts`).
- `ServiceDependency`: Struct (`name: String`, `dependency_type: ServiceDependencyType`, `optional: bool`).
- `ServiceHealth`: Struct (`healthy: bool`, `exit_code: Option<i32>`, `pid: Option<u32>`, `uptime_seconds: Option<u64>`, `restarts: u32`, `last_error: Option<String>`).
- `ServiceSpec`: Comprehensive unit/service definition with execution commands, restart behavior, timeouts, environment variables, and dependencies.
- `ServiceStatus`: Runtime status snapshot representing service health and execution state.
- `ServiceAction`: Enum (`Start`, `Stop`, `Restart`, `Reload`, `Enable`, `Disable`, `Mask`, `Unmask`).
- `ServiceQuery`: Search and filtering query struct (`name_pattern`, `state`, `startup_mode`, `limit`).
- Typed validation function signatures with fail-loud scaffolding stubs:
  - `validate_service_name(name: &str) -> Result<(), String>`
  - `validate_service_spec(spec: &ServiceSpec) -> Result<(), String>`
  - `validate_service_status(status: &ServiceStatus) -> Result<(), String>`

### Module Registration & Exports:
- Exported in `code/aiosh-rust/aiosh-core/src/lib.rs` under `pub mod service;` and re-exported core types.
- Included unit test `test_scaffold_types_instantiation` validating memory layout and Serde serialization/deserialization.
- Verified build and tests clean via `cargo check` and `cargo test --lib service`.
