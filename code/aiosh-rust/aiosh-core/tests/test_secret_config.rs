//! Unit tests for Secrets Configuration Subsystem (T-02645).

use aiosh_core::secret_config::*;
use aiosh_core::secret_data_model::{SecretEntry, SecretKind, SecretScope};
use aiosh_core::secret_service::SecretService;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_secret_config_defaults() {
    let cfg = SecretConfig::default();
    assert_eq!(cfg.version, "1.0.0");
    assert_eq!(cfg.max_secrets_capacity, DEFAULT_MAX_SECRETS_CAPACITY);
    assert_eq!(cfg.max_payload_bytes, DEFAULT_MAX_PAYLOAD_BYTES);
    assert_eq!(cfg.max_store_file_bytes, DEFAULT_MAX_STORE_FILE_BYTES);
    assert!(cfg.require_expose_flag);
    assert!(cfg.enforce_scope_containment);
    assert!(cfg.audit_all_reads);
    assert!(cfg.audit_all_writes);
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_secret_config_bounds_validation() {
    let mut cfg = SecretConfig::default();

    // Invalid version
    cfg.version = "2.0.0".into();
    assert!(cfg.validate().is_err());
    cfg.version = "1.0.0".into();

    // Traversal in store_path
    cfg.store_path = PathBuf::from("../etc/secrets.json");
    let err = cfg.validate().unwrap_err();
    assert!(err.contains(SECCONF_ERR_VALIDATION));
    cfg.store_path = PathBuf::from(DEFAULT_SECRETS_STORE_PATH);

    // Capacity bounds
    cfg.max_secrets_capacity = 0;
    assert!(cfg.validate().unwrap_err().contains(SECCONF_ERR_BOUNDS));
    cfg.max_secrets_capacity = 20000;
    assert!(cfg.validate().unwrap_err().contains(SECCONF_ERR_BOUNDS));
    cfg.max_secrets_capacity = 100;
    assert!(cfg.validate().is_ok());

    // Payload bounds
    cfg.max_payload_bytes = 0;
    assert!(cfg.validate().unwrap_err().contains(SECCONF_ERR_BOUNDS));
    cfg.max_payload_bytes = 2000000;
    assert!(cfg.validate().unwrap_err().contains(SECCONF_ERR_BOUNDS));
    cfg.max_payload_bytes = 1024;
    assert!(cfg.validate().is_ok());

    // Store file size bounds
    cfg.max_store_file_bytes = 100;
    assert!(cfg.validate().unwrap_err().contains(SECCONF_ERR_BOUNDS));
    cfg.max_store_file_bytes = 50000000;
    assert!(cfg.validate().unwrap_err().contains(SECCONF_ERR_BOUNDS));
    cfg.max_store_file_bytes = 1048576;
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_secret_config_save_and_load() {
    let dir = tempdir().unwrap();
    let cfg_path = dir.path().join("sub/config.json");

    let mut cfg = SecretConfig::default();
    cfg.max_secrets_capacity = 500;
    cfg.max_payload_bytes = 4096;
    cfg.require_expose_flag = false;

    assert!(cfg.save_to_path(&cfg_path).is_ok());
    assert!(cfg_path.exists());

    let loaded = SecretConfig::load_from_path(&cfg_path).unwrap();
    assert_eq!(loaded.max_secrets_capacity, 500);
    assert_eq!(loaded.max_payload_bytes, 4096);
    assert!(!loaded.require_expose_flag);
}

#[test]
fn test_secret_config_env_overrides() {
    std::env::set_var("AIOS_SECRETS_STORE", "custom/vault.json");
    std::env::set_var("AIOS_SECRETS_MAX_CAPACITY", "512");
    std::env::set_var("AIOS_SECRETS_REQUIRE_EXPOSE", "false");

    let cfg = SecretConfig::from_env();
    assert_eq!(cfg.store_path, PathBuf::from("custom/vault.json"));
    assert_eq!(cfg.max_secrets_capacity, 512);
    assert!(!cfg.require_expose_flag);

    std::env::remove_var("AIOS_SECRETS_STORE");
    std::env::remove_var("AIOS_SECRETS_MAX_CAPACITY");
    std::env::remove_var("AIOS_SECRETS_REQUIRE_EXPOSE");
}

#[test]
fn test_secret_service_with_custom_config() {
    let mut cfg = SecretConfig::default();
    cfg.max_secrets_capacity = 2;
    cfg.max_payload_bytes = 10;

    let mut srv = SecretService::new_with_config(cfg);
    assert_eq!(srv.config().max_secrets_capacity, 2);
    assert_eq!(srv.config().max_payload_bytes, 10);

    // Storing entry within payload limit
    let entry1 = SecretEntry::new(
        "k1", "Key 1", SecretKind::ApiKey, SecretScope::Global, b"12345678"
    ).unwrap();
    assert!(srv.store_secret(entry1).is_ok());

    // Payload exceeding configured 10 bytes limit
    let entry_large = SecretEntry::new(
        "k2", "Key 2", SecretKind::ApiKey, SecretScope::Global, b"1234567890123"
    ).unwrap();
    assert!(srv.store_secret(entry_large).is_err());

    // Store second valid entry
    let entry2 = SecretEntry::new(
        "k2", "Key 2", SecretKind::ApiKey, SecretScope::Global, b"val2"
    ).unwrap();
    assert!(srv.store_secret(entry2).is_ok());

    // Store 3rd entry -> capacity exceeded (limit=2)
    let entry3 = SecretEntry::new(
        "k3", "Key 3", SecretKind::ApiKey, SecretScope::Global, b"val3"
    ).unwrap();
    assert!(srv.store_secret(entry3).is_err());
}
