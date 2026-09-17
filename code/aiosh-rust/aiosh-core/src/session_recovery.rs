//! Recovery, health check, and deep validation for the User Session Bootstrap subsystem.
//!
//! Provides automated non-destructive self-healing, timestamped quarantine of damaged
//! stores, and deep validation reports satisfying invariants SSR1..SSR5.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::session::{
    validate_user_session_spec, validate_user_session_status, SessionScope, SessionState,
    UserSessionStore,
};
use crate::session_policy::UserSessionSecurityPolicy;
use crate::session_service::UserSessionService;

/// Maximum total sessions permissible in a store before flagging capacity violation.
pub const MAX_STORE_CAPACITY: usize = 10_000;

/// Action taken during store loading and corruption recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionRecoveryAction {
    LoadedExisting,
    CreatedDefaultFresh,
    RecoveredFromBackup { backup_path: String, reason: String },
}

/// Comprehensive deep validation report across managed session specifications and statuses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionValidationReport {
    pub store_path: String,
    pub total_sessions: usize,
    pub valid_sessions: usize,
    pub invalid_sessions: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}

impl SessionValidationReport {
    /// Validates mathematical and consistency invariants SSR1..SSR3.
    pub fn validate_invariants(&self) -> Result<(), String> {
        // SSR1: valid + invalid == total
        if self.valid_sessions + self.invalid_sessions != self.total_sessions {
            return Err(format!(
                "invariant SSR1 violated: valid ({}) + invalid ({}) != total ({})",
                self.valid_sessions, self.invalid_sessions, self.total_sessions
            ));
        }

        // SSR2: healthy == (errors.is_empty() && invalid_sessions == 0)
        let expected_healthy = self.errors.is_empty() && self.invalid_sessions == 0;
        if self.healthy != expected_healthy {
            return Err(format!(
                "invariant SSR2 violated: healthy ({}) != expected ({})",
                self.healthy, expected_healthy
            ));
        }

        // SSR3: invalid_sessions > 0 => errors.len() >= invalid_sessions
        if self.invalid_sessions > 0 && self.errors.len() < self.invalid_sessions {
            return Err(format!(
                "invariant SSR3 violated: error count ({}) < invalid session count ({})",
                self.errors.len(), self.invalid_sessions
            ));
        }

        Ok(())
    }
}

/// Creates a timestamped backup copy of a damaged or corrupted store file.
pub fn create_backup_file(path: &Path) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%6f").to_string();
    let file_name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "sessions.json".to_string());
    let parent = path.parent();

    let mut counter = 0;
    let mut backup_path;
    loop {
        let suffix = if counter == 0 {
            format!("{}.bak.{}", file_name, timestamp)
        } else {
            format!("{}.bak.{}.{}", file_name, timestamp, counter)
        };
        backup_path = match parent {
            Some(p) => p.join(&suffix),
            None => PathBuf::from(&suffix),
        };
        if !backup_path.exists() || counter >= 10_000 {
            break;
        }
        counter += 1;
    }

    if path.exists() {
        let _ = fs::copy(path, &backup_path);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&backup_path, fs::Permissions::from_mode(0o600));
        }
    }

    backup_path
}

/// Pure validation function inspecting all sessions, seats, and invariants in a UserSessionStore.
pub fn validate_session_store(
    store: &UserSessionStore,
    store_path: &Path,
) -> SessionValidationReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut valid_sessions = 0;
    let mut invalid_sessions = 0;

    let total_sessions = store.specs.len();

    // Capacity check
    if total_sessions > MAX_STORE_CAPACITY {
        errors.push(format!(
            "store exceeds maximum capacity of {} sessions (was {})",
            MAX_STORE_CAPACITY, total_sessions
        ));
    }

    // Check status map length alignment
    if store.sessions.len() != total_sessions {
        errors.push(format!(
            "store specs count ({}) does not match session statuses count ({})",
            total_sessions,
            store.sessions.len()
        ));
    }

    // Per-session spec and status validation
    for (key, spec) in &store.specs {
        let mut session_errs = Vec::new();

        if key != &spec.session_id {
            session_errs.push(format!(
                "store key '{}' does not match session_id '{}'",
                key, spec.session_id
            ));
        }

        if let Err(spec_errs) = validate_user_session_spec(spec) {
            session_errs.extend(spec_errs);
        }

        match store.sessions.get(key) {
            Some(status) => {
                if &status.session_id != key {
                    session_errs.push(format!(
                        "status session_id '{}' does not match key '{}'",
                        status.session_id, key
                    ));
                }
                if status.username != spec.username {
                    session_errs.push(format!(
                        "status username '{}' does not match spec username '{}'",
                        status.username, spec.username
                    ));
                }
                if status.uid != spec.uid {
                    session_errs.push(format!(
                        "status uid ({}) does not match spec uid ({})",
                        status.uid, spec.uid
                    ));
                }
                if let Err(status_errs) = validate_user_session_status(status) {
                    session_errs.extend(status_errs);
                }
            }
            None => {
                session_errs.push(format!("missing status entry for session '{}'", key));
            }
        }

        if session_errs.is_empty() {
            valid_sessions += 1;
        } else {
            invalid_sessions += 1;
            errors.extend(session_errs);
        }
    }

    // Leader PID collision detection across non-terminated sessions
    let mut seen_pids: HashSet<u32> = HashSet::new();
    for (id, status) in &store.sessions {
        if status.state != SessionState::Terminated {
            if let Some(pid) = status.leader_pid {
                if !seen_pids.insert(pid) {
                    errors.push(format!(
                        "duplicate leader_pid {} detected on non-terminated session '{}'",
                        pid, id
                    ));
                    if store.specs.contains_key(id) && valid_sessions > 0 {
                        valid_sessions -= 1;
                        invalid_sessions += 1;
                    }
                }
            }
        }
    }

    // Seat mutual exclusion check (at most 1 Foreground session per seat)
    let mut foreground_seats: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, status) in &store.sessions {
        if status.state != SessionState::Terminated && status.scope == SessionScope::Foreground {
            if let Some(spec) = store.specs.get(id) {
                foreground_seats
                    .entry(spec.seat.clone())
                    .or_default()
                    .push(id.clone());
            }
        }
    }
    for (seat, sessions) in foreground_seats {
        if sessions.len() > 1 {
            errors.push(format!(
                "seat '{}' has {} concurrent foreground sessions: {:?}",
                seat,
                sessions.len(),
                sessions
            ));
            warnings.push(format!(
                "seat arbitration required on '{}': demote secondary sessions to background",
                seat
            ));
        }
    }

    let healthy = errors.is_empty() && invalid_sessions == 0;

    SessionValidationReport {
        store_path: store_path.to_string_lossy().to_string(),
        total_sessions,
        valid_sessions,
        invalid_sessions,
        errors,
        warnings,
        healthy,
        evaluated_at: Utc::now().to_rfc3339(),
    }
}

/// Non-destructive quarantine: backs up damaged store to `.bak.<timestamp>` and initializes fresh store.
pub fn recover_session_store_with_backup(path: &Path) -> (UserSessionStore, PathBuf) {
    let backup_path = create_backup_file(path);
    let fresh_service = UserSessionService::new();
    let fresh_store = fresh_service.store;
    let _ = fresh_store.save_to_path(path);
    (fresh_store, backup_path)
}

/// High-level entrypoint: loads, validates, and optionally repairs a session store.
pub fn load_or_recover(
    path: &Path,
) -> Result<(UserSessionService, SessionValidationReport, bool, Option<PathBuf>), String> {
    if !path.exists() {
        let service = UserSessionService::new();
        service
            .store
            .save_to_path(path)
            .map_err(|e| format!("Failed to create initial session store at '{}': {}", path.display(), e))?;
        let report = validate_session_store(&service.store, path);
        return Ok((service, report, true, None));
    }

    match UserSessionService::load_from_path(path) {
        Ok(service) => {
            let report = validate_session_store(&service.store, path);
            if report.healthy {
                Ok((service, report, false, None))
            } else {
                let (recovered_store, backup_path) = recover_session_store_with_backup(path);
                let fresh_service = UserSessionService {
                    store: recovered_store,
                    policy: UserSessionSecurityPolicy::default(),
                };
                let fresh_report = validate_session_store(&fresh_service.store, path);
                Ok((fresh_service, fresh_report, true, Some(backup_path)))
            }
        }
        Err(_) => {
            let (recovered_store, backup_path) = recover_session_store_with_backup(path);
            let fresh_service = UserSessionService {
                store: recovered_store,
                policy: UserSessionSecurityPolicy::default(),
            };
            let fresh_report = validate_session_store(&fresh_service.store, path);
            Ok((fresh_service, fresh_report, true, Some(backup_path)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{SessionClass, SessionType, UserSessionSpec, UserSessionStatus};
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_default_store_healthy() {
        let service = UserSessionService::new();
        let path = Path::new("/var/run/aios/sessions.json");
        let report = validate_session_store(&service.store, path);

        assert!(report.healthy);
        assert_eq!(report.invalid_sessions, 0);
        assert!(report.errors.is_empty());
        assert_eq!(report.valid_sessions, service.store.specs.len());
        assert_eq!(report.total_sessions, service.store.specs.len());
        assert!(report.validate_invariants().is_ok());
    }

    #[test]
    fn test_validate_invalid_spec_and_status() {
        let mut store = UserSessionStore::new();
        let bad_spec = UserSessionSpec {
            session_id: "bad_sess".to_string(),
            username: "BAD_USER!".to_string(), // uppercase and ! are invalid
            uid: 1000,
            gid: 1000,
            session_type: SessionType::X11,
            session_class: SessionClass::User,
            seat: "seat0".to_string(),
            vtnr: Some(1),
            display: None, // X11 requires display
            remote_host: None,
            environment: BTreeMap::new(),
        };
        let bad_status = UserSessionStatus {
            session_id: "bad_sess".to_string(),
            username: "kali".to_string(), // mismatched username
            uid: 1000,
            state: SessionState::Active,
            scope: SessionScope::Foreground,
            leader_pid: Some(500),
            created_at: "2026-09-01T00:00:00Z".to_string(),
            last_active_at: "2026-09-01T00:00:00Z".to_string(),
            idle_seconds: 0,
            locked: false,
        };
        store.specs.insert("bad_sess".to_string(), bad_spec);
        store.sessions.insert("bad_sess".to_string(), bad_status);

        let report = validate_session_store(&store, Path::new("dummy.json"));
        assert!(!report.healthy);
        assert_eq!(report.invalid_sessions, 1);
        assert!(report.errors.len() >= 1);
        assert!(report.validate_invariants().is_ok());
    }

    #[test]
    fn test_duplicate_leader_pid_detection() {
        let mut store = UserSessionStore::new();
        let spec1 = UserSessionSpec {
            session_id: "s1".to_string(),
            username: "kali".to_string(),
            uid: 1000,
            gid: 1000,
            session_type: SessionType::Tty,
            session_class: SessionClass::User,
            seat: "seat0".to_string(),
            vtnr: Some(1),
            display: None,
            remote_host: None,
            environment: BTreeMap::new(),
        };
        let status1 = UserSessionStatus {
            session_id: "s1".to_string(),
            username: "kali".to_string(),
            uid: 1000,
            state: SessionState::Active,
            scope: SessionScope::Foreground,
            leader_pid: Some(9999),
            created_at: "2026-09-01T00:00:00Z".to_string(),
            last_active_at: "2026-09-01T00:00:00Z".to_string(),
            idle_seconds: 0,
            locked: false,
        };
        let spec2 = UserSessionSpec {
            session_id: "s2".to_string(),
            username: "kali".to_string(),
            uid: 1000,
            gid: 1000,
            session_type: SessionType::Tty,
            session_class: SessionClass::User,
            seat: "seat1".to_string(),
            vtnr: Some(1),
            display: None,
            remote_host: None,
            environment: BTreeMap::new(),
        };
        let status2 = UserSessionStatus {
            session_id: "s2".to_string(),
            username: "kali".to_string(),
            uid: 1000,
            state: SessionState::Active,
            scope: SessionScope::Foreground,
            leader_pid: Some(9999), // duplicate pid 9999!
            created_at: "2026-09-01T00:00:00Z".to_string(),
            last_active_at: "2026-09-01T00:00:00Z".to_string(),
            idle_seconds: 0,
            locked: false,
        };
        assert!(store.add_session(spec1, status1).is_ok());
        assert!(store.add_session(spec2, status2).is_ok());

        let report = validate_session_store(&store, Path::new("dummy.json"));
        assert!(!report.healthy);
        assert!(report.errors.iter().any(|e| e.contains("duplicate leader_pid 9999")));
        assert!(report.validate_invariants().is_ok());
    }

    #[test]
    fn test_seat_mutual_exclusion_violation() {
        let mut store = UserSessionStore::new();
        let spec1 = UserSessionSpec {
            session_id: "s1".to_string(),
            username: "kali".to_string(),
            uid: 1000,
            gid: 1000,
            session_type: SessionType::Tty,
            session_class: SessionClass::User,
            seat: "seat0".to_string(),
            vtnr: Some(1),
            display: None,
            remote_host: None,
            environment: BTreeMap::new(),
        };
        let status1 = UserSessionStatus {
            session_id: "s1".to_string(),
            username: "kali".to_string(),
            uid: 1000,
            state: SessionState::Active,
            scope: SessionScope::Foreground,
            leader_pid: Some(1001),
            created_at: "2026-09-01T00:00:00Z".to_string(),
            last_active_at: "2026-09-01T00:00:00Z".to_string(),
            idle_seconds: 0,
            locked: false,
        };
        let spec2 = UserSessionSpec {
            session_id: "s2".to_string(),
            username: "kali".to_string(),
            uid: 1000,
            gid: 1000,
            session_type: SessionType::Tty,
            session_class: SessionClass::User,
            seat: "seat0".to_string(),
            vtnr: Some(2),
            display: None,
            remote_host: None,
            environment: BTreeMap::new(),
        };
        let status2 = UserSessionStatus {
            session_id: "s2".to_string(),
            username: "kali".to_string(),
            uid: 1000,
            state: SessionState::Active,
            scope: SessionScope::Foreground, // both foreground on seat0!
            leader_pid: Some(1002),
            created_at: "2026-09-01T00:00:00Z".to_string(),
            last_active_at: "2026-09-01T00:00:00Z".to_string(),
            idle_seconds: 0,
            locked: false,
        };
        assert!(store.add_session(spec1, status1).is_ok());
        assert!(store.add_session(spec2, status2).is_ok());

        let report = validate_session_store(&store, Path::new("dummy.json"));
        assert!(!report.healthy);
        assert!(report.errors.iter().any(|e| e.contains("seat 'seat0' has 2 concurrent foreground sessions")));
    }

    #[test]
    fn test_load_or_recover_workflow() {
        let tf = NamedTempFile::new().unwrap();
        let store_path = tf.path().to_path_buf();

        // 1. Initial valid store
        let (s1, rep1, rec1, bak1) = load_or_recover(&store_path).unwrap();
        assert!(rep1.healthy);
        assert!(bak1.is_none());
        assert!(!rec1);
        assert!(s1.get_session("greeter-seat0").is_some());

        // 2. Corrupt store with garbage
        fs::write(&store_path, b"{ corrupted not valid json ]").unwrap();
        let (_s2, rep2, rec2, bak2) = load_or_recover(&store_path).unwrap();
        assert!(rep2.healthy);
        assert!(rec2);
        assert!(bak2.is_some());
        let backup_path = bak2.unwrap();
        assert!(backup_path.exists());
        let _ = fs::remove_file(backup_path);

        // 3. Reload recovered store
        let (_s3, rep3, rec3, bak3) = load_or_recover(&store_path).unwrap();
        assert!(rep3.healthy);
        assert!(!rec3);
        assert!(bak3.is_none());
    }
}
