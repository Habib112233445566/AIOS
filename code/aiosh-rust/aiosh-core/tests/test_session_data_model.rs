//! Automated Unit & Integration Tests for User Session Bootstrap Data Model (SB1..SB5)

use aiosh_core::session::{
    transition_session_state, validate_session_id, validate_user_session_spec,
    validate_user_session_status, validate_username, SessionClass, SessionScope, SessionState,
    SessionType, UserSessionAction, UserSessionQuery, UserSessionSpec, UserSessionStatus,
    UserSessionStore, MAX_SESSIONS_PER_USER, MAX_SESSION_STORE_SIZE,
};
use std::collections::BTreeMap;
use tempfile::tempdir;

fn sample_x11_spec() -> UserSessionSpec {
    let mut env = BTreeMap::new();
    env.insert("XDG_RUNTIME_DIR".into(), "/run/user/1000".into());
    env.insert("DISPLAY".into(), ":0".into());
    env.insert("XDG_SESSION_TYPE".into(), "x11".into());

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

fn sample_status() -> UserSessionStatus {
    UserSessionStatus {
        session_id: "sess-01".into(),
        username: "kali".into(),
        uid: 1000,
        state: SessionState::Active,
        scope: SessionScope::Foreground,
        leader_pid: Some(1042),
        created_at: "2026-09-09T12:00:00Z".into(),
        last_active_at: "2026-09-09T12:10:00Z".into(),
        idle_seconds: 0,
        locked: false,
    }
}

#[test]
fn test_sb1_session_id_boundary_and_syntax() {
    // Valid standard IDs
    assert!(validate_session_id("sess-01").is_ok());
    assert!(validate_session_id("c1").is_ok());
    assert!(validate_session_id("2").is_ok());
    assert!(validate_session_id("agent_session.42").is_ok());

    // Boundary: min length (1 char)
    assert!(validate_session_id("s").is_ok());
    assert!(validate_session_id("1").is_ok());

    // Boundary: max length (64 chars)
    let max_len_id = "a".repeat(64);
    assert!(validate_session_id(&max_len_id).is_ok());

    // Negative: oversized (65 chars)
    let over_len_id = "a".repeat(65);
    assert!(validate_session_id(&over_len_id).is_err());

    // Negative: empty
    assert!(validate_session_id("").is_err());

    // Negative: leading symbols
    assert!(validate_session_id("-sess").is_err());
    assert!(validate_session_id(".sess").is_err());
    assert!(validate_session_id("_sess").is_err());

    // Negative: whitespace
    assert!(validate_session_id(" sess").is_err());
    assert!(validate_session_id("sess ").is_err());
    assert!(validate_session_id("sess 01").is_err());
    assert!(validate_session_id("sess\t01").is_err());

    // Negative: path separators
    assert!(validate_session_id("sess/01").is_err());
    assert!(validate_session_id("sess\\01").is_err());

    // Negative: path traversal
    assert!(validate_session_id("..").is_err());
    assert!(validate_session_id("../evil").is_err());
    assert!(validate_session_id("sess..01").is_err());

    // Negative: shell metacharacters and null bytes
    assert!(validate_session_id("sess;rm").is_err());
    assert!(validate_session_id("sess&id").is_err());
    assert!(validate_session_id("sess|id").is_err());
    assert!(validate_session_id("sess>id").is_err());
    assert!(validate_session_id("sess<id").is_err());
    assert!(validate_session_id("sess$id").is_err());
    assert!(validate_session_id("sess\0id").is_err());
}

#[test]
fn test_sb2_username_and_identity_bounds() {
    // Valid standard usernames
    assert!(validate_username("kali").is_ok());
    assert!(validate_username("root").is_ok());
    assert!(validate_username("_apt").is_ok());
    assert!(validate_username("aios-agent").is_ok());
    assert!(validate_username("daemon").is_ok());

    // Boundary: min length (1 char)
    assert!(validate_username("a").is_ok());
    assert!(validate_username("_").is_ok());

    // Boundary: max length (32 chars)
    let max_len_user = format!("u{}", "a".repeat(31));
    assert_eq!(max_len_user.len(), 32);
    assert!(validate_username(&max_len_user).is_ok());

    // Negative: oversized (33 chars)
    let over_len_user = format!("u{}", "a".repeat(32));
    assert_eq!(over_len_user.len(), 33);
    assert!(validate_username(&over_len_user).is_err());

    // Negative: empty
    assert!(validate_username("").is_err());

    // Negative: uppercase (POSIX standard adherence)
    assert!(validate_username("Kali").is_err());
    assert!(validate_username("ROOT").is_err());

    // Negative: starts with digit
    assert!(validate_username("1user").is_err());

    // Negative: whitespace, slashes, metacharacters
    assert!(validate_username(" kali").is_err());
    assert!(validate_username("kali ").is_err());
    assert!(validate_username("kali/user").is_err());
    assert!(validate_username("user;evil").is_err());
    assert!(validate_username("user\0evil").is_err());
}

#[test]
fn test_sb3_lifecycle_state_machine_matrix() {
    // 1. Happy Path Sequence
    let s1 = transition_session_state(
        SessionState::Initializing,
        UserSessionAction::Authenticate,
    )
    .expect("Initializing -> Authenticate -> Authenticating");
    assert_eq!(s1, SessionState::Authenticating);

    let s2 = transition_session_state(s1, UserSessionAction::Activate)
        .expect("Authenticating -> Activate -> Active");
    assert_eq!(s2, SessionState::Active);

    let s3 = transition_session_state(s2, UserSessionAction::Lock)
        .expect("Active -> Lock -> Locked");
    assert_eq!(s3, SessionState::Locked);

    let s4 = transition_session_state(s3, UserSessionAction::Unlock)
        .expect("Locked -> Unlock -> Active");
    assert_eq!(s4, SessionState::Active);

    let s5 = transition_session_state(s4, UserSessionAction::Terminate)
        .expect("Active -> Terminate -> Terminating");
    assert_eq!(s5, SessionState::Terminating);

    let s6 = transition_session_state(s5, UserSessionAction::Terminate)
        .expect("Terminating -> Terminate -> Terminated");
    assert_eq!(s6, SessionState::Terminated);

    // 2. Early Termination paths
    assert_eq!(
        transition_session_state(SessionState::Initializing, UserSessionAction::Terminate).unwrap(),
        SessionState::Terminated
    );
    assert_eq!(
        transition_session_state(SessionState::Authenticating, UserSessionAction::Terminate).unwrap(),
        SessionState::Terminated
    );
    assert_eq!(
        transition_session_state(SessionState::Locked, UserSessionAction::Terminate).unwrap(),
        SessionState::Terminating
    );

    // 3. Illegal Transitions (must fail)
    assert!(transition_session_state(SessionState::Terminated, UserSessionAction::Activate).is_err());
    assert!(transition_session_state(SessionState::Terminated, UserSessionAction::Unlock).is_err());
    assert!(transition_session_state(SessionState::Terminated, UserSessionAction::Terminate).is_err());
    assert!(transition_session_state(SessionState::Initializing, UserSessionAction::Lock).is_err());
    assert!(transition_session_state(SessionState::Initializing, UserSessionAction::Activate).is_err());
    assert!(transition_session_state(SessionState::Authenticating, UserSessionAction::Lock).is_err());
    assert!(transition_session_state(SessionState::Locked, UserSessionAction::Authenticate).is_err());
}

#[test]
fn test_sb4_environment_and_path_isolation() {
    let mut spec = sample_x11_spec();
    assert!(validate_user_session_spec(&spec).is_ok());

    // Boundary: 256 environment keys is ok
    for i in 0..253 {
        spec.environment.insert(format!("VAR_{:04}", i), "val".into());
    }
    assert_eq!(spec.environment.len(), 256);
    assert!(validate_user_session_spec(&spec).is_ok());

    // Negative: 257 keys rejected
    spec.environment.insert("VAR_OVERFLOW".into(), "val".into());
    assert_eq!(spec.environment.len(), 257);
    let errs = validate_user_session_spec(&spec).expect_err("should reject >256 env vars");
    assert!(errs.iter().any(|e| e.contains("exceeds 256 items")));

    // Negative: Invalid key syntax
    let mut spec2 = sample_x11_spec();
    spec2.environment.insert("lowercase_key".into(), "value".into());
    let errs2 = validate_user_session_spec(&spec2).expect_err("should reject lowercase env key");
    assert!(errs2.iter().any(|e| e.contains("must start with an uppercase ASCII letter")));

    // Negative: Value with null byte
    let mut spec3 = sample_x11_spec();
    spec3.environment.insert("SAFE_KEY".into(), "null\0byte".into());
    let errs3 = validate_user_session_spec(&spec3).expect_err("should reject null byte in value");
    assert!(errs3.iter().any(|e| e.contains("contains null byte")));

    // Negative: XDG_RUNTIME_DIR relative path
    let mut spec4 = sample_x11_spec();
    spec4.environment.insert("XDG_RUNTIME_DIR".into(), "run/user/1000".into());
    let errs4 = validate_user_session_spec(&spec4).expect_err("should reject relative XDG_RUNTIME_DIR");
    assert!(errs4.iter().any(|e| e.contains("must be an absolute path")));

    // Negative: XDG_RUNTIME_DIR directory traversal
    let mut spec5 = sample_x11_spec();
    spec5.environment.insert("XDG_RUNTIME_DIR".into(), "/run/user/../evil".into());
    let errs5 = validate_user_session_spec(&spec5).expect_err("should reject .. traversal");
    assert!(errs5.iter().any(|e| e.contains("traversal sequence '..'")));
}

#[test]
fn test_sb5_store_capacity_and_user_limits() {
    let mut store = UserSessionStore::new();
    let spec = sample_x11_spec();
    let status = sample_status();

    // 1. Normal addition
    store.add_session(spec.clone(), status.clone()).expect("added session");

    // 2. Duplicate rejection
    let dup_err = store.add_session(spec.clone(), status.clone()).expect_err("duplicate rejected");
    assert!(dup_err.contains("already exists in store"));

    // 3. User limit saturation (max 32 per user)
    for i in 2..=MAX_SESSIONS_PER_USER {
        let mut s = spec.clone();
        s.session_id = format!("sess-{:02}", i);
        let mut st = status.clone();
        st.session_id = format!("sess-{:02}", i);
        store.add_session(s, st).expect("session added within user limit");
    }

    // Attempting 33rd session for user 'kali' must be rejected
    let mut overflow_spec = spec.clone();
    overflow_spec.session_id = "sess-33".into();
    let mut overflow_status = status.clone();
    overflow_status.session_id = "sess-33".into();

    let user_limit_err = store
        .add_session(overflow_spec, overflow_status)
        .expect_err("must reject session exceeding user limit");
    assert!(user_limit_err.contains("maximum of 32 active sessions"));

    // But another user can still create sessions
    let mut other_spec = sample_x11_spec();
    other_spec.session_id = "other-sess-01".into();
    other_spec.username = "root".into();
    other_spec.uid = 0;
    other_spec.gid = 0;

    let mut other_status = sample_status();
    other_status.session_id = "other-sess-01".into();
    other_status.username = "root".into();
    other_status.uid = 0;

    assert!(store.add_session(other_spec, other_status).is_ok());
}

#[test]
fn test_session_status_consistency_validation() {
    let mut valid = sample_status();
    assert!(validate_user_session_status(&valid).is_ok());

    // Contradiction: state is Locked, but locked is false
    valid.state = SessionState::Locked;
    valid.locked = false;
    let errs1 = validate_user_session_status(&valid).expect_err("should reject contradiction");
    assert!(errs1.iter().any(|e| e.contains("state is Locked, but locked flag is false")));

    // Contradiction: state is Active, but locked is true
    valid.state = SessionState::Active;
    valid.locked = true;
    let errs2 = validate_user_session_status(&valid).expect_err("should reject contradiction");
    assert!(errs2.iter().any(|e| e.contains("state is Active, but locked flag is true")));

    // Contradiction: state is Terminated, but scope is Foreground
    valid.state = SessionState::Terminated;
    valid.locked = false;
    valid.scope = SessionScope::Foreground;
    let errs3 = validate_user_session_status(&valid).expect_err("should reject contradiction");
    assert!(errs3.iter().any(|e| e.contains("Terminated session cannot be in Foreground scope")));
}

#[test]
fn test_session_store_persistence_and_atomic_save() {
    let dir = tempdir().expect("tempdir created");
    let store_path = dir.path().join("sessions.json");

    let mut store = UserSessionStore::new();
    let spec = sample_x11_spec();
    let status = sample_status();
    store.add_session(spec, status).expect("added");

    // Atomic save
    store.save_to_path(&store_path).expect("save succeeds");
    assert!(store_path.exists());

    // Load back
    let loaded = UserSessionStore::load_from_path(&store_path).expect("load succeeds");
    assert_eq!(store, loaded);
    assert_eq!(loaded.sessions.len(), 1);
    assert!(loaded.sessions.contains_key("sess-01"));

    // Size limit verification
    let huge_payload = "x".repeat(MAX_SESSION_STORE_SIZE + 1);
    assert!(UserSessionStore::from_json(&huge_payload).is_err());
}

#[test]
fn test_session_query_and_filtering() {
    let mut store = UserSessionStore::new();

    // Session 1: kali, X11, Active, seat0
    store.add_session(sample_x11_spec(), sample_status()).expect("added 1");

    // Session 2: root, Tty, Active, seat0
    let mut s2_spec = sample_x11_spec();
    s2_spec.session_id = "sess-02".into();
    s2_spec.username = "root".into();
    s2_spec.uid = 0;
    s2_spec.gid = 0;
    s2_spec.session_type = SessionType::Tty;
    s2_spec.vtnr = Some(1);
    s2_spec.display = None;

    let mut s2_status = sample_status();
    s2_status.session_id = "sess-02".into();
    s2_status.username = "root".into();
    s2_status.uid = 0;
    store.add_session(s2_spec, s2_status).expect("added 2");

    // Session 3: aios-agent, AiAgent, Active, seat-agent
    let mut s3_spec = sample_x11_spec();
    s3_spec.session_id = "sess-03".into();
    s3_spec.username = "aios-agent".into();
    s3_spec.uid = 1001;
    s3_spec.gid = 1001;
    s3_spec.session_type = SessionType::AiAgent;
    s3_spec.session_class = SessionClass::Agent;
    s3_spec.seat = "seat-agent".into();
    s3_spec.vtnr = None;
    s3_spec.display = None;

    let mut s3_status = sample_status();
    s3_status.session_id = "sess-03".into();
    s3_status.username = "aios-agent".into();
    s3_status.uid = 1001;
    store.add_session(s3_spec, s3_status).expect("added 3");

    // Query 1: by username
    let q1 = UserSessionQuery {
        username: Some("kali".into()),
        ..Default::default()
    };
    let res1 = store.list_sessions(&q1);
    assert_eq!(res1.len(), 1);
    assert_eq!(res1[0].session_id, "sess-01");

    // Query 2: by session_type Tty
    let q2 = UserSessionQuery {
        session_type: Some(SessionType::Tty),
        ..Default::default()
    };
    let res2 = store.list_sessions(&q2);
    assert_eq!(res2.len(), 1);
    assert_eq!(res2[0].session_id, "sess-02");

    // Query 3: by seat
    let q3 = UserSessionQuery {
        seat: Some("seat-agent".into()),
        ..Default::default()
    };
    let res3 = store.list_sessions(&q3);
    assert_eq!(res3.len(), 1);
    assert_eq!(res3[0].session_id, "sess-03");

    // Query 4: limit
    let q4 = UserSessionQuery {
        limit: Some(2),
        ..Default::default()
    };
    let res4 = store.list_sessions(&q4);
    assert_eq!(res4.len(), 2);
}
