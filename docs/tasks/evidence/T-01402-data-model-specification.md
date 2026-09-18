# T-01402: User Session Bootstrap - Data Model: Specification

## Metadata
- **Task ID:** `T-01402`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap (`code/aiosh-rust/aiosh-core::session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (2/10) — Data Model Specification

---

## 1. Scope & System Interfaces

This specification defines the concrete data structures, lifecycle state enumerations, validation functions, and persistence contracts for the AIOS User Session Bootstrap data model (`code/aiosh-rust/aiosh-core/src/session.rs`).

### Reused Interfaces
- `serde::{Serialize, Deserialize}`: Canonical JSON serialization and deserialization.
- `std::collections::BTreeMap`: Deterministic key-value ordering for environment variables and session maps.
- Standard ISO 8601 timestamps (`chrono::Utc` / RFC 3339 strings) for session creation, activation, and audit records.
- Standard Result/Error envelopes: `Result<T, Vec<String>>` for spec validation and `Result<T, String>` for single-item validation.
- Non-repudiation audit ring buffer (`code/aiosh-rust/aiosh-core/src/audit.rs`) for state-changing lifecycle operations.

### Upstream Alignment & AIOS Specifics
- **Upstream Alignment**:
  - Aligned with `sd-login(3)` and `org.freedesktop.login1(5)` session classes (`user`, `greeter`, `lock-screen`, `background`) and session types (`tty`, `x11`, `wayland`).
  - Aligned with XDG Base Directory specification (`XDG_RUNTIME_DIR`, `XDG_SESSION_ID`, `XDG_SESSION_TYPE`, `XDG_SEAT`).
  - Aligned with POSIX.1-2017 user credentials (UID, GID, username, leader PID).
- **AIOS Specifics**:
  - `SessionType::AiAgent` and `SessionClass::Agent` representing first-class autonomous AI agent execution sessions.
  - Strict mathematical invariants (`SB1..SB5`) ensuring bounded memory, zero panic risk, and protection against resource exhaustion.
  - Deterministic serialization with `#[serde(rename_all = "snake_case")]` for JSON-RPC MCP tools and CLI outputs.

---

## 2. Core Data Structures (`code/aiosh-rust/aiosh-core/src/session.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Execution environment type of the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    /// Text console bound to a physical or virtual TTY (/dev/tty1..tty6).
    Tty,
    /// X Window System desktop session (e.g. XFCE / Kali Undercover on :0).
    X11,
    /// Wayland compositor desktop session (e.g. KWin / labwc).
    Wayland,
    /// Autonomous AI execution context (headless or desktop co-pilot).
    AiAgent,
}

/// Functional classification of the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionClass {
    /// Standard interactive human operator session.
    User,
    /// Display manager / login greeter session prior to authentication.
    Greeter,
    /// Lock-screen overlay session.
    LockScreen,
    /// Non-interactive or lingering background service session.
    Background,
    /// Autonomous AI assistant / co-pilot session.
    Agent,
}

/// Lifecycle state of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Session created, awaiting credential setup.
    Initializing,
    /// Authentication / PAM challenge in progress.
    Authenticating,
    /// Session authenticated and running.
    Active,
    /// Session locked; input muted, requires unlocking.
    Locked,
    /// Tear-down in progress, terminating processes.
    Terminating,
    /// Session completely closed; resources freed.
    Terminated,
}

/// Focus scope of the session on its seat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionScope {
    /// Currently receiving seat input / display focus.
    Foreground,
    /// Running detached or in background.
    Background,
}

/// Canonical specification for initializing or configuring a user session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSessionSpec {
    /// Unique session identifier (e.g. "sess-01", "c1").
    pub session_id: String,
    /// POSIX username (e.g. "kali", "root", "aios-agent").
    pub username: String,
    /// POSIX User ID (UID).
    pub uid: u32,
    /// POSIX Primary Group ID (GID).
    pub gid: u32,
    /// Session display / execution type.
    pub session_type: SessionType,
    /// Functional session class.
    pub session_class: SessionClass,
    /// Physical or virtual seat identifier (e.g. "seat0").
    pub seat: String,
    /// Virtual terminal number (e.g. 1..12).
    pub vtnr: Option<u32>,
    /// X11/Wayland display string (e.g. ":0").
    pub display: Option<String>,
    /// Remote host address if connecting over network (SSH/RDP).
    pub remote_host: Option<String>,
    /// Custom session environment variables.
    pub environment: BTreeMap<String, String>,
}

/// Runtime snapshot of an active or managed session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSessionStatus {
    /// Session identifier.
    pub session_id: String,
    /// POSIX username.
    pub username: String,
    /// POSIX User ID.
    pub uid: u32,
    /// Current lifecycle state.
    pub state: SessionState,
    /// Focus scope on seat.
    pub scope: SessionScope,
    /// Session leader process ID (PID).
    pub leader_pid: Option<u32>,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
    /// Last active / input timestamp (ISO 8601).
    pub last_active_at: String,
    /// Idle duration in seconds.
    pub idle_seconds: u64,
    /// Whether the session is currently locked.
    pub locked: bool,
}

/// Administrative action performed on a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserSessionAction {
    /// Create / initialize a new session.
    Create,
    /// Authenticate credentials.
    Authenticate,
    /// Activate session to foreground.
    Activate,
    /// Lock session screen/input.
    Lock,
    /// Unlock session.
    Unlock,
    /// Terminate session and cleanup processes.
    Terminate,
}

/// Query filter for discovering sessions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSessionQuery {
    pub username: Option<String>,
    pub state: Option<SessionState>,
    pub session_type: Option<SessionType>,
    pub seat: Option<String>,
    pub limit: Option<usize>,
}

/// Canonical in-memory & persistent store of sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSessionStore {
    /// Schema format version.
    pub version: u32,
    /// Map of active / tracked sessions by session_id.
    pub sessions: BTreeMap<String, UserSessionStatus>,
    /// Map of session specifications by session_id.
    pub specs: BTreeMap<String, UserSessionSpec>,
}
```

---

## 3. Formal Invariant Specifications (`SB1..SB5`)

### Invariant SB1: Session Identifier Syntax & Sizing
- **Rule**: A session ID must be non-empty, start with an ASCII alphanumeric character, and contain only `[a-zA-Z0-9_.-]`.
- **Length**: $1 \le \text{len}(\text{session\_id}) \le 64$.
- **Prohibitions**: Forward slashes (`/`), backward slashes (`\`), null bytes (`\0`), directory traversal sequences (`..`), shell metacharacters (`;`, `&`, `|`, `>`, `<`, `$`), and leading dots or hyphens.

### Invariant SB2: Identity & Seat Bounds
- **Username**: Must match POSIX regex `^[a-z_][a-z0-9_-]{0,31}$` (or `"root"` / `"aios-agent"`), length $1 \le \text{len} \le 32$.
- **UID / GID**: Must be $\le 2,147,483,647$ (valid positive 32-bit user/group IDs).
- **Seat**: Must match `^seat[a-zA-Z0-9_-]{0,27}$`, length $1 \le \text{len} \le 32$. Defaults to `"seat0"`.
- **VTNR**: If specified, must be between 1 and 12 (standard Linux VT allocation).
- **Display**: If specified, must match `^:[0-9]{1,3}(\.[0-9]{1,3})?$`, length $\le 16$.

### Invariant SB3: Lifecycle State Machine Transitions
Deterministic state machine transition matrix:

| Current State | Permitted Action | Next State |
|---|---|---|
| `Initializing` | `Authenticate` | `Authenticating` |
| `Initializing` | `Terminate` | `Terminated` |
| `Authenticating` | `Activate` | `Active` |
| `Authenticating` | `Terminate` | `Terminated` |
| `Active` | `Lock` | `Locked` |
| `Active` | `Terminate` | `Terminating` |
| `Locked` | `Unlock` | `Active` |
| `Locked` | `Terminate` | `Terminating` |
| `Terminating` | `Terminate` | `Terminated` |
| `Terminated` | *(any)* | **ERROR**: Terminal state, no further transitions permitted |

Any invalid transition (e.g. `Terminated` $\to$ `Activate`, or `Initializing` $\to$ `Lock`) returns `Err("Invalid session state transition")`.

### Invariant SB4: Environment Variables & Security Isolation
- Environment variable key: Must match `^[A-Z_][A-Z0-9_]{0,63}$`.
- Environment variable value: Bounded to $\le 4,096$ bytes.
- Total environment variables: $\le 256$ entries per session.
- `XDG_RUNTIME_DIR`: If specified, must be an absolute Unix path (`/run/user/<UID>`) without `..` traversal.

### Invariant SB5: Resource Caps & Sizing Boundaries
- Maximum concurrent sessions per user: $\le 32$.
- Maximum system-wide sessions in store: $\le 1,024$.
- Serialized store size limit: $\le 10$ MiB.

---

## 4. Validation Functions Contract

```rust
/// Validates session identifier syntax against SB1.
pub fn validate_session_id(id: &str) -> Result<(), String>;

/// Validates username syntax against SB2.
pub fn validate_username(name: &str) -> Result<(), String>;

/// Validates complete session specification against SB1..SB5.
pub fn validate_user_session_spec(spec: &UserSessionSpec) -> Result<(), Vec<String>>;

/// Evaluates state machine transition validity against SB3.
pub fn transition_session_state(
    current: SessionState,
    action: UserSessionAction,
) -> Result<SessionState, String>;
```

---

## 5. Happy Path, Failure Paths, and Audit Effects

### Happy Path (Interactive Session Bootstrapping)
1. **Spec Creation**: Operator or greeter submits `UserSessionSpec` (e.g., `kali` on `seat0`, display `:0`, type `X11`, class `User`).
2. **Validation**: `validate_user_session_spec` returns `Ok(())`.
3. **Initialization**: Session created in state `Initializing`.
4. **Authentication**: PAM challenge succeeds -> State transitions to `Authenticating` -> `Active`.
5. **Audit Emission**: State transition emits an audit row with `tool: "aios.session"`, `command: "create"`, `outcome: "ok"`.

### Failure Paths
1. **Invalid ID / Malicious Username**: `validate_session_id("../evil")` or `validate_username("; rm -rf /")` fails immediately with descriptive validation errors. Zero state change.
2. **Capacity Limit Exceeded**: Attempting to create a 33rd session for user `kali` returns `Err("User kali has reached the maximum of 32 concurrent sessions")`.
3. **Invalid Lifecycle Transition**: Calling `Activate` on an already `Terminated` session returns `Err("Cannot activate a terminated session")`.
4. **Audit Emission on Consequential Failure**: State-changing failures write an audit row with `outcome: "refused"` or `"error"`.

---

## 6. Persistence Contract

- **Default Location**: `/run/aios/sessions.json` (volatile tmpfs) or custom store path via `--store`.
- **Atomic Writes**: Save operations write to temporary file (`/run/aios/sessions.json.tmp.<epoch>`), sync to disk, and atomically rename over target path.
- **Default Baseline**: When initialized fresh, store contains zero active sessions or canonical defaults (e.g. system console `tty1`).

---

## 7. Acceptance Verification
- [x] Inputs, outputs, error cases, and persistence effects formally defined.
- [x] Reused interfaces (`serde`, `BTreeMap`, `audit.rs`) vs AIOS-specific additions explicitly documented.
- [x] Happy path, failure paths, and audit effects specified.
- [x] Complete specifications reviewable independently without reading source code.
