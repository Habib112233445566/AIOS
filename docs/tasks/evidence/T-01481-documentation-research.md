# T-01481: User Session Bootstrap Documentation Research

**Date:** 2026-09-11  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Documentation  
**Task ID:** T-01481  

---

## 1. Executive Summary & Objective
Task `T-01481` establishes facts, architectural constraints, authoritative prior art, and concrete documentation requirements for the **User Session Bootstrap** subsystem. This research lays the foundation for creating a comprehensive, rot-proof architectural guide (`docs/user_session_bootstrap.md`) and its automated verification suite (`tools/test_session_doc.py`) spanning the complete epic:
- Data model & specification invariants (`SB1..SB5`)
- Core session service lifecycle FSM & seat arbitration (`CS1..CS5`)
- Operator CLI surface (`aiosh session *`)
- Autonomous Agent MCP surface (`aios.session.*`)
- Configuration resolution, capacity limits & precedence (`SC1..SC7`)
- Automated lifecycle & integration testing (`SBT1..SBT5`)
- Security policy & boundary invariants (`SSP1..SSP7`)
- Observability telemetry & distribution metrics (`SSO1..SSO6`)

---

## 2. Existing Codebase Audit & Assets

### 1. Data Model (`code/aiosh-rust/aiosh-core/src/session.rs`)
- **Core Entities**: `UserSessionSpec`, `UserSessionStatus`, `SessionState` (`Initializing`, `Authenticating`, `Active`, `Locked`, `Terminating`, `Terminated`), `SessionScope` (`Foreground`, `Background`), `SessionType` (`Tty`, `X11`, `Wayland`, `AiAgent`), `SessionClass` (`User`, `Greeter`, `LockScreen`, `Background`, `Agent`), `UserSessionAction` (`Authenticate`, `Activate`, `Lock`, `Unlock`, `Terminate`), `UserSessionQuery`, `UserSessionStore`.
- **Invariants SB1..SB5**:
  - `SB1`: Session identifier syntax conforming to systemd/loginctl standards (`^[a-zA-Z0-9][a-zA-Z0-9_.-]*$`), length [1..128].
  - `SB2`: Username syntax validation conforming to POSIX / Debian policy (`^[a-z_][a-z0-9_-]*[$]?$`), length [1..32].
  - `SB3`: Seat identifier syntax validation (`^[a-zA-Z0-9_-]+$`), length [1..64].
  - `SB4`: Environment block validation and sanitization, variable key syntax, size limits ($\le 1024$ chars per value, max 256 variables).
  - `SB5`: Specification coherence and cross-field invariant enforcement.

### 2. Core Service Registry & State Machine (`code/aiosh-rust/aiosh-core/src/session_service.rs`)
- **Core Engine**: `UserSessionService` providing thread-safe in-memory session store seeded with canonical greeter session on `seat0`.
- **Invariants CS1..CS5**:
  - `CS1`: Deterministic lifecycle state machine and two-stage teardown.
  - `CS2`: Seat arbitration and mutual exclusion (at most 1 foreground session per seat, automatic background demotion).
  - `CS3`: Concurrency quota enforcement (max 32 active sessions per user, max 1024 total tracked sessions in store).
  - `CS4`: Query filtering by user, state, session type, seat, and pagination limit bounds.
  - `CS5`: Atomic filesystem persistence with PID-isolated atomic rename.

### 3. Hierarchical Configuration (`code/aiosh-rust/aiosh-core/src/session_config.rs`)
- **Resolution Engine**: `SessionConfig` resolving across precedence: explicit file > environment variables (`AIOS_SESSION_*`) > defaults.
- **Invariants SC1..SC7**:
  - `SC1`: Store path bounds and control character rejection ($\le 1024$ chars).
  - `SC2`: User session quota bounds ($[1 \dots 256]$, default 32).
  - `SC3`: Total session capacity bounds ($[1 \dots 65,536]$, default 1024).
  - `SC4`: Default idle timeout bounds ($[10 \dots 86,400]$s, default 900s).
  - `SC5`: Auto-persist boolean flag.
  - `SC6`: Default seat identifier syntax and length limits.
  - `SC7`: Safe stream and file reading capped at 64 KiB.

### 4. Integration Test Suite (`code/aiosh-rust/aiosh-core/tests/test_session_automated.rs`)
- **Integration Matrix SBT1..SBT5**:
  - `SBT1`: Multi-turn lifecycle FSM cohesion & invalid transition rejection.
  - `SBT2`: Seat arbitration & automatic foreground demotion.
  - `SBT3`: User quotas & store capacity enforcement.
  - `SBT4`: Multi-turn disk persistence & recovery.
  - `SBT5`: Multi-parameter query introspection.

### 5. Security Policy Engine (`code/aiosh-rust/aiosh-core/src/session_policy.rs`)
- **Security Subsystem**: `UserSessionSecurityPolicy` supporting `Enforcing`, `Permissive`, and `Disabled` modes.
- **Invariants SSP1..SSP7**:
  - `SSP1`: Root user session restriction (`disallow_root`).
  - `SSP2`: Greeter session boundaries (unprivileged user cannot claim greeter class).
  - `SSP3`: Seat0 physical protection (remote network sessions prohibited on physical seat0).
  - `SSP4`: Environment hygiene & dynamic linker injection prevention (rejection of `LD_PRELOAD`, `LD_LIBRARY_PATH`).
  - `SSP5`: User concurrency quota compliance.
  - `SSP6`: Autonomous AI agent sandboxing and permission isolation.
  - `SSP7`: File size limits and path safety ($\le 64\text{ KiB}$ policy files, no directory traversal).

### 6. Observability Telemetry (`code/aiosh-rust/aiosh-core/src/session_observability.rs`)
- **Observability Engine**: `SessionObservabilityReport` generating comprehensive telemetry snapshots.
- **Invariants SSO1..SSO6**:
  - `SSO1`: State and class breakdown distribution with zero-initialized bins.
  - `SSO2`: Seat arbitration distribution and foreground/background focus tracking.
  - `SSO3`: Idle time tracking, peak detection, and aggregate duration.
  - `SSO4`: User concurrency distribution & distinct users tally.
  - `SSO5`: Security policy compliance evaluation.
  - `SSO6`: Deterministic canonical JSON serialization.

### 7. Operational Surfaces
- **Operator CLI (`aiosh-cli`)**: 13 subcommands under `aiosh session`: `validate`, `list`, `show`, `status`, `action`, `activate`, `lock`, `unlock`, `terminate`, `auth`, `create`, `config`, `policy`, `stats`.
- **Autonomous Agent MCP (`aiosh-mcp`)**: 8 MCP tools under `aios.session.*`: `validate`, `list`, `get`, `action`, `create`, `config`, `policy`, `stats`, gated with PEP capability tokens and SQLite WAL audit logging.

### 8. Test Runners
- `tools/test_session_suites.py`: Validates criteria `SB1..SB8`.
- `code/aiosh-cli/tests/test_session_cli_smoke.py`: Standalone CLI smoke test suite.
- `code/aiosh-mcp/tests/test_session_mcp_smoke.py`: Standalone MCP stdio smoke test suite.

---

## 3. Authoritative Prior Art & Standards

1. **systemd-logind Architecture (`systemd-logind(8)`, `loginctl(1)`, `sd-login(3)`)**:
   - Seat assignment model (`seat0` primary seat), multi-seat support, virtual terminals (VTs), session states, focus arbitration, and idle time monitoring.
2. **freedesktop.org Session Management Specification**:
   - Desktop and session lifecycle protocols, environment variable conventions (`XDG_RUNTIME_DIR`, `XDG_SESSION_ID`, `XDG_SESSION_TYPE`, `XDG_SEAT`), and inhibitor locks.
3. **POSIX 1003.1 Session & Process Group Management**:
   - Session leaders, controlling terminals, job control, signals, and login accounting (`utmp`, `wtmp`, `lastlog`).
4. **NIST SP 800-53 (Rev 5)**:
   - AC-2 (Account Management), AC-10 (Concurrent Session Control), AC-11 (Session Lock), and AC-12 (Session Termination).
5. **AIOS Standard Result Envelope & Audit Ring (ADR-0035 / ADR-0036)**:
   - Unified `{ "code": 0, "data": ..., "error": ... }` response structure, non-repudiable SHA-256 hash-chained audit logging to SQLite WAL, and capability-gated dispatch.

---

## 4. Facts vs. Assumptions

### Facts (Empirically Verified in Codebase)
- All session IDs, usernames, and seat names reject ASCII control characters and null bytes.
- Seat mutual exclusion guarantees at most 1 foreground session on any seat.
- Terminated sessions are immutable; state machine transitions are strictly deterministic.
- Every state mutation and query emits an immutable audit record to `audit.db` / `audit.log`.
- All test criteria `SB1..SB8` pass via `tools/test_session_suites.py`.

### Assumptions (To Be Codified in Documentation)
- Operators and AI agents require a unified, single-page authoritative reference document detailing all sub-epics.
- An automated validation tool (`tools/test_session_doc.py`) should continuously ensure documentation remains synchronized with code invariants and never rots.

---

## 5. Architectural Decisions Needed for Documentation Asset
1. **Target Document Path**: Create `docs/user_session_bootstrap.md` mirroring the structure of `docs/service_supervision.md` and `docs/package_management.md`.
2. **Automated Verification**: Implement `tools/test_session_doc.py` verifying structural headers, verbatim invariant tokens (`SB1..SB5`, `CS1..CS5`, `SC1..SC7`, `SBT1..SBT5`, `SSP1..SSP7`, `SSO1..SSO6`), zero rot markers (`TODO`, `FIXME`, `TBD`, `XXX`), and working examples.
3. **Repository Index Synchronization**: Register `docs/user_session_bootstrap.md` in `docs/README.md`.
4. **Master Matrix Integration**: Add criterion `SB9` to `tools/test_session_suites.py` to run the documentation verification test.
