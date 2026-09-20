//! Capability Observability Subsystem (CAPOBS1..CAPOBS6) for AIOS Security Kernel.
//!
//! Provides point-in-time state aggregation, lineage depth metrics, quota consumption,
//! scope/rights distributions, and health monitoring for the capability registry.

use std::collections::{HashMap, HashSet};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::capability::CapabilityScope;
use crate::capability_policy::CapabilityPolicyMode;
use crate::capability_service::CapabilityService;

/// Sanitizes a string for safe inclusion in telemetry outputs (removes control characters, trims, limits to 256 chars).
pub fn sanitize_telemetry_text(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(256)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Comprehensive observability and telemetry report for the Capability Model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityObservabilityReport {
    pub total_capabilities: usize,
    pub active_capabilities: usize,
    pub revoked_capabilities: usize,
    pub expired_capabilities: usize,
    pub root_capabilities: usize,
    pub attenuated_capabilities: usize,
    pub max_derivation_depth: usize,
    pub unique_subjects_count: usize,
    pub unique_issuers_count: usize,
    pub total_invocations_consumed: u64,
    pub total_bytes_consumed: u64,
    pub capabilities_by_scope_type: HashMap<String, usize>,
    pub capabilities_by_right: HashMap<String, usize>,
    pub policy_mode: CapabilityPolicyMode,
    pub max_capabilities_capacity: usize,
    pub capacity_utilization_percent: u8,
    pub is_healthy: bool,
    pub generated_at: String,
}

impl CapabilityObservabilityReport {
    /// Generates a comprehensive observability report from a capability service.
    pub fn generate(service: &CapabilityService, timestamp: &str) -> Self {
        let now = Utc::now();
        let generated_at = if timestamp.trim().is_empty() {
            now.to_rfc3339()
        } else {
            sanitize_telemetry_text(timestamp)
        };

        let mut active_capabilities = 0;
        let mut revoked_capabilities = 0;
        let mut expired_capabilities = 0;
        let mut root_capabilities = 0;
        let mut attenuated_capabilities = 0;
        let mut max_derivation_depth = 0;
        let mut total_invocations_consumed: u64 = 0;
        let mut total_bytes_consumed: u64 = 0;

        let mut unique_subjects = HashSet::new();
        let mut unique_issuers = HashSet::new();
        let mut capabilities_by_scope_type = HashMap::new();
        let mut capabilities_by_right = HashMap::new();

        let capabilities = service.capabilities();
        let total_capabilities = capabilities.len();

        for cap in capabilities.values() {
            if cap.revoked {
                revoked_capabilities += 1;
            } else if cap.check_validity_at(now).is_err() {
                expired_capabilities += 1;
            } else {
                active_capabilities += 1;
            }

            if cap.parent_id.is_none() {
                root_capabilities += 1;
            } else {
                attenuated_capabilities += 1;
            }

            let depth = service.get_derivation_depth(&cap.id);
            if depth > max_derivation_depth {
                max_derivation_depth = depth;
            }

            total_invocations_consumed = total_invocations_consumed
                .saturating_add(cap.constraints.current_invocations);
            total_bytes_consumed = total_bytes_consumed
                .saturating_add(cap.constraints.consumed_bytes);

            unique_subjects.insert(cap.subject.clone());
            unique_issuers.insert(cap.issuer.clone());

            let scope_str = match &cap.scope {
                CapabilityScope::Filesystem { .. } => "filesystem",
                CapabilityScope::Network { .. } => "network",
                CapabilityScope::Tool { .. } => "tool",
                CapabilityScope::Process { .. } => "process",
                CapabilityScope::Ipc { .. } => "ipc",
                CapabilityScope::System { .. } => "system",
            };
            *capabilities_by_scope_type.entry(scope_str.to_string()).or_insert(0) += 1;

            for right in &cap.rights {
                *capabilities_by_right.entry(right.to_string()).or_insert(0) += 1;
            }
        }

        let max_capabilities_capacity = service.config().max_capabilities;
        let capacity_utilization_percent = if max_capabilities_capacity > 0 {
            ((total_capabilities * 100) / max_capabilities_capacity).min(100) as u8
        } else {
            0
        };

        let policy_mode = service.policy().mode;
        let is_healthy = capacity_utilization_percent < 95
            && max_derivation_depth <= service.policy().max_attenuation_depth;

        Self {
            total_capabilities,
            active_capabilities,
            revoked_capabilities,
            expired_capabilities,
            root_capabilities,
            attenuated_capabilities,
            max_derivation_depth,
            unique_subjects_count: unique_subjects.len(),
            unique_issuers_count: unique_issuers.len(),
            total_invocations_consumed,
            total_bytes_consumed,
            capabilities_by_scope_type,
            capabilities_by_right,
            policy_mode,
            max_capabilities_capacity,
            capacity_utilization_percent,
            is_healthy,
            generated_at,
        }
    }

    /// Serializes the report to pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}
