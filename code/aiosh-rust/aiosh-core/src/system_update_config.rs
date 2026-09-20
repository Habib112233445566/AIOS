//! System Update Configuration Subsystem (UCONF1..UCONF6)
//!
//! Provides configuration management, validation, environment variable ingestion,
//! and persistent serialization for system update operations.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::system_update::{UpdateChannel, MAX_UPDATE_PAYLOAD_SIZE};
use crate::system_update_service::SystemUpdateServiceConfig;

/// Maximum configuration file size allowed to load (1 MB).
pub const MAX_UPDATE_CONFIG_FILE_BYTES: u64 = 1_048_576;

/// Default store path for system update configuration.
pub const DEFAULT_UPDATE_CONFIG_PATH: &str = ".aios/system_update.json";
/// Default state directory for system update status files.
pub const DEFAULT_UPDATE_STATE_DIR: &str = "/var/lib/aiosh/updates";
/// Default staging directory for system update payloads.
pub const DEFAULT_UPDATE_STAGING_DIR: &str = "/var/lib/aiosh/updates/staging";

pub const UCONF_VALIDATION_ERROR: &str = "UCONF_VALIDATION_ERROR";

/// Configuration for System Update operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemUpdateConfig {
    /// Directory where slot_status.json and update_status.json are persisted.
    pub state_dir: PathBuf,
    /// Directory where incoming payload artifacts are staged.
    pub staging_dir: PathBuf,
    /// Default update channel for release discovery.
    pub default_channel: UpdateChannel,
    /// Polling / check interval in seconds.
    pub check_interval_secs: u64,
    /// Whether updates should be automatically applied without manual confirmation.
    pub allow_auto_apply: bool,
    /// Whether the system automatically triggers rollback on boot failure.
    pub auto_rollback_on_failure: bool,
    /// Maximum allowed payload size in bytes.
    pub max_payload_bytes: u64,
    /// Minimum free disk space in bytes required before staging payloads.
    pub min_free_space_bytes: u64,
    /// Optional download bandwidth limit in bytes per second.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_download_rate_bps: Option<u64>,
    /// List of trusted public key digests or identifiers for signature validation.
    pub trusted_keys: Vec<String>,
}

impl Default for SystemUpdateConfig {
    fn default() -> Self {
        Self {
            state_dir: PathBuf::from(DEFAULT_UPDATE_STATE_DIR),
            staging_dir: PathBuf::from(DEFAULT_UPDATE_STAGING_DIR),
            default_channel: UpdateChannel::Stable,
            check_interval_secs: 86400,
            allow_auto_apply: false,
            auto_rollback_on_failure: true,
            max_payload_bytes: MAX_UPDATE_PAYLOAD_SIZE,
            min_free_space_bytes: 1_073_741_824, // 1 GB
            max_download_rate_bps: None,
            trusted_keys: Vec::new(),
        }
    }
}

impl SystemUpdateConfig {
    /// Validates configuration against invariants UCONF1..UCONF6.
    pub fn validate(&self) -> Result<(), String> {
        // UCONF1: Path hygiene
        for (name, path) in [
            ("state_dir", &self.state_dir),
            ("staging_dir", &self.staging_dir),
        ] {
            let s = path
                .to_str()
                .ok_or_else(|| format!("{}: {} must be valid UTF-8", UCONF_VALIDATION_ERROR, name))?;
            if s.trim().is_empty() {
                return Err(format!("{}: {} cannot be empty", UCONF_VALIDATION_ERROR, name));
            }
            if s.len() > 1024 {
                return Err(format!(
                    "{}: {} exceeds maximum length of 1024 characters",
                    UCONF_VALIDATION_ERROR, name
                ));
            }
            if s.chars().any(|c| c.is_control() || c == '\0') {
                return Err(format!(
                    "{}: {} cannot contain control characters",
                    UCONF_VALIDATION_ERROR, name
                ));
            }
            if path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(format!(
                    "{}: {} cannot contain parent directory traversal ('..')",
                    UCONF_VALIDATION_ERROR, name
                ));
            }
        }

        // UCONF2: Bounds
        if self.check_interval_secs < 60 || self.check_interval_secs > 2_592_000 {
            return Err(format!(
                "{}: check_interval_secs must be between 60 and 2592000 seconds",
                UCONF_VALIDATION_ERROR
            ));
        }
        if self.max_payload_bytes < 1_048_576 || self.max_payload_bytes > 10_737_418_240 {
            return Err(format!(
                "{}: max_payload_bytes must be between 1MB and 10GB",
                UCONF_VALIDATION_ERROR
            ));
        }
        if self.min_free_space_bytes > 107_374_182_400 {
            return Err(format!(
                "{}: min_free_space_bytes exceeds 100GB maximum threshold",
                UCONF_VALIDATION_ERROR
            ));
        }

        // UCONF3: Trusted keys bounds
        if self.trusted_keys.len() > 32 {
            return Err(format!(
                "{}: trusted_keys cannot exceed 32 entries",
                UCONF_VALIDATION_ERROR
            ));
        }
        for key in &self.trusted_keys {
            if key.trim().is_empty() || key.len() > 256 || key.chars().any(|c| c.is_control()) {
                return Err(format!(
                    "{}: trusted key must be non-empty, <= 256 chars, and without control characters",
                    UCONF_VALIDATION_ERROR
                ));
            }
        }

        Ok(())
    }

    /// Ingests configuration overrides from environment variables (UCONF4).
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(val) = std::env::var("AIOSH_UPDATE_STATE_DIR") {
            if !val.trim().is_empty() {
                cfg.state_dir = PathBuf::from(val);
            }
        }
        if let Ok(val) = std::env::var("AIOSH_UPDATE_STAGING_DIR") {
            if !val.trim().is_empty() {
                cfg.staging_dir = PathBuf::from(val);
            }
        }
        if let Ok(val) = std::env::var("AIOSH_UPDATE_CHANNEL") {
            match val.to_lowercase().as_str() {
                "stable" => cfg.default_channel = UpdateChannel::Stable,
                "beta" => cfg.default_channel = UpdateChannel::Beta,
                "nightly" => cfg.default_channel = UpdateChannel::Nightly,
                _ => {}
            }
        }
        if let Ok(val) = std::env::var("AIOSH_UPDATE_CHECK_INTERVAL_SECS") {
            if let Ok(secs) = val.parse::<u64>() {
                cfg.check_interval_secs = secs;
            }
        }
        if let Ok(val) = std::env::var("AIOSH_UPDATE_AUTO_APPLY") {
            match val.to_lowercase().as_str() {
                "true" | "1" | "yes" => cfg.allow_auto_apply = true,
                "false" | "0" | "no" => cfg.allow_auto_apply = false,
                _ => {}
            }
        }
        if let Ok(val) = std::env::var("AIOSH_UPDATE_AUTO_ROLLBACK") {
            match val.to_lowercase().as_str() {
                "true" | "1" | "yes" => cfg.auto_rollback_on_failure = true,
                "false" | "0" | "no" => cfg.auto_rollback_on_failure = false,
                _ => {}
            }
        }
        if let Ok(val) = std::env::var("AIOSH_UPDATE_MAX_PAYLOAD_BYTES") {
            if let Ok(bytes) = val.parse::<u64>() {
                cfg.max_payload_bytes = bytes;
            }
        }
        cfg
    }

    /// Loads and validates configuration from JSON file (UCONF5, UCONF6).
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let meta = fs::symlink_metadata(p)
            .map_err(|e| format!("{}: cannot stat config file: {}", UCONF_VALIDATION_ERROR, e))?;
        if meta.file_type().is_symlink() {
            return Err(format!(
                "{}: config file cannot be a symlink",
                UCONF_VALIDATION_ERROR
            ));
        }
        if meta.len() > MAX_UPDATE_CONFIG_FILE_BYTES {
            return Err(format!(
                "{}: config file size ({} bytes) exceeds limit of {} bytes",
                UCONF_VALIDATION_ERROR,
                meta.len(),
                MAX_UPDATE_CONFIG_FILE_BYTES
            ));
        }
        let bytes = fs::read(p)
            .map_err(|e| format!("{}: failed to read config file: {}", UCONF_VALIDATION_ERROR, e))?;
        let cfg: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("{}: invalid config JSON: {}", UCONF_VALIDATION_ERROR, e))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Persists configuration to file atomically (UCONF5).
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.validate()?;
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("{}: failed to create parent directory: {}", UCONF_VALIDATION_ERROR, e))?;
        }
        let tmp = p.with_extension(format!("tmp.{}", std::process::id()));
        let data = serde_json::to_vec_pretty(self)
            .map_err(|e| format!("{}: serialization failed: {}", UCONF_VALIDATION_ERROR, e))?;
        fs::write(&tmp, data)
            .map_err(|e| format!("{}: failed to write temporary file: {}", UCONF_VALIDATION_ERROR, e))?;
        fs::rename(&tmp, p)
            .map_err(|e| {
                let _ = fs::remove_file(&tmp);
                format!("{}: failed to replace config file: {}", UCONF_VALIDATION_ERROR, e)
            })?;
        Ok(())
    }

    /// Converts into `SystemUpdateServiceConfig` for service initialization.
    pub fn to_service_config(&self) -> SystemUpdateServiceConfig {
        SystemUpdateServiceConfig {
            state_dir: self.state_dir.clone(),
            staging_dir: self.staging_dir.clone(),
            max_payload_bytes: self.max_payload_bytes,
            auto_rollback_on_failure: self.auto_rollback_on_failure,
        }
    }
}
