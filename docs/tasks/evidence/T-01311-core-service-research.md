# T-01311: Init & Service Supervision - Core Service: Research

## Metadata
- **Task ID:** `T-01311`
- **Subsystem:** `code/aiosh-rust/aiosh-core::service_service`
- **Component:** Init & Service Supervision Core Service Research
- **Status:** Complete

## 1. Executive Overview & Prior Art
The Core Service of Init & Service Supervision provides the in-memory state store, service unit registry, query evaluation engine, lifecycle action processor, topological dependency resolver, and atomic persistence engine for system services across AIOS Linux distributions.

### Prior Substrate Patterns
- `code/aiosh-rust/aiosh-core/src/package_service.rs`: `PackageStore` providing registry management, query filtering, deterministic transaction planning, atomic disk persistence with temporary file renaming, and 10 MiB stream read ceilings.
- `code/aiosh-rust/aiosh-core/src/base_image_service.rs`: `ImageStore` with manifest registries, build plan synthesis, and persistence.
- `code/aiosh-rust/aiosh-core/src/distro_service.rs`: `DistroStore` with multi-criteria profile evaluations and disk round-trips.
- `code/aiosh-rust/aiosh-core/src/service.rs`: Canonical data models (`ServiceSpec`, `ServiceStatus`, `ServiceHealth`, `ServiceType`, `ServiceState`, `ServiceRestartPolicy`, `ServiceStartupMode`, `ServiceDependencyType`, `ServiceDependency`, `ServiceAction`, `ServiceQuery`) and invariants `SS1..SS5`.

## 2. Authoritative Sources & Upstream Concepts
1. **systemd Service Manager & Unit File Architecture (`systemd.service(5)`, `systemd.unit(5)`, `org.freedesktop.systemd1(5)`)**:
   - Canonical unit file structures, dependency directives (`Requires`, `Wants`, `After`, `Before`, `Conflicts`), and unit execution lifecycle.
   - Unit enablement mechanics: symlink creation in `.wants` / `.requires` target folders; unit masking by symlinking to `/dev/null`.
   - D-Bus management APIs: discrete commands (`StartUnit`, `StopUnit`, `RestartUnit`, `ReloadUnit`, `EnableUnitFiles`, `MaskUnitFiles`).
2. **OpenRC Service Supervision & Dependency Engine (`openrc(8)`, `rc-service(8)`, `runscript(8)`)**:
   - Directed acyclic graph (DAG) dependency resolution (`need`, `use`, `after`, `before`) across runlevels (`sysinit`, `boot`, `default`, `nonetwork`, `shutdown`).
   - Runlevel state tracking and deterministic service start sequences.
3. **Reproducible Service Operations & State Machine Invariants**:
   - State transition hygiene: active services cannot be masked without stopping; failed services track restart attempts and last error strings.
   - Topological sorting: dependencies must be resolved into an ordered execution schedule to eliminate activation races.

## 3. Facts vs. Assumptions

| Fact | Assumption |
|---|---|
| Service supervision commands (`start`, `stop`, `restart`, `enable`, `disable`, `mask`) must enforce strict state transitions to avoid system corruption or deadlock. | A deterministic in-memory `ServiceStore` providing simulated and tracked state transitions allows autonomous AI agents to evaluate service impacts safely prior to host execution. |
| In-tree services in `aiosh-core` persist state as canonical JSON with atomic renaming and bounded stream sizes. | Persisting the service registry at `/var/lib/aios/services/service_store.json` provides an auditable, offline-queryable catalog of managed services. |
| Complex service dependency graphs can form cycles or refer to non-existent units. | Integrating Kahn's topological sort algorithm in `plan_service_order` detects circular dependencies and computes deterministic start/stop ordering. |

## 4. Proposed Core Service Architecture (`service_service.rs`)

### Core Structures:
1. `ServiceStore`:
   - `services: BTreeMap<String, ServiceSpec>`
   - `statuses: BTreeMap<String, ServiceStatus>`
   - `store_path: Option<PathBuf>`
2. `ServiceActionReport`:
   - `service_name: String`
   - `action: ServiceAction`
   - `previous_state: ServiceState`
   - `new_state: ServiceState`
   - `success: bool`
   - `error: Option<String>`
   - `timestamp: String`

### Proposed Core Service Invariants (CS1..CS5):
- **`CS1` (Registry Uniqueness & Integrity)**: Every service in the store has a unique identifier conforming to SS1 naming syntax.
- **`CS2` (Lifecycle State Transitions)**: Actions follow strict state machine transitions (`Start`: Inactive/Failed $\to$ Active; `Stop`: Active $\to$ Inactive; `Restart`: any $\to$ Active with restart count increment; `Mask`: Inactive/Disabled $\to$ Masked; `Unmask`: Masked $\to$ Disabled).
- **`CS3` (Topological Dependency Ordering)**: Service dependency resolution computes an ordered start list using DAG topological sort, rejecting cycles and verifying dependency existence.
- **`CS4` (Telemetry Consistency)**: Runtime status consistently reflects health, process information, restart counters, and ISO-8601 timestamps; `healthy == false` on `Failed`.
- **`CS5` (Atomic Persistence & Resource Limits)**: Atomic disk writes via `.tmp` file and rename; disk read ceiling enforced at 10 MiB with max 10,000 service entities.

## 5. Decisions Needed Before Implementation
1. **Module Name**: `code/aiosh-rust/aiosh-core/src/service_service.rs`.
2. **Canonical Seeding**: Seed default `ServiceStore::new()` with essential reference daemons (`aios-securityd.service`, `auditd.service`, `dbus.service`, `systemd-journald.service`, `network-manager.service`, `ssh.service`).
3. **Execution Model**: The core service executes state transitions in-memory with validation and report emission; physical host D-Bus and process binding are mapped to future integration sub-epics.
