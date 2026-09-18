//! Focused automated unit and integration tests for Init & Service Supervision Recovery & Validation.
//!
//! Enforces invariants SR1..SR5, deep specification validation, topological acyclicity,
//! non-destructive quarantine backups, and self-healing canonical reconstitution.

use std::collections::BTreeMap;
use std::path::Path;
use aiosh_core::service::{
    ServiceDependency, ServiceDependencyType, ServiceRestartPolicy, ServiceSpec,
    ServiceStartupMode, ServiceState, ServiceStatus, ServiceHealth, ServiceType,
};
use aiosh_core::service_recovery::{
    load_or_recover, recover_service_store_with_backup, validate_service_store,
    ServiceValidationReport,
};
use aiosh_core::service_service::ServiceStore;

#[test]
fn test_sr1_sr2_sr3_invariant_equations() {
    // 1. SR1 violation: valid + invalid != total
    let report_bad_sr1 = ServiceValidationReport {
        store_path: "/tmp/svc.json".into(),
        total_services: 10,
        valid_services: 5,
        invalid_services: 3, // sum is 8 != 10
        errors: vec!["err1".into(), "err2".into(), "err3".into()],
        warnings: vec![],
        healthy: false,
        evaluated_at: "2026-09-09T00:00:00Z".into(),
    };
    let err = report_bad_sr1.validate_invariants().unwrap_err();
    assert!(err.contains("SR1 violated"));

    // 2. SR2 violation: healthy is true but errors exist
    let report_bad_sr2a = ServiceValidationReport {
        store_path: "/tmp/svc.json".into(),
        total_services: 2,
        valid_services: 2,
        invalid_services: 0,
        errors: vec!["hidden error".into()],
        warnings: vec![],
        healthy: true, // violation
        evaluated_at: "2026-09-09T00:00:00Z".into(),
    };
    let err = report_bad_sr2a.validate_invariants().unwrap_err();
    assert!(err.contains("SR2 violated"));

    // 3. SR2 violation: healthy is true but invalid_services > 0
    let report_bad_sr2b = ServiceValidationReport {
        store_path: "/tmp/svc.json".into(),
        total_services: 2,
        valid_services: 1,
        invalid_services: 1,
        errors: vec![],
        warnings: vec![],
        healthy: true, // violation
        evaluated_at: "2026-09-09T00:00:00Z".into(),
    };
    let err = report_bad_sr2b.validate_invariants().unwrap_err();
    assert!(err.contains("SR2 violated"));

    // 4. SR3 violation: errors.len() < invalid_services
    let report_bad_sr3 = ServiceValidationReport {
        store_path: "/tmp/svc.json".into(),
        total_services: 5,
        valid_services: 2,
        invalid_services: 3,
        errors: vec!["single error".into()], // only 1 error for 3 invalid services
        warnings: vec![],
        healthy: false,
        evaluated_at: "2026-09-09T00:00:00Z".into(),
    };
    let err = report_bad_sr3.validate_invariants().unwrap_err();
    assert!(err.contains("SR3 violated"));

    // 5. Valid report satisfying all invariants
    let report_valid = ServiceValidationReport {
        store_path: "/tmp/svc.json".into(),
        total_services: 5,
        valid_services: 3,
        invalid_services: 2,
        errors: vec!["err 1".into(), "err 2".into()],
        warnings: vec![],
        healthy: false,
        evaluated_at: "2026-09-09T00:00:00Z".into(),
    };
    assert!(report_valid.validate_invariants().is_ok());
}

#[test]
fn test_default_store_deep_validation() {
    let store = ServiceStore::new();
    let report = validate_service_store(&store, Path::new("/var/lib/aios/services.json"));

    assert!(report.healthy, "default canonical store must be healthy");
    assert_eq!(report.invalid_services, 0);
    assert!(report.valid_services >= 5);
    assert_eq!(report.valid_services, report.total_services);
    assert!(report.errors.is_empty());
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_negative_service_specs_and_status_invariants() {
    let mut store = ServiceStore::empty();

    // 1. Invalid service name syntax (starts with hyphen)
    let bad_name_spec = ServiceSpec {
        name: "-bad.service".into(),
        description: "Bad Name".into(),
        exec_start: "/bin/true".into(),
        exec_stop: None,
        exec_reload: None,
        service_type: ServiceType::Simple,
        restart_policy: ServiceRestartPolicy::No,
        startup_mode: ServiceStartupMode::Enabled,
        user: None,
        group: None,
        working_dir: None,
        environment: BTreeMap::new(),
        dependencies: vec![],
        timeout_start_secs: 30,
        timeout_stop_secs: 30,
    };
    store.services.insert("-bad.service".into(), bad_name_spec);

    // 2. Relative execution / working_dir traversal violation
    let bad_exec_spec = ServiceSpec {
        name: "bad-exec.service".into(),
        description: "Relative Working Dir".into(),
        exec_start: "/bin/bad".into(),
        exec_stop: None,
        exec_reload: None,
        service_type: ServiceType::Simple,
        restart_policy: ServiceRestartPolicy::No,
        startup_mode: ServiceStartupMode::Enabled,
        user: None,
        group: None,
        working_dir: Some("../relative/dir".into()),
        environment: BTreeMap::new(),
        dependencies: vec![],
        timeout_start_secs: 30,
        timeout_stop_secs: 30,
    };
    store.services.insert("bad-exec.service".into(), bad_exec_spec);

    // 3. Store key mismatch
    let mismatched_key_spec = ServiceSpec {
        name: "real-name.service".into(),
        description: "Key Mismatch".into(),
        exec_start: "/bin/true".into(),
        exec_stop: None,
        exec_reload: None,
        service_type: ServiceType::Simple,
        restart_policy: ServiceRestartPolicy::No,
        startup_mode: ServiceStartupMode::Enabled,
        user: None,
        group: None,
        working_dir: None,
        environment: BTreeMap::new(),
        dependencies: vec![],
        timeout_start_secs: 30,
        timeout_stop_secs: 30,
    };
    store.services.insert("wrong-key.service".into(), mismatched_key_spec);

    // 4. Invalid status
    store.statuses.insert("orphan.service".into(), ServiceStatus {
        name: "orphan.service".into(),
        state: ServiceState::Active,
        startup_mode: ServiceStartupMode::Enabled,
        pid: Some(999),
        health: ServiceHealth {
            healthy: true,
            exit_code: None,
            pid: Some(999),
            uptime_seconds: Some(120),
            restarts: 0,
            last_error: None,
        },
        started_at: None,
    });

    let report = validate_service_store(&store, Path::new("/var/lib/aios/services.json"));
    assert!(!report.healthy);
    assert_eq!(report.invalid_services, 3);
    assert!(report.errors.len() >= 3);
    assert!(report.warnings.iter().any(|w| w.contains("orphan.service")));
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_dependency_cycle_detection_in_store() {
    let mut store = ServiceStore::empty();
    let spec_a = ServiceSpec {
        name: "svc-alpha.service".into(),
        description: "Alpha".into(),
        exec_start: "/bin/alpha".into(),
        exec_stop: None,
        exec_reload: None,
        service_type: ServiceType::Simple,
        restart_policy: ServiceRestartPolicy::No,
        startup_mode: ServiceStartupMode::Enabled,
        user: None,
        group: None,
        working_dir: None,
        environment: BTreeMap::new(),
        dependencies: vec![ServiceDependency {
            name: "svc-beta.service".into(),
            dependency_type: ServiceDependencyType::Requires,
            optional: false,
        }],
        timeout_start_secs: 30,
        timeout_stop_secs: 30,
    };

    let spec_b = ServiceSpec {
        name: "svc-beta.service".into(),
        description: "Beta".into(),
        exec_start: "/bin/beta".into(),
        exec_stop: None,
        exec_reload: None,
        service_type: ServiceType::Simple,
        restart_policy: ServiceRestartPolicy::No,
        startup_mode: ServiceStartupMode::Enabled,
        user: None,
        group: None,
        working_dir: None,
        environment: BTreeMap::new(),
        dependencies: vec![ServiceDependency {
            name: "svc-alpha.service".into(),
            dependency_type: ServiceDependencyType::Requires,
            optional: false,
        }],
        timeout_start_secs: 30,
        timeout_stop_secs: 30,
    };

    store.services.insert("svc-alpha.service".into(), spec_a);
    store.services.insert("svc-beta.service".into(), spec_b);

    let report = validate_service_store(&store, Path::new("/var/lib/aios/services.json"));
    assert!(!report.healthy);
    assert!(report.errors.iter().any(|e| e.contains("cyclic dependency")));
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_non_destructive_corruption_recovery_and_quarantine() {
    let temp_dir = std::env::temp_dir().join(format!("aios_service_rec_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let store_path = temp_dir.join("services.json");

    // 1. Write corrupted JSON payload
    std::fs::write(&store_path, b"--- CORRUPT UNPARSEABLE JSON ---").unwrap();

    // 2. Perform recovery
    let (recovered_store, backup_path) = recover_service_store_with_backup(&store_path);

    // 3. Verify backup file was created and contains the original corrupt data
    assert!(backup_path.is_some(), "backup file must be created on corruption");
    let bak = backup_path.unwrap();
    assert!(bak.exists(), "quarantined backup file must exist on disk");
    assert!(bak.to_string_lossy().contains(".corrupt."));
    let bak_content = std::fs::read(&bak).unwrap();
    assert_eq!(bak_content, b"--- CORRUPT UNPARSEABLE JSON ---");

    // 4. Verify recovered store is written to the original path and is completely healthy
    assert!(store_path.exists());
    let report = validate_service_store(&recovered_store, &store_path);
    assert!(report.healthy, "recovered store must be healthy");
    assert_eq!(report.invalid_services, 0);
    assert!(report.valid_services >= 5);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_load_or_recover_workflow() {
    let temp_dir = std::env::temp_dir().join(format!("aios_service_workflow_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let store_path = temp_dir.join("services.json");

    // Phase 1: File does not exist -> creates default fresh store
    let (s1, rep1, rec1, bak1) = load_or_recover(&store_path).unwrap();
    assert!(rec1, "must flag that creation/recovery occurred");
    assert!(bak1.is_none(), "no backup needed when initial file was absent");
    assert!(rep1.healthy);
    assert!(store_path.exists());
    assert_eq!(s1.services.len(), rep1.total_services);

    // Phase 2: File exists and is healthy -> loads existing without recovery
    let (s2, rep2, rec2, bak2) = load_or_recover(&store_path).unwrap();
    assert!(!rec2, "healthy store must not trigger recovery");
    assert!(bak2.is_none());
    assert!(rep2.healthy);
    assert_eq!(s2.services.len(), s1.services.len());

    // Phase 3: Corrupt file -> triggers quarantine and recovery
    std::fs::write(&store_path, b"{ \"truncated\": ").unwrap();
    let (_s3, rep3, rec3, bak3) = load_or_recover(&store_path).unwrap();
    assert!(rec3, "damaged store must trigger recovery");
    assert!(bak3.is_some(), "damaged store must generate quarantine backup");
    assert!(rep3.healthy);
    assert!(bak3.unwrap().exists());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_service_recovery_hardening() {
    let mut store = ServiceStore::empty();
    // 1. Capacity overflow test: > 10,000 services
    for i in 0..10_005 {
        let name = format!("svc-{}.service", i);
        store.services.insert(name.clone(), ServiceSpec {
            name: name.clone(),
            description: "Hardening test service".into(),
            exec_start: "/bin/true".into(),
            exec_stop: None,
            exec_reload: None,
            service_type: ServiceType::Simple,
            restart_policy: ServiceRestartPolicy::No,
            startup_mode: ServiceStartupMode::Enabled,
            user: None,
            group: None,
            working_dir: None,
            environment: BTreeMap::new(),
            dependencies: vec![],
            timeout_start_secs: 30,
            timeout_stop_secs: 30,
        });
    }

    let report = validate_service_store(&store, Path::new("/var/lib/aios/services.json"));
    assert!(!report.healthy);
    assert!(report.errors.iter().any(|e| e.contains("exceeds maximum capacity of 10,000 services")));
    assert!(report.validate_invariants().is_ok());

    // 2. Temp directory with special characters and cleanup verification
    let temp_dir = std::env::temp_dir().join(format!("aios service rec test {}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let store_path = temp_dir.join("services store.json");

    std::fs::write(&store_path, b"malformed!").unwrap();
    let (s, rep, rec, bak) = load_or_recover(&store_path).unwrap();
    assert!(rec);
    assert!(rep.healthy);
    assert!(bak.is_some());
    assert!(store_path.exists());
    assert_eq!(s.services.len(), rep.total_services);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

