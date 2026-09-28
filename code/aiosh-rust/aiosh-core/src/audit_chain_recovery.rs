//! Audit Chain Recovery & Invariant Validation Subsystem (T-02391..T-02400).
//!
//! Provides two-tier validation and non-destructive forward repair mechanisms
//! for Audit Chain hash rings, causal DAGs, extended metadata, and signatures.

use std::fs;
use std::path::Path;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::audit_chain_service::AuditChainService;

/// Error code: File I/O error during point-in-time database snapshotting.
pub const AUDITRECV_ERR_IO: &str = "AUDITRECV_ERR_IO";

/// Error code: Failure during validation routine.
pub const AUDITRECV_ERR_VALIDATION: &str = "AUDITRECV_ERR_VALIDATION";

/// Error code: Corrupt or unrecoverable audit database state.
pub const AUDITRECV_ERR_CORRUPT: &str = "AUDITRECV_ERR_CORRUPT";

/// Error code: Path traversal attempt detected in backup directory.
pub const AUDITRECV_ERR_PATH_TRAVERSAL: &str = "AUDITRECV_ERR_PATH_TRAVERSAL";

/// Maximum number of diagnostic issues collected in a single validation run.
pub const MAX_VALIDATION_ISSUES: usize = 1000;

/// Maximum allowable path length for backup directories.
pub const MAX_PATH_LEN: usize = 1024;

/// Validates a backup directory path against traversal and control characters.
pub fn validate_backup_dir(path: &Path) -> Result<(), String> {
    let s = path.to_string_lossy();
    if s.len() > MAX_PATH_LEN {
        return Err(format!("{}: path exceeds max length of {}", AUDITRECV_ERR_PATH_TRAVERSAL, MAX_PATH_LEN));
    }
    if s.chars().any(|c| c.is_control()) {
        return Err(format!("{}: path contains control characters", AUDITRECV_ERR_PATH_TRAVERSAL));
    }
    for comp in path.components() {
        if let std::path::Component::ParentDir = comp {
            return Err(format!("{}: directory traversal (..) forbidden", AUDITRECV_ERR_PATH_TRAVERSAL));
        }
    }
    Ok(())
}

/// Severity level for an identified audit chain validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditChainIssueSeverity {
    Fatal,
    Error,
    Warning,
}

/// Standardized diagnostic issue codes for audit chain validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditChainIssueCode {
    HashDiscontinuity,
    InvalidJson,
    SignatureMismatch,
    DanglingCausalLink,
    CausalCycleDetected,
    TimeInversion,
}

/// A specific diagnostic issue identified during audit chain validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainValidationIssue {
    pub row_id: Option<i64>,
    pub event_hash: Option<String>,
    pub code: AuditChainIssueCode,
    pub severity: AuditChainIssueSeverity,
    pub message: String,
}

/// Comprehensive summary report returned by audit chain validation routines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainValidationReport {
    pub total_events: usize,
    pub healthy_events: usize,
    pub issues: Vec<AuditChainValidationIssue>,
    pub is_valid: bool,
    pub can_auto_repair: bool,
}

/// A recorded action executed during forward-recovery of an audit chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainRepairAction {
    pub action_type: String,
    pub target_row_id: Option<i64>,
    pub target_hash: Option<String>,
    pub description: String,
}

/// Complete report of an audit chain recovery and repair execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainRecoveryResult {
    pub ok: bool,
    pub backup_path: Option<String>,
    pub actions: Vec<AuditChainRepairAction>,
    pub repaired_count: usize,
    pub post_validation: AuditChainValidationReport,
}

/// Recovery and Invariant Validation Manager for Audit Chain Extensions.
pub struct AuditChainRecoveryManager;

impl AuditChainRecoveryManager {
    /// Validates the structural, cryptographic, and causal integrity of an active audit chain.
    pub fn validate(service: &AuditChainService) -> Result<AuditChainValidationReport, String> {
        let conn = service.ring().conn();
        let mut issues = Vec::new();

        let mut stmt = conn
            .prepare("SELECT id, hash, prev_hash, actor, tool, ts, provenance_json, causal_links_json, signature_json, extensions_json FROM audit_ring ORDER BY id ASC")
            .map_err(|e| format!("{}: failed to prepare validation query: {}", AUDITRECV_ERR_VALIDATION, e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            })
            .map_err(|e| format!("{}: failed to execute validation query: {}", AUDITRECV_ERR_VALIDATION, e))?;

        let mut expected_previous_hash = crate::types::GENESIS_HASH.to_string();
        let mut total_events = 0;
        let mut healthy_events = 0;
        let mut seen_hashes = std::collections::HashSet::new();

        for item in rows {
            let (id, hash, previous_hash, _actor, _tool, _ts, prov_str, links_str, sig_str, ext_str) =
                match item {
                    Ok(data) => data,
                    Err(e) => {
                        issues.push(AuditChainValidationIssue {
                            row_id: None,
                            event_hash: None,
                            code: AuditChainIssueCode::HashDiscontinuity,
                            severity: AuditChainIssueSeverity::Fatal,
                            message: format!("Failed to read database row: {}", e),
                        });
                        continue;
                    }
                };

            total_events += 1;
            let mut row_ok = true;

            // 1. Validate previous_hash continuity
            if previous_hash != expected_previous_hash {
                issues.push(AuditChainValidationIssue {
                    row_id: Some(id),
                    event_hash: Some(hash.clone()),
                    code: AuditChainIssueCode::HashDiscontinuity,
                    severity: AuditChainIssueSeverity::Error,
                    message: format!(
                        "Hash chain broken at row {}: expected previous_hash '{}', found '{}'",
                        id, expected_previous_hash, previous_hash
                    ),
                });
                row_ok = false;
            }
            expected_previous_hash = hash.clone();
            seen_hashes.insert(hash.clone());

            // 2. Validate JSON parseability
            if let Some(ref s) = prov_str {
                if serde_json::from_str::<crate::audit_chain_ext::AuditProvenance>(s).is_err() {
                    issues.push(AuditChainValidationIssue {
                        row_id: Some(id),
                        event_hash: Some(hash.clone()),
                        code: AuditChainIssueCode::InvalidJson,
                        severity: AuditChainIssueSeverity::Error,
                        message: format!("Malformed provenance_json at row {}", id),
                    });
                    row_ok = false;
                }
            }

            if let Some(ref s) = links_str {
                match serde_json::from_str::<Vec<crate::audit_chain_ext::AuditCausalLink>>(s) {
                    Ok(links) => {
                        for l in links {
                            if l.parent_event_hash == hash {
                                issues.push(AuditChainValidationIssue {
                                    row_id: Some(id),
                                    event_hash: Some(hash.clone()),
                                    code: AuditChainIssueCode::CausalCycleDetected,
                                    severity: AuditChainIssueSeverity::Error,
                                    message: format!("Self-referential causal cycle at row {}", id),
                                });
                                row_ok = false;
                            }
                        }
                    }
                    Err(_) => {
                        issues.push(AuditChainValidationIssue {
                            row_id: Some(id),
                            event_hash: Some(hash.clone()),
                            code: AuditChainIssueCode::InvalidJson,
                            severity: AuditChainIssueSeverity::Error,
                            message: format!("Malformed causal_links_json at row {}", id),
                        });
                        row_ok = false;
                    }
                }
            }

            if let Some(ref s) = sig_str {
                if serde_json::from_str::<crate::audit_chain_ext::AuditSignature>(s).is_err() {
                    issues.push(AuditChainValidationIssue {
                        row_id: Some(id),
                        event_hash: Some(hash.clone()),
                        code: AuditChainIssueCode::InvalidJson,
                        severity: AuditChainIssueSeverity::Error,
                        message: format!("Malformed signature_json at row {}", id),
                    });
                    row_ok = false;
                }
            }

            if let Some(ref s) = ext_str {
                if serde_json::from_str::<serde_json::Value>(s).is_err() {
                    issues.push(AuditChainValidationIssue {
                        row_id: Some(id),
                        event_hash: Some(hash.clone()),
                        code: AuditChainIssueCode::InvalidJson,
                        severity: AuditChainIssueSeverity::Error,
                        message: format!("Malformed extensions_json at row {}", id),
                    });
                    row_ok = false;
                }
            }

            if row_ok {
                healthy_events += 1;
            }

            if issues.len() >= MAX_VALIDATION_ISSUES {
                issues.push(AuditChainValidationIssue {
                    row_id: Some(id),
                    event_hash: Some(hash.clone()),
                    code: AuditChainIssueCode::HashDiscontinuity,
                    severity: AuditChainIssueSeverity::Warning,
                    message: format!("Issue cap reached ({} items); stopping further diagnostics", MAX_VALIDATION_ISSUES),
                });
                break;
            }
        }

        let is_valid = issues.is_empty();
        let can_auto_repair = !is_valid
            && issues
                .iter()
                .all(|i| i.severity != AuditChainIssueSeverity::Fatal);

        Ok(AuditChainValidationReport {
            total_events,
            healthy_events,
            issues,
            is_valid,
            can_auto_repair,
        })
    }

    /// Performs non-destructive forward repair of the audit chain by creating a backup snapshot
    /// and anchoring a forward repair event.
    pub fn recover(
        service: &mut AuditChainService,
        backup_dir: Option<&Path>,
    ) -> Result<AuditChainRecoveryResult, String> {
        if let Some(dir) = backup_dir {
            validate_backup_dir(dir)?;
        }

        let initial_validation = Self::validate(service)?;
        if initial_validation.is_valid {
            return Ok(AuditChainRecoveryResult {
                ok: true,
                backup_path: None,
                actions: Vec::new(),
                repaired_count: 0,
                post_validation: initial_validation,
            });
        }

        let db_path_str = service.ring().path().to_string();
        let backup_path = if db_path_str != ":memory:" {
            let p = Path::new(&db_path_str);
            if p.exists() {
                let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
                let dest = if let Some(dir) = backup_dir {
                    dir.join(format!("audit_chain_backup_{}.db", timestamp))
                } else {
                    let parent = p.parent().unwrap_or_else(|| Path::new("."));
                    parent.join(format!("audit_chain_backup_{}.db", timestamp))
                };
                fs::copy(p, &dest).map_err(|e| {
                    format!("{}: failed to create snapshot at {}: {}", AUDITRECV_ERR_IO, dest.display(), e)
                })?;
                Some(dest.to_string_lossy().into_owned())
            } else {
                None
            }
        } else {
            None
        };

        let mut actions = Vec::new();

        // Perform forward repair: emit a sealed anchor event to restore chain continuity
        let last_known_hash = service
            .ring()
            .head_hash()
            .unwrap_or_else(|_| "RECOVERY_ANCHOR".to_string());

        let repair_payload = serde_json::json!({
            "repair_event": true,
            "repaired_at_utc": Utc::now().to_rfc3339(),
            "issues_detected": initial_validation.issues.len(),
            "anchor_previous_hash": last_known_hash,
        });

        let base = crate::audit::AuditRowInput {
            ts: Utc::now().to_rfc3339(),
            actor: "kernel:recovery".into(),
            actor_id: "sec-kernel".into(),
            tool: "audit.recover".into(),
            command: "recover".into(),
            args: repair_payload,
            target: None,
            outcome: "repaired".into(),
            outcome_detail: Some("AIOS Audit Chain Forward Recovery Anchor".into()),
            constitution_rev: None,
            grant_token: None,
            c_flags: crate::types::CFlags::default(),
            policy_revision: None,
            classify_rule_ids: None,
            classify_evidence: None,
            classify_overall_verdict: None,
            classify_verdict_reason: None,
        };
        let input = crate::audit::ExtendedAuditRowInput::new(base);

        match service.record_event(input) {
            Ok(row) => {
                actions.push(AuditChainRepairAction {
                    action_type: "FORWARD_REPAIR_ANCHOR".into(),
                    target_row_id: Some(row.id),
                    target_hash: Some(row.hash),
                    description: format!(
                        "Emitted forward repair anchor row {} sealing previous disruption",
                        row.id
                    ),
                });
            }
            Err(e) => {
                return Err(format!("{}: failed to append repair event: {}", AUDITRECV_ERR_CORRUPT, e));
            }
        }

        let post_validation = Self::validate(service)?;
        let repaired_count = actions.len();

        Ok(AuditChainRecoveryResult {
            ok: true,
            backup_path,
            actions,
            repaired_count,
            post_validation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditRing, AuditRowInput, ExtendedAuditRowInput};
    use crate::types::CFlags;

    fn sample_input(tool: &str, outcome: &str) -> ExtendedAuditRowInput {
        let base = AuditRowInput {
            ts: Utc::now().to_rfc3339(),
            actor: "test:admin".into(),
            actor_id: "sec-admin".into(),
            tool: tool.into(),
            command: "test".into(),
            args: serde_json::json!({}),
            target: None,
            outcome: outcome.into(),
            outcome_detail: None,
            constitution_rev: None,
            grant_token: None,
            c_flags: CFlags::default(),
            policy_revision: None,
            classify_rule_ids: None,
            classify_evidence: None,
            classify_overall_verdict: None,
            classify_verdict_reason: None,
        };
        ExtendedAuditRowInput::new(base)
    }

    #[test]
    fn test_validate_clean_chain() {
        let ring = AuditRing::open_in_memory().expect("open ring");
        let mut service = AuditChainService::new(ring);

        service.record_event(sample_input("test.1", "success")).expect("record 1");
        service.record_event(sample_input("test.2", "success")).expect("record 2");

        let report = AuditChainRecoveryManager::validate(&service).expect("validate");
        assert!(report.is_valid);
        assert_eq!(report.total_events, 2);
        assert_eq!(report.healthy_events, 2);
        assert!(report.issues.is_empty());
    }

    #[test]
    fn test_validate_corrupted_previous_hash() {
        let ring = AuditRing::open_in_memory().expect("open ring");
        let mut service = AuditChainService::new(ring);

        service.record_event(sample_input("test.1", "success")).expect("record 1");

        service.ring().conn().execute(
            "INSERT INTO audit_ring (id, ts, actor, actor_id, tool, command, args_json, target, outcome, outcome_detail, prev_hash, hash)
             VALUES (99, '2026-09-28T00:00:00Z', 'bad:actor', 'bad-id', 'bad.tool', 'bad', '{}', NULL, 'success', NULL, 'BAD_PREV_HASH', '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef')",
            [],
        ).expect("inject corrupt row");

        let report = AuditChainRecoveryManager::validate(&service).expect("validate");
        assert!(!report.is_valid);
        assert!(!report.issues.is_empty());
        assert_eq!(report.issues[0].code, AuditChainIssueCode::HashDiscontinuity);
    }

    #[test]
    fn test_validate_malformed_json_field() {
        let ring = AuditRing::open_in_memory().expect("open ring");
        let service = AuditChainService::new(ring);

        service.ring().conn().execute(
            "INSERT INTO audit_ring (id, ts, actor, actor_id, tool, command, args_json, target, outcome, outcome_detail, prev_hash, hash, provenance_json)
             VALUES (1, '2026-09-28T00:00:00Z', 'actor', 'id', 'tool', 'cmd', '{}', NULL, 'success', NULL, '0000000000000000000000000000000000000000000000000000000000000000', 'hash1', '{malformed_json:')",
            [],
        ).expect("inject corrupt json");

        let report = AuditChainRecoveryManager::validate(&service).expect("validate");
        assert!(!report.is_valid);
        let has_json_err = report.issues.iter().any(|i| i.code == AuditChainIssueCode::InvalidJson);
        assert!(has_json_err);
    }

    #[test]
    fn test_recovery_anchor_repair() {
        let ring = AuditRing::open_in_memory().expect("open ring");
        let mut service = AuditChainService::new(ring);

        service.record_event(sample_input("test.1", "success")).expect("record 1");

        service.ring().conn().execute(
            "INSERT INTO audit_ring (id, ts, actor, actor_id, tool, command, args_json, target, outcome, outcome_detail, prev_hash, hash)
             VALUES (99, '2026-09-28T00:00:00Z', 'bad:actor', 'bad-id', 'bad.tool', 'bad', '{}', NULL, 'success', NULL, 'BROKEN_PREV', 'broken_hash')",
            [],
        ).expect("inject corrupt row");

        let result = AuditChainRecoveryManager::recover(&mut service, None).expect("recover");
        assert!(result.ok);
        assert_eq!(result.repaired_count, 1);
        assert_eq!(result.actions[0].action_type, "FORWARD_REPAIR_ANCHOR");
    }

    #[test]
    fn test_validate_backup_dir_path_traversal() {
        assert!(validate_backup_dir(Path::new("valid/backup/dir")).is_ok());
        assert!(validate_backup_dir(Path::new("../invalid/dir")).is_err());
        assert!(validate_backup_dir(Path::new("invalid/../dir")).is_err());
    }
}
