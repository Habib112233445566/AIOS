use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::triage::TriageSeverity;

pub const MAX_CONFIG_FILE_BYTES: u64 = 64 * 1024; // 64 KiB
pub const MIN_STORE_BYTES: usize = 16 * 1024; // 16 KiB
pub const MAX_STORE_BYTES: usize = 64 * 1024 * 1024; // 64 MiB

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriageConfig {
    pub max_store_bytes: usize,
    pub default_severity: TriageSeverity,
    pub auto_ingest_suites: Vec<String>,
    pub retention_days: u32,
    pub notify_blockers: bool,
    pub store_path: Option<String>,
}

impl Default for TriageConfig {
    fn default() -> Self {
        Self {
            max_store_bytes: 1024 * 1024,
            default_severity: TriageSeverity::Critical,
            auto_ingest_suites: vec!["*".to_string()],
            retention_days: 90,
            notify_blockers: true,
            store_path: None,
        }
    }
}

impl TriageConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_store_bytes < MIN_STORE_BYTES || self.max_store_bytes > MAX_STORE_BYTES {
            return Err(format!(
                "max_store_bytes must be between {} and {} bytes, got {}",
                MIN_STORE_BYTES, MAX_STORE_BYTES, self.max_store_bytes
            ));
        }
        if self.retention_days == 0 {
            return Err("retention_days must be at least 1".to_string());
        }
        if self.auto_ingest_suites.is_empty() {
            return Err("auto_ingest_suites cannot be empty".to_string());
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
        let cfg: Self = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file '{}': {}", path.display(), e))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn from_env_or_default() -> Self {
        if let Ok(path_str) = std::env::var("AIOS_TRIAGE_CONFIG") {
            let path = Path::new(&path_str);
            if let Ok(cfg) = Self::from_file(path) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn should_ingest_suite(&self, suite: &str) -> bool {
        for pattern in &self.auto_ingest_suites {
            if pattern == "*" || pattern == suite {
                return true;
            }
            if let Some(prefix) = pattern.strip_suffix('*') {
                if suite.starts_with(prefix) {
                    return true;
                }
            }
        }
        false
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize triage config: {}", e))?;
        std::fs::write(path, json_str)
            .map_err(|e| format!("Failed to write config file '{}': {}", path.display(), e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triage_config_defaults_and_validation() {
        let cfg = TriageConfig::default();
        assert!(cfg.validate().is_ok());

        let mut invalid_cfg = cfg.clone();
        invalid_cfg.max_store_bytes = 100;
        assert!(invalid_cfg.validate().is_err());

        let mut zero_ret = cfg.clone();
        zero_ret.retention_days = 0;
        assert!(zero_ret.validate().is_err());

        let mut empty_suites = cfg.clone();
        empty_suites.auto_ingest_suites.clear();
        assert!(empty_suites.validate().is_err());
    }

    #[test]
    fn test_triage_config_save_load() {
        let tmp = std::env::temp_dir().join(format!("test_triage_cfg_{}.json", std::process::id()));
        let mut cfg = TriageConfig::default();
        cfg.retention_days = 45;
        cfg.save_to_file(&tmp).unwrap();

        let loaded = TriageConfig::from_file(&tmp).unwrap();
        assert_eq!(cfg, loaded);
        assert_eq!(loaded.retention_days, 45);

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_triage_config_should_ingest_suite() {
        let mut cfg = TriageConfig::default();
        assert!(cfg.should_ingest_suite("any_suite"));

        cfg.auto_ingest_suites = vec!["suite_a".to_string(), "sec_*".to_string()];
        assert!(cfg.should_ingest_suite("suite_a"));
        assert!(cfg.should_ingest_suite("sec_auth"));
        assert!(cfg.should_ingest_suite("sec_policy"));
        assert!(!cfg.should_ingest_suite("suite_b"));
        assert!(!cfg.should_ingest_suite("other_suite"));
    }

    #[test]
    fn test_triage_config_boundaries() {
        let mut cfg = TriageConfig::default();
        cfg.max_store_bytes = MIN_STORE_BYTES;
        assert!(cfg.validate().is_ok());

        cfg.max_store_bytes = MIN_STORE_BYTES - 1;
        assert!(cfg.validate().is_err());

        cfg.max_store_bytes = MAX_STORE_BYTES;
        assert!(cfg.validate().is_ok());

        cfg.max_store_bytes = MAX_STORE_BYTES + 1;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_triage_config_file_errors() {
        let non_existent = Path::new("non_existent_config_12345.json");
        assert!(TriageConfig::from_file(non_existent).is_err());

        let tmp = std::env::temp_dir().join(format!("test_triage_invalid_{}.json", std::process::id()));
        std::fs::write(&tmp, "{ invalid_json: true }").unwrap();
        assert!(TriageConfig::from_file(&tmp).is_err());
        let _ = std::fs::remove_file(&tmp);
    }
}
