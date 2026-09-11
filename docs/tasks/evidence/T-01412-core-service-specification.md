# T-01412: User Session Bootstrap - Core Service: Specification

## Metadata
- **Task ID:** `T-01412`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Core Service Specification (`code/aiosh-rust/aiosh-core::session_service`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (2/10) — Core Service Specification

---

## 1. Scope & Architectural Goals

The **User Session Bootstrap Core Service** (`session_service.rs`) coordinates the runtime lifecycle, seat arbitration, idle tracking, and persistent state management of user and autonomous agent sessions in AIOS.

It consumes the foundational data structures established in `session.rs` (`T-01401..T-01410`) and implements the central engine (`UserSessionService`) and action execution reporting (`UserSessionActionReport`).

---

## 2. Reused vs. New Interfaces

### 2.1 Reused Interfaces (from `code/aiosh-rust/aiosh-core::session`)
- **Enums**:
  - `SessionType`: `Tty`, `X11`, `Wayland`, `AiAgent`.
  - `SessionClass`: `User`, `Greeter`, `LockScreen`, `Background`, `Agent`.
  - `SessionState`: `Initializing`, `Authenticating`, `Active`, `Locked`, `Terminating`, `Terminated`.
  - `SessionScope`: `Foreground`, `Background`.
  - `UserSessionAction`: `Create`, `Authenticate`, `Activate`, `Lock`, `Unlock`, `Terminate`.
- **Structures**:
  - `UserSessionSpec`: Declarative session definition.
  - `UserSessionStatus`: Runtime status tracking.
  - `UserSessionQuery`: Multi-attribute search criteria.
  - `UserSessionStore`: Persistent and in-memory container.
- **Validation Functions & Invariants**:
  - `validate_session_id`, `validate_username`, `validate_user_session_spec`, `transition_session_state`, `validate_user_session_status`.
  - Constants: `MAX_SESSION_STORE_SIZE` (10 MiB), `MAX_SESSIONS_PER_USER` (32), `MAX_TOTAL_SESSIONS` (1024).

### 2.2 New Interfaces (introduced in `session_service.rs`)
- `UserSessionActionReport`: Typed execution report capturing pre- and post-states, action, success/failure, error explanation, and RFC-3339 timestamp.
- `UserSessionService`: The central service coordinator managing session creation, seat assignment arbitration, action dispatch, idle progression, and disk persistence.

---

## 3. Detailed Data Structures & API Contracts

### 3.1 `UserSessionActionReport`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSessionActionReport {
    /// Identifier of the targeted session.
    pub session_id: String,
    /// Administrative action executed.
    pub action: UserSessionAction,
    /// Session state immediately prior to action execution.
    pub previous_state: SessionState,
    /// Session state resulting from the action.
    pub new_state: SessionState,
    /// Whether the action succeeded completely.
    pub success: bool,
    /// Descriptive error message if action failed.
    pub error: Option<String>,
    /// RFC-3339 timestamp of action execution.
    pub timestamp: String,
}
```

### 3.2 `UserSessionService`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSessionService {
    /// In-memory and persistent session store.
    pub store: UserSessionStore,
}

impl UserSessionService {
    /// Initializes a service pre-seeded with a canonical greeter session on seat0.
    pub fn new() -> Self;

    /// Initializes an unseeded, empty session service for isolated testing.
    pub fn empty() -> Self;

    /// Registers and initializes a new session from a specification (CS1, CS3).
    pub fn create_session(&mut self, spec: UserSessionSpec) -> Result<UserSessionActionReport, String>;

    /// Executes an administrative lifecycle action on a session (CS1, CS2, CS5).
    pub fn apply_action(&mut self, session_id: &str, action: UserSessionAction) -> Result<UserSessionActionReport, String>;

    /// Searches tracked sessions matching query criteria.
    pub fn query_sessions(&self, query: &UserSessionQuery) -> Vec<UserSessionStatus>;

    /// Retrieves session runtime status by session ID.
    pub fn get_session(&self, session_id: &str) -> Option<&UserSessionStatus>;

    /// Retrieves session specification by session ID.
    pub fn get_spec(&self, session_id: &str) -> Option<&UserSessionSpec>;

    /// Lists all tracked sessions sorted by session ID.
    pub fn list_sessions(&self) -> Vec<&UserSessionStatus>;

    /// Updates idle duration for a session in seconds (CS5).
    pub fn update_idle(&mut self, session_id: &str, idle_seconds: u64) -> Result<(), String>;

    /// Records operator/user activity on a session, resetting idle seconds to 0 (CS5).
    pub fn touch_activity(&mut self, session_id: &str) -> Result<(), String>;

    /// Atomically serializes the service's store to disk (CS1).
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> io::Result<()>;

    /// Loads the service state from an existing JSON store on disk (CS1).
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> io::Result<Self>;
}
```

---

## 4. Core Service Invariants (CS1..CS5)

1. **`CS1` — Action Atomicity & State Machine Validity**:
   - Every administrative action (`Create`, `Authenticate`, `Activate`, `Lock`, `Unlock`, `Terminate`) must execute atomically:
     - On success: store is updated, new state is persisted, and `UserSessionActionReport` has `success = true`, `error = None`.
     - On failure: store is completely unmodified, and error is returned (or report with `success = false`, `error = Some(...)`).
   - State transitions must strictly follow the transition matrix defined in `SB3`:
     - `Initializing` $\to$ `Authenticating` | `Terminating`
     - `Authenticating` $\to$ `Active` | `Terminating`
     - `Active` $\to$ `Locked` | `Terminating`
     - `Locked` $\to$ `Active` | `Terminating`
     - `Terminating` $\to$ `Terminated`
     - `Terminated` $\to$ Immutable (any action returns error).

2. **`CS2` — Seat Mutual Exclusion & Focus Arbitration**:
   - For any seat (e.g. `"seat0"`), at most one session may have `SessionScope::Foreground` at any time.
   - When session $S_{new}$ is activated on seat $K$, `UserSessionService` iterates through all sessions on seat $K$ and demotes any existing session with `SessionScope::Foreground` to `SessionScope::Background`.
   - $S_{new}$ is then assigned `SessionScope::Foreground`.

3. **`CS3` — Capacity Limits & Lifecycle Bounds**:
   - Creating a session strictly enforces:
     - Total sessions in store $< 1,024$.
     - Total active (non-terminated) sessions for the given user $< 32$.
   - Sessions in state `Terminated` do not count against the user's active session limit.

4. **`CS4` — Non-Repudiation & Action Reporting**:
   - Every state transition produces a `UserSessionActionReport` containing:
     - `session_id`, `action`, `previous_state`, `new_state`, `success`, `error`, and `timestamp` (RFC-3339).
   - This provides the necessary payload for Policy Enforcement Point (PEP) logging to the SQLite WAL audit ring.

5. **`CS5` — Idle Tracking & Lock Consistency**:
   - Consistency invariant:
     - `status.state == SessionState::Locked` $\iff$ `status.locked == true`.
     - `status.state == SessionState::Active` $\iff$ `status.locked == false`.
   - Idle tracking:
     - Calling `update_idle(session_id, seconds)` updates `idle_seconds`.
     - Calling `touch_activity(session_id)` resets `idle_seconds` to 0 and updates `last_active_at` to the current timestamp.

---

## 5. Happy Paths, Failure Paths & Edge Cases

### 5.1 Happy Paths
1. **Interactive User Session Lifecycle**:
   - `create_session(spec)`: initializes session in `Initializing` state, `Background` scope.
   - `apply_action(id, Authenticate)`: verifies credentials, transitions `Initializing` $\to$ `Authenticating` $\to$ `Active`.
   - `apply_action(id, Activate)`: acquires `Foreground` scope on seat, demotes prior session on seat to `Background`.
   - `apply_action(id, Lock)`: transitions to `Locked`, sets `locked = true`.
   - `apply_action(id, Unlock)`: transitions back to `Active`, sets `locked = false`.
   - `apply_action(id, Terminate)`: transitions to `Terminating` $\to$ `Terminated`, yields foreground scope.

2. **Autonomous AI Agent Session Lifecycle**:
   - `create_session(spec)`: with `SessionType::AiAgent`, `SessionClass::Agent`.
   - `apply_action(id, Authenticate)` $\to$ `Active`.
   - Remains in `SessionScope::Background` for headless tasks or switches to `Foreground` during active desktop co-pilot take-over.

### 5.2 Failure Paths & Error Conditions
1. **Invalid Specification**:
   - Rejection if `session_id` fails SB1 (e.g. contains slashes, `..`, whitespace).
   - Rejection if `username` fails SB2 (e.g. uppercase characters, invalid chars).
   - Rejection if `environment` fails SB4 (e.g. unquoted paths, traversal in `XDG_RUNTIME_DIR`).
2. **Duplicate Session ID**:
   - If `session_id` already exists in the store, `create_session` immediately returns `Err("Session with ID '...' already exists in store")`.
3. **Invalid State Transition**:
   - Calling `Unlock` on an `Active` session returns an error explaining that unlocking is only permitted from `Locked`.
   - Calling any action on a `Terminated` session returns an error stating that terminated sessions cannot be modified.
4. **Capacity Exhaustion**:
   - Attempting to create a 33rd active session for a single user returns `Err("User '...' has reached the maximum of 32 active sessions")`.
   - Attempting to exceed 1,024 total sessions returns `Err("Maximum total sessions limit reached (1024)")`.

---

## 6. Persistence & Store Layout

- **Store Path**: `/run/aios/sessions.json` by default.
- **Serialization Format**: UTF-8 JSON.
- **Atomic Persistence Protocol**:
  1. Write payload to temporary file `<path>.tmp.<pid>.<nanos>`.
  2. Sync data to disk (`flush()` / `sync_all()`).
  3. Atomically rename temporary file to target path (`fs::rename`).
- **Storage Cap**: Serialized store payload cannot exceed 10 MiB (`MAX_SESSION_STORE_SIZE`).

---

## 7. Next Steps

With the specification approved:
1. Proceed to **`T-01413`** (`User Session Bootstrap / core service: Scaffold`).
2. Implement typed skeletons and interfaces in `code/aiosh-rust/aiosh-core/src/session_service.rs`.
3. Wire exports in `code/aiosh-rust/aiosh-core/src/lib.rs`.
