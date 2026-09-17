//! Focused automated unit and integration tests for User Session Bootstrap Recovery & Validation.
//!
//! Enforces invariants SSR1..SSR5, deep specification validation, seat arbitration,
//! process leader collision detection, non-destructive quarantine backups, and self-healing.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use aiosh_core::session::{
    SessionClass, SessionScope, SessionState, SessionType, UserSessionSpec, UserSessionStatus,
    UserSessionStore,
};
use aiosh_core::session_recovery::{
    load_or_recover, recover_session_store_with_backup, validate_session_store,
    SessionValidationReport, MAX_STORE_CAPACITY,
};

#[test]
fn test_ssr1_ssr2_ssr3_invariant_equations() {
    // 1. SSR1 violation: valid + invalid != total
    let report_bad_ssr1 = SessionValidationReport {
        store_path: "/tmp/sessions.json".into(),
        total_sessions: 10,
        valid_sessions: 5,
        invalid_sessions: 3, // sum is 8 != 10
        errors: vec!["err1".into(), "err2".into(), "err3".into()],
        warnings: vec![],
        healthy: false,
        evaluated_at: "2026-09-16T00:00:00Z".into(),
    };
    let err = report_bad_ssr1.validate_invariants().unwrap_err();
    assert!(err.contains("SSR1 violated"), "must detect SSR1 violation: {}", err);

    // 2. SSR2 violation: healthy is true but errors exist
    let report_bad_ssr2a = SessionValidationReport {
        store_path: "/tmp/sessions.json".into(),
        total_sessions: 2,
        valid_sessions: 2,
        invalid_sessions: 0,
        errors: vec!["hidden error".into()],
        warnings: vec![],
        healthy: true, // violation
        evaluated_at: "2026-09-16T00:00:00Z".into(),
    };
    let err = report_bad_ssr2a.validate_invariants().unwrap_err();
    assert!(err.contains("SSR2 violated"), "must detect SSR2 violation: {}", err);

    // 3. SSR2 violation: healthy is true but invalid_sessions > 0
    let report_bad_ssr2b = SessionValidationReport {
        store_path: "/tmp/sessions.json".into(),
        total_sessions: 2,
        valid_sessions: 1,
        invalid_sessions: 1,
        errors: vec![],
        warnings: vec![],
        healthy: true, // violation
        evaluated_at: "2026-09-16T00:00:00Z".into(),
    };
    let err = report_bad_ssr2b.validate_invariants().unwrap_err();
    assert!(err.contains("SSR2 violated"), "must detect SSR2 violation: {}", err);

    // 4. SSR3 violation: errors.len() < invalid_sessions
    let report_bad_ssr3 = SessionValidationReport {
        store_path: "/tmp/sessions.json".into(),
        total_sessions: 5,
        valid_sessions: 2,
        invalid_sessions: 3,
        errors: vec!["only one error".into()], // only 1 error for 3 invalid sessions
        warnings: vec![],
        healthy: false,
        evaluated_at: "2026-09-16T00:00:00Z".into(),
    };
    let err = report_bad_ssr3.validate_invariants().unwrap_err();
    assert!(err.contains("SSR3 violated"), "must detect SSR3 violation: {}", err);

    // 5. Valid report satisfying all SSR invariants
    let report_valid = SessionValidationReport {
        store_path: "/tmp/sessions.json".into(),
        total_sessions: 5,
        valid_sessions: 3,
        invalid_sessions: 2,
        errors: vec!["err 1".into(), "err 2".into()],
        warnings: vec![],
        healthy: false,
        evaluated_at: "2026-09-16T00:00:00Z".into(),
    };
    assert!(report_valid.validate_invariants().is_ok());
}

#[test]
fn test_default_store_deep_validation() {
    let store = UserSessionStore::new();
    let report = validate_session_store(&store, Path::new("/var/run/aios/sessions.json"));

    assert!(report.healthy, "empty store must be healthy");
    assert_eq!(report.invalid_sessions, 0);
    assert_eq!(report.valid_sessions, store.specs.len());
    assert_eq!(report.valid_sessions, report.total_sessions);
    assert!(report.errors.is_empty());
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_negative_session_specs_and_status_invariants() {
    let mut store = UserSessionStore::new();

    // 1. Invalid username (capital letters and symbols)
    let bad_username_spec = UserSessionSpec {
        session_id: "sess_baduser".into(),
        username: "ILLEGAL_USER!".into(),
        uid: 1000,
        gid: 1000,
        session_type: SessionType::Tty,
        session_class: SessionClass::User,
        seat: "seat0".into(),
        vtnr: Some(1),
        display: None,
        remote_host: None,
        environment: BTreeMap::new(),
    };
    let bad_username_status = UserSessionStatus {
        session_id: "sess_baduser".into(),
        username: "ILLEGAL_USER!".into(),
        uid: 1000,
        state: SessionState::Active,
        scope: SessionScope::Foreground,
        leader_pid: Some(101),
        created_at: "2026-09-16T00:00:00Z".into(),
        last_active_at: "2026-09-16T00:00:00Z".into(),
        idle_seconds: 0,
        locked: false,
    };
    store.specs.insert("sess_baduser".into(), bad_username_spec);
    store.sessions.insert("sess_baduser".into(), bad_username_status);

    // 2. Missing display for X11 session
    let bad_x11_spec = UserSessionSpec {
        session_id: "sess_nox11disp".into(),
        username: "alice".into(),
        uid: 1001,
        gid: 1001,
        session_type: SessionType::X11,
        session_class: SessionClass::User,
        seat: "seat0".into(),
        vtnr: Some(2),
        display: None, // Missing display for X11!
        remote_host: None,
        environment: BTreeMap::new(),
    };
    let bad_x11_status = UserSessionStatus {
        session_id: "sess_nox11disp".into(),
        username: "alice".into(),
        uid: 1001,
        state: SessionState::Active,
        scope: SessionScope::Background,
        leader_pid: Some(102),
        created_at: "2026-09-16T00:00:00Z".into(),
        last_active_at: "2026-09-16T00:00:00Z".into(),
        idle_seconds: 0,
        locked: false,
    };
    store.specs.insert("sess_nox11disp".into(), bad_x11_spec);
    store.sessions.insert("sess_nox11disp".into(), bad_x11_status);

    // 3. Store key mismatch
    let mismatched_key_spec = UserSessionSpec {
        session_id: "sess_real_id".into(),
        username: "bob".into(),
        uid: 1002,
        gid: 1002,
        session_type: SessionType::Wayland,
        session_class: SessionClass::User,
        seat: "seat1".into(),
        vtnr: None,
        display: Some("wayland-0".into()),
        remote_host: None,
        environment: BTreeMap::new(),
    };
    let mismatched_key_status = UserSessionStatus {
        session_id: "sess_real_id".into(),
        username: "bob".into(),
        uid: 1002,
        state: SessionState::Active,
        scope: SessionScope::Foreground,
        leader_pid: Some(103),
        created_at: "2026-09-16T00:00:00Z".into(),
        last_active_at: "2026-09-16T00:00:00Z".into(),
        idle_seconds: 0,
        locked: false,
    };
    store.specs.insert("sess_wrong_key".into(), mismatched_key_spec);
    store.sessions.insert("sess_wrong_key".into(), mismatched_key_status);

    // 4. Missing status entry (orphan spec)
    let orphan_spec = UserSessionSpec {
        session_id: "sess_orphan".into(),
        username: "charlie".into(),
        uid: 1003,
        gid: 1003,
        session_type: SessionType::Tty,
        session_class: SessionClass::User,
        seat: "seat2".into(),
        vtnr: Some(3),
        display: None,
        remote_host: None,
        environment: BTreeMap::new(),
    };
    store.specs.insert("sess_orphan".into(), orphan_spec);

    let report = validate_session_store(&store, Path::new("/var/run/aios/sessions.json"));
    assert!(!report.healthy);
    assert_eq!(report.invalid_sessions, 4);
    assert!(report.errors.len() >= 4);
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_seat_mutual_exclusion_violation() {
    let mut store = UserSessionStore::new();

    let spec1 = UserSessionSpec {
        session_id: "sess_fg1".into(),
        username: "user1".into(),
        uid: 1001,
        gid: 1001,
        session_type: SessionType::Tty,
        session_class: SessionClass::User,
        seat: "seat0".into(),
        vtnr: Some(1),
        display: None,
        remote_host: None,
        environment: BTreeMap::new(),
    };
    let status1 = UserSessionStatus {
        session_id: "sess_fg1".into(),
        username: "user1".into(),
        uid: 1001,
        state: SessionState::Active,
        scope: SessionScope::Foreground,
        leader_pid: Some(201),
        created_at: "2026-09-16T00:00:00Z".into(),
        last_active_at: "2026-09-16T00:00:00Z".into(),
        idle_seconds: 0,
        locked: false,
    };

    let spec2 = UserSessionSpec {
        session_id: "sess_fg2".into(),
        username: "user2".into(),
        uid: 1002,
        gid: 1002,
        session_type: SessionType::Tty,
        session_class: SessionClass::User,
        seat: "seat0".into(), // Same seat!
        vtnr: Some(2),
        display: None,
        remote_host: None,
        environment: BTreeMap::new(),
    };
    let status2 = UserSessionStatus {
        session_id: "sess_fg2".into(),
        username: "user2".into(),
        uid: 1002,
        state: SessionState::Active,
        scope: SessionScope::Foreground, // Multiple foreground sessions on seat0!
        leader_pid: Some(202),
        created_at: "2026-09-16T00:00:00Z".into(),
        last_active_at: "2026-09-16T00:00:00Z".into(),
        idle_seconds: 0,
        locked: false,
    };

    store.specs.insert("sess_fg1".into(), spec1);
    store.sessions.insert("sess_fg1".into(), status1);
    store.specs.insert("sess_fg2".into(), spec2);
    store.sessions.insert("sess_fg2".into(), status2);

    let report = validate_session_store(&store, Path::new("test.json"));
    assert!(!report.healthy);
    assert!(report.errors.iter().any(|e| e.contains("concurrent foreground sessions")));
    assert!(report.warnings.iter().any(|w| w.contains("seat arbitration required")));
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_duplicate_leader_pid_collision() {
    let mut store = UserSessionStore::new();

    let spec1 = UserSessionSpec {
        session_id: "sess_p1".into(),
        username: "user1".into(),
        uid: 1001,
        gid: 1001,
        session_type: SessionType::Tty,
        session_class: SessionClass::User,
        seat: "seat0".into(),
        vtnr: Some(1),
        display: None,
        remote_host: None,
        environment: BTreeMap::new(),
    };
    let status1 = UserSessionStatus {
        session_id: "sess_p1".into(),
        username: "user1".into(),
        uid: 1001,
        state: SessionState::Active,
        scope: SessionScope::Foreground,
        leader_pid: Some(4040),
        created_at: "2026-09-16T00:00:00Z".into(),
        last_active_at: "2026-09-16T00:00:00Z".into(),
        idle_seconds: 0,
        locked: false,
    };

    let spec2 = UserSessionSpec {
        session_id: "sess_p2".into(),
        username: "user2".into(),
        uid: 1002,
        gid: 1002,
        session_type: SessionType::Tty,
        session_class: SessionClass::User,
        seat: "seat1".into(),
        vtnr: Some(2),
        display: None,
        remote_host: None,
        environment: BTreeMap::new(),
    };
    let status2 = UserSessionStatus {
        session_id: "sess_p2".into(),
        username: "user2".into(),
        uid: 1002,
        state: SessionState::Active,
        scope: SessionScope::Foreground,
        leader_pid: Some(4040), // Collision!
        created_at: "2026-09-16T00:00:00Z".into(),
        last_active_at: "2026-09-16T00:00:00Z".into(),
        idle_seconds: 0,
        locked: false,
    };

    store.specs.insert("sess_p1".into(), spec1);
    store.sessions.insert("sess_p1".into(), status1);
    store.specs.insert("sess_p2".into(), spec2);
    store.sessions.insert("sess_p2".into(), status2);

    let report = validate_session_store(&store, Path::new("test.json"));
    assert!(!report.healthy);
    assert!(report.errors.iter().any(|e| e.contains("duplicate leader_pid 4040")));
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_capacity_boundary_limit() {
    let mut store = UserSessionStore::new();

    for i in 0..(MAX_STORE_CAPACITY + 5) {
        let sid = format!("sess_{}", i);
        store.specs.insert(sid.clone(), UserSessionSpec {
            session_id: sid.clone(),
            username: "appuser".into(),
            uid: 1000,
            gid: 1000,
            session_type: SessionType::Tty,
            session_class: SessionClass::User,
            seat: format!("seat{}", i),
            vtnr: Some(1),
            display: None,
            remote_host: None,
            environment: BTreeMap::new(),
        });
        store.sessions.insert(sid.clone(), UserSessionStatus {
            session_id: sid.clone(),
            username: "appuser".into(),
            uid: 1000,
            state: SessionState::Terminated, // Terminated so no leader_pid collisions
            scope: SessionScope::Background,
            leader_pid: None,
            created_at: "2026-09-16T00:00:00Z".into(),
            last_active_at: "2026-09-16T00:00:00Z".into(),
            idle_seconds: 0,
            locked: false,
        });
    }

    let report = validate_session_store(&store, Path::new("sessions.json"));
    assert!(!report.healthy);
    assert!(report.errors.iter().any(|e| e.contains("exceeds maximum capacity of 10000 sessions")));
}

#[test]
fn test_non_destructive_corruption_recovery_and_quarantine() {
    let temp_dir = std::env::temp_dir().join(format!("aios_session_rec_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);
    let store_path = temp_dir.join("sessions.json");

    // 1. Write corrupted payload
    fs::write(&store_path, b"--- CORRUPTED SESSION STORE PAYLOAD ---").unwrap();

    // 2. Perform recovery
    let (recovered_store, backup_path) = recover_session_store_with_backup(&store_path);

    // 3. Verify backup file exists and contains the corrupted payload
    assert!(backup_path.exists(), "quarantine backup file must exist on disk");
    assert!(backup_path.to_string_lossy().contains(".bak."));
    let bak_content = fs::read(&backup_path).unwrap();
    assert_eq!(bak_content, b"--- CORRUPTED SESSION STORE PAYLOAD ---");

    // 4. Verify recovered store is written and valid
    assert!(store_path.exists());
    let report = validate_session_store(&recovered_store, &store_path);
    assert!(report.healthy, "recovered store must be completely healthy");
    assert_eq!(report.invalid_sessions, 0);
    assert!(report.valid_sessions >= 1);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_load_or_recover_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!("aios_session_lifecycle_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);
    let store_path = temp_dir.join("sessions.json");

    // Phase 1: Store file does not exist -> creates default store
    let (s1, rep1, rec1, bak1) = load_or_recover(&store_path).unwrap();
    assert!(rec1, "must flag that creation occurred");
    assert!(bak1.is_none(), "no backup needed when initial file was absent");
    assert!(rep1.healthy);
    assert!(store_path.exists());
    assert_eq!(s1.store.specs.len(), rep1.total_sessions);

    // Phase 2: File exists and is healthy -> loads existing without recovery
    let (s2, rep2, rec2, bak2) = load_or_recover(&store_path).unwrap();
    assert!(!rec2, "healthy store must not trigger recovery");
    assert!(bak2.is_none());
    assert!(rep2.healthy);
    assert_eq!(s2.store.specs.len(), s1.store.specs.len());

    // Phase 3: Corrupted JSON -> triggers quarantine and recovery
    fs::write(&store_path, b"{ \"truncated\": ").unwrap();
    let (_s3, rep3, rec3, bak3) = load_or_recover(&store_path).unwrap();
    assert!(rec3, "damaged store must trigger recovery");
    assert!(bak3.is_some(), "damaged store must generate quarantine backup");
    assert!(rep3.healthy);
    assert!(bak3.unwrap().exists());

    // Phase 4: Semantically corrupt store -> triggers quarantine and recovery
    let mut bad_store = UserSessionStore::new();
    bad_store.specs.insert("corrupt_sess".into(), UserSessionSpec {
        session_id: "corrupt_sess".into(),
        username: "INVALID USER NAME".into(),
        uid: 1000,
        gid: 1000,
        session_type: SessionType::Tty,
        session_class: SessionClass::User,
        seat: "seat0".into(),
        vtnr: Some(1),
        display: None,
        remote_host: None,
        environment: BTreeMap::new(),
    });
    bad_store.save_to_path(&store_path).unwrap();

    let (_s4, rep4, rec4, bak4) = load_or_recover(&store_path).unwrap();
    assert!(rec4, "semantically invalid store must trigger recovery");
    assert!(bak4.is_some(), "quarantine backup must be produced");
    assert!(rep4.healthy);
    assert!(bak4.unwrap().exists());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_quarantine_path_special_characters() {
    let temp_dir = std::env::temp_dir().join(format!("aios session test dir {}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);
    let store_path = temp_dir.join("sessions store file.json");

    fs::write(&store_path, b"malformed!").unwrap();
    let (s, rep, rec, bak) = load_or_recover(&store_path).unwrap();
    assert!(rec);
    assert!(rep.healthy);
    assert!(bak.is_some());
    assert!(store_path.exists());
    assert_eq!(s.store.specs.len(), rep.total_sessions);

    let _ = fs::remove_dir_all(&temp_dir);
}
