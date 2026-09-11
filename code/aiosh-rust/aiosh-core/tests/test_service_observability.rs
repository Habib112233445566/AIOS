//! Unit test suite for AIOS Init & Service Supervision Observability Subsystem (SO1..SO6).

use std::collections::BTreeMap;
use aiosh_core::service::*;
use aiosh_core::service_observability::*;
use aiosh_core::service_policy::*;
use aiosh_core::service_service::*;

fn make_svc(
    name: &str,
    st: ServiceType,
    rp: ServiceRestartPolicy,
    sm: ServiceStartupMode,
    deps: usize,
) -> ServiceSpec {
    let mut env = BTreeMap::new();
    env.insert("AIOS_ENV".into(), "test".into());

    let dependencies = (0..deps)
        .map(|i| ServiceDependency {
            name: format!("dep-{}.service", i),
            dependency_type: ServiceDependencyType::Wants,
            optional: true,
        })
        .collect();

    ServiceSpec {
        name: name.into(),
        description: format!("Test service {}", name),
        exec_start: format!("/usr/bin/{}", name),
        exec_stop: None,
        exec_reload: None,
        service_type: st,
        restart_policy: rp,
        startup_mode: sm,
        user: Some("aios".into()),
        group: Some("aios".into()),
        working_dir: Some("/var/lib/aios".into()),
        environment: env,
        dependencies,
        timeout_start_secs: 30,
        timeout_stop_secs: 30,
    }
}

#[test]
fn test_so1_inventory_completeness_and_empty_store() {
    let empty_store = ServiceStore::empty();
    let r_empty = ServiceObservabilityReport::generate(&empty_store, None);

    assert_eq!(r_empty.total_services, 0);
    assert_eq!(r_empty.healthy_count, 0);
    assert_eq!(r_empty.unhealthy_count, 0);
    assert_eq!(r_empty.total_restarts, 0);
    assert!(r_empty.failed_services.is_empty());
    assert_eq!(r_empty.policy_compliant_count, 0);
    assert_eq!(r_empty.policy_violations_count, 0);
    assert!(r_empty.prohibited_services_found.is_empty());
    assert!(r_empty.state_breakdown.is_empty());
    assert!(r_empty.startup_mode_breakdown.is_empty());
    assert!(r_empty.service_type_breakdown.is_empty());
    assert!(r_empty.restart_policy_breakdown.is_empty());
    assert_eq!(r_empty.dependency_distribution.get("0"), Some(&0));
    assert_eq!(r_empty.dependency_distribution.get("1-2"), Some(&0));
    assert_eq!(r_empty.dependency_distribution.get("3-5"), Some(&0));
    assert_eq!(r_empty.dependency_distribution.get("6+"), Some(&0));

    // Default store completeness
    let def_store = ServiceStore::new();
    let r_def = ServiceObservabilityReport::generate(&def_store, None);
    assert!(r_def.total_services > 0);

    let state_sum: usize = r_def.state_breakdown.values().sum();
    let mode_sum: usize = r_def.startup_mode_breakdown.values().sum();
    let type_sum: usize = r_def.service_type_breakdown.values().sum();
    let policy_sum: usize = r_def.restart_policy_breakdown.values().sum();
    let dep_sum: usize = r_def.dependency_distribution.values().sum();

    assert_eq!(state_sum, r_def.total_services);
    assert_eq!(mode_sum, r_def.total_services);
    assert_eq!(type_sum, r_def.total_services);
    assert_eq!(policy_sum, r_def.total_services);
    assert_eq!(dep_sum, r_def.total_services);
}

#[test]
fn test_so2_categorical_distributions() {
    let mut store = ServiceStore::empty();
    store.register_service(make_svc("app-a.service", ServiceType::Simple, ServiceRestartPolicy::Always, ServiceStartupMode::Enabled, 0)).unwrap();
    store.register_service(make_svc("app-b.service", ServiceType::Forking, ServiceRestartPolicy::OnFailure, ServiceStartupMode::Disabled, 1)).unwrap();
    store.register_service(make_svc("app-c.service", ServiceType::Oneshot, ServiceRestartPolicy::No, ServiceStartupMode::Masked, 2)).unwrap();
    store.register_service(make_svc("app-d.service", ServiceType::Notify, ServiceRestartPolicy::Always, ServiceStartupMode::Static, 0)).unwrap();

    let report = ServiceObservabilityReport::generate(&store, None);
    assert_eq!(report.total_services, 4);

    // Startup mode breakdown
    assert_eq!(report.startup_mode_breakdown.get("enabled"), Some(&1));
    assert_eq!(report.startup_mode_breakdown.get("disabled"), Some(&1));
    assert_eq!(report.startup_mode_breakdown.get("masked"), Some(&1));
    assert_eq!(report.startup_mode_breakdown.get("static"), Some(&1));

    // Service type breakdown
    assert_eq!(report.service_type_breakdown.get("simple"), Some(&1));
    assert_eq!(report.service_type_breakdown.get("forking"), Some(&1));
    assert_eq!(report.service_type_breakdown.get("oneshot"), Some(&1));
    assert_eq!(report.service_type_breakdown.get("notify"), Some(&1));

    // Restart policy breakdown
    assert_eq!(report.restart_policy_breakdown.get("always"), Some(&2));
    assert_eq!(report.restart_policy_breakdown.get("on_failure"), Some(&1));
    assert_eq!(report.restart_policy_breakdown.get("no"), Some(&1));
}

#[test]
fn test_so3_health_and_restart_telemetry() {
    let mut store = ServiceStore::empty();
    let s1 = make_svc("healthy-app.service", ServiceType::Simple, ServiceRestartPolicy::Always, ServiceStartupMode::Enabled, 0);
    let s2 = make_svc("crashed-app.service", ServiceType::Simple, ServiceRestartPolicy::Always, ServiceStartupMode::Enabled, 0);
    store.register_service(s1).unwrap();
    store.register_service(s2).unwrap();

    // Set crashed-app to Failed state with unhealthy status and 5 restarts
    let failed_status = ServiceStatus {
        name: "crashed-app.service".into(),
        state: ServiceState::Failed,
        startup_mode: ServiceStartupMode::Enabled,
        pid: None,
        health: ServiceHealth {
            healthy: false,
            exit_code: Some(1),
            pid: None,
            uptime_seconds: Some(0),
            restarts: 5,
            last_error: Some("Segmentation fault".into()),
        },
        started_at: None,
    };
    store.statuses.insert("crashed-app.service".into(), failed_status);

    let report = ServiceObservabilityReport::generate(&store, None);
    assert_eq!(report.total_services, 2);
    assert_eq!(report.healthy_count, 1);
    assert_eq!(report.unhealthy_count, 1);
    assert_eq!(report.total_restarts, 5);
    assert_eq!(report.failed_services, vec!["crashed-app.service".to_string()]);
}

#[test]
fn test_so4_dependency_distribution_histogram() {
    let mut store = ServiceStore::empty();
    store.register_service(make_svc("dep-0.service", ServiceType::Simple, ServiceRestartPolicy::Always, ServiceStartupMode::Enabled, 0)).unwrap();
    store.register_service(make_svc("dep-2.service", ServiceType::Simple, ServiceRestartPolicy::Always, ServiceStartupMode::Enabled, 2)).unwrap();
    store.register_service(make_svc("dep-4.service", ServiceType::Simple, ServiceRestartPolicy::Always, ServiceStartupMode::Enabled, 4)).unwrap();
    store.register_service(make_svc("dep-8.service", ServiceType::Simple, ServiceRestartPolicy::Always, ServiceStartupMode::Enabled, 8)).unwrap();

    let report = ServiceObservabilityReport::generate(&store, None);
    assert_eq!(report.total_services, 4);

    assert_eq!(report.dependency_distribution.get("0"), Some(&1));
    assert_eq!(report.dependency_distribution.get("1-2"), Some(&1));
    assert_eq!(report.dependency_distribution.get("3-5"), Some(&1));
    assert_eq!(report.dependency_distribution.get("6+"), Some(&1));
}

#[test]
fn test_so5_security_policy_compliance() {
    let mut store = ServiceStore::empty();
    store.register_service(make_svc("safe-app.service", ServiceType::Simple, ServiceRestartPolicy::Always, ServiceStartupMode::Enabled, 0)).unwrap();
    store.register_service(make_svc("telnet.service", ServiceType::Simple, ServiceRestartPolicy::Always, ServiceStartupMode::Enabled, 0)).unwrap();

    let policy = ServiceSecurityPolicy::default();
    let report = ServiceObservabilityReport::generate(&store, Some(&policy));

    assert_eq!(report.total_services, 2);
    assert_eq!(report.policy_compliant_count, 1);
    assert!(report.policy_violations_count >= 1);
    assert_eq!(report.prohibited_services_found, vec!["telnet.service".to_string()]);
}

#[test]
fn test_so6_serialization_and_string_helpers() {
    let store = ServiceStore::new();
    let report = ServiceObservabilityReport::generate(&store, None);

    // Serialization roundtrip
    let json_str = report.to_json_pretty().expect("serialize pretty");
    let deserialized: ServiceObservabilityReport = serde_json::from_str(&json_str).expect("deserialize json");
    assert_eq!(report, deserialized);

    // Helper string mapping functions
    assert_eq!(service_type_to_str(ServiceType::Simple), "simple");
    assert_eq!(service_type_to_str(ServiceType::Exec), "exec");
    assert_eq!(service_type_to_str(ServiceType::Forking), "forking");
    assert_eq!(service_type_to_str(ServiceType::Oneshot), "oneshot");
    assert_eq!(service_type_to_str(ServiceType::Notify), "notify");
    assert_eq!(service_type_to_str(ServiceType::Idle), "idle");

    assert_eq!(service_state_to_str(ServiceState::Active), "active");
    assert_eq!(service_state_to_str(ServiceState::Inactive), "inactive");
    assert_eq!(service_state_to_str(ServiceState::Activating), "activating");
    assert_eq!(service_state_to_str(ServiceState::Deactivating), "deactivating");
    assert_eq!(service_state_to_str(ServiceState::Failed), "failed");
    assert_eq!(service_state_to_str(ServiceState::Reloading), "reloading");
    assert_eq!(service_state_to_str(ServiceState::Unknown), "unknown");

    assert_eq!(startup_mode_to_str(ServiceStartupMode::Enabled), "enabled");
    assert_eq!(startup_mode_to_str(ServiceStartupMode::Disabled), "disabled");
    assert_eq!(startup_mode_to_str(ServiceStartupMode::Masked), "masked");
    assert_eq!(startup_mode_to_str(ServiceStartupMode::Static), "static");

    assert_eq!(restart_policy_to_str(ServiceRestartPolicy::No), "no");
    assert_eq!(restart_policy_to_str(ServiceRestartPolicy::Always), "always");
    assert_eq!(restart_policy_to_str(ServiceRestartPolicy::OnSuccess), "on_success");
    assert_eq!(restart_policy_to_str(ServiceRestartPolicy::OnFailure), "on_failure");
    assert_eq!(restart_policy_to_str(ServiceRestartPolicy::OnAbnormal), "on_abnormal");
    assert_eq!(restart_policy_to_str(ServiceRestartPolicy::OnWatchdog), "on_watchdog");
    assert_eq!(restart_policy_to_str(ServiceRestartPolicy::OnAbort), "on_abort");
}

#[test]
fn test_so7_hardening_and_path_boundaries() {
    // 1. Valid generate_from_paths with None
    let res_none = ServiceObservabilityReport::generate_from_paths::<&str, &str>(None, None);
    assert!(res_none.is_ok());
    assert!(res_none.unwrap().total_services > 0);

    // 2. Control character in store path
    let res_bad_store = ServiceObservabilityReport::generate_from_paths::<&str, &str>(Some("bad\0store.json"), None);
    assert!(res_bad_store.is_err());
    assert!(res_bad_store.unwrap_err().contains("control characters"));

    // 3. Control character in policy path
    let res_bad_policy = ServiceObservabilityReport::generate_from_paths::<&str, &str>(None, Some("bad\0policy.json"));
    assert!(res_bad_policy.is_err());
    assert!(res_bad_policy.unwrap_err().contains("control characters"));

    // 4. Non-existent store path
    let res_nonexistent = ServiceObservabilityReport::generate_from_paths::<&str, &str>(Some("nonexistent_store_9999.json"), None);
    assert!(res_nonexistent.is_err());

    // 5. Oversized store path (>1024 characters)
    let long_store = format!("{}.json", "a".repeat(1025));
    let res_long = ServiceObservabilityReport::generate_from_paths::<&str, &str>(Some(&long_store), None);
    assert!(res_long.is_err());
    assert!(res_long.unwrap_err().contains("exceeds 1024 characters"));

    // 6. Temporary file roundtrip
    let temp_dir = std::env::temp_dir();
    let store_path = temp_dir.join(format!("aios_service_obs_store_{}.json", std::process::id()));
    let policy_path = temp_dir.join(format!("aios_service_obs_policy_{}.json", std::process::id()));

    let store = ServiceStore::new();
    store.save_to_path(&store_path).expect("save test store");

    let policy = ServiceSecurityPolicy::default();
    let policy_json = serde_json::to_string_pretty(&policy).expect("serialize policy");
    std::fs::write(&policy_path, policy_json).expect("write test policy");

    let res_files = ServiceObservabilityReport::generate_from_paths(Some(&store_path), Some(&policy_path));
    assert!(res_files.is_ok());
    let report = res_files.unwrap();
    assert!(report.total_services > 0);
    assert_eq!(report.policy_compliant_count, report.total_services);

    let _ = std::fs::remove_file(&store_path);
    let _ = std::fs::remove_file(&policy_path);
}
