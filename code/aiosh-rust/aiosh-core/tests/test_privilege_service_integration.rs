//! Integration test suite for Privilege Escalation Prevention Core Service (`T-02516`).

use aiosh_core::privilege_data_model::{
    PrivilegeCapability, PrivilegeContext, PrivilegeLevel, PrivilegeTransitionRequest,
};
use aiosh_core::privilege_service::PrivilegeService;
use std::sync::{Arc, RwLock};
use std::thread;

#[test]
fn test_service_concurrent_access() {
    let service = Arc::new(RwLock::new(PrivilegeService::with_capacity(50)));

    // Register 10 distinct actors concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let srv = Arc::clone(&service);
        let handle = thread::spawn(move || {
            let actor_id = format!("concurrent_actor_{}", i);
            let ctx = PrivilegeContext::new(actor_id, PrivilegeLevel::User).unwrap();
            let mut guard = srv.write().unwrap();
            guard.register_context(ctx).unwrap();
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    {
        let guard = service.read().unwrap();
        assert_eq!(guard.active_contexts_count(), 10);
    }

    // Elevate 10 actors concurrently
    let mut elev_handles = vec![];
    for i in 0..10 {
        let srv = Arc::clone(&service);
        let handle = thread::spawn(move || {
            let actor_id = format!("concurrent_actor_{}", i);
            let req = PrivilegeTransitionRequest {
                actor_id: actor_id.clone(),
                from_level: PrivilegeLevel::User,
                target_level: PrivilegeLevel::Operator,
                requested_capabilities: vec![PrivilegeCapability::NetworkListen],
                grant_id: Some(format!("GRANT-THREAD-{}", i)),
            };
            let mut guard = srv.write().unwrap();
            guard.request_elevation(req).unwrap();
        });
        elev_handles.push(handle);
    }

    for h in elev_handles {
        h.join().unwrap();
    }

    // Verify all 10 are elevated to Operator
    let guard = service.read().unwrap();
    for i in 0..10 {
        let actor_id = format!("concurrent_actor_{}", i);
        let ctx = guard.get_context(&actor_id).unwrap();
        assert_eq!(ctx.active_level, PrivilegeLevel::Operator);
        assert!(ctx.is_elevation_active);
        assert!(guard.check_capability(&actor_id, PrivilegeCapability::NetworkListen));
    }
}

#[test]
fn test_service_snapshot_serde_roundtrip() {
    let mut srv = PrivilegeService::with_capacity(100);
    let mut ctx1 = PrivilegeContext::new("daemon_node", PrivilegeLevel::Operator).unwrap();
    ctx1.add_capability(PrivilegeCapability::NetworkListen).unwrap();
    srv.register_context(ctx1).unwrap();

    let json_str = serde_json::to_string(&srv).expect("serialize service snapshot");
    assert!(json_str.contains("daemon_node"));

    let restored: PrivilegeService = serde_json::from_str(&json_str).expect("deserialize service snapshot");
    assert_eq!(restored.active_contexts_count(), 1);
    assert!(restored.contains_actor("daemon_node"));
    assert!(restored.check_capability("daemon_node", PrivilegeCapability::NetworkListen));
}
