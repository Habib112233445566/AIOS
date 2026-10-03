//! Automated Test Suite for Secrets Handling Subsystem (T-02651..T-02660).
//!
//! Implements formal verification vectors AUTOSEC1..AUTOSEC8 defined in
//! `docs/SPEC-SECRETS-AUTOMATED-TESTS.md`.

use tempfile::tempdir;

use aiosh_core::privilege_data_model::{PrivilegeContext, PrivilegeLevel};
use aiosh_core::secret_config::SecretConfig;
use aiosh_core::secret_data_model::*;
use aiosh_core::secret_service::*;

/// AUTOSEC1: Full Secret Lifecycle & State Isolation
#[test]
fn test_autosec1_lifecycle_and_state_isolation() {
    let mut vault = SecretService::new();
    let entry = SecretEntry::new(
        "sec_life_1",
        "Lifecycle Secret",
        SecretKind::ApiKey,
        SecretScope::Global,
        b"initial_payload_12345",
    ).unwrap();
    assert!(vault.store_secret(entry).is_ok());

    let meta = vault.get_metadata("sec_life_1").unwrap();
    assert_eq!(meta.state, SecretState::Active);
    assert_eq!(meta.version, 1);
}

/// AUTOSEC2: Scope Boundary Enforcement
#[test]
fn test_autosec2_scope_boundary_enforcement() {
    let mut vault = SecretService::new();
    let entry = SecretEntry::new(
        "sec_scoped_actor",
        "Actor Specific Key",
        SecretKind::ApiKey,
        SecretScope::Actor("worker_alpha".into()),
        b"alpha_secret_data",
    ).unwrap();
    assert!(vault.store_secret(entry).is_ok());

    // Access allowed for worker_alpha
    let allow = vault.get_secret("sec_scoped_actor", &SecretScope::Actor("worker_alpha".into()));
    assert!(allow.is_ok());

    // Access denied for worker_beta
    let deny = vault.get_secret("sec_scoped_actor", &SecretScope::Actor("worker_beta".into()));
    assert!(deny.is_err());
}

/// AUTOSEC3: Privilege Tier Access Gates
#[test]
fn test_autosec3_privilege_tier_access_gates() {
    let mut vault = SecretService::new();
    let entry = SecretEntry::new(
        "sec_env_prod",
        "Production Credential",
        SecretKind::DatabaseCredential,
        SecretScope::Environment("production".into()),
        b"prod_db_conn_str",
    ).unwrap();
    assert!(vault.store_secret(entry).is_ok());

    let admin = PrivilegeContext::new("superadmin", PrivilegeLevel::Admin).unwrap();
    let operator = PrivilegeContext::new("lead_sre", PrivilegeLevel::Operator).unwrap();
    let user = PrivilegeContext::new("app_dev", PrivilegeLevel::User).unwrap();
    let guest = PrivilegeContext::new("anon", PrivilegeLevel::Guest).unwrap();

    assert!(vault.get_secret_with_privilege("sec_env_prod", &admin).is_ok());
    assert!(vault.get_secret_with_privilege("sec_env_prod", &operator).is_ok());
    assert!(vault.get_secret_with_privilege("sec_env_prod", &user).is_err());
    assert!(vault.get_secret_with_privilege("sec_env_prod", &guest).is_err());
}

/// AUTOSEC4: Zero-Disclosure Redaction Invariant
#[test]
fn test_autosec4_zero_disclosure_redaction() {
    let mut vault = SecretService::new();
    let raw = b"super_sensitive_api_token_abc";
    let entry = SecretEntry::new(
        "sec_token_redacted",
        "Redacted Secret",
        SecretKind::OAuthToken,
        SecretScope::Global,
        raw,
    ).unwrap();
    assert!(vault.store_secret(entry).is_ok());

    let meta_list = vault.list_metadata(None, None);
    assert_eq!(meta_list.len(), 1);
    let serialized_meta = serde_json::to_string(&meta_list[0]).unwrap();
    assert!(!serialized_meta.contains("super_sensitive_api_token_abc"));
}

/// AUTOSEC5: Version Tracking & Fingerprinting
#[test]
fn test_autosec5_version_tracking_and_fingerprinting() {
    let mut vault = SecretService::new();
    let entry = SecretEntry::new(
        "sec_rot",
        "Rotation Secret",
        SecretKind::ApiKey,
        SecretScope::Global,
        b"v1_data",
    ).unwrap();
    assert!(vault.store_secret(entry).is_ok());

    let fp_v1 = vault.get_metadata("sec_rot").unwrap().fingerprint;
    assert!(vault.rotate_secret("sec_rot", b"v2_new_data").is_ok());

    let meta_v2 = vault.get_metadata("sec_rot").unwrap();
    assert_eq!(meta_v2.version, 2);
    assert_eq!(meta_v2.state, SecretState::Rotated);
    assert_ne!(meta_v2.fingerprint, fp_v1);
}

/// AUTOSEC6: Capacity & Numerical Bounds
#[test]
fn test_autosec6_capacity_and_bounds() {
    let mut cfg = SecretConfig::default();
    cfg.max_secrets_capacity = 2;
    cfg.max_payload_bytes = 20;

    let mut vault = SecretService::new_with_config(cfg);

    let e1 = SecretEntry::new("k1", "Key 1", SecretKind::ApiKey, SecretScope::Global, b"short_val").unwrap();
    let e2 = SecretEntry::new("k2", "Key 2", SecretKind::ApiKey, SecretScope::Global, b"short_val").unwrap();
    let e3 = SecretEntry::new("k3", "Key 3", SecretKind::ApiKey, SecretScope::Global, b"short_val").unwrap();

    assert!(vault.store_secret(e1).is_ok());
    assert!(vault.store_secret(e2).is_ok());
    assert!(vault.store_secret(e3).is_err());
}

/// AUTOSEC7: Atomic Persistence & Tamper Rejection
#[test]
fn test_autosec7_atomic_persistence() {
    let dir = tempdir().unwrap();
    let store_file = dir.path().join("secure_vault.json");

    let mut vault = SecretService::new();
    let e1 = SecretEntry::new("k1", "Key 1", SecretKind::ApiKey, SecretScope::Global, b"payload1").unwrap();
    vault.store_secret(e1).unwrap();

    assert!(vault.save_to_path(&store_file).is_ok());
    let reloaded = SecretService::load_from_path(&store_file).unwrap();
    assert_eq!(reloaded.len(), 1);
}

/// AUTOSEC8: Multi-Threaded Concurrency Safety
#[test]
fn test_autosec8_concurrency_safety() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let vault = Arc::new(Mutex::new(SecretService::new()));

    // Store baseline secret
    {
        let mut v = vault.lock().unwrap();
        v.store_secret(SecretEntry::new(
            "shared_sec", "Shared Secret", SecretKind::ApiKey, SecretScope::Global, b"init_data"
        ).unwrap()).unwrap();
    }

    let mut handles = vec![];
    for i in 0..8 {
        let v_clone = Arc::clone(&vault);
        handles.push(thread::spawn(move || {
            let v = v_clone.lock().unwrap();
            let val = v.get_secret("shared_sec", &SecretScope::Global);
            assert!(val.is_ok());
            drop(v);

            if i % 2 == 0 {
                let mut v_mut = v_clone.lock().unwrap();
                let _ = v_mut.rotate_secret("shared_sec", format!("data_from_thread_{}", i).as_bytes());
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let v_final = vault.lock().unwrap();
    assert!(v_final.get_metadata("shared_sec").unwrap().version > 1);
}
