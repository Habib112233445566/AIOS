//! Automated Integration Test Suite for AIOS Init & Service Supervision Subsystem.
//!
//! Enforces criteria ST1..ST5:
//! - ST1: Multi-Turn Lifecycle FSM Cohesion & State Invariants
//! - ST2: Dependency DAG Topological Order & Cycle Detection
//! - ST3: Store Persistence & Atomic Recovery
//! - ST4: Configuration-Governed Quotas & Supervision Boundaries
//! - ST5: Filtered Query & Catalog Introspection

use std::collections::BTreeMap;
use std::path::PathBuf;
use aiosh_core::service::*;
use aiosh_core::service_config::*;
use aiosh_core::service_service::*;

/// Helper for constructing synthetic test services with specified parameters.
pub fn create_synthetic_service(
    name: &str,
    description: &str,
    service_type: ServiceType,
    startup_mode: ServiceStartupMode,
    dependencies: Vec<ServiceDependency>,
) -> ServiceSpec {
    ServiceSpec {
        name: name.to_string(),
        description: description.to_string(),
        exec_start: format!("/usr/bin/{}", name.replace(".service", "")),
        exec_stop: None,
        exec_reload: None,
        service_type,
        restart_policy: ServiceRestartPolicy::OnFailure,
        startup_mode,
        user: None,
        group: None,
        working_dir: None,
        environment: BTreeMap::new(),
        dependencies,
        timeout_start_secs: 30,
        timeout_stop_secs: 30,
    }
}

#[test]
fn test_st1_lifecycle_fsm_cohesion_and_masking() {
    let mut store = ServiceStore::empty();

    let spec = create_synthetic_service(
        "st1-app.service",
        "ST1 Lifecycle Integration Service",
        ServiceType::Simple,
        ServiceStartupMode::Enabled,
        vec![],
    );
    store.register_service(spec).expect("register st1-app");

    // 1. Initial state is Inactive
    let st = store.get_status("st1-app.service").unwrap();
    assert_eq!(st.state, ServiceState::Inactive);
    assert_eq!(st.startup_mode, ServiceStartupMode::Enabled);

    // 2. Start
    let rep_start = store.execute_action("st1-app.service", ServiceAction::Start).unwrap();
    assert_eq!(rep_start.new_state, ServiceState::Active);
    assert_eq!(store.get_status("st1-app.service").unwrap().state, ServiceState::Active);

    // 3. Reload while Active
    let rep_reload = store.execute_action("st1-app.service", ServiceAction::Reload).unwrap();
    assert_eq!(rep_reload.new_state, ServiceState::Active);

    // 4. Restart
    let rep_restart = store.execute_action("st1-app.service", ServiceAction::Restart).unwrap();
    assert_eq!(rep_restart.new_state, ServiceState::Active);

    // 5. Attempting to mask an active service MUST fail
    let err_mask_active = store.execute_action("st1-app.service", ServiceAction::Mask).unwrap_err();
    assert!(err_mask_active.contains("cannot mask active running service"));

    // 6. Stop
    let rep_stop = store.execute_action("st1-app.service", ServiceAction::Stop).unwrap();
    assert_eq!(rep_stop.new_state, ServiceState::Inactive);

    // 7. Disable
    let rep_disable = store.execute_action("st1-app.service", ServiceAction::Disable).unwrap();
    assert_eq!(store.get_status("st1-app.service").unwrap().startup_mode, ServiceStartupMode::Disabled);
    assert_eq!(rep_disable.success, true);

    // 8. Mask
    let rep_mask = store.execute_action("st1-app.service", ServiceAction::Mask).unwrap();
    assert_eq!(store.get_status("st1-app.service").unwrap().startup_mode, ServiceStartupMode::Masked);
    assert_eq!(rep_mask.success, true);

    // 9. Attempting to start or restart a masked service MUST fail immediately
    let err_start_masked = store.execute_action("st1-app.service", ServiceAction::Start).unwrap_err();
    assert!(err_start_masked.contains("cannot start masked service"));

    let err_restart_masked = store.execute_action("st1-app.service", ServiceAction::Restart).unwrap_err();
    assert!(err_restart_masked.contains("cannot restart masked service"));

    // 10. Unmask and re-enable
    let rep_unmask = store.execute_action("st1-app.service", ServiceAction::Unmask).unwrap();
    assert_eq!(store.get_status("st1-app.service").unwrap().startup_mode, ServiceStartupMode::Disabled);
    assert_eq!(rep_unmask.success, true);

    let rep_enable = store.execute_action("st1-app.service", ServiceAction::Enable).unwrap();
    assert_eq!(store.get_status("st1-app.service").unwrap().startup_mode, ServiceStartupMode::Enabled);
    assert_eq!(rep_enable.success, true);

    // 11. Start succeeds after unmask
    let rep_start_final = store.execute_action("st1-app.service", ServiceAction::Start).unwrap();
    assert_eq!(rep_start_final.new_state, ServiceState::Active);
}

#[test]
fn test_st2_dependency_dag_order_and_cycle_detection() {
    let mut store = ServiceStore::empty();

    let db = create_synthetic_service("st2-db.service", "Database", ServiceType::Simple, ServiceStartupMode::Enabled, vec![]);
    let cache = create_synthetic_service("st2-cache.service", "Cache", ServiceType::Simple, ServiceStartupMode::Enabled, vec![]);

    let backend = create_synthetic_service(
        "st2-backend.service",
        "Backend API",
        ServiceType::Simple,
        ServiceStartupMode::Enabled,
        vec![
            ServiceDependency {
                name: "st2-db.service".into(),
                dependency_type: ServiceDependencyType::Requires,
                optional: false,
            },
            ServiceDependency {
                name: "st2-cache.service".into(),
                dependency_type: ServiceDependencyType::Requires,
                optional: false,
            },
        ],
    );

    let frontend = create_synthetic_service(
        "st2-frontend.service",
        "Frontend Gateway",
        ServiceType::Simple,
        ServiceStartupMode::Enabled,
        vec![ServiceDependency {
            name: "st2-backend.service".into(),
            dependency_type: ServiceDependencyType::Requires,
            optional: false,
        }],
    );

    store.register_service(db).unwrap();
    store.register_service(cache).unwrap();
    store.register_service(backend).unwrap();
    store.register_service(frontend).unwrap();

    // Verify deterministic topological order (Kahn's algorithm)
    let order = store.plan_service_order("st2-frontend.service").unwrap();
    assert_eq!(order.len(), 4);

    let idx_db = order.iter().position(|s| s == "st2-db.service").unwrap();
    let idx_cache = order.iter().position(|s| s == "st2-cache.service").unwrap();
    let idx_backend = order.iter().position(|s| s == "st2-backend.service").unwrap();
    let idx_frontend = order.iter().position(|s| s == "st2-frontend.service").unwrap();

    assert!(idx_db < idx_backend);
    assert!(idx_cache < idx_backend);
    assert!(idx_backend < idx_frontend);

    // Verify cyclic dependency rejection
    let mut cycle_store = ServiceStore::empty();
    let svc_a = create_synthetic_service(
        "cycle-a.service",
        "Cycle A",
        ServiceType::Simple,
        ServiceStartupMode::Enabled,
        vec![ServiceDependency {
            name: "cycle-b.service".into(),
            dependency_type: ServiceDependencyType::Requires,
            optional: false,
        }],
    );
    let svc_b = create_synthetic_service(
        "cycle-b.service",
        "Cycle B",
        ServiceType::Simple,
        ServiceStartupMode::Enabled,
        vec![ServiceDependency {
            name: "cycle-a.service".into(),
            dependency_type: ServiceDependencyType::Requires,
            optional: false,
        }],
    );
    cycle_store.register_service(svc_a).unwrap();
    cycle_store.register_service(svc_b).unwrap();

    let err_cycle = cycle_store.plan_service_order("cycle-a.service").unwrap_err();
    assert!(err_cycle.contains("cyclic dependency detected") || err_cycle.contains("cycle"));
}

#[test]
fn test_st3_store_persistence_and_atomic_recovery() {
    let temp_dir = std::env::temp_dir();
    let store_file = temp_dir.join(format!("aios_st3_auto_test_{}.json", std::process::id()));

    let mut store = ServiceStore::empty();
    let svc1 = create_synthetic_service("st3-svc1.service", "Service 1", ServiceType::Simple, ServiceStartupMode::Enabled, vec![]);
    let svc2 = create_synthetic_service("st3-svc2.service", "Service 2", ServiceType::Notify, ServiceStartupMode::Disabled, vec![]);

    store.register_service(svc1).unwrap();
    store.register_service(svc2).unwrap();

    store.execute_action("st3-svc1.service", ServiceAction::Start).unwrap();

    // Persist store to path
    store.save_to_path(&store_file).expect("save store to path");
    assert!(store_file.exists());

    // Reload store from path
    let reloaded = ServiceStore::load_from_path(&store_file).expect("load store from path");
    assert_eq!(reloaded.list_services().len(), 2);
    assert_eq!(reloaded.get_status("st3-svc1.service").unwrap().state, ServiceState::Active);
    assert_eq!(reloaded.get_status("st3-svc2.service").unwrap().state, ServiceState::Inactive);
    assert_eq!(reloaded.get_status("st3-svc2.service").unwrap().startup_mode, ServiceStartupMode::Disabled);

    let _ = std::fs::remove_file(&store_file);
}

#[test]
fn test_st4_configuration_governed_quotas() {
    let mut cfg = ServiceConfig::default();
    assert_eq!(cfg.validate(), Ok(()));

    // Valid configuration defaults
    assert_eq!(cfg.store_path, PathBuf::from(DEFAULT_SERVICE_STORE_PATH));
    assert_eq!(cfg.default_timeout_start_secs, 30);
    assert_eq!(cfg.default_timeout_stop_secs, 30);
    assert!(cfg.auto_persist);

    // Boundary quota validation
    cfg.max_entity_count = 5; // sub-minimum
    assert!(cfg.validate().unwrap_err().contains("SC4 violation"));

    cfg = ServiceConfig::default();
    cfg.max_store_size_bytes = 100; // sub-minimum
    assert!(cfg.validate().unwrap_err().contains("SC3 violation"));
}

#[test]
fn test_st5_filtered_query_and_catalog_introspection() {
    let mut store = ServiceStore::empty();

    let s1 = create_synthetic_service("db-postgres.service", "PostgreSQL database", ServiceType::Forking, ServiceStartupMode::Enabled, vec![]);
    let s2 = create_synthetic_service("db-redis.service", "Redis cache", ServiceType::Simple, ServiceStartupMode::Enabled, vec![]);
    let s3 = create_synthetic_service("web-nginx.service", "Nginx reverse proxy", ServiceType::Forking, ServiceStartupMode::Disabled, vec![]);

    store.register_service(s1).unwrap();
    store.register_service(s2).unwrap();
    store.register_service(s3).unwrap();

    store.execute_action("db-postgres.service", ServiceAction::Start).unwrap();

    // 1. Query by pattern
    let q_pat = ServiceQuery {
        name_pattern: Some("redis".into()),
        state: None,
        startup_mode: None,
        limit: None,
    };
    let res_pat = store.query(&q_pat);
    assert_eq!(res_pat.len(), 1);
    assert_eq!(res_pat[0].name, "db-redis.service");

    // 2. Query by state
    let q_active = ServiceQuery {
        name_pattern: None,
        state: Some(ServiceState::Active),
        startup_mode: None,
        limit: None,
    };
    let res_active = store.query(&q_active);
    assert_eq!(res_active.len(), 1);
    assert_eq!(res_active[0].name, "db-postgres.service");

    // 3. Query by startup mode
    let q_disabled = ServiceQuery {
        name_pattern: None,
        state: None,
        startup_mode: Some(ServiceStartupMode::Disabled),
        limit: None,
    };
    let res_disabled = store.query(&q_disabled);
    assert_eq!(res_disabled.len(), 1);
    assert_eq!(res_disabled[0].name, "web-nginx.service");

    // 4. Query with limit
    let q_limit = ServiceQuery {
        name_pattern: Some("db-".into()),
        state: None,
        startup_mode: None,
        limit: Some(1),
    };
    let res_limit = store.query(&q_limit);
    assert_eq!(res_limit.len(), 1);
}

#[test]
fn test_st6_boundary_failure_modes_and_unmet_dependencies() {
    let mut store = ServiceStore::empty();

    // 1. Action on non-existent service returns Err
    let err_missing = store.execute_action("missing.service", ServiceAction::Start).unwrap_err();
    assert!(err_missing.contains("not found"));

    // 2. Unmet dependency during order planning
    let broken_svc = create_synthetic_service(
        "broken.service",
        "Broken Dependency Service",
        ServiceType::Simple,
        ServiceStartupMode::Enabled,
        vec![ServiceDependency {
            name: "nonexistent-prereq.service".into(),
            dependency_type: ServiceDependencyType::Requires,
            optional: false,
        }],
    );
    store.register_service(broken_svc).unwrap();

    let err_order = store.plan_service_order("broken.service").unwrap_err();
    assert!(err_order.contains("unmet dependency"));

    // 3. Duplicate registration fails invariant CS1
    let dup_spec = create_synthetic_service(
        "broken.service",
        "Duplicate Service",
        ServiceType::Simple,
        ServiceStartupMode::Enabled,
        vec![],
    );
    let err_dup = store.register_service(dup_spec).unwrap_err();
    assert!(err_dup.contains("CS1") || err_dup.contains("already registered"));

    // 4. Invalid service name fails syntax validation (SS1)
    let invalid_spec = create_synthetic_service(
        "-invalid-starting-dash.service",
        "Invalid syntax",
        ServiceType::Simple,
        ServiceStartupMode::Enabled,
        vec![],
    );
    assert!(store.register_service(invalid_spec).is_err());
}
