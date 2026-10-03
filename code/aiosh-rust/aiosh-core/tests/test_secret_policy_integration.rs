//! Integration Tests for Secrets Handling Security Policy (T-02666).

use tempfile::tempdir;

use aiosh_core::secret_data_model::{SecretEntry, SecretKind, SecretScope};
use aiosh_core::secret_policy::{
    SecretPolicyMode, SecretSecurityPolicy,
    SECPOL_ERR_GLOBAL_DISALLOWED, SECPOL_ERR_KIND_PROHIBITED, SECPOL_ERR_PAYLOAD_TOO_LARGE,
};
use aiosh_core::secret_service::SecretService;

#[test]
fn test_secret_policy_end_to_end_service_integration() {
    let mut srv = SecretService::new();

    // 1. Configure restrictive security policy
    let mut policy = SecretSecurityPolicy::default();
    policy.disallow_global_secrets = true;
    policy.prohibited_kinds = vec![SecretKind::PrivateKey];
    policy.max_payload_bytes = 64;
    policy.mode = SecretPolicyMode::Enforcing;
    srv.set_policy(policy);

    assert_eq!(srv.policy().disallow_global_secrets, true);
    assert_eq!(srv.policy().max_payload_bytes, 64);

    // 2. Storing global secret must be rejected fail-closed
    let global_sec = SecretEntry::new(
        "sec_global_fail",
        "Global Secret",
        SecretKind::ApiKey,
        SecretScope::Global,
        b"global_val",
    ).unwrap();
    let err_global = srv.store_secret(global_sec).unwrap_err();
    assert!(err_global.contains(SECPOL_ERR_GLOBAL_DISALLOWED), "Got: {}", err_global);

    // 3. Storing prohibited kind (PrivateKey) must be rejected
    let privkey_sec = SecretEntry::new(
        "sec_privkey_fail",
        "Private Key Secret",
        SecretKind::PrivateKey,
        SecretScope::Actor("worker".into()),
        b"privkey_val",
    ).unwrap();
    let err_kind = srv.store_secret(privkey_sec).unwrap_err();
    assert!(err_kind.contains(SECPOL_ERR_KIND_PROHIBITED), "Got: {}", err_kind);

    // 4. Storing oversized payload (> 64 bytes) must be rejected
    let large_sec = SecretEntry::new(
        "sec_large_fail",
        "Large Secret",
        SecretKind::ApiKey,
        SecretScope::Actor("worker".into()),
        &[0xAA; 80],
    ).unwrap();
    let err_large = srv.store_secret(large_sec).unwrap_err();
    assert!(err_large.contains(SECPOL_ERR_PAYLOAD_TOO_LARGE), "Got: {}", err_large);

    // 5. Storing compliant secret must succeed
    let compliant_sec = SecretEntry::new(
        "sec_compliant",
        "Compliant Secret",
        SecretKind::ApiKey,
        SecretScope::Actor("worker".into()),
        b"compliant_val_32_bytes_long_1234",
    ).unwrap();
    assert!(srv.store_secret(compliant_sec).is_ok());
    assert_eq!(srv.len(), 1);

    // 6. Rotating secret with oversized payload must be rejected
    let err_rotate = srv.rotate_secret("sec_compliant", &[0xBB; 100]).unwrap_err();
    assert!(err_rotate.contains(SECPOL_ERR_PAYLOAD_TOO_LARGE), "Got: {}", err_rotate);

    // 7. Rotating secret with compliant payload succeeds
    assert!(srv.rotate_secret("sec_compliant", b"new_rotated_compliant_val").is_ok());

    // 8. Transition policy to Permissive mode
    srv.policy_mut().mode = SecretPolicyMode::Permissive;

    // Permissive mode allows global secret and prohibited kind
    let perm_global = SecretEntry::new(
        "sec_perm_global",
        "Permissive Global",
        SecretKind::ApiKey,
        SecretScope::Global,
        b"val",
    ).unwrap();
    assert!(srv.store_secret(perm_global).is_ok());

    let perm_privkey = SecretEntry::new(
        "sec_perm_privkey",
        "Permissive Privkey",
        SecretKind::PrivateKey,
        SecretScope::Actor("worker".into()),
        b"val",
    ).unwrap();
    assert!(srv.store_secret(perm_privkey).is_ok());

    // Rotating with oversized payload permitted in permissive mode
    assert!(srv.rotate_secret("sec_compliant", &[0xCC; 120]).is_ok());

    // 9. Transition policy to Disabled mode
    srv.set_policy(SecretSecurityPolicy {
        mode: SecretPolicyMode::Disabled,
        ..Default::default()
    });
    assert_eq!(srv.policy().mode, SecretPolicyMode::Disabled);
}

#[test]
fn test_secret_policy_persistence_and_file_integration() {
    let dir = tempdir().unwrap();
    let pol_file = dir.path().join("active_policy.json");

    let mut policy = SecretSecurityPolicy::default();
    policy.disallow_global_secrets = true;
    policy.max_payload_bytes = 128;
    assert!(policy.save_to_path(&pol_file).is_ok());

    let loaded = SecretSecurityPolicy::load_from_path(&pol_file).unwrap();
    let mut srv = SecretService::new();
    srv.set_policy(loaded);

    // Enforce loaded policy
    let global_sec = SecretEntry::new(
        "sec_pol_global",
        "Global Secret",
        SecretKind::ApiKey,
        SecretScope::Global,
        b"val",
    ).unwrap();
    assert!(srv.store_secret(global_sec).is_err());
}
