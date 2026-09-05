//! Unit test suite for AIOS Package Management Observability Subsystem (PO1..PO6).

use aiosh_core::package::*;
use aiosh_core::package_observability::*;
use aiosh_core::package_policy::*;
use aiosh_core::package_service::*;

fn make_pkg(name: &str, fmt: PackageFormat, arch: &str, state: PackageState, size: u64, deps: usize) -> PackageSpec {
    let dependencies = (0..deps)
        .map(|i| PackageDependency {
            name: format!("dep-{}", i),
            version_constraint: None,
            optional: false,
        })
        .collect();

    PackageSpec {
        name: name.into(),
        version: "1.0.0".into(),
        architecture: arch.into(),
        format: fmt,
        state,
        description: format!("Test package {}", name),
        installed_size_bytes: size,
        sha256: Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into()),
        repository_url: Some("https://deb.debian.org/debian".into()),
        dependencies,
    }
}

#[test]
fn test_po1_inventory_completeness_and_empty_store() {
    let empty_store = PackageStore::empty();
    let r_empty = PackageObservabilityReport::generate(&empty_store, None);

    assert_eq!(r_empty.total_packages, 0);
    assert_eq!(r_empty.total_installed_size_bytes, 0);
    assert_eq!(r_empty.average_package_size_bytes, 0);
    assert_eq!(r_empty.policy_compliant_count, 0);
    assert_eq!(r_empty.policy_violations_count, 0);
    assert!(r_empty.prohibited_packages_found.is_empty());
    assert!(r_empty.state_breakdown.is_empty());
    assert!(r_empty.format_breakdown.is_empty());
    assert!(r_empty.architecture_breakdown.is_empty());
    assert_eq!(r_empty.dependency_distribution.get("0"), Some(&0));
    assert_eq!(r_empty.dependency_distribution.get("1-5"), Some(&0));
    assert_eq!(r_empty.dependency_distribution.get("6-10"), Some(&0));
    assert_eq!(r_empty.dependency_distribution.get("11+"), Some(&0));

    // Default store completeness
    let def_store = PackageStore::new();
    let r_def = PackageObservabilityReport::generate(&def_store, None);
    assert!(r_def.total_packages > 0);

    let state_sum: usize = r_def.state_breakdown.values().sum();
    let format_sum: usize = r_def.format_breakdown.values().sum();
    let arch_sum: usize = r_def.architecture_breakdown.values().sum();
    let dep_sum: usize = r_def.dependency_distribution.values().sum();

    assert_eq!(state_sum, r_def.total_packages);
    assert_eq!(format_sum, r_def.total_packages);
    assert_eq!(arch_sum, r_def.total_packages);
    assert_eq!(dep_sum, r_def.total_packages);
}

#[test]
fn test_po2_state_format_arch_breakdown_distributions() {
    let mut store = PackageStore::empty();
    store.register_package(make_pkg("pkg-a", PackageFormat::Deb, "amd64", PackageState::Installed, 1000, 0)).unwrap();
    store.register_package(make_pkg("pkg-b", PackageFormat::Deb, "amd64", PackageState::Available, 2000, 1)).unwrap();
    store.register_package(make_pkg("pkg-c", PackageFormat::Apk, "aarch64", PackageState::Upgradable, 3000, 2)).unwrap();
    store.register_package(make_pkg("pkg-d", PackageFormat::Flatpak, "x86_64", PackageState::PendingRemoval, 4000, 0)).unwrap();

    let report = PackageObservabilityReport::generate(&store, None);
    assert_eq!(report.total_packages, 4);

    // State breakdown
    assert_eq!(report.state_breakdown.get("installed"), Some(&1));
    assert_eq!(report.state_breakdown.get("available"), Some(&1));
    assert_eq!(report.state_breakdown.get("upgradable"), Some(&1));
    assert_eq!(report.state_breakdown.get("pending_removal"), Some(&1));

    // Format breakdown
    assert_eq!(report.format_breakdown.get("deb"), Some(&2));
    assert_eq!(report.format_breakdown.get("apk"), Some(&1));
    assert_eq!(report.format_breakdown.get("flatpak"), Some(&1));

    // Architecture breakdown
    assert_eq!(report.architecture_breakdown.get("amd64"), Some(&2));
    assert_eq!(report.architecture_breakdown.get("aarch64"), Some(&1));
    assert_eq!(report.architecture_breakdown.get("x86_64"), Some(&1));
}

#[test]
fn test_po3_footprint_and_capacity_telemetry() {
    let mut store = PackageStore::empty();
    // Installed (10,000 bytes) + Upgradable (20,000 bytes) + Available (50,000 bytes, not installed)
    store.register_package(make_pkg("installed-app", PackageFormat::Deb, "amd64", PackageState::Installed, 10_000, 0)).unwrap();
    store.register_package(make_pkg("upgraded-app", PackageFormat::Deb, "amd64", PackageState::Upgradable, 20_000, 0)).unwrap();
    store.register_package(make_pkg("available-app", PackageFormat::Deb, "amd64", PackageState::Available, 60_000, 0)).unwrap();

    let report = PackageObservabilityReport::generate(&store, None);
    assert_eq!(report.total_packages, 3);
    // Only installed + upgradable count toward total_installed_size_bytes
    assert_eq!(report.total_installed_size_bytes, 30_000);
    // Average size over all 3 packages: (10,000 + 20,000 + 60,000) / 3 = 30,000
    assert_eq!(report.average_package_size_bytes, 30_000);
}

#[test]
fn test_po4_dependency_distribution_histogram() {
    let mut store = PackageStore::empty();
    store.register_package(make_pkg("dep-zero", PackageFormat::Deb, "amd64", PackageState::Installed, 1000, 0)).unwrap();
    store.register_package(make_pkg("dep-three", PackageFormat::Deb, "amd64", PackageState::Installed, 1000, 3)).unwrap();
    store.register_package(make_pkg("dep-eight", PackageFormat::Deb, "amd64", PackageState::Installed, 1000, 8)).unwrap();
    store.register_package(make_pkg("dep-twelve", PackageFormat::Deb, "amd64", PackageState::Installed, 1000, 12)).unwrap();

    let report = PackageObservabilityReport::generate(&store, None);
    assert_eq!(report.total_packages, 4);

    assert_eq!(report.dependency_distribution.get("0"), Some(&1));
    assert_eq!(report.dependency_distribution.get("1-5"), Some(&1));
    assert_eq!(report.dependency_distribution.get("6-10"), Some(&1));
    assert_eq!(report.dependency_distribution.get("11+"), Some(&1));
}

#[test]
fn test_po5_policy_compliance_and_prohibited_package_detection() {
    let mut store = PackageStore::empty();
    // 1. Compliant package
    store.register_package(make_pkg("good-tool", PackageFormat::Deb, "amd64", PackageState::Installed, 1000, 0)).unwrap();

    // 2. Prohibited package (telnet)
    let mut telnet = make_pkg("telnet", PackageFormat::Deb, "amd64", PackageState::Installed, 1000, 0);
    telnet.name = "telnet".into();
    store.register_package(telnet).unwrap();

    // 3. Package lacking mandatory checksum (valid in store, but rejected by policy PP3)
    let mut no_checksum_pkg = make_pkg("no-hash-pkg", PackageFormat::Deb, "amd64", PackageState::Installed, 1000, 0);
    no_checksum_pkg.sha256 = None;
    store.register_package(no_checksum_pkg).unwrap();

    let policy = PackageSecurityPolicy::default();
    let report = PackageObservabilityReport::generate(&store, Some(&policy));

    assert_eq!(report.total_packages, 3);
    assert_eq!(report.policy_compliant_count, 1);
    assert!(report.policy_violations_count >= 2);
    assert_eq!(report.prohibited_packages_found, vec!["telnet"]);
}

#[test]
fn test_po6_serialization_and_negative_boundary_matrix() {
    let store = PackageStore::new();
    let report = PackageObservabilityReport::generate(&store, None);

    // Serialization roundtrip
    let json_str = report.to_json_pretty().expect("serialize pretty");
    let deserialized: PackageObservabilityReport = serde_json::from_str(&json_str).expect("deserialize json");
    assert_eq!(report, deserialized);

    // Formatting functions
    assert_eq!(format_to_str(PackageFormat::Deb), "deb");
    assert_eq!(format_to_str(PackageFormat::Apk), "apk");
    assert_eq!(format_to_str(PackageFormat::Flatpak), "flatpak");
    assert_eq!(format_to_str(PackageFormat::Tarball), "tarball");

    assert_eq!(state_to_str(PackageState::Available), "available");
    assert_eq!(state_to_str(PackageState::Installed), "installed");
    assert_eq!(state_to_str(PackageState::Upgradable), "upgradable");
    assert_eq!(state_to_str(PackageState::PendingInstall), "pending_install");
    assert_eq!(state_to_str(PackageState::PendingRemoval), "pending_removal");
    assert_eq!(state_to_str(PackageState::Broken), "broken");
}

#[test]
fn test_po7_hardening_and_path_boundaries() {
    // 1. Valid generate_from_paths with None
    let res_none = PackageObservabilityReport::generate_from_paths::<&str, &str>(None, None);
    assert!(res_none.is_ok());
    assert!(res_none.unwrap().total_packages > 0);

    // 2. Control character in store path
    let res_bad_store = PackageObservabilityReport::generate_from_paths::<&str, &str>(Some("bad\0store.json"), None);
    assert!(res_bad_store.is_err());
    assert!(res_bad_store.unwrap_err().contains("control characters"));

    // 3. Control character in policy path
    let res_bad_policy = PackageObservabilityReport::generate_from_paths::<&str, &str>(None, Some("bad\0policy.json"));
    assert!(res_bad_policy.is_err());
    assert!(res_bad_policy.unwrap_err().contains("control characters"));

    // 4. Non-existent store path
    let res_nonexistent = PackageObservabilityReport::generate_from_paths::<&str, &str>(Some("nonexistent_store_file_9999.json"), None);
    assert!(res_nonexistent.is_err());

    // 5. Non-existent policy path
    let res_bad_policy_path = PackageObservabilityReport::generate_from_paths::<&str, &str>(None, Some("nonexistent_policy_file_9999.json"));
    assert!(res_bad_policy_path.is_err());

    // 6. Oversized store path (>1024 characters)
    let long_store_path = format!("{}.json", "a".repeat(1025));
    let res_long_store = PackageObservabilityReport::generate_from_paths::<&str, &str>(Some(&long_store_path), None);
    assert!(res_long_store.is_err());
    assert!(res_long_store.unwrap_err().contains("exceeds 1024 characters"));

    // 7. Oversized policy path (>1024 characters)
    let long_policy_path = format!("{}.json", "b".repeat(1025));
    let res_long_policy = PackageObservabilityReport::generate_from_paths::<&str, &str>(None, Some(&long_policy_path));
    assert!(res_long_policy.is_err());
    assert!(res_long_policy.unwrap_err().contains("exceeds 1024 characters"));

    // 8. Real file roundtrip with temporary store and policy
    let temp_dir = std::env::temp_dir();
    let store_path = temp_dir.join(format!("aios_obs_test_store_{}.json", std::process::id()));
    let policy_path = temp_dir.join(format!("aios_obs_test_policy_{}.json", std::process::id()));

    let store = PackageStore::new();
    store.save_to_path(&store_path).expect("save test store");

    let policy = PackageSecurityPolicy::default();
    let policy_json = serde_json::to_string_pretty(&policy).expect("serialize policy");
    std::fs::write(&policy_path, policy_json).expect("write test policy");

    let res_files = PackageObservabilityReport::generate_from_paths(Some(&store_path), Some(&policy_path));
    assert!(res_files.is_ok());
    let report = res_files.unwrap();
    assert!(report.total_packages > 0);
    assert_eq!(report.policy_compliant_count, report.total_packages);

    let _ = std::fs::remove_file(&store_path);
    let _ = std::fs::remove_file(&policy_path);
}
