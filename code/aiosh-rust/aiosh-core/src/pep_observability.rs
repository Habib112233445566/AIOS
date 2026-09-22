//! PEP Decision Engine Observability Subsystem (PEPOBS1..PEPOBS6).
//!
//! Provides point-in-time state aggregation, capacity utilization monitoring,
//! rule effect distributions, obligation counts, and health diagnostics.

use std::collections::{HashMap, HashSet};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::pep_decision::{PepDecisionEffect, PepObligation};
use crate::pep_decision_service::{PepDecisionService, MAX_RULES_IN_SERVICE};
use crate::pep_security_policy::PepSecurityPolicy;

/// Error code: Telemetry or report invariant validation error.
pub const PEPOBS_ERR_VALIDATION: &str = "PEPOBS_ERR_VALIDATION";

/// Health threshold percentage: utilization at or above 90% transitions health to false.
pub const PEP_HEALTH_UTILIZATION_THRESHOLD: u8 = 90;

/// Sanitizes a string for safe inclusion in telemetry outputs (removes control characters, trims, limits to 256 chars) (PEPOBS5).
pub fn sanitize_telemetry_text(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(256)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Comprehensive observability and telemetry report for the PEP Decision Engine (PEPOBS1..PEPOBS6).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PepObservabilityReport {
    /// Total number of registered policy rules.
    pub total_rules: usize,
    /// Count of rules by decision effect (permit vs deny).
    pub rules_by_effect: HashMap<String, usize>,
    /// Number of rules that have one or more obligations attached.
    pub rules_with_obligations: usize,
    /// Total counts of obligations categorized by obligation variant.
    pub obligations_by_type: HashMap<String, usize>,
    /// Distinct target subjects registered across rules.
    pub unique_subjects_count: usize,
    /// Distinct target resources registered across rules.
    pub unique_resources_count: usize,
    /// Distinct target actions registered across rules.
    pub unique_actions_count: usize,
    /// Configured combining algorithm in the service.
    pub default_algorithm: String,
    /// Active policy enforcement mode (Enforcing, Permissive, Disabled).
    pub enforcement_mode: String,
    /// Active obligation criticality (Strict, BestEffort).
    pub obligation_criticality: String,
    /// Number of restricted resource prefixes defined in policy.
    pub restricted_prefixes_count: usize,
    /// Maximum capacity of rules in the service (MAX_RULES_IN_SERVICE = 5000).
    pub capacity_limit: usize,
    /// Current capacity utilization as a percentage (0..=100).
    pub capacity_utilization_percent: u8,
    /// Composite health indicator (healthy if utilization < 90%).
    pub is_healthy: bool,
    /// Storage path string if configured.
    pub store_path: Option<String>,
    /// ISO 8601 UTC timestamp of report generation.
    pub generated_at: String,
}

impl PepObservabilityReport {
    /// Generates a comprehensive observability report from a policy service and security policy (PEPOBS1..PEPOBS6).
    pub fn generate(
        service: &PepDecisionService,
        policy: &PepSecurityPolicy,
        timestamp: &str,
    ) -> Self {
        let now = Utc::now();
        let sanitized_ts = sanitize_telemetry_text(timestamp);
        let generated_at = if sanitized_ts.is_empty() {
            now.to_rfc3339()
        } else {
            sanitized_ts
        };

        let rules = service.list_rules();
        let total_rules = rules.len();

        let mut rules_by_effect = HashMap::new();
        rules_by_effect.insert("permit".to_string(), 0);
        rules_by_effect.insert("deny".to_string(), 0);
        rules_by_effect.insert("indeterminate".to_string(), 0);
        rules_by_effect.insert("not_applicable".to_string(), 0);

        let mut rules_with_obligations = 0;
        let mut obligations_by_type = HashMap::new();
        obligations_by_type.insert("audit_log".to_string(), 0);
        obligations_by_type.insert("rate_limit".to_string(), 0);
        obligations_by_type.insert("redact_fields".to_string(), 0);
        obligations_by_type.insert("custom".to_string(), 0);

        let mut subjects = HashSet::new();
        let mut resources = HashSet::new();
        let mut actions = HashSet::new();

        for rule in &rules {
            match rule.effect {
                PepDecisionEffect::Permit => {
                    *rules_by_effect.entry("permit".to_string()).or_insert(0) += 1;
                }
                PepDecisionEffect::Deny => {
                    *rules_by_effect.entry("deny".to_string()).or_insert(0) += 1;
                }
                PepDecisionEffect::Indeterminate => {
                    *rules_by_effect.entry("indeterminate".to_string()).or_insert(0) += 1;
                }
                PepDecisionEffect::NotApplicable => {
                    *rules_by_effect.entry("not_applicable".to_string()).or_insert(0) += 1;
                }
            }

            if !rule.obligations.is_empty() {
                rules_with_obligations += 1;
                for ob in &rule.obligations {
                    match ob {
                        PepObligation::AuditLog { .. } => {
                            *obligations_by_type.entry("audit_log".to_string()).or_insert(0) += 1;
                        }
                        PepObligation::RateLimit { .. } => {
                            *obligations_by_type.entry("rate_limit".to_string()).or_insert(0) += 1;
                        }
                        PepObligation::RedactFields { .. } => {
                            *obligations_by_type.entry("redact_fields".to_string()).or_insert(0) += 1;
                        }
                        PepObligation::Custom { .. } => {
                            *obligations_by_type.entry("custom".to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }

            if let Some(ref s) = rule.target_subject {
                subjects.insert(s.clone());
            }
            if let Some(ref r) = rule.target_resource {
                resources.insert(r.clone());
            }
            if let Some(ref a) = rule.target_action {
                actions.insert(a.clone());
            }
        }

        let capacity_limit = MAX_RULES_IN_SERVICE;
        let capacity_utilization_percent = if capacity_limit == 0 {
            0
        } else {
            ((total_rules * 100) / capacity_limit).min(100) as u8
        };

        let is_healthy = capacity_utilization_percent < PEP_HEALTH_UTILIZATION_THRESHOLD;

        let default_algorithm = format!("{:?}", service.algorithm()).to_lowercase();
        let enforcement_mode = format!("{:?}", policy.mode).to_lowercase();
        let obligation_criticality = format!("{:?}", policy.obligation_criticality).to_lowercase();
        let restricted_prefixes_count = policy.restricted_resource_prefixes.len();

        let store_path = service
            .storage_path()
            .map(|p| sanitize_telemetry_text(&p.to_string_lossy()));

        Self {
            total_rules,
            rules_by_effect,
            rules_with_obligations,
            obligations_by_type,
            unique_subjects_count: subjects.len(),
            unique_resources_count: resources.len(),
            unique_actions_count: actions.len(),
            default_algorithm,
            enforcement_mode,
            obligation_criticality,
            restricted_prefixes_count,
            capacity_limit,
            capacity_utilization_percent,
            is_healthy,
            store_path,
            generated_at,
        }
    }

    /// Validates report fields against structural consistency invariants (PEPOBS1, PEPOBS6).
    pub fn validate(&self) -> Result<(), String> {
        if self.total_rules > self.capacity_limit {
            return Err(format!(
                "{}: total_rules ({}) exceeds capacity_limit ({})",
                PEPOBS_ERR_VALIDATION, self.total_rules, self.capacity_limit
            ));
        }

        if self.capacity_utilization_percent > 100 {
            return Err(format!(
                "{}: capacity_utilization_percent ({}) exceeds 100",
                PEPOBS_ERR_VALIDATION, self.capacity_utilization_percent
            ));
        }

        let effect_sum: usize = self.rules_by_effect.values().sum();
        if effect_sum != self.total_rules {
            return Err(format!(
                "{}: sum of rules by effect ({}) does not equal total_rules ({})",
                PEPOBS_ERR_VALIDATION, effect_sum, self.total_rules
            ));
        }

        if self.rules_with_obligations > self.total_rules {
            return Err(format!(
                "{}: rules_with_obligations ({}) exceeds total_rules ({})",
                PEPOBS_ERR_VALIDATION, self.rules_with_obligations, self.total_rules
            ));
        }

        if self.generated_at.trim().is_empty() {
            return Err(format!(
                "{}: generated_at cannot be empty",
                PEPOBS_ERR_VALIDATION
            ));
        }

        Ok(())
    }

    /// Serializes report to pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: serialization failed: {}", PEPOBS_ERR_VALIDATION, e))
    }

    /// Deserializes report from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        serde_json::from_str(json_str)
            .map_err(|e| format!("{}: deserialization failed: {}", PEPOBS_ERR_VALIDATION, e))
    }
}
