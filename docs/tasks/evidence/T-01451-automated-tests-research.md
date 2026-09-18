# T-01451: User Session Bootstrap — Automated Tests: Research

## Metadata
- **Task ID:** `T-01451`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** User Session Bootstrap Automated Testing Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Executive Summary

This research establishes the test criteria, multi-turn lifecycle validation patterns, and regression testing strategy for the AIOS User Session Bootstrap subsystem (`aiosh-core::session`, `aiosh-core::session_service`, and `aiosh-core::session_config`).

In preceding tasks (`T-01401..T-01450`), we developed:
1. Session Data Model (`SB1..SB5`).
2. Core User Session Service and state transition dispatcher (`CS1..CS5`).
3. Operator CLI surface (`aiosh session`).
4. Autonomous Agent MCP API surface (`aios.session.*`).
5. Configuration Subsystem (`SessionConfig`, `SC1..SC7`).

While unit test suites exist for isolated units (`test_session_data_model.rs`, `test_session_service.rs`, `test_session_config.rs`), a dedicated end-to-end integration and automated test harness (`test_session_automated.rs`) is required to validate cross-component interactions, multi-user concurrency, seat contention, forensic recovery, and multi-turn persistence across process boundaries.

---

## 2. Existing Code & Test Invariants

1. **`test_service_automated.rs` (`ST1..ST5`)**:
   - Standardizes comprehensive subsystem test coverage across multi-turn lifecycle, dependency DAGs, atomic persistence, configuration boundaries, and filtered querying.
2. **`session_service.rs` (`UserSessionService`)**:
   - Manages in-memory maps of sessions, status records, and seat allocations.
   - Enforces forward-only lifecycle state machines (`Initializing` $\to$ `Authenticating` $\to$ `Active` $\rightleftharpoons$ `Locked` $\to$ `Terminating` $\to$ `Terminated`).
   - Implements seat arbitration: activating a session on a physical/virtual seat demotes prior foreground sessions on that seat to `Background`.
3. **`test_session_suites.py`**:
   - Master test runner executing `SB1..SB5`.
   - Requires integration of criterion `SB6` (`automated integration tests`).

---

## 3. Authoritative Sources & Citations

1. **`loginctl(1)` & `systemd-logind(8)` Integration Semantics**:
   - Validates seat switching, multi-seat arbitration (`seat0`, `seat1`), session locking (`LockSession`, `UnlockSession`), and termination signals (`TerminateSession`, `TerminateUser`).
   - Citation: systemd v255 manual pages, *loginctl(1)*, *systemd-logind.service(8)*.
2. **PAM Multi-Session Concurrency Standards**:
   - Governs concurrent login boundaries, environment variable hygiene (`XDG_RUNTIME_DIR`), and user isolation across distinct UID/GIDs.
   - Citation: Linux-PAM Project, *pam_limits(8)*, *pam_systemd(8)*.
3. **ADR-0035 §D-2 (Deterministic Result Envelopes & Audit Guarantees)**:
   - Every multi-turn session event must produce deterministic JSON envelopes and append immutable audit records.

---

## 4. Facts vs. Assumptions

| Item | Status | Details |
|---|---|---|
| Seat Arbitration | **Fact** | At most one session per seat can hold `SessionScope::Foreground`. Prior foreground sessions must be demoted to `SessionScope::Background`. |
| Terminal State Immutability | **Fact** | Once a session transitions to `Terminated`, no subsequent actions are permitted. |
| User Capacity Cap | **Fact** | A user cannot hold more than `max_sessions_per_user` (default: 32) active sessions simultaneously. |
| Test Runner Criterion | **Assumption (to codify)** | Automated tests should be codified under criteria `SBT1..SBT5` and exposed as master suite criterion `SB6`. |

---

## 5. Decisions & Criteria Framework (`SBT1..SBT5`)

1. **`SBT1`**: Multi-Turn Lifecycle FSM Cohesion & Invalid Transition Rejection.
2. **`SBT2`**: Physical & Virtual Seat Arbitration with Automatic Foreground Demotion.
3. **`SBT3`**: Per-User Quota and Global Capacity Saturation Enforcement.
4. **`SBT4`**: Atomic Store Serialization, Recovery, and Corrupt Store Quarantine.
5. **`SBT5`**: Multi-Field Catalog Introspection & Query Filtering.
