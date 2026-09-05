//! Package Management Configuration Subsystem (T-01244 Implementation).
//!
//! Enforces configuration resolution, precedence, and validation invariants PC1..PC6.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::package::PackageFormat;

pub const DEFAULT_PACKAGE_STORE_PATH: &str = ".aios/packages.json";
pub const DEFAULT_MAX_STORE_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB
pub const DEFAULT_MAX_ENTITY_COUNT: usize = 10_000;
pub const MAX_CONFIG_FILE_BYTES: u64 = 65_536; // 64 KiB

pub const MIN_STORE_SIZE_BYTES: u64 = 65_536; // 64 KiB
pub const MAX_ALLOWED_STORE_SIZE_BYTES: u64 = 100 * 1024 * 1024; // 100 MiB
pub const MIN_ENTITY_COUNT: usize = 10;
pub const MAX_ALLOWED_ENTITY_COUNT: usize = 100_000;

/// Configuration options for the AIOS Package Management subsystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Canonical filesystem path to the persistent package store JSON file.
    pub store_path: PathBuf,
    /// Default packaging format used when format filter/target is omitted.
    pub default_format: PackageFormat,
    /// Maximum allowed package store file size on disk (bytes).
    pub max_store_size_bytes: u64,
    /// Maximum package entities permitted within a single store.
    pub max_entity_count: usize,
    /// Whether mutations automatically persist to store_path without explicit flag.
    pub auto_persist: bool,
    /// List of trusted HTTPS repository upstream URLs.
    pub allowed_repositories: Vec<String>,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            store_path: PathBuf::from(DEFAULT_PACKAGE_STORE_PATH),
            default_format: PackageFormat::Deb,
            max_store_size_bytes: DEFAULT_MAX_STORE_SIZE_BYTES,
            max_entity_count: DEFAULT_MAX_ENTITY_COUNT,
            auto_persist: false,
            allowed_repositories: vec![
                "https://deb.debian.org/debian".to_string(),
                "https://dl-cdn.alpinelinux.org/alpine/v3.19/main".to_string(),
            ],
        }
    }
}

impl PackageConfig {
    /// Validates the configuration against invariants PC1..PC6.
    pub fn validate(&self) -> Result<(), String> {
        // PC1: store_path validity
        if self.store_path.as_os_str().is_empty() {
            return Err("PC1 violation: store_path cannot be empty".into());
        }
        let path_str = self.store_path.to_string_lossy();
        if path_str.len() > 1024 {
            return Err(format!(
                "PC1 violation: store_path length ({} bytes) exceeds maximum limit of 1024 bytes",
                path_str.len()
            ));
        }
        if path_str.chars().any(|c| c.is_control() || c == '\0') {
            return Err("PC1 violation: store_path cannot contain control characters or null bytes".into());
        }

        // PC2: max_store_size_bytes bounds
        if self.max_store_size_bytes < MIN_STORE_SIZE_BYTES
            || self.max_store_size_bytes > MAX_ALLOWED_STORE_SIZE_BYTES
        {
            return Err(format!(
                "PC2 violation: max_store_size_bytes must be between {} (64 KiB) and {} (100 MiB), got {}",
                MIN_STORE_SIZE_BYTES, MAX_ALLOWED_STORE_SIZE_BYTES, self.max_store_size_bytes
            ));
        }

        // PC3: max_entity_count bounds
        if self.max_entity_count < MIN_ENTITY_COUNT
            || self.max_entity_count > MAX_ALLOWED_ENTITY_COUNT
        {
            return Err(format!(
                "PC3 violation: max_entity_count must be between {} and {}, got {}",
                MIN_ENTITY_COUNT, MAX_ALLOWED_ENTITY_COUNT, self.max_entity_count
            ));
        }

        // PC4: repository transport security
        for repo in &self.allowed_repositories {
            if !repo.starts_with("https://") && !repo.starts_with("file://") {
                return Err(format!(
                    "PC4 violation: repository '{}' must use secure HTTPS or file:// transport",
                    repo
                ));
            }
            if repo.chars().any(|c| c.is_control() || c == '\0') {
                return Err(format!(
                    "PC4 violation: repository '{}' contains control characters",
                    repo
                ));
            }
        }

        Ok(())
    }

    /// Loads configuration from an explicit file path on disk (PC5, PC6).
    pub fn from_file(path: &Path) -> Result<Self, String> {
        use std::io::Read;

        let file = std::fs::File::open(path)
            .map_err(|e| format!("failed to open package config file at '{}': {}", path.display(), e))?;

        let meta = file
            .metadata()
            .map_err(|e| format!("failed to read metadata for '{}': {}", path.display(), e))?;

        if meta.len() > MAX_CONFIG_FILE_BYTES {
            return Err(format!(
                "PC6 violation: package config at '{}' exceeds maximum allowed size of 64 KiB (was {} bytes)",
                path.display(),
                meta.len()
            ));
        }

        let mut bytes = Vec::new();
        file.take(MAX_CONFIG_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("failed to read package config from '{}': {}", path.display(), e))?;

        if bytes.len() as u64 > MAX_CONFIG_FILE_BYTES {
            return Err("PC6 violation: package config content exceeded 64 KiB during stream read".into());
        }

        let config: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("failed to parse package config JSON at '{}': {}", path.display(), e))?;

        config.validate()?;
        Ok(config)
    }

    /// Loads configuration from environment variables with fallback to defaults (PC5).
    pub fn from_env() -> Result<Self, String> {
        let mut cfg = Self::default();

        if let Ok(val) = std::env::var("AIOS_PACKAGE_STORE_PATH") {
            if !val.trim().is_empty() {
                cfg.store_path = PathBuf::from(val.trim());
            }
        }

        if let Ok(val) = std::env::var("AIOS_PACKAGE_DEFAULT_FORMAT") {
            let fmt_str = val.trim().to_lowercase();
            cfg.default_format = match fmt_str.as_str() {
                "deb" => PackageFormat::Deb,
                "apk" => PackageFormat::Apk,
                "flatpak" => PackageFormat::Flatpak,
                "tarball" => PackageFormat::Tarball,
                other => return Err(format!("unknown format '{}' in AIOS_PACKAGE_DEFAULT_FORMAT", other)),
            };
        }

        if let Ok(val) = std::env::var("AIOS_PACKAGE_MAX_STORE_SIZE_BYTES") {
            if let Ok(sz) = val.trim().parse::<u64>() {
                cfg.max_store_size_bytes = sz;
            } else {
                return Err(format!("invalid integer in AIOS_PACKAGE_MAX_STORE_SIZE_BYTES: {}", val));
            }
        }

        if let Ok(val) = std::env::var("AIOS_PACKAGE_MAX_ENTITIES") {
            if let Ok(cnt) = val.trim().parse::<usize>() {
                cfg.max_entity_count = cnt;
            } else {
                return Err(format!("invalid integer in AIOS_PACKAGE_MAX_ENTITIES: {}", val));
            }
        }

        if let Ok(val) = std::env::var("AIOS_PACKAGE_AUTO_PERSIST") {
            let s = val.trim().to_lowercase();
            cfg.auto_persist = s == "1" || s == "true" || s == "yes";
        }

        if let Ok(val) = std::env::var("AIOS_PACKAGE_ALLOWED_REPOS") {
            let repos: Vec<String> = val
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !repos.is_empty() {
                cfg.allowed_repositories = repos;
            }
        }

        cfg.validate()?;
        Ok(cfg)
    }

    /// Resolves configuration adhering to precedence PC5 (file > env > default).
    pub fn resolve(config_path_opt: Option<&Path>) -> Result<Self, String> {
        if let Some(path) = config_path_opt {
            return Self::from_file(path);
        }
        if let Ok(env_path) = std::env::var("AIOS_PACKAGE_CONFIG") {
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
    fn test_package_config_default_and_validation() {
        let cfg = PackageConfig::default();
        assert_eq!(cfg.validate(), Ok(()));
        assert_eq!(cfg.store_path, PathBuf::from(DEFAULT_PACKAGE_STORE_PATH));
        assert_eq!(cfg.default_format, PackageFormat::Deb);
        assert_eq!(cfg.max_store_size_bytes, 10 * 1024 * 1024);
        assert_eq!(cfg.max_entity_count, 10_000);
        assert_eq!(cfg.auto_persist, false);
    }

    #[test]
    fn test_package_config_pc1_store_path_invariants() {
        let mut cfg = PackageConfig::default();
        cfg.store_path = PathBuf::from("");
        assert!(cfg.validate().unwrap_err().contains("PC1 violation"));

        cfg.store_path = PathBuf::from("a".repeat(1025));
        assert!(cfg.validate().unwrap_err().contains("PC1 violation"));

        cfg.store_path = PathBuf::from("packages\0.json");
        assert!(cfg.validate().unwrap_err().contains("PC1 violation"));
    }

    #[test]
    fn test_package_config_pc2_pc3_boundary_invariants() {
        let mut cfg = PackageConfig::default();
        // Size bounds
        cfg.max_store_size_bytes = 100; // too small
        assert!(cfg.validate().unwrap_err().contains("PC2 violation"));

        cfg.max_store_size_bytes = 200 * 1024 * 1024; // too large (>100 MiB)
        assert!(cfg.validate().unwrap_err().contains("PC2 violation"));

        // Entity bounds
        cfg = PackageConfig::default();
        cfg.max_entity_count = 5; // too small (<10)
        assert!(cfg.validate().unwrap_err().contains("PC3 violation"));

        cfg.max_entity_count = 500_000; // too large (>100k)
        assert!(cfg.validate().unwrap_err().contains("PC3 violation"));
    }

    #[test]
    fn test_package_config_pc4_repository_security() {
        let mut cfg = PackageConfig::default();
        cfg.allowed_repositories = vec!["http://insecure.deb.org".into()];
        assert!(cfg.validate().unwrap_err().contains("PC4 violation"));

        cfg.allowed_repositories = vec!["https://valid.deb.org".into(), "file:///tmp/mirror".into()];
        assert_eq!(cfg.validate(), Ok(()));
    }

    #[test]
    fn test_package_config_file_roundtrip_and_pc6() {
        let temp_dir = std::env::temp_dir();
        let config_file = temp_dir.join(format!("aios_pkg_cfg_test_{}.json", std::process::id()));

        let cfg = PackageConfig::default();
        let content = serde_json::to_string_pretty(&cfg).unwrap();
        std::fs::write(&config_file, content).unwrap();

        let loaded = PackageConfig::from_file(&config_file).unwrap();
        assert_eq!(loaded, cfg);

        let _ = std::fs::remove_file(&config_file);
    }
}
