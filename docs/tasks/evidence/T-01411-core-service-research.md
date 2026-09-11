# T-01411: User Session Bootstrap - Core Service: Research

## Metadata
- **Task ID:** `T-01411`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Core Service (`code/aiosh-rust/aiosh-core::session_service`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic Launch: User Session Bootstrap (1/10) — Core Service Research

---

## 1. Executive Overview & Mission Context

Having established and verified the **User Session Bootstrap Data Model** (`T-01401..T-01410`), the objective of the **Core Service** sub-epic (`T-01411..T-01420`) is to research, specify, scaffold, implement, test, integrate, harden, and document the runtime session management service (`UserSessionService`) for AIOS.

In a modern operating system—and specifically for AIOS on Kali Linux—the session service is the core coordinator bridging:
1. **Pillar A (Ethical Hacking on the Inside)**: Dedicated penetration testing user accounts (`kali`, root-delegated tools) running inside secure, isolated session boundaries.
2. **Pillar B (Windows-Look Desktop on the Outside)**: Seat-bound graphical sessions (`X11`/`Wayland`) running display managers (LightDM/Greetd) and desktop shells (XFCE with Kali Undercover) with seat arbitration, lock screens, and VT allocation.
3. **Pillar C (S-Rank AI Subsystem)**: First-class autonomous AI agent execution contexts (`SessionType::AiAgent`, `SessionClass::Agent`) that operate either headlessly or as desktop co-pilots with Policy Enforcement Point (PEP) gating and SQLite WAL non-repudiation auditing.

The core service must provide deterministic, in-memory state tracking, lifecycle transition execution (`create`, `authenticate`, `activate`, `lock`, `unlock`, `terminate`), multi-attribute querying, seat conflict arbitration, and atomic persistence to `/run/aios/sessions.json`.

---

## 2. Existing Codebase Analysis

Prior to specifying new abstractions, the existing codebase was thoroughly audited:

1. **Data Model (`code/aiosh-rust/aiosh-core/src/session.rs`)**:
   - Shipped in `T-01401..T-01410`.
   - Defines core types: `SessionType`, `SessionClass`, `SessionState`, `SessionScope`, `UserSessionSpec`, `UserSessionStatus`, `UserSessionAction`, `UserSessionQuery`, `UserSessionStore`.
   - Enforces data model invariants `SB1..SB5`:
     - `SB1`: Session ID syntax and boundary limits ($[1 \dots 64]$ chars, no `..` traversal).
     - `SB2`: Username syntax ($[1 \dots 32]$ POSIX format) and UID/GID boundaries.
     - `SB3`: Lifecycle state machine transition validation matrix.
     - `SB4`: Environment variable bounds ($\le 256$ entries, $\le 4,096$ chars per value, `XDG_RUNTIME_DIR` syntax).
     - `SB5`: Capacity limits (max 32 active sessions per user, max 1,024 total sessions, 10 MiB store limit).
   - Contains basic atomic save/load routines for `UserSessionStore`.

2. **Parallel Subsystem Reference (`code/aiosh-rust/aiosh-core/src/service_service.rs`)**:
   - Implements `ServiceStore` and `ServiceActionReport` for Init & Service Supervision.
   - Provides a clear architectural template:
     - Clear separation between data model (`service.rs`) and runtime lifecycle management engine (`service_service.rs`).
     - Standard action execution reporting (`ServiceActionReport` containing `service_name`, `action`, `previous_state`, `new_state`, `success`, `error`, `timestamp`).
     - Pre-seeded canonical reference services via `ServiceStore::new()` alongside an unseeded `ServiceStore::empty()`.
     - Deterministic mutation methods returning typed reports.

3. **Current Gaps to Address in `session_service.rs`**:
   - Lack of a dedicated `UserSessionService` coordinator managing runtime mutations and seat arbitration.
   - Lack of a formal `UserSessionActionReport` envelope detailing the exact outcome of session administrative actions.
   - Lack of pre-seeded canonical sessions (such as a default `seat0` login greeter session).
   - Lack of dynamic seat arbitration (ensuring exactly one session has `SessionScope::Foreground` per seat).
   - Lack of idle time tracking and session locking automation logic.

---

## 3. Authoritative Sources & Upstream Standards

1. **`systemd-logind.service(8)` & `org.freedesktop.login1(5)` Session Architecture**:
   - *Source*: freedesktop.org systemd documentation and D-Bus interfaces.
   - Upstream Manager methods: `CreateSession()`, `ReleaseSession()`, `ActivateSession()`, `LockSession()`, `UnlockSession()`, `TerminateSession()`, `KillSession()`.
   - **Seat Arbitration**: A seat (e.g. `seat0`) represents a collection of hardware (display, keyboard, mouse). At any given moment, exactly one session on that seat can be active in the foreground (`active = true` / `SessionScope::Foreground`). Activating a session automatically yields the seat from any prior active session.
   - **Session States**: `online` (processes executing), `active` (holding the seat foreground), `closing` (in termination phase).
   - **Locking**: Emits `Lock` signal to session processes; screensaver or lock screen captures display and input until authentication unlocks the session.

2. **Linux-PAM (Pluggable Authentication Modules) Session Mechanics**:
   - *Source*: The Linux-PAM Application Developers' Guide (RFC 86.0).
   - Session establishment flow: `pam_start()` $\to$ `pam_authenticate()` $\to$ `pam_acct_mgmt()` $\to$ `pam_setcred()` $\to$ `pam_open_session()`.
   - Session teardown flow: `pam_close_session()` $\to$ `pam_setcred(PAM_DELETE_CRED)` $\to$ `pam_end()`.
   - `pam_open_session` mounts runtime directories, sets resource limits (`pam_limits.so`), and injects session variables.

3. **freedesktop.org XDG Base Directory Specification & Seat Environment**:
   - *Source*: XDG Base Directory Specification, Version 0.8.
   - Every active user session requires:
     - `XDG_RUNTIME_DIR`: `/run/user/<UID>` owned by the user (`0700`).
     - `XDG_SESSION_ID`: Unique session identifier.
     - `XDG_SESSION_TYPE`: `tty`, `x11`, `wayland`, or AIOS-specific `ai_agent`.
     - `XDG_SESSION_CLASS`: `user`, `greeter`, `lock-screen`, `background`, or `agent`.
     - `XDG_SEAT`: Hardware seat assignment (e.g. `seat0`).
     - `XDG_VTNR`: Virtual terminal number (e.g. 1..12).

4. **POSIX IEEE Std 1003.1-2017 (Process Groups, Sessions & Job Control)**:
   - *Source*: IEEE Std 1003.1-2017, §3.344, `setsid(2)`, `kill(2)`.
   - The session leader process PID is assigned at session bootstrap.
   - Termination protocol: Signals session leader with `SIGTERM`; if processes linger beyond grace period, issues `SIGKILL` to the entire process group.

5. **AIOS Security Architecture (ADR-0035 / ADR-0036)**:
   - Consequential session lifecycle operations (session allocation, authentication, activation, lock, unlock, termination) must be PEP-gated and write exactly one structured audit row into the SQLite WAL audit ring.
   - Read-only inspection queries (`get`, `list`, `validate`) execute without generating audit overhead.

---

## 4. Facts vs. Assumptions

| Fact | Assumption |
|---|---|
| In Linux, hardware seats (like `seat0`) can only grant interactive display and input focus to one session at a time. | AIOS can implement dynamic seat arbitration entirely in memory and state files, automatically demoting previous foreground sessions to `SessionScope::Background`. |
| Session processes belong to a process tree anchored by a session leader PID. | In testing and containerized environments where root access or real child processes are absent, leader PIDs can be mock PIDs or simulated without failing validation. |
| Terminating a session requires releasing its seat, closing credentials, and setting state to `Terminated`. | Terminated sessions should remain recorded in the store for audit and telemetry queries until explicitly pruned, but must not count toward active user session limits. |
| Autonomous AI agent sessions (`SessionType::AiAgent`) do not require virtual terminals (`vtnr`) or X11 displays. | AI agent sessions can share the unified `UserSessionService` API, allowing unified management, inspection, and security policy gating alongside human user sessions. |
| Production systems rely on PAM for credential verification. | The core service should provide a clean authentication action (`Authenticate`) that validates state transitions and prepares credentials, with pluggable support for mock verification in tests and PAM in production. |

---

## 5. Proposed Core Service Architecture & Invariants (CS1..CS5)

The core service will be implemented in `code/aiosh-rust/aiosh-core/src/session_service.rs` around two key abstractions:
1. `UserSessionActionReport`: Standardized result envelope for lifecycle actions.
2. `UserSessionService`: The central session lifecycle coordinator and store manager.

### 5.1 Invariants Matrix (CS1..CS5)

1. **`CS1` — Action Atomicity & State Validity**:
   - Every administrative action (`Create`, `Authenticate`, `Activate`, `Lock`, `Unlock`, `Terminate`) must either succeed completely and update store state or fail with an error string leaving the previous state unchanged.
   - State transitions must strictly follow the transition matrix defined in `SB3`.

2. **`CS2` — Seat Mutual Exclusion & Focus Arbitration**:
   - For any seat (e.g. `seat0`), at most one session may have `SessionScope::Foreground` at any given time.
   - Activating a session on seat $S$ automatically sets its scope to `Foreground` and demotes any existing foreground session on seat $S$ to `Background`.

3. **`CS3` — Capacity Enforcement & Allocation Limits**:
   - Session creation strictly enforces `SB5`: reject creation if user has $\ge 32$ active sessions or if total sessions in store reach $\ge 1,024$.
   - Terminated sessions do not count against the 32-session active limit per user.

4. **`CS4` — Non-Repudiation & Action Reporting**:
   - Every lifecycle transition returns a `UserSessionActionReport` capturing `session_id`, `action`, `previous_state`, `new_state`, `success`, `error`, and RFC-3339 timestamp.
   - Allows straightforward recording to the SQLite WAL audit trail.

5. **`CS5` — Idle Tracking & Lock State Consistency**:
   - `locked` boolean must be strictly consistent with `state`:
     - `state == SessionState::Locked` $\implies$ `locked == true`.
     - `state == SessionState::Active` $\implies$ `locked == false`.
   - Monotonic idle tracking: `idle_seconds` increments during inactivity; resets upon user activity or unlocking.

---

## 6. Unknowns & Decisions Needed

The following architectural decisions are established before proceeding to specification and scaffolding:

1. **Decision 1: Store Persistence Path**:
   - *Context*: Where does `UserSessionService` persist active session state by default?
   - *Decision*: Canonical default is `/run/aios/sessions.json` (on volatile `tmpfs`), with configurable store paths for tests and persistent environments.
   - *Rationale*: Aligns with `systemd-logind` storing runtime state under `/run/systemd/sessions/` and prevents session state pollution across reboots.

2. **Decision 2: Canonical Pre-seeded Sessions**:
   - *Context*: Should `UserSessionService::new()` include any default session?
   - *Decision*: `UserSessionService::new()` will pre-seed a canonical display greeter session on `seat0` (`greeter-seat0`, user `lightdm` or `greeter`, state `Active`, scope `Foreground`). `UserSessionService::empty()` provides a blank slate for isolated unit testing.
   - *Rationale*: A freshly booted workstation always presents a greeter on `seat0` awaiting user login.

3. **Decision 3: Seat Switching Behavior**:
   - *Context*: When activating session $A$ on `seat0`, what happens to session $B$ which was previously `Foreground` on `seat0`?
   - *Decision*: Session $B$ has its `scope` updated to `SessionScope::Background`. Its `state` remains `Active` (or `Locked`). It does NOT get terminated.
   - *Rationale*: Matches Linux console VT switching where background sessions continue running in the background.

4. **Decision 4: Terminated Session Retention**:
   - *Context*: Should terminated sessions be removed from memory or retained?
   - *Decision*: Terminated sessions are retained in state `Terminated` until explicit pruning or store restart, ensuring post-mortem inspection and auditability.
   - *Rationale*: Necessary for forensic audit logging and historical session queries.

5. **Decision 5: External Dependencies**:
   - *Context*: Does the core service require any new Cargo crates?
   - *Decision*: No new dependencies. Reuses existing `serde`, `serde_json`, and `std` libraries already in `aiosh-core`.

---

## 7. Next Steps

With research complete, facts established, and decisions codified:
1. Advance task pointer to **`T-01412`** (`User Session Bootstrap / core service: Specification`).
2. Author `docs/tasks/evidence/T-01412-core-service-specification.md` defining exact Rust structs, function signatures, error contracts, and persistence behavior.
