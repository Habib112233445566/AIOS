//! Evidence & Audit Trail configuration module (T-00553 scaffold / T-00554 impl).
//!
//! Contract: `docs/tasks/evidence/T-00552-configuration-specification.md`.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub const MAX_CONFIG_BYTES: u64 = 64 * 1024; // 64 KiB
pub const MAX_FILE_SIZE_LIMIT: u64 = 64 * 1024 * 1024; // 64 MiB
pub const DEFAULT_CONFIG_PATH: &str = "config/evidence.config.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceConfig {
    pub evidence_dir: String,
    pub max_file_bytes: u64,
    pub allowed_extensions: Vec<String>,
    pub enforce_checksum: bool,
    pub require_all_steps: bool,
}

impl Default for EvidenceConfig {
    fn default() -> Self {
        EvidenceConfig {
            evidence_dir: "docs/tasks/evidence".into(),
            max_file_bytes: 16 * 1024 * 1024, // 16 MiB
            allowed_extensions: vec![".md".into(), ".json".into()],
            enforce_checksum: true,
            require_all_steps: false,
        }
    }
}

impl EvidenceConfig {
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: EvidenceConfig = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse EvidenceConfig JSON: {}", e))?;
        config.validate()?;
        Ok(config)
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize EvidenceConfig: {}", e))
    }

    pub fn from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err(format!("Config file not found at {}", path.display()));
        }
        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata for {}: {}", path.display(), e))?;
        if metadata.len() > MAX_CONFIG_BYTES {
            return Err(format!(
                "Config file {} exceeds maximum size ({} bytes > {} max)",
                path.display(),
                metadata.len(),
                MAX_CONFIG_BYTES
            ));
        }
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open config file {}: {}", path.display(), e))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("Failed to read config file {}: {}", path.display(), e))?;
        Self::from_json(&content)
    }

    pub fn from_env() -> Result<Self, String> {
        if let Ok(config_path) = std::env::var("AIOS_EVIDENCE_CONFIG_PATH") {
            let path = Path::new(&config_path);
            if path.exists() {
                return Self::from_path(path);
            }
        }

        let default_path = Path::new(DEFAULT_CONFIG_PATH);
        if default_path.exists() {
            if let Ok(cfg) = Self::from_path(default_path) {
                return Ok(cfg);
            }
        }

        let mut config = Self::default();

        if let Ok(dir) = std::env::var("AIOS_EVIDENCE_DIR") {
            config.evidence_dir = dir;
        }

        if let Ok(max_bytes_str) = std::env::var("AIOS_EVIDENCE_MAX_FILE_BYTES") {
            if let Ok(parsed) = max_bytes_str.parse::<u64>() {
                config.max_file_bytes = parsed;
            }
        }

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.evidence_dir.trim().is_empty() {
            return Err("evidence_dir cannot be empty".into());
        }
        if self.evidence_dir.starts_with('/') || self.evidence_dir.contains(':') {
            return Err(format!("evidence_dir must be a relative path: {}", self.evidence_dir));
        }
        if self.evidence_dir.contains("..") {
            return Err(format!("evidence_dir cannot contain path traversal ('..'): {}", self.evidence_dir));
        }
        if self.max_file_bytes == 0 || self.max_file_bytes > MAX_FILE_SIZE_LIMIT {
            return Err(format!(
                "max_file_bytes must be between 1 and {} bytes (got {})",
                MAX_FILE_SIZE_LIMIT,
                self.max_file_bytes
            ));
        }
        if self.allowed_extensions.is_empty() {
            return Err("allowed_extensions cannot be empty".into());
        }
        for ext in &self.allowed_extensions {
            if !ext.starts_with('.') {
                return Err(format!("allowed_extension must start with '.': {}", ext));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_config_default_is_valid() {
        let config = EvidenceConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.evidence_dir, "docs/tasks/evidence");
        assert_eq!(config.max_file_bytes, 16 * 1024 * 1024);
        assert!(config.enforce_checksum);
        assert!(!config.require_all_steps);
    }

    #[test]
    fn test_evidence_config_roundtrip_happy() {
        let original = EvidenceConfig::default();
        let json_str = original.to_json().unwrap();
        let recovered = EvidenceConfig::from_json(&json_str).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_evidence_config_validation_failures() {
        let mut cfg = EvidenceConfig::default();
        cfg.evidence_dir = "".into();
        assert!(cfg.validate().is_err());

        cfg.evidence_dir = "/etc/passwd".into();
        assert!(cfg.validate().is_err());

        cfg.evidence_dir = "docs/../outside".into();
        assert!(cfg.validate().is_err());

        cfg.evidence_dir = "docs/tasks/evidence".into();
        cfg.max_file_bytes = 0;
        assert!(cfg.validate().is_err());

        cfg.max_file_bytes = 100 * 1024 * 1024; // > 64 MiB
        assert!(cfg.validate().is_err());

        cfg.max_file_bytes = 16 * 1024 * 1024;
        cfg.allowed_extensions = vec![];
        assert!(cfg.validate().is_err());

        cfg.allowed_extensions = vec!["md".into()]; // missing dot
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_evidence_config_from_path_and_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("evidence.config.json");

        let cfg = EvidenceConfig::default();
        std::fs::write(&file_path, cfg.to_json().unwrap()).unwrap();

        let loaded = EvidenceConfig::from_path(&file_path).unwrap();
        assert_eq!(cfg, loaded);

        let missing = temp_dir.path().join("missing.json");
        assert!(EvidenceConfig::from_path(&missing).is_err());
    }

    #[test]
    fn test_evidence_config_from_env_fallback() {
        let loaded = EvidenceConfig::from_env().unwrap();
        assert!(loaded.validate().is_ok());
    }

    #[test]
    fn test_evidence_config_oversized_file_error() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("oversized.json");
        let large_content = vec![b' '; (MAX_CONFIG_BYTES + 100) as usize];
        std::fs::write(&file_path, large_content).unwrap();

        let err = EvidenceConfig::from_path(&file_path).unwrap_err();
        assert!(err.contains("exceeds maximum size"));
    }

    #[test]
    fn test_evidence_config_malformed_json_error() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("malformed.json");
        std::fs::write(&file_path, "{ invalid json ").unwrap();

        let err = EvidenceConfig::from_path(&file_path).unwrap_err();
        assert!(err.contains("Failed to parse EvidenceConfig JSON"));
    }

    #[test]
    fn test_real_repo_evidence_config_file() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
        let config_path = repo_root.join("config/evidence.config.json");
        assert!(config_path.exists(), "config/evidence.config.json must exist in repo");

        let cfg = EvidenceConfig::from_path(&config_path).unwrap();
        assert_eq!(cfg.evidence_dir, "docs/tasks/evidence");
        assert_eq!(cfg.max_file_bytes, 16 * 1024 * 1024);
        assert!(cfg.enforce_checksum);
    }
}
