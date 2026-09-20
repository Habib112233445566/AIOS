//! Unit tests for Capability Recovery and Validation Subsystem (CAPREC1..CAPREC6).

use std::fs;
use std::path::Path;
use tempfile::tempdir;

use aiosh_core::capability::{
    Capability, CapabilityConstraints, CapabilityRight, CapabilityScope,
};
use aiosh_core::capability_recovery::{
    recover_capability_store, validate_capability_store, CapabilityRecoveryAction,
    CapabilityValidationReport,
};
use aiosh_core::capability_service::CapabilityService;

#[test]
fn test_recovery_empty_service_validation() {
    let service = CapabilityService::new();
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("store.json");

    let report = validate_capability_store(&service, &store_path);
    assert_eq!(report.total_capabilities, 0);
    assert_eq!(report.valid_capabilities, 0);
    assert_eq!(report.invalid_capabilities, 0);
    assert!(report.errors.is_empty());
    assert!(report.healthy);
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_recovery_valid_hierarchy_validation() {
    let mut service = CapabilityService::new();
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("store.json");

    let root = service
        .issue_root_capability(
            "kernel",
            "agent:root",
            CapabilityScope::Filesystem {
                path: "/workspace".into(),
                recursive: true,
            },
            vec![CapabilityRight::Read, CapabilityRight::Write, CapabilityRight::Delegate],
            CapabilityConstraints::default(),
        )
        .expect("issue root");

    let _child = service
        .attenuate_capability(
            &root.id,
            "agent:child",
            None,
            vec![CapabilityRight::Read],
            None,
        )
        .expect("attenuate child");

    let report = validate_capability_store(&service, &store_path);
    assert_eq!(report.total_capabilities, 2);
    assert_eq!(report.valid_capabilities, 2);
    assert_eq!(report.invalid_capabilities, 0);
    assert!(report.errors.is_empty());
    assert!(report.healthy);
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_recovery_dangling_parent_validation() {
    let mut service = CapabilityService::new();
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("store.json");

    // Manually insert capability with dangling parent_id
    let dangling_cap = Capability {
        id: "cap_dangling".into(),
        parent_id: Some("cap_nonexistent_parent".into()),
        issuer: "kernel".into(),
        subject: "agent:orphan".into(),
        scope: CapabilityScope::System { subsystem: "test".into() },
        rights: vec![CapabilityRight::Read],
        constraints: CapabilityConstraints::default(),
        revoked: false,
        created_at: "2026-09-20T18:00:00Z".into(),
    };
    service.capabilities_mut().insert(dangling_cap.id.clone(), dangling_cap);

    let report = validate_capability_store(&service, &store_path);
    assert_eq!(report.total_capabilities, 1);
    assert_eq!(report.valid_capabilities, 0);
    assert_eq!(report.invalid_capabilities, 1);
    assert!(!report.healthy);
    assert!(report.errors.iter().any(|e| e.contains("references non-existent parent")));
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_recovery_cycle_detection_validation() {
    let mut service = CapabilityService::new();
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("store.json");

    // Manually insert two capabilities referencing each other in a cycle
    let cap_a = Capability {
        id: "cap_cycle_a".into(),
        parent_id: Some("cap_cycle_b".into()),
        issuer: "kernel".into(),
        subject: "agent:a".into(),
        scope: CapabilityScope::System { subsystem: "test".into() },
        rights: vec![CapabilityRight::Read],
        constraints: CapabilityConstraints::default(),
        revoked: false,
        created_at: "2026-09-20T18:00:00Z".into(),
    };
    let cap_b = Capability {
        id: "cap_cycle_b".into(),
        parent_id: Some("cap_cycle_a".into()),
        issuer: "kernel".into(),
        subject: "agent:b".into(),
        scope: CapabilityScope::System { subsystem: "test".into() },
        rights: vec![CapabilityRight::Read],
        constraints: CapabilityConstraints::default(),
        revoked: false,
        created_at: "2026-09-20T18:00:00Z".into(),
    };
    service.capabilities_mut().insert(cap_a.id.clone(), cap_a);
    service.capabilities_mut().insert(cap_b.id.clone(), cap_b);

    let report = validate_capability_store(&service, &store_path);
    assert_eq!(report.total_capabilities, 2);
    assert_eq!(report.invalid_capabilities, 2);
    assert!(!report.healthy);
    assert!(report.errors.iter().any(|e| e.contains("cycle detected")));
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_recovery_privilege_escalation_validation() {
    let mut service = CapabilityService::new();
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("store.json");

    let root = Capability {
        id: "cap_root".into(),
        parent_id: None,
        issuer: "kernel".into(),
        subject: "agent:root".into(),
        scope: CapabilityScope::System { subsystem: "test".into() },
        rights: vec![CapabilityRight::Read],
        constraints: CapabilityConstraints::default(),
        revoked: false,
        created_at: "2026-09-20T18:00:00Z".into(),
    };
    let child_escalated = Capability {
        id: "cap_child_escalated".into(),
        parent_id: Some(root.id.clone()),
        issuer: "agent:root".into(),
        subject: "agent:child".into(),
        scope: CapabilityScope::System { subsystem: "test".into() },
        rights: vec![CapabilityRight::Read, CapabilityRight::Admin], // Escalated!
        constraints: CapabilityConstraints::default(),
        revoked: false,
        created_at: "2026-09-20T18:00:00Z".into(),
    };

    service.capabilities_mut().insert(root.id.clone(), root);
    service.capabilities_mut().insert(child_escalated.id.clone(), child_escalated);

    let report = validate_capability_store(&service, &store_path);
    assert_eq!(report.total_capabilities, 2);
    assert_eq!(report.valid_capabilities, 1);
    assert_eq!(report.invalid_capabilities, 1);
    assert!(!report.healthy);
    assert!(report.errors.iter().any(|e| e.contains("privilege escalation")));
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_recovery_missing_store_creates_default() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("sub").join("nonexistent_store.json");

    let (service, action, report) = recover_capability_store(&store_path).expect("recover");
    assert_eq!(action, CapabilityRecoveryAction::CreatedDefaultFresh);
    assert_eq!(service.capabilities().len(), 0);
    assert!(report.healthy);
    assert!(store_path.exists());
}

#[test]
fn test_recovery_clean_existing_store() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("clean_store.json");

    let mut original_service = CapabilityService::new();
    let _root = original_service
        .issue_root_capability(
            "kernel",
            "agent:test",
            CapabilityScope::System { subsystem: "core".into() },
            vec![CapabilityRight::Read],
            CapabilityConstraints::default(),
        )
        .expect("issue root");
    original_service.save_to_path(&store_path).expect("save");

    let (loaded_service, action, report) = recover_capability_store(&store_path).expect("recover");
    assert_eq!(action, CapabilityRecoveryAction::LoadedExisting);
    assert_eq!(loaded_service.capabilities().len(), 1);
    assert!(report.healthy);
}

#[test]
fn test_recovery_corrupted_store_quarantine() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("corrupt_store.json");

    // Write completely invalid JSON to simulate disk corruption
    fs::write(&store_path, "{ broken json content !!!").expect("write corrupt");

    let (recovered_service, action, report) = recover_capability_store(&store_path).expect("recover");
    match action {
        CapabilityRecoveryAction::RecoveredFromBackup { backup_path, reason } => {
            assert!(Path::new(&backup_path).exists(), "Quarantine backup was not created!");
            assert!(reason.contains("corrupted store file"));
            // Verify original damaged content is preserved in backup
            let backup_content = fs::read_to_string(&backup_path).expect("read backup");
            assert_eq!(backup_content, "{ broken json content !!!");
        }
        _ => panic!("Expected RecoveredFromBackup, got {:?}", action),
    }

    // Recovered store must now be healthy and valid JSON
    assert_eq!(recovered_service.capabilities().len(), 0);
    assert!(report.healthy);
    assert!(store_path.exists());
}

#[test]
fn test_recovery_report_invariants() {
    // 1. Invariant SSR1 / CAPREC1 check (valid + invalid == total)
    let bad_partition_report = CapabilityValidationReport {
        store_path: "store.json".into(),
        total_capabilities: 5,
        valid_capabilities: 3,
        invalid_capabilities: 1, // 3 + 1 != 5
        errors: vec!["some error".into()],
        warnings: Vec::new(),
        healthy: false,
        evaluated_at: "2026-09-20T18:00:00Z".into(),
    };
    assert!(bad_partition_report.validate_invariants().is_err());

    // 2. Invariant SSR2 / CAPREC2 check (healthy mismatch)
    let bad_health_report = CapabilityValidationReport {
        store_path: "store.json".into(),
        total_capabilities: 2,
        valid_capabilities: 1,
        invalid_capabilities: 1,
        errors: vec!["some error".into()],
        warnings: Vec::new(),
        healthy: true, // Should be false!
        evaluated_at: "2026-09-20T18:00:00Z".into(),
    };
    assert!(bad_health_report.validate_invariants().is_err());
}
