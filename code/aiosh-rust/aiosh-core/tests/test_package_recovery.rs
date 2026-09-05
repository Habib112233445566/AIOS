//! Focused automated unit and integration tests for Package Management Recovery & Validation.
//!
//! Enforces invariants RV1..RV4 and verifies non-destructive quarantine and healing.

use std::path::Path;
use aiosh_core::package::{PackageDependency, PackageFormat, PackageSpec, PackageState};
use aiosh_core::package_recovery::{
    load_or_recover, recover_package_store_with_backup, validate_package_store,
    PackageValidationReport,
};
use aiosh_core::package_service::PackageStore;

#[test]
fn test_default_store_validation() {
    let store = PackageStore::new();
    let report = validate_package_store(&store, Path::new("/var/lib/aios/packages.json"));

    assert!(report.healthy, "default store must be healthy");
    assert_eq!(report.invalid_packages, 0);
    assert!(report.valid_packages >= 5);
    assert_eq!(report.valid_packages, report.total_packages);
    assert!(report.errors.is_empty());
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_invariant_equations_rv1_rv2_rv3() {
    // 1. RV1 violation: valid + invalid != total
    let report_bad_rv1 = PackageValidationReport {
        store_path: "/tmp/pkg.json".into(),
        total_packages: 10,
        valid_packages: 5,
        invalid_packages: 3, // sum is 8 != 10
        errors: vec!["err1".into(), "err2".into(), "err3".into()],
        healthy: false,
        evaluated_at: "2026-09-05T00:00:00Z".into(),
    };
    let err = report_bad_rv1.validate_invariants().unwrap_err();
    assert!(err.contains("RV1 violated"));

    // 2. RV2 violation: healthy is true but errors exist
    let report_bad_rv2a = PackageValidationReport {
        store_path: "/tmp/pkg.json".into(),
        total_packages: 2,
        valid_packages: 2,
        invalid_packages: 0,
        errors: vec!["hidden error".into()],
        healthy: true, // violation
        evaluated_at: "2026-09-05T00:00:00Z".into(),
    };
    let err = report_bad_rv2a.validate_invariants().unwrap_err();
    assert!(err.contains("RV2 violated"));

    // 3. RV2 violation: healthy is true but invalid_packages > 0
    let report_bad_rv2b = PackageValidationReport {
        store_path: "/tmp/pkg.json".into(),
        total_packages: 2,
        valid_packages: 1,
        invalid_packages: 1,
        errors: vec![],
        healthy: true, // violation
        evaluated_at: "2026-09-05T00:00:00Z".into(),
    };
    let err = report_bad_rv2b.validate_invariants().unwrap_err();
    assert!(err.contains("RV2 violated"));

    // 4. RV3 violation: errors.len() < invalid_packages
    let report_bad_rv3 = PackageValidationReport {
        store_path: "/tmp/pkg.json".into(),
        total_packages: 5,
        valid_packages: 2,
        invalid_packages: 3,
        errors: vec!["single error".into()], // only 1 error for 3 invalid packages
        healthy: false,
        evaluated_at: "2026-09-05T00:00:00Z".into(),
    };
    let err = report_bad_rv3.validate_invariants().unwrap_err();
    assert!(err.contains("RV3 violated"));

    // 5. Valid report satisfying all invariants
    let report_valid = PackageValidationReport {
        store_path: "/tmp/pkg.json".into(),
        total_packages: 5,
        valid_packages: 3,
        invalid_packages: 2,
        errors: vec!["err 1".into(), "err 2".into()],
        healthy: false,
        evaluated_at: "2026-09-05T00:00:00Z".into(),
    };
    assert!(report_valid.validate_invariants().is_ok());
}

#[test]
fn test_negative_package_specs_and_store_constraints() {
    let mut store = PackageStore::empty();

    // 1. Invalid package name syntax (uppercase)
    store.packages.insert(
        "Bad-Pkg".into(),
        PackageSpec {
            name: "Bad-Pkg".into(),
            version: "1.0.0".into(),
            architecture: "amd64".into(),
            format: PackageFormat::Deb,
            state: PackageState::Available,
            description: "Uppercase name".into(),
            installed_size_bytes: 1024,
            sha256: Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into()),
            repository_url: None,
            dependencies: vec![],
        },
    );

    // 2. Empty version
    store.packages.insert(
        "empty-version-pkg".into(),
        PackageSpec {
            name: "empty-version-pkg".into(),
            version: "".into(),
            architecture: "amd64".into(),
            format: PackageFormat::Deb,
            state: PackageState::Available,
            description: "No version".into(),
            installed_size_bytes: 1024,
            sha256: Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into()),
            repository_url: None,
            dependencies: vec![],
        },
    );

    // 3. Self-referential dependency
    store.packages.insert(
        "self-dep".into(),
        PackageSpec {
            name: "self-dep".into(),
            version: "1.0.0".into(),
            architecture: "amd64".into(),
            format: PackageFormat::Deb,
            state: PackageState::Available,
            description: "Self depending".into(),
            installed_size_bytes: 1024,
            sha256: Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into()),
            repository_url: None,
            dependencies: vec![PackageDependency {
                name: "self-dep".into(),
                version_constraint: None,
                optional: false,
            }],
        },
    );

    // 4. Invalid SHA-256 length / non-hex
    store.packages.insert(
        "bad-sha".into(),
        PackageSpec {
            name: "bad-sha".into(),
            version: "1.0.0".into(),
            architecture: "amd64".into(),
            format: PackageFormat::Deb,
            state: PackageState::Available,
            description: "Bad checksum".into(),
            installed_size_bytes: 1024,
            sha256: Some("not-a-valid-sha256-digest".into()),
            repository_url: None,
            dependencies: vec![],
        },
    );

    // 5. Store key does not match spec.name
    store.packages.insert(
        "mismatched-key".into(),
        PackageSpec {
            name: "actual-name".into(),
            version: "1.0.0".into(),
            architecture: "amd64".into(),
            format: PackageFormat::Deb,
            state: PackageState::Available,
            description: "Mismatched key".into(),
            installed_size_bytes: 1024,
            sha256: Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into()),
            repository_url: None,
            dependencies: vec![],
        },
    );

    let report = validate_package_store(&store, Path::new("/tmp/test_invalid.json"));
    assert!(!report.healthy);
    assert_eq!(report.total_packages, 5);
    assert_eq!(report.valid_packages, 0);
    assert_eq!(report.invalid_packages, 5);
    assert!(report.errors.len() >= 5);
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_recover_corrupted_json_store_rv4() {
    let tmp = tempfile::tempdir().unwrap();
    let store_file = tmp.path().join("packages.json");

    let corrupted_content = b"{\"corrupted\": [ incomplete json buffer";
    std::fs::write(&store_file, corrupted_content).unwrap();

    let (store, report, recovered, backup_opt) = load_or_recover(&store_file).unwrap();

    assert!(recovered, "must signal that recovery occurred");
    assert!(backup_opt.is_some(), "must return path to backup file");

    let backup_path = backup_opt.unwrap();
    assert!(backup_path.exists(), "backup file must exist on disk");
    let backed_up_bytes = std::fs::read(&backup_path).unwrap();
    assert_eq!(
        backed_up_bytes, corrupted_content,
        "backup file must bit-identically preserve damaged state (RV4)"
    );

    assert!(report.healthy);
    assert_eq!(report.invalid_packages, 0);
    assert!(store.packages.len() >= 5);
    assert!(store_file.exists());
}

#[test]
fn test_recover_missing_store_file() {
    let tmp = tempfile::tempdir().unwrap();
    let store_file = tmp.path().join("non_existent_packages.json");

    assert!(!store_file.exists());
    let (store, report, recovered, backup_opt) = load_or_recover(&store_file).unwrap();

    assert!(recovered);
    assert!(backup_opt.is_none(), "no backup needed for missing file");
    assert!(store_file.exists(), "must have created fresh store on disk");
    assert!(report.healthy);
    assert_eq!(report.valid_packages, store.packages.len());
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_healthy_store_no_recovery() {
    let tmp = tempfile::tempdir().unwrap();
    let store_file = tmp.path().join("clean_packages.json");

    let initial_store = PackageStore::new();
    initial_store.save_to_path(&store_file).unwrap();

    let (store, report, recovered, backup_opt) = load_or_recover(&store_file).unwrap();

    assert!(!recovered, "healthy store must not be re-recovered");
    assert!(backup_opt.is_none(), "no backup created for healthy store");
    assert!(report.healthy);
    assert_eq!(store.packages.len(), initial_store.packages.len());
}

#[test]
fn test_corrupted_store_invalid_specs_triggers_backup() {
    let tmp = tempfile::tempdir().unwrap();
    let store_file = tmp.path().join("invalid_specs_packages.json");

    let mut store = PackageStore::empty();
    store.packages.insert(
        "INVALID_NAME".into(),
        PackageSpec {
            name: "INVALID_NAME".into(),
            version: "".into(),
            architecture: "".into(),
            format: PackageFormat::Deb,
            state: PackageState::Available,
            description: "".into(),
            installed_size_bytes: 0,
            sha256: None,
            repository_url: None,
            dependencies: vec![],
        },
    );

    // Save directly to disk (bypassing normal store validation)
    let json_bytes = serde_json::to_string_pretty(&store.packages).unwrap();
    std::fs::write(&store_file, json_bytes.as_bytes()).unwrap();

    let (_fresh_store, report, recovered, backup_opt) = load_or_recover(&store_file).unwrap();

    assert!(recovered, "must recover store containing invalid specs");
    assert!(backup_opt.is_some(), "must create backup of invalid spec store");
    assert!(report.healthy, "reseeded store must be healthy");
}

#[test]
fn test_recover_package_store_with_backup_direct() {
    let tmp = tempfile::tempdir().unwrap();
    let store_file = tmp.path().join("direct_test.json");

    // Case 1: missing
    let (store, backup) = recover_package_store_with_backup(&store_file);
    assert!(backup.is_none());
    assert!(store.packages.len() >= 5);

    // Case 2: corrupted
    std::fs::write(&store_file, b"MALFORMED").unwrap();
    let (store, backup) = recover_package_store_with_backup(&store_file);
    assert!(backup.is_some());
    assert!(store.packages.len() >= 5);
}
