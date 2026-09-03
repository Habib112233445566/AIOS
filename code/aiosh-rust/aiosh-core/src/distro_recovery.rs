//! Health check and corruption recovery module for Linux Distro Selection subsystem.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::distro::validate_distro_profile;
use crate::distro_service::DistroStore;

/// Detailed health check report for a distro store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistroHealthReport {
    pub healthy: bool,
    pub profile_count: usize,
    pub checked_profiles: Vec<String>,
    pub recommended_profile_valid: bool,
    pub errors: Vec<String>,
    pub evaluated_at: String,
}

impl DistroHealthReport {
    /// Validates internal consistency invariants V1..V2.
    pub fn validate(&self) -> Result<(), String> {
        if self.healthy != self.errors.is_empty() {
            return Err("invariant V1 violated: healthy flag does not match errors emptiness".into());
        }
        if self.healthy && !self.recommended_profile_valid {
            return Err("invariant V2 violated: store cannot be healthy without a valid recommended profile".into());
        }
        Ok(())
    }
}

/// Validates all profiles and overall integrity of a DistroStore.
pub fn validate_store_health(store: &DistroStore) -> DistroHealthReport {
    let mut errors = Vec::new();
    let profiles = store.list_profiles();
    let mut checked_profiles = Vec::new();

    if profiles.is_empty() {
        errors.push("distro store is empty: contains zero registered profiles".into());
    }

    for p in &profiles {
        checked_profiles.push(p.id.clone());
        if let Err(e) = validate_distro_profile(p) {
            errors.push(format!("profile '{}' failed validation: {}", p.id, e));
        }
    }

    let recommended = store.get_recommended_profile();
    let recommended_profile_valid = if let Some(rec) = recommended {
        match store.evaluate_profile(&rec.id) {
            Ok(eval) => eval.is_production_ready,
            Err(e) => {
                errors.push(format!("recommended profile '{}' evaluation error: {}", rec.id, e));
                false
            }
        }
    } else {
        errors.push("no recommended profile designated in distro store".into());
        false
    };

    let healthy = errors.is_empty();
    DistroHealthReport {
        healthy,
        profile_count: profiles.len(),
        checked_profiles,
        recommended_profile_valid,
        errors,
        evaluated_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Resiliently loads a DistroStore from disk. If the target file is corrupted or
/// malformed, preserves the damaged file as `<path>.corrupt.<timestamp>.bak`
/// and initializes a fresh canonical default store at `<path>`.
pub fn recover_with_backup(path: &Path) -> (DistroStore, Option<PathBuf>) {
    if !path.exists() {
        let store = DistroStore::new();
        return (store, None);
    }

    match DistroStore::load_from_path(path) {
        Ok(store) => (store, None),
        Err(_) => {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let base_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("distro_store.json");
            let mut backup_name = format!("{}.corrupt.{}.bak", base_name, ts);
            let mut backup_path = path.with_file_name(&backup_name);

            let mut counter = 1;
            while backup_path.exists() {
                backup_name = format!("{}.corrupt.{}_{}.bak", base_name, ts, counter);
                backup_path = path.with_file_name(&backup_name);
                counter += 1;
            }

            let _ = std::fs::rename(path, &backup_path);

            let fresh_store = DistroStore::new();
            let _ = fresh_store.save_to_path(path);
            (fresh_store, Some(backup_path))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_validate_store_health_canonical() {
        let store = DistroStore::new();
        let report = validate_store_health(&store);

        assert!(report.healthy);
        assert_eq!(report.errors.len(), 0);
        assert!(report.profile_count >= 2);
        assert!(report.recommended_profile_valid);
        assert!(report.validate().is_ok());
    }

    #[test]
    fn test_validate_store_health_empty() {
        let store = DistroStore::empty();
        let report = validate_store_health(&store);

        assert!(!report.healthy);
        assert!(!report.recommended_profile_valid);
        assert!(report.errors.len() >= 1);
        assert!(report.validate().is_ok());
    }

    #[test]
    fn test_recover_with_backup_corrupted_file() {
        let tmp = tempfile::tempdir().unwrap();
        let corrupt_path = tmp.path().join("distro_store.json");

        // Write damaged/invalid JSON
        std::fs::write(&corrupt_path, b"NOT VALID JSON").unwrap();

        let (store, backup_opt) = recover_with_backup(&corrupt_path);
        assert!(backup_opt.is_some());
        let backup_path = backup_opt.unwrap();
        assert!(backup_path.exists());
        assert_eq!(std::fs::read(&backup_path).unwrap(), b"NOT VALID JSON");

        // Newly restored file should be valid
        assert!(corrupt_path.exists());
        let health = validate_store_health(&store);
        assert!(health.healthy);
    }

    #[test]
    fn test_distro_health_report_validation_invariants() {
        let mut report = DistroHealthReport {
            healthy: true,
            profile_count: 1,
            checked_profiles: vec!["p1".into()],
            recommended_profile_valid: false, // Invariant V2 violation
            errors: vec![],
            evaluated_at: "2026-09-03T12:00:00Z".into(),
        };
        assert!(report.validate().is_err());

        report.healthy = false;
        report.errors.push("some error".into());
        assert!(report.validate().is_ok());
    }

    #[test]
    fn test_distro_health_report_json_roundtrip() {
        let store = DistroStore::new();
        let report = validate_store_health(&store);
        let json_str = serde_json::to_string(&report).unwrap();
        let deserialized: DistroHealthReport = serde_json::from_str(&json_str).unwrap();
        assert_eq!(report, deserialized);
    }

    #[test]
    fn test_recover_with_backup_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let missing_path = tmp.path().join("non_existent_distro_store.json");
        let (store, backup_opt) = recover_with_backup(&missing_path);
        assert!(backup_opt.is_none());
        assert_eq!(store.list_profiles().len(), 2);
    }
}
