//! Integration and Unit Tests for Privilege Escalation Prevention Security Policy (T-02565).

use tempfile::tempdir;

use aiosh_core::privilege_data_model::*;
use aiosh_core::privilege_policy::*;
use aiosh_core::privilege_service::*;

#[test]
fn test_policy_defaults_and_validation() {
    let policy = PrivilegeSecurityPolicy::default();
    assert_eq!(policy.mode, PrivilegePolicyMode::Enforcing);
    assert_eq!(policy.version, "1.0.0");
    assert!(policy.validate().is_ok());

    // Invalid empty version
    let mut invalid = policy.clone();
    invalid.version = "   ".into();
    assert!(invalid.validate().is_err());

    // Disallowed targets without SystemKernel fails
    let mut no_kernel = policy.clone();
    no_kernel.disallowed_elevation_targets.clear();
    let err = no_kernel.validate().unwrap_err();
    assert!(err.contains(PRIVESC_ERR_KERNEL_TIER_IMMUTABLE));
}

#[test]
fn test_policy_enforcement_modes_and_verdicts() {
    let mut policy = PrivilegeSecurityPolicy::default();
    policy.prohibited_capabilities = vec![PrivilegeCapability::KernelModuleLoad];

    let req = PrivilegeTransitionRequest {
        actor_id: "agent_smith".into(),
        from_level: PrivilegeLevel::Operator,
        target_level: PrivilegeLevel::Admin,
        requested_capabilities: vec![PrivilegeCapability::KernelModuleLoad],
        grant_id: Some("grant-root-admin".into()),
    };

    // 1. Enforcing -> Deny
    policy.mode = PrivilegePolicyMode::Enforcing;
    match policy.evaluate_transition(&req) {
        PrivilegePolicyVerdict::Deny { reason, code } => {
            assert_eq!(code, PRIVESCPOL_ERR_DENIED);
            assert!(reason.contains("prohibited"));
        }
        other => panic!("expected Deny, got {:?}", other),
    }

    // 2. Permissive -> PermitWithWarning
    policy.mode = PrivilegePolicyMode::Permissive;
    match policy.evaluate_transition(&req) {
        PrivilegePolicyVerdict::PermitWithWarning { warning } => {
            assert!(warning.contains("prohibited"));
        }
        other => panic!("expected PermitWithWarning, got {:?}", other),
    }

    // 3. Disabled -> Permit
    policy.mode = PrivilegePolicyMode::Disabled;
    assert_eq!(policy.evaluate_transition(&req), PrivilegePolicyVerdict::Permit);
}

#[test]
fn test_policy_actor_tier_ceilings() {
    let mut policy = PrivilegeSecurityPolicy::default();
    policy.actor_tier_ceilings.insert("contractor".into(), PrivilegeLevel::User);

    let req = PrivilegeTransitionRequest {
        actor_id: "contractor".into(),
        from_level: PrivilegeLevel::User,
        target_level: PrivilegeLevel::Operator,
        requested_capabilities: vec![],
        grant_id: Some("grant-temp-elevate".into()),
    };

    match policy.evaluate_transition(&req) {
        PrivilegePolicyVerdict::Deny { reason, code } => {
            assert_eq!(code, PRIVESCPOL_ERR_DENIED);
            assert!(reason.contains("ceiling"));
        }
        other => panic!("expected Deny, got {:?}", other),
    }
}

#[test]
fn test_policy_service_integration() {
    let mut service = PrivilegeService::new();
    let ctx = PrivilegeContext::new("restricted_user", PrivilegeLevel::User).unwrap();
    assert!(service.register_context(ctx).is_ok());

    let mut policy = PrivilegeSecurityPolicy::default();
    policy.actor_tier_ceilings.insert("restricted_user".into(), PrivilegeLevel::User);
    assert!(service.set_policy(policy).is_ok());

    // Attempt elevation beyond ceiling must fail
    let req = PrivilegeTransitionRequest {
        actor_id: "restricted_user".into(),
        from_level: PrivilegeLevel::User,
        target_level: PrivilegeLevel::Operator,
        requested_capabilities: vec![],
        grant_id: Some("grant-bypass-attempt".into()),
    };
    let err = service.request_elevation(req).unwrap_err();
    assert!(err.contains(PRIVESCPOL_ERR_DENIED));
}

#[test]
fn test_policy_persistence_and_path_hygiene() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("privilege_policy.json");

    let mut policy = PrivilegeSecurityPolicy::default();
    policy.version = "2.0.0".into();
    policy.actor_tier_ceilings.insert("worker1".into(), PrivilegeLevel::Operator);

    assert!(policy.save_to_path(&path).is_ok());
    assert!(path.exists());

    let loaded = PrivilegeSecurityPolicy::load_from_path(&path).expect("load policy");
    assert_eq!(loaded.version, "2.0.0");
    assert_eq!(loaded.actor_tier_ceilings.get("worker1"), Some(&PrivilegeLevel::Operator));

    // Traversal rejection
    let trav_path = dir.path().join("../evil_policy.json");
    assert!(policy.save_to_path(&trav_path).is_err());
    assert!(PrivilegeSecurityPolicy::load_from_path(&trav_path).is_err());
}

#[test]
fn test_policy_env_overrides() {
    std::env::set_var("AIOS_PRIVILEGE_POLICY_MODE", "permissive");
    let pol = PrivilegeSecurityPolicy::load_with_env_overrides();
    assert_eq!(pol.mode, PrivilegePolicyMode::Permissive);
    std::env::remove_var("AIOS_PRIVILEGE_POLICY_MODE");
}
