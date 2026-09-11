//! User Session Bootstrap Data Model (SB1..SB5)
//!
//! Provides canonical data structures, state machines, and validation logic for
//! AIOS user session bootstrapping across console, graphical, and autonomous AI contexts.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

/// Maximum size allowed for serialized session store files (10 MiB).
pub const MAX_SESSION_STORE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum sessions allowed per individual user.
pub const MAX_SESSIONS_PER_USER: usize = 32;

/// Maximum sessions allowed system-wide in the store.
pub const MAX_TOTAL_SESSIONS: usize = 1024;

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

impl Default for UserSessionStore {
    fn default() -> Self {
        Self {
            version: 1,
            sessions: BTreeMap::new(),
            specs: BTreeMap::new(),
        }
    }
}

impl UserSessionStore {
    /// Creates a fresh, empty session store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a validated session specification and status to the store (enforcing SB5).
    pub fn add_session(
        &mut self,
        spec: UserSessionSpec,
        status: UserSessionStatus,
    ) -> Result<(), String> {
        // Enforce SB1..SB4 on spec
        if let Err(errs) = validate_user_session_spec(&spec) {
            return Err(format!("Invalid session specification: {}", errs.join("; ")));
        }

        // Enforce consistency on status
        if let Err(errs) = validate_user_session_status(&status) {
            return Err(format!("Invalid session status: {}", errs.join("; ")));
        }

        if spec.session_id != status.session_id {
            return Err(format!(
                "Session ID mismatch between spec ('{}') and status ('{}')",
                spec.session_id, status.session_id
            ));
        }

        // Duplicate check
        if self.sessions.contains_key(&spec.session_id) {
            return Err(format!(
                "Session with ID '{}' already exists in store",
                spec.session_id
            ));
        }

        // SB5: Capacity checks
        if self.sessions.len() >= MAX_TOTAL_SESSIONS {
            return Err(format!(
                "Maximum total sessions limit reached ({})",
                MAX_TOTAL_SESSIONS
            ));
        }

        let user_sessions = self
            .sessions
            .values()
            .filter(|s| s.username == spec.username && s.state != SessionState::Terminated)
            .count();

        if user_sessions >= MAX_SESSIONS_PER_USER {
            return Err(format!(
                "User '{}' has reached the maximum of {} active sessions",
                spec.username, MAX_SESSIONS_PER_USER
            ));
        }

        self.sessions.insert(status.session_id.clone(), status);
        self.specs.insert(spec.session_id.clone(), spec);

        Ok(())
    }

    /// Retrieves session status by identifier.
    pub fn get_session(&self, session_id: &str) -> Option<&UserSessionStatus> {
        self.sessions.get(session_id)
    }

    /// Retrieves session specification by identifier.
    pub fn get_spec(&self, session_id: &str) -> Option<&UserSessionSpec> {
        self.specs.get(session_id)
    }

    /// Lists sessions matching the provided query filter.
    pub fn list_sessions(&self, query: &UserSessionQuery) -> Vec<&UserSessionStatus> {
        let mut results: Vec<&UserSessionStatus> = self
            .sessions
            .values()
            .filter(|status| {
                if let Some(ref u) = query.username {
                    if &status.username != u {
                        return false;
                    }
                }
                if let Some(st) = query.state {
                    if status.state != st {
                        return false;
                    }
                }
                if let Some(stype) = query.session_type {
                    if let Some(spec) = self.specs.get(&status.session_id) {
                        if spec.session_type != stype {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                if let Some(ref seat) = query.seat {
                    if let Some(spec) = self.specs.get(&status.session_id) {
                        if &spec.seat != seat {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            })
            .collect();

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        results
    }

    /// Applies a lifecycle state transition to an existing session (enforcing SB3).
    pub fn apply_action(
        &mut self,
        session_id: &str,
        action: UserSessionAction,
    ) -> Result<UserSessionStatus, String> {
        let status = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session '{}' not found in store", session_id))?;

        let next_state = transition_session_state(status.state, action)?;
        status.state = next_state;

        match next_state {
            SessionState::Active => {
                status.locked = false;
                status.scope = SessionScope::Foreground;
            }
            SessionState::Locked => {
                status.locked = true;
            }
            SessionState::Terminating => {
                status.scope = SessionScope::Background;
            }
            SessionState::Terminated => {
                status.locked = false;
                status.leader_pid = None;
                status.scope = SessionScope::Background;
            }
            _ => {}
        }

        Ok(status.clone())
    }

    /// Removes a session from the store.
    pub fn remove_session(&mut self, session_id: &str) -> Option<UserSessionStatus> {
        self.specs.remove(session_id);
        self.sessions.remove(session_id)
    }

    /// Serializes the store to a JSON string.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Serialization error: {}", e))
    }

    /// Deserializes a store from a JSON string, enforcing the 10 MiB limit and session count ceilings.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        if json_str.len() > MAX_SESSION_STORE_SIZE {
            return Err(format!(
                "Session store payload exceeds maximum permitted size of {} bytes (was {})",
                MAX_SESSION_STORE_SIZE,
                json_str.len()
            ));
        }

        let store: Self = serde_json::from_str(json_str)
            .map_err(|e| format!("Deserialization error: {}", e))?;

        if store.sessions.len() > MAX_TOTAL_SESSIONS {
            return Err(format!(
                "Session store exceeds maximum permitted total sessions of {} (was {})",
                MAX_TOTAL_SESSIONS,
                store.sessions.len()
            ));
        }

        Ok(store)
    }

    /// Atomically persists the store to the specified file path with bounded retries and clean temp cleanup.
    pub fn save_to_path(&self, path: &Path) -> io::Result<()> {
        let json = self
            .to_json()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = path.with_extension(format!(
            "tmp.{}.{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));

        if let Err(e) = fs::write(&temp_path, json.as_bytes()) {
            let _ = fs::remove_file(&temp_path);
            return Err(e);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600));
        }

        // Bounded retries for rename (up to 3 attempts with backoff) to handle transient locks
        let mut rename_result = fs::rename(&temp_path, path);
        if rename_result.is_err() {
            for _ in 0..3 {
                std::thread::sleep(std::time::Duration::from_millis(10));
                rename_result = fs::rename(&temp_path, path);
                if rename_result.is_ok() {
                    break;
                }
            }
        }

        if let Err(e) = rename_result {
            let _ = fs::remove_file(&temp_path);
            return Err(e);
        }

        Ok(())
    }

    /// Reads and deserializes the store from the specified file path.
    pub fn load_from_path(path: &Path) -> io::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let metadata = fs::metadata(path)?;
        if metadata.len() == 0 {
            return Ok(Self::new());
        }
        if metadata.len() as usize > MAX_SESSION_STORE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Session store file size exceeds 10 MiB limit (was {} bytes)",
                    metadata.len()
                ),
            ));
        }

        let contents = fs::read_to_string(path)?;
        Self::from_json(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

/// Validates session identifier syntax against SB1.
pub fn validate_session_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("session ID cannot be empty".into());
    }
    if id.len() > 64 {
        return Err(format!(
            "session ID exceeds 64 characters (was {})",
            id.len()
        ));
    }

    let bytes = id.as_bytes();
    if !bytes[0].is_ascii_alphanumeric() {
        return Err(format!(
            "session ID must start with an ASCII alphanumeric character: '{}'",
            id
        ));
    }

    if id.contains('/') || id.contains('\\') || id.contains('\0') {
        return Err(format!(
            "session ID contains prohibited path separator or null byte: '{}'",
            id
        ));
    }

    if id.contains("..") {
        return Err(format!(
            "session ID contains prohibited path traversal sequence '..': '{}'",
            id
        ));
    }

    for &b in bytes {
        let valid = b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.';
        if !valid {
            return Err(format!(
                "session ID contains invalid character '{}' in '{}'",
                b as char, id
            ));
        }
    }

    Ok(())
}

/// Validates username syntax against SB2.
pub fn validate_username(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("username cannot be empty".into());
    }
    if name.len() > 32 {
        return Err(format!(
            "username exceeds 32 characters (was {})",
            name.len()
        ));
    }

    let bytes = name.as_bytes();
    // Must start with lowercase alphabetic, underscore, or valid system account
    let first = bytes[0];
    let valid_start = first.is_ascii_lowercase() || first == b'_';
    if !valid_start {
        return Err(format!(
            "username must start with a lowercase ASCII letter or underscore: '{}'",
            name
        ));
    }

    for &b in bytes {
        let valid = b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-';
        if !valid {
            return Err(format!(
                "username contains invalid character '{}' in '{}'",
                b as char, name
            ));
        }
    }

    Ok(())
}

/// Validates complete session specification against SB1..SB5.
pub fn validate_user_session_spec(spec: &UserSessionSpec) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // SB1: Session ID
    if let Err(err) = validate_session_id(&spec.session_id) {
        errors.push(err);
    }

    // SB2: Username & User ID
    if let Err(err) = validate_username(&spec.username) {
        errors.push(err);
    }

    if spec.uid > 2_147_483_647 {
        errors.push(format!("UID exceeds maximum value 2147483647: {}", spec.uid));
    }
    if spec.gid > 2_147_483_647 {
        errors.push(format!("GID exceeds maximum value 2147483647: {}", spec.gid));
    }

    // Seat validation
    if spec.seat.is_empty() {
        errors.push("seat identifier cannot be empty".into());
    } else if spec.seat.len() > 32 {
        errors.push(format!(
            "seat identifier exceeds 32 characters (was {})",
            spec.seat.len()
        ));
    } else if !spec.seat.starts_with("seat") {
        errors.push(format!(
            "seat identifier must start with prefix 'seat': '{}'",
            spec.seat
        ));
    }

    // VTNR bounds
    if let Some(vtnr) = spec.vtnr {
        if !(1..=12).contains(&vtnr) {
            errors.push(format!(
                "virtual terminal number (vtnr) must be between 1 and 12 (was {})",
                vtnr
            ));
        }
    } else if spec.session_type == SessionType::Tty {
        errors.push("TTY session type requires a valid virtual terminal number (vtnr)".into());
    }

    // Display validation
    if let Some(ref disp) = spec.display {
        if disp.is_empty() {
            errors.push("display string cannot be empty when specified".into());
        } else if disp.len() > 16 {
            errors.push(format!(
                "display string exceeds 16 characters (was {})",
                disp.len()
            ));
        } else if !disp.starts_with(':') {
            errors.push(format!(
                "display string must start with colon ':': '{}'",
                disp
            ));
        }
    } else if spec.session_type == SessionType::X11 {
        errors.push("X11 session type requires a display identifier (e.g. ':0')".into());
    }

    // Remote host validation
    if let Some(ref host) = spec.remote_host {
        if host.is_empty() {
            errors.push("remote_host cannot be empty when specified".into());
        } else if host.len() > 255 {
            errors.push(format!(
                "remote_host exceeds 255 characters (was {})",
                host.len()
            ));
        } else if host.contains(' ') || host.contains('\0') {
            errors.push("remote_host contains prohibited whitespace or null byte".into());
        }
    }

    // SB4: Environment validation
    if spec.environment.len() > 256 {
        errors.push(format!(
            "environment map exceeds 256 items (was {})",
            spec.environment.len()
        ));
    }

    for (k, v) in &spec.environment {
        if k.is_empty() {
            errors.push("environment key cannot be empty".into());
        } else if k.len() > 64 {
            errors.push(format!(
                "environment key '{}' exceeds 64 characters",
                k
            ));
        } else {
            let k_bytes = k.as_bytes();
            let first = k_bytes[0];
            if !first.is_ascii_uppercase() && first != b'_' {
                errors.push(format!(
                    "environment key '{}' must start with an uppercase ASCII letter or underscore",
                    k
                ));
            }
            for &b in k_bytes {
                let valid = b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_';
                if !valid {
                    errors.push(format!(
                        "environment key '{}' contains invalid character '{}'",
                        k, b as char
                    ));
                    break;
                }
            }
        }

        if v.len() > 4096 {
            errors.push(format!(
                "environment value for key '{}' exceeds 4096 characters",
                k
            ));
        }
        if v.contains('\0') {
            errors.push(format!(
                "environment value for key '{}' contains null byte",
                k
            ));
        }

        if k == "XDG_RUNTIME_DIR" {
            let is_unix_abs = v.starts_with('/');
            if !is_unix_abs {
                errors.push(format!(
                    "XDG_RUNTIME_DIR must be an absolute path: '{}'",
                    v
                ));
            }
            if v.contains("..") {
                errors.push(format!(
                    "XDG_RUNTIME_DIR contains prohibited path traversal sequence '..': '{}'",
                    v
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Evaluates state machine transition validity against SB3.
pub fn transition_session_state(
    current: SessionState,
    action: UserSessionAction,
) -> Result<SessionState, String> {
    match (current, action) {
        (SessionState::Initializing, UserSessionAction::Authenticate) => {
            Ok(SessionState::Authenticating)
        }
        (SessionState::Initializing, UserSessionAction::Terminate) => {
            Ok(SessionState::Terminated)
        }
        (SessionState::Authenticating, UserSessionAction::Activate) => Ok(SessionState::Active),
        (SessionState::Authenticating, UserSessionAction::Terminate) => {
            Ok(SessionState::Terminated)
        }
        (SessionState::Active, UserSessionAction::Activate) => Ok(SessionState::Active),
        (SessionState::Active, UserSessionAction::Lock) => Ok(SessionState::Locked),
        (SessionState::Active, UserSessionAction::Terminate) => Ok(SessionState::Terminating),
        (SessionState::Locked, UserSessionAction::Unlock) => Ok(SessionState::Active),
        (SessionState::Locked, UserSessionAction::Terminate) => Ok(SessionState::Terminating),
        (SessionState::Terminating, UserSessionAction::Terminate) => {
            Ok(SessionState::Terminated)
        }
        (SessionState::Terminated, _) => Err(format!(
            "Cannot perform action '{:?}' on terminated session",
            action
        )),
        _ => Err(format!(
            "Invalid session state transition: state '{:?}' cannot perform action '{:?}'",
            current, action
        )),
    }
}

/// Validates runtime session status snapshot consistency.
pub fn validate_user_session_status(status: &UserSessionStatus) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Err(err) = validate_session_id(&status.session_id) {
        errors.push(err);
    }
    if let Err(err) = validate_username(&status.username) {
        errors.push(err);
    }

    if status.created_at.is_empty() {
        errors.push("created_at timestamp cannot be empty".into());
    }
    if status.last_active_at.is_empty() {
        errors.push("last_active_at timestamp cannot be empty".into());
    }

    // Consistency checks
    if status.state == SessionState::Locked && !status.locked {
        errors.push("Session state is Locked, but locked flag is false".into());
    }
    if status.state == SessionState::Active && status.locked {
        errors.push("Session state is Active, but locked flag is true".into());
    }
    if status.state == SessionState::Terminated && status.scope == SessionScope::Foreground {
        errors.push("Terminated session cannot be in Foreground scope".into());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_x11_spec() -> UserSessionSpec {
        let mut env = BTreeMap::new();
        env.insert(
            "XDG_RUNTIME_DIR".into(),
            "/run/user/1000".into(),
        );
        env.insert("DISPLAY".into(), ":0".into());

        UserSessionSpec {
            session_id: "sess-01".into(),
            username: "kali".into(),
            uid: 1000,
            gid: 1000,
            session_type: SessionType::X11,
            session_class: SessionClass::User,
            seat: "seat0".into(),
            vtnr: Some(7),
            display: Some(":0".into()),
            remote_host: None,
            environment: env,
        }
    }

    fn valid_status() -> UserSessionStatus {
        UserSessionStatus {
            session_id: "sess-01".into(),
            username: "kali".into(),
            uid: 1000,
            state: SessionState::Active,
            scope: SessionScope::Foreground,
            leader_pid: Some(1234),
            created_at: "2026-09-09T12:00:00Z".into(),
            last_active_at: "2026-09-09T12:05:00Z".into(),
            idle_seconds: 10,
            locked: false,
        }
    }

    #[test]
    fn test_valid_spec_happy_path() {
        let spec = valid_x11_spec();
        assert!(validate_user_session_spec(&spec).is_ok());
    }

    #[test]
    fn test_session_id_syntax_sb1() {
        assert!(validate_session_id("sess-01").is_ok());
        assert!(validate_session_id("c1").is_ok());
        assert!(validate_session_id("agent_session.02").is_ok());

        assert!(validate_session_id("").is_err());
        assert!(validate_session_id("-sess").is_err());
        assert!(validate_session_id("../evil").is_err());
        assert!(validate_session_id("sess/01").is_err());
        assert!(validate_session_id("sess 01").is_err());
        assert!(validate_session_id(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_username_syntax_sb2() {
        assert!(validate_username("kali").is_ok());
        assert!(validate_username("root").is_ok());
        assert!(validate_username("_apt").is_ok());
        assert!(validate_username("aios-agent").is_ok());

        assert!(validate_username("").is_err());
        assert!(validate_username("Kali").is_err()); // uppercase rejected
        assert!(validate_username("1user").is_err()); // starts with digit
        assert!(validate_username("user;evil").is_err());
        assert!(validate_username(&"u".repeat(33)).is_err());
    }

    #[test]
    fn test_state_machine_transitions_sb3() {
        let s1 = transition_session_state(
            SessionState::Initializing,
            UserSessionAction::Authenticate,
        )
        .expect("transition succeeds");
        assert_eq!(s1, SessionState::Authenticating);

        let s2 = transition_session_state(s1, UserSessionAction::Activate)
            .expect("transition succeeds");
        assert_eq!(s2, SessionState::Active);

        let s3 = transition_session_state(s2, UserSessionAction::Lock)
            .expect("transition succeeds");
        assert_eq!(s3, SessionState::Locked);

        let s4 = transition_session_state(s3, UserSessionAction::Unlock)
            .expect("transition succeeds");
        assert_eq!(s4, SessionState::Active);

        let s5 = transition_session_state(s4, UserSessionAction::Terminate)
            .expect("transition succeeds");
        assert_eq!(s5, SessionState::Terminating);

        let s6 = transition_session_state(s5, UserSessionAction::Terminate)
            .expect("transition succeeds");
        assert_eq!(s6, SessionState::Terminated);

        // Invalid transitions
        assert!(transition_session_state(SessionState::Terminated, UserSessionAction::Activate).is_err());
        assert!(transition_session_state(SessionState::Initializing, UserSessionAction::Lock).is_err());
        assert!(transition_session_state(SessionState::Locked, UserSessionAction::Authenticate).is_err());
    }

    #[test]
    fn test_environment_and_path_isolation_sb4() {
        let mut spec = valid_x11_spec();
        spec.environment.insert(
            "XDG_RUNTIME_DIR".into(),
            "/run/user/../evil".into(),
        );
        let errs = validate_user_session_spec(&spec).expect_err("should reject path traversal");
        assert!(errs.iter().any(|e| e.contains("traversal sequence '..'")));

        let mut spec2 = valid_x11_spec();
        spec2.environment.insert("invalid key".into(), "val".into());
        let errs2 = validate_user_session_spec(&spec2).expect_err("should reject lowercase/space key");
        assert!(errs2.iter().any(|e| e.contains("must start with an uppercase ASCII letter")));
    }

    #[test]
    fn test_store_capacity_and_user_limits_sb5() {
        let mut store = UserSessionStore::new();
        let spec = valid_x11_spec();
        let status = valid_status();

        store
            .add_session(spec.clone(), status.clone())
            .expect("first session added");

        // Duplicate rejection
        assert!(store.add_session(spec.clone(), status.clone()).is_err());

        // Fill user sessions up to limit
        for i in 2..=MAX_SESSIONS_PER_USER {
            let mut s = spec.clone();
            s.session_id = format!("sess-{:02}", i);
            let mut st = status.clone();
            st.session_id = format!("sess-{:02}", i);
            store.add_session(s, st).expect("session added within cap");
        }

        // 33rd session for user 'kali' rejected
        let mut overflow_spec = spec.clone();
        overflow_spec.session_id = "sess-overflow".into();
        let mut overflow_st = status.clone();
        overflow_st.session_id = "sess-overflow".into();
        let err = store
            .add_session(overflow_spec, overflow_st)
            .expect_err("should exceed user session limit");
        assert!(err.contains("maximum of 32 active sessions"));
    }

    #[test]
    fn test_store_lifecycle_action_and_query() {
        let mut store = UserSessionStore::new();
        let spec = valid_x11_spec();
        let mut status = valid_status();
        status.state = SessionState::Initializing;
        status.locked = false;

        store.add_session(spec, status).expect("added");

        let updated = store
            .apply_action("sess-01", UserSessionAction::Authenticate)
            .expect("authenticated");
        assert_eq!(updated.state, SessionState::Authenticating);

        let activated = store
            .apply_action("sess-01", UserSessionAction::Activate)
            .expect("activated");
        assert_eq!(activated.state, SessionState::Active);
        assert_eq!(activated.scope, SessionScope::Foreground);

        let locked = store
            .apply_action("sess-01", UserSessionAction::Lock)
            .expect("locked");
        assert_eq!(locked.state, SessionState::Locked);
        assert!(locked.locked);

        // Query listing
        let query = UserSessionQuery {
            username: Some("kali".into()),
            state: Some(SessionState::Locked),
            session_type: Some(SessionType::X11),
            seat: Some("seat0".into()),
            limit: Some(10),
        };
        let matches = store.list_sessions(&query);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].session_id, "sess-01");
    }

    #[test]
    fn test_store_serialization_and_deserialization() {
        let mut store = UserSessionStore::new();
        store
            .add_session(valid_x11_spec(), valid_status())
            .expect("added");

        let json = store.to_json().expect("to_json succeeds");
        let restored = UserSessionStore::from_json(&json).expect("from_json succeeds");
        assert_eq!(store, restored);
    }
}
