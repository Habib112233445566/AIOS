//! Privilege Escalation Prevention Configuration Subsystem (T-02541..T-02550).
//!
//! Governs runtime parameters, capacity ceilings, file paths, and security policies
//! for dynamic privilege management in `aiosh-core`.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::privilege_data_model::PrivilegeLevel;

pub const DEFAULT_PRIVILEGE_CONFIG_PATH: &str = ".aios/privilege_config.json";
pub const DEFAULT_PRIVILEGE_STORE_PATH: &str = "target/privilege_state.json";

pub const DEFAULT_MAX_ACTIVE_CONTEXTS: usize = 1024;
pub const MIN_MAX_ACTIVE_CONTEXTS: usize = 1;
pub const MAX_MAX_ACTIVE_CONTEXTS: usize = 16384;

pub const DEFAULT_MAX_GRANT_DURATION_SECONDS: u64 = 3600;
pub const MIN_MAX_GRANT_DURATION_SECONDS: u64 = 1;
pub const MAX_MAX_GRANT_DURATION_SECONDS: u64 = 86400;

pub const DEFAULT_MAX_CAPABILITIES_PER_CONTEXT: usize = 32;
pub const MIN_MAX_CAPABILITIES_PER_CONTEXT: usize = 1;
pub const MAX_MAX_CAPABILITIES_PER_CONTEXT: usize = 64;

pub const MAX_CONFIG_FILE_BYTES: u64 = 64 * 1024; // 64 KiB

pub const PRIVESCCONF_ERR_IO: &str = "PRIVESCCONF_ERR_IO";
pub const PRIVESCCONF_ERR_PARSE: &str = "PRIVESCCONF_ERR_PARSE";
pub const PRIVESCCONF_ERR_VALIDATION: &str = "PRIVESCCONF_ERR_VALIDATION";
pub const PRIVESCCONF_ERR_BOUNDS: &str = "PRIVESCCONF_ERR_BOUNDS";

/// Configuration settings for the Privilege Escalation Prevention subsystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeConfig {
    pub version: String,
    pub store_path: PathBuf,
    pub default_tier: PrivilegeLevel,
    pub max_active_contexts: usize,
    pub max_grant_duration_seconds: u64,
    pub max_capabilities_per_context: usize,
    pub allow_guest_contexts: bool,
    pub enforce_grant_signatures: bool,
    pub audit_all_transitions: bool,
}

impl Default for PrivilegeConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            store_path: PathBuf::from(DEFAULT_PRIVILEGE_STORE_PATH),
            default_tier: PrivilegeLevel::User,
            max_active_contexts: DEFAULT_MAX_ACTIVE_CONTEXTS,
            max_grant_duration_seconds: DEFAULT_MAX_GRANT_DURATION_SECONDS,
            max_capabilities_per_context: DEFAULT_MAX_CAPABILITIES_PER_CONTEXT,
            allow_guest_contexts: true,
            enforce_grant_signatures: true,
            audit_all_transitions: true,
        }
    }
}

impl PrivilegeConfig {
    /// Validates structural invariants and numerical boundaries.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.is_empty() || !self.version.starts_with("1.") {
            return Err(format!("{}: invalid version '{}'", PRIVESCCONF_ERR_VALIDATION, self.version));
        }
        if self.store_path.to_string_lossy().contains("..") {
            return Err(format!("{}: store_path cannot contain directory traversal '..'", PRIVESCCONF_ERR_VALIDATION));
        }
        if self.max_active_contexts < MIN_MAX_ACTIVE_CONTEXTS || self.max_active_contexts > MAX_MAX_ACTIVE_CONTEXTS {
            return Err(format!(
                "{}: max_active_contexts {} out of bounds ({}..{})",
                PRIVESCCONF_ERR_BOUNDS, self.max_active_contexts, MIN_MAX_ACTIVE_CONTEXTS, MAX_MAX_ACTIVE_CONTEXTS
            ));
        }
        if self.max_grant_duration_seconds < MIN_MAX_GRANT_DURATION_SECONDS || self.max_grant_duration_seconds > MAX_MAX_GRANT_DURATION_SECONDS {
            return Err(format!(
                "{}: max_grant_duration_seconds {} out of bounds ({}..{})",
                PRIVESCCONF_ERR_BOUNDS, self.max_grant_duration_seconds, MIN_MAX_GRANT_DURATION_SECONDS, MAX_MAX_GRANT_DURATION_SECONDS
            ));
        }
        if self.max_capabilities_per_context < MIN_MAX_CAPABILITIES_PER_CONTEXT || self.max_capabilities_per_context > MAX_MAX_CAPABILITIES_PER_CONTEXT {
            return Err(format!(
                "{}: max_capabilities_per_context {} out of bounds ({}..{})",
                PRIVESCCONF_ERR_BOUNDS, self.max_capabilities_per_context, MIN_MAX_CAPABILITIES_PER_CONTEXT, MAX_MAX_CAPABILITIES_PER_CONTEXT
            ));
        }
        Ok(())
    }

    /// Loads configuration from a JSON file path with bounds checking.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let meta = std::fs::metadata(p).map_err(|e| format!("{}: cannot stat {:?}: {}", PRIVESCCONF_ERR_IO, p, e))?;
        if meta.len() > MAX_CONFIG_FILE_BYTES {
            return Err(format!("{}: config file size {} exceeds max {}", PRIVESCCONF_ERR_BOUNDS, meta.len(), MAX_CONFIG_FILE_BYTES));
        }
        let data = std::fs::read_to_string(p).map_err(|e| format!("{}: cannot read {:?}: {}", PRIVESCCONF_ERR_IO, p, e))?;
        let cfg: Self = serde_json::from_str(&data).map_err(|e| format!("{}: JSON parse error: {}", PRIVESCCONF_ERR_PARSE, e))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Saves configuration to a JSON file path.
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.validate()?;
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: serialization error: {}", PRIVESCCONF_ERR_PARSE, e))?;
        std::fs::write(p, json_str)
            .map_err(|e| format!("{}: cannot write {:?}: {}", PRIVESCCONF_ERR_IO, p, e))?;
        Ok(())
    }

    /// Loads configuration merged with supported environment variable overrides.
    pub fn load_with_env_overrides() -> Self {
        let mut cfg = if let Ok(path) = std::env::var("AIOS_PRIVILEGE_CONFIG_PATH") {
            if !path.contains("..") {
                Self::load_from_path(&path).unwrap_or_default()
            } else {
                Self::default()
            }
        } else if let Ok(cfg) = Self::load_from_path(DEFAULT_PRIVILEGE_CONFIG_PATH) {
            cfg
        } else {
            Self::default()
        };

        if let Ok(store_path) = std::env::var("AIOS_PRIVILEGE_STORE_PATH").or_else(|_| std::env::var("AIOS_PRIVILEGE_STORE")) {
            if !store_path.contains("..") {
                cfg.store_path = PathBuf::from(store_path.trim());
            }
        }

        if let Ok(tier_str) = std::env::var("AIOS_PRIVILEGE_DEFAULT_TIER") {
            if let Some(tier) = PrivilegeLevel::parse_level(&tier_str) {
                if tier != PrivilegeLevel::SystemKernel {
                    cfg.default_tier = tier;
                }
            }
        }

        if let Ok(max_ctx_str) = std::env::var("AIOS_PRIVILEGE_MAX_CONTEXTS") {
            if let Ok(val) = max_ctx_str.trim().parse::<usize>() {
                cfg.max_active_contexts = val.clamp(MIN_MAX_ACTIVE_CONTEXTS, MAX_MAX_ACTIVE_CONTEXTS);
            }
        }

        if let Ok(dur_str) = std::env::var("AIOS_PRIVILEGE_MAX_GRANT_DURATION") {
            if let Ok(val) = dur_str.trim().parse::<u64>() {
                cfg.max_grant_duration_seconds = val.clamp(MIN_MAX_GRANT_DURATION_SECONDS, MAX_MAX_GRANT_DURATION_SECONDS);
            }
        }

        if let Ok(audit_str) = std::env::var("AIOS_PRIVILEGE_AUDIT_ALL") {
            match audit_str.trim().to_lowercase().as_str() {
                "1" | "true" | "yes" => cfg.audit_all_transitions = true,
                "0" | "false" | "no" => cfg.audit_all_transitions = false,
                _ => {}
            }
        }

        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_valid() {
        let cfg = PrivilegeConfig::default();
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.version, "1.0.0");
        assert_eq!(cfg.max_active_contexts, DEFAULT_MAX_ACTIVE_CONTEXTS);
    }

    #[test]
    fn test_bounds_validation() {
        let mut cfg = PrivilegeConfig::default();

        // Invalid version
        cfg.version = "2.0.0".into();
        let err = cfg.validate().unwrap_err();
        assert!(err.contains(PRIVESCCONF_ERR_VALIDATION));
        cfg.version = "1.0.0".into();

        // Directory traversal in store_path
        cfg.store_path = PathBuf::from("../escaped/store.json");
        let err = cfg.validate().unwrap_err();
        assert!(err.contains(PRIVESCCONF_ERR_VALIDATION));
        cfg.store_path = PathBuf::from(DEFAULT_PRIVILEGE_STORE_PATH);

        // Context bounds
        cfg.max_active_contexts = 0;
        let err = cfg.validate().unwrap_err();
        assert!(err.contains(PRIVESCCONF_ERR_BOUNDS));
        cfg.max_active_contexts = 20000;
        let err = cfg.validate().unwrap_err();
        assert!(err.contains(PRIVESCCONF_ERR_BOUNDS));
        cfg.max_active_contexts = 512;
        assert!(cfg.validate().is_ok());

        // Grant duration bounds
        cfg.max_grant_duration_seconds = 0;
        let err = cfg.validate().unwrap_err();
        assert!(err.contains(PRIVESCCONF_ERR_BOUNDS));
        cfg.max_grant_duration_seconds = 100000;
        let err = cfg.validate().unwrap_err();
        assert!(err.contains(PRIVESCCONF_ERR_BOUNDS));
        cfg.max_grant_duration_seconds = 1800;
        assert!(cfg.validate().is_ok());

        // Capability bounds
        cfg.max_capabilities_per_context = 0;
        let err = cfg.validate().unwrap_err();
        assert!(err.contains(PRIVESCCONF_ERR_BOUNDS));
        cfg.max_capabilities_per_context = 100;
        let err = cfg.validate().unwrap_err();
        assert!(err.contains(PRIVESCCONF_ERR_BOUNDS));
        cfg.max_capabilities_per_context = 16;
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_persistence_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("privilege_config.json");

        let mut original = PrivilegeConfig::default();
        original.max_active_contexts = 256;
        original.max_grant_duration_seconds = 7200;

        assert!(original.save_to_path(&path).is_ok());
        let loaded = PrivilegeConfig::load_from_path(&path).unwrap();
        assert_eq!(original, loaded);
    }

    #[test]
    fn test_oversized_file_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("oversized_config.json");

        // Write a file exceeding 64 KiB
        let oversized = vec![b' '; (MAX_CONFIG_FILE_BYTES + 1024) as usize];
        std::fs::write(&path, oversized).unwrap();

        let res = PrivilegeConfig::load_from_path(&path);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains(PRIVESCCONF_ERR_BOUNDS));
    }
}

