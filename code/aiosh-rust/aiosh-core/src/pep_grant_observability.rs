//! PEP Grant Lifecycle Observability Subsystem (T-02271..T-02280).
//!
//! Provides point-in-time telemetry aggregation, state distributions, delegation hierarchy
//! metrics, capacity utilization monitoring, and health diagnostics for capability grants.

use std::collections::{HashMap, HashSet};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::capability::CapabilityScope;
use crate::pep_grant::PepGrantState;
use crate::pep_grant_service::{PepGrantService, MAX_GRANTS_IN_SERVICE};

/// Health threshold percentage: utilization at or above 90% transitions health to false.
pub const PEP_GRANT_HEALTH_UTILIZATION_THRESHOLD: u8 = 90;

/// Error code: Telemetry or report invariant validation error.
pub const PEPOBS_GRANT_ERR_VALIDATION: &str = "PEPOBS_GRANT_ERR_VALIDATION";

/// Sanitizes a string for safe inclusion in grant telemetry reports.
pub fn sanitize_grant_telemetry_text(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(256)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Comprehensive observability and telemetry report for the Grant Lifecycle subsystem.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PepGrantObservabilityReport {
    /// Total number of grants in the service.
    pub total_grants: usize,
    /// Number of requested grants pending authorization.
    pub requested_grants: usize,
    /// Number of active grants.
    pub active_grants: usize,
    /// Number of suspended grants.
    pub suspended_grants: usize,
    /// Number of revoked grants.
    pub revoked_grants: usize,
    /// Number of expired grants.
    pub expired_grants: usize,
    /// Number of root grants without a parent.
    pub root_grants_count: usize,
    /// Number of derived child grants.
    pub derived_grants_count: usize,
    /// Distinct subjects receiving grants.
    pub unique_subjects_count: usize,
    /// Distinct issuing authorities.
    pub unique_issuers_count: usize,
    /// Counts of grants broken down by lifecycle state string.
    pub grants_by_state: HashMap<String, usize>,
    /// Counts of grants containing specific capability rights.
    pub grants_by_right: HashMap<String, usize>,
    /// Counts of grants grouped by scope type.
    pub grants_by_scope_type: HashMap<String, usize>,
    /// Maximum capacity of grants in the service.
    pub capacity_limit: usize,
    /// Current capacity utilization as a percentage (0..=100).
    pub capacity_utilization_percent: u8,
    /// Composite health indicator: healthy if utilization < 90%.
    pub is_healthy: bool,
    /// RFC 3339 timestamp when the report was compiled.
    pub generated_at: String,
}

impl PepGrantObservabilityReport {
    /// Validates internal consistency invariants of the observability report.
    pub fn validate(&self) -> Result<(), String> {
        let sum_states = self.requested_grants + self.active_grants + self.suspended_grants + self.revoked_grants + self.expired_grants;
        if sum_states != self.total_grants {
            return Err(format!(
                "{}: state counts sum ({}) does not match total_grants ({})",
                PEPOBS_GRANT_ERR_VALIDATION, sum_states, self.total_grants
            ));
        }

        let sum_hierarchy = self.root_grants_count + self.derived_grants_count;
        if sum_hierarchy != self.total_grants {
            return Err(format!(
                "{}: hierarchy counts sum ({}) does not match total_grants ({})",
                PEPOBS_GRANT_ERR_VALIDATION, sum_hierarchy, self.total_grants
            ));
        }

        if self.capacity_limit == 0 {
            return Err(format!("{}: capacity_limit must be > 0", PEPOBS_GRANT_ERR_VALIDATION));
        }

        let expected_util = if self.capacity_limit > 0 {
            let pct = (self.total_grants as u64 * 100) / (self.capacity_limit as u64);
            pct.min(100) as u8
        } else {
            100
        };

        if self.capacity_utilization_percent != expected_util {
            return Err(format!(
                "{}: capacity utilization mismatch: recorded {}, calculated {}",
                PEPOBS_GRANT_ERR_VALIDATION, self.capacity_utilization_percent, expected_util
            ));
        }

        let expected_health = self.capacity_utilization_percent < PEP_GRANT_HEALTH_UTILIZATION_THRESHOLD;
        if self.is_healthy != expected_health {
            return Err(format!(
                "{}: is_healthy ({}) does not match threshold rule (expected {})",
                PEPOBS_GRANT_ERR_VALIDATION, self.is_healthy, expected_health
            ));
        }

        Ok(())
    }
}

impl PepGrantService {
    /// Generates a point-in-time observability report for the grant service.
    pub fn generate_observability_report(&self) -> PepGrantObservabilityReport {
        let grants = self.list_grants();
        let total_grants = grants.len();

        let mut requested_grants = 0;
        let mut active_grants = 0;
        let mut suspended_grants = 0;
        let mut revoked_grants = 0;
        let mut expired_grants = 0;
        let mut root_grants_count = 0;
        let mut derived_grants_count = 0;

        let mut unique_subjects = HashSet::new();
        let mut unique_issuers = HashSet::new();

        let mut grants_by_state = HashMap::new();
        let mut grants_by_right = HashMap::new();
        let mut grants_by_scope_type = HashMap::new();

        for grant in &grants {
            match grant.state {
                PepGrantState::Requested => requested_grants += 1,
                PepGrantState::Active => active_grants += 1,
                PepGrantState::Suspended => suspended_grants += 1,
                PepGrantState::Revoked => revoked_grants += 1,
                PepGrantState::Expired => expired_grants += 1,
            }
            *grants_by_state.entry(format!("{:?}", grant.state).to_ascii_lowercase()).or_insert(0) += 1;

            if grant.parent_grant_id.is_none() {
                root_grants_count += 1;
            } else {
                derived_grants_count += 1;
            }

            unique_subjects.insert(sanitize_grant_telemetry_text(&grant.subject));
            unique_issuers.insert(sanitize_grant_telemetry_text(&grant.issuer));

            for right in &grant.rights {
                *grants_by_right.entry(format!("{:?}", right).to_ascii_lowercase()).or_insert(0) += 1;
            }

            let scope_name = match &grant.scope {
                CapabilityScope::Filesystem { .. } => "filesystem",
                CapabilityScope::Network { .. } => "network",
                CapabilityScope::Ipc { .. } => "ipc",
                CapabilityScope::System { .. } => "system",
                CapabilityScope::Tool { .. } => "tool",
                CapabilityScope::Process { .. } => "process",
            };
            *grants_by_scope_type.entry(scope_name.to_string()).or_insert(0) += 1;
        }

        let capacity_limit = MAX_GRANTS_IN_SERVICE;
        let capacity_utilization_percent = if capacity_limit > 0 {
            let pct = (total_grants as u64 * 100) / (capacity_limit as u64);
            pct.min(100) as u8
        } else {
            100
        };

        let is_healthy = capacity_utilization_percent < PEP_GRANT_HEALTH_UTILIZATION_THRESHOLD;

        PepGrantObservabilityReport {
            total_grants,
            requested_grants,
            active_grants,
            suspended_grants,
            revoked_grants,
            expired_grants,
            root_grants_count,
            derived_grants_count,
            unique_subjects_count: unique_subjects.len(),
            unique_issuers_count: unique_issuers.len(),
            grants_by_state,
            grants_by_right,
            grants_by_scope_type,
            capacity_limit,
            capacity_utilization_percent,
            is_healthy,
            generated_at: Utc::now().to_rfc3339(),
        }
    }
}

impl crate::pep_grant::PepGrantStore {
    /// Generates an observability report directly from the stored grants.
    pub fn generate_observability_report(&self) -> PepGrantObservabilityReport {
        let mut service = PepGrantService::new();
        for grant in self.grants.values() {
            let _ = service.issue_grant(grant.clone());
        }
        service.generate_observability_report()
    }
}
