//! Integration Tests for Secrets Handling Data Model (T-02606).

use aiosh_core::privilege_data_model::{PrivilegeContext, PrivilegeLevel};
use aiosh_core::secret_data_model::*;

#[test]
fn test_secret_metadata_json_serialization_roundtrip() {
    let mut meta = SecretMetadata::new(
        "sec_stripe_api_key",
        "Stripe Live Secret Key",
        SecretKind::ApiKey,
        SecretScope::Environment("production".into()),
        "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca7",
    ).unwrap();
    meta.description = "Primary payment gateway secret token".into();
    meta.labels.insert("service".into(), "billing".into());
    meta.labels.insert("criticality".into(), "tier-0".into());

    let json_str = serde_json::to_string_pretty(&meta).expect("serialize metadata");
    assert!(json_str.contains("sec_stripe_api_key"));
    assert!(json_str.contains("Stripe Live Secret Key"));
    assert!(json_str.contains("api_key"));
    assert!(json_str.contains("production"));

    let deserialized: SecretMetadata = serde_json::from_str(&json_str).expect("deserialize metadata");
    assert_eq!(meta, deserialized);
}

#[test]
fn test_secret_scope_and_privilege_context_integration() {
    // A regular user actor context
    let user_ctx = PrivilegeContext::new("developer_bob", PrivilegeLevel::User).unwrap();
    // An operator actor context
    let operator_ctx = PrivilegeContext::new("ops_alice", PrivilegeLevel::Operator).unwrap();

    let user_secret = SecretScope::Actor("developer_bob".into());
    let other_user_secret = SecretScope::Actor("developer_charlie".into());
    let prod_secret = SecretScope::Environment("production".into());

    // User scope matches their own actor ID
    assert_eq!(user_secret, SecretScope::Actor(user_ctx.actor_id.clone()));
    assert_ne!(other_user_secret, SecretScope::Actor(user_ctx.actor_id.clone()));

    // Operator checking scope permissions
    let global_scope = SecretScope::Global;
    assert!(global_scope.allows(&user_secret));
    assert!(global_scope.allows(&prod_secret));
    assert!(operator_ctx.active_level >= user_ctx.active_level);
}
