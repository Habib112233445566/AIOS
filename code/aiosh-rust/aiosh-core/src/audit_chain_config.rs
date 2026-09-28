//! Audit Chain Extensions Configuration Module (T-02341..T-02350).
//!
//! Contract: `docs/tasks/evidence/T-02342-configuration-specification.md`.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Default path for the audit database.
pub const DEFAULT_AUDIT_DB_PATH: &str = ".aios/audit.db";

/// Default query limit for pagination.
pub const DEFAULT_MAX_QUERY_LIMIT: usize = 50;

/// Maximum permissible query limit hard cap.
pub const MAX_PERMISSIBLE_QUERY_LIMIT: usize = 1000;

/// Default lineage traversal depth for causal DAGs.
pub const DEFAULT_LINEAGE_DEPTH: usize = 16;

/// Maximum permissible lineage traversal depth.
pub const MAX_LINEAGE_DEPTH_BOUND: usize = 64;

/// Default maximum causal links per event.
pub const DEFAULT_MAX_CAUSAL_LINKS: usize = 16;

/// Maximum permissible causal links per event.
pub const MAX_PERMISSIBLE_CAUSAL_LINKS: usize = 32;

/// Default maximum extensions payload size in bytes (64 KiB).
pub const DEFAULT_MAX_EXTENSIONS_BYTES: usize = 65536;

/// Maximum configuration file size in bytes (64 KiB).
pub const MAX_CONFIG_FILE_BYTES: u64 = 64 * 1024;

pub const AUDITCONF_ERR_IO: &str = "AUDITCONF_ERR_IO";
pub const AUDITCONF_ERR_PARSE: &str = "AUDITCONF_ERR_PARSE";
pub const AUDITCONF_ERR_VALIDATION: &str = "AUDITCONF_ERR_VALIDATION";
pub const AUDITCONF_ERR_BOUNDS: &str = "AUDITCONF_ERR_BOUNDS";

/// Configuration for Audit Chain Extensions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditChainConfig {
    pub version: String,
    pub db_path: PathBuf,
    pub max_query_limit: usize,
    pub default_lineage_depth: usize,
    pub max_causal_links: usize,
    pub max_extensions_bytes: usize,
    pub verify_signatures_on_read: bool,
    pub strict_provenance: bool,
}

impl Default for AuditChainConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            db_path: PathBuf::from(DEFAULT_AUDIT_DB_PATH),
            max_query_limit: DEFAULT_MAX_QUERY_LIMIT,
            default_lineage_depth: DEFAULT_LINEAGE_DEPTH,
            max_causal_links: DEFAULT_MAX_CAUSAL_LINKS,
            max_extensions_bytes: DEFAULT_MAX_EXTENSIONS_BYTES,
            verify_signatures_on_read: true,
            strict_provenance: false,
        }
    }
}

impl AuditChainConfig {
    /// Validates the configuration bounds.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.is_empty() || !self.version.starts_with("1.") {
            return Err(format!("{}: invalid version '{}'", AUDITCONF_ERR_VALIDATION, self.version));
        }
        if self.max_query_limit == 0 || self.max_query_limit > MAX_PERMISSIBLE_QUERY_LIMIT {
            return Err(format!("{}: max_query_limit {} out of bounds (1..{})", AUDITCONF_ERR_BOUNDS, self.max_query_limit, MAX_PERMISSIBLE_QUERY_LIMIT));
        }
        if self.default_lineage_depth == 0 || self.default_lineage_depth > MAX_LINEAGE_DEPTH_BOUND {
            return Err(format!("{}: default_lineage_depth {} out of bounds (1..{})", AUDITCONF_ERR_BOUNDS, self.default_lineage_depth, MAX_LINEAGE_DEPTH_BOUND));
        }
        if self.max_causal_links == 0 || self.max_causal_links > MAX_PERMISSIBLE_CAUSAL_LINKS {
            return Err(format!("{}: max_causal_links {} out of bounds (1..{})", AUDITCONF_ERR_BOUNDS, self.max_causal_links, MAX_PERMISSIBLE_CAUSAL_LINKS));
        }
        if self.max_extensions_bytes < 1024 || self.max_extensions_bytes > 1_048_576 {
            return Err(format!("{}: max_extensions_bytes {} out of bounds (1024..1048576)", AUDITCONF_ERR_BOUNDS, self.max_extensions_bytes));
        }
        Ok(())
    }

    /// Deserializes and validates AuditChainConfig from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: AuditChainConfig = serde_json::from_str(json_str)
            .map_err(|e| format!("{}: failed to parse JSON: {}", AUDITCONF_ERR_PARSE, e))?;
        config.validate()?;
        Ok(config)
    }

    /// Loads and validates configuration from a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("{}: metadata error for '{}': {}", AUDITCONF_ERR_IO, path.display(), e))?;
        if metadata.len() > MAX_CONFIG_FILE_BYTES {
            return Err(format!("{}: file size {} exceeds limit of {}", AUDITCONF_ERR_BOUNDS, metadata.len(), MAX_CONFIG_FILE_BYTES));
        }
        let mut file = File::open(path)
            .map_err(|e| format!("{}: failed to open file '{}': {}", AUDITCONF_ERR_IO, path.display(), e))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("{}: failed to read file '{}': {}", AUDITCONF_ERR_IO, path.display(), e))?;
        Self::from_json(&content)
    }

    /// Loads configuration with environment variable overrides.
    pub fn from_env() -> Self {
        let mut cfg = if let Ok(path) = std::env::var("AIOS_AUDIT_CONFIG_PATH") {
            Self::from_file(&path).unwrap_or_default()
        } else {
            Self::default()
        };

        if let Ok(db) = std::env::var("AIOS_AUDIT_DB_PATH") {
            if !db.trim().is_empty() {
                cfg.db_path = PathBuf::from(db.trim());
            }
        }
        if let Ok(limit) = std::env::var("AIOS_AUDIT_MAX_QUERY_LIMIT") {
            if let Ok(val) = limit.parse::<usize>() {
                if val > 0 && val <= MAX_PERMISSIBLE_QUERY_LIMIT {
                    cfg.max_query_limit = val;
                }
            }
        }
        if let Ok(depth) = std::env::var("AIOS_AUDIT_LINEAGE_DEPTH") {
            if let Ok(val) = depth.parse::<usize>() {
                if val > 0 && val <= MAX_LINEAGE_DEPTH_BOUND {
                    cfg.default_lineage_depth = val;
                }
            }
        }

        cfg
    }

    /// Atomically persists configuration to file.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.validate()?;
        let path = path.as_ref();
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: failed to create directory '{}': {}", AUDITCONF_ERR_IO, parent.display(), e))?;
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: serialization failed: {}", AUDITCONF_ERR_PARSE, e))?;
        let tmp_path = parent.join(format!(".tmp_audit_cfg_{}", std::process::id()));
        std::fs::write(&tmp_path, json_str.as_bytes())
            .map_err(|e| format!("{}: failed to write temp config: {}", AUDITCONF_ERR_IO, e))?;
        std::fs::rename(&tmp_path, path)
            .map_err(|e| format!("{}: failed to rename config file: {}", AUDITCONF_ERR_IO, e))?;
        Ok(())
    }
}
