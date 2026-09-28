//! Unit tests for Audit Chain Extensions Configuration (T-02345).

use aiosh_core::audit_chain_config::*;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_config_default_and_validation() {
    let cfg = AuditChainConfig::default();
    assert_eq!(cfg.version, "1.0.0");
    assert_eq!(cfg.max_query_limit, 50);
    assert_eq!(cfg.default_lineage_depth, 16);
    assert_eq!(cfg.max_causal_links, 16);
    assert_eq!(cfg.max_extensions_bytes, 65536);
    assert!(cfg.verify_signatures_on_read);
    assert!(!cfg.strict_provenance);
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_config_json_roundtrip() {
    let mut cfg = AuditChainConfig::default();
    cfg.max_query_limit = 250;
    cfg.default_lineage_depth = 32;
    cfg.strict_provenance = true;

    let json_str = serde_json::to_string(&cfg).expect("serialize");
    let loaded = AuditChainConfig::from_json(&json_str).expect("deserialize");
    assert_eq!(cfg, loaded);
}

#[test]
fn test_config_bounds_enforcement() {
    // 1. Invalid version
    let mut bad_cfg = AuditChainConfig::default();
    bad_cfg.version = "2.0.0".to_string();
    assert!(bad_cfg.validate().is_err());

    // 2. Query limit boundary (0 or > 1000)
    let mut bad_query = AuditChainConfig::default();
    bad_query.max_query_limit = 0;
    assert!(bad_query.validate().is_err());

    bad_query.max_query_limit = 1001;
    assert!(bad_query.validate().is_err());

    // 3. Lineage depth boundary (0 or > 64)
    let mut bad_depth = AuditChainConfig::default();
    bad_depth.default_lineage_depth = 0;
    assert!(bad_depth.validate().is_err());

    bad_depth.default_lineage_depth = 65;
    assert!(bad_depth.validate().is_err());

    // 4. Causal links boundary (0 or > 32)
    let mut bad_links = AuditChainConfig::default();
    bad_links.max_causal_links = 0;
    assert!(bad_links.validate().is_err());

    bad_links.max_causal_links = 33;
    assert!(bad_links.validate().is_err());

    // 5. Extensions bytes boundary (< 1024 or > 1048576)
    let mut bad_bytes = AuditChainConfig::default();
    bad_bytes.max_extensions_bytes = 512;
    assert!(bad_bytes.validate().is_err());

    bad_bytes.max_extensions_bytes = 2_000_000;
    assert!(bad_bytes.validate().is_err());
}

#[test]
fn test_config_file_save_and_load() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("audit_cfg.json");

    let mut cfg = AuditChainConfig::default();
    cfg.db_path = PathBuf::from("/custom/path/audit.db");
    cfg.max_query_limit = 500;

    cfg.save_to_file(&config_path).expect("save_to_file");
    assert!(config_path.exists());

    let loaded = AuditChainConfig::from_file(&config_path).expect("from_file");
    assert_eq!(cfg, loaded);
}

#[test]
fn test_config_file_oversized_rejected() {
    let dir = tempdir().expect("tempdir");
    let oversized_path = dir.path().join("oversized.json");

    // Create file exceeding 64 KiB
    let big_data = vec![b' '; (MAX_CONFIG_FILE_BYTES + 100) as usize];
    std::fs::write(&oversized_path, big_data).expect("write");

    let res = AuditChainConfig::from_file(&oversized_path);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.contains(AUDITCONF_ERR_BOUNDS));
}

#[test]
fn test_config_from_env_overrides() {
    std::env::set_var("AIOS_AUDIT_DB_PATH", "custom_env_audit.db");
    std::env::set_var("AIOS_AUDIT_MAX_QUERY_LIMIT", "300");
    std::env::set_var("AIOS_AUDIT_LINEAGE_DEPTH", "48");

    let cfg = AuditChainConfig::from_env();
    assert_eq!(cfg.db_path, PathBuf::from("custom_env_audit.db"));
    assert_eq!(cfg.max_query_limit, 300);
    assert_eq!(cfg.default_lineage_depth, 48);

    // Cleanup
    std::env::remove_var("AIOS_AUDIT_DB_PATH");
    std::env::remove_var("AIOS_AUDIT_MAX_QUERY_LIMIT");
    std::env::remove_var("AIOS_AUDIT_LINEAGE_DEPTH");
}
