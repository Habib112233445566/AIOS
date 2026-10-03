//! Unit Tests for Secrets Handling Security Policy (T-02665).

use std::fs;
use tempfile::tempdir;

use aiosh_core::secret_data_model::{SecretEntry, SecretKind, SecretScope};
use aiosh_core::secret_policy::{
    SecretPolicyMode, SecretPolicyVerdict, SecretSecurityPolicy,
    SECPOL_ERR_DENIED, SECPOL_ERR_EXPOSE_REQUIRED, SECPOL_ERR_GLOBAL_DISALLOWED,
    SECPOL_ERR_KIND_PROHIBITED, SECPOL_ERR_PAYLOAD_TOO_LARGE, SECPOL_ERR_VALIDATION,
};

fn make_test_entry(id: &str, kind: SecretKind, scope: SecretScope, val: &[u8]) -> SecretEntry {
    SecretEntry::new(id, &format!("Name {}", id), kind, scope, val).expect("valid entry")
}

#[test]
fn test_policy_defaults_and_validation() {
    let policy = SecretSecurityPolicy::default();
    assert_eq!(policy.mode, SecretPolicyMode::Enforcing);
    assert_eq!(policy.version, "1.0.0");
    assert_eq!(policy.disallow_global_secrets, false);
    assert_eq!(policy.max_payload_bytes, 65536);
    assert_eq!(policy.require_expose_flag, true);
    assert_eq!(policy.prohibited_kinds.len(), 0);
    assert!(policy.validate().is_ok());

    // Invalid version: empty
    let mut bad_ver = policy.clone();
    bad_ver.version = "   ".into();
    assert!(bad_ver.validate().is_err());

    // Invalid version: not starting with 1.
    let mut bad_ver2 = policy.clone();
    bad_ver2.version = "2.0.0".into();
    assert!(bad_ver2.validate().is_err());

    // Invalid payload bound: 0
    let mut bad_payload = policy.clone();
    bad_payload.max_payload_bytes = 0;
    assert!(bad_payload.validate().is_err());

    // Invalid payload bound: > 1MB
    let mut bad_payload2 = policy.clone();
    bad_payload2.max_payload_bytes = 2 * 1024 * 1024;
    assert!(bad_payload2.validate().is_err());

    // Invalid lifetime bound: 0
    let mut bad_life = policy.clone();
    bad_life.max_lifetime_seconds = 0;
    assert!(bad_life.validate().is_err());

    // Invalid lifetime bound: > 1 year
    let mut bad_life2 = policy.clone();
    bad_life2.max_lifetime_seconds = 86400 * 366;
    assert!(bad_life2.validate().is_err());
}

#[test]
fn test_policy_store_evaluation() {
    let mut policy = SecretSecurityPolicy::default();
    policy.disallow_global_secrets = true;
    policy.prohibited_kinds = vec![SecretKind::PrivateKey];
    policy.max_payload_bytes = 100;

    let global_entry = make_test_entry("sec_global", SecretKind::ApiKey, SecretScope::Global, b"global_secret");
    let privkey_entry = make_test_entry("sec_key", SecretKind::PrivateKey, SecretScope::Actor("actor1".into()), b"privkey_val");
    let large_entry = make_test_entry("sec_large", SecretKind::ApiKey, SecretScope::Actor("actor1".into()), &[42u8; 150]);
    let ok_entry = make_test_entry("sec_ok", SecretKind::ApiKey, SecretScope::Actor("actor1".into()), b"valid_key");

    // 1. Enforcing mode
    policy.mode = SecretPolicyMode::Enforcing;
    match policy.evaluate_store(&global_entry) {
        SecretPolicyVerdict::Deny { code, reason } => {
            assert_eq!(code, SECPOL_ERR_GLOBAL_DISALLOWED);
            assert!(reason.contains("global"));
        }
        other => panic!("expected Deny for global secret, got {:?}", other),
    }

    match policy.evaluate_store(&privkey_entry) {
        SecretPolicyVerdict::Deny { code, reason } => {
            assert_eq!(code, SECPOL_ERR_KIND_PROHIBITED);
            assert!(reason.contains("private_key"));
        }
        other => panic!("expected Deny for prohibited kind, got {:?}", other),
    }

    match policy.evaluate_store(&large_entry) {
        SecretPolicyVerdict::Deny { code, reason } => {
            assert_eq!(code, SECPOL_ERR_PAYLOAD_TOO_LARGE);
            assert!(reason.contains("exceeds"));
        }
        other => panic!("expected Deny for large payload, got {:?}", other),
    }

    assert_eq!(policy.evaluate_store(&ok_entry), SecretPolicyVerdict::Permit);

    // 2. Permissive mode
    policy.mode = SecretPolicyMode::Permissive;
    match policy.evaluate_store(&global_entry) {
        SecretPolicyVerdict::PermitWithWarning { warning } => {
            assert!(warning.contains("global"));
        }
        other => panic!("expected PermitWithWarning, got {:?}", other),
    }

    match policy.evaluate_store(&privkey_entry) {
        SecretPolicyVerdict::PermitWithWarning { warning } => {
            assert!(warning.contains("private_key"));
        }
        other => panic!("expected PermitWithWarning, got {:?}", other),
    }

    match policy.evaluate_store(&large_entry) {
        SecretPolicyVerdict::PermitWithWarning { warning } => {
            assert!(warning.contains("exceeds"));
        }
        other => panic!("expected PermitWithWarning, got {:?}", other),
    }

    // 3. Disabled mode
    policy.mode = SecretPolicyMode::Disabled;
    assert_eq!(policy.evaluate_store(&global_entry), SecretPolicyVerdict::Permit);
    assert_eq!(policy.evaluate_store(&privkey_entry), SecretPolicyVerdict::Permit);
    assert_eq!(policy.evaluate_store(&large_entry), SecretPolicyVerdict::Permit);
}

#[test]
fn test_policy_get_evaluation() {
    let mut policy = SecretSecurityPolicy::default();
    policy.require_expose_flag = true;

    let target_scope = SecretScope::Actor("agent_alpha".into());
    let entry = make_test_entry("sec_alpha", SecretKind::ApiKey, target_scope, b"secret_value");

    let caller_authorized = SecretScope::Actor("agent_alpha".into());
    let caller_unauthorized = SecretScope::Actor("agent_beta".into());

    // 1. Enforcing mode: unauthorized scope denied
    policy.mode = SecretPolicyMode::Enforcing;
    match policy.evaluate_get(&entry, &caller_unauthorized, true) {
        SecretPolicyVerdict::Deny { code, .. } => assert_eq!(code, SECPOL_ERR_DENIED),
        other => panic!("expected Deny, got {:?}", other),
    }

    // 2. Enforcing mode: authorized scope without expose flag denied
    match policy.evaluate_get(&entry, &caller_authorized, false) {
        SecretPolicyVerdict::Deny { code, .. } => assert_eq!(code, SECPOL_ERR_EXPOSE_REQUIRED),
        other => panic!("expected Deny, got {:?}", other),
    }

    // 3. Enforcing mode: authorized with expose flag permitted
    assert_eq!(policy.evaluate_get(&entry, &caller_authorized, true), SecretPolicyVerdict::Permit);

    // 4. Permissive mode: warnings emitted
    policy.mode = SecretPolicyMode::Permissive;
    match policy.evaluate_get(&entry, &caller_unauthorized, true) {
        SecretPolicyVerdict::PermitWithWarning { warning } => assert!(warning.contains("denied")),
        other => panic!("expected PermitWithWarning, got {:?}", other),
    }

    match policy.evaluate_get(&entry, &caller_authorized, false) {
        SecretPolicyVerdict::PermitWithWarning { warning } => assert!(warning.contains("expose")),
        other => panic!("expected PermitWithWarning, got {:?}", other),
    }

    // 5. Disabled mode
    policy.mode = SecretPolicyMode::Disabled;
    assert_eq!(policy.evaluate_get(&entry, &caller_unauthorized, false), SecretPolicyVerdict::Permit);
}

#[test]
fn test_policy_rotate_evaluation() {
    let mut policy = SecretSecurityPolicy::default();
    policy.max_payload_bytes = 100;
    let entry = make_test_entry("sec_rot", SecretKind::ApiKey, SecretScope::Global, b"original");

    // 1. Enforcing
    policy.mode = SecretPolicyMode::Enforcing;
    assert_eq!(policy.evaluate_rotate(&entry, 80), SecretPolicyVerdict::Permit);
    match policy.evaluate_rotate(&entry, 150) {
        SecretPolicyVerdict::Deny { code, .. } => assert_eq!(code, SECPOL_ERR_PAYLOAD_TOO_LARGE),
        other => panic!("expected Deny, got {:?}", other),
    }

    // 2. Permissive
    policy.mode = SecretPolicyMode::Permissive;
    match policy.evaluate_rotate(&entry, 150) {
        SecretPolicyVerdict::PermitWithWarning { warning } => assert!(warning.contains("exceeds")),
        other => panic!("expected PermitWithWarning, got {:?}", other),
    }

    // 3. Disabled
    policy.mode = SecretPolicyMode::Disabled;
    assert_eq!(policy.evaluate_rotate(&entry, 150), SecretPolicyVerdict::Permit);
}

#[test]
fn test_policy_persistence_and_traversal_rejection() {
    let dir = tempdir().unwrap();
    let pol_path = dir.path().join("policy.json");

    let mut policy = SecretSecurityPolicy::default();
    policy.disallow_global_secrets = true;
    policy.max_payload_bytes = 4096;
    policy.prohibited_kinds = vec![SecretKind::PrivateKey, SecretKind::DatabaseCredential];

    assert!(policy.save_to_path(&pol_path).is_ok());
    assert!(pol_path.exists());

    let loaded = SecretSecurityPolicy::load_from_path(&pol_path).expect("load policy");
    assert_eq!(loaded.disallow_global_secrets, true);
    assert_eq!(loaded.max_payload_bytes, 4096);
    assert_eq!(loaded.prohibited_kinds.len(), 2);

    // Traversal rejection on save
    let bad_save = dir.path().join("../evil_policy.json");
    assert!(policy.save_to_path(&bad_save).is_err());

    // Traversal rejection on load
    assert!(SecretSecurityPolicy::load_from_path(&bad_save).is_err());

    // Oversized policy file (> 64 KiB) rejection
    let big_path = dir.path().join("big_policy.json");
    let big_data = " ".repeat(65 * 1024);
    fs::write(&big_path, big_data).unwrap();
    let err = SecretSecurityPolicy::load_from_path(&big_path).unwrap_err();
    assert!(err.contains(SECPOL_ERR_VALIDATION));
}

#[test]
fn test_policy_env_overrides() {
    std::env::set_var("AIOS_SECRETS_POLICY_MODE", "permissive");
    let pol1 = SecretSecurityPolicy::load_with_env_overrides();
    assert_eq!(pol1.mode, SecretPolicyMode::Permissive);

    std::env::set_var("AIOS_SECRETS_POLICY_MODE", "disabled");
    let pol2 = SecretSecurityPolicy::load_with_env_overrides();
    assert_eq!(pol2.mode, SecretPolicyMode::Disabled);

    std::env::remove_var("AIOS_SECRETS_POLICY_MODE");
    let pol3 = SecretSecurityPolicy::load_with_env_overrides();
    assert_eq!(pol3.mode, SecretPolicyMode::Enforcing);
}
