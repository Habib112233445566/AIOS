//! Unit tests for Sandbox Enforcement Observability Subsystem (T-02475).

use tempfile::tempdir;

use aiosh_core::audit::{AuditRing, OpenOptions};
use aiosh_core::sandbox_data_model::*;
use aiosh_core::sandbox_observability::*;
use aiosh_core::sandbox_service::*;

#[test]
fn test_observability_default_generation() {
    let svc = SandboxService::with_default_profiles(None);
    let report = svc.generate_observability_report().expect("generate report");

    assert_eq!(report.total_profiles_registered, 3);
    assert_eq!(report.total_executions_recorded, 0);
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
fn test_observability_with_executions_and_outcomes() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("obs_audit.db");
    let ring = AuditRing::open(OpenOptions {
        path: Some(db_path.to_str().unwrap().to_string()),
        home: None,
    }).expect("open ring");

    let mut svc = SandboxService::with_default_profiles(Some(ring));
    let profile = svc.get_profile("permissive").unwrap();

    // 1. Success execution
    let req_ok = SandboxExecutionRequest {
        command: "python".into(),
        args: vec!["-c".into(), "print('obs_ok')".into()],
        cwd: None,
        profile: profile.clone(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };
    let _ = svc.execute(&req_ok);

    // 2. Failure execution
    let req_fail = SandboxExecutionRequest {
        command: "python".into(),
        args: vec!["-c".into(), "import sys; sys.exit(5)".into()],
        cwd: None,
        profile,
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };
    let _ = svc.execute(&req_fail);

    let report = svc.generate_observability_report().expect("generate report");
    assert_eq!(report.total_executions_recorded, 2);
    assert_eq!(report.executions_by_outcome.get("ok"), Some(&1));
    assert_eq!(report.executions_by_outcome.get("error"), Some(&1));
    assert_eq!(report.executions_by_profile.get("permissive"), Some(&2));
}

#[test]
fn test_observability_validation_bounds() {
    let svc = SandboxService::with_default_profiles(None);
    let mut report = svc.generate_observability_report().unwrap();

    // Empty timestamp fails
    report.generated_at_utc = "   ".into();
    assert!(report.validate().is_err());
}

#[test]
fn test_observability_cardinality_bounds() {
    let svc = SandboxService::with_default_profiles(None);
    let mut report = svc.generate_observability_report().unwrap();

    for i in 0..MAX_OUTCOME_DISTRIBUTION_ENTRIES + 5 {
        report.executions_by_outcome.insert(format!("outcome_{}", i), 1);
    }
    assert!(report.validate().is_err(), "Should reject cardinality above MAX_OUTCOME_DISTRIBUTION_ENTRIES");
}

