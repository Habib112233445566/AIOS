//! Unit test suite for Privilege Escalation Prevention Data Model (`T-02505`).

use aiosh_core::privilege_data_model::{
    PrivilegeCapability, PrivilegeContext, PrivilegeEscalationVerdict, PrivilegeLevel,
    PrivilegeTransitionRequest, MAX_ACTOR_ID_LEN, MAX_CAPABILITIES_COUNT,
    PRIVESC_ERR_INVALID_ACTOR, PRIVESC_ERR_KERNEL_TIER_IMMUTABLE,
    PRIVESC_ERR_UNAUTHORIZED_ELEVATION,
};

#[test]
fn test_actor_id_validation_rules() {
    // Empty actor ID rejected
    assert!(PrivilegeContext::new("", PrivilegeLevel::User).is_err());
    assert!(PrivilegeContext::new("   ", PrivilegeLevel::User).is_err());

    // Control characters rejected
    assert!(PrivilegeContext::new("alice\n", PrivilegeLevel::User).is_err());
    assert!(PrivilegeContext::new("bob\t", PrivilegeLevel::User).is_err());

    // Excessively long actor ID rejected
    let long_actor = "a".repeat(MAX_ACTOR_ID_LEN + 1);
    let res = PrivilegeContext::new(long_actor, PrivilegeLevel::User);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains(PRIVESC_ERR_INVALID_ACTOR));

    // Valid actor accepted and trimmed
    let ctx = PrivilegeContext::new("  valid_actor_01  ", PrivilegeLevel::User).unwrap();
    assert_eq!(ctx.actor_id, "valid_actor_01");
}

#[test]
fn test_capability_capacity_limit() {
    let mut ctx = PrivilegeContext::new("cap_tester", PrivilegeLevel::Admin).unwrap();

    // Insert valid capabilities
    assert!(ctx.add_capability(PrivilegeCapability::ProcessSpawn).is_ok());
    assert!(ctx.add_capability(PrivilegeCapability::NetworkConnect).is_ok());
    assert!(ctx.add_capability(PrivilegeCapability::NetworkListen).is_ok());
    assert!(ctx.add_capability(PrivilegeCapability::FilesystemWrite).is_ok());
    assert!(ctx.add_capability(PrivilegeCapability::MemoryInspect).is_ok());
    assert!(ctx.add_capability(PrivilegeCapability::AuditLogAdmin).is_ok());
    assert!(ctx.add_capability(PrivilegeCapability::SystemReboot).is_ok());
    assert!(ctx.add_capability(PrivilegeCapability::KernelModuleLoad).is_ok());

    // Inserting existing capability is idempotent
    assert!(ctx.add_capability(PrivilegeCapability::ProcessSpawn).is_ok());
    assert_eq!(ctx.capabilities.len(), 8);
    assert!(ctx.capabilities.len() <= MAX_CAPABILITIES_COUNT);
    assert!(ctx.validate().is_ok());
}

#[test]
fn test_privilege_dropping_boundary_enforcement() {
    let mut ctx = PrivilegeContext::new("downgrade_user", PrivilegeLevel::Admin).unwrap();
    ctx.add_capability(PrivilegeCapability::KernelModuleLoad).unwrap();
    ctx.add_capability(PrivilegeCapability::ProcessSpawn).unwrap();
    ctx.add_capability(PrivilegeCapability::NetworkConnect).unwrap();
    ctx.elevation_grant_id = Some("GRANT-ADMIN-01".to_string());
    ctx.is_elevation_active = true;

    // Dropping from Admin to User
    assert!(ctx.drop_to_level(PrivilegeLevel::User).is_ok());
    assert_eq!(ctx.active_level, PrivilegeLevel::User);
    assert!(!ctx.is_elevation_active);
    assert_eq!(ctx.elevation_grant_id, None);

    // KernelModuleLoad requires Admin, so it must be pruned
    assert!(!ctx.has_capability(&PrivilegeCapability::KernelModuleLoad));
    // ProcessSpawn requires User, so it remains
    assert!(ctx.has_capability(&PrivilegeCapability::ProcessSpawn));
    // NetworkConnect requires Guest (<= User), so it remains
    assert!(ctx.has_capability(&PrivilegeCapability::NetworkConnect));

    // Further drop to Guest
    assert!(ctx.drop_to_level(PrivilegeLevel::Guest).is_ok());
    assert_eq!(ctx.active_level, PrivilegeLevel::Guest);
    // ProcessSpawn requires User (> Guest), so it is pruned
    assert!(!ctx.has_capability(&PrivilegeCapability::ProcessSpawn));
    // NetworkConnect requires Guest, remains
    assert!(ctx.has_capability(&PrivilegeCapability::NetworkConnect));

    // Attempting to drop to higher level must fail with PRIVESC_ERR_UNAUTHORIZED_ELEVATION
    let drop_err = ctx.drop_to_level(PrivilegeLevel::Operator).unwrap_err();
    assert!(drop_err.contains(PRIVESC_ERR_UNAUTHORIZED_ELEVATION));
}

#[test]
fn test_elevation_grant_rules_and_kernel_tier_defense() {
    let mut ctx = PrivilegeContext::new("normal_user", PrivilegeLevel::User).unwrap();

    // Elevation without grant token fails
    assert!(ctx.elevate_with_grant(PrivilegeLevel::Operator, "").is_err());
    assert!(ctx.elevate_with_grant(PrivilegeLevel::Operator, "   ").is_err());

    // Valid elevation with token succeeds
    assert!(ctx.elevate_with_grant(PrivilegeLevel::Operator, "GRANT-TOKEN-100").is_ok());
    assert_eq!(ctx.active_level, PrivilegeLevel::Operator);
    assert!(ctx.is_elevation_active);
    assert_eq!(ctx.elevation_grant_id.as_deref(), Some("GRANT-TOKEN-100"));

    // Elevation to SystemKernel is strictly denied
    let kernel_err = ctx.elevate_with_grant(PrivilegeLevel::SystemKernel, "GRANT-ROOT-999").unwrap_err();
    assert!(kernel_err.contains(PRIVESC_ERR_KERNEL_TIER_IMMUTABLE));
    assert_ne!(ctx.active_level, PrivilegeLevel::SystemKernel);

    // Revocation restores base level
    ctx.revoke_elevation(PrivilegeLevel::User);
    assert_eq!(ctx.active_level, PrivilegeLevel::User);
    assert!(!ctx.is_elevation_active);
    assert_eq!(ctx.elevation_grant_id, None);
}

#[test]
fn test_transition_request_verdict_matrix() {
    // 1. Same tier transition without grant -> Allowed
    let req_same = PrivilegeTransitionRequest {
        actor_id: "agent_a".to_string(),
        from_level: PrivilegeLevel::User,
        target_level: PrivilegeLevel::User,
        requested_capabilities: vec![],
        grant_id: None,
    };
    assert_eq!(req_same.evaluate(), PrivilegeEscalationVerdict::Allowed);

    // 2. Privilege reduction without grant -> Allowed
    let req_demote = PrivilegeTransitionRequest {
        actor_id: "agent_a".to_string(),
        from_level: PrivilegeLevel::Operator,
        target_level: PrivilegeLevel::User,
        requested_capabilities: vec![],
        grant_id: None,
    };
    assert_eq!(req_demote.evaluate(), PrivilegeEscalationVerdict::Allowed);

    // 3. Monotonic escalation without grant -> GrantRequired
    let req_elevate = PrivilegeTransitionRequest {
        actor_id: "agent_a".to_string(),
        from_level: PrivilegeLevel::User,
        target_level: PrivilegeLevel::Admin,
        requested_capabilities: vec![],
        grant_id: None,
    };
    match req_elevate.evaluate() {
        PrivilegeEscalationVerdict::GrantRequired { reason } => {
            assert!(reason.contains(PRIVESC_ERR_UNAUTHORIZED_ELEVATION));
        }
        other => panic!("expected GrantRequired, got {:?}", other),
    }

    // 4. Monotonic escalation with grant -> Allowed
    let req_elevate_ok = PrivilegeTransitionRequest {
        actor_id: "agent_a".to_string(),
        from_level: PrivilegeLevel::User,
        target_level: PrivilegeLevel::Admin,
        requested_capabilities: vec![],
        grant_id: Some("TOKEN-XYZ".to_string()),
    };
    assert_eq!(req_elevate_ok.evaluate(), PrivilegeEscalationVerdict::Allowed);

    // 5. SystemKernel target -> Denied regardless of grant
    let req_kernel = PrivilegeTransitionRequest {
        actor_id: "agent_a".to_string(),
        from_level: PrivilegeLevel::Admin,
        target_level: PrivilegeLevel::SystemKernel,
        requested_capabilities: vec![],
        grant_id: Some("ANY-TOKEN".to_string()),
    };
    match req_kernel.evaluate() {
        PrivilegeEscalationVerdict::Denied { reason } => {
            assert!(reason.contains(PRIVESC_ERR_KERNEL_TIER_IMMUTABLE));
        }
        other => panic!("expected Denied, got {:?}", other),
    }
}
