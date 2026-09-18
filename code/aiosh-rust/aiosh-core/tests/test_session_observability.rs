//! Comprehensive integration tests for User Session Observability (SSO1..SSO6).

use aiosh_core::session::{
    SessionClass, SessionScope, SessionState, SessionType, UserSessionSpec, UserSessionStatus,
    UserSessionStore,
};
use aiosh_core::session_observability::SessionObservabilityReport;
use aiosh_core::session_policy::{SessionPolicyMode, UserSessionSecurityPolicy};

fn make_spec(id: &str, user: &str, uid: u32, seat: &str, st: SessionType, sc: SessionClass) -> UserSessionSpec {
    UserSessionSpec {
        session_id: id.to_string(),
        username: user.to_string(),
        uid,
        gid: uid,
        session_type: st,
        session_class: sc,
        seat: seat.to_string(),
        vtnr: Some(1),
        display: Some(":0".to_string()),
        remote_host: None,
        environment: [("DISPLAY".to_string(), ":0".to_string())].into_iter().collect(),
    }
}

fn make_status(id: &str, user: &str, uid: u32, state: SessionState, scope: SessionScope, locked: bool, idle: u64) -> UserSessionStatus {
    UserSessionStatus {
        session_id: id.to_string(),
        username: user.to_string(),
        uid,
        state,
        scope,
        leader_pid: Some(1234),
        created_at: "2026-09-11T12:00:00Z".to_string(),
        last_active_at: "2026-09-11T12:05:00Z".to_string(),
        idle_seconds: idle,
        locked,
    }
}

#[test]
fn test_sso1_state_and_class_distribution() {
    let mut store = UserSessionStore::new();
    let spec1 = make_spec("sess-1", "alice", 1001, "seat0", SessionType::Wayland, SessionClass::User);
    let stat1 = make_status("sess-1", "alice", 1001, SessionState::Active, SessionScope::Foreground, false, 0);
    store.add_session(spec1, stat1).unwrap();

    let spec2 = make_spec("sess-2", "greeter", 105, "seat0", SessionType::Wayland, SessionClass::Greeter);
    let stat2 = make_status("sess-2", "greeter", 105, SessionState::Locked, SessionScope::Foreground, true, 45);
    store.add_session(spec2, stat2).unwrap();

    let spec3 = make_spec("sess-3", "agent-bob", 1002, "seat1", SessionType::AiAgent, SessionClass::Agent);
    let stat3 = make_status("sess-3", "agent-bob", 1002, SessionState::Initializing, SessionScope::Background, false, 0);
    store.add_session(spec3, stat3).unwrap();

    let report = SessionObservabilityReport::generate(&store, None);

    assert_eq!(report.total_sessions, 3);
    assert_eq!(*report.state_breakdown.get("active").unwrap(), 1);
    assert_eq!(*report.state_breakdown.get("locked").unwrap(), 1);
    assert_eq!(*report.state_breakdown.get("initializing").unwrap(), 1);
    assert_eq!(*report.state_breakdown.get("terminated").unwrap(), 0);

    assert_eq!(*report.session_class_breakdown.get("user").unwrap(), 1);
    assert_eq!(*report.session_class_breakdown.get("greeter").unwrap(), 1);
    assert_eq!(*report.session_class_breakdown.get("agent").unwrap(), 1);
    assert_eq!(*report.session_class_breakdown.get("background").unwrap(), 0);

    assert_eq!(*report.session_type_breakdown.get("wayland").unwrap(), 2);
    assert_eq!(*report.session_type_breakdown.get("ai_agent").unwrap(), 1);
    assert_eq!(*report.session_type_breakdown.get("tty").unwrap(), 0);
}

#[test]
fn test_sso2_seat_and_scope_arbitration() {
    let mut store = UserSessionStore::new();
    let spec1 = make_spec("sess-1", "alice", 1001, "seat0", SessionType::Wayland, SessionClass::User);
    let stat1 = make_status("sess-1", "alice", 1001, SessionState::Active, SessionScope::Foreground, false, 0);
    store.add_session(spec1, stat1).unwrap();

    let spec2 = make_spec("sess-2", "charlie", 1003, "seat0", SessionType::X11, SessionClass::User);
    let stat2 = make_status("sess-2", "charlie", 1003, SessionState::Active, SessionScope::Background, false, 10);
    store.add_session(spec2, stat2).unwrap();

    let spec3 = make_spec("sess-3", "david", 1004, "seat-usb-1", SessionType::Tty, SessionClass::User);
    let stat3 = make_status("sess-3", "david", 1004, SessionState::Active, SessionScope::Foreground, false, 0);
    store.add_session(spec3, stat3).unwrap();

    let report = SessionObservabilityReport::generate(&store, None);

    assert_eq!(*report.seat_breakdown.get("seat0").unwrap(), 2);
    assert_eq!(*report.seat_breakdown.get("seat-usb-1").unwrap(), 1);
    assert_eq!(*report.scope_breakdown.get("foreground").unwrap(), 2);
    assert_eq!(*report.scope_breakdown.get("background").unwrap(), 1);
}

#[test]
fn test_sso3_idle_time_tracking() {
    let mut store = UserSessionStore::new();
    let spec1 = make_spec("sess-1", "alice", 1001, "seat0", SessionType::Tty, SessionClass::User);
    let stat1 = make_status("sess-1", "alice", 1001, SessionState::Active, SessionScope::Foreground, false, 0);
    store.add_session(spec1, stat1).unwrap();

    let spec2 = make_spec("sess-2", "bob", 1002, "seat0", SessionType::Wayland, SessionClass::User);
    let stat2 = make_status("sess-2", "bob", 1002, SessionState::Locked, SessionScope::Background, true, 120);
    store.add_session(spec2, stat2).unwrap();

    let spec3 = make_spec("sess-3", "carol", 1005, "seat1", SessionType::X11, SessionClass::User);
    let stat3 = make_status("sess-3", "carol", 1005, SessionState::Active, SessionScope::Foreground, false, 360);
    store.add_session(spec3, stat3).unwrap();

    let report = SessionObservabilityReport::generate(&store, None);

    assert_eq!(report.locked_count, 1);
    assert_eq!(report.idle_sessions_count, 2);
    assert_eq!(report.max_idle_seconds, 360);
    assert_eq!(report.total_idle_seconds, 480);
}

#[test]
fn test_sso4_user_concurrency_breakdown() {
    let mut store = UserSessionStore::new();
    let spec1 = make_spec("sess-1", "alice", 1001, "seat0", SessionType::Wayland, SessionClass::User);
    let stat1 = make_status("sess-1", "alice", 1001, SessionState::Active, SessionScope::Foreground, false, 0);
    store.add_session(spec1, stat1).unwrap();

    let spec2 = make_spec("sess-2", "alice", 1001, "seat0", SessionType::Tty, SessionClass::User);
    let stat2 = make_status("sess-2", "alice", 1001, SessionState::Active, SessionScope::Background, false, 15);
    store.add_session(spec2, stat2).unwrap();

    let spec3 = make_spec("sess-3", "bob", 1002, "seat1", SessionType::Wayland, SessionClass::User);
    let stat3 = make_status("sess-3", "bob", 1002, SessionState::Active, SessionScope::Foreground, false, 5);
    store.add_session(spec3, stat3).unwrap();

    let report = SessionObservabilityReport::generate(&store, None);

    assert_eq!(report.distinct_users_count, 2);
    assert_eq!(*report.user_breakdown.get("alice").unwrap(), 2);
    assert_eq!(*report.user_breakdown.get("bob").unwrap(), 1);
}

#[test]
fn test_sso5_policy_compliance_evaluation() {
    let mut store = UserSessionStore::new();
    // Valid user session
    let spec1 = make_spec("sess-1", "alice", 1001, "seat0", SessionType::Wayland, SessionClass::User);
    let stat1 = make_status("sess-1", "alice", 1001, SessionState::Active, SessionScope::Foreground, false, 0);
    store.add_session(spec1, stat1).unwrap();

    // Violating session (root interactive session without allow_root)
    let mut spec2 = make_spec("sess-2", "root", 0, "seat0", SessionType::Wayland, SessionClass::User);
    spec2.environment.insert("LD_PRELOAD".to_string(), "/tmp/bad.so".to_string());
    let stat2 = make_status("sess-2", "root", 0, SessionState::Active, SessionScope::Foreground, false, 0);
    store.add_session(spec2, stat2).unwrap();

    let policy = UserSessionSecurityPolicy {
        mode: SessionPolicyMode::Enforcing,
        disallow_root: true,
        ..Default::default()
    };

    let report = SessionObservabilityReport::generate(&store, Some(&policy));

    assert_eq!(report.total_sessions, 2);
    assert_eq!(report.policy_compliant_count, 1);
    assert_eq!(report.policy_violations_count, 1);
    assert_eq!(report.violating_sessions, vec!["sess-2"]);
}

#[test]
fn test_sso6_canonical_serialization() {
    let mut store = UserSessionStore::new();
    let spec = make_spec("sess-1", "alice", 1001, "seat0", SessionType::Wayland, SessionClass::User);
    let stat = make_status("sess-1", "alice", 1001, SessionState::Active, SessionScope::Foreground, false, 0);
    store.add_session(spec, stat).unwrap();

    let report = SessionObservabilityReport::generate(&store, None);
    let json_str = serde_json::to_string(&report).expect("must serialize");

    assert!(json_str.contains("\"total_sessions\":1"));
    assert!(json_str.contains("\"distinct_users_count\":1"));
    assert!(json_str.contains("\"state_breakdown\""));
    assert!(json_str.contains("\"generated_at\""));

    let deserialized: SessionObservabilityReport = serde_json::from_str(&json_str).expect("must deserialize");
    assert_eq!(report, deserialized);
}
