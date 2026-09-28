//! Integration tests for Privilege Escalation Prevention Data Model (`T-02506`).

use aiosh_core::privilege_data_model::{
    PrivilegeCapability, PrivilegeContext, PrivilegeEscalationVerdict, PrivilegeLevel,
    PrivilegeTransitionRequest,
};
use serde_json::json;

#[test]
fn test_privilege_data_model_wire_compatibility() {
    let mut ctx = PrivilegeContext::new("sys_operator", PrivilegeLevel::Operator).expect("valid context");
    ctx.add_capability(PrivilegeCapability::ProcessSpawn).unwrap();
    ctx.add_capability(PrivilegeCapability::NetworkListen).unwrap();

    let serialized = serde_json::to_string(&ctx).expect("serialization succeeds");
    let val: serde_json::Value = serde_json::from_str(&serialized).expect("parse json");

    assert_eq!(val["actor_id"], "sys_operator");
    assert_eq!(val["active_level"], "operator");
    assert_eq!(val["is_elevation_active"], false);

    let deserialized: PrivilegeContext = serde_json::from_str(&serialized).expect("deserialization succeeds");
    assert_eq!(deserialized, ctx);
}

#[test]
fn test_transition_request_json_interop() {
    let raw_payload = json!({
        "actor_id": "service_worker",
        "from_level": "guest",
        "target_level": "user",
        "requested_capabilities": ["process_spawn", "filesystem_write"],
        "grant_id": "PEP-GRANT-0042"
    });

    let req: PrivilegeTransitionRequest = serde_json::from_value(raw_payload).expect("deserializes from json");
    assert_eq!(req.actor_id, "service_worker");
    assert_eq!(req.from_level, PrivilegeLevel::Guest);
    assert_eq!(req.target_level, PrivilegeLevel::User);
    assert_eq!(req.requested_capabilities.len(), 2);
    assert_eq!(req.grant_id.as_deref(), Some("PEP-GRANT-0042"));

    // Verdict evaluation with grant should be Allowed
    let verdict = req.evaluate();
    assert_eq!(verdict, PrivilegeEscalationVerdict::Allowed);
}

#[test]
fn test_privilege_escalation_flow_integration() {
    // 1. Initial guest context
    let mut ctx = PrivilegeContext::guest();
    assert_eq!(ctx.active_level, PrivilegeLevel::Guest);
    assert!(!ctx.is_elevation_active);

    // 2. Request elevation to Operator tier without grant
    let unauth_req = PrivilegeTransitionRequest {
        actor_id: ctx.actor_id.clone(),
        from_level: ctx.active_level,
        target_level: PrivilegeLevel::Operator,
        requested_capabilities: vec![PrivilegeCapability::NetworkListen],
        grant_id: None,
    };
    match unauth_req.evaluate() {
        PrivilegeEscalationVerdict::GrantRequired { reason } => {
            assert!(reason.contains("PEP authorization grant"));
        }
        other => panic!("expected GrantRequired, got {:?}", other),
    }

    // 3. Obtain simulated PEP grant and apply elevation
    let grant_id = "GRANT-OP-2026-ALPHA";
    assert!(ctx.elevate_with_grant(PrivilegeLevel::Operator, grant_id).is_ok());
    assert_eq!(ctx.active_level, PrivilegeLevel::Operator);
    assert!(ctx.is_elevation_active);
    assert!(ctx.add_capability(PrivilegeCapability::NetworkListen).is_ok());

    // 4. Drop privilege back to User
    assert!(ctx.drop_to_level(PrivilegeLevel::User).is_ok());
    assert_eq!(ctx.active_level, PrivilegeLevel::User);
    assert!(!ctx.is_elevation_active);
    assert!(!ctx.has_capability(&PrivilegeCapability::NetworkListen));
}
