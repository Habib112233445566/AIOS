//! Automated Integration Test Suite for User Session Bootstrap Subsystem (T-01454 Implementation).
//!
//! Enforces criteria SBT1..SBT5:
//! - SBT1: Multi-Turn Lifecycle FSM Cohesion & Invalid Transition Rejection
//! - SBT2: Seat Arbitration & Automatic Foreground Demotion
//! - SBT3: User Quotas & Store Capacity Enforcement
//! - SBT4: Multi-Turn Disk Persistence & Recovery
//! - SBT5: Multi-Parameter Query Introspection

use std::collections::BTreeMap;
use aiosh_core::session::*;
use aiosh_core::session_service::*;

/// Helper for constructing synthetic test sessions with specified parameters.
pub fn create_test_session_spec(
    session_id: &str,
    username: &str,
    uid: u32,
    gid: u32,
    session_type: SessionType,
    session_class: SessionClass,
    seat: &str,
) -> UserSessionSpec {
    UserSessionSpec {
        session_id: session_id.to_string(),
        username: username.to_string(),
        uid,
        gid,
        session_type,
        session_class,
        seat: seat.to_string(),
        vtnr: Some(1),
        display: Some(":0".to_string()),
        remote_host: None,
        environment: BTreeMap::new(),
    }
}

#[test]
fn test_sbt1_lifecycle_fsm_cohesion() {
    let mut service = UserSessionService::empty();
    let spec = create_test_session_spec("sbt1-user", "kali", 1000, 1000, SessionType::Wayland, SessionClass::User, "seat0");

    // 1. Create -> Initializing
    let rep_create = service.create_session(spec).expect("create session");
    assert_eq!(rep_create.new_state, SessionState::Initializing);
    assert_eq!(service.get_session("sbt1-user").unwrap().state, SessionState::Initializing);

    // 2. Reject illegal transition (cannot lock while initializing)
    let err_lock_init = service.apply_action("sbt1-user", UserSessionAction::Lock);
    assert!(err_lock_init.is_err(), "cannot lock unauthenticated session");

    // 3. Authenticate -> Authenticating
    let rep_auth = service.apply_action("sbt1-user", UserSessionAction::Authenticate).expect("authenticate");
    assert_eq!(rep_auth.new_state, SessionState::Authenticating);

    // 4. Activate -> Active (Foreground)
    let rep_act = service.apply_action("sbt1-user", UserSessionAction::Activate).expect("activate");
    assert_eq!(rep_act.new_state, SessionState::Active);
    assert_eq!(service.get_session("sbt1-user").unwrap().scope, SessionScope::Foreground);
    assert!(!service.get_session("sbt1-user").unwrap().locked);

    // 5. Lock -> Locked
    let rep_lock = service.apply_action("sbt1-user", UserSessionAction::Lock).expect("lock");
    assert_eq!(rep_lock.new_state, SessionState::Locked);
    assert!(service.get_session("sbt1-user").unwrap().locked);

    // 6. Unlock -> Active
    let rep_unlock = service.apply_action("sbt1-user", UserSessionAction::Unlock).expect("unlock");
    assert_eq!(rep_unlock.new_state, SessionState::Active);
    assert!(!service.get_session("sbt1-user").unwrap().locked);

    // 7. Terminate -> Terminating -> Terminated
    let rep_term = service.apply_action("sbt1-user", UserSessionAction::Terminate).expect("terminate");
    assert_eq!(rep_term.new_state, SessionState::Terminating);

    let rep_final = service.apply_action("sbt1-user", UserSessionAction::Terminate).expect("final terminate");
    assert_eq!(rep_final.new_state, SessionState::Terminated);

    // 8. Terminal state is immutable
    let err_post_term = service.apply_action("sbt1-user", UserSessionAction::Activate);
    assert!(err_post_term.is_err(), "cannot reactivate terminated session");
}

#[test]
fn test_sbt2_seat_arbitration_and_demotion() {
    let mut service = UserSessionService::empty();

    // Session 1 on seat0
    let s1 = create_test_session_spec("sbt2-sess-1", "user1", 1001, 1001, SessionType::Wayland, SessionClass::User, "seat0");
    service.create_session(s1).expect("create s1");
    service.apply_action("sbt2-sess-1", UserSessionAction::Authenticate).unwrap();
    service.apply_action("sbt2-sess-1", UserSessionAction::Activate).unwrap();

    // Verify Session 1 is Foreground on seat0
    assert_eq!(service.get_session("sbt2-sess-1").unwrap().scope, SessionScope::Foreground);

    // Session 2 also on seat0
    let s2 = create_test_session_spec("sbt2-sess-2", "user2", 1002, 1002, SessionType::X11, SessionClass::User, "seat0");
    service.create_session(s2).expect("create s2");
    service.apply_action("sbt2-sess-2", UserSessionAction::Authenticate).unwrap();
    service.apply_action("sbt2-sess-2", UserSessionAction::Activate).unwrap();

    // Session 2 is now Foreground, Session 1 demoted to Background
    assert_eq!(service.get_session("sbt2-sess-2").unwrap().scope, SessionScope::Foreground);
    assert_eq!(service.get_session("sbt2-sess-1").unwrap().scope, SessionScope::Background);

    // Session 3 on seat1 (independent seat)
    let s3 = create_test_session_spec("sbt2-sess-3", "user3", 1003, 1003, SessionType::Tty, SessionClass::User, "seat1");
    service.create_session(s3).expect("create s3");
    service.apply_action("sbt2-sess-3", UserSessionAction::Authenticate).unwrap();
    service.apply_action("sbt2-sess-3", UserSessionAction::Activate).unwrap();

    // Session 3 is Foreground on seat1, without demoting Session 2 on seat0
    assert_eq!(service.get_session("sbt2-sess-3").unwrap().scope, SessionScope::Foreground);
    assert_eq!(service.get_session("sbt2-sess-2").unwrap().scope, SessionScope::Foreground);
}

#[test]
fn test_sbt3_capacity_quotas() {
    let mut service = UserSessionService::empty();

    // Create 32 active sessions for "quota-user"
    for i in 1..=32 {
        let sid = format!("quota-sess-{:02}", i);
        let spec = create_test_session_spec(&sid, "quota-user", 2000, 2000, SessionType::AiAgent, SessionClass::Agent, "seat0");
        service.create_session(spec).expect("create session within quota");
        service.apply_action(&sid, UserSessionAction::Authenticate).unwrap();
        service.apply_action(&sid, UserSessionAction::Activate).unwrap();
    }

    // 33rd session creation must fail due to MAX_SESSIONS_PER_USER quota
    let overflow_spec = create_test_session_spec("quota-sess-33", "quota-user", 2000, 2000, SessionType::AiAgent, SessionClass::Agent, "seat0");
    let err = service.create_session(overflow_spec);
    assert!(err.is_err(), "33rd active session must violate user quota");
    assert!(err.unwrap_err().contains("maximum of 32 active sessions"));
}

#[test]
fn test_sbt4_store_persistence() {
    let temp_dir = std::env::temp_dir();
    let store_path = temp_dir.join(format!("aios_sbt4_persist_{}.json", std::process::id()));

    let mut service = UserSessionService::empty();
    let spec1 = create_test_session_spec("p-sess-1", "alice", 1001, 1001, SessionType::Wayland, SessionClass::User, "seat0");
    let spec2 = create_test_session_spec("p-sess-2", "bob", 1002, 1002, SessionType::AiAgent, SessionClass::Agent, "seat0");

    service.create_session(spec1).expect("create p-sess-1");
    service.create_session(spec2).expect("create p-sess-2");
    service.apply_action("p-sess-1", UserSessionAction::Authenticate).unwrap();
    service.apply_action("p-sess-1", UserSessionAction::Activate).unwrap();
    service.apply_action("p-sess-1", UserSessionAction::Lock).unwrap();

    // Save to disk
    service.save_to_path(&store_path).expect("save store to disk");

    // Load back from disk
    let loaded = UserSessionService::load_from_path(&store_path).expect("load store from disk");

    // Assert parity
    assert_eq!(loaded.list_sessions().len(), 2);
    let s1 = loaded.get_session("p-sess-1").expect("p-sess-1 exists");
    assert_eq!(s1.username, "alice");
    assert_eq!(s1.state, SessionState::Locked);
    assert!(s1.locked);

    let s2 = loaded.get_session("p-sess-2").expect("p-sess-2 exists");
    assert_eq!(s2.username, "bob");
    assert_eq!(s2.state, SessionState::Initializing);

    let _ = std::fs::remove_file(&store_path);
}

#[test]
fn test_sbt5_catalog_introspection() {
    let mut service = UserSessionService::empty();

    let specs = vec![
        create_test_session_spec("q1", "alice", 1001, 1001, SessionType::Wayland, SessionClass::User, "seat0"),
        create_test_session_spec("q2", "alice", 1001, 1001, SessionType::Tty, SessionClass::User, "seat0"),
        create_test_session_spec("q3", "bob", 1002, 1002, SessionType::Wayland, SessionClass::User, "seat1"),
        create_test_session_spec("q4", "agent1", 1003, 1003, SessionType::AiAgent, SessionClass::Agent, "seat0"),
    ];

    for sp in specs {
        service.create_session(sp).unwrap();
    }

    // Filter by username
    let query_alice = UserSessionQuery {
        username: Some("alice".to_string()),
        ..Default::default()
    };
    assert_eq!(service.query_sessions(&query_alice).len(), 2);

    // Filter by session_type
    let query_agent = UserSessionQuery {
        session_type: Some(SessionType::AiAgent),
        ..Default::default()
    };
    assert_eq!(service.query_sessions(&query_agent).len(), 1);

    // Filter by seat
    let query_seat1 = UserSessionQuery {
        seat: Some("seat1".to_string()),
        ..Default::default()
    };
    assert_eq!(service.query_sessions(&query_seat1).len(), 1);

    // Filter with limit
    let query_limit = UserSessionQuery {
        limit: Some(2),
        ..Default::default()
    };
    assert_eq!(service.query_sessions(&query_limit).len(), 2);
}
