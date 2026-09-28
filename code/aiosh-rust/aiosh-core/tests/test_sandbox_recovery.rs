//! Unit tests for Sandbox Enforcement Recovery & Validation Subsystem (T-02494/T-02495).

use std::fs;
use tempfile::tempdir;

use aiosh_core::sandbox_data_model::*;
use aiosh_core::sandbox_recovery::*;
use aiosh_core::sandbox_service::*;

#[test]
fn test_sandbox_recovery_healthy_default() {
    let svc = SandboxService::with_default_profiles(None);
    let report = svc.validate_state(None);

    assert!(report.is_healthy, "Default factory service should be healthy");
    assert!(report.factory_profiles_intact);
    assert_eq!(report.valid_profiles_count, 3);
    assert_eq!(report.corrupt_profiles_count, 0);
    assert!(report.issues.is_empty());
}

#[test]
fn test_sandbox_recovery_missing_factory_profile_and_restore() {
    // Intentionally create an empty service without factory profiles
    // by using SandboxService::empty and registering a single custom profile
    let mut custom_svc = SandboxService::empty(None, aiosh_core::sandbox_config::SandboxConfig::default());
    let custom_profile = SandboxProfileBuilder::new("my_custom")
        .build()
        .expect("build custom");
    custom_svc.register_profile(custom_profile).unwrap();

    let report = custom_svc.validate_state(None);
    assert!(!report.is_healthy, "Service missing factory profiles must be unhealthy");
    assert!(!report.factory_profiles_intact);
    assert!(report.issues.iter().any(|i| i.code == SANDBOXRECV_ERR_MISSING_FACTORY));

    // Execute recovery with RestoreFactoryDefaults
    let res = custom_svc.recover_state(SandboxRecoveryStrategy::RestoreFactoryDefaults, None).expect("recover");
    assert!(res.success);
    assert_eq!(res.profiles_restored, 3);

    // Verify service is now healthy and factory profiles exist
    let report2 = custom_svc.validate_state(None);
    assert!(report2.is_healthy);
    assert!(report2.factory_profiles_intact);
}

#[test]
fn test_sandbox_recovery_corrupt_manifest_quarantine() {
    let dir = tempdir().unwrap();
    let corrupt_file = dir.path().join("corrupt_profile.json");
    fs::write(&corrupt_file, "{ malformed json ").unwrap();

    let valid_profile = SandboxProfileBuilder::new("valid_custom")
        .build()
        .expect("valid profile");
    let valid_file = dir.path().join("valid_custom.json");
    fs::write(&valid_file, serde_json::to_string(&valid_profile).unwrap()).unwrap();

    let mut svc = SandboxService::with_default_profiles(None);

    // Validation detects the corrupt file in custom_dir
    let report = svc.validate_state(Some(dir.path()));
    assert!(!report.is_healthy);
    assert_eq!(report.corrupt_profiles_count, 1);
    assert!(report.issues.iter().any(|i| i.code == SANDBOXRECV_ERR_CORRUPT));

    // Recovery with QuarantineAndReset moves corrupt file
    let res = svc.recover_state(SandboxRecoveryStrategy::QuarantineAndReset, Some(dir.path())).expect("recover");
    assert!(res.success);
    assert!(res.quarantine_path.is_some());

    let q_path = std::path::PathBuf::from(res.quarantine_path.unwrap());
    assert!(q_path.exists());
    assert!(q_path.join("corrupt_profile.json").exists());
    assert!(!dir.path().join("corrupt_profile.json").exists());
    assert!(dir.path().join("valid_custom.json").exists());
}

#[test]
fn test_sandbox_recovery_traversal_detection() {
    let traversal_path = std::path::Path::new("../secret/profile.json");
    let res = SandboxRecoveryManager::validate_profile_file(traversal_path);
    assert!(res.is_err());
    let issues = res.unwrap_err();
    assert!(issues.iter().any(|i| i.code == SANDBOXRECV_ERR_TRAVERSAL));
}

#[test]
fn test_sandbox_recovery_oversized_file() {
    let dir = tempdir().unwrap();
    let big_file = dir.path().join("big_profile.json");
    // Create file exceeding 64 KiB
    let big_content = vec![b' '; (MAX_PROFILE_FILE_BYTES + 1024) as usize];
    fs::write(&big_file, big_content).unwrap();

    let res = SandboxRecoveryManager::validate_profile_file(&big_file);
    assert!(res.is_err());
    let issues = res.unwrap_err();
    assert!(issues.iter().any(|i| i.code == SANDBOXRECV_ERR_CORRUPT));
}

#[test]
fn test_sandbox_recovery_dry_run_strategy() {
    let mut svc = SandboxService::with_default_profiles(None);
    let res = svc.recover_state(SandboxRecoveryStrategy::DryRun, None).expect("dry run");
    assert!(res.success);
    assert_eq!(res.profiles_restored, 0);
    assert!(res.quarantine_path.is_none());
}

#[test]
fn test_sandbox_recovery_serialization() {
    let svc = SandboxService::with_default_profiles(None);
    let report = svc.validate_state(None);

    let json = serde_json::to_string(&report).expect("serialize report");
    assert!(json.contains("is_healthy"));
    assert!(json.contains("factory_profiles_intact"));

    let deserialized: SandboxValidationReport = serde_json::from_str(&json).expect("deserialize report");
    assert_eq!(deserialized.is_healthy, report.is_healthy);
}

