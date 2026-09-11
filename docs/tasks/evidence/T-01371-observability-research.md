# T-01371: Init & Service Supervision - Observability: Research

## Metadata
- **Task ID:** `T-01371`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Observability
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Research Overview & Prior Art
Observability and runtime telemetry are critical for operators and autonomous AI agents managing the AIOS operating system. A supervisor must provide complete visibility into the status of all managed daemons, their execution states, startup enablement modes, health telemetry, restart frequencies, dependency complexity, and security policy compliance.

### Authoritative Upstream Standards
1. **systemd Supervision Telemetry (`systemctl(1)`, `systemd-analyze(1)`)**:
   - `systemctl list-units --type=service`: Provides state breakdowns (`active`, `inactive`, `failed`, `activating`, `deactivating`).
   - `systemctl list-unit-files --type=service`: Reports unit enablement modes (`enabled`, `disabled`, `masked`, `static`).
   - `systemctl is-system-running`: Synthesizes global system operational health based on individual service states (degraded when services are in `failed` state).
2. **OpenRC Service Status (`rc-status(8)`)**:
   - Runlevel inspection and crashed service detection (`crashed`, `started`, `stopped`, `inactive`).
3. **Prometheus Node Exporter & OpenTelemetry (OTel)**:
   - `node_systemd_unit_state`: Gauge of services in each state.
   - `process_restart_total`: Counter tracking process restart loops and flapping daemons.
   - Standard semantic attributes: `service.name`, `service.state`, `service.version`.
4. **AIOS System Architecture (ADR-0035 & AI Constitution)**:
   - Deterministic, read-only telemetry reports serialized in canonical JSON.
   - Integration with `ServiceStore` and `ServiceSecurityPolicy`.
   - Immutable audit logging of administrative telemetry queries via the SQLite WAL ring.

---

## 2. Facts vs. Assumptions

### Established Facts
1. **Fact**: `ServiceStore` (`code/aiosh-rust/aiosh-core/src/service_service.rs`) maintains the in-memory registry of all `ServiceSpec` instances and their associated `ServiceStatus` and `ServiceHealth`.
2. **Fact**: `ServiceSecurityPolicy` (`code/aiosh-rust/aiosh-core/src/service_policy.rs`) provides `evaluate_spec` and `evaluate_store` to calculate security verdicts and violations.
3. **Fact**: Subsystems such as package management (`package_observability.rs`), base image (`base_image_observability.rs`), and distro (`distro_observability.rs`) established a standard observability report pattern:
   - A dedicated report struct (`*ObservabilityReport`) containing inventory counts, categorical distributions (`BTreeMap<String, usize>`), health summaries, and policy compliance.
   - `generate(&store, policy_opt)` and `generate_from_paths(store_path, policy_path)` constructors.
   - Serialization to pretty-printed JSON (`to_json_pretty()`).
4. **Fact**: No source code was modified during this research task.

### Assumptions
1. **Assumption**: `ServiceObservabilityReport` will be implemented in `code/aiosh-rust/aiosh-core/src/service_observability.rs` and re-exported via `lib.rs`.
2. **Assumption**: The report will establish criteria `SO1..SO6`:
   - `SO1`: Inventory completeness (`total_services == sum(state_breakdown) == sum(startup_mode_breakdown)`).
   - `SO2`: State and mode categorical distributions.
   - `SO3`: Health metrics and process restart telemetry aggregation.
   - `SO4`: Dependency complexity distribution histogram.
   - `SO5`: Security policy compliance evaluation.
   - `SO6`: Deterministic formatting, size bounds, and audit emission.
3. **Assumption**: Surface integrations:
   - Operator CLI: `aiosh service stats [--store <path>] [--policy <path>] [--json]` or `aiosh service observability`.
   - Autonomous Agent MCP: `aios.service.stats` / `aios.service.observability`.
   - Standalone test suite: `tools/test_service_suites.py` criterion `SS8`.

---

## 3. Unknowns & Decisions Needed

1. **Decision**: Command naming and MCP tool naming:
   - For CLI: `aiosh service stats` (following `aiosh package stats` convention) with alias `aiosh service observability`.
   - For MCP: `aios.service.stats` (mirroring `aios.package.stats`).
2. **Decision**: Metric breakdowns to include:
   - State breakdown (`active`, `inactive`, `activating`, `deactivating`, `failed`, `reloading`, `unknown`).
   - Startup mode breakdown (`enabled`, `disabled`, `masked`, `static`).
   - Service type breakdown (`simple`, `exec`, `forking`, `oneshot`, `notify`, `idle`).
   - Restart policy breakdown (`no`, `always`, `on_success`, `on_failure`, etc.).
   - Health summary (`healthy_count`, `unhealthy_count`, `total_restarts`, `failed_services`).
   - Dependency distribution histogram (`0`, `1-2`, `3-5`, `6+`).
   - Policy compliance summary (`policy_compliant_count`, `policy_violations_count`, `prohibited_services_found`).
