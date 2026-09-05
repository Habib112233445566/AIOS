//! Health check, validation, and corruption recovery for the Package Management subsystem.
//!
//! Provides automated non-destructive self-healing, timestamped quarantine of damaged
//! stores, and deep validation reports satisfying invariants RV1..RV4.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::package::{validate_package_name, validate_package_spec};
use crate::package_service::PackageStore;

/// Comprehensive validation report for a package store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageValidationReport {
    pub store_path: String,
    pub total_packages: usize,
    pub valid_packages: usize,
    pub invalid_packages: usize,
    pub errors: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}

impl PackageValidationReport {
    /// Validates internal consistency invariants RV1..RV3.
    pub fn validate_invariants(&self) -> Result<(), String> {
        // RV1: valid + invalid == total
        if self.valid_packages + self.invalid_packages != self.total_packages {
            return Err(format!(
                "RV1 violated: valid_packages ({}) + invalid_packages ({}) != total_packages ({})",
                self.valid_packages, self.invalid_packages, self.total_packages
            ));
        }

        // RV2: healthy == (errors.is_empty() && invalid_packages == 0)
        let expected_healthy = self.errors.is_empty() && self.invalid_packages == 0;
        if self.healthy != expected_healthy {
            return Err(format!(
                "RV2 violated: healthy ({}) != expected_healthy ({})",
                self.healthy, expected_healthy
            ));
        }

        // RV3: errors.len() >= invalid_packages
        if self.errors.len() < self.invalid_packages {
            return Err(format!(
                "RV3 violated: errors.len() ({}) < invalid_packages ({})",
                self.errors.len(), self.invalid_packages
            ));
        }

        Ok(())
    }
}

/// Validates all packages and structural constraints in a PackageStore.
pub fn validate_package_store(store: &PackageStore, store_path: &Path) -> PackageValidationReport {
    let mut errors = Vec::new();
    let mut valid_packages = 0;
    let mut invalid_packages = 0;

    let total_packages = store.packages.len();

    if total_packages > 10_000 {
        errors.push(format!(
            "store exceeds maximum capacity of 10,000 packages (was {})",
            total_packages
        ));
    }

    // Check each package in alphabetical order
    let mut sorted_entries: Vec<(&String, &crate::package::PackageSpec)> = store.packages.iter().collect();
    sorted_entries.sort_by(|a, b| a.0.cmp(b.0));

    for (key, spec) in sorted_entries {
        let mut pkg_errors = Vec::new();

        if key != &spec.name {
            pkg_errors.push(format!(
                "store key '{}' does not match package name '{}'",
                key, spec.name
            ));
        }

        if let Err(name_err) = validate_package_name(&spec.name) {
            pkg_errors.push(format!("invalid package name syntax: {}", name_err));
        }

        if let Err(spec_errs) = validate_package_spec(spec) {
            pkg_errors.extend(spec_errs);
        }

        if pkg_errors.is_empty() {
            valid_packages += 1;
        } else {
            invalid_packages += 1;
            for err in pkg_errors {
                errors.push(format!("package '{}': {}", spec.name, err));
            }
        }
    }

    let healthy = errors.is_empty() && invalid_packages == 0;

    let report = PackageValidationReport {
        store_path: store_path.to_string_lossy().to_string(),
        total_packages,
        valid_packages,
        invalid_packages,
        errors,
        healthy,
        evaluated_at: chrono::Utc::now().to_rfc3339(),
    };

    let _ = report.validate_invariants();
    report
}

/// Recovers a package store from disk, creating a timestamped backup if corrupted.
/// Implements RV4 (non-destructive forensic preservation).
pub fn recover_package_store_with_backup(path: &Path) -> (PackageStore, Option<PathBuf>) {
    if !path.exists() {
        let store = PackageStore::new();
        let _ = store.save_to_path(path);
        return (store, None);
    }

    match PackageStore::load_from_path(path) {
        Ok(store) => {
            let report = validate_package_store(&store, path);
            if report.healthy {
                (store, None)
            } else {
                let backup_path = create_backup_file(path);
                let fresh_store = PackageStore::new();
                let _ = fresh_store.save_to_path(path);
                (fresh_store, Some(backup_path))
            }
        }
        Err(_) => {
            let backup_path = create_backup_file(path);
            let fresh_store = PackageStore::new();
            let _ = fresh_store.save_to_path(path);
            (fresh_store, Some(backup_path))
        }
    }
}

/// High-level entrypoint: loads store, validates, and optionally recovers if damaged.
pub fn load_or_recover(
    path: &Path,
) -> Result<(PackageStore, PackageValidationReport, bool, Option<PathBuf>), String> {
    if !path.exists() {
        let store = PackageStore::new();
        store.save_to_path(path)?;
        let report = validate_package_store(&store, path);
        return Ok((store, report, true, None));
    }

    match PackageStore::load_from_path(path) {
        Ok(store) => {
            let report = validate_package_store(&store, path);
            if report.healthy {
                Ok((store, report, false, None))
            } else {
                let (recovered_store, backup_path) = recover_package_store_with_backup(path);
                let fresh_report = validate_package_store(&recovered_store, path);
                Ok((recovered_store, fresh_report, true, backup_path))
            }
        }
        Err(_) => {
            let (recovered_store, backup_path) = recover_package_store_with_backup(path);
            let fresh_report = validate_package_store(&recovered_store, path);
            Ok((recovered_store, fresh_report, true, backup_path))
        }
    }
}

/// Helper function to create a timestamped backup file (<path>.bak.<ts>).
fn create_backup_file(path: &Path) -> PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let base_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("packages.json");

    let mut backup_name = format!("{}.bak.{}", base_name, ts);
    let mut backup_path = path.with_file_name(&backup_name);

    let mut counter = 1;
    while backup_path.exists() {
        backup_name = format!("{}.bak.{}_{}", base_name, ts, counter);
        backup_path = path.with_file_name(&backup_name);
        counter += 1;
    }

    // Attempt rename first; if it fails (e.g. cross-filesystem), copy and remove.
    if std::fs::rename(path, &backup_path).is_err() {
        if std::fs::copy(path, &backup_path).is_ok() {
            let _ = std::fs::remove_file(path);
        }
    }

    backup_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::{PackageFormat, PackageSpec, PackageState};

    #[test]
    fn test_validate_default_store_healthy() {
        let store = PackageStore::new();
        let report = validate_package_store(&store, Path::new("/var/lib/aios/packages.json"));

        assert!(report.healthy);
        assert_eq!(report.invalid_packages, 0);
        assert_eq!(report.errors.len(), 0);
        assert_eq!(report.valid_packages, report.total_packages);
        assert!(report.total_packages >= 5);
        assert!(report.validate_invariants().is_ok());
    }

    #[test]
    fn test_validate_store_with_invalid_package() {
        let mut store = PackageStore::empty();
        // Valid package
        store.packages.insert(
            "valid-pkg".into(),
            PackageSpec {
                name: "valid-pkg".into(),
                version: "1.0.0".into(),
                architecture: "amd64".into(),
                format: PackageFormat::Deb,
                state: PackageState::Available,
                description: "Valid package".into(),
                installed_size_bytes: 1024,
                sha256: Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into()),
                repository_url: None,
                dependencies: vec![],
            },
        );

        // Invalid package (bad name, bad version, bad sha)
        store.packages.insert(
            "INVALID_NAME".into(),
            PackageSpec {
                name: "INVALID_NAME".into(),
                version: "".into(),
                architecture: "".into(),
                format: PackageFormat::Deb,
                state: PackageState::Available,
                description: "Invalid package".into(),
                installed_size_bytes: 1024,
                sha256: Some("short".into()),
                repository_url: None,
                dependencies: vec![],
            },
        );

        let report = validate_package_store(&store, Path::new("/tmp/test_packages.json"));
        assert!(!report.healthy);
        assert_eq!(report.total_packages, 2);
        assert_eq!(report.valid_packages, 1);
        assert_eq!(report.invalid_packages, 1);
        assert!(!report.errors.is_empty());
        assert!(report.validate_invariants().is_ok());
    }

    #[test]
    fn test_recover_corrupted_json_file() {
        let tmp = tempfile::tempdir().unwrap();
        let store_file = tmp.path().join("packages.json");

        // Write damaged JSON content
        std::fs::write(&store_file, b"{\"packages\": TRUNCATED_GARBAGE").unwrap();

        let (_store, report, recovered, backup_opt) = load_or_recover(&store_file).unwrap();
        assert!(recovered);
        assert!(backup_opt.is_some());

        let backup_path = backup_opt.unwrap();
        assert!(backup_path.exists());
        assert_eq!(
            std::fs::read(&backup_path).unwrap(),
            b"{\"packages\": TRUNCATED_GARBAGE"
        );

        // Recovered store must be healthy
        assert!(report.healthy);
        assert!(report.total_packages >= 5);
        assert!(store_file.exists());
    }

    #[test]
    fn test_recover_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let store_file = tmp.path().join("new_packages.json");

        assert!(!store_file.exists());
        let (store, report, recovered, backup_opt) = load_or_recover(&store_file).unwrap();
        assert!(recovered);
        assert!(backup_opt.is_none());
        assert!(store_file.exists());
        assert!(report.healthy);
        assert_eq!(report.valid_packages, store.packages.len());
    }
}
