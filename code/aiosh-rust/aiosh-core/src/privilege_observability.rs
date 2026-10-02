//! Privilege Escalation Prevention Observability Subsystem (PRIVESCOBS1..PRIVESCOBS6).
//!
//! Aggregates point-in-time metrics, active context tiers, elevation outcomes,
//! and security policy states for operators and automated agents.

use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::audit::AuditRing;
use crate::privilege_service::PrivilegeService;

/// Error code: Telemetry validation or invariant failure.
pub const PRIVESCOBS_ERR_VALIDATION: &str = "PRIVESCOBS_ERR_VALIDATION";

/// Maximum length for sanitized telemetry text.
pub const MAX_TELEMETRY_TEXT_LEN: usize = 256;

/// Maximum number of outcome categories collected in telemetry distribution.
pub const MAX_OUTCOME_DISTRIBUTION_ENTRIES: usize = 128;

/// Maximum number of audit ring items inspected during report generation.
pub const MAX_AUDIT_LOG_TAIL_ITEMS: i64 = 1000;

/// Sanitizes a string for safe inclusion in telemetry outputs.
pub fn sanitize_telemetry_text(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(MAX_TELEMETRY_TEXT_LEN)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Comprehensive observability and telemetry report for Privilege Escalation Prevention.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrivilegeObservabilityReport {
    pub generated_at_utc: String,
    pub total_registered_actors: usize,
    pub active_contexts_count: usize,
    pub actors_by_tier: HashMap<String, usize>,
    pub total_transitions_recorded: usize,
    pub transitions_by_outcome: HashMap<String, usize>,
    pub policy_mode: String,
    pub is_healthy: bool,
}

impl PrivilegeObservabilityReport {
    /// Generates a point-in-time observability report by inspecting the service and audit ring.
    pub fn generate(service: &PrivilegeService, ring: Option<&AuditRing>) -> Result<Self, String> {
        let actors = service.list_actors();
        let active_count = service.active_contexts_count();
        let policy = service.policy();

        let mut actors_by_tier = HashMap::new();
        for actor in &actors {
            if let Some(ctx) = service.get_context(actor) {
                let tier_str = ctx.active_level.as_str().to_string();
                if actors_by_tier.len() < MAX_OUTCOME_DISTRIBUTION_ENTRIES {
                    *actors_by_tier.entry(tier_str).or_insert(0) += 1;
                }
            }
        }

        let mut total_transitions = 0;
        let mut by_outcome = HashMap::new();

        if let Some(r) = ring {
            if let Ok(rows) = r.tail(MAX_AUDIT_LOG_TAIL_ITEMS) {
                for row in rows {
                    if row.tool == "privilege" || row.tool.starts_with("aios.privilege") {
                        total_transitions += 1;
                        let outcome_clean = sanitize_telemetry_text(&row.outcome);
                        if by_outcome.len() < MAX_OUTCOME_DISTRIBUTION_ENTRIES {
                            *by_outcome.entry(outcome_clean).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        let is_healthy = active_count <= service.max_contexts();

        let report = Self {
            generated_at_utc: Utc::now().to_rfc3339(),
            total_registered_actors: actors.len(),
            active_contexts_count: active_count,
            actors_by_tier,
            total_transitions_recorded: total_transitions,
            transitions_by_outcome: by_outcome,
            policy_mode: format!("{:?}", policy.mode).to_lowercase(),
            is_healthy,
        };

        report.validate()?;
        Ok(report)
    }

    /// Validates the report fields against formatting rules and cardinality bounds.
    pub fn validate(&self) -> Result<(), String> {
        if self.generated_at_utc.trim().is_empty() {
            return Err(format!("{}: generated_at_utc timestamp cannot be empty", PRIVESCOBS_ERR_VALIDATION));
        }

        if self.active_contexts_count > self.total_registered_actors {
            return Err(format!("{}: active_contexts_count ({}) cannot exceed total_registered_actors ({})",
                PRIVESCOBS_ERR_VALIDATION, self.active_contexts_count, self.total_registered_actors));
        }

        if self.actors_by_tier.len() > MAX_OUTCOME_DISTRIBUTION_ENTRIES {
            return Err(format!("{}: actors_by_tier distribution exceeds {}", PRIVESCOBS_ERR_VALIDATION, MAX_OUTCOME_DISTRIBUTION_ENTRIES));
        }

        if self.transitions_by_outcome.len() > MAX_OUTCOME_DISTRIBUTION_ENTRIES {
            return Err(format!("{}: transitions_by_outcome distribution exceeds {}", PRIVESCOBS_ERR_VALIDATION, MAX_OUTCOME_DISTRIBUTION_ENTRIES));
        }

        Ok(())
    }
}
