//! Unit and integration tests for Grant Lifecycle Security Policy (T-02265).

use std::path::PathBuf;
use chrono::{Duration, Utc};
use tempfile::tempdir;

use aiosh_core::capability::{CapabilityRight, CapabilityScope};
use aiosh_core::pep_grant::PepGrant;
use aiosh_core::pep_grant_security_policy::{
    PepGrantEnforcementMode, PepGrantSecurityPolicy,
    GRANTPOL_ERR_DELEGATION_REJECTED, GRANTPOL_ERR_IO, GRANTPOL_ERR_LIFETIME_EXCEEDED,
    GRANTPOL_ERR_POLICY_VIOLATION, GRANTPOL_ERR_VALIDATION,
};
use aiosh_core::pep_grant_service::PepGrantService;

fn create_valid_grant(id: &str, subject: &str) -> PepGrant {
    let now = Utc::now();
    let exp = now + Duration::days(7);
    let mut g = PepGrant::new(
        id,
        "kernel",
        subject,
        CapabilityScope::System {
            subsystem: "fs".into(),
        },
        vec![CapabilityRight::Read, CapabilityRight::Write],
    );
    g.constraints.not_before = Some(now.to_rfc3339());
    g.constraints.expires_at = Some(exp.to_rfc3339());
    g.constraints.max_delegation_depth = 4;
    g
}

#[test]
fn test_grant_policy_default_and_validation_bounds() {
    let mut pol = PepGrantSecurityPolicy::default();
    assert!(pol.validate().is_ok());
    assert_eq!(pol.mode, PepGrantEnforcementMode::Enforcing);
    assert_eq!(pol.max_delegation_depth, 5);

    // Empty version
    pol.version = "   ".into();
    assert!(pol.validate().unwrap_err().contains(GRANTPOL_ERR_VALIDATION));
    pol.version = "1.0.0".into();

    // Duration bound
    pol.max_grant_duration_seconds = 10;
    assert!(pol.validate().unwrap_err().contains(GRANTPOL_ERR_VALIDATION));
    pol.max_grant_duration_seconds = 86400;

    // Depth bound
    pol.max_delegation_depth = 0;
    assert!(pol.validate().unwrap_err().contains(GRANTPOL_ERR_VALIDATION));
    pol.max_delegation_depth = 9;
    assert!(pol.validate().unwrap_err().contains(GRANTPOL_ERR_VALIDATION));
    pol.max_delegation_depth = 5;

    // Capacity bound
    pol.max_store_capacity = 0;
    assert!(pol.validate().unwrap_err().contains(GRANTPOL_ERR_VALIDATION));
    pol.max_store_capacity = 100_000;
    assert!(pol.validate().unwrap_err().contains(GRANTPOL_ERR_VALIDATION));
}

#[test]
fn test_grant_policy_prohibited_subjects() {
    let pol = PepGrantSecurityPolicy::default();

    // Valid subject
    let valid_grant = create_valid_grant("g-1", "agent:trusted_worker");
    assert!(pol.validate_grant(&valid_grant).is_ok());

    // Prohibited subject: anonymous
    let anon_grant = create_valid_grant("g-anon", "agent:anonymous_user");
    let err = pol.validate_grant(&anon_grant).unwrap_err();
    assert!(err.contains(GRANTPOL_ERR_POLICY_VIOLATION));
    assert!(err.contains("anonymous"));

    // Prohibited subject: nobody
    let nobody_grant = create_valid_grant("g-nobody", "nobody");
    assert!(pol.validate_grant(&nobody_grant).unwrap_err().contains("nobody"));
}

#[test]
fn test_grant_policy_enforcement_modes() {
    let mut pol = PepGrantSecurityPolicy::default();
    let anon_grant = create_valid_grant("g-anon", "agent:anonymous_guest");

    // Enforcing: fails
    pol.mode = PepGrantEnforcementMode::Enforcing;
    assert!(pol.validate_grant(&anon_grant).is_err());

    // Permissive: succeeds with audit warning
    pol.mode = PepGrantEnforcementMode::Permissive;
    assert!(pol.validate_grant(&anon_grant).is_ok());

    // Disabled: succeeds unconditionally
    pol.mode = PepGrantEnforcementMode::Disabled;
    assert!(pol.validate_grant(&anon_grant).is_ok());
}

#[test]
fn test_grant_policy_delegation_disallowed_rights() {
    let pol = PepGrantSecurityPolicy::default();

    let parent = create_valid_grant("g-p", "agent:admin");
    let mut child = create_valid_grant("g-c", "agent:child");
    child.parent_grant_id = Some("g-p".into());
    child.constraints.max_delegation_depth = 3;

    // Happy child: read right only
    child.rights = vec![CapabilityRight::Read];
    assert!(pol.validate_attenuation(&parent, &child).is_ok());

    // Disallowed delegation right: Admin
    child.rights = vec![CapabilityRight::Read, CapabilityRight::Admin];
    let err = pol.validate_attenuation(&parent, &child).unwrap_err();
    assert!(err.contains(GRANTPOL_ERR_DELEGATION_REJECTED));
}

#[test]
fn test_grant_policy_duration_ceiling() {
    let mut pol = PepGrantSecurityPolicy::default();
    pol.max_grant_duration_seconds = 3600; // 1 hour max

    let now = Utc::now();
    let mut grant = create_valid_grant("g-long", "agent:worker");
    grant.constraints.not_before = Some(now.to_rfc3339());
    grant.constraints.expires_at = Some((now + Duration::hours(5)).to_rfc3339()); // 5 hours

    let err = pol.validate_grant(&grant).unwrap_err();
    assert!(err.contains(GRANTPOL_ERR_LIFETIME_EXCEEDED));
}

#[test]
fn test_grant_policy_service_integration() {
    let pol = PepGrantSecurityPolicy::default();
    let mut service = PepGrantService::new().with_security_policy(pol);

    // 1. Issue valid grant -> OK
    let valid = create_valid_grant("g-svc-valid", "agent:authorized");
    assert!(service.issue_grant(valid).is_ok());

    // 2. Issue prohibited subject grant -> Rejected by policy
    let anon = create_valid_grant("g-svc-anon", "agent:anonymous_probe");
    let err = service.issue_grant(anon).unwrap_err();
    assert!(err.contains(GRANTPOL_ERR_POLICY_VIOLATION));
}

#[test]
fn test_grant_policy_persistence_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("pep_grant_security_policy.json");

    let pol = PepGrantSecurityPolicy::default();
    assert!(pol.save_to_path(&path).is_ok());

    let loaded = PepGrantSecurityPolicy::load_from_path(&path).expect("load policy");
    assert_eq!(pol, loaded);

    // Path traversal rejection
    let bad_path = PathBuf::from("../../evil_policy.json");
    assert!(PepGrantSecurityPolicy::load_from_path(&bad_path).unwrap_err().contains(GRANTPOL_ERR_IO));
}
