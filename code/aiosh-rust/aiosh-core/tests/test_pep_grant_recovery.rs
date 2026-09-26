//! Automated tests for PEP Grant Lifecycle Recovery & Validation Subsystem (T-02295).

use aiosh_core::capability::{CapabilityRight, CapabilityScope};
use aiosh_core::pep_grant::{PepGrant, PepGrantState};
use aiosh_core::pep_grant_recovery::{
    PepGrantIssueCode, PepGrantIssueSeverity, PepGrantRecoveryManager,
};
use aiosh_core::pep_grant_service::PepGrantService;

fn create_test_grant(id: &str, parent_id: Option<&str>, rights: Vec<CapabilityRight>, max_depth: u32) -> PepGrant {
    let mut grant = PepGrant::new(
        id,
        "operator",
        "agent-1",
        CapabilityScope::Filesystem {
            path: "/workspace".to_string(),
            recursive: true,
        },
        rights,
    );
    grant.parent_grant_id = parent_id.map(|s| s.to_string());
    grant.constraints.max_delegation_depth = max_depth;
    grant.state = PepGrantState::Active;
    grant
}

#[test]
fn test_validate_healthy_grant_store() {
    let parent = create_test_grant("grant-parent", None, vec![CapabilityRight::Read, CapabilityRight::Delegate], 2);
    let child = create_test_grant("grant-child", Some("grant-parent"), vec![CapabilityRight::Read], 1);

    let report = PepGrantRecoveryManager::validate_grants(&[parent, child]);
    assert!(report.is_valid);
    assert_eq!(report.total_grants, 2);
    assert_eq!(report.healthy_grants, 2);
    assert!(report.issues.is_empty());
    assert!(report.can_auto_repair);
}

#[test]
fn test_validate_cycle_detection() {
    let mut g1 = create_test_grant("grant-1", Some("grant-2"), vec![CapabilityRight::Read, CapabilityRight::Delegate], 2);
    let mut g2 = create_test_grant("grant-2", Some("grant-1"), vec![CapabilityRight::Read, CapabilityRight::Delegate], 2);
    g1.parent_grant_id = Some("grant-2".to_string());
    g2.parent_grant_id = Some("grant-1".to_string());

    let report = PepGrantRecoveryManager::validate_grants(&[g1, g2]);
    assert!(!report.is_valid);
    assert!(report.issues.iter().any(|i| i.code == PepGrantIssueCode::CycleDetected));
}

#[test]
fn test_validate_orphan_grant() {
    let orphan = create_test_grant("grant-orphan", Some("grant-ghost"), vec![CapabilityRight::Read], 1);

    let report = PepGrantRecoveryManager::validate_grants(&[orphan]);
    assert!(!report.is_valid);
    let orphan_issue = report.issues.iter().find(|i| i.code == PepGrantIssueCode::OrphanGrant);
    assert!(orphan_issue.is_some());
    assert_eq!(orphan_issue.unwrap().severity, PepGrantIssueSeverity::Error);
}

#[test]
fn test_validate_attenuation_violation() {
    // Parent only has Read + Delegate, child claims Write
    let parent = create_test_grant("grant-p", None, vec![CapabilityRight::Read, CapabilityRight::Delegate], 2);
    let child = create_test_grant("grant-c", Some("grant-p"), vec![CapabilityRight::Read, CapabilityRight::Write], 1);

    let report = PepGrantRecoveryManager::validate_grants(&[parent, child]);
    assert!(!report.is_valid);
    assert!(report.issues.iter().any(|i| i.code == PepGrantIssueCode::AttenuationViolation));
}

#[test]
fn test_validate_cascade_desync() {
    let mut parent = create_test_grant("grant-p", None, vec![CapabilityRight::Read, CapabilityRight::Delegate], 2);
    parent.state = PepGrantState::Revoked;
    let child = create_test_grant("grant-c", Some("grant-p"), vec![CapabilityRight::Read], 1);

    let report = PepGrantRecoveryManager::validate_grants(&[parent, child]);
    assert!(!report.is_valid);
    assert!(report.issues.iter().any(|i| i.code == PepGrantIssueCode::CascadeDesync));
}

#[test]
fn test_recover_corrupt_json_quarantine() {
    let dir = tempfile::tempdir().unwrap();
    let store_path = dir.path().join("grants.json");

    // Write invalid/corrupt JSON
    std::fs::write(&store_path, "{ broken json content !!!").unwrap();

    let res = PepGrantRecoveryManager::recover_store_file(&store_path, false).unwrap();
    assert!(res.ok);
    assert_eq!(res.repaired_count, 1);
    assert!(res.quarantine_path.is_some());

    // Verify quarantined file exists and store is restored to clean state
    let qpath = res.quarantine_path.unwrap();
    assert!(std::path::Path::new(&qpath).exists());
    assert!(store_path.exists());

    let val = PepGrantRecoveryManager::validate_store_file(&store_path).unwrap();
    assert!(val.is_valid);
    assert_eq!(val.total_grants, 0);
}

#[test]
fn test_recover_auto_repair_orphans_and_cascade() {
    let dir = tempfile::tempdir().unwrap();
    let store_path = dir.path().join("grants.json");

    // Setup a store with an orphan and an un-cascaded revoked parent
    let mut parent = create_test_grant("grant-p", None, vec![CapabilityRight::Read, CapabilityRight::Delegate], 3);
    parent.state = PepGrantState::Revoked;
    let child = create_test_grant("grant-c", Some("grant-p"), vec![CapabilityRight::Read, CapabilityRight::Delegate], 2);
    let grandchild = create_test_grant("grant-gc", Some("grant-c"), vec![CapabilityRight::Read], 1);
    let orphan = create_test_grant("grant-orphan", Some("grant-missing"), vec![CapabilityRight::Read], 1);

    let service = PepGrantService::from_grants(vec![parent, child, grandchild, orphan]);
    service.save_to_path(&store_path).unwrap();

    // Verify pre-repair report is invalid
    let pre_val = PepGrantRecoveryManager::validate_store_file(&store_path).unwrap();
    assert!(!pre_val.is_valid);

    // Run recovery
    let res = PepGrantRecoveryManager::recover_store_file(&store_path, false).unwrap();
    assert!(res.ok);
    assert!(res.backup_path.is_some());
    assert!(res.repaired_count >= 3); // orphan revoked, child cascade, grandchild cascade

    // Post-repair report should be valid
    let post_val = PepGrantRecoveryManager::validate_store_file(&store_path).unwrap();
    assert!(post_val.is_valid);
    assert_eq!(post_val.healthy_grants, 4);
}
