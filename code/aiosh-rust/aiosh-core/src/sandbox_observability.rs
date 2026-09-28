//! Sandbox Enforcement Observability Subsystem (SANDBOXOBS1..SANDBOXOBS6).
//!
//! Aggregates point-in-time metrics, execution outcomes, profile utilization,
//! host containment capability health, and security policy states.

use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::sandbox_service::{HostSandboxCapabilities, SandboxService};

/// Error code: Telemetry validation or invariant failure.
pub const SANDBOXOBS_ERR_VALIDATION: &str = "SANDBOXOBS_ERR_VALIDATION";

/// Error code: Error querying database during aggregation.
pub const SANDBOXOBS_ERR_QUERY: &str = "SANDBOXOBS_ERR_QUERY";

/// Maximum length for sanitized telemetry text.
pub const MAX_TELEMETRY_TEXT_LEN: usize = 256;

/// Maximum number of outcome categories collected in telemetry distribution.
pub const MAX_OUTCOME_DISTRIBUTION_ENTRIES: usize = 128;

/// Sanitizes a string for safe inclusion in telemetry outputs.
pub fn sanitize_telemetry_text(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(MAX_TELEMETRY_TEXT_LEN)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Comprehensive observability and telemetry report for the Sandbox Enforcement subsystem.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SandboxObservabilityReport {
    pub generated_at_utc: String,
    pub total_profiles_registered: usize,
    pub total_executions_recorded: usize,
    pub executions_by_outcome: HashMap<String, usize>,
    pub executions_by_profile: HashMap<String, usize>,
    pub policy_mode: String,
    pub host_capabilities: HostSandboxCapabilities,
    pub is_healthy: bool,
}

impl SandboxObservabilityReport {
    pub fn generate(service: &SandboxService) -> Result<Self, String> {
        let profiles = service.list_profiles();
        let host_caps = service.probe_host_capabilities();
        let policy = service.policy();

        let mut total_executions = 0;
        let mut by_outcome = HashMap::new();
        let mut by_profile = HashMap::new();

        if let Some(ring) = service.ring() {
            if let Ok(rows) = ring.tail(1000) {
                for r in rows {
                    if r.tool == "sandbox" || r.tool.starts_with("aios.sandbox") {
                        total_executions += 1;
                        let outcome_clean = sanitize_telemetry_text(&r.outcome);
                        if by_outcome.len() < MAX_OUTCOME_DISTRIBUTION_ENTRIES {
                            *by_outcome.entry(outcome_clean).or_insert(0) += 1;
                        }

                        // Extract profile if present in args json
                        if let Some(prof_val) = r.args.get("profile").and_then(|v| v.as_str()) {
                            let prof_clean = sanitize_telemetry_text(prof_val);
                            if by_profile.len() < MAX_OUTCOME_DISTRIBUTION_ENTRIES {
                                *by_profile.entry(prof_clean).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }

        // Health evaluates whether default factory profiles exist
        let has_standard = service.get_profile("standard").is_some();
        let has_strict = service.get_profile("strict").is_some();
        let has_permissive = service.get_profile("permissive").is_some();
        let is_healthy = has_standard && has_strict && has_permissive;

        let report = Self {
            generated_at_utc: Utc::now().to_rfc3339(),
            total_profiles_registered: profiles.len(),
            total_executions_recorded: total_executions,
            executions_by_outcome: by_outcome,
            executions_by_profile: by_profile,
            policy_mode: format!("{:?}", policy.mode).to_lowercase(),
            host_capabilities: host_caps,
            is_healthy,
        };

        report.validate()?;
        Ok(report)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.generated_at_utc.trim().is_empty() {
            return Err(format!("{}: generated_at_utc cannot be empty", SANDBOXOBS_ERR_VALIDATION));
        }
        if self.executions_by_outcome.len() > MAX_OUTCOME_DISTRIBUTION_ENTRIES {
            return Err(format!("{}: outcome distribution entries exceed maximum {}", SANDBOXOBS_ERR_VALIDATION, MAX_OUTCOME_DISTRIBUTION_ENTRIES));
        }
        if self.executions_by_profile.len() > MAX_OUTCOME_DISTRIBUTION_ENTRIES {
            return Err(format!("{}: profile distribution entries exceed maximum {}", SANDBOXOBS_ERR_VALIDATION, MAX_OUTCOME_DISTRIBUTION_ENTRIES));
        }
        Ok(())
    }
}
