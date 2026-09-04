//! Automated Unit & Integration Tests for Package Management Data Model (PM1..PM5)

use aiosh_core::package::{
    validate_package_name, validate_package_spec, validate_package_transaction, PackageAction,
    PackageActionType, PackageDependency, PackageFormat, PackageQuery, PackageSpec, PackageState,
    PackageTransaction,
};

#[test]
fn test_pm1_package_name_boundary_and_syntax() {
    // Valid standard names
    assert!(validate_package_name("bash").is_ok());
    assert!(validate_package_name("coreutils").is_ok());
    assert!(validate_package_name("g++").is_ok());
    assert!(validate_package_name("libssl3").is_ok());
    assert!(validate_package_name("python3.11").is_ok());
    assert!(validate_package_name("zlib1g").is_ok());
    assert!(validate_package_name("apk-tools").is_ok());
    assert!(validate_package_name("alpine-baselayout").is_ok());

    // Boundary: min length (1 char)
    assert!(validate_package_name("a").is_ok());
    assert!(validate_package_name("1").is_ok());

    // Boundary: max length (128 chars)
    let max_len_name = "a".repeat(128);
    assert!(validate_package_name(&max_len_name).is_ok());

    // Negative: oversized (129 chars)
    let over_len_name = "a".repeat(129);
    assert!(validate_package_name(&over_len_name).is_err());

    // Negative: empty
    assert!(validate_package_name("").is_err());

    // Negative: uppercase
    assert!(validate_package_name("Bash").is_err());
    assert!(validate_package_name("cUrl").is_err());

    // Negative: leading symbols
    assert!(validate_package_name("-curl").is_err());
    assert!(validate_package_name("+curl").is_err());
    assert!(validate_package_name(".curl").is_err());

    // Negative: invalid characters
    assert!(validate_package_name("curl@latest").is_err());
    assert!(validate_package_name("curl!").is_err());
    assert!(validate_package_name("curl/pkg").is_err());
    assert!(validate_package_name("curl\\pkg").is_err());
    assert!(validate_package_name("curl pkg").is_err());
    assert!(validate_package_name("curl\0pkg").is_err());
    assert!(validate_package_name("curl\npkg").is_err());
}

#[test]
fn test_pm2_bounds_and_lengths() {
    let base_spec = PackageSpec {
        name: "test-pkg".into(),
        version: "1.0.0".into(),
        architecture: "x86_64".into(),
        format: PackageFormat::Deb,
        state: PackageState::Available,
        description: "A valid test package".into(),
        installed_size_bytes: 1024,
        sha256: None,
        repository_url: None,
        dependencies: vec![],
    };
    assert!(validate_package_spec(&base_spec).is_ok());

    // Version length bounds
    let mut bad_version = base_spec.clone();
    bad_version.version = "".into();
    assert!(validate_package_spec(&bad_version).is_err());

    bad_version.version = "1".repeat(64);
    assert!(validate_package_spec(&bad_version).is_ok());

    bad_version.version = "1".repeat(65);
    assert!(validate_package_spec(&bad_version).is_err());

    bad_version.version = "1.0.0\0".into();
    assert!(validate_package_spec(&bad_version).is_err());

    // Architecture bounds
    let mut bad_arch = base_spec.clone();
    bad_arch.architecture = "".into();
    assert!(validate_package_spec(&bad_arch).is_err());

    bad_arch.architecture = "x86_64\n".into();
    assert!(validate_package_spec(&bad_arch).is_err());

    // Description bounds (4096 max)
    let mut bad_desc = base_spec.clone();
    bad_desc.description = "x".repeat(4096);
    assert!(validate_package_spec(&bad_desc).is_ok());

    bad_desc.description = "x".repeat(4097);
    assert!(validate_package_spec(&bad_desc).is_err());

    // Size bounds (100 GiB)
    const MAX_GIB: u64 = 100 * 1024 * 1024 * 1024;
    let mut bad_size = base_spec.clone();
    bad_size.installed_size_bytes = MAX_GIB;
    assert!(validate_package_spec(&bad_size).is_ok());

    bad_size.installed_size_bytes = MAX_GIB + 1;
    assert!(validate_package_spec(&bad_size).is_err());

    // Dependencies count bounds (256 max)
    let mut bad_deps = base_spec.clone();
    bad_deps.dependencies = (0..256)
        .map(|i| PackageDependency {
            name: format!("dep-{}", i),
            version_constraint: None,
            optional: false,
        })
        .collect();
    assert!(validate_package_spec(&bad_deps).is_ok());

    bad_deps.dependencies.push(PackageDependency {
        name: "dep-256".into(),
        version_constraint: None,
        optional: false,
    });
    assert!(validate_package_spec(&bad_deps).is_err());
}

#[test]
fn test_pm3_dependency_hygiene() {
    let mut spec = PackageSpec {
        name: "web-server".into(),
        version: "2.4.50".into(),
        architecture: "x86_64".into(),
        format: PackageFormat::Deb,
        state: PackageState::Available,
        description: "Web server daemon".into(),
        installed_size_bytes: 2048,
        sha256: None,
        repository_url: None,
        dependencies: vec![
            PackageDependency {
                name: "libc6".into(),
                version_constraint: Some(">= 2.34".into()),
                optional: false,
            },
            PackageDependency {
                name: "libssl3".into(),
                version_constraint: Some(">= 3.0.0".into()),
                optional: false,
            },
        ],
    };
    assert!(validate_package_spec(&spec).is_ok());

    // Self dependency
    spec.dependencies.push(PackageDependency {
        name: "web-server".into(),
        version_constraint: None,
        optional: false,
    });
    let errs = validate_package_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("cannot depend on itself")));

    // Duplicate dependency
    spec.dependencies.pop();
    spec.dependencies.push(PackageDependency {
        name: "libc6".into(),
        version_constraint: None,
        optional: true,
    });
    let errs = validate_package_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("duplicate dependency detected")));

    // Invalid dependency name
    spec.dependencies.pop();
    spec.dependencies.push(PackageDependency {
        name: "Bad_Dep_Name".into(),
        version_constraint: None,
        optional: false,
    });
    let errs = validate_package_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("invalid dependency name")));
}

#[test]
fn test_pm4_checksum_and_provenance() {
    let mut spec = PackageSpec {
        name: "curl".into(),
        version: "8.5.0".into(),
        architecture: "x86_64".into(),
        format: PackageFormat::Deb,
        state: PackageState::Available,
        description: "Curl tool".into(),
        installed_size_bytes: 400_000,
        sha256: Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into()),
        repository_url: Some("https://deb.debian.org/debian".into()),
        dependencies: vec![],
    };
    assert!(validate_package_spec(&spec).is_ok());

    // Localhost / loopback HTTP accepted
    spec.repository_url = Some("http://127.0.0.1:8080/repo".into());
    assert!(validate_package_spec(&spec).is_ok());
    spec.repository_url = Some("http://localhost:8080/repo".into());
    assert!(validate_package_spec(&spec).is_ok());

    // Insecure public HTTP rejected
    spec.repository_url = Some("http://unencrypted.debian.org/debian".into());
    let errs = validate_package_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("must use secure HTTPS protocol")));

    // Invalid sha256: non-hex
    spec.repository_url = Some("https://deb.debian.org/debian".into());
    spec.sha256 = Some("z3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into());
    let errs = validate_package_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("invalid sha256 checksum format")));

    // Invalid sha256: length 63
    spec.sha256 = Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85".into());
    let errs = validate_package_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("invalid sha256 checksum format")));
}

#[test]
fn test_pm5_state_consistency() {
    let mut spec = PackageSpec {
        name: "openssh-server".into(),
        version: "9.6p1".into(),
        architecture: "x86_64".into(),
        format: PackageFormat::Deb,
        state: PackageState::Installed,
        description: "SSH server".into(),
        installed_size_bytes: 1_200_000,
        sha256: None,
        repository_url: None,
        dependencies: vec![],
    };
    assert!(validate_package_spec(&spec).is_ok());

    // Installed package with 0 bytes size is illegal
    spec.installed_size_bytes = 0;
    let errs = validate_package_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("installed package must have positive installed_size_bytes")));

    // Available package with 0 bytes size is allowed (e.g. virtual or uninstalled)
    spec.state = PackageState::Available;
    assert!(validate_package_spec(&spec).is_ok());
}

#[test]
fn test_package_transaction_invariants() {
    let valid_tx = PackageTransaction {
        id: "tx-pkg-20260904-01".into(),
        created_at: "2026-09-04T07:00:00Z".into(),
        actions: vec![
            PackageAction {
                action: PackageActionType::Install,
                package_name: "curl".into(),
                target_version: Some("8.5.0".into()),
            },
            PackageAction {
                action: PackageActionType::Remove,
                package_name: "telnet".into(),
                target_version: None,
            },
        ],
        dry_run: false,
        total_size_delta_bytes: 200_000,
    };
    assert!(validate_package_transaction(&valid_tx).is_ok());

    // Conflicting operations on same package
    let mut bad_tx = valid_tx.clone();
    bad_tx.actions.push(PackageAction {
        action: PackageActionType::Upgrade,
        package_name: "curl".into(),
        target_version: Some("8.6.0".into()),
    });
    let errs = validate_package_transaction(&bad_tx).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("multiple conflicting actions")));

    // Empty actions
    bad_tx.actions.clear();
    let errs = validate_package_transaction(&bad_tx).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("actions list cannot be empty")));

    // Invalid timestamp
    let mut bad_time = valid_tx.clone();
    bad_time.created_at = "not-a-timestamp".into();
    let errs = validate_package_transaction(&bad_time).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("valid RFC 3339 timestamp")));

    // Non-graphic transaction ID
    let mut bad_id = valid_tx.clone();
    bad_id.id = "bad id with space".into();
    let errs = validate_package_transaction(&bad_id).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("printable graphic ASCII characters")));
}

#[test]
fn test_serde_json_roundtrip() {
    let spec = PackageSpec {
        name: "neovim".into(),
        version: "0.9.5".into(),
        architecture: "x86_64".into(),
        format: PackageFormat::Apk,
        state: PackageState::Upgradable,
        description: "Vim-fork focused on extensibility".into(),
        installed_size_bytes: 18_000_000,
        sha256: Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into()),
        repository_url: Some("https://dl-cdn.alpinelinux.org/alpine/v3.19/main".into()),
        dependencies: vec![PackageDependency {
            name: "lua5.1".into(),
            version_constraint: Some(">= 5.1.5".into()),
            optional: false,
        }],
    };

    let json_str = serde_json::to_string(&spec).expect("Serialization must succeed");
    assert!(json_str.contains("\"format\":\"apk\""));
    assert!(json_str.contains("\"state\":\"upgradable\""));

    let deserialized: PackageSpec =
        serde_json::from_str(&json_str).expect("Deserialization must succeed");
    assert_eq!(spec, deserialized);

    let query = PackageQuery {
        name_pattern: Some("neo*".into()),
        format: Some(PackageFormat::Apk),
        state: Some(PackageState::Upgradable),
        limit: Some(10),
    };
    let q_json = serde_json::to_string(&query).expect("Query serialization must succeed");
    let q_deser: PackageQuery = serde_json::from_str(&q_json).expect("Query deserialization must succeed");
    assert_eq!(query, q_deser);
}
