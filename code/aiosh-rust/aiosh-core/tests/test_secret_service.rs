//! Unit Tests for Secrets Handling Core Service (T-02615).

use tempfile::tempdir;

use aiosh_core::privilege_data_model::{PrivilegeContext, PrivilegeLevel};
use aiosh_core::secret_data_model::*;
use aiosh_core::secret_service::*;

#[test]
fn test_secret_service_store_and_get() {
    let mut service = SecretService::new();
    let entry = SecretEntry::new(
        "sec_anthropic_api",
        "Anthropic API Key",
        SecretKind::ApiKey,
        SecretScope::Global,
        b"sk-ant-api03-testkey12345678",
    ).unwrap();

    service.store_secret(entry).unwrap();
    assert_eq!(service.len(), 1);
    assert!(service.contains("sec_anthropic_api"));

    // Retrieve with global scope
    let val = service.get_secret("sec_anthropic_api", &SecretScope::Global).unwrap();
    assert_eq!(val.as_bytes(), b"sk-ant-api03-testkey12345678");

    // Retrieve non-existent
    assert!(service.get_secret("sec_missing", &SecretScope::Global).is_err());
}

#[test]
fn test_secret_service_scope_denial() {
    let mut service = SecretService::new();
    let entry = SecretEntry::new(
        "sec_agent_private",
        "Agent Private Token",
        SecretKind::OAuthToken,
        SecretScope::Actor("agent_alpha".into()),
        b"token_alpha_secret_999",
    ).unwrap();
    service.store_secret(entry).unwrap();

    // Matching actor scope can read
    let val = service.get_secret("sec_agent_private", &SecretScope::Actor("agent_alpha".into())).unwrap();
    assert_eq!(val.as_bytes(), b"token_alpha_secret_999");

    // Mismatched actor scope denied
    let err = service.get_secret("sec_agent_private", &SecretScope::Actor("agent_beta".into())).unwrap_err();
    assert!(err.contains(SECSVC_ERR_ACCESS_DENIED));

    // Global scope can read
    let val_global = service.get_secret("sec_agent_private", &SecretScope::Global).unwrap();
    assert_eq!(val_global.as_bytes(), b"token_alpha_secret_999");
}

#[test]
fn test_secret_service_state_inaccessible() {
    let mut service = SecretService::new();
    let entry = SecretEntry::new(
        "sec_revoke_test",
        "Revocable Secret",
        SecretKind::Generic,
        SecretScope::Global,
        b"will_be_revoked",
    ).unwrap();
    service.store_secret(entry).unwrap();

    service.revoke_secret("sec_revoke_test").unwrap();
    let err = service.get_secret("sec_revoke_test", &SecretScope::Global).unwrap_err();
    assert!(err.contains(SECSVC_ERR_INACCESSIBLE));
}

#[test]
fn test_secret_service_list_metadata_filtering() {
    let mut service = SecretService::new();
    service.store_secret(SecretEntry::new("sec_1", "Key 1", SecretKind::ApiKey, SecretScope::Global, b"val1").unwrap()).unwrap();
    service.store_secret(SecretEntry::new("sec_2", "Cert 2", SecretKind::TlsCertificate, SecretScope::Environment("prod".into()), b"val2").unwrap()).unwrap();
    service.store_secret(SecretEntry::new("sec_3", "Key 3", SecretKind::ApiKey, SecretScope::Environment("dev".into()), b"val3").unwrap()).unwrap();

    // List all
    let all = service.list_metadata(None, None);
    assert_eq!(all.len(), 3);

    // Filter by kind
    let keys = service.list_metadata(Some(SecretKind::ApiKey), None);
    assert_eq!(keys.len(), 2);

    // Filter by scope
    let prod_filter = SecretScope::Environment("prod".into());
    let prod_meta = service.list_metadata(None, Some(&prod_filter));
    assert_eq!(prod_meta.len(), 1);
    assert_eq!(prod_meta[0].id, "sec_2");
}

#[test]
fn test_secret_service_rotate_and_revoke() {
    let mut service = SecretService::new();
    let entry = SecretEntry::new("sec_rot", "Rotation Test", SecretKind::SymmetricKey, SecretScope::Global, b"first_val").unwrap();
    service.store_secret(entry).unwrap();

    let meta_v1 = service.get_metadata("sec_rot").unwrap();
    assert_eq!(meta_v1.version, 1);

    service.rotate_secret("sec_rot", b"second_rotated_val").unwrap();
    let meta_v2 = service.get_metadata("sec_rot").unwrap();
    assert_eq!(meta_v2.version, 2);
    assert_ne!(meta_v1.fingerprint, meta_v2.fingerprint);

    let val = service.get_secret("sec_rot", &SecretScope::Global).unwrap();
    assert_eq!(val.as_bytes(), b"second_rotated_val");
}

#[test]
fn test_secret_service_atomic_persistence_and_reload() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("vault.json");

    let mut service = SecretService::new();
    service.store_secret(SecretEntry::new("sec_p1", "Persist 1", SecretKind::ApiKey, SecretScope::Global, b"secret_payload_persist").unwrap()).unwrap();
    service.save_to_path(&store_path).unwrap();

    assert!(store_path.exists());

    let loaded = SecretService::load_from_path(&store_path).unwrap();
    assert_eq!(loaded.len(), 1);
    let val = loaded.get_secret("sec_p1", &SecretScope::Global).unwrap();
    assert_eq!(val.as_bytes(), b"secret_payload_persist");
}

#[test]
fn test_secret_service_privilege_context_gate() {
    let mut service = SecretService::new();
    service.store_secret(SecretEntry::new("sec_env", "Env Secret", SecretKind::DatabaseCredential, SecretScope::Environment("prod".into()), b"db_secret").unwrap()).unwrap();
    service.store_secret(SecretEntry::new("sec_user", "User Secret", SecretKind::OAuthToken, SecretScope::Actor("alice".into()), b"token_alice").unwrap()).unwrap();

    let guest = PrivilegeContext::new("guest_user", PrivilegeLevel::Guest).unwrap();
    let alice = PrivilegeContext::new("alice", PrivilegeLevel::User).unwrap();
    let bob = PrivilegeContext::new("bob", PrivilegeLevel::User).unwrap();
    let op = PrivilegeContext::new("operator_ops", PrivilegeLevel::Operator).unwrap();
    let admin = PrivilegeContext::new("admin_root", PrivilegeLevel::Admin).unwrap();

    // Guest denied
    assert!(service.get_secret_with_privilege("sec_user", &guest).is_err());

    // Alice can read her own secret
    assert!(service.get_secret_with_privilege("sec_user", &alice).is_ok());

    // Bob cannot read Alice's secret
    assert!(service.get_secret_with_privilege("sec_user", &bob).is_err());

    // Alice (User) cannot read Environment secret
    assert!(service.get_secret_with_privilege("sec_env", &alice).is_err());

    // Operator can read Environment secret
    assert!(service.get_secret_with_privilege("sec_env", &op).is_ok());

    // Admin can read all
    assert!(service.get_secret_with_privilege("sec_env", &admin).is_ok());
    assert!(service.get_secret_with_privilege("sec_user", &admin).is_ok());
}

#[test]
fn test_secret_service_path_traversal_rejection() {
    let bad_path = std::path::Path::new("../forbidden/vault.json");
    let service = SecretService::new();
    let err = service.save_to_path(bad_path).unwrap_err();
    assert!(err.contains(SECSVC_ERR_PATH_TRAVERSAL));

    let err_load = SecretService::load_from_path(bad_path).unwrap_err();
    assert!(err_load.contains(SECSVC_ERR_PATH_TRAVERSAL));
}

#[test]
fn test_secret_service_capacity_boundary() {
    let mut service = SecretService::new();
    for i in 0..MAX_SECRETS_VAULT_CAPACITY {
        let entry = SecretEntry::new(
            &format!("sec_{}", i),
            "Bench",
            SecretKind::Generic,
            SecretScope::Global,
            b"val",
        ).unwrap();
        service.store_secret(entry).unwrap();
    }
    assert_eq!(service.len(), MAX_SECRETS_VAULT_CAPACITY);

    // 1025th entry rejected
    let extra = SecretEntry::new(
        "sec_overflow",
        "Overflow",
        SecretKind::Generic,
        SecretScope::Global,
        b"val",
    ).unwrap();
    let err = service.store_secret(extra).unwrap_err();
    assert!(err.contains(SECSVC_ERR_CAPACITY_EXCEEDED));
}

