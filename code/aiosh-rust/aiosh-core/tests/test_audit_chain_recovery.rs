//! Integration and standalone tests for Audit Chain Recovery & Invariant Validation (T-02394).

use chrono::Utc;
use aiosh_core::audit::{AuditRing, AuditRowInput, ExtendedAuditRowInput};
use aiosh_core::audit_chain_recovery::{
    AuditChainIssueCode, AuditChainRecoveryManager,
};
use aiosh_core::audit_chain_service::AuditChainService;
use aiosh_core::types::CFlags;

fn make_sample_row(tool: &str, outcome: &str) -> ExtendedAuditRowInput {
    let base = AuditRowInput {
        ts: Utc::now().to_rfc3339(),
        actor: "sec-admin".into(),
        actor_id: "sec-admin-01".into(),
        tool: tool.into(),
        command: "exec".into(),
        args: serde_json::json!({"action": "check"}),
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
fn test_integration_recovery_validate_healthy() {
    let ring = AuditRing::open_in_memory().expect("open ring");
    let mut service = AuditChainService::new(ring);

    for i in 1..=5 {
        service
            .record_event(make_sample_row(&format!("tool.{}", i), "success"))
            .expect("record event");
    }

    let report = AuditChainRecoveryManager::validate(&service).expect("validate");
    assert!(report.is_valid);
    assert_eq!(report.total_events, 5);
    assert_eq!(report.healthy_events, 5);
    assert!(report.issues.is_empty());
}

#[test]
fn test_integration_recovery_detect_discontinuity() {
    let ring = AuditRing::open_in_memory().expect("open ring");
    let mut service = AuditChainService::new(ring);

    service
        .record_event(make_sample_row("tool.valid", "success"))
        .expect("record event");

    // Manually corrupt sequence
    service.ring().conn().execute(
        "INSERT INTO audit_ring (id, ts, actor, actor_id, tool, command, args_json, target, outcome, outcome_detail, prev_hash, hash)
         VALUES (2, '2026-09-28T00:00:00Z', 'bad-actor', 'bad-id', 'bad.tool', 'run', '{}', NULL, 'success', NULL, 'INVALID_PREV', 'invalid_hash_1234567890')",
        [],
    ).expect("insert corrupt");

    let report = AuditChainRecoveryManager::validate(&service).expect("validate");
    assert!(!report.is_valid);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].code, AuditChainIssueCode::HashDiscontinuity);
}

#[test]
fn test_integration_recovery_forward_repair_execution() {
    let ring = AuditRing::open_in_memory().expect("open ring");
    let mut service = AuditChainService::new(ring);

    service
        .record_event(make_sample_row("tool.pre", "success"))
        .expect("record event");

    // Corrupt
    service.ring().conn().execute(
        "INSERT INTO audit_ring (id, ts, actor, actor_id, tool, command, args_json, target, outcome, outcome_detail, prev_hash, hash)
         VALUES (2, '2026-09-28T00:00:00Z', 'corrupt', 'corrupt', 'corrupt', 'corrupt', '{}', NULL, 'success', NULL, 'MISMATCH_PREV', 'corrupt_hash_val')",
        [],
    ).expect("insert corrupt");

    let recovery_result = AuditChainRecoveryManager::recover(&mut service, None).expect("recover");
    assert!(recovery_result.ok);
    assert_eq!(recovery_result.repaired_count, 1);
    assert_eq!(recovery_result.actions[0].action_type, "FORWARD_REPAIR_ANCHOR");
}
