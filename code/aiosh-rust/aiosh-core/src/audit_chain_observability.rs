//! Audit Chain Extensions Observability Subsystem (AUDITOBS1..AUDITOBS6).
//!
//! Provides point-in-time telemetry aggregation, cardinality metrics,
//! cryptographic signature statistics, and database health diagnostics.

use std::collections::HashMap;
use std::fs;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::audit_chain_service::AuditChainService;

/// Error code: Telemetry validation or invariant failure.
pub const AUDITOBS_ERR_VALIDATION: &str = "AUDITOBS_ERR_VALIDATION";

/// Error code: Database query error during telemetry aggregation.
pub const AUDITOBS_ERR_QUERY: &str = "AUDITOBS_ERR_QUERY";

/// Error code: File I/O error retrieving storage metrics.
pub const AUDITOBS_ERR_IO: &str = "AUDITOBS_ERR_IO";

/// Maximum length for sanitized telemetry text.
pub const MAX_TELEMETRY_TEXT_LEN: usize = 256;

/// Maximum number of outcome categories collected in telemetry distribution.
pub const MAX_OUTCOME_DISTRIBUTION_ENTRIES: usize = 128;

/// Maximum tracked distinct items for in-memory session/trace sets.
pub const MAX_TRACKED_CARDINALITY_ITEMS: usize = 100_000;

/// Sanitizes a string for safe inclusion in telemetry outputs (removes control characters, trims, limits to 256 chars).
pub fn sanitize_telemetry_text(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(MAX_TELEMETRY_TEXT_LEN)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Comprehensive observability and telemetry report for the Audit Chain Extensions subsystem.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditChainObservabilityReport {
    pub generated_at_utc: String,
    pub total_rows: usize,
    pub total_extended_rows: usize,
    pub total_causal_links: usize,
    pub total_signed_events: usize,
    pub unique_actors_count: usize,
    pub unique_tools_count: usize,
    pub unique_sessions_count: usize,
    pub unique_traces_count: usize,
    pub outcomes_by_type: HashMap<String, usize>,
    pub active_policy_mode: String,
    pub db_file_bytes: u64,
    pub chain_integrity_checked: bool,
    pub chain_integrity_ok: bool,
    pub is_healthy: bool,
}

impl AuditChainObservabilityReport {
    /// Generates a point-in-time telemetry snapshot from an active `AuditChainService`.
    pub fn generate(service: &AuditChainService) -> Result<Self, String> {
        let conn = service.ring().conn();

        let total_rows: usize = conn
            .query_row("SELECT COUNT(*) FROM audit_ring", [], |r| r.get(0))
            .map_err(|e| format!("{}: failed to query total_rows: {}", AUDITOBS_ERR_QUERY, e))?;

        let total_extended_rows: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM audit_ring WHERE provenance_json IS NOT NULL OR causal_links_json IS NOT NULL OR signature_json IS NOT NULL OR extensions_json IS NOT NULL",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let unique_actors_count: usize = conn
            .query_row("SELECT COUNT(DISTINCT actor) FROM audit_ring", [], |r| r.get(0))
            .unwrap_or(0);

        let unique_tools_count: usize = conn
            .query_row("SELECT COUNT(DISTINCT tool) FROM audit_ring", [], |r| r.get(0))
            .unwrap_or(0);

        // Outcomes distribution (bounded to prevent high-cardinality state exhaustion)
        let mut outcomes_by_type = HashMap::new();
        if let Ok(mut stmt) = conn.prepare("SELECT outcome, COUNT(*) FROM audit_ring GROUP BY outcome ORDER BY COUNT(*) DESC LIMIT 128") {
            if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, usize>(1)?))) {
                for item in rows.flatten() {
                    if outcomes_by_type.len() < MAX_OUTCOME_DISTRIBUTION_ENTRIES {
                        outcomes_by_type.insert(sanitize_telemetry_text(&item.0), item.1);
                    }
                }
            }
        }

        // Causal links, signatures, sessions, traces
        let mut total_causal_links = 0;
        let mut total_signed_events = 0;
        let mut sessions_set = std::collections::HashSet::new();
        let mut traces_set = std::collections::HashSet::new();

        if let Ok(mut stmt) = conn.prepare("SELECT causal_links_json, signature_json, provenance_json FROM audit_ring WHERE causal_links_json IS NOT NULL OR signature_json IS NOT NULL OR provenance_json IS NOT NULL") {
            if let Ok(rows) = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, Option<String>>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            }) {
                for item in rows.flatten() {
                    if let Some(links_str) = item.0 {
                        if let Ok(links) = serde_json::from_str::<Vec<crate::audit_chain_ext::AuditCausalLink>>(&links_str) {
                            total_causal_links += links.len();
                        }
                    }
                    if let Some(sig_str) = item.1 {
                        if let Ok(sig) = serde_json::from_str::<crate::audit_chain_ext::AuditSignature>(&sig_str) {
                            if !sig.signature.is_empty() {
                                total_signed_events += 1;
                            }
                        }
                    }
                    if let Some(prov_str) = item.2 {
                        if let Ok(prov) = serde_json::from_str::<crate::audit_chain_ext::AuditProvenance>(&prov_str) {
                            if let Some(sid) = prov.session_id {
                                if sessions_set.len() < MAX_TRACKED_CARDINALITY_ITEMS {
                                    sessions_set.insert(sid);
                                }
                            }
                            if let Some(tid) = prov.trace_id {
                                if traces_set.len() < MAX_TRACKED_CARDINALITY_ITEMS {
                                    traces_set.insert(tid);
                                }
                            }
                        }
                    }
                }
            }
        }

        let db_file_bytes = fs::metadata(service.ring().path())
            .map(|m| m.len())
            .unwrap_or(0);

        let active_policy_mode = format!("{:?}", service.policy().mode).to_lowercase();

        // Integrity check
        let (chain_integrity_checked, chain_integrity_ok) = match service.verify_integrity() {
            Ok(report) => (true, report.ok),
            Err(_) => (false, false),
        };

        let is_healthy = chain_integrity_ok || total_rows == 0;

        let report = Self {
            generated_at_utc: Utc::now().to_rfc3339(),
            total_rows,
            total_extended_rows,
            total_causal_links,
            total_signed_events,
            unique_actors_count,
            unique_tools_count,
            unique_sessions_count: sessions_set.len(),
            unique_traces_count: traces_set.len(),
            outcomes_by_type,
            active_policy_mode,
            db_file_bytes,
            chain_integrity_checked,
            chain_integrity_ok,
            is_healthy,
        };

        report.validate()?;
        Ok(report)
    }

    /// Validates report telemetry invariants.
    pub fn validate(&self) -> Result<(), String> {
        if self.generated_at_utc.trim().is_empty() {
            return Err(format!("{}: generated_at_utc cannot be empty", AUDITOBS_ERR_VALIDATION));
        }

        if self.total_extended_rows > self.total_rows {
            return Err(format!(
                "{}: total_extended_rows ({}) cannot exceed total_rows ({})",
                AUDITOBS_ERR_VALIDATION, self.total_extended_rows, self.total_rows
            ));
        }

        if self.unique_actors_count > self.total_rows {
            return Err(format!(
                "{}: unique_actors_count ({}) cannot exceed total_rows ({})",
                AUDITOBS_ERR_VALIDATION, self.unique_actors_count, self.total_rows
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditRing, AuditRowInput, ExtendedAuditRowInput};
    use crate::audit_chain_ext::{AuditCausalLink, AuditProvenance, AuditSignature};

    #[test]
    fn test_observability_sanitization() {
        let dirty = "  hello\x00\x07world\n  ";
        let clean = sanitize_telemetry_text(dirty);
        assert_eq!(clean, "helloworld");
    }

    #[test]
    fn test_observability_empty_ring() {
        let ring = AuditRing::open_in_memory().expect("open memory ring");
        let service = AuditChainService::new(ring);

        let report = AuditChainObservabilityReport::generate(&service).expect("generate report");
        assert_eq!(report.total_rows, 0);
        assert_eq!(report.total_extended_rows, 0);
        assert!(report.is_healthy);
        assert!(report.validate().is_ok());
    }

    #[test]
    fn test_observability_populated_ring() {
        let ring = AuditRing::open_in_memory().expect("open memory ring");
        let mut service = AuditChainService::new(ring);

        // Row 1: unsigned
        let mut base1 = AuditRowInput::default();
        base1.actor = "agent_obs".into();
        base1.tool = "tool.obs1".into();
        base1.outcome = "success".into();

        let mut input1 = ExtendedAuditRowInput::new(base1);
        input1.provenance = Some(AuditProvenance {
            session_id: Some("sess_1".into()),
            trace_id: Some("trace_1".into()),
            span_id: None,
            pep_grant_id: None,
            delegation_depth: 0,
        });

        let row1 = service.record_event(input1).expect("record row 1");

        // Row 2: with causal link and signature
        let mut base2 = AuditRowInput::default();
        base2.actor = "agent_obs".into();
        base2.tool = "tool.obs2".into();
        base2.outcome = "denied".into();

        let mut input2 = ExtendedAuditRowInput::new(base2);
        input2.causal_links = vec![AuditCausalLink::new(row1.hash.clone(), "caused_by")];
        input2.signature = Some(AuditSignature::new(
            "ed25519",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ));

        service.record_event(input2).expect("record row 2");

        let report = AuditChainObservabilityReport::generate(&service).expect("generate report");
        assert_eq!(report.total_rows, 2);
        assert_eq!(report.total_extended_rows, 2);
        assert_eq!(report.total_causal_links, 1);
        assert_eq!(report.total_signed_events, 1);
        assert_eq!(report.unique_actors_count, 1);
        assert_eq!(report.unique_tools_count, 2);
        assert_eq!(report.unique_sessions_count, 1);
        assert_eq!(report.unique_traces_count, 1);
        assert_eq!(report.outcomes_by_type.get("success"), Some(&1));
        assert_eq!(report.outcomes_by_type.get("denied"), Some(&1));
        assert!(report.chain_integrity_ok);
        assert!(report.is_healthy);
    }

    #[test]
    fn test_observability_validation_bounds() {
        let mut report = AuditChainObservabilityReport {
            generated_at_utc: "2026-09-28T12:00:00Z".into(),
            total_rows: 5,
            total_extended_rows: 10, // Invalid: > total_rows
            total_causal_links: 2,
            total_signed_events: 1,
            unique_actors_count: 2,
            unique_tools_count: 2,
            unique_sessions_count: 1,
            unique_traces_count: 1,
            outcomes_by_type: HashMap::new(),
            active_policy_mode: "enforcing".into(),
            db_file_bytes: 1024,
            chain_integrity_checked: true,
            chain_integrity_ok: true,
            is_healthy: true,
        };

        assert!(report.validate().is_err());
        report.total_extended_rows = 5;
        assert!(report.validate().is_ok());
    }
}
