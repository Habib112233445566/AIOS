//! Base Image Build Configuration
//!
//! Enforces configuration loading, precedence, and validation invariants CF1..CF6.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Configuration options for the AIOS Base Image Build subsystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageBuildConfig {
    /// Workspace scratch directory for building rootfs images.
    pub build_dir: PathBuf,
    /// Target publishing directory for compiled artifacts.
    pub output_dir: PathBuf,
    /// Default base image manifest identifier.
    pub default_target: String,
    /// Maximum build duration allowed per stage execution (seconds).
    pub max_build_duration_secs: u64,
    /// Maximum allowed image artifact size in bytes.
    pub max_artifact_size_bytes: u64,
    /// Compression level for target archive creation (1..22).
    pub compression_level: u32,
}

impl Default for ImageBuildConfig {
    fn default() -> Self {
        Self {
            build_dir: PathBuf::from(".aios/build"),
            output_dir: PathBuf::from(".aios/images"),
            default_target: "debian-12-minimal-raw".to_string(),
            max_build_duration_secs: 1800,
            max_artifact_size_bytes: 10 * 1024 * 1024 * 1024, // 10 GiB
            compression_level: 3,
        }
    }
}

impl ImageBuildConfig {
    /// Validates the configuration against criteria CF1..CF6.
    pub fn validate(&self) -> Result<(), String> {
        if self.build_dir.as_os_str().is_empty() {
            return Err("CF1 violation: build_dir cannot be empty".into());
        }
        if let Some(s) = self.build_dir.to_str() {
            if s.chars().any(|c| c.is_control() || c == '\0') {
                return Err("CF1 violation: build_dir cannot contain control characters or null bytes".into());
            }
        }
        if self.output_dir.as_os_str().is_empty() {
            return Err("CF2 violation: output_dir cannot be empty".into());
        }
        if let Some(s) = self.output_dir.to_str() {
            if s.chars().any(|c| c.is_control() || c == '\0') {
                return Err("CF2 violation: output_dir cannot contain control characters or null bytes".into());
            }
        }
        if self.default_target.is_empty()
            || self.default_target.len() > 128
            || !self.default_target.chars().all(|c| c.is_ascii_graphic())
        {
            return Err("CF3 violation: default_target must be 1..128 printable ASCII characters".into());
        }
        if self.max_build_duration_secs < 10 || self.max_build_duration_secs > 86400 {
            return Err("CF4 violation: max_build_duration_secs must be between 10 and 86400 seconds".into());
        }
        if self.max_artifact_size_bytes < 1024 * 1024 || self.max_artifact_size_bytes > 100 * 1024 * 1024 * 1024 {
            return Err("CF5 violation: max_artifact_size_bytes must be between 1 MiB and 100 GiB".into());
        }
        if self.compression_level < 1 || self.compression_level > 22 {
            return Err("CF6 violation: compression_level must be between 1 and 22".into());
        }
        Ok(())
    }

    /// Loads configuration from environment variables with fallback to defaults.
    pub fn from_env() -> Result<Self, String> {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("AIOS_IMAGE_BUILD_DIR") {
            if !v.trim().is_empty() {
                cfg.build_dir = PathBuf::from(v.trim());
            }
        }
        if let Ok(v) = std::env::var("AIOS_IMAGE_OUTPUT_DIR") {
            if !v.trim().is_empty() {
                cfg.output_dir = PathBuf::from(v.trim());
            }
        }
        if let Ok(v) = std::env::var("AIOS_IMAGE_DEFAULT_TARGET") {
            if !v.trim().is_empty() {
                cfg.default_target = v.trim().to_string();
            }
        }
        if let Ok(v) = std::env::var("AIOS_IMAGE_TIMEOUT_SECS") {
            if let Ok(secs) = v.trim().parse::<u64>() {
                cfg.max_build_duration_secs = secs;
            }
        }
        if let Ok(v) = std::env::var("AIOS_IMAGE_MAX_SIZE_BYTES") {
            if let Ok(sz) = v.trim().parse::<u64>() {
                cfg.max_artifact_size_bytes = sz;
            }
        }
        if let Ok(v) = std::env::var("AIOS_IMAGE_COMPRESSION_LEVEL") {
            if let Ok(lvl) = v.trim().parse::<u32>() {
                cfg.compression_level = lvl;
            }
        }
        cfg.validate()?;
        Ok(cfg)
    }

    /// Loads configuration from a JSON file on disk, enforcing a 10 MiB limit.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        use std::io::Read;
        let file = std::fs::File::open(path)
            .map_err(|e| format!("failed to open image config at '{}': {}", path.display(), e))?;
        let meta = file.metadata()
            .map_err(|e| format!("failed to read metadata for '{}': {}", path.display(), e))?;
        if meta.len() > 10 * 1024 * 1024 {
            return Err(format!("image config file at '{}' exceeds maximum allowed size of 10 MiB", path.display()));
        }
        let mut bytes = Vec::new();
        file.take(10 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("failed to read image config from '{}': {}", path.display(), e))?;
        let cfg: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("failed to parse image config from '{}': {}", path.display(), e))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Persists configuration to a JSON file on disk with safe permissions.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize image config: {}", e))?;
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(path, content.as_bytes())
            .map_err(|e| format!("failed to write image config to '{}': {}", path.display(), e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644));
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_default_config_valid() {
        let cfg = ImageBuildConfig::default();
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.default_target, "debian-12-minimal-raw");
    }

    #[test]
    fn test_validation_cf1_cf6_failures() {
        // CF1
        let mut c = ImageBuildConfig::default();
        c.build_dir = PathBuf::from("");
        assert!(c.validate().unwrap_err().contains("CF1"));
        c.build_dir = PathBuf::from("/tmp/bad\x07dir");
        assert!(c.validate().unwrap_err().contains("CF1"));

        // CF2
        let mut c = ImageBuildConfig::default();
        c.output_dir = PathBuf::from("");
        assert!(c.validate().unwrap_err().contains("CF2"));
        c.output_dir = PathBuf::from("/tmp/bad\x07dir");
        assert!(c.validate().unwrap_err().contains("CF2"));

        // CF3
        let mut c = ImageBuildConfig::default();
        c.default_target = "".into();
        assert!(c.validate().unwrap_err().contains("CF3"));
        c.default_target = "bad\x07id".into();
        assert!(c.validate().unwrap_err().contains("CF3"));

        // CF4
        let mut c = ImageBuildConfig::default();
        c.max_build_duration_secs = 5;
        assert!(c.validate().unwrap_err().contains("CF4"));
        c.max_build_duration_secs = 90000;
        assert!(c.validate().unwrap_err().contains("CF4"));

        // CF5
        let mut c = ImageBuildConfig::default();
        c.max_artifact_size_bytes = 100;
        assert!(c.validate().unwrap_err().contains("CF5"));

        // CF6
        let mut c = ImageBuildConfig::default();
        c.compression_level = 0;
        assert!(c.validate().unwrap_err().contains("CF6"));
        c.compression_level = 23;
        assert!(c.validate().unwrap_err().contains("CF6"));
    }

    #[test]
    fn test_persistence_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("image_cfg.json");

        let mut original = ImageBuildConfig::default();
        original.default_target = "alpine-319-container-tarball".into();
        original.compression_level = 9;

        assert!(original.save_to_path(&path).is_ok());
        let loaded = ImageBuildConfig::from_file(&path).unwrap();
        assert_eq!(original, loaded);
    }
}
