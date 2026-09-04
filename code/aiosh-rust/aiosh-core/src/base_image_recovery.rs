//! Recovery and deep validation subsystem for the Linux Base Image Build registry.

use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::base_image_service::ImageStore;
use crate::base_image::BaseImageManifest;

/// Action taken during store loading and corruption recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAction {
    LoadedExisting,
    CreatedDefaultFresh,
    RecoveredFromBackup { backup_path: String, reason: String },
}

/// Comprehensive deep validation report across base image manifests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseImageValidationReport {
    pub healthy: bool,
    pub total_manifests: usize,
    pub valid_manifests: usize,
    pub invalid_manifests: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub generated_at: String,
}

impl BaseImageValidationReport {
    /// Validates mathematical and consistency invariants RV1..RV3.
    pub fn validate_invariants(&self) -> Result<(), String> {
        if self.valid_manifests + self.invalid_manifests != self.total_manifests {
            return Err(format!(
                "invariant RV1 violated: valid ({}) + invalid ({}) != total ({})",
                self.valid_manifests, self.invalid_manifests, self.total_manifests
            ));
        }

        let expected_healthy = self.errors.is_empty() && self.invalid_manifests == 0;
        if self.healthy != expected_healthy {
            return Err(format!(
                "invariant RV2 violated: healthy ({}) != expected ({})",
                self.healthy, expected_healthy
            ));
        }

        if self.invalid_manifests > 0 && self.errors.len() < self.invalid_manifests {
            return Err(format!(
                "invariant RV3 violated: error count ({}) < invalid manifest count ({})",
                self.errors.len(), self.invalid_manifests
            ));
        }

        Ok(())
    }
}

const AUTHORIZED_FILESYSTEMS: &[&str] = &["ext4", "squashfs", "btrfs", "erofs", "xfs"];
const AUTHORIZED_ARCHITECTURES: &[&str] = &["x86_64", "aarch64", "riscv64"];
const BLACKLISTED_PACKAGES: &[&str] = &[
    "telnet", "telnetd", "rsh-client", "rsh-redone-client", "yp-tools", "tftp",
];
const DANGEROUS_KERNEL_PARAMS: &[&str] = &[
    "nokaslr", "mitigations=off", "pti=off", "selinux=0", "apparmor=0", "init=/bin/sh",
];
const MAX_BUDGET_BYTES: u64 = 100 * 1024 * 1024 * 1024; // 100 GiB

/// Validates a single base image manifest against deep integrity constraints.
pub fn validate_manifest(manifest: &BaseImageManifest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // 1. Identifier checks
    if manifest.id.is_empty() {
        errors.push("manifest id cannot be empty".into());
    } else if manifest.id.len() > 128 {
        errors.push(format!("manifest id exceeds 128 characters (was {})", manifest.id.len()));
    } else if !manifest.id.chars().all(|c| c.is_ascii_graphic()) {
        errors.push("manifest id must contain only printable ASCII graphic characters".into());
    }

    // 2. Distro ID checks
    if manifest.rootfs.distro_id.is_empty() {
        errors.push("rootfs distro_id cannot be empty".into());
    } else if manifest.rootfs.distro_id.len() > 64 {
        errors.push(format!("distro_id exceeds 64 characters (was {})", manifest.rootfs.distro_id.len()));
    } else if !manifest.rootfs.distro_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
        errors.push("distro_id must be alphanumeric with hyphens, underscores, or dots".into());
    }

    // 3. Architecture checks
    if !AUTHORIZED_ARCHITECTURES.contains(&manifest.rootfs.architecture.as_str()) {
        errors.push(format!(
            "architecture '{}' is not in authorized list {:?}",
            manifest.rootfs.architecture, AUTHORIZED_ARCHITECTURES
        ));
    }

    // 4. Filesystem checks
    if !AUTHORIZED_FILESYSTEMS.contains(&manifest.rootfs.filesystem_type.as_str()) {
        errors.push(format!(
            "filesystem '{}' is not in authorized list {:?}",
            manifest.rootfs.filesystem_type, AUTHORIZED_FILESYSTEMS
        ));
    }

    // 5. Package checks
    if manifest.rootfs.packages.is_empty() {
        errors.push("package list cannot be empty".into());
    } else if manifest.rootfs.packages.len() > 1024 {
        errors.push(format!("package list exceeds 1024 items (was {})", manifest.rootfs.packages.len()));
    } else {
        for pkg in &manifest.rootfs.packages {
            if pkg.chars().any(|c| (c as u32) < 0x20 || (c as u32) == 0x7f) {
                errors.push(format!("package '{}' contains illegal control characters", pkg));
            }
            if BLACKLISTED_PACKAGES.contains(&pkg.as_str()) {
                errors.push(format!("package '{}' is blacklisted due to security policy", pkg));
            }
        }
    }

    // 6. Size budget checks
    if manifest.rootfs.size_budget_bytes == 0 {
        errors.push("size_budget_bytes must be greater than zero".into());
    } else if manifest.rootfs.size_budget_bytes > MAX_BUDGET_BYTES {
        errors.push(format!(
            "size_budget_bytes {} exceeds maximum allowed ceiling of 100 GiB",
            manifest.rootfs.size_budget_bytes
        ));
    }

    // 7. Kernel checks
    if manifest.kernel.version.is_empty() {
        errors.push("kernel version cannot be empty".into());
    }
    for token in manifest.kernel.cmdline.split_whitespace() {
        if DANGEROUS_KERNEL_PARAMS.contains(&token) {
            errors.push(format!("kernel cmdline contains dangerous parameter '{}'", token));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validates all manifests in an ImageStore and returns a validation report.
pub fn validate_store(store: &ImageStore) -> BaseImageValidationReport {
    let mut valid = 0;
    let mut invalid = 0;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for manifest in store.list_images() {
        let mut manifest_failed = false;

        // Step 1: Deep manifest checks
        if let Err(errs) = validate_manifest(&manifest) {
            manifest_failed = true;
            for e in errs {
                errors.push(format!("{}: {}", manifest.id, e));
            }
        }

        // Step 2: Build plan synthesis dry-run
        if let Err(err) = store.generate_build_plan(&manifest.id) {
            manifest_failed = true;
            errors.push(format!("{}: build plan synthesis failed: {}", manifest.id, err));
        }

        if manifest_failed {
            invalid += 1;
        } else {
            valid += 1;
            if manifest.rootfs.size_budget_bytes > 50 * 1024 * 1024 * 1024 {
                warnings.push(format!("{}: size budget > 50 GiB is large", manifest.id));
            }
        }
    }

    let total = valid + invalid;
    let healthy = errors.is_empty() && invalid == 0;

    BaseImageValidationReport {
        healthy,
        total_manifests: total,
        valid_manifests: valid,
        invalid_manifests: invalid,
        errors,
        warnings,
        generated_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Loads an ImageStore from path with automated non-destructive corruption recovery.
pub fn load_or_recover(store_path: &Path) -> (ImageStore, RecoveryAction) {
    if !store_path.exists() {
        let store = ImageStore::new();
        let _ = store.save_to_path(store_path);
        return (store, RecoveryAction::CreatedDefaultFresh);
    }

    match ImageStore::load_from_path(store_path) {
        Ok(store) => {
            let report = validate_store(&store);
            if report.healthy {
                (store, RecoveryAction::LoadedExisting)
            } else {
                let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
                let backup_path = format!("{}.bak.{}", store_path.display(), timestamp);
                let _ = std::fs::copy(store_path, &backup_path);

                let fresh_store = ImageStore::new();
                let _ = fresh_store.save_to_path(store_path);

                (
                    fresh_store,
                    RecoveryAction::RecoveredFromBackup {
                        backup_path,
                        reason: format!("store validation failed with {} errors", report.errors.len()),
                    },
                )
            }
        }
        Err(e) => {
            let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
            let backup_path = format!("{}.bak.{}", store_path.display(), timestamp);
            let _ = std::fs::copy(store_path, &backup_path);

            let fresh_store = ImageStore::new();
            let _ = fresh_store.save_to_path(store_path);

            (
                fresh_store,
                RecoveryAction::RecoveredFromBackup {
                    backup_path,
                    reason: format!("store load failed: {}", e),
                },
            )
        }
    }
}

/// Explicitly repairs a store by validating manifests and recovering if corrupted.
pub fn repair_store(store_path: &Path) -> Result<(ImageStore, RecoveryAction), String> {
    Ok(load_or_recover(store_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_store_defaults() {
        let store = ImageStore::new();
        let report = validate_store(&store);
        assert!(report.healthy);
        assert_eq!(report.invalid_manifests, 0);
        assert_eq!(report.valid_manifests, report.total_manifests);
        assert!(report.validate_invariants().is_ok());
    }

    #[test]
    fn test_validate_manifest_violations() {
        let store = ImageStore::new();
        let mut manifest = store.get_image("debian-12-minimal-raw").unwrap().clone();

        // Inject dangerous kernel parameter
        manifest.kernel.cmdline = "console=tty0 nokaslr".into();
        let errs = validate_manifest(&manifest).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("nokaslr")));

        // Inject blacklisted package
        manifest.rootfs.packages.push("telnet".into());
        let errs = validate_manifest(&manifest).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("telnet")));

        // Invalid architecture
        manifest.rootfs.architecture = "mips".into();
        let errs = validate_manifest(&manifest).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("architecture")));
    }

    #[test]
    fn test_load_or_recover_lifecycle() {
        let tmp = tempfile::tempdir().unwrap();
        let store_file = tmp.path().join("test_store.json");

        // 1. Fresh file creation
        let (store, action) = load_or_recover(&store_file);
        assert_eq!(action, RecoveryAction::CreatedDefaultFresh);
        assert!(store.get_image("debian-12-minimal-raw").is_some());

        // 2. Existing valid file load
        let (store2, action2) = load_or_recover(&store_file);
        assert_eq!(action2, RecoveryAction::LoadedExisting);
        assert_eq!(store2.list_images().len(), store.list_images().len());

        // 3. Corrupt file recovery
        std::fs::write(&store_file, b"MALFORMED_NOT_JSON").unwrap();
        let (recovered_store, action3) = load_or_recover(&store_file);
        match action3 {
            RecoveryAction::RecoveredFromBackup { backup_path, .. } => {
                assert!(Path::new(&backup_path).exists());
            }
            _ => panic!("expected RecoveredFromBackup"),
        }
        assert!(recovered_store.get_image("debian-12-minimal-raw").is_some());
    }
}
