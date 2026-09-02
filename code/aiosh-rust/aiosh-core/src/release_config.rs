//! Release & Backup configuration loader (Rust side).
//!
//! Reads from `$AIOSH_RELEASE_CONFIG` or `config/release.json`,
//! falling back to hardcoded defaults if the file is absent.

use serde::{Deserialize, Serialize};
use std::path::Path;

const MIN_FILE_SIZE: u64 = 1_048_576;         // 1 MB
const MAX_FILE_SIZE: u64 = 10_737_418_240;     // 10 GB
const DEFAULT_MAX_FILE_SIZE: u64 = 2_147_483_648; // 2 GB

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupDefaults {
    #[serde(default = "default_true")]
    pub include_audit: bool,
    #[serde(default)]
    pub include_memory: bool,
}

fn default_true() -> bool { true }

impl Default for BackupDefaults {
    fn default() -> Self {
        Self { include_audit: true, include_memory: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseConfig {
    #[serde(default = "default_max_file_size")]
    pub max_file_size_bytes: u64,
    #[serde(default = "default_components")]
    pub default_components: Vec<String>,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default)]
    pub backup_defaults: BackupDefaults,
}

fn default_max_file_size() -> u64 { DEFAULT_MAX_FILE_SIZE }
fn default_components() -> Vec<String> { vec!["core".into()] }
fn default_output_dir() -> String { "output/release".into() }

impl Default for ReleaseConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: DEFAULT_MAX_FILE_SIZE,
            default_components: default_components(),
            output_dir: default_output_dir(),
            backup_defaults: BackupDefaults::default(),
        }
    }
}

/// Load release configuration from a JSON file.
///
/// Resolution order:
///   1. Explicit `path` argument
///   2. `$AIOSH_RELEASE_CONFIG` environment variable
///   3. `config/release.json` relative to CWD
///
/// Returns `ReleaseConfig::default()` if the file is not found.
/// Returns `Err` if the file exists but is malformed.
pub fn load_config(path: Option<&str>) -> Result<ReleaseConfig, String> {
    let resolved = path
        .map(|p| p.to_string())
        .or_else(|| std::env::var("AIOSH_RELEASE_CONFIG").ok())
        .unwrap_or_else(|| "config/release.json".into());

    let p = Path::new(&resolved);
    if !p.exists() {
        return Ok(ReleaseConfig::default());
    }

    let f = std::fs::File::open(p)
        .map_err(|e| format!("Failed to open release config: {e}"))?;
    let mut content = String::new();
    use std::io::Read;
    f.take(65_536)
        .read_to_string(&mut content)
        .map_err(|e| format!("Failed to read release config: {e}"))?;
        
    let mut cfg: ReleaseConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Malformed release config: {e}"))?;
        
    if cfg.output_dir.contains("..") || cfg.output_dir.starts_with("/") || cfg.output_dir.starts_with("\\") || cfg.output_dir.contains(":") {
        return Err("output_dir contains illegal characters or is an absolute path".into());
    }

    // Clamp max_file_size_bytes
    cfg.max_file_size_bytes = cfg.max_file_size_bytes.clamp(MIN_FILE_SIZE, MAX_FILE_SIZE);

    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config_size_bound() {
        let mut tmp = NamedTempFile::new().unwrap();
        let data = "{\"output_dir\": \"a\"}";
        let padding = " ".repeat(70_000);
        let content = format!("{}{}", padding, data);
        tmp.write_all(content.as_bytes()).unwrap();
        let path = tmp.path().to_str().unwrap();

        let res = load_config(Some(path));
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Malformed release config"));
    }

    #[test]
    fn test_load_config_rejects_path_traversal() {
        let mut tmp = NamedTempFile::new().unwrap();
        let content = r#"{"output_dir": "../hacked"}"#;
        tmp.write_all(content.as_bytes()).unwrap();
        let path = tmp.path().to_str().unwrap();

        let res = load_config(Some(path));
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("illegal characters or is an absolute path"));
    }

    #[test]
    fn test_load_config_rejects_absolute_paths() {
        let mut tmp = NamedTempFile::new().unwrap();
        let content = r#"{"output_dir": "/var/aios"}"#;
        tmp.write_all(content.as_bytes()).unwrap();
        let path = tmp.path().to_str().unwrap();

        let res = load_config(Some(path));
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("illegal characters or is an absolute path"));
    }

    #[test]
    fn test_load_config_happy_path() {
        let mut tmp = NamedTempFile::new().unwrap();
        let content = r#"{"output_dir": "custom_output", "max_file_size_bytes": 104857600}"#;
        tmp.write_all(content.as_bytes()).unwrap();
        let path = tmp.path().to_str().unwrap();

        let res = load_config(Some(path));
        assert!(res.is_ok());
        let cfg = res.unwrap();
        assert_eq!(cfg.output_dir, "custom_output");
        assert_eq!(cfg.max_file_size_bytes, 104857600);
    }
}
