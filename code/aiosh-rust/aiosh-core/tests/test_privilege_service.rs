//! Unit test suite for Privilege Escalation Prevention Core Service (`T-02515`).

use aiosh_core::privilege_data_model::{
    PrivilegeCapability, PrivilegeContext, PrivilegeLevel, PrivilegeTransitionRequest,
    PRIVESC_ERR_KERNEL_TIER_IMMUTABLE,
};
use aiosh_core::privilege_service::{
    PrivilegeService, PRIVESC_ERR_ACTOR_NOT_FOUND, PRIVESC_ERR_CAPACITY_EXCEEDED,
    PRIVESC_ERR_CONTEXT_EXISTS,
};

#[test]
fn test_service_capacity_and_bounds() {
    let mut srv = PrivilegeService::with_capacity(2);
    assert_eq!(srv.active_contexts_count(), 0);

    let ctx1 = PrivilegeContext::new("actor1", PrivilegeLevel::Guest).unwrap();
    let ctx2 = PrivilegeContext::new("actor2", PrivilegeLevel::User).unwrap();
    assert!(srv.register_context(ctx1).is_ok());
    assert!(srv.register_context(ctx2).is_ok());
    assert_eq!(srv.active_contexts_count(), 2);

    // Overflow fails
    let ctx3 = PrivilegeContext::new("actor3", PrivilegeLevel::Operator).unwrap();
    let err = srv.register_context(ctx3).unwrap_err();
    assert!(err.contains(PRIVESC_ERR_CAPACITY_EXCEEDED));

    // Clear empties service
    srv.clear();
    assert_eq!(srv.active_contexts_count(), 0);
    assert!(!srv.contains_actor("actor1"));
}

#[test]
fn test_service_actor_not_found_errors() {
    let mut srv = PrivilegeService::new();

    // Unknown actor queries
    assert_eq!(srv.get_context("ghost"), None);
    assert_eq!(srv.get_base_level("ghost"), None);
    assert!(!srv.check_capability("ghost", PrivilegeCapability::ProcessSpawn));

    // Unknown actor unregister
    let unreg_err = srv.unregister_context("ghost").unwrap_err();
    assert!(unreg_err.contains(PRIVESC_ERR_ACTOR_NOT_FOUND));

    // Unknown actor drop
    let drop_err = srv.drop_privilege("ghost", PrivilegeLevel::Guest).unwrap_err();
    assert!(drop_err.contains(PRIVESC_ERR_ACTOR_NOT_FOUND));

    // Unknown actor revoke
    let revoke_err = srv.revoke_elevation("ghost").unwrap_err();
    assert!(revoke_err.contains(PRIVESC_ERR_ACTOR_NOT_FOUND));

    // Unknown actor elevation
    let req = PrivilegeTransitionRequest {
        actor_id: "ghost".to_string(),
        from_level: PrivilegeLevel::User,
        target_level: PrivilegeLevel::Admin,
        requested_capabilities: vec![],
        grant_id: Some("GRANT-1".to_string()),
    };
    let elev_err = srv.request_elevation(req).unwrap_err();
    assert!(elev_err.contains(PRIVESC_ERR_ACTOR_NOT_FOUND));
}

#[test]
fn test_service_multi_step_escalation_and_dropping() {
    let mut srv = PrivilegeService::new();
    let ctx = PrivilegeContext::new("worker_service", PrivilegeLevel::User).unwrap();
    srv.register_context(ctx).unwrap();
    assert_eq!(srv.get_base_level("worker_service"), Some(PrivilegeLevel::User));

    // 1. Elevate to Operator with NetworkListen
    let req1 = PrivilegeTransitionRequest {
        actor_id: "worker_service".to_string(),
        from_level: PrivilegeLevel::User,
        target_level: PrivilegeLevel::Operator,
        requested_capabilities: vec![PrivilegeCapability::NetworkListen],
        grant_id: Some("GRANT-OP-1".to_string()),
    };
    let c1 = srv.request_elevation(req1).unwrap();
    assert_eq!(c1.active_level, PrivilegeLevel::Operator);
    assert!(srv.check_capability("worker_service", PrivilegeCapability::NetworkListen));

    // 2. Elevate to Admin with AuditLogAdmin
    let req2 = PrivilegeTransitionRequest {
        actor_id: "worker_service".to_string(),
        from_level: PrivilegeLevel::Operator,
        target_level: PrivilegeLevel::Admin,
        requested_capabilities: vec![PrivilegeCapability::AuditLogAdmin],
        grant_id: Some("GRANT-ADM-2".to_string()),
    };
    let c2 = srv.request_elevation(req2).unwrap();
    assert_eq!(c2.active_level, PrivilegeLevel::Admin);
    assert!(srv.check_capability("worker_service", PrivilegeCapability::AuditLogAdmin));

    // 3. Drop down to Operator (prunes AuditLogAdmin)
    let c3 = srv.drop_privilege("worker_service", PrivilegeLevel::Operator).unwrap();
    assert_eq!(c3.active_level, PrivilegeLevel::Operator);
    assert!(!srv.check_capability("worker_service", PrivilegeCapability::AuditLogAdmin));

    // 4. Revoke elevation entirely (restores User base level)
    let c4 = srv.revoke_elevation("worker_service").unwrap();
    assert_eq!(c4.active_level, PrivilegeLevel::User);
    assert!(!c4.is_elevation_active);
    assert_eq!(c4.elevation_grant_id, None);
}

#[test]
fn test_service_kernel_tier_defense() {
    let mut srv = PrivilegeService::new();
    let ctx = PrivilegeContext::new("super_admin", PrivilegeLevel::Admin).unwrap();
    srv.register_context(ctx).unwrap();

    let kernel_req = PrivilegeTransitionRequest {
        actor_id: "super_admin".to_string(),
        from_level: PrivilegeLevel::Admin,
        target_level: PrivilegeLevel::SystemKernel,
        requested_capabilities: vec![],
        grant_id: Some("GRANT-ROOT-ATTEMPT".to_string()),
    };
    let err = srv.request_elevation(kernel_req).unwrap_err();
    assert!(err.contains(PRIVESC_ERR_KERNEL_TIER_IMMUTABLE));

    // Ensure state remained Admin and not corrupted
    let curr = srv.get_context("super_admin").unwrap();
    assert_eq!(curr.active_level, PrivilegeLevel::Admin);
}

#[test]
fn test_service_duplicate_actor_handling() {
    let mut srv = PrivilegeService::new();
    let ctx = PrivilegeContext::new("alice", PrivilegeLevel::User).unwrap();
    assert!(srv.register_context(ctx).is_ok());

    let ctx_dup = PrivilegeContext::new("alice", PrivilegeLevel::Admin).unwrap();
    let err = srv.register_context(ctx_dup).unwrap_err();
    assert!(err.contains(PRIVESC_ERR_CONTEXT_EXISTS));
}
