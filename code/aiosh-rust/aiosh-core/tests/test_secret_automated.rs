//! Automated Test Suite for Secrets Handling Subsystem (T-02651..T-02660).
//!
//! Implements formal verification vectors AUTOSEC1..AUTOSEC8 defined in
//! `docs/SPEC-SECRETS-AUTOMATED-TESTS.md`.

use std::sync::{Arc, Mutex};
use std::thread;
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

    // 1. Initial active state
    let meta = vault.get_metadata("sec_life_1").unwrap();
    assert_eq!(meta.state, SecretState::Active);
    assert_eq!(meta.version, 1);
    assert!(vault.get_secret("sec_life_1", &SecretScope::Global).is_ok());

    // 2. Rotate to version 2
    assert!(vault.rotate_secret("sec_life_1", b"rotated_payload_67890").is_ok());
    let meta2 = vault.get_metadata("sec_life_1").unwrap();
    assert_eq!(meta2.state, SecretState::Rotated);
    assert_eq!(meta2.version, 2);

    // 3. Revoke
    assert!(vault.revoke_secret("sec_life_1").is_ok());
    let meta3 = vault.get_metadata("sec_life_1").unwrap();
    assert_eq!(meta3.state, SecretState::Revoked);

    // Revoked secret access denied
    let err = vault.get_secret("sec_life_1", &SecretScope::Global).unwrap_err();
    assert!(err.contains(SECSVC_ERR_INACCESSIBLE));
}

/// AUTOSEC2: Scope Boundary Enforcement
#[test]
fn test_autosec2_scope_boundary_enforcement() {
    let mut vault = SecretService::new();

    // 1. Global Scope
    let e_global = SecretEntry::new(
        "sec_global", "Global Secret", SecretKind::ApiKey, SecretScope::Global, b"global_secret"
    ).unwrap();
    vault.store_secret(e_global).unwrap();
    assert!(vault.get_secret("sec_global", &SecretScope::Global).is_ok());

    // 2. Actor Scope
    let e_actor = SecretEntry::new(
        "sec_actor", "Actor Secret", SecretKind::ApiKey,
        SecretScope::Actor("worker_alpha".into()), b"alpha_data"
    ).unwrap();
    vault.store_secret(e_actor).unwrap();
    assert!(vault.get_secret("sec_actor", &SecretScope::Global).is_ok());
    assert!(vault.get_secret("sec_actor", &SecretScope::Actor("worker_alpha".into())).is_ok());
    assert!(vault.get_secret("sec_actor", &SecretScope::Actor("worker_beta".into())).is_err());

    // 3. Session Scope
    let e_sess = SecretEntry::new(
        "sec_sess", "Session Secret", SecretKind::OAuthToken,
        SecretScope::Session("sess_100".into()), b"sess_data"
    ).unwrap();
    vault.store_secret(e_sess).unwrap();
    assert!(vault.get_secret("sec_sess", &SecretScope::Session("sess_100".into())).is_ok());
    assert!(vault.get_secret("sec_sess", &SecretScope::Session("sess_200".into())).is_err());

    // 4. Environment Scope
    let e_env = SecretEntry::new(
        "sec_env", "Env Secret", SecretKind::DatabaseCredential,
        SecretScope::Environment("staging".into()), b"staging_conn"
    ).unwrap();
    vault.store_secret(e_env).unwrap();
    assert!(vault.get_secret("sec_env", &SecretScope::Environment("staging".into())).is_ok());
    assert!(vault.get_secret("sec_env", &SecretScope::Environment("production".into())).is_err());
}

/// AUTOSEC3: Privilege Tier Access Gates
#[test]
fn test_autosec3_privilege_tier_access_gates() {
    let mut vault = SecretService::new();

    let e_env = SecretEntry::new(
        "sec_env_prod", "Production DB", SecretKind::DatabaseCredential,
        SecretScope::Environment("production".into()), b"prod_db_conn"
    ).unwrap();
    let e_actor = SecretEntry::new(
        "sec_actor_bob", "Bob Key", SecretKind::ApiKey,
        SecretScope::Actor("bob".into()), b"bob_secret"
    ).unwrap();

    vault.store_secret(e_env).unwrap();
    vault.store_secret(e_actor).unwrap();

    let kernel = PrivilegeContext::new("root_kernel", PrivilegeLevel::SystemKernel).unwrap();
    let admin = PrivilegeContext::new("superadmin", PrivilegeLevel::Admin).unwrap();
    let operator = PrivilegeContext::new("lead_sre", PrivilegeLevel::Operator).unwrap();
    let user_bob = PrivilegeContext::new("bob", PrivilegeLevel::User).unwrap();
    let user_alice = PrivilegeContext::new("alice", PrivilegeLevel::User).unwrap();
    let guest = PrivilegeContext::new("anon", PrivilegeLevel::Guest).unwrap();

    // Environment scope: kernel/admin/operator allowed, user/guest denied
    assert!(vault.get_secret_with_privilege("sec_env_prod", &kernel).is_ok());
    assert!(vault.get_secret_with_privilege("sec_env_prod", &admin).is_ok());
    assert!(vault.get_secret_with_privilege("sec_env_prod", &operator).is_ok());
    assert!(vault.get_secret_with_privilege("sec_env_prod", &user_bob).is_err());
    assert!(vault.get_secret_with_privilege("sec_env_prod", &guest).is_err());

    // Actor scope: kernel/admin allowed; bob allowed; alice denied; guest denied
    assert!(vault.get_secret_with_privilege("sec_actor_bob", &kernel).is_ok());
    assert!(vault.get_secret_with_privilege("sec_actor_bob", &admin).is_ok());
    assert!(vault.get_secret_with_privilege("sec_actor_bob", &user_bob).is_ok());
    assert!(vault.get_secret_with_privilege("sec_actor_bob", &user_alice).is_err());
    assert!(vault.get_secret_with_privilege("sec_actor_bob", &guest).is_err());
}

/// AUTOSEC4: Zero-Disclosure Redaction Invariant
#[test]
fn test_autosec4_zero_disclosure_redaction() {
    let mut vault = SecretService::new();
    let raw = b"super_sensitive_api_token_abc123456789";
    let entry = SecretEntry::new(
        "sec_token_redacted",
        "Redacted Secret",
        SecretKind::OAuthToken,
        SecretScope::Global,
        raw,
    ).unwrap();
    assert!(vault.store_secret(entry).is_ok());

    // 1. List metadata does not contain raw bytes or plaintext
    let meta_list = vault.list_metadata(None, None);
    assert_eq!(meta_list.len(), 1);
    let serialized_meta = serde_json::to_string(&meta_list[0]).unwrap();
    assert!(!serialized_meta.contains("super_sensitive_api_token_abc123456789"));

    // 2. SecretValue debug formatting does not disclose plaintext
    let val = vault.get_secret("sec_token_redacted", &SecretScope::Global).unwrap();
    let val_debug = format!("{:?}", val);
    assert!(!val_debug.contains("super_sensitive_api_token_abc123456789"));
}

/// AUTOSEC5: Version Tracking & Fingerprinting
#[test]
fn test_autosec5_version_tracking_and_fingerprinting() {
    let mut vault = SecretService::new();
    let entry = SecretEntry::new(
        "sec_rot", "Rotation Secret", SecretKind::ApiKey, SecretScope::Global, b"v1_data"
    ).unwrap();
    vault.store_secret(entry).unwrap();

    let fp_v1 = vault.get_metadata("sec_rot").unwrap().fingerprint;

    // Rotate to v2
    assert!(vault.rotate_secret("sec_rot", b"v2_new_data").is_ok());
    let meta_v2 = vault.get_metadata("sec_rot").unwrap();
    assert_eq!(meta_v2.version, 2);
    assert_eq!(meta_v2.state, SecretState::Rotated);
    assert_ne!(meta_v2.fingerprint, fp_v1);

    // Rotate to v3
    assert!(vault.rotate_secret("sec_rot", b"v3_third_data").is_ok());
    let meta_v3 = vault.get_metadata("sec_rot").unwrap();
    assert_eq!(meta_v3.version, 3);
    assert_ne!(meta_v3.fingerprint, meta_v2.fingerprint);

    // Verify retrieval reflects v3 value
    let val_v3 = vault.get_secret("sec_rot", &SecretScope::Global).unwrap();
    assert_eq!(val_v3.as_bytes(), b"v3_third_data");
}

/// AUTOSEC6: Capacity & Numerical Bounds
#[test]
fn test_autosec6_capacity_and_bounds() {
    let mut cfg = SecretConfig::default();
    cfg.max_secrets_capacity = 2;
    cfg.max_payload_bytes = 16;

    let mut vault = SecretService::new_with_config(cfg);

    // 1. Within payload bound (16 bytes)
    let e1 = SecretEntry::new("k1", "Key 1", SecretKind::ApiKey, SecretScope::Global, b"12345678").unwrap();
    assert!(vault.store_secret(e1).is_ok());

    // 2. Exceeding payload bound (> 16 bytes) fails fail-closed
    let e_toolarge = SecretEntry::new(
        "k_big", "Big Key", SecretKind::ApiKey, SecretScope::Global, b"this_is_longer_than_16_bytes"
    ).unwrap();
    let err_size = vault.store_secret(e_toolarge).unwrap_err();
    assert!(err_size.contains(SECSVC_ERR_FILE_SIZE));

    // 3. Storing 2nd valid entry succeeds
    let e2 = SecretEntry::new("k2", "Key 2", SecretKind::ApiKey, SecretScope::Global, b"12345678").unwrap();
    assert!(vault.store_secret(e2).is_ok());

    // 4. Storing 3rd entry exceeds capacity limit (2)
    let e3 = SecretEntry::new("k3", "Key 3", SecretKind::ApiKey, SecretScope::Global, b"12345678").unwrap();
    let err_cap = vault.store_secret(e3).unwrap_err();
    assert!(err_cap.contains(SECSVC_ERR_CAPACITY_EXCEEDED));
}

/// AUTOSEC7: Atomic Persistence & Tamper Rejection
#[test]
fn test_autosec7_atomic_persistence() {
    let dir = tempdir().unwrap();
    let store_file = dir.path().join("secure_vault.json");

    let mut vault = SecretService::new();
    let e1 = SecretEntry::new("k1", "Key 1", SecretKind::ApiKey, SecretScope::Global, b"payload1").unwrap();
    vault.store_secret(e1).unwrap();

    // 1. Atomic save
    assert!(vault.save_to_path(&store_file).is_ok());
    assert!(store_file.exists());

    // 2. Reload and verify fidelity
    let reloaded = SecretService::load_from_path(&store_file).unwrap();
    assert_eq!(reloaded.len(), 1);
    let val = reloaded.get_secret("k1", &SecretScope::Global).unwrap();
    assert_eq!(val.as_bytes(), b"payload1");

    // 3. Traversal rejection in save and load
    let bad_path = std::path::Path::new("../forbidden/vault.json");
    assert!(vault.save_to_path(bad_path).is_err());
    assert!(SecretService::load_from_path(bad_path).is_err());

    // 4. Corrupted file fail-closed rejection
    let corrupt_file = dir.path().join("corrupt.json");
    std::fs::write(&corrupt_file, "{ not_valid_json }").unwrap();
    assert!(SecretService::load_from_path(&corrupt_file).is_err());
}

/// AUTOSEC8: Multi-Threaded Concurrency Safety
#[test]
fn test_autosec8_concurrency_safety() {
    let vault = Arc::new(Mutex::new(SecretService::new()));

    // Store baseline secrets
    {
        let mut v = vault.lock().unwrap();
        for i in 0..4 {
            v.store_secret(SecretEntry::new(
                &format!("shared_sec_{}", i),
                &format!("Shared Secret {}", i),
                SecretKind::ApiKey,
                SecretScope::Global,
                format!("init_payload_{}", i).as_bytes(),
            ).unwrap()).unwrap();
        }
    }

    let mut handles = vec![];
    for t in 0..16 {
        let v_clone = Arc::clone(&vault);
        handles.push(thread::spawn(move || {
            let key = format!("shared_sec_{}", t % 4);
            // Read
            {
                let v = v_clone.lock().unwrap();
                let val = v.get_secret(&key, &SecretScope::Global);
                assert!(val.is_ok());
            }

            // Rotate on some threads
            if t % 3 == 0 {
                let mut v_mut = v_clone.lock().unwrap();
                let _ = v_mut.rotate_secret(&key, format!("rot_thread_{}", t).as_bytes());
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let v_final = vault.lock().unwrap();
    assert_eq!(v_final.len(), 4);
    for i in 0..4 {
        let meta = v_final.get_metadata(&format!("shared_sec_{}", i)).unwrap();
        assert!(meta.version >= 1);
    }
}

/// AUTOSEC9: Partial File and Corrupt Vault Recovery
#[test]
fn test_autosec9_partial_file_and_empty_vault_recovery() {
    let dir = tempdir().unwrap();

    // 1. Zero-byte file returns parse error fail-closed
    let empty_file = dir.path().join("empty.json");
    std::fs::write(&empty_file, b"").unwrap();
    assert!(SecretService::load_from_path(&empty_file).is_err());

    // 2. Truncated JSON returns parse error fail-closed
    let trunc_file = dir.path().join("trunc.json");
    std::fs::write(&trunc_file, b"{\"version\": \"1.0.0\", \"secrets\": {").unwrap();
    assert!(SecretService::load_from_path(&trunc_file).is_err());

    // 3. Valid empty secrets map succeeds
    let valid_empty = dir.path().join("valid_empty.json");
    std::fs::write(&valid_empty, b"{\"version\": \"1.0.0\", \"secrets\": {}}").unwrap();
    let srv = SecretService::load_from_path(&valid_empty).unwrap();
    assert_eq!(srv.len(), 0);
}

/// AUTOSEC10: Rapid Rotation Churn & Monotonic Versioning
#[test]
fn test_autosec10_rapid_rotation_churn() {
    let mut vault = SecretService::new();
    let entry = SecretEntry::new(
        "churn_sec", "Churn Secret", SecretKind::ApiKey, SecretScope::Global, b"init_0"
    ).unwrap();
    vault.store_secret(entry).unwrap();

    let mut last_fp = vault.get_metadata("churn_sec").unwrap().fingerprint;
    for rot in 1..=30 {
        let payload = format!("churn_payload_{}", rot);
        assert!(vault.rotate_secret("churn_sec", payload.as_bytes()).is_ok());

        let meta = vault.get_metadata("churn_sec").unwrap();
        assert_eq!(meta.version, rot + 1);
        assert_eq!(meta.state, SecretState::Rotated);
        assert_ne!(meta.fingerprint, last_fp);
        last_fp = meta.fingerprint;
    }

    let final_val = vault.get_secret("churn_sec", &SecretScope::Global).unwrap();
    assert_eq!(final_val.as_bytes(), b"churn_payload_30");
}

