//! Unit tests for PEP Decision Engine Configuration (T-02145).

use aiosh_core::pep_config::{
    PepConfig, DEFAULT_MAX_RULES, DEFAULT_MAX_STORE_BYTES, DEFAULT_PEP_STORE_PATH,
    MAX_RULES_COUNT, MAX_STORE_BYTES, MIN_STORE_BYTES,
    PEPCONF_ERR_BOUNDS, PEPCONF_ERR_VALIDATION,
};
use aiosh_core::pep_decision::PepCombiningAlgorithm;
use std::path::PathBuf;

#[test]
fn test_pep_config_default() {
    let config = PepConfig::default();
    assert_eq!(config.version, "1.0.0");
    assert_eq!(config.store_path, PathBuf::from(DEFAULT_PEP_STORE_PATH));
    assert_eq!(config.max_store_bytes, DEFAULT_MAX_STORE_BYTES);
    assert_eq!(config.max_rules, DEFAULT_MAX_RULES);
    assert_eq!(config.default_algorithm, PepCombiningAlgorithm::DenyOverrides);
    assert!(config.audit_all_evaluations);
    assert!(config.auto_quarantine_corrupt);
    assert!(config.validate().is_ok());
}

#[test]
fn test_pep_config_json_roundtrip() {
    let mut config = PepConfig::default();
    config.max_rules = 1200;
    config.default_algorithm = PepCombiningAlgorithm::PermitOverrides;
    config.store_path = PathBuf::from("custom/path/pep.json");

    let json_str = config.to_json().expect("serialize failed");
    let parsed = PepConfig::from_json(&json_str).expect("deserialize failed");
    assert_eq!(config, parsed);
}

#[test]
fn test_pep_config_validation_empty_version() {
    let mut config = PepConfig::default();
    config.version = "   ".to_string();
    let err = config.validate().unwrap_err();
    assert!(err.contains(PEPCONF_ERR_VALIDATION));
    assert!(err.contains("version cannot be empty"));
}

#[test]
fn test_pep_config_validation_path_traversal() {
    let mut config = PepConfig::default();
    config.store_path = PathBuf::from("../evil/store.json");
    let err = config.validate().unwrap_err();
    assert!(err.contains(PEPCONF_ERR_VALIDATION));
    assert!(err.contains("parent traversal"));
}

#[test]
fn test_pep_config_validation_invalid_extension() {
    let mut config = PepConfig::default();
    config.store_path = PathBuf::from(".aios/pep.yaml");
    let err = config.validate().unwrap_err();
    assert!(err.contains(PEPCONF_ERR_VALIDATION));
    assert!(err.contains(".json extension"));
}

#[test]
fn test_pep_config_validation_bounds() {
    let mut config = PepConfig::default();

    // max_rules < MIN
    config.max_rules = 0;
    let err = config.validate().unwrap_err();
    assert!(err.contains(PEPCONF_ERR_BOUNDS));

    // max_rules > MAX
    config.max_rules = MAX_RULES_COUNT + 1;
    let err = config.validate().unwrap_err();
    assert!(err.contains(PEPCONF_ERR_BOUNDS));

    // Reset max_rules
    config.max_rules = DEFAULT_MAX_RULES;

    // max_store_bytes < MIN
    config.max_store_bytes = MIN_STORE_BYTES - 1;
    let err = config.validate().unwrap_err();
    assert!(err.contains(PEPCONF_ERR_BOUNDS));

    // max_store_bytes > MAX
    config.max_store_bytes = MAX_STORE_BYTES + 1;
    let err = config.validate().unwrap_err();
    assert!(err.contains(PEPCONF_ERR_BOUNDS));
}

#[test]
fn test_pep_config_file_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("create tempdir");
    let file_path = dir.path().join("test_pep_config.json");

    let mut config = PepConfig::default();
    config.max_rules = 2500;
    config.store_path = PathBuf::from(".aios/policies.json");

    config.save_to_path(&file_path).expect("save failed");
    assert!(file_path.exists());

    let loaded = PepConfig::from_path(&file_path).expect("load failed");
    assert_eq!(config, loaded);
}

#[test]
fn test_pep_config_from_env() {
    std::env::set_var("AIOSH_PEP_STORE_PATH", "env_store/policies.json");
    std::env::set_var("AIOSH_PEP_MAX_RULES", "3500");
    std::env::set_var("AIOSH_PEP_DEFAULT_ALGORITHM", "permit_overrides");

    let config = PepConfig::from_env().expect("from_env failed");
    assert_eq!(config.store_path, PathBuf::from("env_store/policies.json"));
    assert_eq!(config.max_rules, 3500);
    assert_eq!(config.default_algorithm, PepCombiningAlgorithm::PermitOverrides);

    // Clean up env
    std::env::remove_var("AIOSH_PEP_STORE_PATH");
    std::env::remove_var("AIOSH_PEP_MAX_RULES");
    std::env::remove_var("AIOSH_PEP_DEFAULT_ALGORITHM");
}
