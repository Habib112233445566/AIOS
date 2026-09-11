//! User Session Bootstrap Core Service (CS1..CS5)
//!
//! Provides the runtime UserSessionService coordinator, lifecycle action execution,
//! seat arbitration, idle tracking, and atomic persistence mechanisms for user and agent sessions.

use crate::session::{
    SessionClass, SessionScope, SessionState, SessionType, UserSessionAction, UserSessionQuery,
    UserSessionSpec, UserSessionStatus, UserSessionStore, MAX_SESSIONS_PER_USER, MAX_TOTAL_SESSIONS,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
use std::path::Path;

/// Report detailing the outcome of an administrative user session action.
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

/// Central runtime coordinator and store manager for user and agent sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSessionService {
    /// In-memory and persistent session store.
    pub store: UserSessionStore,
}

impl Default for UserSessionService {
    fn default() -> Self {
        Self::new()
    }
}

impl UserSessionService {
    /// Initializes a service pre-seeded with a canonical greeter session on seat0.
    pub fn new() -> Self {
        let mut store = UserSessionStore::new();

        let greeter_spec = UserSessionSpec {
            session_id: "greeter-seat0".to_string(),
            username: "lightdm".to_string(),
            uid: 62000,
            gid: 62000,
            session_type: SessionType::X11,
            session_class: SessionClass::Greeter,
            seat: "seat0".to_string(),
            vtnr: Some(7),
            display: Some(":0".to_string()),
            remote_host: None,
            environment: BTreeMap::new(),
        };

        let greeter_status = UserSessionStatus {
            session_id: "greeter-seat0".to_string(),
            username: "lightdm".to_string(),
            uid: 62000,
            state: SessionState::Active,
            scope: SessionScope::Foreground,
            leader_pid: Some(1001),
            created_at: "2026-09-09T00:00:00Z".to_string(),
            last_active_at: "2026-09-09T00:00:00Z".to_string(),
            idle_seconds: 0,
            locked: false,
        };

        let _ = store.add_session(greeter_spec, greeter_status);
        Self { store }
    }

    /// Initializes an unseeded, empty session service for isolated testing.
    pub fn empty() -> Self {
        Self {
            store: UserSessionStore::new(),
        }
    }

    /// Registers and initializes a new session from a specification (CS1, CS3).
    pub fn create_session(
        &mut self,
        spec: UserSessionSpec,
    ) -> Result<UserSessionActionReport, String> {
        if let Err(errs) = crate::session::validate_user_session_spec(&spec) {
            return Err(format!("Invalid session specification: {}", errs.join("; ")));
        }

        if self.store.sessions.contains_key(&spec.session_id) {
            return Err(format!(
                "Session with ID '{}' already exists in store",
                spec.session_id
            ));
        }

        if self.store.sessions.len() >= MAX_TOTAL_SESSIONS {
            return Err(format!(
                "Maximum total sessions limit reached ({})",
                MAX_TOTAL_SESSIONS
            ));
        }

        let active_user_sessions = self
            .store
            .sessions
            .values()
            .filter(|s| s.username == spec.username && s.state != SessionState::Terminated)
            .count();

        if active_user_sessions >= MAX_SESSIONS_PER_USER {
            return Err(format!(
                "User '{}' has reached the maximum of {} active sessions",
                spec.username, MAX_SESSIONS_PER_USER
            ));
        }

        let ts = chrono::Utc::now().to_rfc3339();
        let status = UserSessionStatus {
            session_id: spec.session_id.clone(),
            username: spec.username.clone(),
            uid: spec.uid,
            state: SessionState::Initializing,
            scope: SessionScope::Background,
            leader_pid: None,
            created_at: ts.clone(),
            last_active_at: ts.clone(),
            idle_seconds: 0,
            locked: false,
        };

        let session_id = spec.session_id.clone();
        self.store.sessions.insert(session_id.clone(), status);
        self.store.specs.insert(session_id.clone(), spec);

        Ok(UserSessionActionReport {
            session_id,
            action: UserSessionAction::Create,
            previous_state: SessionState::Initializing,
            new_state: SessionState::Initializing,
            success: true,
            error: None,
            timestamp: ts,
        })
    }

    /// Executes an administrative lifecycle action on a session (CS1, CS2, CS5).
    pub fn apply_action(
        &mut self,
        session_id: &str,
        action: UserSessionAction,
    ) -> Result<UserSessionActionReport, String> {
        let current_status = match self.store.sessions.get(session_id) {
            Some(s) => s.clone(),
            None => return Err(format!("Session '{}' not found in store", session_id)),
        };

        let prev_state = current_status.state;
        let next_state = crate::session::transition_session_state(prev_state, action)?;
        let ts = chrono::Utc::now().to_rfc3339();

        // Seat arbitration & focus management (CS2)
        if action == UserSessionAction::Activate {
            let target_seat = self.store.specs.get(session_id).map(|sp| sp.seat.clone());
            if let Some(seat_name) = target_seat {
                for (other_id, other_status) in self.store.sessions.iter_mut() {
                    if other_id != session_id {
                        if let Some(other_spec) = self.store.specs.get(other_id) {
                            if other_spec.seat == seat_name
                                && other_status.scope == SessionScope::Foreground
                            {
                                other_status.scope = SessionScope::Background;
                            }
                        }
                    }
                }
            }
        }

        // Mutate target session status
        let status = self
            .store
            .sessions
            .get_mut(session_id)
            .expect("Session exists");
        status.state = next_state;
        status.last_active_at = ts.clone();

        match action {
            UserSessionAction::Activate => {
                status.scope = SessionScope::Foreground;
                status.idle_seconds = 0;
            }
            UserSessionAction::Lock => {
                status.locked = true;
            }
            UserSessionAction::Unlock => {
                status.locked = false;
                status.idle_seconds = 0;
            }
            UserSessionAction::Terminate => {
                if next_state == SessionState::Terminated {
                    status.scope = SessionScope::Background;
                    status.locked = false;
                }
            }
            _ => {}
        }

        Ok(UserSessionActionReport {
            session_id: session_id.to_string(),
            action,
            previous_state: prev_state,
            new_state: next_state,
            success: true,
            error: None,
            timestamp: ts,
        })
    }

    /// Searches tracked sessions matching query criteria.
    pub fn query_sessions(&self, query: &UserSessionQuery) -> Vec<UserSessionStatus> {
        let mut results: Vec<UserSessionStatus> = self
            .store
            .sessions
            .values()
            .filter(|s| {
                if let Some(ref u) = query.username {
                    if &s.username != u {
                        return false;
                    }
                }
                if let Some(ref st) = query.state {
                    if &s.state != st {
                        return false;
                    }
                }
                if let Some(ref target_type) = query.session_type {
                    if let Some(spec) = self.store.specs.get(&s.session_id) {
                        if &spec.session_type != target_type {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                if let Some(ref target_seat) = query.seat {
                    if let Some(spec) = self.store.specs.get(&s.session_id) {
                        if &spec.seat != target_seat {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        if let Some(limit) = query.limit {
            if results.len() > limit {
                results.truncate(limit);
            }
        }
        results
    }

    /// Retrieves session runtime status by session ID.
    pub fn get_session(&self, session_id: &str) -> Option<&UserSessionStatus> {
        self.store.sessions.get(session_id)
    }

    /// Retrieves session specification by session ID.
    pub fn get_spec(&self, session_id: &str) -> Option<&UserSessionSpec> {
        self.store.specs.get(session_id)
    }

    /// Lists all tracked sessions sorted by session ID.
    pub fn list_sessions(&self) -> Vec<&UserSessionStatus> {
        self.store.sessions.values().collect()
    }

    /// Updates idle duration for a session in seconds (CS5).
    pub fn update_idle(&mut self, session_id: &str, idle_seconds: u64) -> Result<(), String> {
        let status = self
            .store
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session '{}' not found in store", session_id))?;
        status.idle_seconds = idle_seconds;
        Ok(())
    }

    /// Records operator/user activity on a session, resetting idle seconds to 0 (CS5).
    pub fn touch_activity(&mut self, session_id: &str) -> Result<(), String> {
        let status = self
            .store
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session '{}' not found in store", session_id))?;
        status.idle_seconds = 0;
        status.last_active_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// Atomically serializes the service's store to disk (CS1).
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.store.save_to_path(path.as_ref())
    }

    /// Loads the service state from an existing JSON store on disk (CS1).
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        if !path.as_ref().exists()
            || std::fs::metadata(path.as_ref())
                .map(|m| m.len() == 0)
                .unwrap_or(false)
        {
            return Ok(Self::new());
        }
        let store = UserSessionStore::load_from_path(path.as_ref())?;
        Ok(Self { store })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_user_spec(id: &str, user: &str, seat: &str) -> UserSessionSpec {
        UserSessionSpec {
            session_id: id.to_string(),
            username: user.to_string(),
            uid: 1000,
            gid: 1000,
            session_type: SessionType::X11,
            session_class: SessionClass::User,
            seat: seat.to_string(),
            vtnr: Some(7),
            display: Some(":0".to_string()),
            remote_host: None,
            environment: BTreeMap::new(),
        }
    }

    #[test]
    fn test_canonical_new_and_empty_service() {
        let service = UserSessionService::new();
        assert_eq!(service.store.sessions.len(), 1);
        let greeter = service.get_session("greeter-seat0").expect("greeter exists");
        assert_eq!(greeter.username, "lightdm");
        assert_eq!(greeter.state, SessionState::Active);
        assert_eq!(greeter.scope, SessionScope::Foreground);

        let empty = UserSessionService::empty();
        assert_eq!(empty.store.sessions.len(), 0);
    }

    #[test]
    fn test_session_lifecycle_progression_cs1() {
        let mut service = UserSessionService::empty();
        let spec = sample_user_spec("sess-01", "kali", "seat0");

        // 1. Create session -> Initializing
        let rep1 = service.create_session(spec).expect("Create session");
        assert_eq!(rep1.session_id, "sess-01");
        assert_eq!(rep1.action, UserSessionAction::Create);
        assert_eq!(rep1.new_state, SessionState::Initializing);
        assert!(rep1.success);

        // 2. Authenticate session -> Authenticating
        let rep2 = service
            .apply_action("sess-01", UserSessionAction::Authenticate)
            .expect("Authenticate session");
        assert_eq!(rep2.previous_state, SessionState::Initializing);
        assert_eq!(rep2.new_state, SessionState::Authenticating);

        // 3. Activate session -> Active (and Foreground)
        let rep3 = service
            .apply_action("sess-01", UserSessionAction::Activate)
            .expect("Activate session");
        assert_eq!(rep3.previous_state, SessionState::Authenticating);
        assert_eq!(rep3.new_state, SessionState::Active);
        let status = service.get_session("sess-01").unwrap();
        assert_eq!(status.scope, SessionScope::Foreground);
        assert!(!status.locked);

        // 4. Lock session -> Locked (CS5)
        let rep4 = service
            .apply_action("sess-01", UserSessionAction::Lock)
            .expect("Lock session");
        assert_eq!(rep4.new_state, SessionState::Locked);
        let status = service.get_session("sess-01").unwrap();
        assert!(status.locked);

        // 5. Unlock session -> Active (CS5)
        let rep5 = service
            .apply_action("sess-01", UserSessionAction::Unlock)
            .expect("Unlock session");
        assert_eq!(rep5.new_state, SessionState::Active);
        let status = service.get_session("sess-01").unwrap();
        assert!(!status.locked);

        // 6. Terminate session -> Terminating -> Terminated
        let rep6 = service
            .apply_action("sess-01", UserSessionAction::Terminate)
            .expect("Terminate session");
        assert_eq!(rep6.new_state, SessionState::Terminating);

        let rep7 = service
            .apply_action("sess-01", UserSessionAction::Terminate)
            .expect("Terminate session to Terminated");
        assert_eq!(rep7.new_state, SessionState::Terminated);
        let status = service.get_session("sess-01").unwrap();
        assert_eq!(status.scope, SessionScope::Background);

        // 7. Modifying terminated session must fail
        let err = service.apply_action("sess-01", UserSessionAction::Activate);
        assert!(err.is_err());
    }

    #[test]
    fn test_seat_arbitration_mutual_exclusion_cs2() {
        let mut service = UserSessionService::empty();
        let spec1 = sample_user_spec("sess-01", "kali", "seat0");
        let spec2 = sample_user_spec("sess-02", "alice", "seat0");

        service.create_session(spec1).unwrap();
        service.create_session(spec2).unwrap();

        // Authenticate both
        service
            .apply_action("sess-01", UserSessionAction::Authenticate)
            .unwrap();
        service
            .apply_action("sess-02", UserSessionAction::Authenticate)
            .unwrap();

        // Activate sess-01 -> foreground on seat0
        service
            .apply_action("sess-01", UserSessionAction::Activate)
            .unwrap();
        assert_eq!(
            service.get_session("sess-01").unwrap().scope,
            SessionScope::Foreground
        );
        assert_eq!(
            service.get_session("sess-02").unwrap().scope,
            SessionScope::Background
        );

        // Activate sess-02 -> demotes sess-01 to background, sess-02 becomes foreground
        service
            .apply_action("sess-02", UserSessionAction::Activate)
            .unwrap();
        assert_eq!(
            service.get_session("sess-01").unwrap().scope,
            SessionScope::Background
        );
        assert_eq!(
            service.get_session("sess-02").unwrap().scope,
            SessionScope::Foreground
        );

        // Re-activate sess-01 -> demotes sess-02 to background, sess-01 becomes foreground
        service
            .apply_action("sess-01", UserSessionAction::Activate)
            .unwrap();
        assert_eq!(
            service.get_session("sess-01").unwrap().scope,
            SessionScope::Foreground
        );
        assert_eq!(
            service.get_session("sess-02").unwrap().scope,
            SessionScope::Background
        );
    }

    #[test]
    fn test_capacity_limits_cs3() {
        let mut service = UserSessionService::empty();

        // Fill user sessions up to limit
        for i in 0..MAX_SESSIONS_PER_USER {
            let id = format!("sess-user-{}", i);
            let mut spec = sample_user_spec(&id, "kali", "seat0");
            spec.vtnr = Some((i % 12 + 1) as u32);
            service.create_session(spec).unwrap();
        }

        // 33rd session for user 'kali' must be rejected
        let spec_over = sample_user_spec("sess-user-33", "kali", "seat0");
        let err = service.create_session(spec_over);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("maximum of 32 active sessions"));

        // Another user can still create a session
        let spec_alice = sample_user_spec("sess-alice-01", "alice", "seat0");
        assert!(service.create_session(spec_alice).is_ok());
    }

    #[test]
    fn test_idle_and_activity_tracking_cs5() {
        let mut service = UserSessionService::empty();
        let spec = sample_user_spec("sess-01", "kali", "seat0");
        service.create_session(spec).unwrap();

        assert_eq!(service.get_session("sess-01").unwrap().idle_seconds, 0);

        service.update_idle("sess-01", 300).unwrap();
        assert_eq!(service.get_session("sess-01").unwrap().idle_seconds, 300);

        service.touch_activity("sess-01").unwrap();
        assert_eq!(service.get_session("sess-01").unwrap().idle_seconds, 0);
    }

    #[test]
    fn test_query_and_filtering() {
        let mut service = UserSessionService::empty();
        let spec1 = sample_user_spec("sess-01", "kali", "seat0");
        let mut spec2 = sample_user_spec("sess-02", "alice", "seat1");
        spec2.session_type = SessionType::Tty;
        spec2.display = None;

        service.create_session(spec1).unwrap();
        service.create_session(spec2).unwrap();

        let q_user = UserSessionQuery {
            username: Some("kali".to_string()),
            ..Default::default()
        };
        let res = service.query_sessions(&q_user);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].session_id, "sess-01");

        let q_seat = UserSessionQuery {
            seat: Some("seat1".to_string()),
            ..Default::default()
        };
        let res = service.query_sessions(&q_seat);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].session_id, "sess-02");
    }

    #[test]
    fn test_save_and_load_persistence() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let store_path = temp_dir.path().join("sessions.json");

        let mut service = UserSessionService::new();
        let spec = sample_user_spec("sess-persist", "kali", "seat0");
        service.create_session(spec).unwrap();

        service.save_to_path(&store_path).expect("save store");
        assert!(store_path.exists());

        let loaded = UserSessionService::load_from_path(&store_path).expect("load store");
        assert_eq!(loaded.store.sessions.len(), 2);
        assert!(loaded.get_session("greeter-seat0").is_some());
        assert!(loaded.get_session("sess-persist").is_some());
    }
}
