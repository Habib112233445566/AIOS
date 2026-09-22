//! PEP Grant Lifecycle Configuration Module (T-02241..T-02250).
//!
//! Contract: `docs/tasks/evidence/T-02242-configuration-specification.md`.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Maximum configuration file size in bytes (64 KiB).
pub const MAX_CONFIG_BYTES: u64 = 64 * 1024;

/// Default path for the PEP grant store JSON file.
pub const DEFAULT_PEP_GRANT_STORE_PATH: &str = ".aios/pep_grants.json";

/// Minimum permissible store size in bytes (1 KiB).
pub const MIN_STORE_BYTES: u64 = 1024;

/// Maximum permissible store size in bytes (100 MiB).
pub const MAX_STORE_BYTES: u64 = 104_857_600;

/// Default maximum store size in bytes (10 MiB).
pub const DEFAULT_MAX_STORE_BYTES: u64 = 10_485_760;

/// Minimum grant capacity.
pub const MIN_GRANTS_COUNT: usize = 1;

/// Maximum grant capacity in registry.
pub const MAX_GRANTS_COUNT: usize = 50_000;

/// Default grant capacity in registry.
pub const DEFAULT_MAX_GRANTS: usize = 5_000;

/// Minimum delegation depth.
pub const MIN_DELEGATION_DEPTH: u32 = 1;

/// Maximum delegation depth.
pub const MAX_DELEGATION_DEPTH: u32 = 10;

/// Default delegation depth.
pub const DEFAULT_MAX_DELEGATION_DEPTH: u32 = 3;

pub const GRANTCONF_ERR_IO: &str = "GRANTCONF_ERR_IO";
pub const GRANTCONF_ERR_PARSE: &str = "GRANTCONF_ERR_PARSE";
pub const GRANTCONF_ERR_VALIDATION: &str = "GRANTCONF_ERR_VALIDATION";
pub const GRANTCONF_ERR_BOUNDS: &str = "GRANTCONF_ERR_BOUNDS";

/// Configuration for the PEP Grant Lifecycle Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PepGrantConfig {
    pub version: String,
    pub store_path: PathBuf,
    pub max_store_bytes: u64,
    pub max_grants: usize,
    pub default_max_delegation_depth: u32,
    pub auto_sweep_on_load: bool,
    pub cascade_revocation_by_default: bool,
}

impl Default for PepGrantConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            store_path: PathBuf::from(DEFAULT_PEP_GRANT_STORE_PATH),
            max_store_bytes: DEFAULT_MAX_STORE_BYTES,
            max_grants: DEFAULT_MAX_GRANTS,
            default_max_delegation_depth: DEFAULT_MAX_DELEGATION_DEPTH,
            auto_sweep_on_load: true,
            cascade_revocation_by_default: false,
        }
    }
}

impl PepGrantConfig {
    /// Deserializes and validates a PepGrantConfig from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: PepGrantConfig = serde_json::from_str(json_str)
            .map_err(|e| format!("{}: failed to parse JSON: {}", GRANTCONF_ERR_PARSE, e))?;
        config.validate()?;
        Ok(config)
    }

    /// Validates and serializes the PepGrantConfig to pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: failed to serialize JSON: {}", GRANTCONF_ERR_PARSE, e))
    }

    /// Loads and validates a PepGrantConfig from a file path.
    pub fn from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err(format!("{}: config file not found at {}", GRANTCONF_ERR_IO, path.display()));
        }
        if let Ok(meta) = std::fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() {
                return Err(format!("{}: config file at {} is a symlink", GRANTCONF_ERR_VALIDATION, path.display()));
            }
        }
        let mut file = File::open(path)
            .map_err(|e| format!("{}: failed to open config file {}: {}", GRANTCONF_ERR_IO, path.display(), e))?;
        let mut content = String::new();
        file.by_ref()
            .take(MAX_CONFIG_BYTES)
            .read_to_string(&mut content)
            .map_err(|e| format!("{}: failed to read config file {}: {}", GRANTCONF_ERR_IO, path.display(), e))?;
        Self::from_json(&content)
    }

    /// Saves the validated configuration to a file path atomically.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let json_data = self.to_json()?;

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{}: failed to create directory {}: {}", GRANTCONF_ERR_IO, parent.display(), e))?;
        }

        let tmp_path = parent.join(format!(".pep_grant_config.tmp.{}", std::process::id()));
        std::fs::write(&tmp_path, json_data.as_bytes())
            .map_err(|e| format!("{}: failed to write temporary file {}: {}", GRANTCONF_ERR_IO, tmp_path.display(), e))?;

        if let Err(e) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("{}: failed to rename temporary file to {}: {}", GRANTCONF_ERR_IO, path.display(), e));
        }

        Ok(())
    }

    /// Loads configuration from environment variables or defaults.
    pub fn from_env() -> Result<Self, String> {
        if let Ok(env_path) = std::env::var("AIOSH_PEP_GRANT_CONFIG") {
            if !env_path.trim().is_empty() {
                return Self::from_path(Path::new(&env_path));
            }
        }

        let mut config = Self::default();

        let store_var = std::env::var("AIOSH_PEP_GRANT_STORE_PATH")
            .or_else(|_| std::env::var("AIOSH_PEP_GRANT_STORE"));
        if let Ok(store_path) = store_var {
            let trimmed = store_path.trim();
            if !trimmed.is_empty() {
                config.store_path = PathBuf::from(trimmed);
            }
        }

        if let Ok(max_grants_str) = std::env::var("AIOSH_PEP_GRANT_MAX_GRANTS") {
            let trimmed = max_grants_str.trim();
            if !trimmed.is_empty() {
                config.max_grants = trimmed
                    .parse::<usize>()
                    .map_err(|e| format!("{}: invalid AIOSH_PEP_GRANT_MAX_GRANTS '{}': {}", GRANTCONF_ERR_BOUNDS, trimmed, e))?;
            }
        }

        if let Ok(max_store_str) = std::env::var("AIOSH_PEP_GRANT_MAX_STORE_BYTES") {
            let trimmed = max_store_str.trim();
            if !trimmed.is_empty() {
                config.max_store_bytes = trimmed
                    .parse::<u64>()
                    .map_err(|e| format!("{}: invalid AIOSH_PEP_GRANT_MAX_STORE_BYTES '{}': {}", GRANTCONF_ERR_BOUNDS, trimmed, e))?;
            }
        }

        if let Ok(depth_str) = std::env::var("AIOSH_PEP_GRANT_MAX_DELEGATION_DEPTH") {
            let trimmed = depth_str.trim();
            if !trimmed.is_empty() {
                config.default_max_delegation_depth = trimmed
                    .parse::<u32>()
                    .map_err(|e| format!("{}: invalid AIOSH_PEP_GRANT_MAX_DELEGATION_DEPTH '{}': {}", GRANTCONF_ERR_BOUNDS, trimmed, e))?;
            }
        }

        if let Ok(sweep_str) = std::env::var("AIOSH_PEP_GRANT_AUTO_SWEEP") {
            let trimmed = sweep_str.trim().to_ascii_lowercase();
            if !trimmed.is_empty() {
                match trimmed.as_str() {
                    "true" | "1" | "yes" => config.auto_sweep_on_load = true,
                    "false" | "0" | "no" => config.auto_sweep_on_load = false,
                    _ => return Err(format!("{}: invalid AIOSH_PEP_GRANT_AUTO_SWEEP '{}'", GRANTCONF_ERR_VALIDATION, trimmed)),
                }
            }
        }

        if let Ok(cascade_str) = std::env::var("AIOSH_PEP_GRANT_CASCADE_REVOCATION") {
            let trimmed = cascade_str.trim().to_ascii_lowercase();
            if !trimmed.is_empty() {
                match trimmed.as_str() {
                    "true" | "1" | "yes" => config.cascade_revocation_by_default = true,
                    "false" | "0" | "no" => config.cascade_revocation_by_default = false,
                    _ => return Err(format!("{}: invalid AIOSH_PEP_GRANT_CASCADE_REVOCATION '{}'", GRANTCONF_ERR_VALIDATION, trimmed)),
                }
            }
        }

        config.validate()?;
        Ok(config)
    }

    /// Validates the configuration against invariants GRANTCONF1..GRANTCONF6.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() {
            return Err(format!("{}: version cannot be empty", GRANTCONF_ERR_VALIDATION));
        }

        let path_str = self.store_path.to_string_lossy();
        if path_str.len() > 1024 {
            return Err(format!("{}: store_path exceeds 1024 characters", GRANTCONF_ERR_VALIDATION));
        }
        if path_str.chars().any(|c| c.is_control()) {
            return Err(format!("{}: store_path contains control characters", GRANTCONF_ERR_VALIDATION));
        }
        for comp in self.store_path.components() {
            if let std::path::Component::ParentDir = comp {
                return Err(format!("{}: store_path cannot contain parent traversal ('..')", GRANTCONF_ERR_VALIDATION));
            }
        }
        match self.store_path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => (),
            _ => return Err(format!("{}: store_path must have a .json extension", GRANTCONF_ERR_VALIDATION)),
        }

        if self.max_store_bytes < MIN_STORE_BYTES || self.max_store_bytes > MAX_STORE_BYTES {
            return Err(format!(
                "{}: max_store_bytes {} outside allowed range [{}, {}]",
                GRANTCONF_ERR_BOUNDS, self.max_store_bytes, MIN_STORE_BYTES, MAX_STORE_BYTES
            ));
        }

        if self.max_grants < MIN_GRANTS_COUNT || self.max_grants > MAX_GRANTS_COUNT {
            return Err(format!(
                "{}: max_grants {} outside allowed range [{}, {}]",
                GRANTCONF_ERR_BOUNDS, self.max_grants, MIN_GRANTS_COUNT, MAX_GRANTS_COUNT
            ));
        }

        if self.default_max_delegation_depth < MIN_DELEGATION_DEPTH || self.default_max_delegation_depth > MAX_DELEGATION_DEPTH {
            return Err(format!(
                "{}: default_max_delegation_depth {} outside allowed range [{}, {}]",
                GRANTCONF_ERR_BOUNDS, self.default_max_delegation_depth, MIN_DELEGATION_DEPTH, MAX_DELEGATION_DEPTH
            ));
        }

        Ok(())
    }
}
