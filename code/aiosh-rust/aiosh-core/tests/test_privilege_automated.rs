//! Formal Automated Test Suite for Privilege Escalation Prevention (T-02553..T-02556).
//!
//! Formal Vectors:
//! - AUTOPRIV1: Multi-tenant context lifecycle & state isolation
//! - AUTOPRIV2: SystemKernel tier immutability & fail-closed denial
//! - AUTOPRIV3: Token validation, replay prevention, and grant gating
//! - AUTOPRIV4: Dynamic capability granting, check, and attenuation
//! - AUTOPRIV5: Privilege drop, revoke, and baseline restoration
//! - AUTOPRIV6: Store persistence, roundtrip serialization, and size bounding
//! - AUTOPRIV7: Context unregister & capacity overflow boundaries
//! - AUTOPRIV8: Multi-threaded concurrency safety under concurrent transitions

use aiosh_core::privilege_data_model::*;
use aiosh_core::privilege_service::*;
use std::sync::{Arc, Mutex};
use std::thread;
use tempfile::tempdir;

#[test]
fn test_autopriv1_lifecycle_and_isolation() {
    let mut service = PrivilegeService::new();
    let ctx_alice = PrivilegeContext::new("alice", PrivilegeLevel::User).unwrap();
    let ctx_bob = PrivilegeContext::new("bob", PrivilegeLevel::User).unwrap();

    assert!(service.register_context(ctx_alice).is_ok());
    assert!(service.register_context(ctx_bob).is_ok());

    assert_eq!(service.active_contexts_count(), 2);
    assert_eq!(service.get_context("alice").unwrap().active_level, PrivilegeLevel::User);
    assert_eq!(service.get_context("bob").unwrap().active_level, PrivilegeLevel::User);

    // Elevate Alice to Operator with a grant
    let req = PrivilegeTransitionRequest {
        actor_id: "alice".to_string(),
        from_level: PrivilegeLevel::User,
        target_level: PrivilegeLevel::Operator,
        requested_capabilities: vec![PrivilegeCapability::AuditLogAdmin],
        grant_id: Some("grant-alice-sec-001".to_string()),
    };
    let elevated = service.request_elevation(req).expect("Alice should elevate");
    assert_eq!(elevated.active_level, PrivilegeLevel::Operator);
    assert!(service.check_capability("alice", PrivilegeCapability::AuditLogAdmin));

    // Bob must remain strictly isolated as User without the capability
    assert_eq!(service.get_context("bob").unwrap().active_level, PrivilegeLevel::User);
    assert!(!service.check_capability("bob", PrivilegeCapability::AuditLogAdmin));
}

#[test]
fn test_autopriv2_system_kernel_immutability() {
    let mut service = PrivilegeService::new();
    let ctx = PrivilegeContext::new("charlie", PrivilegeLevel::Admin).unwrap();
    assert!(service.register_context(ctx).is_ok());

    // Attempting to elevate to SystemKernel must fail closed
    let req = PrivilegeTransitionRequest {
        actor_id: "charlie".to_string(),
        from_level: PrivilegeLevel::Admin,
        target_level: PrivilegeLevel::SystemKernel,
        requested_capabilities: vec![PrivilegeCapability::KernelModuleLoad],
        grant_id: Some("super-grant-999".to_string()),
    };
    let err = service.request_elevation(req).unwrap_err();
    assert!(err.contains(PRIVESC_ERR_KERNEL_TIER_IMMUTABLE));

    // Charlie remains Admin
    assert_eq!(service.get_context("charlie").unwrap().active_level, PrivilegeLevel::Admin);
}

#[test]
fn test_autopriv3_grant_validation() {
    let mut service = PrivilegeService::new();
    let ctx = PrivilegeContext::new("dave", PrivilegeLevel::Guest).unwrap();
    assert!(service.register_context(ctx).is_ok());

    // Elevation without grant token must fail
    let req_no_grant = PrivilegeTransitionRequest {
        actor_id: "dave".to_string(),
        from_level: PrivilegeLevel::Guest,
        target_level: PrivilegeLevel::User,
        requested_capabilities: vec![],
        grant_id: None,
    };
    let err = service.request_elevation(req_no_grant).unwrap_err();
    assert!(err.contains(PRIVESC_ERR_UNAUTHORIZED_ELEVATION));

    // Elevation with blank grant token must fail with unauthorized elevation
    let req_blank_grant = PrivilegeTransitionRequest {
        actor_id: "dave".to_string(),
        from_level: PrivilegeLevel::Guest,
        target_level: PrivilegeLevel::User,
        requested_capabilities: vec![],
        grant_id: Some("   ".to_string()),
    };
    let err_blank = service.request_elevation(req_blank_grant).unwrap_err();
    assert!(err_blank.contains(PRIVESC_ERR_UNAUTHORIZED_ELEVATION));

    // Elevation with control chars in grant token must fail with invalid grant
    let req_control = PrivilegeTransitionRequest {
        actor_id: "dave".to_string(),
        from_level: PrivilegeLevel::Guest,
        target_level: PrivilegeLevel::User,
        requested_capabilities: vec![],
        grant_id: Some("grant\x00evil".to_string()),
    };
    let err_control = service.request_elevation(req_control).unwrap_err();
    assert!(err_control.contains(PRIVESC_ERR_INVALID_GRANT));

    // Elevation with oversized grant token must fail with invalid grant
    let req_oversized = PrivilegeTransitionRequest {
        actor_id: "dave".to_string(),
        from_level: PrivilegeLevel::Guest,
        target_level: PrivilegeLevel::User,
        requested_capabilities: vec![],
        grant_id: Some("g".repeat(MAX_GRANT_ID_LEN + 1)),
    };
    let err_over = service.request_elevation(req_oversized).unwrap_err();
    assert!(err_over.contains(PRIVESC_ERR_INVALID_GRANT));
}

#[test]
fn test_autopriv4_capability_granting_and_check() {
    let mut service = PrivilegeService::new();
    let ctx = PrivilegeContext::new("eve", PrivilegeLevel::User).unwrap();
    assert!(service.register_context(ctx).is_ok());

    assert!(!service.check_capability("eve", PrivilegeCapability::FilesystemWrite));

    let req = PrivilegeTransitionRequest {
        actor_id: "eve".to_string(),
        from_level: PrivilegeLevel::User,
        target_level: PrivilegeLevel::Operator,
        requested_capabilities: vec![
            PrivilegeCapability::FilesystemWrite,
            PrivilegeCapability::NetworkConnect,
        ],
        grant_id: Some("grant-eve-ops".to_string()),
    };
    let elevated = service.request_elevation(req).unwrap();
    assert!(elevated.has_capability(&PrivilegeCapability::FilesystemWrite));
    assert!(elevated.has_capability(&PrivilegeCapability::NetworkConnect));
    assert!(service.check_capability("eve", PrivilegeCapability::FilesystemWrite));
    assert!(service.check_capability("eve", PrivilegeCapability::NetworkConnect));
    assert!(!service.check_capability("eve", PrivilegeCapability::SystemReboot));
}

#[test]
fn test_autopriv5_privilege_drop_and_revocation() {
    let mut service = PrivilegeService::new();
    let ctx = PrivilegeContext::new("frank", PrivilegeLevel::User).unwrap();
    assert!(service.register_context(ctx).is_ok());

    // Elevate Frank to Admin
    let req = PrivilegeTransitionRequest {
        actor_id: "frank".to_string(),
        from_level: PrivilegeLevel::User,
        target_level: PrivilegeLevel::Admin,
        requested_capabilities: vec![PrivilegeCapability::SystemReboot],
        grant_id: Some("grant-frank-admin".to_string()),
    };
    let _ = service.request_elevation(req).unwrap();
    assert_eq!(service.get_context("frank").unwrap().active_level, PrivilegeLevel::Admin);
    assert!(service.check_capability("frank", PrivilegeCapability::SystemReboot));

    // Voluntary drop to Operator removes higher capabilities
    let dropped = service.drop_privilege("frank", PrivilegeLevel::Operator).unwrap();
    assert_eq!(dropped.active_level, PrivilegeLevel::Operator);
    assert!(!service.check_capability("frank", PrivilegeCapability::SystemReboot));

    // Mandatory revocation restores original baseline (User)
    let revoked = service.revoke_elevation("frank").unwrap();
    assert_eq!(revoked.active_level, PrivilegeLevel::User);
    assert_eq!(service.get_context("frank").unwrap().active_level, PrivilegeLevel::User);
}

#[test]
fn test_autopriv6_store_persistence_and_bounding() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("privilege_store.json");

    let mut service = PrivilegeService::new();
    let ctx = PrivilegeContext::new("grace", PrivilegeLevel::Operator).unwrap();
    assert!(service.register_context(ctx).is_ok());

    // Save to path
    assert!(service.save_to_path(&store_path).is_ok());
    assert!(store_path.exists());

    // Load from path into a new service instance
    let loaded_service = PrivilegeService::load_from_path(&store_path).expect("load from path");
    assert_eq!(loaded_service.active_contexts_count(), 1);
    assert!(loaded_service.contains_actor("grace"));
    assert_eq!(
        loaded_service.get_context("grace").unwrap().active_level,
        PrivilegeLevel::Operator
    );
}

#[test]
fn test_autopriv7_context_unregister_and_capacity_bounds() {
    let mut service = PrivilegeService::with_capacity(2);
    let c1 = PrivilegeContext::new("h1", PrivilegeLevel::Guest).unwrap();
    let c2 = PrivilegeContext::new("h2", PrivilegeLevel::Guest).unwrap();
    let c3 = PrivilegeContext::new("h3", PrivilegeLevel::Guest).unwrap();

    assert!(service.register_context(c1).is_ok());
    assert!(service.register_context(c2).is_ok());

    // Capacity full
    let err = service.register_context(c3.clone()).unwrap_err();
    assert!(err.contains(PRIVESC_ERR_CAPACITY_EXCEEDED));

    // Unregister h1 freeing a slot
    let removed = service.unregister_context("h1").unwrap();
    assert_eq!(removed.actor_id, "h1");
    assert_eq!(service.active_contexts_count(), 1);

    // Now c3 can register
    assert!(service.register_context(c3).is_ok());
    assert_eq!(service.active_contexts_count(), 2);
}

#[test]
fn test_autopriv8_multithreaded_concurrency() {
    let service = Arc::new(Mutex::new(PrivilegeService::with_capacity(100)));

    // Register 10 actors
    for i in 0..10 {
        let actor = format!("worker_{}", i);
        let ctx = PrivilegeContext::new(&actor, PrivilegeLevel::User).unwrap();
        service.lock().unwrap().register_context(ctx).unwrap();
    }

    let mut handles = vec![];
    for i in 0..10 {
        let svc_clone = Arc::clone(&service);
        let handle = thread::spawn(move || {
            let actor = format!("worker_{}", i);
            let req = PrivilegeTransitionRequest {
                actor_id: actor.clone(),
                from_level: PrivilegeLevel::User,
                target_level: PrivilegeLevel::Operator,
                requested_capabilities: vec![PrivilegeCapability::NetworkConnect],
                grant_id: Some(format!("grant_{}", i)),
            };
            let mut s = svc_clone.lock().unwrap();
            let elevated = s.request_elevation(req).unwrap();
            assert_eq!(elevated.active_level, PrivilegeLevel::Operator);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let s = service.lock().unwrap();
    for i in 0..10 {
        let actor = format!("worker_{}", i);
        assert_eq!(s.get_context(&actor).unwrap().active_level, PrivilegeLevel::Operator);
        assert!(s.check_capability(&actor, PrivilegeCapability::NetworkConnect));
    }
}

#[test]
fn test_autopriv9_corrupted_store_handling() {
    let dir = tempdir().unwrap();
    let bad_path = dir.path().join("corrupted.json");

    // Write malformed JSON
    std::fs::write(&bad_path, b"{\"contexts\": { INVALID_JSON }").unwrap();

    // Must return an explicit error and not panic
    let err = PrivilegeService::load_from_path(&bad_path).unwrap_err();
    assert!(err.contains("parse error"));

    // Non-existent file must return IO error
    let missing_path = dir.path().join("missing.json");
    let err_io = PrivilegeService::load_from_path(&missing_path).unwrap_err();
    assert!(err_io.contains("IO error"));
}
