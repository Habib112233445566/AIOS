//! Repository Health configuration module (T-00654).
//!
//! Contract: `docs/tasks/evidence/T-00652-configuration-specification.md`.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub const MAX_CONFIG_BYTES: u64 = 64 * 1024; // 64 KiB
pub const DEFAULT_CONFIG_PATH: &str = "docs/repo_health_config.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoHealthConfig {
    pub version: String,
    pub max_file_bytes: u64,
    pub ignored_dirs: Vec<String>,
    pub require_clean_git: bool,
    pub security_policy_path: String,
    pub min_security_policy_bytes: u64,
}

impl Default for RepoHealthConfig {
    fn default() -> Self {
        RepoHealthConfig {
            version: "1.0.0".into(),
            max_file_bytes: 16 * 1024 * 1024, // 16 MiB
            ignored_dirs: vec![
                ".git".into(),
                "target".into(),
                "node_modules".into(),
                ".venv".into(),
            ],
            require_clean_git: false,
            security_policy_path: "SECURITY.md".into(),
            min_security_policy_bytes: 100,
        }
    }
}

impl RepoHealthConfig {
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: RepoHealthConfig = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse RepoHealthConfig JSON: {}", e))?;
        config.validate()?;
        Ok(config)
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize RepoHealthConfig: {}", e))
    }

    pub fn from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err(format!("Config file not found at {}", path.display()));
        }
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open config file {}: {}", path.display(), e))?;
        let mut content = String::new();
        file.by_ref()
            .take(MAX_CONFIG_BYTES)
            .read_to_string(&mut content)
            .map_err(|e| format!("Failed to read config file {}: {}", path.display(), e))?;
        Self::from_json(&content)
    }

    pub fn from_env() -> Result<Self, String> {
        if let Ok(env_path) = std::env::var("AIOS_REPO_HEALTH_CONFIG") {
            if !env_path.trim().is_empty() {
                return Self::from_path(Path::new(&env_path));
            }
        }
        let default_p = Path::new(DEFAULT_CONFIG_PATH);
        if default_p.exists() {
            return Self::from_path(default_p);
        }
        Ok(Self::default())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() {
            return Err("RepoHealthConfig 'version' must not be empty".into());
        }
        if self.version.len() > 32 {
            return Err("RepoHealthConfig 'version' exceeds maximum length of 32 characters".into());
        }
        if self.max_file_bytes < 1024 || self.max_file_bytes > 1024 * 1024 * 1024 {
            return Err(format!(
                "RepoHealthConfig 'max_file_bytes' ({}) must be between 1024 and 1073741824 bytes",
                self.max_file_bytes
            ));
        }
        if self.ignored_dirs.is_empty() {
            return Err("RepoHealthConfig 'ignored_dirs' must not be empty".into());
        }
        if self.ignored_dirs.len() > 50 {
            return Err("RepoHealthConfig 'ignored_dirs' exceeds maximum of 50 entries".into());
        }
        for dir in &self.ignored_dirs {
            if dir.trim().is_empty() {
                return Err("RepoHealthConfig ignored directory name must not be empty".into());
            }
            if dir.contains("..") || dir.contains('/') || dir.contains('\\') {
                return Err(format!(
                    "RepoHealthConfig ignored directory '{}' must be a simple directory name without '..' or path separators",
                    dir
                ));
            }
        }
        if self.security_policy_path.trim().is_empty() {
            return Err("RepoHealthConfig 'security_policy_path' must not be empty".into());
        }
        if self.security_policy_path.len() > 255 {
            return Err("RepoHealthConfig 'security_policy_path' exceeds maximum of 255 characters".into());
        }
        if self.security_policy_path.contains("..") {
            return Err("RepoHealthConfig 'security_policy_path' must not contain '..'".into());
        }
        if self.min_security_policy_bytes < 1 || self.min_security_policy_bytes > 65536 {
            return Err(format!(
                "RepoHealthConfig 'min_security_policy_bytes' ({}) must be between 1 and 65536 bytes",
                self.min_security_policy_bytes
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_repo_health_config_default_and_roundtrip() {
        let config = RepoHealthConfig::default();
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.max_file_bytes, 16 * 1024 * 1024);
        assert!(config.ignored_dirs.contains(&".git".to_string()));

        let json = config.to_json().expect("serialize");
        let parsed = RepoHealthConfig::from_json(&json).expect("deserialize");
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_repo_health_config_validation_errors() {
        let mut c = RepoHealthConfig::default();
        c.version = "".into();
        assert!(c.validate().unwrap_err().contains("'version' must not be empty"));

        let mut c = RepoHealthConfig::default();
        c.max_file_bytes = 500;
        assert!(c.validate().unwrap_err().contains("'max_file_bytes'"));

        let mut c = RepoHealthConfig::default();
        c.ignored_dirs = vec![];
        assert!(c.validate().unwrap_err().contains("'ignored_dirs' must not be empty"));

        let mut c = RepoHealthConfig::default();
        c.ignored_dirs = vec!["../escaped".into()];
        assert!(c.validate().unwrap_err().contains("without '..' or path separators"));

        let mut c = RepoHealthConfig::default();
        c.security_policy_path = "../SECRET.md".into();
        assert!(c.validate().unwrap_err().contains("must not contain '..'"));

        let mut c = RepoHealthConfig::default();
        c.min_security_policy_bytes = 0;
        assert!(c.validate().unwrap_err().contains("'min_security_policy_bytes'"));
    }

    #[test]
    fn test_repo_health_config_from_path() {
        let temp_dir = std::env::temp_dir().join(format!("repo_cfg_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config_file = temp_dir.join("repo_health_config.json");

        let mut f = File::create(&config_file).unwrap();
        f.write_all(br#"{
            "version": "1.2.0",
            "max_file_bytes": 8388608,
            "ignored_dirs": [".git", "build"],
            "require_clean_git": true,
            "security_policy_path": "SECURITY.md",
            "min_security_policy_bytes": 120
        }"#).unwrap();
        drop(f);

        let loaded = RepoHealthConfig::from_path(&config_file).expect("load from path");
        assert_eq!(loaded.version, "1.2.0");
        assert_eq!(loaded.max_file_bytes, 8388608);
        assert_eq!(loaded.require_clean_git, true);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
