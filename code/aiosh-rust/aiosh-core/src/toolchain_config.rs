//! Configuration resolution for Dependency & Toolchain Pinning (T-00313).
//!
//! Contract: `docs/tasks/evidence/T-00312-data-model-specification.md`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::io::Read;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolchainManifest {
    pub rust_version: String,
    pub python_version: String,
    pub node_version: Option<String>,
    pub enforce_hashes: bool,
}

impl Default for ToolchainManifest {
    fn default() -> Self {
        Self {
            rust_version: "1.99.0".into(),
            python_version: "3.14".into(),
            node_version: Some("v24.18".into()),
            enforce_hashes: false,
        }
    }
}

impl ToolchainManifest {
    /// Loads the toolchain manifest from the environment or default path.
    pub fn from_env() -> Result<ToolchainManifest, String> {
        Self::from_source(&|name| std::env::var(name).ok())
    }

    /// Loads the toolchain manifest from an explicit file path.
    pub fn from_path(path: &str) -> Result<ToolchainManifest, String> {
        Self::from_source(&|_name| Some(path.to_string()))
    }

    /// Loads the toolchain manifest using a provided getter function.
    pub fn from_source(
        get: &dyn Fn(&str) -> Option<String>,
    ) -> Result<ToolchainManifest, String> {
        let resolved = get("AIOSH_TOOLCHAIN_CONFIG")
            .unwrap_or_else(|| "config/toolchain.json".into());

        let p = Path::new(&resolved);
        if !p.exists() {
            return Err(format!("toolchain config not found at {}", resolved));
        }

        let f = std::fs::File::open(p)
            .map_err(|e| format!("Failed to open toolchain config: {e}"))?;
        let mut content = String::new();
        f.take(65_536)
            .read_to_string(&mut content)
            .map_err(|e| format!("Failed to read toolchain config: {e}"))?;

        let cfg: ToolchainManifest = serde_json::from_str(&content)
            .map_err(|e| format!("Malformed toolchain config: {e}"))?;

        if cfg.rust_version.trim().is_empty() {
            return Err("invalid toolchain config: rust_version cannot be empty".into());
        }
        if cfg.python_version.trim().is_empty() {
            return Err("invalid toolchain config: python_version cannot be empty".into());
        }
        if let Some(nv) = &cfg.node_version {
            if nv.trim().is_empty() {
                return Err("invalid toolchain config: node_version cannot be empty".into());
            }
        }

        Ok(cfg)
    }

    /// Exports the manifest to JSON, appending source provenance (env vs default).
    pub fn to_json_with_sources(&self) -> Value {
        self.to_json_with_sources_from(&|name| std::env::var(name).is_ok())
    }

    /// Exports the manifest to JSON using a custom `is_set` checker.
    pub fn to_json_with_sources_from(
        &self,
        is_set: &dyn Fn(&str) -> bool,
    ) -> Value {
        let src = |name: &str| if is_set(name) { "env" } else { "default" };
        json!({
            "rust_version": {"value": self.rust_version, "source": src("AIOSH_TOOLCHAIN_CONFIG")},
            "python_version": {"value": self.python_version, "source": src("AIOSH_TOOLCHAIN_CONFIG")},
            "node_version": {"value": self.node_version, "source": src("AIOSH_TOOLCHAIN_CONFIG")},
            "enforce_hashes": {"value": self.enforce_hashes, "source": src("AIOSH_TOOLCHAIN_CONFIG")},
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_toolchain_config_happy_path() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("toolchain.json");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(br#"{"rust_version": "1.80.0", "python_version": "3.10", "node_version": "20", "enforce_hashes": true}"#).unwrap();

        let path_str = file_path.to_str().unwrap().to_string();
        let get = |name: &str| -> Option<String> {
            if name == "AIOSH_TOOLCHAIN_CONFIG" { Some(path_str.clone()) } else { None }
        };

        let cfg = ToolchainManifest::from_source(&get).unwrap();
        assert_eq!(cfg.rust_version, "1.80.0");
        assert_eq!(cfg.python_version, "3.10");
        assert_eq!(cfg.node_version.as_deref(), Some("20"));
        assert_eq!(cfg.enforce_hashes, true);
    }

    #[test]
    fn test_load_toolchain_config_empty_version() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("toolchain_empty.json");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(br#"{"rust_version": "", "python_version": "3.10", "enforce_hashes": true}"#).unwrap();

        let path_str = file_path.to_str().unwrap().to_string();
        let get = |name: &str| -> Option<String> {
            if name == "AIOSH_TOOLCHAIN_CONFIG" { Some(path_str.clone()) } else { None }
        };

        let err = ToolchainManifest::from_source(&get).unwrap_err();
        assert!(err.contains("rust_version cannot be empty"));
    }

    #[test]
    fn test_load_toolchain_config_missing_file() {
        let get = |_name: &str| -> Option<String> {
            Some("/does/not/exist/missing_toolchain.json".into())
        };

        let err = ToolchainManifest::from_source(&get).unwrap_err();
        assert!(err.contains("toolchain config not found"));
    }

    #[test]
    fn test_load_toolchain_config_malformed_json() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("toolchain_bad.json");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(br#"{"rust_version": "1.80.0", bad_json "#).unwrap();

        let path_str = file_path.to_str().unwrap().to_string();
        let get = |name: &str| -> Option<String> {
            if name == "AIOSH_TOOLCHAIN_CONFIG" { Some(path_str.clone()) } else { None }
        };

        let err = ToolchainManifest::from_source(&get).unwrap_err();
        assert!(err.contains("Malformed toolchain config"));
    }

    #[test]
    fn test_load_toolchain_config_missing_field() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("toolchain_missing.json");
        let mut f = std::fs::File::create(&file_path).unwrap();
        // Missing python_version
        f.write_all(br#"{"rust_version": "1.80.0", "enforce_hashes": true}"#).unwrap();

        let path_str = file_path.to_str().unwrap().to_string();
        let get = |name: &str| -> Option<String> {
            if name == "AIOSH_TOOLCHAIN_CONFIG" { Some(path_str.clone()) } else { None }
        };

        let err = ToolchainManifest::from_source(&get).unwrap_err();
        assert!(err.contains("missing field `python_version`"));
    }

    #[test]
    fn test_to_json_with_sources() {
        let manifest = ToolchainManifest {
            rust_version: "1.98.0".into(),
            python_version: "3.12".into(),
            node_version: None,
            enforce_hashes: true,
        };

        let is_set = |name: &str| -> bool {
            name == "AIOSH_TOOLCHAIN_CONFIG"
        };

        let val = manifest.to_json_with_sources_from(&is_set);
        assert_eq!(val["rust_version"]["value"], "1.98.0");
        assert_eq!(val["rust_version"]["source"], "env");
        assert_eq!(val["node_version"]["value"], serde_json::Value::Null);
    }

    #[test]
    fn test_from_path_happy() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("tc.json");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(br#"{"rust_version": "1.99.0", "python_version": "3.14", "enforce_hashes": false}"#).unwrap();

        let cfg = ToolchainManifest::from_path(file_path.to_str().unwrap()).unwrap();
        assert_eq!(cfg.rust_version, "1.99.0");
        assert_eq!(cfg.python_version, "3.14");
        assert_eq!(cfg.node_version, None);
        assert_eq!(cfg.enforce_hashes, false);
    }

    #[test]
    fn test_from_path_missing() {
        let err = ToolchainManifest::from_path("/nonexistent/toolchain.json").unwrap_err();
        assert!(err.contains("toolchain config not found"));
    }
}
