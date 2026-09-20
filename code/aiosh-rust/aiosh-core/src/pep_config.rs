//! PEP Decision Engine Configuration Module (T-02141..T-02146).
//!
//! Contract: `docs/tasks/evidence/T-02142-configuration-specification.md`.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::pep_decision::PepCombiningAlgorithm;

/// Maximum configuration file size in bytes (64 KiB).
pub const MAX_CONFIG_BYTES: u64 = 64 * 1024;

/// Default path for the PEP policy store JSON file.
pub const DEFAULT_PEP_STORE_PATH: &str = ".aios/pep_policies.json";

/// Minimum permissible store size in bytes (1 KiB).
pub const MIN_STORE_BYTES: u64 = 1024;

/// Maximum permissible store size in bytes (100 MiB).
pub const MAX_STORE_BYTES: u64 = 104_857_600;

/// Default maximum store size in bytes (10 MiB).
pub const DEFAULT_MAX_STORE_BYTES: u64 = 10_485_760;

/// Minimum rule capacity.
pub const MIN_RULES_COUNT: usize = 1;

/// Maximum rule capacity in registry.
pub const MAX_RULES_COUNT: usize = 50_000;

/// Default rule capacity in registry.
pub const DEFAULT_MAX_RULES: usize = 5_000;

pub const PEPCONF_ERR_IO: &str = "PEPCONF_ERR_IO";
pub const PEPCONF_ERR_PARSE: &str = "PEPCONF_ERR_PARSE";
pub const PEPCONF_ERR_VALIDATION: &str = "PEPCONF_ERR_VALIDATION";
pub const PEPCONF_ERR_BOUNDS: &str = "PEPCONF_ERR_BOUNDS";

/// Configuration for the PEP Decision Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PepConfig {
    pub version: String,
    pub store_path: PathBuf,
    pub max_store_bytes: u64,
    pub max_rules: usize,
    pub default_algorithm: PepCombiningAlgorithm,
    pub audit_all_evaluations: bool,
    pub auto_quarantine_corrupt: bool,
}

impl Default for PepConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            store_path: PathBuf::from(DEFAULT_PEP_STORE_PATH),
            max_store_bytes: DEFAULT_MAX_STORE_BYTES,
            max_rules: DEFAULT_MAX_RULES,
            default_algorithm: PepCombiningAlgorithm::DenyOverrides,
            audit_all_evaluations: true,
            auto_quarantine_corrupt: true,
        }
    }
}

impl PepConfig {
    /// Deserializes and validates a PepConfig from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: PepConfig = serde_json::from_str(json_str)
            .map_err(|e| format!("{}: failed to parse JSON: {}", PEPCONF_ERR_PARSE, e))?;
        config.validate()?;
        Ok(config)
    }

    /// Validates and serializes the PepConfig to pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: failed to serialize JSON: {}", PEPCONF_ERR_PARSE, e))
    }

    /// Loads and validates a PepConfig from a file path.
    pub fn from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err(format!("{}: config file not found at {}", PEPCONF_ERR_IO, path.display()));
        }
        if let Ok(meta) = std::fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() {
                return Err(format!("{}: config file at {} is a symlink", PEPCONF_ERR_VALIDATION, path.display()));
            }
        }
        let mut file = File::open(path)
            .map_err(|e| format!("{}: failed to open config file {}: {}", PEPCONF_ERR_IO, path.display(), e))?;
        let mut content = String::new();
        file.by_ref()
            .take(MAX_CONFIG_BYTES)
            .read_to_string(&mut content)
            .map_err(|e| format!("{}: failed to read config file {}: {}", PEPCONF_ERR_IO, path.display(), e))?;
        Self::from_json(&content)
    }

    /// Saves the validated configuration to a file path atomically.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let json_data = self.to_json()?;

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{}: failed to create directory {}: {}", PEPCONF_ERR_IO, parent.display(), e))?;
        }

        let tmp_path = parent.join(format!(".pep_config.tmp.{}", std::process::id()));
        std::fs::write(&tmp_path, json_data.as_bytes())
            .map_err(|e| format!("{}: failed to write temporary file {}: {}", PEPCONF_ERR_IO, tmp_path.display(), e))?;

        if let Err(e) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("{}: failed to rename temporary file to {}: {}", PEPCONF_ERR_IO, path.display(), e));
        }

        Ok(())
    }

    /// Loads configuration from environment variables or defaults.
    pub fn from_env() -> Result<Self, String> {
        if let Ok(env_path) = std::env::var("AIOSH_PEP_CONFIG") {
            if !env_path.trim().is_empty() {
                return Self::from_path(Path::new(&env_path));
            }
        }

        let mut config = Self::default();

        if let Ok(store_path) = std::env::var("AIOSH_PEP_STORE_PATH") {
            let trimmed = store_path.trim();
            if !trimmed.is_empty() {
                config.store_path = PathBuf::from(trimmed);
            }
        }

        if let Ok(max_rules) = std::env::var("AIOSH_PEP_MAX_RULES") {
            let trimmed = max_rules.trim();
            if !trimmed.is_empty() {
                config.max_rules = trimmed
                    .parse::<usize>()
                    .map_err(|e| format!("{}: invalid AIOSH_PEP_MAX_RULES '{}': {}", PEPCONF_ERR_BOUNDS, trimmed, e))?;
            }
        }

        if let Ok(algo_str) = std::env::var("AIOSH_PEP_DEFAULT_ALGORITHM") {
            let trimmed = algo_str.trim();
            if !trimmed.is_empty() {
                match trimmed.to_ascii_lowercase().as_str() {
                    "deny_overrides" => config.default_algorithm = PepCombiningAlgorithm::DenyOverrides,
                    "permit_overrides" => config.default_algorithm = PepCombiningAlgorithm::PermitOverrides,
                    "first_applicable" => config.default_algorithm = PepCombiningAlgorithm::FirstApplicable,
                    _ => return Err(format!("{}: invalid AIOSH_PEP_DEFAULT_ALGORITHM '{}'", PEPCONF_ERR_VALIDATION, trimmed)),
                }
            }
        }

        config.validate()?;
        Ok(config)
    }

    /// Validates the configuration against invariants PEPCONF1..PEPCONF6.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() {
            return Err(format!("{}: version cannot be empty", PEPCONF_ERR_VALIDATION));
        }

        let path_str = self.store_path.to_string_lossy();
        if path_str.len() > 1024 {
            return Err(format!("{}: store_path exceeds 1024 characters", PEPCONF_ERR_VALIDATION));
        }
        if path_str.chars().any(|c| c.is_control()) {
            return Err(format!("{}: store_path contains control characters", PEPCONF_ERR_VALIDATION));
        }
        for comp in self.store_path.components() {
            if let std::path::Component::ParentDir = comp {
                return Err(format!("{}: store_path cannot contain parent traversal ('..')", PEPCONF_ERR_VALIDATION));
            }
        }
        match self.store_path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => (),
            _ => return Err(format!("{}: store_path must have a .json extension", PEPCONF_ERR_VALIDATION)),
        }

        if self.max_store_bytes < MIN_STORE_BYTES || self.max_store_bytes > MAX_STORE_BYTES {
            return Err(format!(
                "{}: max_store_bytes {} outside allowed range [{}, {}]",
                PEPCONF_ERR_BOUNDS, self.max_store_bytes, MIN_STORE_BYTES, MAX_STORE_BYTES
            ));
        }

        if self.max_rules < MIN_RULES_COUNT || self.max_rules > MAX_RULES_COUNT {
            return Err(format!(
                "{}: max_rules {} outside allowed range [{}, {}]",
                PEPCONF_ERR_BOUNDS, self.max_rules, MIN_RULES_COUNT, MAX_RULES_COUNT
            ));
        }

        Ok(())
    }
}
