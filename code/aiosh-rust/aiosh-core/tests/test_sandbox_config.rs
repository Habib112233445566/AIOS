//! Unit & integration tests for Sandbox Enforcement Configuration (T-02445).

use aiosh_core::sandbox_config::*;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_config_default_and_validation() {
    let cfg = SandboxConfig::default();
    assert_eq!(cfg.version, "1.0.0");
    assert_eq!(cfg.default_profile_name, "standard");
    assert_eq!(cfg.max_output_capture_bytes, DEFAULT_MAX_OUTPUT_CAPTURE_BYTES);
    assert_eq!(cfg.execution_timeout_seconds, DEFAULT_TIMEOUT_SECONDS);
    assert_eq!(cfg.max_registered_profiles, DEFAULT_MAX_PROFILES);
    assert!(!cfg.enforce_pep_grants);
    assert!(cfg.audit_enabled);
    assert!(cfg.custom_profiles_dir.is_none());
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_config_json_roundtrip() {
    let mut cfg = SandboxConfig::default();
    cfg.default_profile_name = "strict".to_string();
    cfg.max_output_capture_bytes = 20 * 1024 * 1024;
    cfg.execution_timeout_seconds = 600;
    cfg.enforce_pep_grants = true;

    let json_str = serde_json::to_string_pretty(&cfg).expect("serialize");
    let loaded: SandboxConfig = serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(cfg, loaded);
}

#[test]
fn test_config_bounds_enforcement() {
    // 1. Invalid version
    let mut bad_ver = SandboxConfig::default();
    bad_ver.version = "2.0.0".to_string();
    assert!(bad_ver.validate().is_err());

    // 2. Empty profile name
    let mut bad_profile = SandboxConfig::default();
    bad_profile.default_profile_name = "  ".to_string();
    assert!(bad_profile.validate().is_err());

    // 3. Output capture below MIN (1024) or above MAX (64 MiB)
    let mut bad_output = SandboxConfig::default();
    bad_output.max_output_capture_bytes = 512;
    assert!(bad_output.validate().is_err());

    bad_output.max_output_capture_bytes = 65 * 1024 * 1024;
    assert!(bad_output.validate().is_err());

    // 4. Timeout below MIN (1) or above MAX (86400)
    let mut bad_timeout = SandboxConfig::default();
    bad_timeout.execution_timeout_seconds = 0;
    assert!(bad_timeout.validate().is_err());

    bad_timeout.execution_timeout_seconds = 100_000;
    assert!(bad_timeout.validate().is_err());

    // 5. Registered profiles capacity
    let mut bad_profs = SandboxConfig::default();
    bad_profs.max_registered_profiles = 0;
    assert!(bad_profs.validate().is_err());

    bad_profs.max_registered_profiles = 2000;
    assert!(bad_profs.validate().is_err());

    // 6. Directory traversal in custom_profiles_dir
    let mut bad_dir = SandboxConfig::default();
    bad_dir.custom_profiles_dir = Some(PathBuf::from("/etc/../shadow"));
    assert!(bad_dir.validate().is_err());
}

#[test]
fn test_config_file_persistence() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("sub").join("sandbox_conf.json");

    let mut cfg = SandboxConfig::default();
    cfg.default_profile_name = "permissive".into();
    cfg.execution_timeout_seconds = 120;

    assert!(cfg.save_to_path(&file_path).is_ok());
    assert!(file_path.exists());

    let loaded = SandboxConfig::load_from_path(&file_path).expect("load");
    assert_eq!(cfg, loaded);
}

#[test]
fn test_config_file_size_limit() {
    let dir = tempdir().unwrap();
    let huge_file = dir.path().join("oversized.json");
    let oversized_data = vec![b'a'; (MAX_CONFIG_FILE_BYTES + 1024) as usize];
    std::fs::write(&huge_file, &oversized_data).unwrap();

    let res = SandboxConfig::load_from_path(&huge_file);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains(SANDBOXCONF_ERR_BOUNDS));
}

#[test]
fn test_config_env_overrides() {
    std::env::set_var("AIOS_SANDBOX_DEFAULT_PROFILE", "strict");
    std::env::set_var("AIOS_SANDBOX_ENFORCE_PEP", "1");
    std::env::set_var("AIOS_SANDBOX_MAX_OUTPUT_BYTES", "4194304"); // 4 MiB
    std::env::set_var("AIOS_SANDBOX_TIMEOUT_SECS", "45");

    let cfg = SandboxConfig::load_with_env_overrides();
    assert_eq!(cfg.default_profile_name, "strict");
    assert!(cfg.enforce_pep_grants);
    assert_eq!(cfg.max_output_capture_bytes, 4194304);
    assert_eq!(cfg.execution_timeout_seconds, 45);

    // Clean up
    std::env::remove_var("AIOS_SANDBOX_DEFAULT_PROFILE");
    std::env::remove_var("AIOS_SANDBOX_ENFORCE_PEP");
    std::env::remove_var("AIOS_SANDBOX_MAX_OUTPUT_BYTES");
    std::env::remove_var("AIOS_SANDBOX_TIMEOUT_SECS");
}
