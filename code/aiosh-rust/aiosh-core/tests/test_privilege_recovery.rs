//! Unit and Integration Tests for Privilege Escalation Prevention Recovery & Validation (T-02595).

use std::fs;
use tempfile::tempdir;

use aiosh_core::privilege_data_model::*;
use aiosh_core::privilege_recovery::*;
use aiosh_core::privilege_service::*;

#[test]
fn test_privilege_recovery_non_existent_file() {
    let dir = tempdir().unwrap();
    let missing_path = dir.path().join("missing_privileges.json");

    let report = PrivilegeRecoveryManager::validate_store_file(&missing_path).expect("validate missing");
    assert!(report.is_valid);
    assert_eq!(report.total_contexts, 0);

    let repair_res = PrivilegeRecoveryManager::repair_store_file(&missing_path).expect("repair missing");
    assert!(repair_res.ok);
    assert!(missing_path.exists());
}

#[test]
fn test_privilege_recovery_healthy_store() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("healthy_privileges.json");

    let mut service = PrivilegeService::new();
    let c1 = PrivilegeContext::new("alice", PrivilegeLevel::User).unwrap();
    let c2 = PrivilegeContext::new("bob", PrivilegeLevel::Operator).unwrap();
    service.register_context(c1).unwrap();
    service.register_context(c2).unwrap();
    service.save_to_path(&store_path).unwrap();

    let report = PrivilegeRecoveryManager::validate_store_file(&store_path).expect("validate healthy");
    assert!(report.is_valid);
    assert_eq!(report.total_contexts, 2);
    assert_eq!(report.healthy_contexts, 2);
    assert!(report.issues.is_empty());
}

#[test]
fn test_privilege_recovery_illegal_kernel_detection_and_repair() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("tampered_privileges.json");

    // Construct tampered JSON with SystemKernel
    let tampered_json = r#"{
        "contexts": {
            "attacker": {
                "actor_id": "attacker",
                "active_level": "system_kernel",
                "is_elevation_active": true,
                "elevation_grant_id": "grant-fake-001",
                "capabilities": ["kernel_module_load"],
                "session_id": null
            }
        }
    }"#;
    fs::write(&store_path, tampered_json).unwrap();

    // 1. Validation detects illegal kernel tier
    let report = PrivilegeRecoveryManager::validate_store_file(&store_path).expect("validate tampered");
    assert!(!report.is_valid);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].code, PrivilegeIssueCode::IllegalKernelTier);
    assert_eq!(report.issues[0].severity, PrivilegeIssueSeverity::Fatal);

    // 2. Repair demotes actor to User and creates backup
    let res = PrivilegeRecoveryManager::repair_store_file(&store_path).expect("repair tampered");
    assert!(res.ok);
    assert!(res.backup_path.is_some());
    assert_eq!(res.repaired_count, 1);
    assert_eq!(res.actions[0].action, "demote_from_kernel");

    // 3. Post-validation is now valid
    assert!(res.post_validation.is_valid);

    // 4. Verify on-disk service state
    let repaired_service = PrivilegeService::load_from_path(&store_path).expect("load repaired");
    let ctx = repaired_service.get_context("attacker").expect("attacker context");
    assert_eq!(ctx.active_level, PrivilegeLevel::User);
}

#[test]
fn test_privilege_recovery_corrupted_json_quarantine() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("corrupted_privileges.json");

    // Corrupted garbage content
    fs::write(&store_path, "{ broken invalid json <<<").unwrap();

    // 1. Validation flags invalid JSON
    let report = PrivilegeRecoveryManager::validate_store_file(&store_path).expect("validate corrupt");
    assert!(!report.is_valid);
    assert_eq!(report.issues[0].code, PrivilegeIssueCode::InvalidJson);

    // 2. Repair creates quarantine and initializes clean store
    let res = PrivilegeRecoveryManager::repair_store_file(&store_path).expect("repair corrupt");
    assert!(res.ok);
    assert!(res.quarantine_path.is_some());

    let post = PrivilegeRecoveryManager::validate_store_file(&store_path).expect("validate after repair");
    assert!(post.is_valid);
}

#[test]
fn test_privilege_recovery_grant_inconsistency() {
    let raw = r#"{
        "contexts": {
            "user1": {
                "actor_id": "user1",
                "active_level": "operator",
                "is_elevation_active": true,
                "elevation_grant_id": null,
                "capabilities": [],
                "session_id": null
            }
        }
    }"#;
    let report = PrivilegeRecoveryManager::validate_raw_json(raw).expect("validate raw");
    assert!(!report.is_valid);
    assert_eq!(report.issues[0].code, PrivilegeIssueCode::GrantInconsistency);
}

#[test]
fn test_privilege_recovery_path_traversal_rejection() {
    let bad_path = std::path::Path::new("../secret/privileges.json");
    let res = PrivilegeRecoveryManager::validate_store_file(bad_path);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains(PRIVRECV_ERR_PATH_TRAVERSAL));
}

#[test]
fn test_privilege_recovery_capacity_bounds() {
    // Generate JSON with 16385 items
    let mut items = Vec::new();
    for i in 0..16385 {
        items.push(format!(r#"{{"actor_id":"user_{}","active_level":"user","is_elevation_active":false}}"#, i));
    }
    let large_json = format!("[{}]", items.join(","));
    let report = PrivilegeRecoveryManager::validate_raw_json(&large_json).expect("validate raw large");
    assert!(!report.is_valid);
    assert_eq!(report.issues[0].code, PrivilegeIssueCode::CapacityExceeded);
}

