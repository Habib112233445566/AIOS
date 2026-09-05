//! Integration test suite for Package Management Core Service (CS1..CS5).

use aiosh_core::package::{
    PackageAction, PackageActionType, PackageFormat, PackageQuery, PackageSpec, PackageState,
};
use aiosh_core::package_service::PackageStore;
use std::fs::File;
use std::io::Write;

#[test]
fn test_package_store_seeding_and_lookup() {
    let store = PackageStore::new();
    assert_eq!(store.packages.len(), 8);

    // Debian packages
    let libc6 = store.get_package("libc6").expect("libc6 must exist");
    assert_eq!(libc6.format, PackageFormat::Deb);
    assert_eq!(libc6.state, PackageState::Installed);

    let curl = store.get_package("curl").expect("curl must exist");
    assert_eq!(curl.state, PackageState::Available);
    assert_eq!(curl.dependencies.len(), 2);

    // Alpine packages
    let musl = store.get_package("musl").expect("musl must exist");
    assert_eq!(musl.format, PackageFormat::Apk);
    assert_eq!(musl.state, PackageState::Installed);

    let busybox = store.get_package("busybox").expect("busybox must exist");
    assert_eq!(busybox.state, PackageState::Installed);

    // Non-existent lookup
    assert!(store.get_package("non-existent-pkg").is_none());
}

#[test]
fn test_package_store_cs1_uniqueness_and_lifecycle() {
    let mut store = PackageStore::empty();

    let custom_spec = PackageSpec {
        name: "ripgrep".into(),
        version: "14.1.0".into(),
        architecture: "x86_64".into(),
        format: PackageFormat::Tarball,
        state: PackageState::Available,
        description: "fast line-oriented search tool".into(),
        installed_size_bytes: 8_388_608,
        sha256: Some("abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd".into()),
        repository_url: Some("https://github.com/BurntSushi/ripgrep".into()),
        dependencies: vec![],
    };

    // Register success
    assert!(store.register_package(custom_spec.clone()).is_ok());
    assert_eq!(store.packages.len(), 1);

    // Duplicate registration fails with CS1
    let err = store.register_package(custom_spec).unwrap_err();
    assert!(err.contains("CS1"));
    assert!(err.contains("already registered"));

    // Unregister success
    let removed = store.unregister_package("ripgrep").expect("unregister succeeds");
    assert_eq!(removed.name, "ripgrep");
    assert_eq!(store.packages.len(), 0);

    // Unregister non-existent fails
    let err = store.unregister_package("ripgrep").unwrap_err();
    assert!(err.contains("not found"));
}

#[test]
fn test_package_store_query_matrix() {
    let store = PackageStore::new();

    // Query Debian packages
    let deb_query = PackageQuery {
        name_pattern: None,
        format: Some(PackageFormat::Deb),
        state: None,
        limit: None,
    };
    let deb_results = store.query(&deb_query);
    assert_eq!(deb_results.len(), 5);
    assert!(deb_results.iter().all(|p| p.format == PackageFormat::Deb));

    // Query Alpine packages
    let apk_query = PackageQuery {
        name_pattern: None,
        format: Some(PackageFormat::Apk),
        state: None,
        limit: None,
    };
    let apk_results = store.query(&apk_query);
    assert_eq!(apk_results.len(), 3);
    assert!(apk_results.iter().all(|p| p.format == PackageFormat::Apk));

    // Query by substring and state
    let search_query = PackageQuery {
        name_pattern: Some("lib".into()),
        format: None,
        state: Some(PackageState::Available),
        limit: Some(1),
    };
    let search_results = store.query(&search_query);
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].name, "libssl3");
}

#[test]
fn test_package_store_cs2_determinism() {
    let store = PackageStore::new();
    let actions = vec![
        PackageAction {
            action: PackageActionType::Install,
            package_name: "libssl3".into(),
            target_version: None,
        },
        PackageAction {
            action: PackageActionType::Install,
            package_name: "curl".into(),
            target_version: None,
        },
    ];

    let plan_a = store.plan_transaction(actions.clone(), false).unwrap();
    let plan_b = store.plan_transaction(actions, false).unwrap();

    assert_eq!(plan_a, plan_b);
    assert_eq!(plan_a.id, plan_b.id);
    assert_eq!(plan_a.total_size_delta_bytes, plan_b.total_size_delta_bytes);
}

#[test]
fn test_package_store_cs3_dependency_closure() {
    let store = PackageStore::new();

    // Negative: curl requires libssl3 which is Available, not Installed
    let bad_actions = vec![PackageAction {
        action: PackageActionType::Install,
        package_name: "curl".into(),
        target_version: None,
    }];
    let err = store.plan_transaction(bad_actions, false).unwrap_err();
    assert!(err.contains("CS3"));
    assert!(err.contains("unmet dependency"));
    assert!(err.contains("libssl3"));

    // Positive: batch installing libssl3 alongside curl satisfies closure
    let good_actions = vec![
        PackageAction {
            action: PackageActionType::Install,
            package_name: "libssl3".into(),
            target_version: None,
        },
        PackageAction {
            action: PackageActionType::Install,
            package_name: "curl".into(),
            target_version: None,
        },
    ];
    assert!(store.plan_transaction(good_actions, false).is_ok());

    // Negative: target package does not exist
    let unknown_actions = vec![PackageAction {
        action: PackageActionType::Install,
        package_name: "non-existent".into(),
        target_version: None,
    }];
    let err = store.plan_transaction(unknown_actions, false).unwrap_err();
    assert!(err.contains("not found in store"));
}

#[test]
fn test_package_store_cs4_delta_arithmetic_and_tamper_detection() {
    let mut store = PackageStore::new();

    let install_actions = vec![
        PackageAction {
            action: PackageActionType::Install,
            package_name: "libssl3".into(),
            target_version: None,
        },
        PackageAction {
            action: PackageActionType::Install,
            package_name: "curl".into(),
            target_version: None,
        },
    ];

    let plan = store.plan_transaction(install_actions, false).unwrap();
    let expected_delta = 5_242_880 + 4_194_304;
    assert_eq!(plan.total_size_delta_bytes, expected_delta);

    // Negative: Tampered delta fails execute_transaction with CS4 error
    let mut tampered_tx = plan.clone();
    tampered_tx.total_size_delta_bytes += 100;
    let err = store.execute_transaction(&tampered_tx).unwrap_err();
    assert!(err.contains("CS4"));
    assert!(err.contains("delta"));
}

#[test]
fn test_package_store_dry_run_vs_actual_execution() {
    let mut store = PackageStore::new();

    let actions = vec![
        PackageAction {
            action: PackageActionType::Install,
            package_name: "libssl3".into(),
            target_version: None,
        },
        PackageAction {
            action: PackageActionType::Install,
            package_name: "curl".into(),
            target_version: None,
        },
    ];

    let plan = store.plan_transaction(actions, true).unwrap();
    assert!(plan.dry_run);

    // Execute dry run
    let report = store.execute_transaction(&plan).unwrap();
    assert!(report.success);
    assert_eq!(report.packages_installed.len(), 2);
    // In dry run, store is unmodified
    assert_eq!(store.get_package("curl").unwrap().state, PackageState::Available);
    assert_eq!(store.get_package("libssl3").unwrap().state, PackageState::Available);

    // Execute real transaction
    let mut real_tx = plan.clone();
    real_tx.dry_run = false;
    let real_report = store.execute_transaction(&real_tx).unwrap();
    assert!(real_report.success);
    assert_eq!(store.get_package("curl").unwrap().state, PackageState::Installed);
    assert_eq!(store.get_package("libssl3").unwrap().state, PackageState::Installed);

    // Now remove curl
    let remove_actions = vec![PackageAction {
        action: PackageActionType::Remove,
        package_name: "curl".into(),
        target_version: None,
    }];
    let remove_plan = store.plan_transaction(remove_actions, false).unwrap();
    assert_eq!(remove_plan.total_size_delta_bytes, -4_194_304);
    let remove_report = store.execute_transaction(&remove_plan).unwrap();
    assert!(remove_report.success);
    assert_eq!(store.get_package("curl").unwrap().state, PackageState::Available);
}

#[test]
fn test_package_store_cs5_persistence_and_bounds() {
    let store = PackageStore::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let store_path = temp_dir.path().join("package_store.json");

    // Save and load roundtrip
    assert!(store.save_to_path(&store_path).is_ok());
    let loaded = PackageStore::load_from_path(&store_path).expect("load succeeds");
    assert_eq!(loaded.packages.len(), 8);
    assert_eq!(
        loaded.get_package("bash").unwrap().description,
        "GNU Bourne Again SHell"
    );

    // Bounded load rejects files exceeding 10 MiB
    let huge_file_path = temp_dir.path().join("oversized_store.json");
    let mut huge_file = File::create(&huge_file_path).unwrap();
    huge_file.set_len(11 * 1024 * 1024).unwrap(); // 11 MiB
    huge_file.flush().unwrap();

    let err = PackageStore::load_from_path(&huge_file_path).unwrap_err();
    assert!(err.contains("exceeds maximum allowed size of 10 MiB"));
}

#[test]
fn test_package_store_hardening_and_error_paths() {
    let store = PackageStore::new();

    // 1. Transaction actions bounds: empty actions
    let empty_actions: Vec<PackageAction> = vec![];
    let err_empty = store.plan_transaction(empty_actions, false).unwrap_err();
    assert!(err_empty.contains("cannot be empty"));

    // 2. Transaction actions bounds: >256 actions
    let huge_actions: Vec<PackageAction> = (0..257)
        .map(|i| PackageAction {
            action: PackageActionType::Install,
            package_name: format!("pkg-{}", i),
            target_version: None,
        })
        .collect();
    let err_huge = store.plan_transaction(huge_actions, false).unwrap_err();
    assert!(err_huge.contains("exceeds 256"));

    // 3. Resource cleanup on failure path: no temp file left behind
    let temp_dir = tempfile::tempdir().unwrap();
    let dummy_file = temp_dir.path().join("dummy_blocked_file");
    std::fs::write(&dummy_file, b"blocked").unwrap();
    let unreachable_target = dummy_file.join("sub").join("packages.json");
    let err_save = store.save_to_path(&unreachable_target);
    assert!(err_save.is_err());
    assert!(!unreachable_target.with_extension("tmp").exists());
}
