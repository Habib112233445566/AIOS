//! Integration Tests for Secrets Handling Core Service (T-02616).

use tempfile::tempdir;

use aiosh_core::privilege_data_model::{PrivilegeContext, PrivilegeLevel};
use aiosh_core::secret_data_model::*;
use aiosh_core::secret_service::*;

#[test]
fn test_secret_service_multi_actor_vault_lifecycle() {
    let dir = tempdir().unwrap();
    let vault_file = dir.path().join("production_vault.json");

    let mut vault = SecretService::new();

    // 1. Register multiple diverse secrets
    let s1 = SecretEntry::new(
        "stripe_sk_live",
        "Stripe Live Secret Key",
        SecretKind::ApiKey,
        SecretScope::Environment("production".into()),
        b"sk_live_stripe_99999999",
    ).unwrap();

    let s2 = SecretEntry::new(
        "agent_db_creds",
        "Agent Database Credentials",
        SecretKind::DatabaseCredential,
        SecretScope::Actor("db_agent".into()),
        b"postgres://db_agent:secure_pass_123@db.prod:5432/main",
    ).unwrap();

    let s3 = SecretEntry::new(
        "jwt_signing_key",
        "JWT HMAC Symmetric Key",
        SecretKind::SymmetricKey,
        SecretScope::Global,
        b"super_secret_jwt_symmetric_key_256",
    ).unwrap();

    vault.store_secret(s1).unwrap();
    vault.store_secret(s2).unwrap();
    vault.store_secret(s3).unwrap();
    assert_eq!(vault.len(), 3);

    // 2. Persist to disk atomically
    vault.save_to_path(&vault_file).unwrap();

    // 3. Reload into fresh service instance
    let mut restored_vault = SecretService::load_from_path(&vault_file).unwrap();
    assert_eq!(restored_vault.len(), 3);

    // 4. Test multi-actor authorization
    let db_agent = PrivilegeContext::new("db_agent", PrivilegeLevel::User).unwrap();
    let web_agent = PrivilegeContext::new("web_agent", PrivilegeLevel::User).unwrap();
    let ops_user = PrivilegeContext::new("devops_lead", PrivilegeLevel::Operator).unwrap();

    // db_agent can access its own secret
    let creds = restored_vault.get_secret_with_privilege("agent_db_creds", &db_agent).unwrap();
    assert_eq!(creds.as_bytes(), b"postgres://db_agent:secure_pass_123@db.prod:5432/main");

    // web_agent is denied access to db_agent's secret
    assert!(restored_vault.get_secret_with_privilege("agent_db_creds", &web_agent).is_err());

    // ops_user can access production environment secret
    let stripe_key = restored_vault.get_secret_with_privilege("stripe_sk_live", &ops_user).unwrap();
    assert_eq!(stripe_key.as_bytes(), b"sk_live_stripe_99999999");

    // 5. Rotate production secret and verify updated persistence
    restored_vault.rotate_secret("stripe_sk_live", b"sk_live_stripe_rotated_8888").unwrap();
    restored_vault.save_to_path(&vault_file).unwrap();

    let reloaded_v2 = SecretService::load_from_path(&vault_file).unwrap();
    let meta_v2 = reloaded_v2.get_metadata("stripe_sk_live").unwrap();
    assert_eq!(meta_v2.version, 2);
    assert_eq!(meta_v2.state, SecretState::Rotated);
}
