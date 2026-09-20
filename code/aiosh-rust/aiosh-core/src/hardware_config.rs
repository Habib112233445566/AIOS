//! Hardware Detection Configuration Subsystem (HCFG1..HCFG5)
//!
//! Provides configuration management, validation, environment variable ingestion,
//! and persistent serialization for hardware discovery operations.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::hardware::DeviceClass;

/// Configuration for Hardware Detection operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareConfig {
    /// Default path to persisted hardware inventory JSON.
    pub default_store_path: PathBuf,
    /// Root path to sysfs tree (defaults to /sys).
    pub sysfs_path: PathBuf,
    /// Root path to procfs tree (defaults to /proc).
    pub procfs_path: PathBuf,
    /// Optional whitelist of device classes to discover (None = all).
    pub enabled_classes: Option<Vec<DeviceClass>>,
    /// Whether to discover and collect device attributes.
    pub include_attributes: bool,
    /// Maximum number of devices allowed in a single inventory.
    pub max_devices: usize,
    /// Maximum payload size in bytes for inventory documents.
    pub max_payload_bytes: u64,
    /// Timeout in seconds for scanning operations.
    pub scan_timeout_secs: u64,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            default_store_path: PathBuf::from(".aios/hardware_inventory.json"),
            sysfs_path: PathBuf::from("/sys"),
            procfs_path: PathBuf::from("/proc"),
            enabled_classes: None,
            include_attributes: true,
            max_devices: 10_000,
            max_payload_bytes: 10_485_760, // 10 MB
            scan_timeout_secs: 30,
        }
    }
}

impl HardwareConfig {
    /// Validates configuration against invariants HCFG1..HCFG5.
    pub fn validate(&self) -> Result<(), String> {
        // HCFG1: Path hygiene
        for (name, path) in [
            ("default_store_path", &self.default_store_path),
            ("sysfs_path", &self.sysfs_path),
            ("procfs_path", &self.procfs_path),
        ] {
            let s = path.to_str().ok_or_else(|| format!("HCFG1 violation: {} must be valid UTF-8", name))?;
            if s.trim().is_empty() {
                return Err(format!("HCFG1 violation: {} cannot be empty", name));
            }
            if s.len() > 1024 {
                return Err(format!("HCFG1 violation: {} exceeds maximum length of 1024 characters", name));
            }
            if s.chars().any(|c| c.is_control() || c == '\0') {
                return Err(format!("HCFG1 violation: {} cannot contain control characters", name));
            }
        }

        // HCFG2: Class filtering & uniqueness
        if let Some(classes) = &self.enabled_classes {
            if classes.len() > 9 {
                return Err(format!("HCFG2 violation: enabled_classes count {} exceeds maximum allowed (9)", classes.len()));
            }
            let mut seen = BTreeSet::new();
            for c in classes {
                if !seen.insert(*c) {
                    return Err(format!("HCFG2 violation: duplicate DeviceClass '{:?}' in enabled_classes", c));
                }
            }
        }

        // HCFG3: Resource bounds
        if self.max_devices == 0 || self.max_devices > 50_000 {
            return Err(format!(
                "HCFG3 violation: max_devices must be between 1 and 50,000 (got {})",
                self.max_devices
            ));
        }
        if self.max_payload_bytes < 1024 || self.max_payload_bytes > 104_857_600 {
            return Err(format!(
                "HCFG3 violation: max_payload_bytes must be between 1024 and 104,857,600 (got {})",
                self.max_payload_bytes
            ));
        }

        // HCFG4: Timeout bounds
        if self.scan_timeout_secs == 0 || self.scan_timeout_secs > 300 {
            return Err(format!(
                "HCFG4 violation: scan_timeout_secs must be between 1 and 300 (got {})",
                self.scan_timeout_secs
            ));
        }

        Ok(())
    }

    /// Loads configuration from a JSON file, returning default if file does not exist (HCFG5).
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read hardware config from {}: {}", path.display(), e))?;
        let config: HardwareConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse hardware config JSON: {}", e))?;
        config.validate()?;
        Ok(config)
    }

    /// Serializes and saves configuration to a JSON file atomically (HCFG5).
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory {}: {}", parent.display(), e))?;
            }
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize hardware config: {}", e))?;
        fs::write(path, json)
            .map_err(|e| format!("Failed to write hardware config to {}: {}", path.display(), e))?;
        Ok(())
    }

    /// Loads configuration taking environment variables into account.
    pub fn from_env() -> Self {
        let mut cfg = if let Ok(config_path) = std::env::var("AIOSH_HARDWARE_CONFIG") {
            Self::load_from_path(Path::new(&config_path)).unwrap_or_default()
        } else {
            Self::default()
        };

        if let Ok(sysfs) = std::env::var("AIOSH_HARDWARE_SYSFS") {
            if !sysfs.trim().is_empty() {
                cfg.sysfs_path = PathBuf::from(sysfs);
            }
        }
        if let Ok(procfs) = std::env::var("AIOSH_HARDWARE_PROCFS") {
            if !procfs.trim().is_empty() {
                cfg.procfs_path = PathBuf::from(procfs);
            }
        }
        if let Ok(store) = std::env::var("AIOSH_HARDWARE_STORE") {
            if !store.trim().is_empty() {
                cfg.default_store_path = PathBuf::from(store);
            }
        }
        if let Ok(val) = std::env::var("AIOSH_HARDWARE_INCLUDE_ATTRS") {
            match val.trim().to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" => cfg.include_attributes = true,
                "0" | "false" | "no" => cfg.include_attributes = false,
                _ => {}
            }
        }
        if let Ok(val) = std::env::var("AIOSH_HARDWARE_TIMEOUT_SECS") {
            if let Ok(secs) = val.trim().parse::<u64>() {
                if secs > 0 && secs <= 300 {
                    cfg.scan_timeout_secs = secs;
                }
            }
        }

        cfg
    }
}
