//! Integration tests for Privilege Escalation Prevention Configuration (T-02546).

use aiosh_core::privilege_config::*;
use aiosh_core::privilege_data_model::PrivilegeLevel;
use aiosh_core::privilege_service::PrivilegeService;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_config_integration_with_service() {
    let dir = tempdir().unwrap();
    let cfg_path = dir.path().join("privilege.json");

    let mut cfg = PrivilegeConfig::default();
    cfg.max_active_contexts = 128;
    cfg.default_tier = PrivilegeLevel::User;
    cfg.save_to_path(&cfg_path).unwrap();

    let loaded = PrivilegeConfig::load_from_path(&cfg_path).unwrap();
    assert_eq!(loaded.max_active_contexts, 128);

    // Instantiate service using loaded configuration capacity
    let mut srv = PrivilegeService::with_capacity(loaded.max_active_contexts);
    assert_eq!(srv.active_contexts_count(), 0);

    let ctx = aiosh_core::privilege_data_model::PrivilegeContext::new("test_agent", loaded.default_tier).unwrap();
    assert!(srv.register_context(ctx).is_ok());
    assert_eq!(srv.active_contexts_count(), 1);
}

#[test]
fn test_config_env_overrides_integration() {
    std::env::set_var("AIOS_PRIVILEGE_MAX_CONTEXTS", "2048");
    std::env::set_var("AIOS_PRIVILEGE_MAX_GRANT_DURATION", "7200");
    std::env::set_var("AIOS_PRIVILEGE_AUDIT_ALL", "false");

    let cfg = PrivilegeConfig::load_with_env_overrides();
    assert_eq!(cfg.max_active_contexts, 2048);
    assert_eq!(cfg.max_grant_duration_seconds, 7200);
    assert!(!cfg.audit_all_transitions);

    // Clean up
    std::env::remove_var("AIOS_PRIVILEGE_MAX_CONTEXTS");
    std::env::remove_var("AIOS_PRIVILEGE_MAX_GRANT_DURATION");
    std::env::remove_var("AIOS_PRIVILEGE_AUDIT_ALL");
}

#[test]
fn test_config_path_traversal_shielding() {
    let mut cfg = PrivilegeConfig::default();
    cfg.store_path = PathBuf::from("../../etc/shadow");
    let err = cfg.validate().unwrap_err();
    assert!(err.contains(PRIVESCCONF_ERR_VALIDATION));
}
