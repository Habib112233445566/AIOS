//! Configuration resolution for CI Smoke Orchestration (T-00153).
//!
//! Contract: `docs/tasks/evidence/T-00152-spec.md`.

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct CiConfig {
    pub results_path: String,
    pub max_file_bytes: usize,
    pub timeout_default_s: usize,
    pub load_retries: usize,
    pub retry_sleep_ms: usize,
}

impl CiConfig {
    pub fn from_env() -> Result<CiConfig, String> {
        Self::from_source(&|name| std::env::var(name).ok())
    }

    pub fn from_source(
        get: &dyn Fn(&str) -> Option<String>,
    ) -> Result<CiConfig, String> {
        let parse_usize = |name: &str, raw: &str, floor: usize| -> Result<usize, String> {
            let v: usize = raw.parse().map_err(|_| format!("invalid {}='{}': must be integer", name, raw))?;
            if v < floor {
                return Err(format!("invalid {}='{}': must be >= {}", name, raw, floor));
            }
            Ok(v)
        };

        let results_path = get("AIOSH_CI_RESULTS").unwrap_or_else(|| "/tmp/aiosh-ci-results.json".to_string());
        if results_path.is_empty() {
            return Err("invalid AIOSH_CI_RESULTS='': must not be empty".to_string());
        }

        let max_file_bytes = match get("AIOSH_CI_MAX_FILE_BYTES") {
            Some(raw) => parse_usize("AIOSH_CI_MAX_FILE_BYTES", &raw, 1024)?,
            None => 1048576,
        };
        let timeout_default_s = match get("AIOSH_CI_TIMEOUT_DEFAULT_S") {
            Some(raw) => parse_usize("AIOSH_CI_TIMEOUT_DEFAULT_S", &raw, 10)?,
            None => 900,
        };
        let load_retries = match get("AIOSH_CI_LOAD_RETRIES") {
            Some(raw) => parse_usize("AIOSH_CI_LOAD_RETRIES", &raw, 0)?,
            None => 3,
        };
        let retry_sleep_ms = match get("AIOSH_CI_RETRY_SLEEP_MS") {
            Some(raw) => parse_usize("AIOSH_CI_RETRY_SLEEP_MS", &raw, 10)?,
            None => 500,
        };

        Ok(CiConfig {
            results_path,
            max_file_bytes,
            timeout_default_s,
            load_retries,
            retry_sleep_ms,
        })
    }

    pub fn to_json_with_sources(&self) -> Value {
        self.to_json_with_sources_from(&|name| std::env::var(name).is_ok())
    }

    pub fn to_json_with_sources_from(
        &self,
        is_set: &dyn Fn(&str) -> bool,
    ) -> Value {
        let src = |name: &str| if is_set(name) { "env" } else { "default" };
        json!({
            "results_path": {"value": self.results_path, "source": src("AIOSH_CI_RESULTS")},
            "max_file_bytes": {"value": self.max_file_bytes, "source": src("AIOSH_CI_MAX_FILE_BYTES")},
            "timeout_default_s": {"value": self.timeout_default_s, "source": src("AIOSH_CI_TIMEOUT_DEFAULT_S")},
            "load_retries": {"value": self.load_retries, "source": src("AIOSH_CI_LOAD_RETRIES")},
            "retry_sleep_ms": {"value": self.retry_sleep_ms, "source": src("AIOSH_CI_RETRY_SLEEP_MS")},
        })
    }
}
