//! Standalone unit test suite for User Session Bootstrap Core Service (CS1..CS5).

use aiosh_core::session::{
    SessionClass, SessionScope, SessionState, SessionType, UserSessionAction, UserSessionQuery,
    UserSessionSpec, MAX_SESSIONS_PER_USER,
};
use aiosh_core::session_service::UserSessionService;
use std::collections::BTreeMap;

fn sample_spec(id: &str, user: &str, seat: &str, stype: SessionType) -> UserSessionSpec {
    UserSessionSpec {
        session_id: id.to_string(),
        username: user.to_string(),
        uid: 1000,
        gid: 1000,
        session_type: stype,
        session_class: SessionClass::User,
        seat: seat.to_string(),
        vtnr: if stype == SessionType::Tty {
            Some(1)
        } else {
            Some(7)
        },
        display: if stype == SessionType::X11 {
            Some(":0".to_string())
        } else {
            None
        },
        remote_host: None,
        environment: BTreeMap::new(),
    }
}

#[test]
fn test_cs1_canonical_seeding_and_empty() {
    let service = UserSessionService::new();
    assert_eq!(service.store.sessions.len(), 1);

    let greeter = service
        .get_session("greeter-seat0")
        .expect("Greeter session must exist");
    assert_eq!(greeter.username, "lightdm");
    assert_eq!(greeter.state, SessionState::Active);
    assert_eq!(greeter.scope, SessionScope::Foreground);
    assert_eq!(greeter.leader_pid, Some(1001));
    assert!(!greeter.locked);

    let empty = UserSessionService::empty();
    assert_eq!(empty.store.sessions.len(), 0);
    assert!(empty.get_session("greeter-seat0").is_none());
}

#[test]
fn test_cs1_session_lifecycle_state_machine() {
    let mut service = UserSessionService::empty();
    let spec = sample_spec("sess-test-01", "kali", "seat0", SessionType::X11);

    // 1. Create -> Initializing
    let rep_create = service.create_session(spec).expect("Create session");
    assert_eq!(rep_create.session_id, "sess-test-01");
    assert_eq!(rep_create.action, UserSessionAction::Create);
    assert_eq!(rep_create.previous_state, SessionState::Initializing);
    assert_eq!(rep_create.new_state, SessionState::Initializing);
    assert!(rep_create.success);
    assert!(rep_create.error.is_none());

    // 2. Authenticate -> Authenticating
    let rep_auth = service
        .apply_action("sess-test-01", UserSessionAction::Authenticate)
        .expect("Authenticate");
    assert_eq!(rep_auth.previous_state, SessionState::Initializing);
    assert_eq!(rep_auth.new_state, SessionState::Authenticating);

    // 3. Activate -> Active (takes foreground)
    let rep_act = service
        .apply_action("sess-test-01", UserSessionAction::Activate)
        .expect("Activate");
    assert_eq!(rep_act.previous_state, SessionState::Authenticating);
    assert_eq!(rep_act.new_state, SessionState::Active);
    let st = service.get_session("sess-test-01").unwrap();
    assert_eq!(st.scope, SessionScope::Foreground);
    assert!(!st.locked);

    // 4. Lock -> Locked
    let rep_lock = service
        .apply_action("sess-test-01", UserSessionAction::Lock)
        .expect("Lock");
    assert_eq!(rep_lock.previous_state, SessionState::Active);
    assert_eq!(rep_lock.new_state, SessionState::Locked);
    let st = service.get_session("sess-test-01").unwrap();
    assert!(st.locked);

    // 5. Unlock -> Active
    let rep_unlock = service
        .apply_action("sess-test-01", UserSessionAction::Unlock)
        .expect("Unlock");
    assert_eq!(rep_unlock.previous_state, SessionState::Locked);
    assert_eq!(rep_unlock.new_state, SessionState::Active);
    let st = service.get_session("sess-test-01").unwrap();
    assert!(!st.locked);

    // 6. Terminate -> Terminating
    let rep_term1 = service
        .apply_action("sess-test-01", UserSessionAction::Terminate)
        .expect("Terminate step 1");
    assert_eq!(rep_term1.previous_state, SessionState::Active);
    assert_eq!(rep_term1.new_state, SessionState::Terminating);

    // 7. Terminate -> Terminated
    let rep_term2 = service
        .apply_action("sess-test-01", UserSessionAction::Terminate)
        .expect("Terminate step 2");
    assert_eq!(rep_term2.previous_state, SessionState::Terminating);
    assert_eq!(rep_term2.new_state, SessionState::Terminated);
    let st = service.get_session("sess-test-01").unwrap();
    assert_eq!(st.scope, SessionScope::Background);
    assert!(!st.locked);
}

#[test]
fn test_cs1_negative_transitions_and_error_handling() {
    let mut service = UserSessionService::empty();
    let spec = sample_spec("sess-neg", "kali", "seat0", SessionType::X11);
    service.create_session(spec).unwrap();

    // Cannot lock Initializing
    let err = service.apply_action("sess-neg", UserSessionAction::Lock);
    assert!(err.is_err());

    // Cannot unlock Initializing
    let err = service.apply_action("sess-neg", UserSessionAction::Unlock);
    assert!(err.is_err());

    // Non-existent session
    let err = service.apply_action("no-such-session", UserSessionAction::Activate);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("not found"));

    // Move to terminated
    service
        .apply_action("sess-neg", UserSessionAction::Terminate)
        .unwrap();
    assert_eq!(
        service.get_session("sess-neg").unwrap().state,
        SessionState::Terminated
    );

    // Any action on Terminated must fail
    assert!(service
        .apply_action("sess-neg", UserSessionAction::Activate)
        .is_err());
    assert!(service
        .apply_action("sess-neg", UserSessionAction::Authenticate)
        .is_err());
    assert!(service
        .apply_action("sess-neg", UserSessionAction::Lock)
        .is_err());
    assert!(service
        .apply_action("sess-neg", UserSessionAction::Unlock)
        .is_err());
    assert!(service
        .apply_action("sess-neg", UserSessionAction::Terminate)
        .is_err());
}

#[test]
fn test_cs2_seat_arbitration_and_foreground_uniqueness() {
    let mut service = UserSessionService::empty();

    let spec_a = sample_spec("sess-a", "kali", "seat0", SessionType::X11);
    let spec_b = sample_spec("sess-b", "alice", "seat0", SessionType::X11);
    let spec_c = sample_spec("sess-c", "bob", "seat0", SessionType::X11);
    let spec_remote = sample_spec("sess-remote", "remoteuser", "seat1", SessionType::Tty);

    service.create_session(spec_a).unwrap();
    service.create_session(spec_b).unwrap();
    service.create_session(spec_c).unwrap();
    service.create_session(spec_remote).unwrap();

    // Authenticate all
    for id in &["sess-a", "sess-b", "sess-c", "sess-remote"] {
        service
            .apply_action(id, UserSessionAction::Authenticate)
            .unwrap();
    }

    // Activate A on seat0 -> A is Foreground
    service
        .apply_action("sess-a", UserSessionAction::Activate)
        .unwrap();
    assert_eq!(
        service.get_session("sess-a").unwrap().scope,
        SessionScope::Foreground
    );
    assert_eq!(
        service.get_session("sess-b").unwrap().scope,
        SessionScope::Background
    );
    assert_eq!(
        service.get_session("sess-c").unwrap().scope,
        SessionScope::Background
    );

    // Activate B on seat0 -> B becomes Foreground, A demoted to Background
    service
        .apply_action("sess-b", UserSessionAction::Activate)
        .unwrap();
    assert_eq!(
        service.get_session("sess-a").unwrap().scope,
        SessionScope::Background
    );
    assert_eq!(
        service.get_session("sess-b").unwrap().scope,
        SessionScope::Foreground
    );
    assert_eq!(
        service.get_session("sess-c").unwrap().scope,
        SessionScope::Background
    );

    // Activate C on seat0 -> C becomes Foreground, B demoted to Background
    service
        .apply_action("sess-c", UserSessionAction::Activate)
        .unwrap();
    assert_eq!(
        service.get_session("sess-a").unwrap().scope,
        SessionScope::Background
    );
    assert_eq!(
        service.get_session("sess-b").unwrap().scope,
        SessionScope::Background
    );
    assert_eq!(
        service.get_session("sess-c").unwrap().scope,
        SessionScope::Foreground
    );

    // Activate remote session on seat1 -> Independent from seat0
    service
        .apply_action("sess-remote", UserSessionAction::Activate)
        .unwrap();
    assert_eq!(
        service.get_session("sess-remote").unwrap().scope,
        SessionScope::Foreground
    );
    assert_eq!(
        service.get_session("sess-c").unwrap().scope,
        SessionScope::Foreground
    );
}

#[test]
fn test_cs3_capacity_limits_and_user_boundaries() {
    let mut service = UserSessionService::empty();

    // Create 32 active sessions for 'kali'
    for i in 0..MAX_SESSIONS_PER_USER {
        let id = format!("sess-cap-{}", i);
        let mut sp = sample_spec(&id, "kali", "seat0", SessionType::X11);
        sp.vtnr = Some((i % 12 + 1) as u32);
        service.create_session(sp).unwrap();
    }

    // 33rd active session must fail
    let sp_33 = sample_spec("sess-cap-33", "kali", "seat0", SessionType::X11);
    let err = service.create_session(sp_33);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("maximum of 32 active sessions"));

    // Terminate one session
    service
        .apply_action("sess-cap-0", UserSessionAction::Terminate)
        .unwrap();
    assert_eq!(
        service.get_session("sess-cap-0").unwrap().state,
        SessionState::Terminated
    );

    // Now 'kali' has 31 active sessions, so creating another must succeed
    let sp_new = sample_spec("sess-cap-new", "kali", "seat0", SessionType::X11);
    assert!(service.create_session(sp_new).is_ok());
}

#[test]
fn test_cs4_action_reporting_and_envelope_integrity() {
    let mut service = UserSessionService::empty();
    let spec = sample_spec("sess-rep", "kali", "seat0", SessionType::X11);
    let rep = service.create_session(spec).unwrap();

    assert_eq!(rep.session_id, "sess-rep");
    assert_eq!(rep.action, UserSessionAction::Create);
    assert!(rep.success);
    assert!(rep.error.is_none());
    assert!(!rep.timestamp.is_empty());

    let json = serde_json::to_string(&rep).unwrap();
    assert!(json.contains("\"session_id\":\"sess-rep\""));
    assert!(json.contains("\"action\":\"create\""));
}

#[test]
fn test_cs5_idle_tracking_and_activity_resets() {
    let mut service = UserSessionService::empty();
    let spec = sample_spec("sess-idle", "kali", "seat0", SessionType::X11);
    service.create_session(spec).unwrap();

    assert_eq!(service.get_session("sess-idle").unwrap().idle_seconds, 0);

    // Update idle to 120 seconds
    service.update_idle("sess-idle", 120).unwrap();
    assert_eq!(service.get_session("sess-idle").unwrap().idle_seconds, 120);

    // Touch activity resets to 0
    service.touch_activity("sess-idle").unwrap();
    assert_eq!(service.get_session("sess-idle").unwrap().idle_seconds, 0);

    // Authenticate and activate
    service
        .apply_action("sess-idle", UserSessionAction::Authenticate)
        .unwrap();
    service.update_idle("sess-idle", 45).unwrap();
    assert_eq!(service.get_session("sess-idle").unwrap().idle_seconds, 45);

    // Activate resets idle
    service
        .apply_action("sess-idle", UserSessionAction::Activate)
        .unwrap();
    assert_eq!(service.get_session("sess-idle").unwrap().idle_seconds, 0);

    // Lock session
    service
        .apply_action("sess-idle", UserSessionAction::Lock)
        .unwrap();
    service.update_idle("sess-idle", 600).unwrap();
    assert_eq!(service.get_session("sess-idle").unwrap().idle_seconds, 600);

    // Unlock resets idle
    service
        .apply_action("sess-idle", UserSessionAction::Unlock)
        .unwrap();
    assert_eq!(service.get_session("sess-idle").unwrap().idle_seconds, 0);
}

#[test]
fn test_session_query_and_atomic_persistence() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let store_path = temp_dir.path().join("sessions.json");

    let mut service = UserSessionService::empty();
    let spec1 = sample_spec("sess-q1", "kali", "seat0", SessionType::X11);
    let spec2 = sample_spec("sess-q2", "alice", "seat1", SessionType::Tty);

    service.create_session(spec1).unwrap();
    service.create_session(spec2).unwrap();

    // Query tests
    let q_user = UserSessionQuery {
        username: Some("alice".to_string()),
        ..Default::default()
    };
    let res = service.query_sessions(&q_user);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].session_id, "sess-q2");

    let q_all = UserSessionQuery::default();
    let res = service.query_sessions(&q_all);
    assert_eq!(res.len(), 2);

    // Atomic save & load
    service.save_to_path(&store_path).expect("save store");
    assert!(store_path.exists());

    let loaded = UserSessionService::load_from_path(&store_path).expect("load store");
    assert_eq!(loaded.store.sessions.len(), 2);
    assert!(loaded.get_session("sess-q1").is_some());
    assert!(loaded.get_session("sess-q2").is_some());
}

#[test]
fn test_hardening_atomic_save_and_error_cleanup() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let store_path = temp_dir.path().join("hardened_sessions.json");

    let mut service = UserSessionService::empty();
    let spec = sample_spec("sess-h1", "kali", "seat0", SessionType::X11);
    service.create_session(spec).unwrap();

    // 1. Valid save cleans up temporary file
    service.save_to_path(&store_path).expect("save must succeed");
    assert!(store_path.exists());

    // Check no temp files leaked in directory
    let entries = std::fs::read_dir(temp_dir.path()).expect("read dir");
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        files.push(entry.file_name().to_string_lossy().to_string());
    }
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], "hardened_sessions.json");

    // 2. Corrupted JSON file load fails with explicit error
    let corrupt_path = temp_dir.path().join("corrupted.json");
    std::fs::write(&corrupt_path, b"{ invalid json ...").expect("write corrupt");
    let load_err = UserSessionService::load_from_path(&corrupt_path);
    assert!(load_err.is_err());
    let err_str = load_err.unwrap_err().to_string();
    assert!(err_str.contains("Deserialization error"));

    // 3. Oversized file (> 10 MiB) load rejected
    let oversize_path = temp_dir.path().join("oversized.json");
    let big_file = std::fs::File::create(&oversize_path).expect("create big file");
    big_file.set_len(11 * 1024 * 1024).expect("set len to 11 MiB");
    drop(big_file);
    let load_big_err = UserSessionService::load_from_path(&oversize_path);
    assert!(load_big_err.is_err());
    let err_msg = load_big_err.unwrap_err().to_string();
    assert!(err_msg.contains("10 MiB limit"));
}

#[test]
fn test_hardening_session_service_explicit_error_envelopes() {
    let mut service = UserSessionService::empty();
    let spec = sample_spec("sess-err", "kali", "seat0", SessionType::X11);
    service.create_session(spec).unwrap();

    // Explicit error on nonexistent session
    let res_unknown = service.apply_action("ghost-session", UserSessionAction::Activate);
    assert!(res_unknown.is_err());
    assert_eq!(res_unknown.unwrap_err(), "Session 'ghost-session' not found in store");

    // Explicit error on invalid state transition (Initializing -> Lock)
    let res_invalid = service.apply_action("sess-err", UserSessionAction::Lock);
    assert!(res_invalid.is_err());
    assert!(res_invalid.unwrap_err().contains("Invalid session state transition"));

    // Explicit error on update_idle with unknown session
    let res_idle = service.update_idle("ghost-session", 100);
    assert!(res_idle.is_err());
    assert_eq!(res_idle.unwrap_err(), "Session 'ghost-session' not found in store");

    // Explicit error on touch_activity with unknown session
    let res_touch = service.touch_activity("ghost-session");
    assert!(res_touch.is_err());
    assert_eq!(res_touch.unwrap_err(), "Session 'ghost-session' not found in store");
}

