//! Integration and Invariant Test Suite for User Session Security Policy (SSP1..SSP7).

use std::collections::BTreeMap;
use aiosh_core::session::*;
use aiosh_core::session_policy::*;

fn sample_spec(id: &str, user: &str, uid: u32, st: SessionType, sc: SessionClass, seat: &str) -> UserSessionSpec {
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
        environment: BTreeMap::new(),
    }
}

#[test]
fn test_default_policy_validation() {
    let policy = UserSessionSecurityPolicy::default();
    assert!(policy.validate().is_ok());
}

#[test]
fn test_ssp1_root_and_greeter_rules() {
    let policy = UserSessionSecurityPolicy::default();

    // 1. Root disallowed by default
    let root_spec = sample_spec("root-sess", "root", 0, SessionType::Tty, SessionClass::User, "seat0");
    let v = policy.evaluate_spec(&root_spec);
    assert!(!v.allowed);
    assert!(v.violations.iter().any(|viol| viol.rule_id == "SSP1-ROOT-DISALLOWED"));

    // 2. Unprivileged greeter disallowed
    let greeter_bad = sample_spec("greet-bad", "alice", 1000, SessionType::Wayland, SessionClass::Greeter, "seat0");
    let v_greet = policy.evaluate_spec(&greeter_bad);
    assert!(!v_greet.allowed);
    assert!(v_greet.violations.iter().any(|viol| viol.rule_id == "SSP1-GREETER-UNPRIVILEGED"));

    // 3. Valid greeter (lightdm, uid < 1000)
    let greeter_good = sample_spec("greet-ok", "lightdm", 62000, SessionType::X11, SessionClass::Greeter, "seat0");
    let v_good = policy.evaluate_spec(&greeter_good);
    assert!(v_good.allowed);
}

#[test]
fn test_ssp2_session_type_and_class_rules() {
    let policy = UserSessionSecurityPolicy::default();

    // Agent class with non-agent type fails
    let agent_bad = sample_spec("agent-bad", "agent1", 1001, SessionType::Tty, SessionClass::Agent, "seat0");
    let v = policy.evaluate_spec(&agent_bad);
    assert!(!v.allowed);
    assert!(v.violations.iter().any(|viol| viol.rule_id == "SSP2-AGENT-CLASS-TYPE-MISMATCH"));

    // Greeter without display fails
    let mut greeter_nodisp = sample_spec("greet-nodisp", "lightdm", 62000, SessionType::X11, SessionClass::Greeter, "seat0");
    greeter_nodisp.display = None;
    let v2 = policy.evaluate_spec(&greeter_nodisp);
    assert!(!v2.allowed);
    assert!(v2.violations.iter().any(|viol| viol.rule_id == "SSP2-GREETER-DISPLAY-REQUIRED"));

    // Greeter with remote host fails
    let mut greeter_remote = sample_spec("greet-rem", "lightdm", 62000, SessionType::X11, SessionClass::Greeter, "seat0");
    greeter_remote.remote_host = Some("192.168.1.100".into());
    let v3 = policy.evaluate_spec(&greeter_remote);
    assert!(!v3.allowed);
    assert!(v3.violations.iter().any(|viol| viol.rule_id == "SSP2-GREETER-REMOTE-DISALLOWED"));
}

#[test]
fn test_ssp3_seat_and_display_rules() {
    let policy = UserSessionSecurityPolicy::default();

    // Remote session attaching to console seat0 is forbidden
    let mut rem_spec = sample_spec("rem-sess", "alice", 1001, SessionType::Wayland, SessionClass::User, "seat0");
    rem_spec.remote_host = Some("10.0.0.5".into());
    let v = policy.evaluate_spec(&rem_spec);
    assert!(!v.allowed);
    assert!(v.violations.iter().any(|viol| viol.rule_id == "SSP3-REMOTE-SEAT0-FORBIDDEN"));

    // Remote on non-seat0 (e.g. seat-remote) is permitted
    rem_spec.seat = "seat-remote".into();
    let v2 = policy.evaluate_spec(&rem_spec);
    assert!(v2.allowed);

    // Invalid VT number (> 64)
    let mut vt_spec = sample_spec("vt-bad", "alice", 1001, SessionType::Tty, SessionClass::User, "seat0");
    vt_spec.vtnr = Some(99);
    let v3 = policy.evaluate_spec(&vt_spec);
    assert!(!v3.allowed);
    assert!(v3.violations.iter().any(|viol| viol.rule_id == "SSP3-VTNR-OUT-OF-BOUNDS"));
}

#[test]
fn test_ssp4_environment_sanitization() {
    let policy = UserSessionSecurityPolicy::default();

    // Dangerous environment variables disallowed
    let mut env_spec = sample_spec("env-bad", "alice", 1001, SessionType::Wayland, SessionClass::User, "seat0");
    env_spec.environment.insert("LD_PRELOAD".into(), "/tmp/libhack.so".into());
    let v = policy.evaluate_spec(&env_spec);
    assert!(!v.allowed);
    assert!(v.violations.iter().any(|viol| viol.rule_id == "SSP4-DISALLOWED-ENV-VAR"));

    // Normal environment permitted
    env_spec.environment.clear();
    env_spec.environment.insert("LANG".into(), "en_US.UTF-8".into());
    env_spec.environment.insert("TERM".into(), "xterm-256color".into());
    let v2 = policy.evaluate_spec(&env_spec);
    assert!(v2.allowed);
}

#[test]
fn test_ssp5_concurrency_quotas() {
    let mut policy = UserSessionSecurityPolicy::default();
    policy.max_sessions_per_user = 2;
    policy.max_total_sessions = 3;

    let mut store = UserSessionStore::new();
    // Add 3 sessions for alice
    for i in 1..=3 {
        let spec = sample_spec(&format!("s-{}", i), "alice", 1001, SessionType::Wayland, SessionClass::User, "seat0");
        let status = UserSessionStatus {
            session_id: format!("s-{}", i),
            username: "alice".into(),
            uid: 1001,
            state: SessionState::Active,
            scope: SessionScope::Background,
            leader_pid: None,
            created_at: "2026-09-11T00:00:00Z".into(),
            last_active_at: "2026-09-11T00:00:00Z".into(),
            idle_seconds: 0,
            locked: false,
        };
        store.specs.insert(format!("s-{}", i), spec);
        store.sessions.insert(format!("s-{}", i), status);
    }

    let verdicts = policy.evaluate_store(&store);
    assert!(verdicts.iter().any(|v| v.violations.iter().any(|viol| viol.rule_id == "SSP5-USER-QUOTA-EXCEEDED")));
}

#[test]
fn test_ssp6_agent_sandboxing() {
    let policy = UserSessionSecurityPolicy::default();

    // Agent with UID 0 (root) forbidden
    let agent_root = sample_spec("agent-root", "root", 0, SessionType::AiAgent, SessionClass::Agent, "seat0");
    let v = policy.evaluate_spec(&agent_root);
    assert!(!v.allowed);
    assert!(v.violations.iter().any(|viol| viol.rule_id == "SSP6-AGENT-ROOT-FORBIDDEN"));

    // Agent with UID 1001 permitted
    let agent_ok = sample_spec("agent-ok", "agent", 1001, SessionType::AiAgent, SessionClass::Agent, "seat0");
    let v2 = policy.evaluate_spec(&agent_ok);
    assert!(v2.allowed);
}

#[test]
fn test_ssp7_policy_modes_and_file_caps() {
    let mut policy = UserSessionSecurityPolicy::default();
    let root_spec = sample_spec("root-sess", "root", 0, SessionType::Tty, SessionClass::User, "seat0");

    // Enforcing: not allowed
    policy.mode = SessionPolicyMode::Enforcing;
    assert!(!policy.evaluate_spec(&root_spec).allowed);

    // Audit: allowed = true, but violations present
    policy.mode = SessionPolicyMode::Audit;
    let v_audit = policy.evaluate_spec(&root_spec);
    assert!(v_audit.allowed);
    assert!(!v_audit.violations.is_empty());

    // Permissive: allowed = true
    policy.mode = SessionPolicyMode::Permissive;
    assert!(policy.evaluate_spec(&root_spec).allowed);
}
