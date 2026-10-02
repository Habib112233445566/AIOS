//! Unit and Integration Tests for Privilege Escalation Prevention Observability (T-02575).

use tempfile::tempdir;

use aiosh_core::audit::{AuditRing, OpenOptions};
use aiosh_core::privilege_data_model::*;
use aiosh_core::privilege_observability::*;
use aiosh_core::privilege_service::*;

#[test]
fn test_observability_default_generation() {
    let service = PrivilegeService::new();
    let report = service.generate_observability_report().expect("generate report");

    assert_eq!(report.total_registered_actors, 0);
    assert_eq!(report.active_contexts_count, 0);
    assert!(report.is_healthy);
    assert_eq!(report.policy_mode, "enforcing");
    assert!(!report.generated_at_utc.is_empty());
}

#[test]
fn test_observability_text_sanitization() {
    let dirty = "malicious\x1b[31m\x00\tstring\n";
    let clean = sanitize_telemetry_text(dirty);
    assert!(!clean.contains('\x1b'));
    assert!(!clean.contains('\x00'));

    let long = "x".repeat(300);
    let clamped = sanitize_telemetry_text(&long);
    assert_eq!(clamped.len(), MAX_TELEMETRY_TEXT_LEN);
}

#[test]
fn test_observability_populated_service() {
    let mut service = PrivilegeService::new();
    let c1 = PrivilegeContext::new("alice", PrivilegeLevel::User).unwrap();
    let c2 = PrivilegeContext::new("bob", PrivilegeLevel::Operator).unwrap();
    let c3 = PrivilegeContext::new("charlie", PrivilegeLevel::User).unwrap();

    assert!(service.register_context(c1).is_ok());
    assert!(service.register_context(c2).is_ok());
    assert!(service.register_context(c3).is_ok());

    let report = service.generate_observability_report().unwrap();
    assert_eq!(report.total_registered_actors, 3);
    assert_eq!(report.active_contexts_count, 3);
    assert_eq!(report.actors_by_tier.get("user"), Some(&2));
    assert_eq!(report.actors_by_tier.get("operator"), Some(&1));
    assert!(report.is_healthy);
}

#[test]
fn test_observability_validation_bounds() {
    let service = PrivilegeService::new();
    let mut report = service.generate_observability_report().unwrap();

    // Empty timestamp fails
    report.generated_at_utc = "   ".into();
    assert!(report.validate().is_err());
}

#[test]
fn test_observability_with_audit_ring() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("priv_obs_audit.db");
    let ring = AuditRing::open(OpenOptions {
        path: Some(db_path.to_str().unwrap().to_string()),
        home: None,
    }).expect("open ring");

    let service = PrivilegeService::new();
    let report = service.generate_observability_report_with_ring(Some(&ring)).unwrap();
    assert!(report.is_healthy);
}

#[test]
fn test_observability_cardinality_limits() {
    let service = PrivilegeService::new();
    let mut report = service.generate_observability_report().unwrap();

    // Inconsistent actor context count
    report.total_registered_actors = 1;
    report.active_contexts_count = 5;
    assert!(report.validate().is_err());
    assert!(report.validate().unwrap_err().contains(PRIVESCOBS_ERR_VALIDATION));

    // Reset counts
    report.total_registered_actors = 200;
    report.active_contexts_count = 10;
    assert!(report.validate().is_ok());

    // Cardinality limit: actors_by_tier
    for i in 0..=MAX_OUTCOME_DISTRIBUTION_ENTRIES {
        report.actors_by_tier.insert(format!("tier_{}", i), 1);
    }
    assert!(report.validate().is_err());

    // Reset actors_by_tier and test transitions_by_outcome
    report.actors_by_tier.clear();
    for i in 0..=MAX_OUTCOME_DISTRIBUTION_ENTRIES {
        report.transitions_by_outcome.insert(format!("outcome_{}", i), 1);
    }
    assert!(report.validate().is_err());
}

#[test]
fn test_observability_unicode_and_control_sanitization() {
    let raw = "ValidTelemetry \u{1F512} \x07 \x08 \x1B[32mOK\x1B[0m";
    let sanitized = sanitize_telemetry_text(raw);
    assert!(!sanitized.contains('\x07'));
    assert!(!sanitized.contains('\x08'));
    assert!(!sanitized.contains('\x1b'));
    assert!(sanitized.contains("\u{1F512}"));
    assert!(sanitized.contains("OK"));
}
