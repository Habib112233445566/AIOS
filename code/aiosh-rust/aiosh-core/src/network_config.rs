//! Network Configuration Subsystem (NCONF1..NCONF6)
//!
//! Provides configuration management, validation, environment variable ingestion,
//! and persistent serialization for network bootstrap operations.

use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Maximum configuration file size allowed to load (1 MB).
pub const MAX_CONFIG_FILE_BYTES: u64 = 1_048_576;

/// Default store path for network state JSON.
pub const DEFAULT_STORE_PATH: &str = ".aios/network_state.json";
/// Default sysfs network class path.
pub const DEFAULT_SYSFS_NET_PATH: &str = "/sys/class/net";
/// Default procfs network path.
pub const DEFAULT_PROCFS_PATH: &str = "/proc/net";
/// Default resolv.conf path.
pub const DEFAULT_RESOLV_CONF_PATH: &str = "/etc/resolv.conf";

/// Configuration for Network Bootstrap operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Default path to persisted network state JSON.
    pub default_store_path: PathBuf,
    /// Root path to sysfs net directory (defaults to /sys/class/net).
    pub sysfs_net_path: PathBuf,
    /// Root path to procfs net directory (defaults to /proc/net).
    pub procfs_path: PathBuf,
    /// Path to resolv.conf file (defaults to /etc/resolv.conf).
    pub resolv_conf_path: PathBuf,
    /// Maximum number of network interfaces allowed.
    pub max_interfaces: usize,
    /// Maximum number of routes allowed.
    pub max_routes: usize,
    /// Maximum number of DNS servers allowed.
    pub max_dns_servers: usize,
    /// Maximum payload size in bytes for network state documents.
    pub max_payload_bytes: u64,
    /// Timeout in seconds for network discovery operations.
    pub scan_timeout_secs: u64,
    /// Fallback DNS servers to use if none discovered or configured.
    pub fallback_dns_servers: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            default_store_path: PathBuf::from(DEFAULT_STORE_PATH),
            sysfs_net_path: PathBuf::from(DEFAULT_SYSFS_NET_PATH),
            procfs_path: PathBuf::from(DEFAULT_PROCFS_PATH),
            resolv_conf_path: PathBuf::from(DEFAULT_RESOLV_CONF_PATH),
            max_interfaces: 1024,
            max_routes: 4096,
            max_dns_servers: 32,
            max_payload_bytes: 10_485_760, // 10 MB
            scan_timeout_secs: 30,
            fallback_dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
        }
    }
}

impl NetworkConfig {
    /// Validates configuration against invariants NCONF1..NCONF6.
    pub fn validate(&self) -> Result<(), String> {
        // NCONF1: Path hygiene
        for (name, path) in [
            ("default_store_path", &self.default_store_path),
            ("sysfs_net_path", &self.sysfs_net_path),
            ("procfs_path", &self.procfs_path),
            ("resolv_conf_path", &self.resolv_conf_path),
        ] {
            let s = path
                .to_str()
                .ok_or_else(|| format!("NCONF1 violation: {} must be valid UTF-8", name))?;
            if s.trim().is_empty() {
                return Err(format!("NCONF1 violation: {} cannot be empty", name));
            }
            if s.len() > 1024 {
                return Err(format!(
                    "NCONF1 violation: {} exceeds maximum length of 1024 characters",
                    name
                ));
            }
            if s.chars().any(|c| c.is_control() || c == '\0') {
                return Err(format!(
                    "NCONF1 violation: {} cannot contain control characters",
                    name
                ));
            }
            if path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(format!(
                    "NCONF1 violation: {} cannot contain parent directory traversal ('..')",
                    name
                ));
            }
        }

        // NCONF2: Capacity limits
        if self.max_interfaces == 0 || self.max_interfaces > 10_000 {
            return Err(format!(
                "NCONF2 violation: max_interfaces must be between 1 and 10,000 (got {})",
                self.max_interfaces
            ));
        }
        if self.max_routes == 0 || self.max_routes > 50_000 {
            return Err(format!(
                "NCONF2 violation: max_routes must be between 1 and 50,000 (got {})",
                self.max_routes
            ));
        }
        if self.max_dns_servers == 0 || self.max_dns_servers > 64 {
            return Err(format!(
                "NCONF2 violation: max_dns_servers must be between 1 and 64 (got {})",
                self.max_dns_servers
            ));
        }

        // NCONF3: Resource & timeout bounds
        if self.max_payload_bytes < 1024 || self.max_payload_bytes > 104_857_600 {
            return Err(format!(
                "NCONF3 violation: max_payload_bytes must be between 1024 and 104,857,600 (got {})",
                self.max_payload_bytes
            ));
        }
        if self.scan_timeout_secs == 0 || self.scan_timeout_secs > 300 {
            return Err(format!(
                "NCONF3 violation: scan_timeout_secs must be between 1 and 300 (got {})",
                self.scan_timeout_secs
            ));
        }

        // NCONF4: Fallback DNS validation
        if self.fallback_dns_servers.len() > self.max_dns_servers {
            return Err(format!(
                "NCONF4 violation: fallback_dns_servers count ({}) exceeds max_dns_servers ({})",
                self.fallback_dns_servers.len(),
                self.max_dns_servers
            ));
        }
        for (idx, dns) in self.fallback_dns_servers.iter().enumerate() {
            let trimmed = dns.trim();
            if trimmed.is_empty() {
                return Err(format!(
                    "NCONF4 violation: fallback_dns_servers[{}] cannot be empty",
                    idx
                ));
            }
            if trimmed.parse::<IpAddr>().is_err() {
                return Err(format!(
                    "NCONF4 violation: fallback_dns_servers[{}] '{}' is not a valid IP address",
                    idx, trimmed
                ));
            }
        }

        Ok(())
    }

    /// Loads configuration from a JSON file, returning default if file does not exist (NCONF6).
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata for {}: {}", path.display(), e))?;
        if metadata.len() > MAX_CONFIG_FILE_BYTES {
            return Err(format!(
                "Network config file {} size {} exceeds maximum allowed ({} bytes)",
                path.display(),
                metadata.len(),
                MAX_CONFIG_FILE_BYTES
            ));
        }
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read network config from {}: {}", path.display(), e))?;
        let config: NetworkConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse network config JSON: {}", e))?;
        config.validate()?;
        Ok(config)
    }

    /// Alias for `load_from_path`.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        Self::load_from_path(path)
    }

    /// Serializes and saves configuration to a JSON file atomically (NCONF6).
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Failed to create parent directory {}: {}",
                    parent.display(),
                    e
                )
            })?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize network config: {}", e))?;

        // Atomic write via temporary sibling file + rename
        let tmp_file_name = format!(
            ".{}.tmp.{}",
            path.file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_else(|| "cfg".into()),
            std::process::id()
        );
        let tmp_path = if parent.as_os_str().is_empty() {
            PathBuf::from(tmp_file_name)
        } else {
            parent.join(tmp_file_name)
        };

        fs::write(&tmp_path, &json).map_err(|e| {
            format!(
                "Failed to write network config temp file {}: {}",
                tmp_path.display(),
                e
            )
        })?;
        if let Err(e) = fs::rename(&tmp_path, path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(format!(
                "Failed to atomically rename {} to {}: {}",
                tmp_path.display(),
                path.display(),
                e
            ));
        }
        Ok(())
    }

    /// Loads configuration taking environment variables into account (NCONF5).
    pub fn from_env() -> Self {
        let mut cfg = if let Ok(config_path) = std::env::var("AIOS_NETWORK_CONFIG") {
            Self::load_from_path(Path::new(&config_path)).unwrap_or_default()
        } else {
            Self::default()
        };

        // Check store path env vars
        if let Ok(store) = std::env::var("AIOS_NETWORK_STORE_PATH").or_else(|_| std::env::var("AIOS_NETWORK_STORE")) {
            if !store.trim().is_empty() {
                cfg.default_store_path = PathBuf::from(store);
            }
        }
        // Check sysfs path env vars
        if let Ok(sysfs) = std::env::var("AIOS_NETWORK_SYSFS_PATH").or_else(|_| std::env::var("AIOS_NETWORK_SYSFS")) {
            if !sysfs.trim().is_empty() {
                cfg.sysfs_net_path = PathBuf::from(sysfs);
            }
        }
        // Check procfs path env vars
        if let Ok(procfs) = std::env::var("AIOS_NETWORK_PROCFS_PATH").or_else(|_| std::env::var("AIOS_NETWORK_PROCFS")) {
            if !procfs.trim().is_empty() {
                cfg.procfs_path = PathBuf::from(procfs);
            }
        }
        // Check resolv.conf path env vars
        if let Ok(resolv) = std::env::var("AIOS_NETWORK_RESOLV_PATH").or_else(|_| std::env::var("AIOS_NETWORK_RESOLV_CONF")) {
            if !resolv.trim().is_empty() {
                cfg.resolv_conf_path = PathBuf::from(resolv);
            }
        }
        // Check capacity bounds
        if let Ok(val) = std::env::var("AIOS_NETWORK_MAX_INTERFACES") {
            if let Ok(v) = val.trim().parse::<usize>() {
                if (1..=10_000).contains(&v) {
                    cfg.max_interfaces = v;
                }
            }
        }
        if let Ok(val) = std::env::var("AIOS_NETWORK_MAX_ROUTES") {
            if let Ok(v) = val.trim().parse::<usize>() {
                if (1..=50_000).contains(&v) {
                    cfg.max_routes = v;
                }
            }
        }
        if let Ok(val) = std::env::var("AIOS_NETWORK_MAX_DNS") {
            if let Ok(v) = val.trim().parse::<usize>() {
                if (1..=64).contains(&v) {
                    cfg.max_dns_servers = v;
                }
            }
        }
        // Check timeout env vars
        if let Ok(val) = std::env::var("AIOS_NETWORK_TIMEOUT").or_else(|_| std::env::var("AIOS_NETWORK_TIMEOUT_SECS")) {
            if let Ok(secs) = val.trim().parse::<u64>() {
                if secs > 0 && secs <= 300 {
                    cfg.scan_timeout_secs = secs;
                }
            }
        }

        // Post-validation guard: if env overrides result in an invalid state, fallback to default
        if cfg.validate().is_err() {
            return Self::default();
        }

        cfg
    }
}
