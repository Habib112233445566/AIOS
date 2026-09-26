//! Unit tests for PEP Grant Lifecycle Observability Subsystem (T-02275).

use aiosh_core::capability::{CapabilityRight, CapabilityScope};
use aiosh_core::pep_grant::{PepGrant, PepGrantState, PepGrantStore};
use aiosh_core::pep_grant_observability::{
    sanitize_grant_telemetry_text, PepGrantObservabilityReport,
    PEPOBS_GRANT_ERR_VALIDATION,
};
use aiosh_core::pep_grant_service::PepGrantService;

fn create_sample_grant(id: &str, subject: &str, parent: Option<&str>, state: PepGrantState) -> PepGrant {
    let mut g = PepGrant::new(
        id,
        "kernel-issuer",
        subject,
        CapabilityScope::Filesystem {
            path: "/var/data".into(),
            recursive: true,
        },
        vec![CapabilityRight::Read, CapabilityRight::Write],
    );
    g.parent_grant_id = parent.map(|s| s.to_string());
    g.state = state;
    g
}

#[test]
fn test_grant_observability_empty_service() {
    let service = PepGrantService::new();
    let rep = service.generate_observability_report();

    assert_eq!(rep.total_grants, 0);
    assert_eq!(rep.active_grants, 0);
    assert_eq!(rep.root_grants_count, 0);
    assert_eq!(rep.derived_grants_count, 0);
    assert_eq!(rep.capacity_utilization_percent, 0);
    assert!(rep.is_healthy);
    assert!(rep.validate().is_ok());
}

#[test]
fn test_grant_observability_populated_service() {
    let mut service = PepGrantService::new();

    // 1. Root active grant
    let g1 = create_sample_grant("g-root", "agent:root", None, PepGrantState::Active);
    service.issue_grant(g1).unwrap();

    // 2. Derived suspended grant
    let g2 = create_sample_grant("g-child-1", "agent:worker1", Some("g-root"), PepGrantState::Suspended);
    service.issue_grant(g2).unwrap();

    // 3. Derived revoked grant
    let g3 = create_sample_grant("g-child-2", "agent:worker2", Some("g-root"), PepGrantState::Revoked);
    service.issue_grant(g3).unwrap();

    let rep = service.generate_observability_report();
    assert_eq!(rep.total_grants, 3);
    assert_eq!(rep.active_grants, 1);
    assert_eq!(rep.suspended_grants, 1);
    assert_eq!(rep.revoked_grants, 1);
    assert_eq!(rep.expired_grants, 0);
    assert_eq!(rep.root_grants_count, 1);
    assert_eq!(rep.derived_grants_count, 2);
    assert_eq!(rep.unique_subjects_count, 3);
    assert_eq!(rep.unique_issuers_count, 1);
    assert!(rep.is_healthy);
    assert!(rep.validate().is_ok());
}

#[test]
fn test_grant_observability_invariant_validation() {
    let mut service = PepGrantService::new();
    let g1 = create_sample_grant("g-1", "agent:w", None, PepGrantState::Active);
    service.issue_grant(g1).unwrap();

    let mut rep = service.generate_observability_report();
    assert!(rep.validate().is_ok());

    // Tamper total grants
    rep.total_grants = 999;
    let err = rep.validate().unwrap_err();
    assert!(err.contains(PEPOBS_GRANT_ERR_VALIDATION));
    assert!(err.contains("state counts sum"));
}

#[test]
fn test_grant_observability_health_threshold_flip() {
    let mut rep = PepGrantObservabilityReport {
        total_grants: 4600,
        requested_grants: 0,
        active_grants: 4600,
        suspended_grants: 0,
        revoked_grants: 0,
        expired_grants: 0,
        root_grants_count: 4600,
        derived_grants_count: 0,
        unique_subjects_count: 10,
        unique_issuers_count: 1,
        grants_by_state: Default::default(),
        grants_by_right: Default::default(),
        grants_by_scope_type: Default::default(),
        capacity_limit: 5000,
        capacity_utilization_percent: 92, // >= 90%
        is_healthy: false,
        generated_at: "2026-09-22T00:00:00Z".into(),
    };

    assert!(rep.validate().is_ok());
    assert!(!rep.is_healthy);

    // Mismatched health indicator
    rep.is_healthy = true;
    let err = rep.validate().unwrap_err();
    assert!(err.contains("is_healthy"));
}

#[test]
fn test_grant_observability_store_integration() {
    let mut store = PepGrantStore::new();
    let g = create_sample_grant("g-store-1", "agent:store_test", None, PepGrantState::Active);
    store.add_grant(g).unwrap();

    let rep = store.generate_observability_report();
    assert_eq!(rep.total_grants, 1);
    assert_eq!(rep.active_grants, 1);
    assert!(rep.is_healthy);
    assert!(rep.validate().is_ok());
}

#[test]
fn test_grant_telemetry_sanitization() {
    let dirty = "  agent:worker\x00\x07\n\t_test  ";
    let cleaned = sanitize_grant_telemetry_text(dirty);
    assert_eq!(cleaned, "agent:worker_test");

    // Length bound
    let overlong = "a".repeat(500);
    let truncated = sanitize_grant_telemetry_text(&overlong);
    assert_eq!(truncated.len(), 256);
}
