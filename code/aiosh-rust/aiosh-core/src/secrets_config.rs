//! Secrets & Access Hygiene configuration module (T-00753).
//!
//! Contract: `docs/tasks/evidence/T-00752-configuration-specification.md`.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub const MAX_CONFIG_BYTES: u64 = 64 * 1024; // 64 KiB
pub const DEFAULT_CONFIG_PATH: &str = "docs/secrets_config.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretsConfig {
    pub version: String,
    pub max_file_bytes: u64,
    pub max_line_bytes: usize,
    pub ignored_dirs: Vec<String>,
    pub allow_patterns: Vec<String>,
    pub require_clean: bool,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        SecretsConfig {
            version: "1.0.0".into(),
            max_file_bytes: 16 * 1024 * 1024, // 16 MiB
            max_line_bytes: 4096,
            ignored_dirs: vec![
                ".git".into(),
                "target".into(),
                "node_modules".into(),
                ".venv".into(),
                "dist".into(),
            ],
            allow_patterns: Vec::new(),
            require_clean: false,
        }
    }
}

impl SecretsConfig {
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: SecretsConfig = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse SecretsConfig JSON: {}", e))?;
        config.validate()?;
        Ok(config)
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize SecretsConfig: {}", e))
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
        if let Ok(env_path) = std::env::var("AIOS_SECRETS_CONFIG") {
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
            return Err("SecretsConfig 'version' must not be empty".into());
        }
        if self.version.len() > 32 {
            return Err("SecretsConfig 'version' exceeds maximum length of 32 characters".into());
        }
        if self.max_file_bytes < 1024 || self.max_file_bytes > 1024 * 1024 * 1024 {
            return Err(format!(
                "SecretsConfig 'max_file_bytes' ({}) must be between 1024 and 1073741824 bytes",
                self.max_file_bytes
            ));
        }
        if self.max_line_bytes < 128 || self.max_line_bytes > 65536 {
            return Err(format!(
                "SecretsConfig 'max_line_bytes' ({}) must be between 128 and 65536 bytes",
                self.max_line_bytes
            ));
        }
        if self.ignored_dirs.is_empty() {
            return Err("SecretsConfig 'ignored_dirs' must not be empty".into());
        }
        if self.ignored_dirs.len() > 50 {
            return Err("SecretsConfig 'ignored_dirs' exceeds maximum of 50 entries".into());
        }
        for (idx, d) in self.ignored_dirs.iter().enumerate() {
            if d.trim().is_empty() {
                return Err(format!("SecretsConfig 'ignored_dirs[{}]' must not be empty", idx));
            }
        }
        if self.allow_patterns.len() > 100 {
            return Err("SecretsConfig 'allow_patterns' exceeds maximum of 100 entries".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_valid() {
        let config = SecretsConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.max_file_bytes, 16 * 1024 * 1024);
        assert_eq!(config.max_line_bytes, 4096);
        assert_eq!(config.ignored_dirs.len(), 5);
        assert!(!config.require_clean);
    }

    #[test]
    fn test_json_roundtrip() {
        let config = SecretsConfig::default();
        let json_str = config.to_json().expect("serialize to json");
        let parsed = SecretsConfig::from_json(&json_str).expect("deserialize from json");
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_invalid_bounds() {
        let mut config = SecretsConfig::default();
        config.max_file_bytes = 100; // below 1024
        assert!(config.validate().is_err());

        let mut config2 = SecretsConfig::default();
        config2.max_line_bytes = 50; // below 128
        assert!(config2.validate().is_err());

        let mut config3 = SecretsConfig::default();
        config3.ignored_dirs.clear();
        assert!(config3.validate().is_err());
    }
}
