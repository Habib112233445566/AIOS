//! User Session Bootstrap Configuration Subsystem (T-01444 Implementation).
//!
//! Enforces configuration resolution, precedence, and validation invariants SC1..SC7.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub const DEFAULT_SESSION_STORE_PATH: &str = ".aios/session_store.json";
pub const DEFAULT_MAX_SESSIONS_PER_USER: usize = 32;
pub const DEFAULT_MAX_TOTAL_SESSIONS: usize = 1024;
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 900; // 15 minutes
pub const DEFAULT_MAX_STORE_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB
pub const DEFAULT_AUTO_PERSIST: bool = true;
pub const MAX_CONFIG_FILE_BYTES: u64 = 65_536; // 64 KiB

pub const MIN_SESSIONS_PER_USER: usize = 1;
pub const MAX_ALLOWED_SESSIONS_PER_USER: usize = 128;
pub const MIN_TOTAL_SESSIONS: usize = 10;
pub const MAX_ALLOWED_TOTAL_SESSIONS: usize = 10_000;
pub const MIN_IDLE_TIMEOUT_SECS: u64 = 10;
pub const MAX_IDLE_TIMEOUT_SECS: u64 = 86_400; // 24 hours
pub const MIN_STORE_SIZE_BYTES: u64 = 65_536; // 64 KiB
pub const MAX_ALLOWED_STORE_SIZE_BYTES: u64 = 100 * 1024 * 1024; // 100 MiB

/// Configuration parameters for User Session Bootstrap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Canonical filesystem path to the persistent session store JSON file.
    pub store_path: PathBuf,
    /// Maximum concurrent active sessions per individual user account.
    pub max_sessions_per_user: usize,
    /// Maximum total sessions permitted in store across all users.
    pub max_total_sessions: usize,
    /// Default inactivity threshold in seconds before session auto-lock.
    pub default_idle_timeout_seconds: u64,
    /// Maximum allowed session store file size on disk (bytes).
    pub max_store_size_bytes: u64,
    /// Whether mutations automatically persist to disk.
    pub auto_persist: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            store_path: PathBuf::from(DEFAULT_SESSION_STORE_PATH),
            max_sessions_per_user: DEFAULT_MAX_SESSIONS_PER_USER,
            max_total_sessions: DEFAULT_MAX_TOTAL_SESSIONS,
            default_idle_timeout_seconds: DEFAULT_IDLE_TIMEOUT_SECS,
            max_store_size_bytes: DEFAULT_MAX_STORE_SIZE_BYTES,
            auto_persist: DEFAULT_AUTO_PERSIST,
        }
    }
}

impl SessionConfig {
    /// Validates configuration against invariants SC1..SC5.
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

        // SC2: max_sessions_per_user bounds [1..128]
        if self.max_sessions_per_user < MIN_SESSIONS_PER_USER
            || self.max_sessions_per_user > MAX_ALLOWED_SESSIONS_PER_USER
        {
            return Err(format!(
                "SC2 violation: max_sessions_per_user must be between {} and {}, got {}",
                MIN_SESSIONS_PER_USER, MAX_ALLOWED_SESSIONS_PER_USER, self.max_sessions_per_user
            ));
        }

        // SC3: max_total_sessions bounds [10..10,000]
        if self.max_total_sessions < MIN_TOTAL_SESSIONS
            || self.max_total_sessions > MAX_ALLOWED_TOTAL_SESSIONS
        {
            return Err(format!(
                "SC3 violation: max_total_sessions must be between {} and {}, got {}",
                MIN_TOTAL_SESSIONS, MAX_ALLOWED_TOTAL_SESSIONS, self.max_total_sessions
            ));
        }

        // SC4: default_idle_timeout_seconds bounds [10..86,400]
        if self.default_idle_timeout_seconds < MIN_IDLE_TIMEOUT_SECS
            || self.default_idle_timeout_seconds > MAX_IDLE_TIMEOUT_SECS
        {
            return Err(format!(
                "SC4 violation: default_idle_timeout_seconds must be between {} and {} seconds, got {}",
                MIN_IDLE_TIMEOUT_SECS, MAX_IDLE_TIMEOUT_SECS, self.default_idle_timeout_seconds
            ));
        }

        // SC5: max_store_size_bytes bounds [64 KiB..100 MiB]
        if self.max_store_size_bytes < MIN_STORE_SIZE_BYTES
            || self.max_store_size_bytes > MAX_ALLOWED_STORE_SIZE_BYTES
        {
            return Err(format!(
                "SC5 violation: max_store_size_bytes must be between {} (64 KiB) and {} (100 MiB), got {}",
                MIN_STORE_SIZE_BYTES, MAX_ALLOWED_STORE_SIZE_BYTES, self.max_store_size_bytes
            ));
        }

        Ok(())
    }

    /// Loads configuration from an explicit file path on disk (SC6, SC7).
    pub fn from_file(path: &Path) -> Result<Self, String> {
        use std::io::Read;

        let file = std::fs::File::open(path)
            .map_err(|e| format!("failed to open session config file at '{}': {}", path.display(), e))?;

        let meta = file
            .metadata()
            .map_err(|e| format!("failed to read metadata for '{}': {}", path.display(), e))?;

        if meta.len() > MAX_CONFIG_FILE_BYTES {
            return Err(format!(
                "SC7 violation: session config at '{}' exceeds maximum allowed size of 64 KiB (was {} bytes)",
                path.display(),
                meta.len()
            ));
        }

        let mut bytes = Vec::new();
        file.take(MAX_CONFIG_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("failed to read session config from '{}': {}", path.display(), e))?;

        if bytes.len() as u64 > MAX_CONFIG_FILE_BYTES {
            return Err("SC7 violation: session config content exceeded 64 KiB during stream read".into());
        }

        let config: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("failed to parse session config JSON at '{}': {}", path.display(), e))?;

        config.validate()?;
        Ok(config)
    }

    /// Loads configuration from environment variables with fallback to defaults (SC6).
    pub fn from_env() -> Result<Self, String> {
        let mut cfg = Self::default();

        if let Ok(val) = std::env::var("AIOS_SESSION_STORE_PATH") {
            if !val.trim().is_empty() {
                cfg.store_path = PathBuf::from(val.trim());
            }
        }

        if let Ok(val) = std::env::var("AIOS_SESSION_MAX_PER_USER") {
            if let Ok(cnt) = val.trim().parse::<usize>() {
                cfg.max_sessions_per_user = cnt;
            } else {
                return Err(format!("invalid integer in AIOS_SESSION_MAX_PER_USER: {}", val));
            }
        }

        if let Ok(val) = std::env::var("AIOS_SESSION_MAX_TOTAL") {
            if let Ok(cnt) = val.trim().parse::<usize>() {
                cfg.max_total_sessions = cnt;
            } else {
                return Err(format!("invalid integer in AIOS_SESSION_MAX_TOTAL: {}", val));
            }
        }

        if let Ok(val) = std::env::var("AIOS_SESSION_IDLE_TIMEOUT_SECS") {
            if let Ok(secs) = val.trim().parse::<u64>() {
                cfg.default_idle_timeout_seconds = secs;
            } else {
                return Err(format!("invalid integer in AIOS_SESSION_IDLE_TIMEOUT_SECS: {}", val));
            }
        }

        if let Ok(val) = std::env::var("AIOS_SESSION_MAX_STORE_SIZE_BYTES") {
            if let Ok(sz) = val.trim().parse::<u64>() {
                cfg.max_store_size_bytes = sz;
            } else {
                return Err(format!("invalid integer in AIOS_SESSION_MAX_STORE_SIZE_BYTES: {}", val));
            }
        }

        if let Ok(val) = std::env::var("AIOS_SESSION_AUTO_PERSIST") {
            let s = val.trim().to_lowercase();
            cfg.auto_persist = s == "1" || s == "true" || s == "yes";
        }

        cfg.validate()?;
        Ok(cfg)
    }

    /// Resolves configuration adhering to precedence SC6 (file > env > default).
    pub fn resolve(config_path_opt: Option<&Path>) -> Result<Self, String> {
        if let Some(path) = config_path_opt {
            return Self::from_file(path);
        }
        if let Ok(env_path) = std::env::var("AIOS_SESSION_CONFIG") {
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
    fn test_session_config_default_and_validation() {
        let cfg = SessionConfig::default();
        assert_eq!(cfg.validate(), Ok(()));
        assert_eq!(cfg.store_path, PathBuf::from(DEFAULT_SESSION_STORE_PATH));
        assert_eq!(cfg.max_sessions_per_user, 32);
        assert_eq!(cfg.max_total_sessions, 1024);
        assert_eq!(cfg.default_idle_timeout_seconds, 900);
        assert_eq!(cfg.max_store_size_bytes, 10 * 1024 * 1024);
        assert!(cfg.auto_persist);
    }

    #[test]
    fn test_session_config_sc1_store_path_invariants() {
        let mut cfg = SessionConfig::default();
        cfg.store_path = PathBuf::from("");
        assert!(cfg.validate().unwrap_err().contains("SC1 violation"));

        cfg.store_path = PathBuf::from("a".repeat(1025));
        assert!(cfg.validate().unwrap_err().contains("SC1 violation"));

        cfg.store_path = PathBuf::from("session\0_store.json");
        assert!(cfg.validate().unwrap_err().contains("SC1 violation"));
    }

    #[test]
    fn test_session_config_sc2_user_capacity_invariants() {
        let mut cfg = SessionConfig::default();
        cfg.max_sessions_per_user = 0;
        assert!(cfg.validate().unwrap_err().contains("SC2 violation"));

        cfg = SessionConfig::default();
        cfg.max_sessions_per_user = 200;
        assert!(cfg.validate().unwrap_err().contains("SC2 violation"));
    }

    #[test]
    fn test_session_config_sc3_total_capacity_invariants() {
        let mut cfg = SessionConfig::default();
        cfg.max_total_sessions = 5;
        assert!(cfg.validate().unwrap_err().contains("SC3 violation"));

        cfg = SessionConfig::default();
        cfg.max_total_sessions = 20_000;
        assert!(cfg.validate().unwrap_err().contains("SC3 violation"));
    }

    #[test]
    fn test_session_config_sc4_idle_timeout_invariants() {
        let mut cfg = SessionConfig::default();
        cfg.default_idle_timeout_seconds = 5;
        assert!(cfg.validate().unwrap_err().contains("SC4 violation"));

        cfg = SessionConfig::default();
        cfg.default_idle_timeout_seconds = 100_000;
        assert!(cfg.validate().unwrap_err().contains("SC4 violation"));
    }

    #[test]
    fn test_session_config_sc5_store_size_invariants() {
        let mut cfg = SessionConfig::default();
        cfg.max_store_size_bytes = 100;
        assert!(cfg.validate().unwrap_err().contains("SC5 violation"));

        cfg = SessionConfig::default();
        cfg.max_store_size_bytes = 200 * 1024 * 1024;
        assert!(cfg.validate().unwrap_err().contains("SC5 violation"));
    }

    #[test]
    fn test_session_config_file_roundtrip_and_sc7() {
        let temp_dir = std::env::temp_dir();
        let config_file = temp_dir.join(format!("aios_session_cfg_test_{}.json", std::process::id()));

        let cfg = SessionConfig::default();
        let content = serde_json::to_string_pretty(&cfg).unwrap();
        std::fs::write(&config_file, content).unwrap();

        let loaded = SessionConfig::from_file(&config_file).unwrap();
        assert_eq!(loaded, cfg);

        let _ = std::fs::remove_file(&config_file);
    }
}
