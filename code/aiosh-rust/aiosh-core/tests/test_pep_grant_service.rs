//! Automated tests for PepGrantService (GSVC1..GSVC6).

use aiosh_core::capability::{CapabilityRight, CapabilityScope};
use aiosh_core::pep_grant::{PepGrant, PepGrantState};
use aiosh_core::pep_grant_service::{PepGrantService, GSVC_ERR_ATTENUATION};

#[test]
fn test_pep_grant_service_scaffold_creation() {
    let service = PepGrantService::new();
    assert_eq!(service.len(), 0);
    assert!(service.is_empty());
}

#[test]
fn test_pep_grant_service_issue_and_query() {
    let mut service = PepGrantService::new();

    let grant = PepGrant::new(
        "g-svc-1",
        "admin",
        "worker-alpha",
        CapabilityScope::System { subsystem: "io".into() },
        vec![CapabilityRight::Read, CapabilityRight::Write],
    );

    assert!(service.issue_grant(grant).is_ok());
    assert_eq!(service.len(), 1);
    assert!(!service.is_empty());

    // Query by ID
    assert_eq!(service.get_grant("g-svc-1").unwrap().subject, "worker-alpha");

    // Query by subject
    let for_subject = service.list_grants_for_subject("worker-alpha");
    assert_eq!(for_subject.len(), 1);
    assert_eq!(for_subject[0].id, "g-svc-1");

    // Query by state (initially Requested)
    let requested = service.list_grants_by_state(PepGrantState::Requested);
    assert_eq!(requested.len(), 1);

    let active = service.list_grants_by_state(PepGrantState::Active);
    assert_eq!(active.len(), 0);
}

#[test]
fn test_pep_grant_service_transition_and_indexes() {
    let mut service = PepGrantService::new();

    let grant = PepGrant::new(
        "g-trans-1",
        "admin",
        "worker-1",
        CapabilityScope::System { subsystem: "net".into() },
        vec![CapabilityRight::Read],
    );

    service.issue_grant(grant).unwrap();

    // Transition Requested -> Active
    assert!(service.transition_grant("g-trans-1", PepGrantState::Active).is_ok());
    assert_eq!(service.list_grants_by_state(PepGrantState::Active).len(), 1);
    assert_eq!(service.list_grants_by_state(PepGrantState::Requested).len(), 0);

    // Transition Active -> Suspended
    assert!(service.transition_grant("g-trans-1", PepGrantState::Suspended).is_ok());
    assert_eq!(service.list_grants_by_state(PepGrantState::Suspended).len(), 1);
    assert_eq!(service.list_grants_by_state(PepGrantState::Active).len(), 0);
}

#[test]
fn test_pep_grant_service_attenuation() {
    let mut service = PepGrantService::new();

    let mut parent = PepGrant::new(
        "g-parent",
        "admin",
        "lead-agent",
        CapabilityScope::Filesystem { path: "/secure".into(), recursive: true },
        vec![CapabilityRight::Read, CapabilityRight::Write, CapabilityRight::Delegate],
    );
    parent.state = PepGrantState::Active;
    service.issue_grant(parent).unwrap();

    // Valid attenuation
    let child = service.attenuate_grant(
        "g-parent",
        "g-child",
        "sub-agent-1",
        vec![CapabilityRight::Read],
    );
    assert!(child.is_ok());
    assert_eq!(service.len(), 2);
    assert_eq!(service.get_grant("g-child").unwrap().parent_grant_id.as_deref(), Some("g-parent"));

    // Attenuation escalation fails
    let escalation = service.attenuate_grant(
        "g-parent",
        "g-child-bad",
        "sub-agent-2",
        vec![CapabilityRight::Admin],
    );
    assert!(escalation.unwrap_err().contains(GSVC_ERR_ATTENUATION));
}

#[test]
fn test_pep_grant_service_evaluation_and_usage() {
    let mut service = PepGrantService::new();

    let mut grant = PepGrant::new(
        "g-eval-1",
        "admin",
        "worker-1",
        CapabilityScope::System { subsystem: "cpu".into() },
        vec![CapabilityRight::Execute],
    );
    grant.state = PepGrantState::Active;
    grant.constraints.max_invocations = Some(2);
    service.issue_grant(grant).unwrap();

    let now = "2026-09-22T14:00:00Z";

    // Evaluation OK
    assert!(service.evaluate_grant("g-eval-1", "worker-1", CapabilityRight::Execute, now).is_ok());

    // Record usage
    assert!(service.record_grant_usage("g-eval-1", 1024).is_ok());
    assert_eq!(service.get_grant("g-eval-1").unwrap().constraints.invocations_used, 1);
    assert_eq!(service.get_grant("g-eval-1").unwrap().constraints.bytes_used, 1024);

    // Record usage reaching quota limit (2)
    assert!(service.record_grant_usage("g-eval-1", 2048).is_ok());
    assert_eq!(service.get_grant("g-eval-1").unwrap().state, PepGrantState::Expired);
    assert_eq!(service.list_grants_by_state(PepGrantState::Expired).len(), 1);
}

#[test]
fn test_pep_grant_service_cascade_revocation() {
    let mut service = PepGrantService::new();

    let mut g1 = PepGrant::new("root-g", "admin", "agent-1", CapabilityScope::System { subsystem: "mem".into() }, vec![CapabilityRight::Read, CapabilityRight::Delegate]);
    g1.state = PepGrantState::Active;
    let mut g2 = PepGrant::new("child-g", "agent-1", "agent-2", CapabilityScope::System { subsystem: "mem".into() }, vec![CapabilityRight::Read, CapabilityRight::Delegate]);
    g2.parent_grant_id = Some("root-g".into());
    g2.state = PepGrantState::Active;
    let mut g3 = PepGrant::new("leaf-g", "agent-2", "agent-3", CapabilityScope::System { subsystem: "mem".into() }, vec![CapabilityRight::Read]);
    g3.parent_grant_id = Some("child-g".into());
    g3.state = PepGrantState::Active;

    service.issue_grant(g1).unwrap();
    service.issue_grant(g2).unwrap();
    service.issue_grant(g3).unwrap();

    let count = service.revoke_grant("root-g", "security-officer", "Compromised", true);
    assert_eq!(count, Ok(3));

    assert_eq!(service.get_grant("root-g").unwrap().state, PepGrantState::Revoked);
    assert_eq!(service.get_grant("child-g").unwrap().state, PepGrantState::Revoked);
    assert_eq!(service.get_grant("leaf-g").unwrap().state, PepGrantState::Revoked);
    assert_eq!(service.list_grants_by_state(PepGrantState::Revoked).len(), 3);
}

#[test]
fn test_pep_grant_service_sweep_expired() {
    let mut service = PepGrantService::new();

    let mut g1 = PepGrant::new("g-sweep-1", "admin", "agent-1", CapabilityScope::System { subsystem: "disk".into() }, vec![CapabilityRight::Read]);
    g1.state = PepGrantState::Active;
    g1.constraints.expires_at = Some("2026-09-22T10:00:00Z".into());

    let mut g2 = PepGrant::new("g-sweep-2", "admin", "agent-2", CapabilityScope::System { subsystem: "disk".into() }, vec![CapabilityRight::Read]);
    g2.state = PepGrantState::Active;
    g2.constraints.expires_at = Some("2026-09-23T00:00:00Z".into());

    service.issue_grant(g1).unwrap();
    service.issue_grant(g2).unwrap();

    let now = "2026-09-22T15:00:00Z";
    let swept = service.sweep_expired(now);
    assert_eq!(swept, Ok(1));

    assert_eq!(service.get_grant("g-sweep-1").unwrap().state, PepGrantState::Expired);
    assert_eq!(service.get_grant("g-sweep-2").unwrap().state, PepGrantState::Active);
}

#[test]
fn test_pep_grant_service_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let store_file = temp_dir.path().join("grant_service_test.json");

    let mut service = PepGrantService::new().with_storage_path(store_file.clone());

    let grant = PepGrant::new("g-persist", "admin", "agent-1", CapabilityScope::System { subsystem: "auth".into() }, vec![CapabilityRight::Read]);
    service.issue_grant(grant).unwrap();

    assert!(service.sync().is_ok());
    assert!(store_file.exists());

    let loaded = PepGrantService::load_from_path(&store_file).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded.list_grants_for_subject("agent-1").len(), 1);
}

#[test]
fn test_pep_grant_service_negative_capacity_and_transitions() {
    let mut service = PepGrantService::new();

    // 1. Transition non-existent grant
    let err_missing = service.transition_grant("does_not_exist", PepGrantState::Active);
    assert!(err_missing.unwrap_err().contains("not found"));

    // 2. Transition from terminal states (Revoked or Expired)
    let mut g = PepGrant::new("g-term", "admin", "agent-1", CapabilityScope::System { subsystem: "sec".into() }, vec![CapabilityRight::Read]);
    g.state = PepGrantState::Active;
    service.issue_grant(g).unwrap();

    service.revoke_grant("g-term", "admin", "test", false).unwrap();
    assert_eq!(service.get_grant("g-term").unwrap().state, PepGrantState::Revoked);

    let err_revoked = service.transition_grant("g-term", PepGrantState::Active);
    assert!(err_revoked.unwrap_err().contains("cannot transition"));
}

#[test]
fn test_pep_grant_service_negative_attenuation_and_eval() {
    let mut service = PepGrantService::new();

    // 1. Attenuate non-existent parent
    let err_parent = service.attenuate_grant("ghost-parent", "child-1", "agent-2", vec![CapabilityRight::Read]);
    assert!(err_parent.unwrap_err().contains("not found"));

    // 2. Attenuate inactive parent (in Requested state)
    let parent = PepGrant::new("p-req", "admin", "agent-1", CapabilityScope::System { subsystem: "sec".into() }, vec![CapabilityRight::Read, CapabilityRight::Delegate]);
    service.issue_grant(parent).unwrap();
    let err_inactive = service.attenuate_grant("p-req", "child-2", "agent-2", vec![CapabilityRight::Read]);
    assert!(err_inactive.unwrap_err().contains("cannot attenuate non-active"));

    // 3. Evaluate non-existent grant
    let err_eval_missing = service.evaluate_grant("ghost-eval", "agent-1", CapabilityRight::Read, "2026-09-22T12:00:00Z");
    assert!(err_eval_missing.unwrap_err().contains("not found"));

    // 4. Evaluate with mismatched subject
    let mut active_g = PepGrant::new("g-active", "admin", "agent-alice", CapabilityScope::System { subsystem: "sec".into() }, vec![CapabilityRight::Read]);
    active_g.state = PepGrantState::Active;
    service.issue_grant(active_g).unwrap();

    let err_subj = service.evaluate_grant("g-active", "agent-bob", CapabilityRight::Read, "2026-09-22T12:00:00Z");
    assert!(err_subj.unwrap_err().contains("does not match"));

    // 5. Evaluate with mismatched right
    let err_right = service.evaluate_grant("g-active", "agent-alice", CapabilityRight::Write, "2026-09-22T12:00:00Z");
    assert!(err_right.unwrap_err().contains("does not confer right"));
}

#[test]
fn test_pep_grant_service_negative_path_and_file_checks() {
    let temp_dir = tempfile::tempdir().unwrap();

    // 1. Invalid extension
    let bad_ext = temp_dir.path().join("store.txt");
    assert!(service_save_fails(&bad_ext));

    // 2. Non-existent file load
    let missing_file = temp_dir.path().join("missing.json");
    let err_load = PepGrantService::load_from_path(&missing_file);
    assert!(err_load.unwrap_err().contains("does not exist"));

    // 3. Directory load rejection
    let json_dir = temp_dir.path().join("dir.json");
    std::fs::create_dir(&json_dir).unwrap();
    let dir_load = PepGrantService::load_from_path(&json_dir);
    assert!(dir_load.unwrap_err().contains("is a directory"));
}

fn service_save_fails(path: &std::path::Path) -> bool {
    let service = PepGrantService::new();
    service.save_to_path(path).is_err()
}

