# T-01301: Init & Service Supervision - Data Model: Research

## Metadata
- **Task ID:** `T-01301`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision (`code/aiosh-rust/aiosh-core::service`)
- **Status:** Complete

---

## 1. Executive Overview & Mission Context
The **Init & Service Supervision** subsystem in AIOS is responsible for system initialization, process supervision, daemon lifecycle management (start, stop, restart, reload, enable, disable), health monitoring, and dependency resolution. In the AIOS architecture, autonomous AI agents and human operators interact with system services through deterministic, policy-governed interfaces (CLI `aiosh service` and MCP `aios.service.*`), backed by strict non-repudiation audit logging (ADR-0035/ADR-0036).

### Prior Art & Existing AIOS Context
- **Distribution Profiles (`distro.rs`)**: Explicitly defines `InitSystem` with variants `Systemd`, `OpenRC`, and `None`. Debian 12 targets `Systemd`; Alpine 3.19 targets `OpenRC`.
- **Rootfs Assembly (`base_image.rs`)**: Boots into PID 1 init binary (`/sbin/init` or `/lib/systemd/systemd`).
- **Package Management (`package.rs`, `package_service.rs`)**: Successfully established the pattern for multi-backend abstraction (Debian/APT vs Alpine/APK), bounded stores, and formal validation invariants (`PM1..PM5`).
- **Audit Logging (`audit.rs`)**: Consequential service state changes (starting/stopping daemons, modifying service definitions) must emit audit log events to SQLite WAL ring buffer.

---

## 2. Authoritative Sources & Upstream Standards

1. **systemd System and Service Manager (`systemd.service(5)`, `systemd.unit(5)`)**:
   - Unit naming syntax: `^[a-zA-Z0-9_.-]+(\.[a-zA-Z0-9]+)$` (e.g. `myservice.service`, `dbus.socket`).
   - Unit sections: `[Unit]` (Description, Documentation, Requires, Wants, After, Before, Conflicts), `[Service]` (Type, ExecStart, ExecStop, ExecReload, Restart, User, Group, WorkingDirectory, Environment), `[Install]` (WantedBy, RequiredBy).
   - Service execution types: `simple`, `exec`, `forking`, `oneshot`, `dbus`, `notify`, `idle`.
   - Restart behaviors: `no`, `always`, `on-success`, `on-failure`, `on-abnormal`, `on-watchdog`, `on-abort`.
   - Control API: `org.freedesktop.systemd1.Manager` via D-Bus / SD-Bus (`StartUnit`, `StopUnit`, `RestartUnit`, `ReloadUnit`, `GetUnit`).

2. **OpenRC Service Supervisor (Alpine Linux / Gentoo)**:
   - Runscript specification in `/etc/init.d/*` with standard functions: `start()`, `stop()`, `status()`, `restart()`.
   - Runlevel management: `rc-update add <service> <runlevel>` (`default`, `boot`, `sysinit`, `shutdown`).
   - Dependency declarations: `need` (strict requirement), `use` (optional requirement), `before`, `after`, `provide`.
   - Supervision backends: `supervise-daemon`, `s6`, or process polling.

3. **POSIX.1-2017 & Linux Kernel Process Semantics**:
   - PID 1 responsibilities: Process reaping (`waitpid`), signal routing (`SIGCHLD`, `SIGTERM`, `SIGINT`, `SIGHUP`), orphan reparenting.
   - Process sandboxing & cgroups v2: Namespaces (PID, Mount, Net, IPC, User), resource accounting (CPU, memory, file descriptors), standard stream redirection.

4. **NIST SP 800-53 (AC-6 Least Privilege, AU-2 Audit Events)**:
   - Services must execute under dedicated least-privilege service accounts (`User=`, `Group=`).
   - All daemon lifecycle transitions must generate non-repudiation audit trails.

---

## 3. Facts vs. Assumptions

| Fact | Assumption |
|---|---|
| AIOS supports both Debian 12 (systemd) and Alpine 3.19 (OpenRC) base operating environments. | A unified, canonical data model can model both systemd services and OpenRC services without loss of operational safety. |
| In production, PID 1 supervises system services; in containerized/CLI environments, AIOS may supervise user-space daemon processes. | The data model can represent both host-level system services and in-tree AIOS companion daemons (e.g., `aiosh-mcp`, `aios-securityd`). |
| Unbounded service inputs or unvalidated `exec_start` strings introduce severe command injection and denial-of-service vectors. | Enforcing strict syntactic constraints (`SS1..SS5`) on service definitions will prevent command injection and cyclic dependency deadlocks. |
| Service state changes are consequential administrative actions requiring ADR-0035 audit records. | Service queries (`aiosh service list`, `aios.service.get`) are non-consequential and do not require audit logging. |
| The Rust `aiosh-core` library enforces zero unhandled panics and deterministic error envelopes. | The service data model can be serialized and deserialized via `serde` with bounded memory footprints ($\le 10$ MiB). |

---

## 4. Proposed Data Model & Invariants

### Core Types (`code/aiosh-rust/aiosh-core/src/service.rs`)
1. **`ServiceType`**: Enum (`Simple`, `Exec`, `Forking`, `Oneshot`, `Notify`, `Idle`).
2. **`ServiceState`**: Enum (`Active`, `Inactive`, `Activating`, `Deactivating`, `Failed`, `Reloading`, `Unknown`).
3. **`ServiceRestartPolicy`**: Enum (`No`, `Always`, `OnSuccess`, `OnFailure`, `OnAbnormal`, `OnWatchdog`, `OnAbort`).
4. **`ServiceStartupMode`**: Enum (`Enabled`, `Disabled`, `Masked`, `Static`).
5. **`ServiceDependencyType`**: Enum (`Requires`, `Wants`, `After`, `Before`, `Conflicts`).
6. **`ServiceDependency`**: Struct (`name: String`, `dependency_type: ServiceDependencyType`, `optional: bool`).
7. **`ServiceHealth`**: Struct (`healthy: bool`, `exit_code: Option<i32>`, `pid: Option<u32>`, `uptime_seconds: Option<u64>`, `restarts: u32`, `last_error: Option<String>`).
8. **`ServiceSpec`**: Comprehensive specification of a managed service:
   - `name: String` (validated identifier, e.g. `aios-securityd.service`)
   - `description: String`
   - `exec_start: String` (validated command line)
   - `exec_stop: Option<String>`
   - `exec_reload: Option<String>`
   - `service_type: ServiceType`
   - `restart_policy: ServiceRestartPolicy`
   - `startup_mode: ServiceStartupMode`
   - `user: Option<String>`
   - `group: Option<String>`
   - `working_dir: Option<String>`
   - `environment: BTreeMap<String, String>`
   - `dependencies: Vec<ServiceDependency>`
   - `timeout_start_secs: u64`
   - `timeout_stop_secs: u64`
9. **`ServiceStatus`**: Runtime representation of a service:
   - `name: String`
   - `state: ServiceState`
   - `startup_mode: ServiceStartupMode`
   - `pid: Option<u32>`
   - `health: ServiceHealth`
   - `started_at: Option<String>`
10. **`ServiceAction`**: Enum (`Start`, `Stop`, `Restart`, `Reload`, `Enable`, `Disable`, `Mask`, `Unmask`).
11. **`ServiceQuery`**: Filter parameters for listing and querying services (`name_pattern`, `state`, `startup_mode`, `limit`).

### Service Supervision Invariants (`SS1..SS5`)
- **`SS1` (Service Naming Syntax)**:
  - Service names must match `^[a-zA-Z0-9][a-zA-Z0-9_.-]*(\.service)?$`.
  - Length must be between 1 and 128 characters.
  - Prohibited: whitespace, forward/backward slashes, null bytes, shell metacharacters (`;`, `&`, `|`, `>`, `<`, `$`).
- **`SS2` (Execution Command Bounds)**:
  - `exec_start` must not be empty; length bounded between 1 and 4096 characters.
  - Optional `exec_stop` and `exec_reload` bounded $\le 4096$ characters.
  - Optional `working_dir` must be an absolute path (begins with `/` or drive letter) and must not contain directory traversal sequences (`..`).
- **`SS3` (Dependency Hygiene & Acyclicity)**:
  - A service cannot depend on itself (`dep.name != service.name`).
  - Duplicate dependencies are prohibited within a single `ServiceSpec`.
  - Dependency names must satisfy `SS1`.
- **`SS4` (Resource & Limit Bounds)**:
  - `timeout_start_secs` and `timeout_stop_secs` must be $\ge 1$ and $\le 86400$ (24 hours).
  - Maximum 128 dependencies per service.
  - Maximum 256 environment variables; keys $\le 256$ chars, values $\le 4096$ chars.
  - Description length $\le 4096$ characters.
- **`SS5` (State Consistency & Lifecycle)**:
  - If `state == ServiceState::Active`, `Activating`, or `Reloading` on a live supervised process, `pid` should be present when known.
  - If `state == ServiceState::Failed`, `health.healthy` must evaluate to `false`.
  - Masked services (`startup_mode == ServiceStartupMode::Masked`) cannot be enabled.

---

## 5. Decisions Needed Before Specification (`T-01302`)
1. **Module Location**:
   - Create `code/aiosh-rust/aiosh-core/src/service.rs` and expose `pub mod service;` in `lib.rs`.
2. **Naming Convention for Invariants**:
   - Adopt `SS1..SS5` (Service Supervision Invariants) matching the established `PM1..PM5` pattern from Package Management.
3. **Validation Functions**:
   - Provide `validate_service_name(name: &str) -> Result<(), String>`
   - Provide `validate_service_spec(spec: &ServiceSpec) -> Result<(), String>`
4. **Zero External Dependencies**:
   - Keep data model self-contained in `aiosh-core` using existing workspace crates (`serde`, `chrono`). No external D-Bus or systemd C libraries required for the data model.
