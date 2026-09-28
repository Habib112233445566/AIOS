//! Unit tests for Sandbox Enforcement Security Policy (T-02465).

use std::collections::BTreeMap;
use tempfile::tempdir;

use aiosh_core::audit::{AuditRing, OpenOptions};
use aiosh_core::sandbox_data_model::*;
use aiosh_core::sandbox_policy::*;
use aiosh_core::sandbox_service::*;

#[test]
fn test_policy_defaults_and_validation() {
    let policy = SandboxSecurityPolicy::default();
    assert_eq!(policy.mode, SandboxPolicyMode::Enforcing);
    assert!(policy.validate().is_ok());

    let mut invalid = policy.clone();
    invalid.version = "a".repeat(MAX_SANDBOX_POLICY_VERSION_LEN + 1);
    assert!(invalid.validate().is_err());

    let mut invalid_time = policy.clone();
    invalid_time.max_permissible_wall_time_ms = 0;
    assert!(invalid_time.validate().is_err());
}

#[test]
fn test_policy_prohibited_command_denial() {
    let mut policy = SandboxSecurityPolicy::default();
    policy.prohibited_commands = vec!["rm".into(), "dd".into()];

    let profile = SandboxProfile::standard();
    let req = SandboxExecutionRequest {
        command: "rm".into(),
        args: vec!["-rf".into(), "/tmp/test".into()],
        cwd: None,
        profile: profile.clone(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };

    // 1. Enforcing mode -> Deny
    policy.mode = SandboxPolicyMode::Enforcing;
    match policy.evaluate(&req) {
        SandboxPolicyVerdict::Deny { reason, code } => {
            assert_eq!(code, SANDBOXPOL_ERR_DENIED);
            assert!(reason.contains("rm"));
        }
        other => panic!("expected Deny, got {:?}", other),
    }

    // 2. Permissive mode -> PermitWithWarning
    policy.mode = SandboxPolicyMode::Permissive;
    match policy.evaluate(&req) {
        SandboxPolicyVerdict::PermitWithWarning { warning } => {
            assert!(warning.contains("rm"));
        }
        other => panic!("expected PermitWithWarning, got {:?}", other),
    }

    // 3. Disabled mode -> Permit
    policy.mode = SandboxPolicyMode::Disabled;
    assert_eq!(policy.evaluate(&req), SandboxPolicyVerdict::Permit);
}

#[test]
fn test_policy_prohibited_env_vars() {
    let policy = SandboxSecurityPolicy::default();
    let mut profile = SandboxProfile::standard();
    let mut injected = BTreeMap::new();
    injected.insert("LD_PRELOAD".into(), "/opt/evil.so".into());
    profile.environment.injected_vars = injected;

    let req = SandboxExecutionRequest {
        command: "python".into(),
        args: vec!["-c".into(), "print('test')".into()],
        cwd: None,
        profile,
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };

    match policy.evaluate(&req) {
        SandboxPolicyVerdict::Deny { reason, code } => {
            assert_eq!(code, SANDBOXPOL_ERR_DENIED);
            assert!(reason.contains("LD_PRELOAD"));
        }
        other => panic!("expected Deny, got {:?}", other),
    }
}

#[test]
fn test_policy_pep_grant_mandate() {
    let mut policy = SandboxSecurityPolicy::default();
    policy.require_pep_grant_for_profiles = vec!["custom_privileged".into()];

    let mut profile = SandboxProfile::standard();
    profile.name = "custom_privileged".into();

    let mut req = SandboxExecutionRequest {
        command: "python".into(),
        args: vec!["-c".into(), "print('test')".into()],
        cwd: None,
        profile,
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };

    // Missing grant -> Deny
    match policy.evaluate(&req) {
        SandboxPolicyVerdict::Deny { reason, code } => {
            assert_eq!(code, SANDBOXPOL_ERR_DENIED);
            assert!(reason.contains("PEP capability grant"));
        }
        other => panic!("expected Deny, got {:?}", other),
    }

    // Valid grant -> Permit
    req.pep_grant_id = Some("pep_grant_999".into());
    assert_eq!(policy.evaluate(&req), SandboxPolicyVerdict::Permit);
}

#[test]
fn test_policy_resource_ceilings() {
    let mut policy = SandboxSecurityPolicy::default();
    policy.max_permissible_wall_time_ms = 5000; // 5 seconds cap

    let mut profile = SandboxProfile::standard();
    profile.resources.max_wall_time_ms = 10000; // 10 seconds requested

    let req = SandboxExecutionRequest {
        command: "python".into(),
        args: vec!["-c".into(), "print('test')".into()],
        cwd: None,
        profile,
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };

    match policy.evaluate(&req) {
        SandboxPolicyVerdict::Deny { reason, code } => {
            assert_eq!(code, SANDBOXPOL_ERR_DENIED);
            assert!(reason.contains("requested wall time"));
        }
        other => panic!("expected Deny, got {:?}", other),
    }
}

#[test]
fn test_policy_persistence_and_bounds() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("sandbox_policy.json");

    let policy = SandboxSecurityPolicy::default();
    assert!(policy.save_to_path(&file_path).is_ok());

    let loaded = SandboxSecurityPolicy::load_from_path(&file_path).expect("load policy");
    assert_eq!(policy, loaded);
}

#[test]
fn test_service_policy_enforcement_and_audit() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("policy_audit.db");
    let ring = AuditRing::open(OpenOptions {
        path: Some(db_path.to_str().unwrap().to_string()),
        home: None,
    }).expect("open ring");

    let mut svc = SandboxService::with_default_profiles(Some(ring));
    let mut policy = SandboxSecurityPolicy::default();
    policy.prohibited_commands = vec!["prohibited_tool".into()];
    svc.set_policy(policy);

    let profile = svc.get_profile("permissive").unwrap();
    let req = SandboxExecutionRequest {
        command: "prohibited_tool".into(),
        args: vec![],
        cwd: None,
        profile,
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };

    let res = svc.execute(&req);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains(SANDBOXPOL_ERR_DENIED));

    // Verify audit entry
    if let Some(r) = svc.ring() {
        let rows = r.tail(5).expect("tail audit rows");
        assert!(!rows.is_empty());
        assert_eq!(rows[0].outcome, "denied");
        assert_eq!(rows[0].command, "prohibited_tool");
    }
}
