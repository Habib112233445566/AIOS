//! Unit tests for PEP Grant Lifecycle Configuration (T-02245).

use aiosh_core::pep_grant_config::{
    PepGrantConfig, DEFAULT_MAX_DELEGATION_DEPTH, DEFAULT_MAX_GRANTS,
    DEFAULT_MAX_STORE_BYTES, DEFAULT_PEP_GRANT_STORE_PATH,
    GRANTCONF_ERR_BOUNDS, GRANTCONF_ERR_VALIDATION,
    MAX_DELEGATION_DEPTH, MAX_GRANTS_COUNT, MAX_STORE_BYTES,
    MIN_DELEGATION_DEPTH, MIN_GRANTS_COUNT, MIN_STORE_BYTES,
};
use std::path::PathBuf;

#[test]
fn test_pep_grant_config_default() {
    let config = PepGrantConfig::default();
    assert_eq!(config.version, "1.0.0");
    assert_eq!(config.store_path, PathBuf::from(DEFAULT_PEP_GRANT_STORE_PATH));
    assert_eq!(config.max_store_bytes, DEFAULT_MAX_STORE_BYTES);
    assert_eq!(config.max_grants, DEFAULT_MAX_GRANTS);
    assert_eq!(config.default_max_delegation_depth, DEFAULT_MAX_DELEGATION_DEPTH);
    assert!(config.auto_sweep_on_load);
    assert!(!config.cascade_revocation_by_default);
    assert!(config.validate().is_ok());
}

#[test]
fn test_pep_grant_config_json_roundtrip() {
    let mut config = PepGrantConfig::default();
    config.max_grants = 1200;
    config.default_max_delegation_depth = 5;
    config.store_path = PathBuf::from("custom/path/grants.json");
    config.auto_sweep_on_load = false;
    config.cascade_revocation_by_default = true;

    let json_str = config.to_json().expect("serialize failed");
    let parsed = PepGrantConfig::from_json(&json_str).expect("deserialize failed");
    assert_eq!(config, parsed);
}

#[test]
fn test_pep_grant_config_validation_empty_version() {
    let mut config = PepGrantConfig::default();
    config.version = "   ".to_string();
    let err = config.validate().unwrap_err();
    assert!(err.contains(GRANTCONF_ERR_VALIDATION));
    assert!(err.contains("version cannot be empty"));
}

#[test]
fn test_pep_grant_config_validation_path_traversal() {
    let mut config = PepGrantConfig::default();
    config.store_path = PathBuf::from("../evil/store.json");
    let err = config.validate().unwrap_err();
    assert!(err.contains(GRANTCONF_ERR_VALIDATION));
    assert!(err.contains("parent traversal"));
}

#[test]
fn test_pep_grant_config_validation_invalid_extension() {
    let mut config = PepGrantConfig::default();
    config.store_path = PathBuf::from(".aios/grants.yaml");
    let err = config.validate().unwrap_err();
    assert!(err.contains(GRANTCONF_ERR_VALIDATION));
    assert!(err.contains(".json extension"));
}

#[test]
fn test_pep_grant_config_validation_bounds() {
    let mut config = PepGrantConfig::default();

    // max_grants < MIN
    config.max_grants = MIN_GRANTS_COUNT - 1;
    let err = config.validate().unwrap_err();
    assert!(err.contains(GRANTCONF_ERR_BOUNDS));

    // max_grants > MAX
    config.max_grants = MAX_GRANTS_COUNT + 1;
    let err = config.validate().unwrap_err();
    assert!(err.contains(GRANTCONF_ERR_BOUNDS));

    // Reset max_grants
    config.max_grants = DEFAULT_MAX_GRANTS;

    // max_store_bytes < MIN
    config.max_store_bytes = MIN_STORE_BYTES - 1;
    let err = config.validate().unwrap_err();
    assert!(err.contains(GRANTCONF_ERR_BOUNDS));

    // max_store_bytes > MAX
    config.max_store_bytes = MAX_STORE_BYTES + 1;
    let err = config.validate().unwrap_err();
    assert!(err.contains(GRANTCONF_ERR_BOUNDS));

    // Reset max_store_bytes
    config.max_store_bytes = DEFAULT_MAX_STORE_BYTES;

    // delegation_depth < MIN
    config.default_max_delegation_depth = MIN_DELEGATION_DEPTH - 1;
    let err = config.validate().unwrap_err();
    assert!(err.contains(GRANTCONF_ERR_BOUNDS));

    // delegation_depth > MAX
    config.default_max_delegation_depth = MAX_DELEGATION_DEPTH + 1;
    let err = config.validate().unwrap_err();
    assert!(err.contains(GRANTCONF_ERR_BOUNDS));
}

#[test]
fn test_pep_grant_config_file_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("create tempdir");
    let file_path = dir.path().join("test_pep_grant_config.json");

    let mut config = PepGrantConfig::default();
    config.max_grants = 2500;
    config.default_max_delegation_depth = 4;
    config.store_path = PathBuf::from(".aios/custom_grants.json");

    config.save_to_path(&file_path).expect("save failed");
    assert!(file_path.exists());

    let loaded = PepGrantConfig::from_path(&file_path).expect("load failed");
    assert_eq!(config, loaded);
}

#[test]
fn test_pep_grant_config_from_env() {
    std::env::set_var("AIOSH_PEP_GRANT_STORE_PATH", "env_store/grants.json");
    std::env::set_var("AIOSH_PEP_GRANT_MAX_GRANTS", "3500");
    std::env::set_var("AIOSH_PEP_GRANT_MAX_STORE_BYTES", "20971520");
    std::env::set_var("AIOSH_PEP_GRANT_MAX_DELEGATION_DEPTH", "6");
    std::env::set_var("AIOSH_PEP_GRANT_AUTO_SWEEP", "false");
    std::env::set_var("AIOSH_PEP_GRANT_CASCADE_REVOCATION", "true");

    let config = PepGrantConfig::from_env().expect("from_env failed");
    assert_eq!(config.store_path, PathBuf::from("env_store/grants.json"));
    assert_eq!(config.max_grants, 3500);
    assert_eq!(config.max_store_bytes, 20971520);
    assert_eq!(config.default_max_delegation_depth, 6);
    assert!(!config.auto_sweep_on_load);
    assert!(config.cascade_revocation_by_default);

    // Clean up env
    std::env::remove_var("AIOSH_PEP_GRANT_STORE_PATH");
    std::env::remove_var("AIOSH_PEP_GRANT_MAX_GRANTS");
    std::env::remove_var("AIOSH_PEP_GRANT_MAX_STORE_BYTES");
    std::env::remove_var("AIOSH_PEP_GRANT_MAX_DELEGATION_DEPTH");
    std::env::remove_var("AIOSH_PEP_GRANT_AUTO_SWEEP");
    std::env::remove_var("AIOSH_PEP_GRANT_CASCADE_REVOCATION");
}
