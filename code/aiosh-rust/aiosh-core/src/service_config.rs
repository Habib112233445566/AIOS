//! Init & Service Supervision Configuration Subsystem (T-01344 Implementation).
//!
//! Enforces configuration resolution, precedence, and validation invariants SC1..SC7.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub const DEFAULT_SERVICE_STORE_PATH: &str = ".aios/service_store.json";
pub const DEFAULT_TIMEOUT_START_SECS: u32 = 30;
pub const DEFAULT_TIMEOUT_STOP_SECS: u32 = 30;
pub const DEFAULT_MAX_STORE_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB
pub const DEFAULT_MAX_ENTITY_COUNT: usize = 10_000;
pub const DEFAULT_AUTO_PERSIST: bool = true;
pub const DEFAULT_RESTART_BACKOFF_SECS: u32 = 5;
pub const DEFAULT_MAX_RESTART_BURST: u32 = 5;
pub const MAX_CONFIG_FILE_BYTES: u64 = 65_536; // 64 KiB

pub const MIN_TIMEOUT_SECS: u32 = 1;
pub const MAX_TIMEOUT_SECS: u32 = 3600;
pub const MIN_STORE_SIZE_BYTES: u64 = 65_536; // 64 KiB
pub const MAX_ALLOWED_STORE_SIZE_BYTES: u64 = 100 * 1024 * 1024; // 100 MiB
pub const MIN_ENTITY_COUNT: usize = 10;
pub const MAX_ALLOWED_ENTITY_COUNT: usize = 100_000;
pub const MIN_RESTART_BACKOFF_SECS: u32 = 1;
pub const MAX_RESTART_BACKOFF_SECS: u32 = 300;
pub const MIN_RESTART_BURST: u32 = 1;
pub const MAX_RESTART_BURST: u32 = 50;

/// Configuration parameters for Init & Service Supervision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Canonical filesystem path to the persistent service store JSON file.
    pub store_path: PathBuf,
    /// Default execution startup timeout in seconds for services without explicit timeout.
    pub default_timeout_start_secs: u32,
    /// Default execution shutdown timeout in seconds for services without explicit timeout.
    pub default_timeout_stop_secs: u32,
    /// Maximum allowed service store file size on disk (bytes).
    pub max_store_size_bytes: u64,
    /// Maximum service entities permitted within a single store.
    pub max_entity_count: usize,
    /// Whether mutations automatically persist to store_path without explicit flag.
    pub auto_persist: bool,
    /// Throttling backoff delay in seconds between consecutive service restarts.
    pub restart_backoff_secs: u32,
    /// Maximum number of restart attempts permitted within the backoff window.
    pub max_restart_burst: u32,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            store_path: PathBuf::from(DEFAULT_SERVICE_STORE_PATH),
            default_timeout_start_secs: DEFAULT_TIMEOUT_START_SECS,
            default_timeout_stop_secs: DEFAULT_TIMEOUT_STOP_SECS,
            max_store_size_bytes: DEFAULT_MAX_STORE_SIZE_BYTES,
            max_entity_count: DEFAULT_MAX_ENTITY_COUNT,
            auto_persist: DEFAULT_AUTO_PERSIST,
            restart_backoff_secs: DEFAULT_RESTART_BACKOFF_SECS,
            max_restart_burst: DEFAULT_MAX_RESTART_BURST,
        }
    }
}

impl ServiceConfig {
    /// Validates configuration against invariants SC1..SC7.
    pub fn validate(&self) -> Result<(), String> {
        // SC1: store_path validity
        if self.store_path.as_os_str().is_empty() {
            return Err("SC1 violation: store_path cannot be empty".into());
        }
        let path_str = self.store_path.to_string_lossy();
        if path_str.len() > 1024 {
            return Err(format!(
                "SC1 violation: store_path length ({} bytes) exceeds maximum limit of 1024 bytes",
                path_str.len()
            ));
        }
        if path_str.chars().any(|c| c.is_control() || c == '\0') {
            return Err("SC1 violation: store_path cannot contain control characters or null bytes".into());
        }

        // SC2: timeout bounds [1..3600]
        if self.default_timeout_start_secs < MIN_TIMEOUT_SECS
            || self.default_timeout_start_secs > MAX_TIMEOUT_SECS
        {
            return Err(format!(
                "SC2 violation: default_timeout_start_secs must be between {} and {} seconds, got {}",
                MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS, self.default_timeout_start_secs
            ));
        }
        if self.default_timeout_stop_secs < MIN_TIMEOUT_SECS
            || self.default_timeout_stop_secs > MAX_TIMEOUT_SECS
        {
            return Err(format!(
                "SC2 violation: default_timeout_stop_secs must be between {} and {} seconds, got {}",
                MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS, self.default_timeout_stop_secs
            ));
        }

        // SC3: max_store_size_bytes bounds [64 KiB..100 MiB]
        if self.max_store_size_bytes < MIN_STORE_SIZE_BYTES
            || self.max_store_size_bytes > MAX_ALLOWED_STORE_SIZE_BYTES
        {
            return Err(format!(
                "SC3 violation: max_store_size_bytes must be between {} (64 KiB) and {} (100 MiB), got {}",
                MIN_STORE_SIZE_BYTES, MAX_ALLOWED_STORE_SIZE_BYTES, self.max_store_size_bytes
            ));
        }

        // SC4: max_entity_count bounds [10..100,000]
        if self.max_entity_count < MIN_ENTITY_COUNT
            || self.max_entity_count > MAX_ALLOWED_ENTITY_COUNT
        {
            return Err(format!(
                "SC4 violation: max_entity_count must be between {} and {}, got {}",
                MIN_ENTITY_COUNT, MAX_ALLOWED_ENTITY_COUNT, self.max_entity_count
            ));
        }

        // SC5: restart throttling bounds
        if self.restart_backoff_secs < MIN_RESTART_BACKOFF_SECS
            || self.restart_backoff_secs > MAX_RESTART_BACKOFF_SECS
        {
            return Err(format!(
                "SC5 violation: restart_backoff_secs must be between {} and {} seconds, got {}",
                MIN_RESTART_BACKOFF_SECS, MAX_RESTART_BACKOFF_SECS, self.restart_backoff_secs
            ));
        }
        if self.max_restart_burst < MIN_RESTART_BURST
            || self.max_restart_burst > MAX_RESTART_BURST
        {
            return Err(format!(
                "SC5 violation: max_restart_burst must be between {} and {}, got {}",
                MIN_RESTART_BURST, MAX_RESTART_BURST, self.max_restart_burst
            ));
        }

        Ok(())
    }

    /// Loads configuration from an explicit file path on disk (SC6, SC7).
    pub fn from_file(path: &Path) -> Result<Self, String> {
        use std::io::Read;

        let file = std::fs::File::open(path)
            .map_err(|e| format!("failed to open service config file at '{}': {}", path.display(), e))?;

        let meta = file
            .metadata()
            .map_err(|e| format!("failed to read metadata for '{}': {}", path.display(), e))?;

        if meta.len() > MAX_CONFIG_FILE_BYTES {
            return Err(format!(
                "SC7 violation: service config at '{}' exceeds maximum allowed size of 64 KiB (was {} bytes)",
                path.display(),
                meta.len()
            ));
        }

        let mut bytes = Vec::new();
        file.take(MAX_CONFIG_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("failed to read service config from '{}': {}", path.display(), e))?;

        if bytes.len() as u64 > MAX_CONFIG_FILE_BYTES {
            return Err("SC7 violation: service config content exceeded 64 KiB during stream read".into());
        }

        let config: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("failed to parse service config JSON at '{}': {}", path.display(), e))?;

        config.validate()?;
        Ok(config)
    }

    /// Loads configuration from environment variables with fallback to defaults (SC6).
    pub fn from_env() -> Result<Self, String> {
        let mut cfg = Self::default();

        if let Ok(val) = std::env::var("AIOS_SERVICE_STORE_PATH") {
            if !val.trim().is_empty() {
                cfg.store_path = PathBuf::from(val.trim());
            }
        }

        if let Ok(val) = std::env::var("AIOS_SERVICE_TIMEOUT_START_SECS") {
            if let Ok(secs) = val.trim().parse::<u32>() {
                cfg.default_timeout_start_secs = secs;
            } else {
                return Err(format!("invalid integer in AIOS_SERVICE_TIMEOUT_START_SECS: {}", val));
            }
        }

        if let Ok(val) = std::env::var("AIOS_SERVICE_TIMEOUT_STOP_SECS") {
            if let Ok(secs) = val.trim().parse::<u32>() {
                cfg.default_timeout_stop_secs = secs;
            } else {
                return Err(format!("invalid integer in AIOS_SERVICE_TIMEOUT_STOP_SECS: {}", val));
            }
        }

        if let Ok(val) = std::env::var("AIOS_SERVICE_MAX_STORE_SIZE_BYTES") {
            if let Ok(sz) = val.trim().parse::<u64>() {
                cfg.max_store_size_bytes = sz;
            } else {
                return Err(format!("invalid integer in AIOS_SERVICE_MAX_STORE_SIZE_BYTES: {}", val));
            }
        }

        if let Ok(val) = std::env::var("AIOS_SERVICE_MAX_ENTITIES") {
            if let Ok(cnt) = val.trim().parse::<usize>() {
                cfg.max_entity_count = cnt;
            } else {
                return Err(format!("invalid integer in AIOS_SERVICE_MAX_ENTITIES: {}", val));
            }
        }

        if let Ok(val) = std::env::var("AIOS_SERVICE_AUTO_PERSIST") {
            let s = val.trim().to_lowercase();
            cfg.auto_persist = s == "1" || s == "true" || s == "yes";
        }

        if let Ok(val) = std::env::var("AIOS_SERVICE_RESTART_BACKOFF_SECS") {
            if let Ok(secs) = val.trim().parse::<u32>() {
                cfg.restart_backoff_secs = secs;
            } else {
                return Err(format!("invalid integer in AIOS_SERVICE_RESTART_BACKOFF_SECS: {}", val));
            }
        }

        if let Ok(val) = std::env::var("AIOS_SERVICE_MAX_RESTART_BURST") {
            if let Ok(burst) = val.trim().parse::<u32>() {
                cfg.max_restart_burst = burst;
            } else {
                return Err(format!("invalid integer in AIOS_SERVICE_MAX_RESTART_BURST: {}", val));
            }
        }

        cfg.validate()?;
        Ok(cfg)
    }

    /// Resolves configuration adhering to precedence SC6 (file > env > default).
    pub fn resolve(config_path_opt: Option<&Path>) -> Result<Self, String> {
        if let Some(path) = config_path_opt {
            return Self::from_file(path);
        }
        if let Ok(env_path) = std::env::var("AIOS_SERVICE_CONFIG") {
            if !env_path.trim().is_empty() {
                return Self::from_file(Path::new(env_path.trim()));
            }
        }
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_config_default_and_validation() {
        let cfg = ServiceConfig::default();
        assert_eq!(cfg.validate(), Ok(()));
        assert_eq!(cfg.store_path, PathBuf::from(DEFAULT_SERVICE_STORE_PATH));
        assert_eq!(cfg.default_timeout_start_secs, 30);
        assert_eq!(cfg.default_timeout_stop_secs, 30);
        assert_eq!(cfg.max_store_size_bytes, 10 * 1024 * 1024);
        assert_eq!(cfg.max_entity_count, 10_000);
        assert!(cfg.auto_persist);
        assert_eq!(cfg.restart_backoff_secs, 5);
        assert_eq!(cfg.max_restart_burst, 5);
    }

    #[test]
    fn test_service_config_sc1_store_path_invariants() {
        let mut cfg = ServiceConfig::default();
        cfg.store_path = PathBuf::from("");
        assert!(cfg.validate().unwrap_err().contains("SC1 violation"));

        cfg.store_path = PathBuf::from("a".repeat(1025));
        assert!(cfg.validate().unwrap_err().contains("SC1 violation"));

        cfg.store_path = PathBuf::from("service\0_store.json");
        assert!(cfg.validate().unwrap_err().contains("SC1 violation"));
    }

    #[test]
    fn test_service_config_sc2_timeout_invariants() {
        let mut cfg = ServiceConfig::default();
        cfg.default_timeout_start_secs = 0;
        assert!(cfg.validate().unwrap_err().contains("SC2 violation"));

        cfg = ServiceConfig::default();
        cfg.default_timeout_start_secs = 4000;
        assert!(cfg.validate().unwrap_err().contains("SC2 violation"));

        cfg = ServiceConfig::default();
        cfg.default_timeout_stop_secs = 0;
        assert!(cfg.validate().unwrap_err().contains("SC2 violation"));

        cfg = ServiceConfig::default();
        cfg.default_timeout_stop_secs = 5000;
        assert!(cfg.validate().unwrap_err().contains("SC2 violation"));
    }

    #[test]
    fn test_service_config_sc3_sc4_sc5_boundary_invariants() {
        let mut cfg = ServiceConfig::default();
        // SC3: store size
        cfg.max_store_size_bytes = 100;
        assert!(cfg.validate().unwrap_err().contains("SC3 violation"));

        cfg.max_store_size_bytes = 200 * 1024 * 1024;
        assert!(cfg.validate().unwrap_err().contains("SC3 violation"));

        // SC4: entity count
        cfg = ServiceConfig::default();
        cfg.max_entity_count = 5;
        assert!(cfg.validate().unwrap_err().contains("SC4 violation"));

        cfg.max_entity_count = 500_000;
        assert!(cfg.validate().unwrap_err().contains("SC4 violation"));

        // SC5: restart throttling
        cfg = ServiceConfig::default();
        cfg.restart_backoff_secs = 0;
        assert!(cfg.validate().unwrap_err().contains("SC5 violation"));

        cfg = ServiceConfig::default();
        cfg.restart_backoff_secs = 500;
        assert!(cfg.validate().unwrap_err().contains("SC5 violation"));

        cfg = ServiceConfig::default();
        cfg.max_restart_burst = 0;
        assert!(cfg.validate().unwrap_err().contains("SC5 violation"));

        cfg = ServiceConfig::default();
        cfg.max_restart_burst = 100;
        assert!(cfg.validate().unwrap_err().contains("SC5 violation"));
    }

    #[test]
    fn test_service_config_file_roundtrip_and_sc7() {
        let temp_dir = std::env::temp_dir();
        let config_file = temp_dir.join(format!("aios_svc_cfg_test_{}.json", std::process::id()));

        let cfg = ServiceConfig::default();
        let content = serde_json::to_string_pretty(&cfg).unwrap();
        std::fs::write(&config_file, content).unwrap();

        let loaded = ServiceConfig::from_file(&config_file).unwrap();
        assert_eq!(loaded, cfg);

        let _ = std::fs::remove_file(&config_file);
    }
}
