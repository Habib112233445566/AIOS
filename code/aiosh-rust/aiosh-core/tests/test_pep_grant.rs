//! Comprehensive automated unit tests for Grant Lifecycle Data Model (PEPGRANT1..PEPGRANT6).

use aiosh_core::capability::{CapabilityRight, CapabilityScope};
use aiosh_core::pep_grant::{
    PepGrant, PepGrantState, PepGrantStore,
    PEPGRANT_ERR_ATTENUATION, PEPGRANT_ERR_EXPIRED, PEPGRANT_ERR_INVALID_ID,
    PEPGRANT_ERR_INVALID_TRANSITION, PEPGRANT_ERR_NOT_YET_VALID,
    PEPGRANT_ERR_VALIDATION,
};

#[test]
fn test_pep_grant_valid_creation_and_validation() {
    let grant = PepGrant::new(
        "grnt-001",
        "system-admin",
        "agent-worker-1",
        CapabilityScope::Filesystem {
            path: "/var/log".to_string(),
            recursive: true,
        },
        vec![CapabilityRight::Read, CapabilityRight::Write],
    );

    assert_eq!(grant.id, "grnt-001");
    assert_eq!(grant.issuer, "system-admin");
    assert_eq!(grant.subject, "agent-worker-1");
    assert_eq!(grant.state, PepGrantState::Requested);
    assert_eq!(grant.rights.len(), 2);
    assert!(grant.validate().is_ok());
}

#[test]
fn test_pep_grant_invalid_identifier_and_scope() {
    // Empty ID
    let g1 = PepGrant::new("", "system", "agent", CapabilityScope::System { subsystem: "core".into() }, vec![CapabilityRight::Read]);
    assert!(g1.validate().unwrap_err().contains(PEPGRANT_ERR_INVALID_ID));

    // ID with control character
    let g2 = PepGrant::new("grant\x00bad", "system", "agent", CapabilityScope::System { subsystem: "core".into() }, vec![CapabilityRight::Read]);
    assert!(g2.validate().unwrap_err().contains(PEPGRANT_ERR_INVALID_ID));

    // Subject with path traversal / illegal characters
    let g3 = PepGrant::new("grnt-002", "system", "agent/../../root", CapabilityScope::System { subsystem: "core".into() }, vec![CapabilityRight::Read]);
    assert!(g3.validate().unwrap_err().contains(PEPGRANT_ERR_VALIDATION));

    // Filesystem scope with '..' traversal
    let g4 = PepGrant::new(
        "grnt-003",
        "system",
        "agent-1",
        CapabilityScope::Filesystem { path: "/etc/../shadow".into(), recursive: false },
        vec![CapabilityRight::Read],
    );
    assert!(g4.validate().unwrap_err().contains(PEPGRANT_ERR_VALIDATION));

    // Empty rights
    let g5 = PepGrant::new("grnt-004", "system", "agent-1", CapabilityScope::System { subsystem: "core".into() }, vec![]);
    assert!(g5.validate().unwrap_err().contains(PEPGRANT_ERR_VALIDATION));
}

#[test]
fn test_pep_grant_fsm_transitions() {
    let mut grant = PepGrant::new(
        "grnt-fsm-1",
        "admin",
        "agent-1",
        CapabilityScope::System { subsystem: "storage".into() },
        vec![CapabilityRight::Read],
    );

    assert_eq!(grant.state, PepGrantState::Requested);

    // Requested -> Active: OK
    assert!(grant.transition_to(PepGrantState::Active).is_ok());
    assert_eq!(grant.state, PepGrantState::Active);

    // Active -> Suspended: OK
    assert!(grant.transition_to(PepGrantState::Suspended).is_ok());
    assert_eq!(grant.state, PepGrantState::Suspended);

    // Suspended -> Active: OK
    assert!(grant.transition_to(PepGrantState::Active).is_ok());
    assert_eq!(grant.state, PepGrantState::Active);

    // Active -> Revoked: OK
    assert!(grant.revoke("admin-operator", "Policy violation detected").is_ok());
    assert_eq!(grant.state, PepGrantState::Revoked);
    assert!(grant.revocation.is_some());
    assert_eq!(grant.revocation.as_ref().unwrap().revoked_by, "admin-operator");

    // Terminal state: Revoked cannot transition anywhere
    let err = grant.transition_to(PepGrantState::Active).unwrap_err();
    assert!(err.contains(PEPGRANT_ERR_INVALID_TRANSITION));

    // Expired terminal test
    let mut grant2 = PepGrant::new(
        "grnt-fsm-2",
        "admin",
        "agent-2",
        CapabilityScope::System { subsystem: "storage".into() },
        vec![CapabilityRight::Read],
    );
    assert!(grant2.transition_to(PepGrantState::Active).is_ok());
    assert!(grant2.transition_to(PepGrantState::Expired).is_ok());
    let err2 = grant2.transition_to(PepGrantState::Active).unwrap_err();
    assert!(err2.contains(PEPGRANT_ERR_INVALID_TRANSITION));
}

#[test]
fn test_pep_grant_temporal_and_quota_evaluation() {
    let mut grant = PepGrant::new(
        "grnt-quota-1",
        "admin",
        "agent-1",
        CapabilityScope::System { subsystem: "compute".into() },
        vec![CapabilityRight::Execute],
    );
    grant.state = PepGrantState::Active;

    // Temporal bounds: future not_before
    grant.constraints.not_before = Some("2026-10-01T00:00:00Z".to_string());
    grant.constraints.expires_at = Some("2026-12-31T23:59:59Z".to_string());

    let early_check = grant.is_usable_at("2026-09-22T10:00:00Z");
    assert!(early_check.unwrap_err().contains(PEPGRANT_ERR_NOT_YET_VALID));

    // Valid time window
    grant.constraints.not_before = Some("2026-09-01T00:00:00Z".to_string());
    assert!(grant.is_usable_at("2026-09-22T10:00:00Z").is_ok());

    // Expired time
    let late_check = grant.is_usable_at("2027-01-01T00:00:00Z");
    assert!(late_check.unwrap_err().contains(PEPGRANT_ERR_EXPIRED));

    // Quotas: max 2 invocations
    grant.constraints.expires_at = None;
    grant.constraints.max_invocations = Some(2);
    grant.constraints.invocations_used = 0;

    assert!(grant.record_invocation(100).is_ok());
    assert_eq!(grant.state, PepGrantState::Active);
    assert_eq!(grant.constraints.invocations_used, 1);
    assert_eq!(grant.constraints.bytes_used, 100);

    assert!(grant.record_invocation(200).is_ok());
    assert_eq!(grant.state, PepGrantState::Expired); // Auto-transitions to Expired
    assert_eq!(grant.constraints.invocations_used, 2);
    assert_eq!(grant.constraints.bytes_used, 300);

    let quota_err = grant.is_usable_at("2026-09-22T10:00:00Z");
    assert!(quota_err.unwrap_err().contains(PEPGRANT_ERR_EXPIRED));
}

#[test]
fn test_pep_grant_attenuation() {
    let mut parent = PepGrant::new(
        "grnt-parent-1",
        "root-admin",
        "agent-lead",
        CapabilityScope::Filesystem { path: "/data".into(), recursive: true },
        vec![CapabilityRight::Read, CapabilityRight::Write, CapabilityRight::Delegate],
    );
    parent.state = PepGrantState::Active;
    parent.constraints.max_delegation_depth = 2;

    // Attenuation with valid subset of rights
    let child = parent.attenuate("grnt-child-1", "agent-worker-1", vec![CapabilityRight::Read]);
    assert!(child.is_ok());
    let child_grant = child.unwrap();
    assert_eq!(child_grant.parent_grant_id.as_deref(), Some("grnt-parent-1"));
    assert_eq!(child_grant.rights, vec![CapabilityRight::Read]);
    assert_eq!(child_grant.constraints.max_delegation_depth, 1);
    assert_eq!(child_grant.issuer, "agent-lead");
    assert_eq!(child_grant.subject, "agent-worker-1");

    // Attenuation requesting rights outside parent rights
    let invalid_rights = parent.attenuate("grnt-child-2", "agent-worker-2", vec![CapabilityRight::Admin]);
    assert!(invalid_rights.unwrap_err().contains(PEPGRANT_ERR_ATTENUATION));

    // Attenuation on parent without 'delegate' right
    parent.rights = vec![CapabilityRight::Read];
    let no_delegate = parent.attenuate("grnt-child-3", "agent-worker-3", vec![CapabilityRight::Read]);
    assert!(no_delegate.unwrap_err().contains(PEPGRANT_ERR_ATTENUATION));

    // Attenuation when depth is 0
    parent.rights = vec![CapabilityRight::Read, CapabilityRight::Delegate];
    parent.constraints.max_delegation_depth = 0;
    let depth_exhausted = parent.attenuate("grnt-child-4", "agent-worker-4", vec![CapabilityRight::Read]);
    assert!(depth_exhausted.unwrap_err().contains(PEPGRANT_ERR_ATTENUATION));
}

#[test]
fn test_pep_grant_store_operations_and_cascade_revocation() {
    let mut store = PepGrantStore::new();

    let mut g1 = PepGrant::new("g-root", "admin", "agent-1", CapabilityScope::System { subsystem: "net".into() }, vec![CapabilityRight::Read, CapabilityRight::Delegate]);
    g1.state = PepGrantState::Active;
    let mut g2 = PepGrant::new("g-child", "agent-1", "agent-2", CapabilityScope::System { subsystem: "net".into() }, vec![CapabilityRight::Read, CapabilityRight::Delegate]);
    g2.parent_grant_id = Some("g-root".into());
    g2.state = PepGrantState::Active;
    let mut g3 = PepGrant::new("g-grandchild", "agent-2", "agent-3", CapabilityScope::System { subsystem: "net".into() }, vec![CapabilityRight::Read]);
    g3.parent_grant_id = Some("g-child".into());
    g3.state = PepGrantState::Active;

    assert!(store.add_grant(g1).is_ok());
    assert!(store.add_grant(g2).is_ok());
    assert!(store.add_grant(g3).is_ok());
    assert_eq!(store.len(), 3);

    // Lookups by subject
    assert_eq!(store.list_grants_for_subject("agent-2").len(), 1);

    // Cascade revocation from root should revoke g-root, g-child, and g-grandchild
    let count = store.revoke_grant("g-root", "security-officer", "Breach containment", true);
    assert_eq!(count, Ok(3));

    assert_eq!(store.get_grant("g-root").unwrap().state, PepGrantState::Revoked);
    assert_eq!(store.get_grant("g-child").unwrap().state, PepGrantState::Revoked);
    assert_eq!(store.get_grant("g-grandchild").unwrap().state, PepGrantState::Revoked);
}

#[test]
fn test_pep_grant_store_atomic_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let store_path = temp_dir.path().join("pep_grants.json");

    let mut store = PepGrantStore::new();
    let grant = PepGrant::new("g-persist-1", "admin", "agent-1", CapabilityScope::System { subsystem: "ipc".into() }, vec![CapabilityRight::Read]);
    store.add_grant(grant).unwrap();

    assert!(store.save_to_path(&store_path).is_ok());
    assert!(store_path.exists());

    let loaded = PepGrantStore::load_from_path(&store_path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded.get_grant("g-persist-1").unwrap().subject, "agent-1");
}

#[test]
fn test_pep_grant_action_validation() {
    let mut store = PepGrantStore::new();
    let mut grant = PepGrant::new("g-act-1", "admin", "agent-1", CapabilityScope::System { subsystem: "db".into() }, vec![CapabilityRight::Read]);
    grant.state = PepGrantState::Active;
    store.add_grant(grant).unwrap();

    let now = "2026-09-22T12:00:00Z";

    // Matching subject and right: OK
    assert!(store.validate_grant_for_action("g-act-1", "agent-1", CapabilityRight::Read, now).is_ok());

    // Mismatched subject: Error
    let err_subj = store.validate_grant_for_action("g-act-1", "agent-2", CapabilityRight::Read, now);
    assert!(err_subj.unwrap_err().contains(PEPGRANT_ERR_VALIDATION));

    // Mismatched right: Error
    let err_right = store.validate_grant_for_action("g-act-1", "agent-1", CapabilityRight::Write, now);
    assert!(err_right.unwrap_err().contains(PEPGRANT_ERR_ATTENUATION));
}
