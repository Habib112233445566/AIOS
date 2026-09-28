//! Standalone unit test suite for Audit Chain Observability (T-02375).

use aiosh_core::audit::{AuditRing, AuditRowInput, ExtendedAuditRowInput};
use aiosh_core::audit_chain_ext::AuditProvenance;
use aiosh_core::audit_chain_observability::{
    sanitize_telemetry_text, AuditChainObservabilityReport, AUDITOBS_ERR_VALIDATION,
};
use aiosh_core::audit_chain_service::AuditChainService;

#[test]
fn test_observability_empty_database_lifecycle() {
    let ring = AuditRing::open_in_memory().expect("open ring");
    let service = AuditChainService::new(ring);

    let report = AuditChainObservabilityReport::generate(&service).expect("report");
    assert_eq!(report.total_rows, 0);
    assert_eq!(report.total_extended_rows, 0);
    assert_eq!(report.total_causal_links, 0);
    assert_eq!(report.total_signed_events, 0);
    assert!(report.is_healthy);
    assert!(report.validate().is_ok());
}

#[test]
fn test_observability_multi_session_and_trace_aggregation() {
    let ring = AuditRing::open_in_memory().expect("open ring");
    let mut service = AuditChainService::new(ring);

    for i in 1..=5 {
        let mut base = AuditRowInput::default();
        base.actor = format!("worker_{}", i);
        base.tool = format!("task.exec_{}", i % 2);
        base.outcome = if i % 2 == 0 { "success".into() } else { "warning".into() };

        let mut input = ExtendedAuditRowInput::new(base);
        input.provenance = Some(AuditProvenance {
            session_id: Some(format!("session_{}", i % 3)),
            trace_id: Some(format!("trace_{}", i % 2)),
            span_id: None,
            pep_grant_id: None,
            delegation_depth: 0,
        });

        service.record_event(input).expect("record");
    }

    let report = AuditChainObservabilityReport::generate(&service).expect("report");
    assert_eq!(report.total_rows, 5);
    assert_eq!(report.total_extended_rows, 5);
    assert_eq!(report.unique_actors_count, 5);
    assert_eq!(report.unique_tools_count, 2);
    // Sessions: 1%3=1, 2%3=2, 3%3=0, 4%3=1, 5%3=2 -> 3 unique
    assert_eq!(report.unique_sessions_count, 3);
    // Traces: 1%2=1, 2%2=0, 3%2=1, 4%2=0, 5%2=1 -> 2 unique
    assert_eq!(report.unique_traces_count, 2);
}

#[test]
fn test_observability_outcome_histogram() {
    let ring = AuditRing::open_in_memory().expect("open ring");
    let mut service = AuditChainService::new(ring);

    for (tool, outcome) in [
        ("tool.a", "success"),
        ("tool.b", "success"),
        ("tool.c", "denied"),
        ("tool.d", "error"),
    ] {
        let mut base = AuditRowInput::default();
        base.actor = "agent_histo".into();
        base.tool = tool.into();
        base.outcome = outcome.into();

        let input = ExtendedAuditRowInput::new(base);
        service.record_event(input).expect("record");
    }

    let report = AuditChainObservabilityReport::generate(&service).expect("report");
    assert_eq!(report.outcomes_by_type.get("success"), Some(&2));
    assert_eq!(report.outcomes_by_type.get("denied"), Some(&1));
    assert_eq!(report.outcomes_by_type.get("error"), Some(&1));
}

#[test]
fn test_observability_sanitization_negative_control_chars() {
    let dirty = "\x1b[31mRed Alert\x00\r\n\t--inject--";
    let clean = sanitize_telemetry_text(dirty);
    assert!(!clean.contains('\x1b'));
    assert!(!clean.contains('\x00'));
    assert!(!clean.contains('\r'));
    assert!(!clean.contains('\n'));
    assert!(!clean.contains('\t'));
}

#[test]
fn test_observability_report_validation_failures() {
    let report = AuditChainObservabilityReport {
        generated_at_utc: "".into(), // Invalid: empty
        total_rows: 10,
        total_extended_rows: 5,
        total_causal_links: 0,
        total_signed_events: 0,
        unique_actors_count: 3,
        unique_tools_count: 2,
        unique_sessions_count: 1,
        unique_traces_count: 1,
        outcomes_by_type: std::collections::HashMap::new(),
        active_policy_mode: "enforcing".into(),
        db_file_bytes: 1024,
        chain_integrity_checked: true,
        chain_integrity_ok: true,
        is_healthy: true,
    };

    assert!(report.validate().is_err());
    let err = report.validate().err().unwrap();
    assert!(err.contains(AUDITOBS_ERR_VALIDATION));
}
