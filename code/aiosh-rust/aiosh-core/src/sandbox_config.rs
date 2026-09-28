//! Sandbox Enforcement Configuration Subsystem (T-02441..T-02450).
//!
//! Provides configuration management, schema validation, persistence, and environment
//! overrides for Sandbox Enforcement.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DEFAULT_SANDBOX_CONFIG_PATH: &str = ".aios/sandbox.json";
pub const DEFAULT_MAX_OUTPUT_CAPTURE_BYTES: usize = 10 * 1024 * 1024; // 10 MiB
pub const MIN_OUTPUT_CAPTURE_BYTES: usize = 1024; // 1 KiB
pub const MAX_OUTPUT_CAPTURE_BYTES_HARD_CAP: usize = 64 * 1024 * 1024; // 64 MiB

pub const DEFAULT_TIMEOUT_SECONDS: u64 = 300;
pub const MIN_TIMEOUT_SECONDS: u64 = 1;
pub const MAX_TIMEOUT_SECONDS: u64 = 86400;

pub const DEFAULT_MAX_PROFILES: usize = 256;
pub const MIN_MAX_PROFILES: usize = 1;
pub const MAX_MAX_PROFILES: usize = 1024;

pub const MAX_CONFIG_FILE_BYTES: u64 = 64 * 1024; // 64 KiB

pub const SANDBOXCONF_ERR_IO: &str = "SANDBOXCONF_ERR_IO";
pub const SANDBOXCONF_ERR_PARSE: &str = "SANDBOXCONF_ERR_PARSE";
pub const SANDBOXCONF_ERR_VALIDATION: &str = "SANDBOXCONF_ERR_VALIDATION";
pub const SANDBOXCONF_ERR_BOUNDS: &str = "SANDBOXCONF_ERR_BOUNDS";

/// Configuration settings for Sandbox Enforcement runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub version: String,
    pub default_profile_name: String,
    pub max_output_capture_bytes: usize,
    pub execution_timeout_seconds: u64,
    pub max_registered_profiles: usize,
    pub enforce_pep_grants: bool,
    pub audit_enabled: bool,
    pub custom_profiles_dir: Option<PathBuf>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            default_profile_name: "standard".into(),
            max_output_capture_bytes: DEFAULT_MAX_OUTPUT_CAPTURE_BYTES,
            execution_timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
            max_registered_profiles: DEFAULT_MAX_PROFILES,
            enforce_pep_grants: false,
            audit_enabled: true,
            custom_profiles_dir: None,
        }
    }
}

impl SandboxConfig {
    /// Validates the configuration bounds and fields.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.is_empty() || !self.version.starts_with("1.") {
            return Err(format!("{}: invalid version '{}'", SANDBOXCONF_ERR_VALIDATION, self.version));
        }
        if self.default_profile_name.trim().is_empty() {
            return Err(format!("{}: default_profile_name cannot be empty", SANDBOXCONF_ERR_VALIDATION));
        }
        if self.max_output_capture_bytes < MIN_OUTPUT_CAPTURE_BYTES || self.max_output_capture_bytes > MAX_OUTPUT_CAPTURE_BYTES_HARD_CAP {
            return Err(format!(
                "{}: max_output_capture_bytes {} out of bounds ({}..{})",
                SANDBOXCONF_ERR_BOUNDS, self.max_output_capture_bytes, MIN_OUTPUT_CAPTURE_BYTES, MAX_OUTPUT_CAPTURE_BYTES_HARD_CAP
            ));
        }
        if self.execution_timeout_seconds < MIN_TIMEOUT_SECONDS || self.execution_timeout_seconds > MAX_TIMEOUT_SECONDS {
            return Err(format!(
                "{}: execution_timeout_seconds {} out of bounds ({}..{})",
                SANDBOXCONF_ERR_BOUNDS, self.execution_timeout_seconds, MIN_TIMEOUT_SECONDS, MAX_TIMEOUT_SECONDS
            ));
        }
        if self.max_registered_profiles < MIN_MAX_PROFILES || self.max_registered_profiles > MAX_MAX_PROFILES {
            return Err(format!(
                "{}: max_registered_profiles {} out of bounds ({}..{})",
                SANDBOXCONF_ERR_BOUNDS, self.max_registered_profiles, MIN_MAX_PROFILES, MAX_MAX_PROFILES
            ));
        }
        if let Some(ref dir) = self.custom_profiles_dir {
            if dir.to_string_lossy().contains("..") {
                return Err(format!("{}: custom_profiles_dir cannot contain traversal '..'", SANDBOXCONF_ERR_VALIDATION));
            }
        }
        Ok(())
    }

    /// Loads configuration from a JSON file path with bounds checking.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let meta = std::fs::metadata(p).map_err(|e| format!("{}: cannot stat {:?}: {}", SANDBOXCONF_ERR_IO, p, e))?;
        if meta.len() > MAX_CONFIG_FILE_BYTES {
            return Err(format!("{}: config file size {} exceeds max {}", SANDBOXCONF_ERR_BOUNDS, meta.len(), MAX_CONFIG_FILE_BYTES));
        }
        let data = std::fs::read_to_string(p).map_err(|e| format!("{}: cannot read {:?}: {}", SANDBOXCONF_ERR_IO, p, e))?;
        let cfg: Self = serde_json::from_str(&data).map_err(|e| format!("{}: JSON parse error: {}", SANDBOXCONF_ERR_PARSE, e))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Saves configuration to a JSON file path after validation.
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.validate()?;
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("{}: serialize error: {}", SANDBOXCONF_ERR_PARSE, e))?;
        std::fs::write(p, json).map_err(|e| format!("{}: cannot write {:?}: {}", SANDBOXCONF_ERR_IO, p, e))?;
        Ok(())
    }

    /// Builds a configuration with environment variable overrides.
    pub fn load_with_env_overrides() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("AIOS_SANDBOX_DEFAULT_PROFILE") {
            if !v.trim().is_empty() {
                cfg.default_profile_name = v;
            }
        }
        if let Ok(v) = std::env::var("AIOS_SANDBOX_ENFORCE_PEP") {
            cfg.enforce_pep_grants = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = std::env::var("AIOS_SANDBOX_MAX_OUTPUT_BYTES") {
            if let Ok(n) = v.parse::<usize>() {
                if n >= MIN_OUTPUT_CAPTURE_BYTES && n <= MAX_OUTPUT_CAPTURE_BYTES_HARD_CAP {
                    cfg.max_output_capture_bytes = n;
                }
            }
        }
        if let Ok(v) = std::env::var("AIOS_SANDBOX_TIMEOUT_SECS") {
            if let Ok(n) = v.parse::<u64>() {
                if n >= MIN_TIMEOUT_SECONDS && n <= MAX_TIMEOUT_SECONDS {
                    cfg.execution_timeout_seconds = n;
                }
            }
        }
        if let Ok(v) = std::env::var("AIOS_SANDBOX_PROFILES_DIR") {
            if !v.trim().is_empty() {
                cfg.custom_profiles_dir = Some(PathBuf::from(v));
            }
        }
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_sandbox_config_defaults() {
        let cfg = SandboxConfig::default();
        assert_eq!(cfg.version, "1.0.0");
        assert_eq!(cfg.default_profile_name, "standard");
        assert_eq!(cfg.max_output_capture_bytes, DEFAULT_MAX_OUTPUT_CAPTURE_BYTES);
        assert_eq!(cfg.execution_timeout_seconds, 300);
        assert_eq!(cfg.max_registered_profiles, 256);
        assert!(!cfg.enforce_pep_grants);
        assert!(cfg.audit_enabled);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_sandbox_config_bounds_rejection() {
        let mut cfg = SandboxConfig::default();
        cfg.max_output_capture_bytes = 100; // Below MIN (1024)
        assert!(cfg.validate().is_err());

        cfg = SandboxConfig::default();
        cfg.execution_timeout_seconds = 0; // Below MIN (1)
        assert!(cfg.validate().is_err());

        cfg = SandboxConfig::default();
        cfg.max_registered_profiles = 2000; // Above MAX (1024)
        assert!(cfg.validate().is_err());

        cfg = SandboxConfig::default();
        cfg.custom_profiles_dir = Some(PathBuf::from("../escape"));
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_sandbox_config_file_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("sandbox.json");

        let mut cfg = SandboxConfig::default();
        cfg.enforce_pep_grants = true;
        cfg.execution_timeout_seconds = 600;

        assert!(cfg.save_to_path(&path).is_ok());
        let loaded = SandboxConfig::load_from_path(&path).unwrap();
        assert_eq!(loaded, cfg);
    }
}
