# T-01381: Init & Service Supervision Documentation Research

**Date:** 2026-09-09  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Init & Service Supervision / Documentation  
**Task ID:** T-01381  

---

## 1. Executive Summary & Objective
Task `T-01381` establishes facts, architectural constraints, authoritative prior art, and concrete documentation requirements for the **Init & Service Supervision** subsystem. This research lays the foundation for creating a comprehensive, rot-proof architectural guide (`docs/service_supervision.md`) and its automated verification suite (`tools/test_service_doc.py`) spanning the complete epic:
- Data model & specification invariants (SS1..SS5)
- Core service registry, FSM state machine & dependency ordering (CS1..CS5)
- Operator CLI surface (`aiosh service *`)
- Autonomous Agent MCP surface (`aios.service.*`)
- Configuration resolution & hierarchy (SC1..SC7)
- Automated lifecycle & integration testing (ST1..ST5)
- Security policy & prohibited services (SP1..SP6)
- Observability telemetry & health reporting (SO1..SO6)

---

## 2. Existing Codebase Audit & Assets

### 1. Data Model (`code/aiosh-rust/aiosh-core/src/service.rs`)
- **Core Entities**: `ServiceSpec`, `ServiceStatus`, `ServiceHealth`, `ServiceType` (`Simple`, `Exec`, `Forking`, `Oneshot`, `Notify`, `Idle`), `ServiceRestartPolicy` (`No`, `Always`, `OnSuccess`, `OnFailure`, `OnAbnormal`, `OnWatchdog`, `OnAbort`), `ServiceStartupMode` (`Enabled`, `Disabled`, `Masked`, `Static`), `ServiceState` (`Active`, `Inactive`, `Activating`, `Deactivating`, `Failed`, `Reloading`, `Unknown`), `ServiceDependencyType` (`Requires`, `Wants`, `Before`, `After`, `Conflicts`), `ServiceDependency`, `ServiceAction`, `ServiceQuery`.
- **Invariants SS1..SS5**:
  - `SS1`: Service naming syntax conforming to systemd/OpenRC standard (`^[a-zA-Z0-9][a-zA-Z0-9_.-]*$`), string length bounds [1..128].
  - `SS2`: Execution commands and path traversal protection (rejects relative paths and `..`).
  - `SS3`: Dependency graph hygiene (rejects self-loops).
  - `SS4`: Timeout boundaries ($[1..86400]$s) and environment variable sizing ($\le 1024$).
  - `SS5`: Lifecycle state machine consistency.

### 2. Core Service Registry & State Machine (`code/aiosh-rust/aiosh-core/src/service_service.rs`)
- **Core Engine**: `ServiceStore` providing thread-safe in-memory registry seeded with reference services (`aios-securityd.service`, `auditd.service`, `dbus.service`, `systemd-journald.service`, `ssh.service`).
- **Invariants CS1..CS5**:
  - `CS1`: Registry uniqueness and specification validation.
  - `CS2`: Deterministic FSM lifecycle state machine (`start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`).
  - `CS3`: Topological startup execution planning using Kahn's algorithm with cycle detection.
  - `CS4`: Filtered querying by name pattern, lifecycle state, startup mode, and limit bounds.
  - `CS5`: Atomic filesystem persistence using PID-isolated tempfile rename and 10 MiB payload ceiling.

### 3. Hierarchical Configuration (`code/aiosh-rust/aiosh-core/src/service_config.rs`)
- **Resolution Engine**: `ServiceConfig` resolving across precedence: explicit file > environment variables (`AIOS_SERVICE_*`) > defaults.
- **Invariants SC1..SC7**:
  - `SC1`: Store path bounds and control character rejection.
  - `SC2`: Timeout bounds ($[1..86400]$s).
  - `SC3`: Store size capacity ceiling ($[64\text{ KiB} \dots 100\text{ MiB}]$).
  - `SC4`: Entity count bounds ($[10 \dots 100,000]$).
  - `SC5`: Auto-persistence toggle.
  - `SC6`: Restart backoff delay and max burst bounding.
  - `SC7`: Safe stream reading capped at 64 KiB.

### 4. Integration Test Suite (`code/aiosh-rust/aiosh-core/tests/test_service_automated.rs`)
- **Integration Matrix ST1..ST5**:
  - `ST1`: Deterministic FSM lifecycle transitions with state integrity.
  - `ST2`: Topological ordering with cycle detection.
  - `ST3`: Atomic persistence and reload roundtrip.
  - `ST4`: Store entity and sizing boundary limits.
  - `ST5`: Error handling and invalid transition rejection.

### 5. Security Policy Engine (`code/aiosh-rust/aiosh-core/src/service_policy.rs`)
- **Security Subsystem**: `ServiceSecurityPolicy` supporting `Enforcing`, `Audit`, and `Permissive` modes.
- **Invariants SP1..SP6**:
  - `SP1`: Policy configuration bounds and validation.
  - `SP2`: Prohibited service daemon blocking (`telnet`, `rsh`, `rlogin`, `rexec`, `tftp`, `xinetd`, `ypserv`, `ypbind`).
  - `SP3`: Executable path hygiene (rejecting `/tmp`, `/var/tmp`, `/dev/shm`, relative paths, and directory traversal `..`).
  - `SP4`: User privilege & root hygiene (`require_service_user`, `disallow_root`, `allowed_root_services`).
  - `SP5`: Environment sanitization (blocking `LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS`).
  - `SP6`: Tri-state policy modes with immutable SQLite WAL audit row emission.

### 6. Observability Telemetry (`code/aiosh-rust/aiosh-core/src/service_observability.rs`)
- **Observability Engine**: `ServiceObservabilityReport` generating deterministic telemetry snapshots.
- **Invariants SO1..SO6**:
  - `SO1`: Inventory completeness with strict mathematical conservation.
  - `SO2`: Multi-dimensional categorical breakdowns (state, mode, type, policy).
  - `SO3`: Health metrics, failed service tracking, and saturated restart arithmetic.
  - `SO4`: Bounded $O(1)$ memory dependency distribution histogram buckets (`"0"`, `"1-2"`, `"3-5"`, `"6+"`).
  - `SO5`: Security policy compliance and prohibited service detection.
  - `SO6`: Deterministic JSON serialization and audit logging.

### 7. Operational Surfaces
- **Operator CLI (`aiosh-cli`)**: 12 subcommands under `aiosh service`: `validate`, `list`, `show`/`status`, `action`, `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`, `order`, `config`, `policy`, `stats`/`observability`.
- **Autonomous Agent MCP (`aiosh-mcp`)**: 8 MCP tools under `aios.service.*`: `validate`, `list`, `get`, `action`, `order`, `config`, `policy`, `stats`, gated with PEP capability tokens and SQLite WAL audit logging.

### 8. Test Runners
- `tools/test_service_suites.py`: Validates all 8 criteria `SS1..SS8`.
- `code/aiosh-cli/tests/test_service_cli_smoke.py`: Standalone CLI smoke test suite.
- `code/aiosh-mcp/tests/test_service_mcp_smoke.py`: Standalone MCP stdio smoke test suite.

---

## 3. Authoritative Prior Art & Standards

1. **systemd Service & Unit Architecture (`systemd.service(5)`, `systemd.unit(5)`, `systemctl(1)`)**:
   - Unit file syntax, execution types (`Simple`, `Forking`, `Oneshot`, `Notify`), lifecycle dependencies (`Requires`, `Wants`, `After`, `Before`), restart conditions, and cgroup resource bounding.
2. **OpenRC Service Supervision**:
   - Runlevel management, deterministic dependency-based init sequencing, lightweight service configuration, and process supervision.
3. **POSIX 1003.1 Process Management**:
   - Process lifecycle semantics, process groups, session leaders, standard signal delivery (`SIGTERM`, `SIGKILL`, `SIGHUP`), and exit code conventions.
4. **Kahn's Algorithm for Directed Acyclic Graphs (DAG)**:
   - $O(V + E)$ topological sequencing with linear cycle detection ensuring deterministic initialization order without deadlocks.
5. **NIST SP 800-53 (Rev 5)**:
   - Security controls AC-6 (Least Privilege) and CM-7 (Least Functionality) mandating prohibition of insecure daemons and root execution minimization.
6. **AIOS Standard Result Envelope & Audit Ring (ADR-0035 / ADR-0036)**:
   - Unified `{ "code": 0, "data": ..., "error": ... }` response structure, non-repudiable SHA-256 hash-chained audit logging to SQLite WAL, and capability-gated dispatch.

---

## 4. Facts vs. Assumptions

### Facts (Empirically Verified in Codebase)
- All service names and filesystem paths reject ASCII control characters and null bytes.
- Dependency startup ordering deterministically detects dependency cycles and returns an explicit error before any state changes.
- Masked services cannot be started or enabled under any condition.
- Every state mutation and query emits an immutable audit record to `audit.db` / `audit.log`.
- Saturated arithmetic is used across restart metrics to prevent integer overflow.
- All test criteria `SS1..SS8` pass via `tools/test_service_suites.py`.

### Assumptions (To Be Codified in Documentation)
- Operators and agents require single-page authoritative reference documentation detailing all 8 sub-epics.
- Automated validation (`tools/test_service_doc.py`) should ensure documentation remains synchronized with code invariants and never rots.

---

## 5. Architectural Decisions Needed for Documentation Asset
1. **Target Document Path**: Create `docs/service_supervision.md` mirroring the high-standard structure of `docs/package_management.md`.
2. **Automated Verification**: Implement `tools/test_service_doc.py` verifying structural headers, verbatim invariant tokens (`SS1..SS5`, `CS1..CS5`, `SC1..SC7`, `SP1..SP6`, `SO1..SO6`, `ST1..ST5`), zero rot markers (`TODO`, `FIXME`, `TBD`, `XXX`), and examples.
3. **Repository Index Synchronization**: Register `docs/service_supervision.md` in `docs/README.md` and link evidence files.
4. **Master Matrix Integration**: Add criterion `SS9` to `tools/test_service_suites.py` to run the documentation verification test.
