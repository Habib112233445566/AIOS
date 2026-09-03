//! Configuration resolution for Linux Distribution Selection & Justification (T-01044).
//!
//! Contract: `docs/tasks/evidence/T-01042-distro-configuration-specification.md`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Read;
use std::path::Path;

pub const DEFAULT_DISTRO_STORE_PATH: &str = "config/distros.json";
pub const DEFAULT_DISTRO_CONFIG_PATH: &str = "config/distro.json";
pub const DEFAULT_PINNED_REFERENCE_ID: &str = "debian-12-minimal-x86_64";
pub const DEFAULT_MIN_RECOMMENDATION_SCORE: f32 = 0.75;
pub const MAX_DISTRO_CONFIG_BYTES: u64 = 65_536;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistroEvaluationWeights {
    pub binary_compatibility: f32,
    pub security: f32,
    pub footprint: f32,
}

impl Default for DistroEvaluationWeights {
    fn default() -> Self {
        Self {
            binary_compatibility: 0.40,
            security: 0.30,
            footprint: 0.30,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistroConfig {
    pub store_path: String,
    pub pinned_reference_id: String,
    pub min_recommendation_score: f32,
    pub weights: DistroEvaluationWeights,
    pub auto_evaluate: bool,
}

impl Default for DistroConfig {
    fn default() -> Self {
        Self {
            store_path: DEFAULT_DISTRO_STORE_PATH.into(),
            pinned_reference_id: DEFAULT_PINNED_REFERENCE_ID.into(),
            min_recommendation_score: DEFAULT_MIN_RECOMMENDATION_SCORE,
            weights: DistroEvaluationWeights::default(),
            auto_evaluate: true,
        }
    }
}

impl DistroConfig {
    /// Loads distro configuration from environment variables or default path.
    pub fn from_env() -> Result<Self, String> {
        Self::from_source(&|name| std::env::var(name).ok())
    }

    /// Loads distro configuration from an explicit file path.
    pub fn from_path(path: &str) -> Result<Self, String> {
        Self::from_source_with_path(path, &|_name| None)
    }

    /// Loads distro configuration using a custom environment/path resolver function.
    pub fn from_source(
        get: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Self, String> {
        let path = get("AIOSH_DISTRO_CONFIG").unwrap_or_else(|| DEFAULT_DISTRO_CONFIG_PATH.into());
        Self::from_source_with_path(&path, get)
    }

    /// Loads distro configuration from a specific path with environment variable overrides.
    pub fn from_source_with_path(
        path: &str,
        get: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Self, String> {
        let p = Path::new(path);
        let mut cfg = if p.exists() {
            let f = std::fs::File::open(p)
                .map_err(|e| format!("Failed to open distro config at {path}: {e}"))?;
            let mut content = String::new();
            f.take(MAX_DISTRO_CONFIG_BYTES)
                .read_to_string(&mut content)
                .map_err(|e| format!("Failed to read distro config at {path}: {e}"))?;
            let loaded: DistroConfig = serde_json::from_str(&content)
                .map_err(|e| format!("Malformed distro config at {path}: {e}"))?;
            loaded
        } else {
            Self::default()
        };

        // Environment overrides
        if let Some(store) = get("AIOSH_DISTRO_STORE_PATH") {
            cfg.store_path = store;
        }
        if let Some(distro) = get("AIOSH_DEFAULT_DISTRO") {
            cfg.pinned_reference_id = distro;
        }

        cfg.validate()?;
        Ok(cfg)
    }

    /// Validate the configuration against specification rules V1..V5 with security hardening.
    pub fn validate(&self) -> Result<(), String> {
        if self.store_path.trim().is_empty() {
            return Err("invalid distro config: store_path cannot be empty".into());
        }
        if self.store_path.contains("..") {
            return Err("invalid distro config: store_path cannot contain path traversal ('..')".into());
        }
        if self.pinned_reference_id.trim().is_empty() {
            return Err("invalid distro config: pinned_reference_id cannot be empty".into());
        }
        if self.min_recommendation_score.is_nan() || !(0.0..=1.0).contains(&self.min_recommendation_score) {
            return Err(format!(
                "invalid distro config: min_recommendation_score must be a valid number between 0.0 and 1.0 (got {})",
                self.min_recommendation_score
            ));
        }
        if self.weights.binary_compatibility.is_nan()
            || self.weights.security.is_nan()
            || self.weights.footprint.is_nan()
        {
            return Err("invalid distro config: weights cannot be NaN".into());
        }
        if self.weights.binary_compatibility < 0.0
            || self.weights.security < 0.0
            || self.weights.footprint < 0.0
        {
            return Err("invalid distro config: weights cannot be negative".into());
        }
        let total_weight = self.weights.binary_compatibility
            + self.weights.security
            + self.weights.footprint;
        if total_weight <= 0.0 || total_weight.is_nan() {
            return Err("invalid distro config: total weights must be positive".into());
        }
        Ok(())
    }

    /// Exports configuration to JSON with property source provenance.
    pub fn to_json_with_sources(&self) -> Value {
        self.to_json_with_sources_from(&|name| std::env::var(name).is_ok())
    }

    /// Exports configuration to JSON using a custom `is_set` checker.
    pub fn to_json_with_sources_from(&self, is_set: &dyn Fn(&str) -> bool) -> Value {
        let file_exists = Path::new(DEFAULT_DISTRO_CONFIG_PATH).exists();
        let base_source = if file_exists { "file" } else { "default" };

        let store_source = if is_set("AIOSH_DISTRO_STORE_PATH") {
            "env"
        } else {
            base_source
        };
        let distro_source = if is_set("AIOSH_DEFAULT_DISTRO") {
            "env"
        } else {
            base_source
        };

        json!({
            "store_path": {"value": self.store_path, "source": store_source},
            "pinned_reference_id": {"value": self.pinned_reference_id, "source": distro_source},
            "min_recommendation_score": {"value": self.min_recommendation_score, "source": base_source},
            "weights": {"value": self.weights, "source": base_source},
            "auto_evaluate": {"value": self.auto_evaluate, "source": base_source},
        })
    }

    /// Serializes and saves the configuration to the specified file.
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        self.validate()?;
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize distro config: {e}"))?;
        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory {}: {e}", parent.display()))?;
            }
        }
        std::fs::write(path, json_str)
            .map_err(|e| format!("Failed to write distro config to {path}: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distro_config_default_and_roundtrip() {
        let cfg = DistroConfig::default();
        assert_eq!(cfg.store_path, "config/distros.json");
        assert_eq!(cfg.pinned_reference_id, "debian-12-minimal-x86_64");
        assert_eq!(cfg.min_recommendation_score, 0.75);
        assert!(cfg.auto_evaluate);
        assert!(cfg.validate().is_ok());

        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: DistroConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, parsed);
    }

    #[test]
    fn test_distro_config_validation_errors() {
        let mut cfg = DistroConfig::default();
        cfg.store_path = "   ".into();
        assert!(cfg.validate().is_err());

        let mut cfg = DistroConfig::default();
        cfg.pinned_reference_id = "".into();
        assert!(cfg.validate().is_err());

        let mut cfg = DistroConfig::default();
        cfg.min_recommendation_score = 1.5;
        assert!(cfg.validate().is_err());

        let mut cfg = DistroConfig::default();
        cfg.min_recommendation_score = -0.1;
        assert!(cfg.validate().is_err());

        let mut cfg = DistroConfig::default();
        cfg.weights.binary_compatibility = -0.1;
        assert!(cfg.validate().is_err());

        let mut cfg = DistroConfig::default();
        cfg.weights.binary_compatibility = 0.0;
        cfg.weights.security = 0.0;
        cfg.weights.footprint = 0.0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_distro_config_from_source_overrides() {
        let env_map = |key: &str| match key {
            "AIOSH_DISTRO_STORE_PATH" => Some("/tmp/custom_distros.json".into()),
            "AIOSH_DEFAULT_DISTRO" => Some("alpine-custom".into()),
            _ => None,
        };

        let cfg = DistroConfig::from_source(&env_map).unwrap();
        assert_eq!(cfg.store_path, "/tmp/custom_distros.json");
        assert_eq!(cfg.pinned_reference_id, "alpine-custom");

        let sources = cfg.to_json_with_sources_from(&|name| name == "AIOSH_DISTRO_STORE_PATH" || name == "AIOSH_DEFAULT_DISTRO");
        assert_eq!(sources["store_path"]["source"], "env");
        assert_eq!(sources["pinned_reference_id"]["source"], "env");
    }

    #[test]
    fn test_distro_config_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/distro.json");
        let path_str = path.to_str().unwrap();

        let mut cfg = DistroConfig::default();
        cfg.min_recommendation_score = 0.82;
        cfg.save_to_file(path_str).unwrap();

        let loaded = DistroConfig::from_path(path_str).unwrap();
        assert_eq!(loaded.min_recommendation_score, 0.82);
    }

    #[test]
    fn test_distro_config_malformed_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "{ invalid json").unwrap();

        let res = DistroConfig::from_path(path.to_str().unwrap());
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Malformed distro config"));
    }

    #[test]
    fn test_distro_config_hardening_nan_and_traversal() {
        let mut cfg = DistroConfig::default();
        cfg.min_recommendation_score = f32::NAN;
        assert!(cfg.validate().is_err());

        let mut cfg = DistroConfig::default();
        cfg.weights.binary_compatibility = f32::NAN;
        assert!(cfg.validate().is_err());

        let mut cfg = DistroConfig::default();
        cfg.store_path = "../etc/passwd".into();
        assert!(cfg.validate().is_err());
    }
}
