//! Automated Integration Test Suite for AIOS Package Management Subsystem.
//!
//! Enforces criteria PT1..PT5:
//! - PT1: Transaction Plan Determinism & Reproducibility
//! - PT2: Multi-Step Lifecycle Cohesion
//! - PT3: Dependency Closure Failure Modes & Negative Bounds
//! - PT4: Configuration-Governed Store Bounds Enforcement
//! - PT5: Transaction Anti-Tamper & Rollback Integrity

use aiosh_core::package::*;
use aiosh_core::package_config::*;
use aiosh_core::package_service::*;

/// Helper for constructing synthetic test packages with specified parameters.
pub fn create_synthetic_package(
    name: &str,
    version: &str,
    format: PackageFormat,
    state: PackageState,
    size_bytes: u64,
    dependencies: Vec<PackageDependency>,
) -> PackageSpec {
    PackageSpec {
        name: name.to_string(),
        version: version.to_string(),
        architecture: "x86_64".to_string(),
        format,
        state,
        description: format!("Synthetic test package for {}", name),
        installed_size_bytes: size_bytes,
        sha256: Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()),
        repository_url: Some("https://deb.debian.org/debian".to_string()),
        dependencies,
    }
}

#[test]
fn test_pt1_plan_determinism_and_reproducibility() {
    let mut store = PackageStore::new();

    let p1 = create_synthetic_package("pt1-alpha", "1.0.0", PackageFormat::Deb, PackageState::Available, 1024, vec![]);
    let p2 = create_synthetic_package("pt1-beta", "2.1.0", PackageFormat::Deb, PackageState::Available, 2048, vec![]);
    let p3 = create_synthetic_package("pt1-gamma", "3.0.0", PackageFormat::Deb, PackageState::Available, 4096, vec![]);

    store.register_package(p1).expect("register p1");
    store.register_package(p2).expect("register p2");
    store.register_package(p3).expect("register p3");

    let actions = vec![
        PackageAction {
            package_name: "pt1-alpha".into(),
            action: PackageActionType::Install,
            target_version: None,
        },
        PackageAction {
            package_name: "pt1-beta".into(),
            action: PackageActionType::Install,
            target_version: None,
        },
        PackageAction {
            package_name: "pt1-gamma".into(),
            action: PackageActionType::Install,
            target_version: None,
        },
    ];

    let initial_plan = store.plan_transaction(actions.clone(), false).expect("initial plan");

    for _ in 0..50 {
        let plan = store.plan_transaction(actions.clone(), false).expect("subsequent plan");
        assert_eq!(initial_plan.id, plan.id);
        assert_eq!(initial_plan.total_size_delta_bytes, plan.total_size_delta_bytes);
        assert_eq!(initial_plan.actions.len(), plan.actions.len());
        for (a1, a2) in initial_plan.actions.iter().zip(plan.actions.iter()) {
            assert_eq!(a1.package_name, a2.package_name);
            assert_eq!(a1.action, a2.action);
            assert_eq!(a1.target_version, a2.target_version);
        }
    }

    // Assert dry_run produces identical transaction hash and delta
    let dry_plan = store.plan_transaction(actions, true).expect("dry plan");
    assert_eq!(initial_plan.id, dry_plan.id);
    assert_eq!(initial_plan.total_size_delta_bytes, dry_plan.total_size_delta_bytes);
    assert!(dry_plan.dry_run);
}

#[test]
fn test_pt2_multi_step_lifecycle_cohesion() {
    let mut store = PackageStore::empty();

    // Step 0: Register initial package in Available state
    let pkg_v1 = create_synthetic_package(
        "pt2-editor",
        "1.0.0",
        PackageFormat::Deb,
        PackageState::Available,
        500_000,
        vec![],
    );
    store.register_package(pkg_v1).expect("register v1");

    // Step 1: Install
    let install_actions = vec![PackageAction {
        package_name: "pt2-editor".into(),
        action: PackageActionType::Install,
        target_version: None,
    }];
    let install_tx = store.plan_transaction(install_actions, false).expect("plan install");
    assert_eq!(install_tx.total_size_delta_bytes, 500_000);

    let install_report = store.execute_transaction(&install_tx).expect("execute install");
    assert_eq!(install_report.packages_installed, vec!["pt2-editor"]);
    assert_eq!(
        store.get_package("pt2-editor").unwrap().state,
        PackageState::Installed
    );

    // Step 2: Upgrade
    let mut pkg_v2 = create_synthetic_package(
        "pt2-editor",
        "1.2.0",
        PackageFormat::Deb,
        PackageState::Upgradable,
        550_000,
        vec![],
    );
    // Replace spec in store
    let _ = store.unregister_package("pt2-editor");
    pkg_v2.state = PackageState::Upgradable;
    store.register_package(pkg_v2).expect("register v2");

    let upgrade_actions = vec![PackageAction {
        package_name: "pt2-editor".into(),
        action: PackageActionType::Upgrade,
        target_version: Some("1.2.0".into()),
    }];
    let upgrade_tx = store.plan_transaction(upgrade_actions, false).expect("plan upgrade");
    let upgrade_report = store.execute_transaction(&upgrade_tx).expect("execute upgrade");
    assert_eq!(upgrade_report.packages_upgraded, vec!["pt2-editor"]);
    let current_pkg = store.get_package("pt2-editor").unwrap();
    assert_eq!(current_pkg.state, PackageState::Installed);
    assert_eq!(current_pkg.version, "1.2.0");

    // Step 3: Remove
    let remove_actions = vec![PackageAction {
        package_name: "pt2-editor".into(),
        action: PackageActionType::Remove,
        target_version: None,
    }];
    let remove_tx = store.plan_transaction(remove_actions, false).expect("plan remove");
    assert_eq!(remove_tx.total_size_delta_bytes, -550_000);

    let remove_report = store.execute_transaction(&remove_tx).expect("execute remove");
    assert_eq!(remove_report.packages_removed, vec!["pt2-editor"]);
    assert_eq!(
        store.get_package("pt2-editor").unwrap().state,
        PackageState::Available
    );
}

#[test]
fn test_pt3_dependency_closure_failure_modes() {
    let mut store = PackageStore::empty();

    // Case A: Missing required dependency
    let leaf_pkg = create_synthetic_package(
        "pt3-leaf",
        "1.0.0",
        PackageFormat::Deb,
        PackageState::Available,
        100_000,
        vec![PackageDependency {
            name: "nonexistent-runtime".into(),
            version_constraint: None,
            optional: false,
        }],
    );
    store.register_package(leaf_pkg).expect("register leaf");

    let plan_err = store.plan_transaction(
        vec![PackageAction {
            package_name: "pt3-leaf".into(),
            action: PackageActionType::Install,
            target_version: None,
        }],
        false,
    );
    assert!(plan_err.is_err());
    assert!(plan_err.unwrap_err().contains("invariant CS3 violated: unmet dependency"));

    // Case B: Dependency exists but is not installed and not in actions
    let dep_core = create_synthetic_package(
        "pt3-core",
        "1.0.0",
        PackageFormat::Deb,
        PackageState::Available,
        200_000,
        vec![],
    );
    let dep_plugin = create_synthetic_package(
        "pt3-plugin",
        "1.0.0",
        PackageFormat::Deb,
        PackageState::Available,
        50_000,
        vec![PackageDependency {
            name: "pt3-core".into(),
            version_constraint: None,
            optional: false,
        }],
    );
    store.register_package(dep_core).expect("register dep_core");
    store.register_package(dep_plugin).expect("register dep_plugin");

    let plugin_only_err = store.plan_transaction(
        vec![PackageAction {
            package_name: "pt3-plugin".into(),
            action: PackageActionType::Install,
            target_version: None,
        }],
        false,
    );
    assert!(plugin_only_err.is_err());
    assert!(plugin_only_err.unwrap_err().contains("requires 'pt3-core'"));

    // Satisfied when both are in transaction actions
    let joint_plan = store.plan_transaction(
        vec![
            PackageAction {
                package_name: "pt3-core".into(),
                action: PackageActionType::Install,
                target_version: None,
            },
            PackageAction {
                package_name: "pt3-plugin".into(),
                action: PackageActionType::Install,
                target_version: None,
            },
        ],
        false,
    );
    assert!(joint_plan.is_ok());

    // Case C: Target package not in store
    let missing_pkg_err = store.plan_transaction(
        vec![PackageAction {
            package_name: "phantom-tool".into(),
            action: PackageActionType::Install,
            target_version: None,
        }],
        false,
    );
    assert!(missing_pkg_err.is_err());
    assert!(missing_pkg_err.unwrap_err().contains("not found in store"));
}

#[test]
fn test_pt4_config_governed_store_bounds() {
    let mut config = PackageConfig::default();
    assert!(config.validate().is_ok());

    // PC2 ceiling validation: max_store_size_bytes must be >= 64 KiB and <= 100 MiB
    config.max_store_size_bytes = 32 * 1024; // Too small
    assert!(config.validate().is_err());

    config.max_store_size_bytes = 200 * 1024 * 1024; // Too big
    assert!(config.validate().is_err());

    config.max_store_size_bytes = 10 * 1024 * 1024; // Valid 10 MiB

    // PC3 entity count validation: [10 .. 100,000]
    config.max_entity_count = 5; // Too small
    assert!(config.validate().is_err());

    config.max_entity_count = 500_000; // Too big
    assert!(config.validate().is_err());

    config.max_entity_count = 50; // Valid

    // Persistence with store
    let tmp = tempfile::tempdir().unwrap();
    let store_file = tmp.path().join("pt4_store.json");

    let mut store = PackageStore::empty();
    for i in 0..15 {
        let pkg = create_synthetic_package(
            &format!("pt4-pkg-{:02}", i),
            "1.0.0",
            PackageFormat::Deb,
            PackageState::Available,
            1024,
            vec![],
        );
        store.register_package(pkg).expect("register");
    }

    assert_eq!(store.list_packages().len(), 15);
    store.save_to_path(&store_file).expect("save store");

    let loaded = PackageStore::load_from_path(&store_file).expect("load store");
    assert_eq!(loaded.list_packages().len(), 15);
    assert!(loaded.get_package("pt4-pkg-00").is_some());
    assert!(loaded.get_package("pt4-pkg-14").is_some());
}

#[test]
fn test_pt5_anti_tamper_and_rollback_integrity() {
    let mut store = PackageStore::empty();

    let pkg = create_synthetic_package(
        "pt5-guard",
        "1.0.0",
        PackageFormat::Deb,
        PackageState::Available,
        300_000,
        vec![],
    );
    store.register_package(pkg).expect("register");

    let actions = vec![PackageAction {
        package_name: "pt5-guard".into(),
        action: PackageActionType::Install,
        target_version: None,
    }];
    let mut valid_tx = store.plan_transaction(actions, false).expect("plan");
    assert_eq!(valid_tx.total_size_delta_bytes, 300_000);

    // Tamper delta to violate CS4
    valid_tx.total_size_delta_bytes = 999_999;
    let tamper_res = store.execute_transaction(&valid_tx);
    assert!(tamper_res.is_err());
    assert!(tamper_res.unwrap_err().contains("invariant CS4 violated"));

    // Verify pristine state preserved (rollback)
    assert_eq!(
        store.get_package("pt5-guard").unwrap().state,
        PackageState::Available
    );

    // Test dry-run non-mutation
    let dry_actions = vec![PackageAction {
        package_name: "pt5-guard".into(),
        action: PackageActionType::Install,
        target_version: None,
    }];
    let dry_tx = store.plan_transaction(dry_actions, true).expect("dry plan");
    let dry_report = store.execute_transaction(&dry_tx).expect("execute dry-run");
    assert_eq!(dry_report.packages_installed, vec!["pt5-guard"]);
    // Store MUST remain Available because dry_run = true
    assert_eq!(
        store.get_package("pt5-guard").unwrap().state,
        PackageState::Available
    );
}

#[test]
fn test_pt6_boundary_and_negative_matrix() {
    let mut store = PackageStore::empty();

    // 1. Empty actions list rejected
    let empty_err = store.plan_transaction(vec![], false);
    assert!(empty_err.is_err());
    assert!(empty_err.unwrap_err().contains("cannot be empty"));

    // 2. Action count limit exceeded (> 256)
    let p = create_synthetic_package("pt6-dummy", "1.0.0", PackageFormat::Deb, PackageState::Available, 100, vec![]);
    store.register_package(p).expect("register");
    let mut too_many_actions = Vec::new();
    for _ in 0..257 {
        too_many_actions.push(PackageAction {
            package_name: "pt6-dummy".into(),
            action: PackageActionType::Install,
            target_version: None,
        });
    }
    let overflow_err = store.plan_transaction(too_many_actions, false);
    assert!(overflow_err.is_err());
    assert!(overflow_err.unwrap_err().contains("exceeds 256 entries"));

    // 3. Unregister nonexistent package rejected
    let unreg_err = store.unregister_package("nonexistent-package");
    assert!(unreg_err.is_err());
    assert!(unreg_err.unwrap_err().contains("not found in store"));

    // 4. Duplicate registration rejected (CS1)
    let duplicate_spec = create_synthetic_package("pt6-dummy", "1.0.1", PackageFormat::Deb, PackageState::Available, 100, vec![]);
    let dup_err = store.register_package(duplicate_spec);
    assert!(dup_err.is_err());
    assert!(dup_err.unwrap_err().contains("invariant CS1 violated"));
}
