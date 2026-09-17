# T-01491: User Session Bootstrap Recovery & Validation Research

**Date:** 2026-09-16  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Recovery & Validation  
**Task ID:** T-01491  

---

## 1. Executive Summary & Objective

Task `T-01491` establishes facts, architectural constraints, authoritative prior art, and implementation requirements for the **Recovery & Validation** epic of the User Session Bootstrap subsystem.

The objective of this epic is to design, implement, test, and integrate:
1. **Deep Session Store Validation**: An automated, deterministic verification engine that audits all registered session specifications (`UserSessionSpec`) and statuses (`UserSessionStatus`) in a session store against syntactic, semantic, hardware seat, and lifecycle FSM rules.
2. **Non-Destructive Self-Healing & Quarantine**: Resilience against store corruption, truncated writes, unparseable JSON, tampered fields, or process crashes by atomically archiving damaged stores into timestamped quarantine backups (`.bak.<timestamp>`) with strict `0600` permissions, followed by clean state recovery.
3. **Session State Reconciliation**: Reconciling runtime inconsistencies (e.g. duplicate session leader PIDs, orphaned foreground seats, illegal state transitions) upon startup or recovery.
4. **Substrate Parity & Tool Surfaces**: Exposing deep validation and recovery across the operator CLI (`aiosh session recover`, `aiosh session check`) and autonomous agent MCP surface (`aios.session.recover`, `aios.session.check`).

---

## 2. Existing Codebase Audit & Sister Subsystems

### 2.1 Current User Session Bootstrap Subsystem
- **Data Model (`code/aiosh-rust/aiosh-core/src/session.rs`)**:
  - `UserSessionStore` contains `sessions: BTreeMap<String, UserSessionSpec>` and `statuses: BTreeMap<String, UserSessionStatus>`.
  - `load_from_path(path: &Path)` reads JSON from disk. If the file is missing or corrupted, it currently returns a bare `std::io::Error`.
  - `from_json()` validates spec/status invariants and ensures non-terminated sessions have unique leader PIDs.
  - Does not currently provide quarantine, automated backup creation, partial salvage, or deep health reporting.
- **Service Coordinator (`code/aiosh-rust/aiosh-core/src/session_service.rs`)**:
  - `UserSessionService` wraps `UserSessionStore` and `UserSessionSecurityPolicy`.
  - `load_from_path()` delegates directly to `UserSessionStore::load_from_path`. It fails completely if the JSON is malformed or unreadable.
- **Missing Asset**:
  - No `session_recovery.rs` module exists in `aiosh-core`.

### 2.2 Sister Recovery Subsystems in AIOS
AIOS has established a consistent, robust architectural pattern for recovery across other Phase 1 subsystems:
1. **`service_recovery.rs` (Init & Service Supervision)**:
   - `validate_service_store(&store, path) -> ServiceValidationReport`
   - Invariants `SR1..SR5`:
     - Mathematical consistency: $Valid + Invalid = Total$.
     - Health definition: $Healthy \iff (Errors.is\_empty() \land Invalid == 0)$.
     - Error cardinality: $Invalid > 0 \implies Errors.len() \ge Invalid$.
   - `recover_service_store_with_backup(path) -> (ServiceStore, PathBuf)`:
     - Copies corrupted store to `<path>.bak.<timestamp>`.
     - Initializes fresh default store and saves it.
   - `load_or_recover(path) -> Result<(ServiceStore, ServiceValidationReport, bool, Option<PathBuf>), String>`:
     - Handles missing files, valid files, and corrupted files cleanly.
2. **`package_recovery.rs` (Package Management)**:
   - Implements `PackageValidationReport`, `recover_package_store_with_backup`, `load_or_recover`.
   - Checks package format, version bounds, checksum digests, and dependency acyclicity.
3. **`base_image_recovery.rs` (Base Image Generation)**:
   - Implements `ImageStore` validation, repair, and quarantine with `load_or_recover`.
4. **`handoff_service.rs` (Agent Task Handoff)**:
   - `HandoffStore::load_or_recover` non-destructively recovers corrupted task handoff state.

---

## 3. Authoritative Sources & Upstream Prior Art

| Source | Relevance to Session Recovery | Key Architectural Principles |
|---|---|---|
| **`systemd-logind(8)`** | Linux Session Lifecycle Manager | Maintains state files under `/run/systemd/sessions/<id>`. On daemon restart or crash, scans runtime session files, probes `/proc/<pid>` to verify session leader liveness, and reaps dead sessions. |
| **`loginctl(1)`** | Session Control CLI | Commands like `terminate-session`, `lock-session`, and `session-status` allow operators to reconcile stuck or broken sessions without rebooting. |
| **`freedesktop.org` Multi-Seat Specification** | Hardware Seat Management | Defines physical seat `seat0` as the master seat containing physical display and console VTs. Mandates mutual exclusion of foreground sessions per seat. |
| **POSIX.1-2017 (§11, General Terminal Interface)** | Process Groups & Sessions | A process group has a single process group leader (`pid == pgid`). A session has a single session leader (`pid == sid`). Two active sessions cannot share the same leader PID. |
| **RFC 1123 / IETF Standards** | Hostname & Remote Identification | Validation rules for remote network origins. |
| **ADR-0035 §D-2 / §F-2** | AIOS Tooling & Audit Invariants | Fail-safe auditing: recovery actions, quarantine events, and repairs must generate explicit audit rows. |

---

## 4. Facts vs. Assumptions

### Verified Facts
1. **Fact**: `UserSessionStore` persists state as JSON using atomic sibling writes (`0600` permissions) and atomic renaming.
2. **Fact**: On sudden crash, power disruption, or manual tampering, a JSON store can be left truncated, unparseable, or containing contradictory state (e.g. two foreground sessions on `seat0`).
3. **Fact**: Current `UserSessionService::load_from_path` fails hard with an error on unparseable JSON, leaving CLI commands and MCP tools unable to start or manage sessions until manual intervention.
4. **Fact**: All other core subsystems in `aiosh-core` implement a `_recovery.rs` module providing `load_or_recover` and deep validation reports.

### Working Assumptions
1. **Assumption**: When a store is damaged, the safest automated default is non-destructive quarantine: copy the corrupt file to `<path>.bak.<timestamp>`, log the event, and seed a clean store containing the default `greeter-seat0` session.
2. **Assumption**: A dedicated validation report structure (`SessionValidationReport`) should evaluate specs (`SB1..SB5`), statuses, FSM state legality, seat exclusivity, and capacity quotas.
3. **Assumption**: The operator should be able to invoke `aiosh session check` (read-only validation) and `aiosh session recover` (active repair/quarantine), with matching MCP tools `aios.session.check` and `aios.session.recover`.

---

## 5. Proposed Recovery & Validation Invariants (`SSR1..SSR7`)

1. **`SSR1` (Mathematical Completeness)**:  
   $TotalSessions = ValidSessions + InvalidSessions$.
2. **`SSR2` (Strict Health Truth Condition)**:  
   $Healthy \iff (Errors.is\_empty() \land InvalidSessions == 0)$.
3. **`SSR3` (Error Cardinality Lower Bound)**:  
   $InvalidSessions > 0 \implies Errors.len() \ge InvalidSessions$.
4. **`SSR4` (Non-Destructive Quarantine)**:  
   Whenever a corrupted or invalid store is recovered, the original file is preserved at `<path>.bak.<timestamp>` with POSIX permissions `0600` before any replacement file is written.
5. **`SSR5` (Leader PID Uniqueness & Consistency)**:  
   No two non-terminated sessions may share the same `leader_pid`.
6. **`SSR6` (Seat Mutual Exclusion Integrity)**:  
   At most one active session per seat may hold `SessionScope::Foreground`. If multiple are detected, recovery demotes all but the most recently active session to `SessionScope::Background`.
7. **`SSR7` (Full Invariant Validation)**:  
   Every session specification in the store must pass `validate_user_session_spec`, and every status must pass `validate_user_session_status`.

---

## 6. Decisions & Open Questions

### Decided
1. **Module Architecture**: Create `code/aiosh-rust/aiosh-core/src/session_recovery.rs` and export it in `code/aiosh-rust/aiosh-core/src/lib.rs`.
2. **Core Types**:
   - `SessionRecoveryAction`: `LoadedExisting`, `CreatedDefaultFresh`, `RecoveredFromBackup { backup_path, reason }`.
   - `SessionValidationReport`: `store_path`, `total_sessions`, `valid_sessions`, `invalid_sessions`, `errors`, `warnings`, `healthy`, `evaluated_at`.
3. **Core Functions**:
   - `validate_session_store(store: &UserSessionStore, path: &Path) -> SessionValidationReport`
   - `recover_session_store_with_backup(path: &Path) -> (UserSessionStore, PathBuf)`
   - `load_or_recover(path: &Path) -> Result<(UserSessionService, SessionValidationReport, bool, Option<PathBuf>), String>`

### Questions for Specification Phase (`T-01492`)
1. Should `aiosh session recover` support a dry-run flag (`--dry-run`) to inspect the quarantine and repair actions before modifying the disk? *(Proposed: Yes)*
2. Should `aios.session.recover` be PEP-gated? *(Proposed: Yes, since it can quarantine and re-seed the session store)*
3. Should partial salvage be attempted (extracting valid JSON session blocks from a semi-corrupted file) or is whole-file quarantine with default seed sufficient? *(Proposed: Whole-file quarantine with default seed conforms to sister recovery modules and avoids executing partially corrupted agent sessions)*

---

## 7. Citations & References
1. `systemd-logind.service(8)` — Systemd session management daemon documentation.
2. `loginctl(1)` — Control the systemd login manager.
3. POSIX.1-2017 Standard — IEEE Std 1003.1-2017, Process Groups and Sessions.
4. `code/aiosh-rust/aiosh-core/src/service_recovery.rs` — Sister subsystem recovery implementation.
5. `docs/user_session_bootstrap.md` — AIOS User Session Bootstrap Architecture & Operational Guide.
