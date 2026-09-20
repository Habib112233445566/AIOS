//! Unit tests for System Update Configuration subsystem (T-01945).

use std::fs;
use std::path::PathBuf;
use aiosh_core::system_update::UpdateChannel;
use aiosh_core::system_update_config::{
    SystemUpdateConfig, UCONF_VALIDATION_ERROR,
    DEFAULT_UPDATE_STATE_DIR, DEFAULT_UPDATE_STAGING_DIR,
};

#[test]
fn test_default_config_valid() {
    let cfg = SystemUpdateConfig::default();
    assert!(cfg.validate().is_ok());
    assert_eq!(cfg.state_dir, PathBuf::from(DEFAULT_UPDATE_STATE_DIR));
    assert_eq!(cfg.staging_dir, PathBuf::from(DEFAULT_UPDATE_STAGING_DIR));
    assert_eq!(cfg.default_channel, UpdateChannel::Stable);
    assert_eq!(cfg.check_interval_secs, 86400);
    assert!(!cfg.allow_auto_apply);
    assert!(cfg.auto_rollback_on_failure);
    assert_eq!(cfg.min_free_space_bytes, 1_073_741_824);
    assert!(cfg.trusted_keys.is_empty());

    let svc_cfg = cfg.to_service_config();
    assert_eq!(svc_cfg.state_dir, cfg.state_dir);
    assert_eq!(svc_cfg.staging_dir, cfg.staging_dir);
    assert_eq!(svc_cfg.max_payload_bytes, cfg.max_payload_bytes);
    assert_eq!(svc_cfg.auto_rollback_on_failure, cfg.auto_rollback_on_failure);
}

#[test]
fn test_path_hygiene_and_traversal() {
    let mut cfg = SystemUpdateConfig::default();

    // 1. Empty state_dir
    cfg.state_dir = PathBuf::from("");
    assert!(cfg.validate().unwrap_err().contains("cannot be empty"));

    // 2. Oversized path (> 1024)
    cfg.state_dir = PathBuf::from("a".repeat(1025));
    assert!(cfg.validate().unwrap_err().contains("exceeds maximum length"));

    // 3. Control characters
    cfg.state_dir = PathBuf::from("/var/lib/aiosh\nupdates");
    assert!(cfg.validate().unwrap_err().contains("cannot contain control characters"));

    // 4. Parent directory traversal (..)
    cfg.state_dir = PathBuf::from("/var/lib/aiosh/../updates");
    assert!(cfg.validate().unwrap_err().contains("parent directory traversal"));

    // 5. Same for staging_dir
    cfg.state_dir = PathBuf::from(DEFAULT_UPDATE_STATE_DIR);
    cfg.staging_dir = PathBuf::from("/staging/../escape");
    assert!(cfg.validate().unwrap_err().contains("parent directory traversal"));
}

#[test]
fn test_bounds_validation() {
    let mut cfg = SystemUpdateConfig::default();

    // Check interval bounds (60..=2592000)
    cfg.check_interval_secs = 59;
    assert!(cfg.validate().unwrap_err().contains(UCONF_VALIDATION_ERROR));
    cfg.check_interval_secs = 2_592_001;
    assert!(cfg.validate().unwrap_err().contains(UCONF_VALIDATION_ERROR));
    cfg.check_interval_secs = 3600;
    assert!(cfg.validate().is_ok());

    // Payload bounds (1MB..=10GB)
    cfg.max_payload_bytes = 1024; // < 1MB
    assert!(cfg.validate().unwrap_err().contains("max_payload_bytes"));
    cfg.max_payload_bytes = 10_737_418_241; // > 10GB
    assert!(cfg.validate().unwrap_err().contains("max_payload_bytes"));
    cfg.max_payload_bytes = 100 * 1024 * 1024;
    assert!(cfg.validate().is_ok());

    // Min free space bounds (<= 100GB)
    cfg.min_free_space_bytes = 107_374_182_401;
    assert!(cfg.validate().unwrap_err().contains("min_free_space_bytes"));
    cfg.min_free_space_bytes = 2 * 1024 * 1024 * 1024;
    assert!(cfg.validate().is_ok());

    // Trusted keys bounds (<= 32 keys, each <= 256 chars)
    cfg.trusted_keys = (0..33).map(|i| format!("key_{}", i)).collect();
    assert!(cfg.validate().unwrap_err().contains("trusted_keys cannot exceed 32"));

    cfg.trusted_keys = vec!["k".repeat(257)];
    assert!(cfg.validate().unwrap_err().contains("trusted key must be non-empty"));

    cfg.trusted_keys = vec!["valid_key_hash_123".to_string()];
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_from_env_overrides() {
    std::env::set_var("AIOSH_UPDATE_STATE_DIR", "/tmp/env_state");
    std::env::set_var("AIOSH_UPDATE_STAGING_DIR", "/tmp/env_staging");
    std::env::set_var("AIOSH_UPDATE_CHANNEL", "nightly");
    std::env::set_var("AIOSH_UPDATE_CHECK_INTERVAL_SECS", "7200");
    std::env::set_var("AIOSH_UPDATE_AUTO_APPLY", "true");
    std::env::set_var("AIOSH_UPDATE_AUTO_ROLLBACK", "false");
    std::env::set_var("AIOSH_UPDATE_MAX_PAYLOAD_BYTES", "524288000");

    let cfg = SystemUpdateConfig::from_env();
    assert_eq!(cfg.state_dir, PathBuf::from("/tmp/env_state"));
    assert_eq!(cfg.staging_dir, PathBuf::from("/tmp/env_staging"));
    assert_eq!(cfg.default_channel, UpdateChannel::Nightly);
    assert_eq!(cfg.check_interval_secs, 7200);
    assert!(cfg.allow_auto_apply);
    assert!(!cfg.auto_rollback_on_failure);
    assert_eq!(cfg.max_payload_bytes, 524288000);

    // Clean up env vars
    std::env::remove_var("AIOSH_UPDATE_STATE_DIR");
    std::env::remove_var("AIOSH_UPDATE_STAGING_DIR");
    std::env::remove_var("AIOSH_UPDATE_CHANNEL");
    std::env::remove_var("AIOSH_UPDATE_CHECK_INTERVAL_SECS");
    std::env::remove_var("AIOSH_UPDATE_AUTO_APPLY");
    std::env::remove_var("AIOSH_UPDATE_AUTO_ROLLBACK");
    std::env::remove_var("AIOSH_UPDATE_MAX_PAYLOAD_BYTES");
}

#[test]
fn test_file_persistence_and_loading() {
    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let cfg_path = tmp_dir.path().join("config.json");

    let mut orig = SystemUpdateConfig::default();
    orig.state_dir = tmp_dir.path().join("state");
    orig.staging_dir = tmp_dir.path().join("staging");
    orig.default_channel = UpdateChannel::Beta;
    orig.check_interval_secs = 14400;
    orig.allow_auto_apply = true;
    orig.trusted_keys = vec!["key_alpha".to_string(), "key_beta".to_string()];

    // 1. Save to file
    assert!(orig.save_to_file(&cfg_path).is_ok());
    assert!(cfg_path.exists());

    // 2. Load from file and compare
    let loaded = SystemUpdateConfig::from_file(&cfg_path).expect("from_file");
    assert_eq!(loaded, orig);

    // 3. Reject non-existent file
    let bad_path = tmp_dir.path().join("non_existent.json");
    assert!(SystemUpdateConfig::from_file(&bad_path).is_err());

    // 4. Reject oversized file (> 1MB)
    let huge_path = tmp_dir.path().join("huge.json");
    let huge_data = vec![b' '; 1_048_577];
    fs::write(&huge_path, huge_data).expect("write huge");
    assert!(SystemUpdateConfig::from_file(&huge_path).unwrap_err().contains("exceeds limit"));
}
