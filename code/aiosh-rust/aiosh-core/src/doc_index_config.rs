//! Documentation Index Control configuration module (T-00454).
//!
//! Contract: `docs/tasks/evidence/T-00452-configuration-specification.md`.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub const MAX_CONFIG_BYTES: u64 = 64 * 1024; // 64 KiB
pub const DEFAULT_CONFIG_PATH: &str = "docs/doc_index_config.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocIndexConfig {
    pub version: String,
    pub root_dirs: Vec<String>,
    pub include_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub enforce_strict_links: bool,
}

impl Default for DocIndexConfig {
    fn default() -> Self {
        DocIndexConfig {
            version: "1.0.0".into(),
            root_dirs: vec!["docs".into()],
            include_extensions: vec![".md".into()],
            exclude_patterns: vec!["**/node_modules/**".into(), "**/target/**".into()],
            enforce_strict_links: true,
        }
    }
}

impl DocIndexConfig {
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: DocIndexConfig = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse DocIndexConfig JSON: {}", e))?;
        config.validate()?;
        Ok(config)
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize DocIndexConfig: {}", e))
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
        if let Ok(env_path) = std::env::var("AIOS_DOC_INDEX_CONFIG") {
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
            return Err("DocIndexConfig 'version' must not be empty".into());
        }
        if self.root_dirs.is_empty() {
            return Err("DocIndexConfig 'root_dirs' must not be empty".into());
        }
        if self.root_dirs.len() > 50 {
            return Err("DocIndexConfig 'root_dirs' exceeds maximum of 50 directories".into());
        }
        for dir in &self.root_dirs {
            if dir.trim().is_empty() {
                return Err("DocIndexConfig root directory path must not be empty".into());
            }
            if dir.contains("..") {
                return Err(format!("DocIndexConfig root directory '{}' must not contain '..'", dir));
            }
        }
        if self.include_extensions.is_empty() {
            return Err("DocIndexConfig 'include_extensions' must not be empty".into());
        }
        for ext in &self.include_extensions {
            if !ext.starts_with('.') {
                return Err(format!("DocIndexConfig extension '{}' must start with '.'", ext));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_index_config_default_is_valid() {
        let config = DocIndexConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.root_dirs, vec!["docs"]);
    }

    #[test]
    fn test_doc_index_config_roundtrip_happy() {
        let original = DocIndexConfig::default();
        let json_str = original.to_json().unwrap();
        let decoded = DocIndexConfig::from_json(&json_str).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_doc_index_config_from_path_and_missing() {
        let temp_dir = std::env::temp_dir().join("aios_test_config");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let cfg_path = temp_dir.join("test_cfg.json");
        let default_cfg = DocIndexConfig::default();
        std::fs::write(&cfg_path, default_cfg.to_json().unwrap()).unwrap();

        let loaded = DocIndexConfig::from_path(&cfg_path).unwrap();
        assert_eq!(loaded, default_cfg);

        let missing = DocIndexConfig::from_path(&temp_dir.join("nonexistent.json"));
        assert!(missing.is_err());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_doc_index_config_validation_failures() {
        // 1. Empty version
        let mut cfg = DocIndexConfig::default();
        cfg.version = "".into();
        assert!(cfg.validate().unwrap_err().contains("'version'"));

        // 2. Empty root_dirs
        let mut cfg = DocIndexConfig::default();
        cfg.root_dirs = vec![];
        assert!(cfg.validate().unwrap_err().contains("'root_dirs'"));

        // 3. Path traversal in root_dirs
        let mut cfg = DocIndexConfig::default();
        cfg.root_dirs = vec!["../etc".into()];
        assert!(cfg.validate().unwrap_err().contains("must not contain '..'"));

        // 4. Empty include_extensions
        let mut cfg = DocIndexConfig::default();
        cfg.include_extensions = vec![];
        assert!(cfg.validate().unwrap_err().contains("'include_extensions'"));

        // 5. Extension missing dot
        let mut cfg = DocIndexConfig::default();
        cfg.include_extensions = vec!["md".into()];
        assert!(cfg.validate().unwrap_err().contains("must start with '.'"));

        // 6. Malformed JSON
        assert!(DocIndexConfig::from_json("{invalid json").is_err());
    }

    #[test]
    fn test_doc_index_config_from_env_fallback() {
        std::env::remove_var("AIOS_DOC_INDEX_CONFIG");
        let cfg = DocIndexConfig::from_env().unwrap();
        assert_eq!(cfg.version, "1.0.0");
    }
}
