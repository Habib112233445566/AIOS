//! Capability Configuration module for AIOS Security Kernel (T-02041..T-02046).
//!
//! Contract: `docs/tasks/evidence/T-02042-configuration-specification.md`.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Maximum configuration file size in bytes (64 KiB).
pub const MAX_CONFIG_BYTES: u64 = 64 * 1024;

/// Default path for capability store JSON.
pub const DEFAULT_CAPABILITY_STORE_PATH: &str = ".aios/capability_store.json";

/// Minimum permissible store size in bytes (1 KiB).
pub const MIN_STORE_BYTES: u64 = 1024;

/// Maximum permissible store size in bytes (100 MiB).
pub const MAX_STORE_BYTES: u64 = 104_857_600;

/// Default store size in bytes (10 MiB).
pub const DEFAULT_MAX_STORE_BYTES: u64 = 10_485_760;

/// Maximum capability count in registry.
pub const MAX_CAPABILITY_COUNT: usize = 1_000_000;

/// Default capability count in registry.
pub const DEFAULT_MAX_CAPABILITY_COUNT: usize = 10_000;

/// Maximum expiration duration in seconds (10 years).
pub const MAX_EXPIRES_SECS: u64 = 315_360_000;

/// Configuration for the Capability Registry and Lifecycle Service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityConfig {
    pub version: String,
    pub store_path: PathBuf,
    pub max_store_bytes: u64,
    pub max_capabilities: usize,
    pub default_expires_secs: Option<u64>,
    pub enforce_strict_monotonic: bool,
    pub auto_prune_on_load: bool,
}

impl Default for CapabilityConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            store_path: PathBuf::from(DEFAULT_CAPABILITY_STORE_PATH),
            max_store_bytes: DEFAULT_MAX_STORE_BYTES,
            max_capabilities: DEFAULT_MAX_CAPABILITY_COUNT,
            default_expires_secs: None,
            enforce_strict_monotonic: true,
            auto_prune_on_load: true,
        }
    }
}

impl CapabilityConfig {
    /// Deserializes and validates a CapabilityConfig from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: CapabilityConfig = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse CapabilityConfig JSON: {}", e))?;
        config.validate()?;
        Ok(config)
    }

    /// Validates and serializes the CapabilityConfig to pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize CapabilityConfig: {}", e))
    }

    /// Loads and validates a CapabilityConfig from a file path.
    pub fn from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err(format!("Config file not found at {}", path.display()));
        }
        if let Ok(meta) = std::fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() {
                return Err(format!("Config file at {} is a symlink", path.display()));
            }
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

    /// Loads configuration from environment variables or defaults.
    pub fn from_env() -> Result<Self, String> {
        if let Ok(env_path) = std::env::var("AIOS_CAPABILITY_CONFIG") {
            if !env_path.trim().is_empty() {
                return Self::from_path(Path::new(&env_path));
            }
        }

        let mut config = Self::default();

        if let Ok(store_path) = std::env::var("AIOS_CAPABILITY_STORE_PATH") {
            let trimmed = store_path.trim();
            if !trimmed.is_empty() {
                config.store_path = PathBuf::from(trimmed);
            }
        }

        if let Ok(max_caps) = std::env::var("AIOS_CAPABILITY_MAX_CAPABILITIES") {
            let trimmed = max_caps.trim();
            if !trimmed.is_empty() {
                config.max_capabilities = trimmed
                    .parse::<usize>()
                    .map_err(|e| format!("Invalid AIOS_CAPABILITY_MAX_CAPABILITIES '{}': {}", trimmed, e))?;
            }
        }

        if let Ok(max_bytes) = std::env::var("AIOS_CAPABILITY_MAX_STORE_BYTES") {
            let trimmed = max_bytes.trim();
            if !trimmed.is_empty() {
                config.max_store_bytes = trimmed
                    .parse::<u64>()
                    .map_err(|e| format!("Invalid AIOS_CAPABILITY_MAX_STORE_BYTES '{}': {}", trimmed, e))?;
            }
        }

        config.validate()?;
        Ok(config)
    }

    /// Validates all configuration invariants and bounds.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() {
            return Err("CapabilityConfig 'version' must not be empty".into());
        }
        if self.version.len() > 32 {
            return Err("CapabilityConfig 'version' exceeds maximum length of 32 characters".into());
        }
        if self.version.chars().any(|c| c.is_control()) {
            return Err("CapabilityConfig 'version' contains control characters".into());
        }

        let path_str = self.store_path.to_string_lossy();
        if path_str.trim().is_empty() {
            return Err("CapabilityConfig 'store_path' must not be empty".into());
        }
        if path_str.len() > 1024 {
            return Err("CapabilityConfig 'store_path' exceeds maximum length of 1024 characters".into());
        }
        if path_str.chars().any(|c| c.is_control()) {
            return Err("CapabilityConfig 'store_path' contains control characters".into());
        }
        for component in self.store_path.components() {
            if let std::path::Component::ParentDir = component {
                return Err("CapabilityConfig 'store_path' path traversal ('..') is not allowed".into());
            }
        }
        match self.store_path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => (),
            _ => return Err("CapabilityConfig 'store_path' must have a .json extension".into()),
        }

        if self.max_store_bytes < MIN_STORE_BYTES || self.max_store_bytes > MAX_STORE_BYTES {
            return Err(format!(
                "CapabilityConfig 'max_store_bytes' ({}) must be between {} and {} bytes",
                self.max_store_bytes, MIN_STORE_BYTES, MAX_STORE_BYTES
            ));
        }

        if self.max_capabilities == 0 || self.max_capabilities > MAX_CAPABILITY_COUNT {
            return Err(format!(
                "CapabilityConfig 'max_capabilities' ({}) must be between 1 and {}",
                self.max_capabilities, MAX_CAPABILITY_COUNT
            ));
        }

        if let Some(expires) = self.default_expires_secs {
            if expires == 0 || expires > MAX_EXPIRES_SECS {
                return Err(format!(
                    "CapabilityConfig 'default_expires_secs' ({}) must be between 1 and {}",
                    expires, MAX_EXPIRES_SECS
                ));
            }
        }

        Ok(())
    }

    pub fn store_path(&self) -> &Path {
        &self.store_path
    }

    pub fn max_store_bytes(&self) -> u64 {
        self.max_store_bytes
    }

    pub fn max_capabilities(&self) -> usize {
        self.max_capabilities
    }

    pub fn default_expires_secs(&self) -> Option<u64> {
        self.default_expires_secs
    }

    pub fn enforce_strict_monotonic(&self) -> bool {
        self.enforce_strict_monotonic
    }

    pub fn auto_prune_on_load(&self) -> bool {
        self.auto_prune_on_load
    }
}
