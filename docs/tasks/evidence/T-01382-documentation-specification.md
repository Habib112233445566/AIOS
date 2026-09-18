# T-01382: Init & Service Supervision Documentation Specification

**Date:** 2026-09-09  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Init & Service Supervision / Documentation  
**Task ID:** T-01382  

---

## 1. Specification Overview
This document formally specifies the structural layout, content contracts, interface definitions, error envelopes, and automated verification requirements for the comprehensive Init & Service Supervision architectural documentation asset: `docs/service_supervision.md`, alongside its automated verification script `tools/test_service_doc.py`.

---

## 2. Document Structure & Section Contracts

The authoritative guide `docs/service_supervision.md` must strictly contain the following 9 canonical structural sections:

### Section 1: Executive Overview & Architectural Role
- **Context**: Role within AIOS Phase 1 (Linux Base System & Bootable Target).
- **Responsibilities**: Deterministic process lifecycle management, dependency-ordered init sequencing, security policy enforcement, and autonomous agent service supervision abstracting systemd and OpenRC semantics.
- **Mermaid Diagram**: Visual architecture diagram illustrating relationships between Operator CLI, Autonomous Agent MCP, Core Service Store, Policy Enforcement Point (PEP), and SQLite WAL Audit Ring.

### Section 2: Core Data Model & Invariants
- Types defined in `code/aiosh-rust/aiosh-core/src/service.rs`:
  - `ServiceSpec`, `ServiceStatus`, `ServiceHealth`, `ServiceType`, `ServiceRestartPolicy`, `ServiceStartupMode`, `ServiceState`, `ServiceDependencyType`, `ServiceDependency`, `ServiceAction`, `ServiceQuery`.
- Invariants `SS1..SS5`:
  - `SS1`: Service naming syntax matching systemd/OpenRC standard (`^[a-zA-Z0-9][a-zA-Z0-9_.-]*$`), length bounds [1..128].
  - `SS2`: Execution commands and path traversal protection (absolute paths, rejection of `..`).
  - `SS3`: Dependency graph hygiene (rejection of self-loops).
  - `SS4`: Timeout boundaries ($[1..86400]$s) and environment variable bounds ($\le 1024$).
  - `SS5`: Lifecycle state machine consistency.

### Section 3: Core Service Registry, FSM State Machine & Dependency Ordering
- Architecture in `code/aiosh-rust/aiosh-core/src/service_service.rs`:
  - `ServiceStore`: Thread-safe registry seeded with reference platform services (`aios-securityd.service`, `auditd.service`, `dbus.service`, `systemd-journald.service`, `ssh.service`).
  - Invariants `CS1..CS5`: Registry uniqueness, deterministic FSM lifecycle transitions, Kahn's algorithm topological sorting with cycle detection, multi-parameter querying, and atomic filesystem persistence with RAII `.tmp` cleanup.

### Section 4: Configuration Subsystem (`ServiceConfig`)
- Architecture in `code/aiosh-rust/aiosh-core/src/service_config.rs`:
  - Resolution precedence: Explicit file (`--config <path>`) > environment variables (`AIOS_SERVICE_*`) > defaults.
  - Invariants `SC1..SC7`: Store path validation, timeout bounds, store size ceiling ($[64\text{ KiB} \dots 100\text{ MiB}]$), entity bounds ($[10 \dots 100,000]$), auto-persistence toggle, restart backoff/burst, and 64 KiB config stream read cap.

### Section 5: Security Policy Subsystem (`ServiceSecurityPolicy`)
- Architecture in `code/aiosh-rust/aiosh-core/src/service_policy.rs`:
  - Invariants `SP1..SP6`: Configuration limits, prohibited daemon rejection (`telnet`, `rsh`, `rlogin`, `rexec`, `tftp`, `xinetd`, `ypserv`, `ypbind`), executable path hygiene (`/tmp`, `/var/tmp`, `/dev/shm`, `..`), user privilege & root restriction (`require_service_user`, `disallow_root`, `allowed_root_services`), environment sanitization (`LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS`), and tri-state policy modes (`Enforcing`, `Audit`, `Permissive`).

### Section 6: Observability Telemetry Subsystem (`ServiceObservabilityReport`)
- Architecture in `code/aiosh-rust/aiosh-core/src/service_observability.rs`:
  - Invariants `SO1..SO6`: Inventory completeness with mathematical conservation, multi-dimensional breakdowns, health accounting and restart saturation arithmetic, $O(1)$ memory dependency distribution histogram buckets (`"0"`, `"1-2"`, `"3-5"`, `"6+"`), policy compliance summary, and deterministic JSON emission.

### Section 7: Operator CLI Surface Reference (`aiosh service *`)
- Detailed command reference, options, exit codes, and examples for:
  - `aiosh service validate (--name <name> | --spec <file_or_json>) [--json]`
  - `aiosh service list [--state <state>] [--mode <mode>] [--pattern <pattern>] [--limit <n>] [--store <path>] [--json]`
  - `aiosh service show <name> [--store <path>] [--json]` (alias: `status`)
  - `aiosh service action <name> <action> [--store <path>] [--json]`
  - Direct action shortcuts: `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`
  - `aiosh service order <name> [--store <path>] [--json]`
  - `aiosh service config [--config <path>] [--json]`
  - `aiosh service policy [--service <name>] [--config <path>] [--store <path>] [--json]`
  - `aiosh service stats [--store <path>] [--policy <path>] [--json]` (alias: `observability`)

### Section 8: Autonomous Agent MCP Tool Surface Reference (`aios.service.*`)
- JSON-RPC 2.0 interface contracts, schemas, parameters, and return payloads for:
  - `aios.service.validate`
  - `aios.service.list`
  - `aios.service.get`
  - `aios.service.action`
  - `aios.service.order`
  - `aios.service.config`
  - `aios.service.policy`
  - `aios.service.stats`

### Section 9: Failure Modes, Error Envelopes, and Audit Trail
- Structured error codes: `INVALID_ARGUMENT`, `SERVICE_NOT_FOUND`, `LOAD_STORE_FAILED`, `LOAD_POLICY_FAILED`, `PERSIST_FAILED`, `ACTION_FAILED`, `ORDER_FAILED`, `CONFIG_RESOLUTION_FAILED`, `POLICY_RESOLUTION_FAILED`, `PAYLOAD_TOO_LARGE`.
- Non-repudiation audit trail: CLI invocations to `audit.db` via `classify_and_emit`; MCP tool calls to SQLite WAL ring buffer via `dispatch::recorded_call`.

---

## 3. Automated Documentation Unit Test Contract (`tools/test_service_doc.py`)

The automated verification suite must assert criteria `D1..D6`:
- **D1 (File Presence & Size)**: File `docs/service_supervision.md` exists and file size is within $[1,000 \dots 5,242,880]$ bytes.
- **D2 (Required Section Headings)**: Verbatim presence of all 9 required structural headers.
- **D3 (Zero Rot Markers)**: Document contains zero forbidden markers (`TODO`, `FIXME`, `TBD`, `XXX`, `PLACEHOLDER`).
- **D4 (Invariant & Command Coverage)**: Verbatim token presence for all invariants (`SS1..SS5`, `CS1..CS5`, `SC1..SC7`, `SP1..SP6`, `SO1..SO6`, `ST1..ST5`), all 12 CLI subcommands/shortcuts, and all 8 MCP tools.
- **D5 (Negative Test Cases)**: Asserts that the test runner fails if section headings or invariant tokens are missing.
- **D6 (Syntax & Code Block Hygiene)**: All code blocks specify valid language identifiers (bash, json, text, mermaid).
