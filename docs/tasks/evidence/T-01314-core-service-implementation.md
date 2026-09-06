# T-01314: Init & Service Supervision - Core Service: Implementation

## Metadata
- **Task ID:** `T-01314`
- **Subsystem:** `code/aiosh-rust/aiosh-core::service_service`
- **Component:** Init & Service Supervision Core Service Implementation
- **Status:** Complete

## 1. Implementation Overview
Implemented the production `ServiceStore` and `ServiceActionReport` data structures and algorithms in `code/aiosh-rust/aiosh-core/src/service_service.rs`, realizing the five core service invariants `CS1..CS5`:

1. **Registry & In-Memory Store (`CS1`)**:
   - `ServiceStore` encapsulates `services: BTreeMap<String, ServiceSpec>` and `statuses: BTreeMap<String, ServiceStatus>`.
   - Pre-seeded with 6 canonical system services: `auditd.service`, `dbus.service`, `systemd-journald.service`, `network-manager.service`, `aios-securityd.service`, and `ssh.service`.
   - `register_service(&mut self, spec: ServiceSpec) -> Result<(), String>` validates incoming specs against `validate_service_spec` and rejects duplicate service names.
   - `unregister_service(&mut self, name: &str) -> Result<ServiceSpec, String>` removes service specifications and runtime statuses.

2. **Lifecycle Finite State Machine (`CS2`)**:
   - `execute_action(&mut self, service_name: &str, action: ServiceAction) -> Result<ServiceActionReport, String>`:
     - `Start`: Strictly rejects `Masked` services. Transitions state to `Active`, sets `health.healthy = true`, assigns PID (1001), clears previous errors, and records `started_at` timestamp.
     - `Stop`: Transitions state to `Inactive`, clears PID, uptime, and `started_at`.
     - `Restart`: Strictly rejects `Masked` services. Reboots to `Active`, assigns new PID (1002), increments `health.restarts`, and updates `started_at`.
     - `Reload`: Requires the service to be in `Active` state; errors if inactive or failed.
     - `Enable`: Updates `startup_mode` on both `ServiceSpec` and `ServiceStatus` to `Enabled`. Rejects masked units.
     - `Disable`: Updates `startup_mode` on both `ServiceSpec` and `ServiceStatus` to `Disabled`. Rejects masked units.
     - `Mask`: Strictly forbidden if the service is currently `Active` or `Activating` (must be stopped first). Sets `startup_mode` to `Masked`.
     - `Unmask`: Resets `Masked` services back to `Disabled`.
   - Returns structured `ServiceActionReport` containing previous state, new state, action, success flag, timestamp, and optional error message.

3. **Topological Dependency Resolution (`CS3`)**:
   - `plan_service_order(&self, target_service: &str) -> Result<Vec<String>, String>`:
     - Recursively traverses dependency declarations (`Requires`, `Wants`, `After`, `Before`) to compute the transitive dependency closure.
     - Validates that all required dependencies exist in the store (or skips optional dependencies).
     - Builds directed adjacency graph where dependencies precede dependents.
     - Executes Kahn's algorithm with zero-in-degree queue to produce deterministic startup sequence.
     - Detects cycles by verifying closure cardinality, returning `invariant CS3 violated: cyclic dependency detected` upon cycle detection.

4. **Multi-Criteria Query Engine**:
   - `query(&self, query: &ServiceQuery) -> Vec<&ServiceSpec>`:
     - Case-insensitive substring matching on service name and description (`name_pattern`).
     - State filtering (`state: Option<ServiceState>`).
     - Startup mode filtering (`startup_mode: Option<ServiceStartupMode>`).
     - Optional result pagination limit (`limit: Option<usize>`).

5. **Atomic Persistence & Resource Limits (`CS5`)**:
   - `save_to_path(&self, path: &Path) -> Result<(), String>`:
     - Serializes store as formatted JSON.
     - Writes to temporary file `<path>.tmp` and renames atomically to target path.
     - Enforces permissions `0o644` on Unix platforms.
   - `load_from_path(path: &Path) -> Result<ServiceStore, String>`:
     - Enforces 10 MiB disk read ceiling.
     - Enforces 10,000 maximum entity count.
     - Runs validation passes over all loaded specs (`validate_service_spec`) and statuses (`validate_service_status`).

## 2. Verification
- `cargo check`: Zero warnings, zero errors across entire workspace (`aiosh-core`, `aiosh-mcp`, `aiosh-cli`, `aiosh-sandbox`).
- `cargo test`: Full test suite passed (76 tests in integration test binaries, 10 tests in `aiosh-mcp`, all green).
