//! Configuration management for the Agent Handoff Protocol.

use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::handoff::HandoffPriority;

pub const MAX_CONFIG_FILE_BYTES: u64 = 64 * 1024; // 64 KiB
pub const MIN_STORE_BYTES: usize = 16 * 1024; // 16 KiB
pub const MAX_STORE_BYTES: usize = 64 * 1024 * 1024; // 64 MiB

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandoffConfig {
    pub max_store_bytes: usize,
    pub default_priority: HandoffPriority,
    pub default_ttl_seconds: u64,
    pub allow_auto_accept: bool,
    pub store_path: Option<String>,
}

impl Default for HandoffConfig {
    fn default() -> Self {
        Self {
            max_store_bytes: 1024 * 1024,
            default_priority: HandoffPriority::Normal,
            default_ttl_seconds: 86400,
            allow_auto_accept: false,
            store_path: None,
        }
    }
}

impl HandoffConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_store_bytes < MIN_STORE_BYTES || self.max_store_bytes > MAX_STORE_BYTES {
            return Err(format!(
                "max_store_bytes must be between {} and {} bytes, got {}",
                MIN_STORE_BYTES, MAX_STORE_BYTES, self.max_store_bytes
            ));
        }
        if self.default_ttl_seconds == 0 {
            return Err("default_ttl_seconds must be at least 1".to_string());
        }
        Ok(())
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err(format!("Config file '{}' does not exist", path.display()));
        }
        let meta = std::fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata for '{}': {}", path.display(), e))?;
        if meta.len() > MAX_CONFIG_FILE_BYTES {
            return Err(format!(
                "Config file '{}' is too large ({} bytes > {} max)",
                path.display(),
                meta.len(),
                MAX_CONFIG_FILE_BYTES
            ));
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file '{}': {}", path.display(), e))?;
        let config: HandoffConfig = serde_json::from_str(&content)
            .map_err(|e| format!("JSON parse error in '{}': {}", path.display(), e))?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_env_or_default() -> Self {
        if let Ok(env_path) = std::env::var("AIOSH_HANDOFF_CONFIG") {
            if let Ok(cfg) = Self::from_file(Path::new(&env_path)) {
                return cfg;
            }
        }
        let default_file = Path::new("docs/handoff_config.json");
        if default_file.exists() {
            if let Ok(cfg) = Self::from_file(default_file) {
                return cfg;
            }
        }
        let mut cfg = Self::default();
        if let Ok(store_path) = std::env::var("AIOSH_HANDOFF_STORE") {
            cfg.store_path = Some(store_path);
        }
        cfg
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
            }
        }
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write config file '{}': {}", path.display(), e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handoff_config_defaults_and_validation() {
        let mut cfg = HandoffConfig::default();
        assert!(cfg.validate().is_ok());

        cfg.max_store_bytes = 100;
        assert!(cfg.validate().is_err());

        cfg.max_store_bytes = 1024 * 1024;
        cfg.default_ttl_seconds = 0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_handoff_config_roundtrip() {
        let tmp = std::env::temp_dir().join(format!("test_handoff_cfg_{}.json", std::process::id()));
        let cfg = HandoffConfig::default();
        assert!(cfg.save_to_file(&tmp).is_ok());

        let loaded = HandoffConfig::from_file(&tmp).expect("load config");
        assert_eq!(cfg, loaded);

        let _ = std::fs::remove_file(&tmp);
    }
}
