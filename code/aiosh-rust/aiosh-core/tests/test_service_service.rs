//! Integration and unit test suite for Init & Service Supervision Core Service (CS1..CS5).

use aiosh_core::service::{
    ServiceAction, ServiceDependency, ServiceDependencyType, ServiceQuery,
    ServiceRestartPolicy, ServiceSpec, ServiceStartupMode, ServiceState, ServiceType,
};
use aiosh_core::service_service::ServiceStore;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

fn sample_custom_spec(name: &str) -> ServiceSpec {
    ServiceSpec {
        name: name.to_string(),
        description: "AIOS Custom Test Service".into(),
        exec_start: format!("/usr/bin/{}", name),
        exec_stop: Some(format!("/usr/bin/{} --stop", name)),
        exec_reload: Some(format!("/usr/bin/{} --reload", name)),
        service_type: ServiceType::Simple,
        restart_policy: ServiceRestartPolicy::OnFailure,
        startup_mode: ServiceStartupMode::Enabled,
        user: Some("aios".into()),
        group: Some("aios".into()),
        working_dir: Some("/var/lib/aios".into()),
        environment: BTreeMap::new(),
        dependencies: vec![],
        timeout_start_secs: 30,
        timeout_stop_secs: 30,
    }
}

#[test]
fn test_service_store_seeding_and_lookup() {
    let store = ServiceStore::new();
    assert_eq!(store.services.len(), 6);
    assert_eq!(store.statuses.len(), 6);

    // Verify canonical services exist
    let auditd = store.get_service("auditd.service").expect("auditd must exist");
    assert_eq!(auditd.service_type, ServiceType::Forking);
    assert_eq!(auditd.startup_mode, ServiceStartupMode::Enabled);

    let auditd_status = store.get_status("auditd.service").expect("auditd status must exist");
    assert_eq!(auditd_status.state, ServiceState::Active);
    assert!(auditd_status.health.healthy);
    assert!(auditd_status.pid.is_some());

    let dbus = store.get_service("dbus.service").expect("dbus must exist");
    assert_eq!(dbus.service_type, ServiceType::Simple);

    let securityd = store.get_service("aios-securityd.service").expect("aios-securityd must exist");
    assert_eq!(securityd.dependencies.len(), 2);

    let ssh = store.get_service("ssh.service").expect("ssh must exist");
    assert_eq!(ssh.dependencies.len(), 1);

    // Non-existent lookup
    assert!(store.get_service("non-existent.service").is_none());
    assert!(store.get_status("non-existent.service").is_none());
}

#[test]
fn test_service_store_cs1_uniqueness_and_lifecycle() {
    let mut store = ServiceStore::empty();
    assert_eq!(store.services.len(), 0);
    assert_eq!(store.statuses.len(), 0);

    let spec = sample_custom_spec("aios-telemetry.service");

    // Register success
    assert!(store.register_service(spec.clone()).is_ok());
    assert_eq!(store.services.len(), 1);
    assert_eq!(store.statuses.len(), 1);

    let status = store.get_status("aios-telemetry.service").unwrap();
    assert_eq!(status.state, ServiceState::Inactive);
    assert_eq!(status.startup_mode, ServiceStartupMode::Enabled);
    assert!(status.pid.is_none());

    // Duplicate registration fails invariant CS1
    let dup_err = store.register_service(spec).unwrap_err();
    assert!(dup_err.contains("CS1"));
    assert!(dup_err.contains("already registered"));

    // Invalid spec fails validation
    let invalid_spec = sample_custom_spec("invalid service name with spaces");
    let err = store.register_service(invalid_spec).unwrap_err();
    assert!(err.contains("service name contains invalid character"));

    // Unregister success
    let unregistered = store.unregister_service("aios-telemetry.service").unwrap();
    assert_eq!(unregistered.name, "aios-telemetry.service");
    assert_eq!(store.services.len(), 0);
    assert_eq!(store.statuses.len(), 0);

    // Unregister non-existent fails
    let not_found_err = store.unregister_service("aios-telemetry.service").unwrap_err();
    assert!(not_found_err.contains("not found"));
}

#[test]
fn test_service_store_cs2_fsm_lifecycle_actions() {
    let mut store = ServiceStore::empty();
    let spec = sample_custom_spec("aios-worker.service");
    store.register_service(spec).unwrap();

    // Initial state: Inactive
    assert_eq!(
        store.get_status("aios-worker.service").unwrap().state,
        ServiceState::Inactive
    );

    // Reload on Inactive service fails
    let reload_err = store
        .execute_action("aios-worker.service", ServiceAction::Reload)
        .unwrap_err();
    assert!(reload_err.contains("cannot reload inactive service"));

    // Start action
    let start_report = store
        .execute_action("aios-worker.service", ServiceAction::Start)
        .unwrap();
    assert!(start_report.success);
    assert_eq!(start_report.previous_state, ServiceState::Inactive);
    assert_eq!(start_report.new_state, ServiceState::Active);

    let active_status = store.get_status("aios-worker.service").unwrap();
    assert_eq!(active_status.state, ServiceState::Active);
    assert_eq!(active_status.pid, Some(1001));
    assert!(active_status.health.healthy);
    assert!(active_status.started_at.is_some());

    // Reload on Active service succeeds
    let reload_report = store
        .execute_action("aios-worker.service", ServiceAction::Reload)
        .unwrap();
    assert!(reload_report.success);
    assert_eq!(reload_report.new_state, ServiceState::Active);

    // Restart action
    let restart_report = store
        .execute_action("aios-worker.service", ServiceAction::Restart)
        .unwrap();
    assert!(restart_report.success);
    assert_eq!(restart_report.new_state, ServiceState::Active);
    let restarted_status = store.get_status("aios-worker.service").unwrap();
    assert_eq!(restarted_status.pid, Some(1002));
    assert_eq!(restarted_status.health.restarts, 1);

    // Mask on Active service is forbidden
    let mask_err = store
        .execute_action("aios-worker.service", ServiceAction::Mask)
        .unwrap_err();
    assert!(mask_err.contains("cannot mask active running service"));

    // Stop action
    let stop_report = store
        .execute_action("aios-worker.service", ServiceAction::Stop)
        .unwrap();
    assert!(stop_report.success);
    assert_eq!(stop_report.new_state, ServiceState::Inactive);

    let stopped_status = store.get_status("aios-worker.service").unwrap();
    assert_eq!(stopped_status.state, ServiceState::Inactive);
    assert!(stopped_status.pid.is_none());
    assert!(stopped_status.started_at.is_none());

    // Disable action
    let disable_report = store
        .execute_action("aios-worker.service", ServiceAction::Disable)
        .unwrap();
    assert!(disable_report.success);
    assert_eq!(
        store.get_service("aios-worker.service").unwrap().startup_mode,
        ServiceStartupMode::Disabled
    );
    assert_eq!(
        store.get_status("aios-worker.service").unwrap().startup_mode,
        ServiceStartupMode::Disabled
    );

    // Enable action
    let enable_report = store
        .execute_action("aios-worker.service", ServiceAction::Enable)
        .unwrap();
    assert!(enable_report.success);
    assert_eq!(
        store.get_service("aios-worker.service").unwrap().startup_mode,
        ServiceStartupMode::Enabled
    );

    // Mask action when stopped
    let mask_report = store
        .execute_action("aios-worker.service", ServiceAction::Mask)
        .unwrap();
    assert!(mask_report.success);
    assert_eq!(
        store.get_service("aios-worker.service").unwrap().startup_mode,
        ServiceStartupMode::Masked
    );
    assert_eq!(
        store.get_status("aios-worker.service").unwrap().startup_mode,
        ServiceStartupMode::Masked
    );

    // Starting a masked service is blocked
    let start_masked_err = store
        .execute_action("aios-worker.service", ServiceAction::Start)
        .unwrap_err();
    assert!(start_masked_err.contains("cannot start masked service"));

    // Restarting a masked service is blocked
    let restart_masked_err = store
        .execute_action("aios-worker.service", ServiceAction::Restart)
        .unwrap_err();
    assert!(restart_masked_err.contains("cannot restart masked service"));

    // Unmask action
    let unmask_report = store
        .execute_action("aios-worker.service", ServiceAction::Unmask)
        .unwrap();
    assert!(unmask_report.success);
    assert_eq!(
        store.get_service("aios-worker.service").unwrap().startup_mode,
        ServiceStartupMode::Disabled
    );
}

#[test]
fn test_service_store_cs3_topological_ordering_and_cycle_detection() {
    let store = ServiceStore::new();

    // aios-securityd requires auditd and dbus
    let sec_order = store.plan_service_order("aios-securityd.service").unwrap();
    assert_eq!(sec_order.len(), 3);
    assert_eq!(sec_order[2], "aios-securityd.service");
    let auditd_pos = sec_order.iter().position(|s| s == "auditd.service").unwrap();
    let dbus_pos = sec_order.iter().position(|s| s == "dbus.service").unwrap();
    let sec_pos = sec_order.iter().position(|s| s == "aios-securityd.service").unwrap();
    assert!(auditd_pos < sec_pos);
    assert!(dbus_pos < sec_pos);

    // ssh depends on network-manager, which depends on dbus
    let ssh_order = store.plan_service_order("ssh.service").unwrap();
    assert_eq!(ssh_order.len(), 3);
    assert_eq!(ssh_order[2], "ssh.service");
    let dbus_pos = ssh_order.iter().position(|s| s == "dbus.service").unwrap();
    let nm_pos = ssh_order.iter().position(|s| s == "network-manager.service").unwrap();
    let ssh_pos = ssh_order.iter().position(|s| s == "ssh.service").unwrap();
    assert!(dbus_pos < nm_pos);
    assert!(nm_pos < ssh_pos);

    // Missing dependency detection
    let mut broken_store = ServiceStore::empty();
    let mut broken_spec = sample_custom_spec("broken.service");
    broken_spec.dependencies.push(ServiceDependency {
        name: "nonexistent.service".into(),
        dependency_type: ServiceDependencyType::Requires,
        optional: false,
    });
    broken_store.register_service(broken_spec).unwrap();

    let missing_err = broken_store.plan_service_order("broken.service").unwrap_err();
    assert!(missing_err.contains("unmet dependency"));
    assert!(missing_err.contains("nonexistent.service"));

    // Cycle detection
    let mut cyclic_store = ServiceStore::empty();
    let mut svc_a = sample_custom_spec("cycle-a.service");
    svc_a.dependencies.push(ServiceDependency {
        name: "cycle-b.service".into(),
        dependency_type: ServiceDependencyType::Requires,
        optional: false,
    });

    let mut svc_b = sample_custom_spec("cycle-b.service");
    svc_b.dependencies.push(ServiceDependency {
        name: "cycle-a.service".into(),
        dependency_type: ServiceDependencyType::Requires,
        optional: false,
    });

    cyclic_store.register_service(svc_a).unwrap();
    cyclic_store.register_service(svc_b).unwrap();

    let cycle_err = cyclic_store.plan_service_order("cycle-a.service").unwrap_err();
    assert!(cycle_err.contains("CS3"));
    assert!(cycle_err.contains("cyclic dependency detected"));
}

#[test]
fn test_service_store_query_matrix() {
    let store = ServiceStore::new();

    // Query by name_pattern
    let q_audit = ServiceQuery {
        name_pattern: Some("auditd".into()),
        state: None,
        startup_mode: None,
        limit: None,
    };
    let results = store.query(&q_audit);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "auditd.service");

    // Query by description keyword
    let q_desc = ServiceQuery {
        name_pattern: Some("daemon".into()),
        state: None,
        startup_mode: None,
        limit: None,
    };
    let results = store.query(&q_desc);
    assert!(results.len() >= 2);

    // Query by startup_mode Static
    let q_static = ServiceQuery {
        name_pattern: None,
        state: None,
        startup_mode: Some(ServiceStartupMode::Static),
        limit: None,
    };
    let static_results = store.query(&q_static);
    assert_eq!(static_results.len(), 1);
    assert_eq!(static_results[0].name, "systemd-journald.service");

    // Query with limit
    let q_limited = ServiceQuery {
        name_pattern: None,
        state: None,
        startup_mode: Some(ServiceStartupMode::Enabled),
        limit: Some(2),
    };
    let limited = store.query(&q_limited);
    assert_eq!(limited.len(), 2);
}

#[test]
fn test_service_store_cs5_persistence_and_bounds() {
    let original_store = ServiceStore::new();
    let temp_dir = std::env::temp_dir();
    let store_path = temp_dir.join("test_aios_service_store.json");

    // Atomic save
    assert!(original_store.save_to_path(&store_path).is_ok());
    assert!(store_path.exists());

    // Atomic load
    let loaded_store = ServiceStore::load_from_path(&store_path).expect("load must succeed");
    assert_eq!(loaded_store.services.len(), 6);
    assert_eq!(loaded_store.statuses.len(), 6);
    assert_eq!(original_store, loaded_store);

    // Clean up
    let _ = std::fs::remove_file(&store_path);

    // Load non-existent file
    let not_found_path = temp_dir.join("non_existent_service_store.json");
    let err = ServiceStore::load_from_path(&not_found_path).unwrap_err();
    assert!(err.contains("does not exist"));

    // 10 MiB ceiling rejection
    let oversize_path = temp_dir.join("test_oversize_service_store.json");
    {
        let mut file = File::create(&oversize_path).unwrap();
        let chunk = vec![b' '; 1024 * 1024]; // 1 MiB
        for _ in 0..11 {
            // 11 MiB > 10 MiB
            file.write_all(&chunk).unwrap();
        }
    }
    let oversize_err = ServiceStore::load_from_path(&oversize_path).unwrap_err();
    assert!(oversize_err.contains("exceeds 10 MiB ceiling"));
    let _ = std::fs::remove_file(&oversize_path);
}
