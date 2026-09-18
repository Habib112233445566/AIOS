# T-01482: User Session Bootstrap Documentation Specification

**Date:** 2026-09-11  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Documentation  
**Task ID:** T-01482  

---

## 1. Specification Overview
This document formally specifies the structural layout, content contracts, interface definitions, error envelopes, and automated verification requirements for the comprehensive User Session Bootstrap architectural documentation asset: `docs/user_session_bootstrap.md`, alongside its automated verification script `tools/test_session_doc.py`.

---

## 2. Document Structure & Section Contracts

The authoritative guide `docs/user_session_bootstrap.md` must strictly contain the following 9 canonical structural sections:

### Section 1: Executive Overview & Architectural Role
- **Context**: Role within AIOS Phase 1 (Linux Base System & Bootable Target).
- **Responsibilities**: Session bootstrap lifecycle management, multi-seat hardware arbitration, physical display and VT assignment, security boundary enforcement, and autonomous AI agent session supervision conforming to `loginctl(1)` and freedesktop standards.
- **Mermaid Diagram**: Visual architecture diagram illustrating relationships between Operator CLI, Autonomous Agent MCP, Core Session Store, Policy Enforcement Point (PEP), Physical Seats, and SQLite WAL Audit Ring.

### Section 2: Core Data Model & Specification Invariants
- Types defined in `code/aiosh-rust/aiosh-core/src/session.rs`:
  - `UserSessionSpec`, `UserSessionStatus`, `SessionState`, `SessionScope`, `SessionType`, `SessionClass`, `UserSessionAction`, `UserSessionQuery`, `UserSessionStore`.
- Invariants `SB1..SB5`:
  - `SB1`: Session identifier syntax conforming to systemd/loginctl standards (`^[a-zA-Z0-9][a-zA-Z0-9_.-]*$`), length [1..128].
  - `SB2`: POSIX username syntax validation (`^[a-z_][a-z0-9_-]*[$]?$`), length [1..32].
  - `SB3`: Seat identifier syntax validation (`^[a-zA-Z0-9_-]+$`), length [1..64].
  - `SB4`: Environment block validation and sanitization, variable key syntax, size limits ($\le 1024$ chars per value, max 256 variables).
  - `SB5`: Specification coherence and cross-field invariant enforcement.

### Section 3: Core Service Registry, Lifecycle FSM & Seat Arbitration
- Architecture in `code/aiosh-rust/aiosh-core/src/session_service.rs`:
  - `UserSessionService`: Thread-safe session registry seeded with canonical greeter session on `seat0`.
  - Invariants `CS1..CS5`:
    - `CS1`: Deterministic FSM lifecycle state machine (`authenticate`, `activate`, `lock`, `unlock`, `terminate`) with two-stage teardown (`terminating` -> `terminated`).
    - `CS2`: Seat arbitration and mutual exclusion (at most 1 foreground session per seat, automatic background demotion).
    - `CS3`: Concurrency quota enforcement (max 32 active sessions per user, max 1024 total tracked sessions in store).
    - `CS4`: Query filtering by username, lifecycle state, session type, physical seat, and pagination limit bounds.
    - `CS5`: Atomic filesystem persistence with PID-isolated atomic rename.

### Section 4: Configuration Subsystem (`SessionConfig`)
- Architecture in `code/aiosh-rust/aiosh-core/src/session_config.rs`:
  - Resolution precedence: Explicit file (`--config <path>`) > environment variables (`AIOS_SESSION_*`) > defaults.
  - Invariants `SC1..SC7`: Store path validation, user quota bounds ($[1 \dots 256]$, default 32), total capacity bounds ($[1 \dots 65,536]$, default 1024), default idle timeout bounds ($[10 \dots 86,400]$s, default 900s), auto-persistence toggle, default seat identifier, and 64 KiB config stream read cap.

### Section 5: Security Policy Subsystem (`UserSessionSecurityPolicy`)
- Architecture in `code/aiosh-rust/aiosh-core/src/session_policy.rs`:
  - Invariants `SSP1..SSP7`:
    - `SSP1`: Root user session restriction (`disallow_root`).
    - `SSP2`: Greeter session boundaries (unprivileged user cannot claim greeter class).
    - `SSP3`: Seat0 physical protection (remote network sessions prohibited on physical seat0).
    - `SSP4`: Environment hygiene & dynamic linker injection prevention (rejection of `LD_PRELOAD`, `LD_LIBRARY_PATH`).
    - `SSP5`: User concurrency quota compliance.
    - `SSP6`: Autonomous AI agent sandboxing and permission isolation.
    - `SSP7`: File size limits and path safety ($\le 64\text{ KiB}$ policy files, no directory traversal).

### Section 6: Observability Telemetry Subsystem (`SessionObservabilityReport`)
- Architecture in `code/aiosh-rust/aiosh-core/src/session_observability.rs`:
  - Invariants `SSO1..SSO6`:
    - `SSO1`: State and functional class breakdown distributions with complete zero-initialized bins.
    - `SSO2`: Seat arbitration distribution and foreground/background focus tracking.
    - `SSO3`: Idle duration metrics, peak idle detection, aggregate idle seconds, and locked session count.
    - `SSO4`: User concurrency distribution & distinct users tally.
    - `SSO5`: Security policy compliance evaluation and violating session tracking.
    - `SSO6`: Deterministic canonical JSON serialization.

### Section 7: Operator CLI Surface Reference (`aiosh session *`)
- Detailed command reference, options, exit codes, and copy-pasteable examples for:
  - `aiosh session validate (--id <id> | --user <username> | --spec <file_or_json>) [--json]`
  - `aiosh session list [--user <username>] [--state <state>] [--type <type>] [--seat <seat>] [--limit <n>] [--store <path>] [--json]`
  - `aiosh session show <session_id> [--store <path>] [--json]` (alias: `status`)
  - `aiosh session action <session_id> <action> [--store <path>] [--json]`
  - Direct action shortcuts: `activate`, `lock`, `unlock`, `terminate`, `auth`
  - `aiosh session create <spec_file_or_json> [--store <path>] [--json]`
  - `aiosh session config [--config <path>] [--json]`
  - `aiosh session policy [--policy <path>] [--spec <file_or_json>] [--store <path>] [--json]`
  - `aiosh session stats [--policy <path>] [--store <path>] [--json]`

### Section 8: Autonomous Agent MCP Tool Surface Reference (`aios.session.*`)
- JSON-RPC 2.0 interface contracts, schemas, parameters, and return payloads for:
  - `aios.session.validate`
  - `aios.session.list`
  - `aios.session.get`
  - `aios.session.action`
  - `aios.session.create`
  - `aios.session.config`
  - `aios.session.policy`
  - `aios.session.stats`

### Section 9: Failure Modes, Error Envelopes, and Audit Trail
- Structured error codes: `INVALID_ARGUMENT`, `SESSION_NOT_FOUND`, `LOAD_STORE_FAILED`, `POLICY_RESOLUTION_FAILED`, `CONFIG_RESOLUTION_FAILED`, `PAYLOAD_TOO_LARGE`, `ACTION_FAILED`, `VALIDATION_FAILED`.
- Non-repudiation audit trail: CLI invocations to `audit.db` via `classify_and_emit`; MCP tool calls to SQLite WAL ring buffer via `dispatch::recorded_call`.

---

## 3. Automated Documentation Unit Test Contract (`tools/test_session_doc.py`)

The automated verification suite must assert criteria `D1..D6`:
- **D1 (File Presence & Size)**: File `docs/user_session_bootstrap.md` exists and file size is within $[1,000 \dots 5,242,880]$ bytes.
- **D2 (Required Section Headings)**: Verbatim presence of all 9 required structural headers.
- **D3 (Zero Rot Markers)**: Document contains zero forbidden markers (`TODO`, `FIXME`, `TBD`, `XXX`, `PLACEHOLDER`).
- **D4 (Invariant & Command Coverage)**: Verbatim token presence for all invariants (`SB1..SB5`, `CS1..CS5`, `SC1..SC7`, `SBT1..SBT5`, `SSP1..SSP7`, `SSO1..SSO6`), all 13 CLI subcommands/shortcuts, and all 8 MCP tools.
- **D5 (Negative Test Cases)**: Asserts that the test runner fails if section headings or invariant tokens are missing.
- **D6 (Syntax & Code Block Hygiene)**: All code blocks specify valid language identifiers (bash, json, text, mermaid).
