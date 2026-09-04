//! Focused automated unit tests for Base Image Build Recovery & Validation subsystem.

use std::path::Path;
use aiosh_core::base_image_recovery::{
    load_or_recover, repair_store, validate_manifest, validate_store,
    BaseImageValidationReport, RecoveryAction,
};
use aiosh_core::base_image_service::ImageStore;

#[test]
fn test_default_store_validation() {
    let store = ImageStore::new();
    let report = validate_store(&store);

    assert!(report.healthy, "default seeded store must be healthy");
    assert_eq!(report.invalid_manifests, 0);
    assert_eq!(report.valid_manifests, report.total_manifests);
    assert!(report.errors.is_empty());
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_manifest_boundary_and_negative_rules() {
    let store = ImageStore::new();
    let base_manifest = store.get_image("debian-12-minimal-raw").unwrap().clone();

    // 1. Empty ID
    let mut m = base_manifest.clone();
    m.id = "".into();
    let errs = validate_manifest(&m).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("id cannot be empty")));

    // 2. Oversized ID (>128 chars)
    let mut m = base_manifest.clone();
    m.id = "a".repeat(129);
    let errs = validate_manifest(&m).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("exceeds 128 characters")));

    // 3. ID with control characters
    let mut m = base_manifest.clone();
    m.id = "bad\x01id".into();
    let errs = validate_manifest(&m).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("printable ASCII graphic")));

    // 4. Unauthorized architecture
    let mut m = base_manifest.clone();
    m.rootfs.architecture = "sparc64".into();
    let errs = validate_manifest(&m).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("architecture")));

    // 5. Unauthorized filesystem
    let mut m = base_manifest.clone();
    m.rootfs.filesystem_type = "ntfs".into();
    let errs = validate_manifest(&m).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("filesystem")));

    // 6. Empty packages
    let mut m = base_manifest.clone();
    m.rootfs.packages.clear();
    let errs = validate_manifest(&m).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("package list cannot be empty")));

    // 7. Blacklisted package
    let mut m = base_manifest.clone();
    m.rootfs.packages.push("rsh-client".into());
    let errs = validate_manifest(&m).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("blacklisted")));

    // 8. Package with control char
    let mut m = base_manifest.clone();
    m.rootfs.packages.push("bad\x00pkg".into());
    let errs = validate_manifest(&m).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("control characters")));

    // 9. Dangerous kernel parameter
    let mut m = base_manifest.clone();
    m.kernel.cmdline = "console=ttyS0 mitigations=off".into();
    let errs = validate_manifest(&m).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("mitigations=off")));

    // 10. Zero size budget
    let mut m = base_manifest.clone();
    m.rootfs.size_budget_bytes = 0;
    let errs = validate_manifest(&m).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("greater than zero")));

    // 11. Oversized packages (>1024)
    let mut m = base_manifest.clone();
    m.rootfs.packages = (0..1025).map(|i| format!("pkg{}", i)).collect();
    let errs = validate_manifest(&m).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("exceeds 1024 items")));
}

#[test]
fn test_invariant_violations_rv1_rv2_rv3() {
    // RV1 violation: valid + invalid != total
    let report_bad_rv1 = BaseImageValidationReport {
        healthy: false,
        total_manifests: 5,
        valid_manifests: 2,
        invalid_manifests: 2, // 2 + 2 != 5
        errors: vec!["some error".into()],
        warnings: vec![],
        generated_at: "2026-09-04T06:00:00Z".into(),
    };
    assert!(report_bad_rv1.validate_invariants().is_err());

    // RV2 violation: healthy == true but invalid > 0
    let report_bad_rv2 = BaseImageValidationReport {
        healthy: true,
        total_manifests: 2,
        valid_manifests: 1,
        invalid_manifests: 1,
        errors: vec!["some error".into()],
        warnings: vec![],
        generated_at: "2026-09-04T06:00:00Z".into(),
    };
    assert!(report_bad_rv2.validate_invariants().is_err());

    // RV3 violation: invalid > 0 but errors is empty
    let report_bad_rv3 = BaseImageValidationReport {
        healthy: false,
        total_manifests: 2,
        valid_manifests: 1,
        invalid_manifests: 1,
        errors: vec![], // empty!
        warnings: vec![],
        generated_at: "2026-09-04T06:00:00Z".into(),
    };
    assert!(report_bad_rv3.validate_invariants().is_err());
}

#[test]
fn test_corruption_recovery_and_backup_creation() {
    let tmp = tempfile::tempdir().unwrap();
    let store_path = tmp.path().join("subsystem_images.json");

    // Phase 1: Missing file -> fresh default created
    let (s1, a1) = load_or_recover(&store_path);
    assert_eq!(a1, RecoveryAction::CreatedDefaultFresh);
    assert!(store_path.exists());
    assert!(s1.get_image("debian-12-minimal-raw").is_some());

    // Phase 2: Existing valid file -> loaded
    let (_s2, a2) = load_or_recover(&store_path);
    assert_eq!(a2, RecoveryAction::LoadedExisting);

    // Phase 3: Corrupted file (garbage text) -> backed up and reseeded
    std::fs::write(&store_path, b"{corrupted: true, not: json]").unwrap();
    let (s3, a3) = load_or_recover(&store_path);
    match a3 {
        RecoveryAction::RecoveredFromBackup { backup_path, reason } => {
            assert!(Path::new(&backup_path).exists());
            assert!(reason.contains("store load failed"));
        }
        _ => panic!("expected RecoveredFromBackup for corrupt file"),
    }
    assert!(s3.get_image("debian-12-minimal-raw").is_some());

    // Phase 4: Valid JSON but invalid manifest schema -> backed up and reseeded
    let mut invalid_store = ImageStore::empty();
    let mut bad_manifest = s1.get_image("debian-12-minimal-raw").unwrap().clone();
    bad_manifest.rootfs.architecture = "unsupported_arch".into();
    invalid_store.register_image(bad_manifest).unwrap();
    invalid_store.save_to_path(&store_path).unwrap();

    let (s4, a4) = load_or_recover(&store_path);
    match a4 {
        RecoveryAction::RecoveredFromBackup { backup_path, reason } => {
            assert!(Path::new(&backup_path).exists());
            assert!(reason.contains("validation failed"));
        }
        _ => panic!("expected RecoveredFromBackup for invalid manifest"),
    }
    assert!(s4.get_image("debian-12-minimal-raw").is_some());
}

#[test]
fn test_repair_store_api() {
    let tmp = tempfile::tempdir().unwrap();
    let store_path = tmp.path().join("repairable_images.json");

    std::fs::write(&store_path, b"SYNTAX ERROR").unwrap();
    let (store, action) = repair_store(&store_path).unwrap();
    assert!(store.get_image("debian-12-minimal-raw").is_some());
    match action {
        RecoveryAction::RecoveredFromBackup { backup_path, .. } => {
            assert!(Path::new(&backup_path).exists());
        }
        _ => panic!("expected recovery from backup"),
    }
}
