//! AIOS Linux Distro Store & Evaluation Service.
//!
//! Manages Linux distribution profile registries, persistence,
//! and scoring evaluations.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::distro::{validate_distro_profile, DistroEvaluation, DistroProfile};

/// In-memory store and evaluation service for Linux distro profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroStore {
    profiles: HashMap<String, DistroProfile>,
}

impl Default for DistroStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DistroStore {
    /// Creates a new DistroStore initialized with canonical default profiles.
    pub fn new() -> Self {
        let mut store = Self {
            profiles: HashMap::new(),
        };
        let debian = DistroProfile::debian_12_bookworm_x86_64();
        let alpine = DistroProfile::alpine_319_x86_64();
        store.profiles.insert(debian.id.clone(), debian);
        store.profiles.insert(alpine.id.clone(), alpine);
        store
    }

    /// Registers a custom or updated distro profile after validation.
    pub fn register_profile(&mut self, profile: DistroProfile) -> Result<(), String> {
        validate_distro_profile(&profile)?;
        self.profiles.insert(profile.id.clone(), profile);
        Ok(())
    }

    /// Retrieves a reference to a profile by ID.
    pub fn get_profile(&self, id: &str) -> Option<&DistroProfile> {
        self.profiles.get(id)
    }

    /// Lists all registered profiles sorted by ID.
    pub fn list_profiles(&self) -> Vec<&DistroProfile> {
        let mut list: Vec<&DistroProfile> = self.profiles.values().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    /// Evaluates a single registered profile by ID.
    pub fn evaluate_profile(&self, id: &str) -> Result<DistroEvaluation, String> {
        let profile = self
            .get_profile(id)
            .ok_or_else(|| format!("Distro profile '{}' not found", id))?;
        Ok(DistroEvaluation::evaluate(profile))
    }

    /// Evaluates all profiles and returns list sorted by overall score descending.
    pub fn evaluate_all(&self) -> Vec<DistroEvaluation> {
        let mut evals: Vec<DistroEvaluation> = self
            .profiles
            .values()
            .map(DistroEvaluation::evaluate)
            .collect();
        evals.sort_by(|a, b| {
            b.overall_score
                .partial_cmp(&a.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        evals
    }

    /// Returns the recommended production distribution profile.
    pub fn get_recommended_profile(&self) -> Option<&DistroProfile> {
        self.profiles.values().find(|p| p.recommended)
    }

    /// Maximum allowed file size for a serialized distro store (10MB).
    pub const MAX_STORE_BYTES: u64 = 10 * 1024 * 1024;

    /// Saves the registry atomically to disk with defensive tempfile cleanup.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize distro store: {}", e))?;
        let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));
        if let Err(e) = fs::write(&tmp_path, json) {
            let _ = fs::remove_file(&tmp_path);
            return Err(format!("Failed to write tmp distro store: {}", e));
        }
        if let Err(e) = fs::rename(&tmp_path, path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(format!("Failed to rename distro store: {}", e));
        }
        Ok(())
    }

    /// Loads the registry from disk with file size bounds check.
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to inspect distro store file metadata: {}", e))?;
        if metadata.len() > Self::MAX_STORE_BYTES {
            return Err(format!(
                "Distro store file size {} bytes exceeds maximum limit of {} bytes",
                metadata.len(),
                Self::MAX_STORE_BYTES
            ));
        }
        let data = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read distro store: {}", e))?;
        let store: Self = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse distro store: {}", e))?;
        Ok(store)
    }

    /// Loads registry from disk or recovers with default canonical profiles on error.
    pub fn load_or_recover(path: &Path) -> Self {
        match Self::load_from_path(path) {
            Ok(store) => store,
            Err(_) => Self::new(),
        }
    }

    /// Loads the store based on the provided DistroConfig store_path.
    pub fn load_from_config(cfg: &crate::distro_config::DistroConfig) -> Result<Self, String> {
        let p = Path::new(&cfg.store_path);
        if p.exists() {
            Self::load_from_path(p)
        } else {
            Ok(Self::new())
        }
    }

    /// Checks all registered profiles against the given security policy.
    pub fn check_security_policy(
        &self,
        policy: &crate::distro_policy::DistroSecurityPolicy,
    ) -> Vec<crate::distro_policy::DistroPolicyVerdict> {
        policy.check_all(self)
    }

    /// Returns all registered profiles that comply with the security policy.
    pub fn get_policy_compliant_profiles(
        &self,
        policy: &crate::distro_policy::DistroSecurityPolicy,
    ) -> Vec<DistroProfile> {
        policy.filter_compliant_profiles(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distro_store_lifecycle_and_evaluations() {
        let store = DistroStore::new();
        assert_eq!(store.list_profiles().len(), 2);

        let rec = store.get_recommended_profile();
        assert!(rec.is_some());
        assert_eq!(rec.unwrap().id, "debian-12-minimal-x86_64");

        let evals = store.evaluate_all();
        assert_eq!(evals.len(), 2);
        assert!(evals[0].overall_score >= evals[1].overall_score);
    }

    #[test]
    fn test_distro_store_persistence_and_recovery() {
        let store = DistroStore::new();
        let tmp_file = std::env::temp_dir().join("aios_distro_store_test.json");
        assert!(store.save_to_path(&tmp_file).is_ok());

        let loaded = DistroStore::load_from_path(&tmp_file).unwrap();
        assert_eq!(loaded.list_profiles().len(), 2);

        let recovered = DistroStore::load_or_recover(Path::new("/nonexistent/path/distro.json"));
        assert_eq!(recovered.list_profiles().len(), 2);

        let _ = fs::remove_file(tmp_file);
    }
}
