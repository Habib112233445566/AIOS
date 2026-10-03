//! Secrets Handling Configuration Subsystem (T-02641..T-02650).
//!
//! Governs runtime parameters, capacity ceilings, file paths, and security constraints
//! for secret storage in `aiosh-core`.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DEFAULT_SECRETS_CONFIG_PATH: &str = ".aios/secrets_config.json";
pub const DEFAULT_SECRETS_STORE_PATH: &str = "target/secrets_vault.json";

pub const DEFAULT_MAX_SECRETS_CAPACITY: usize = 1024;
pub const MIN_MAX_SECRETS_CAPACITY: usize = 1;
pub const MAX_MAX_SECRETS_CAPACITY: usize = 16384;

pub const DEFAULT_MAX_PAYLOAD_BYTES: usize = 65536; // 64 KiB
pub const MIN_MAX_PAYLOAD_BYTES: usize = 1;
pub const MAX_MAX_PAYLOAD_BYTES: usize = 1048576; // 1 MiB

pub const DEFAULT_MAX_STORE_FILE_BYTES: u64 = 1048576; // 1 MiB
pub const MIN_MAX_STORE_FILE_BYTES: u64 = 4096; // 4 KiB
pub const MAX_MAX_STORE_FILE_BYTES: u64 = 16777216; // 16 MiB

pub const MAX_CONFIG_FILE_BYTES: u64 = 65536; // 64 KiB

pub const SECCONF_ERR_IO: &str = "SECCONF_ERR_IO";
pub const SECCONF_ERR_PARSE: &str = "SECCONF_ERR_PARSE";
pub const SECCONF_ERR_VALIDATION: &str = "SECCONF_ERR_VALIDATION";
pub const SECCONF_ERR_BOUNDS: &str = "SECCONF_ERR_BOUNDS";

/// Configuration settings for the Secrets Handling subsystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretConfig {
    pub version: String,
    pub store_path: PathBuf,
    pub max_secrets_capacity: usize,
    pub max_payload_bytes: usize,
    pub max_store_file_bytes: u64,
    pub require_expose_flag: bool,
    pub enforce_scope_containment: bool,
    pub audit_all_reads: bool,
    pub audit_all_writes: bool,
}

impl Default for SecretConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            store_path: PathBuf::from(DEFAULT_SECRETS_STORE_PATH),
            max_secrets_capacity: DEFAULT_MAX_SECRETS_CAPACITY,
            max_payload_bytes: DEFAULT_MAX_PAYLOAD_BYTES,
            max_store_file_bytes: DEFAULT_MAX_STORE_FILE_BYTES,
            require_expose_flag: true,
            enforce_scope_containment: true,
            audit_all_reads: true,
            audit_all_writes: true,
        }
    }
}

impl SecretConfig {
    /// Validates structural invariants and numerical boundaries.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.is_empty() || !self.version.starts_with("1.") {
            return Err(format!("{}: invalid version '{}'", SECCONF_ERR_VALIDATION, self.version));
        }
        if self.store_path.to_string_lossy().contains("..") {
            return Err(format!("{}: store_path cannot contain directory traversal '..'", SECCONF_ERR_VALIDATION));
        }
        if self.max_secrets_capacity < MIN_MAX_SECRETS_CAPACITY || self.max_secrets_capacity > MAX_MAX_SECRETS_CAPACITY {
            return Err(format!(
                "{}: max_secrets_capacity {} out of bounds ({}..{})",
                SECCONF_ERR_BOUNDS, self.max_secrets_capacity, MIN_MAX_SECRETS_CAPACITY, MAX_MAX_SECRETS_CAPACITY
            ));
        }
        if self.max_payload_bytes < MIN_MAX_PAYLOAD_BYTES || self.max_payload_bytes > MAX_MAX_PAYLOAD_BYTES {
            return Err(format!(
                "{}: max_payload_bytes {} out of bounds ({}..{})",
                SECCONF_ERR_BOUNDS, self.max_payload_bytes, MIN_MAX_PAYLOAD_BYTES, MAX_MAX_PAYLOAD_BYTES
            ));
        }
        if self.max_store_file_bytes < MIN_MAX_STORE_FILE_BYTES || self.max_store_file_bytes > MAX_MAX_STORE_FILE_BYTES {
            return Err(format!(
                "{}: max_store_file_bytes {} out of bounds ({}..{})",
                SECCONF_ERR_BOUNDS, self.max_store_file_bytes, MIN_MAX_STORE_FILE_BYTES, MAX_MAX_STORE_FILE_BYTES
            ));
        }
        Ok(())
    }

    /// Loads configuration from a JSON file path with bounds checking.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let meta = std::fs::symlink_metadata(p)
            .map_err(|e| format!("{}: cannot stat {:?}: {}", SECCONF_ERR_IO, p, e))?;
        if meta.file_type().is_symlink() {
            return Err(format!("{}: symlinks rejected for config files", SECCONF_ERR_VALIDATION));
        }
        if meta.len() > MAX_CONFIG_FILE_BYTES {
            return Err(format!("{}: config file size {} exceeds max {}", SECCONF_ERR_BOUNDS, meta.len(), MAX_CONFIG_FILE_BYTES));
        }
        let data = std::fs::read_to_string(p)
            .map_err(|e| format!("{}: cannot read {:?}: {}", SECCONF_ERR_IO, p, e))?;
        let config: Self = serde_json::from_str(&data)
            .map_err(|e| format!("{}: invalid JSON in {:?}: {}", SECCONF_ERR_PARSE, p, e))?;
        config.validate()?;
        Ok(config)
    }

    /// Atomically saves configuration to a JSON file.
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.validate()?;
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{}: cannot create directory {:?}: {}", SECCONF_ERR_IO, parent, e))?;
        }
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: serialization failed: {}", SECCONF_ERR_PARSE, e))?;
        let temp_path = p.with_extension(format!("tmp.{}", std::process::id()));
        std::fs::write(&temp_path, &data)
            .map_err(|e| format!("{}: failed writing tempfile: {}", SECCONF_ERR_IO, e))?;
        std::fs::rename(&temp_path, p).map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            format!("{}: failed renaming to {:?}: {}", SECCONF_ERR_IO, p, e)
        })?;
        Ok(())
    }

    /// Loads configuration with environment variable overrides.
    pub fn from_env() -> Self {
        let mut config = if let Ok(custom_path) = std::env::var("AIOS_SECRETS_CONFIG") {
            if !custom_path.contains("..") {
                Self::load_from_path(&custom_path).unwrap_or_default()
            } else {
                Self::default()
            }
        } else if Path::new(DEFAULT_SECRETS_CONFIG_PATH).exists() {
            Self::load_from_path(DEFAULT_SECRETS_CONFIG_PATH).unwrap_or_default()
        } else {
            Self::default()
        };

        if let Ok(store) = std::env::var("AIOS_SECRETS_STORE") {
            if !store.contains("..") {
                config.store_path = PathBuf::from(store);
            }
        }
        if let Ok(cap_str) = std::env::var("AIOS_SECRETS_MAX_CAPACITY") {
            if let Ok(cap) = cap_str.parse::<usize>() {
                if (MIN_MAX_SECRETS_CAPACITY..=MAX_MAX_SECRETS_CAPACITY).contains(&cap) {
                    config.max_secrets_capacity = cap;
                }
            }
        }
        if let Ok(expose_str) = std::env::var("AIOS_SECRETS_REQUIRE_EXPOSE") {
            if let Ok(val) = expose_str.parse::<bool>() {
                config.require_expose_flag = val;
            }
        }

        config
    }
}
