//! Unit test suite for AIOS Package Management Security Policy Subsystem.
//! Validates criteria PP1..PP6.

use aiosh_core::package::*;
use aiosh_core::package_policy::*;
use aiosh_core::package_service::*;

fn make_test_spec(name: &str) -> PackageSpec {
    PackageSpec {
        name: name.into(),
        version: "1.0.0".into(),
        architecture: "amd64".into(),
        format: PackageFormat::Deb,
        state: PackageState::Installed,
        description: format!("Test package for {}", name),
        installed_size_bytes: 1_048_576,
        sha256: Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into()),
        repository_url: Some("https://deb.debian.org/debian".into()),
        dependencies: vec![],
    }
}

#[test]
fn test_pp1_policy_configuration_bounds_and_defaults() {
    let mut policy = PackageSecurityPolicy::default();
    assert!(policy.validate().is_ok());

    // Negative: empty architectures
    policy.allowed_architectures.clear();
    let err1 = policy.validate();
    assert!(err1.is_err());
    assert!(err1.unwrap_err().contains("allowed_architectures cannot be empty"));

    policy.allowed_architectures = vec!["amd64".into()];

    // Negative: empty formats
    policy.allowed_formats.clear();
    let err2 = policy.validate();
    assert!(err2.is_err());
    assert!(err2.unwrap_err().contains("allowed_formats cannot be empty"));

    policy.allowed_formats = vec![PackageFormat::Deb];

    // Negative: size boundary (< 10 KiB)
    policy.max_package_size_bytes = 1024;
    let err3 = policy.validate();
    assert!(err3.is_err());
    assert!(err3.unwrap_err().contains("max_package_size_bytes"));
}

#[test]
fn test_pp2_prohibited_package_blocking() {
    let policy = PackageSecurityPolicy::default();

    // Positive: legitimate tools allowed
    let valid_spec = make_test_spec("aiosh-tools");
    let v_valid = policy.evaluate_spec(&valid_spec);
    assert!(v_valid.allowed);
    assert!(v_valid.violations.is_empty());

    // Negative: telnet prohibited
    let telnet_spec = make_test_spec("telnet");
    let v_telnet = policy.evaluate_spec(&telnet_spec);
    assert!(!v_telnet.allowed);
    assert!(v_telnet.violations.iter().any(|v| v.rule_id == "PP2-PROHIBITED-PACKAGE"));

    // Negative: rsh-server prohibited
    let rsh_spec = make_test_spec("rsh-server");
    let v_rsh = policy.evaluate_spec(&rsh_spec);
    assert!(!v_rsh.allowed);
    assert!(v_rsh.violations.iter().any(|v| v.rule_id == "PP2-PROHIBITED-PACKAGE"));
}

#[test]
fn test_pp3_cryptographic_checksum_enforcement() {
    let policy = PackageSecurityPolicy::default();

    // Negative: missing sha256
    let mut no_checksum = make_test_spec("safe-tool");
    no_checksum.sha256 = None;
    let v_none = policy.evaluate_spec(&no_checksum);
    assert!(!v_none.allowed);
    assert!(v_none.violations.iter().any(|v| v.rule_id == "PP3-MISSING-CHECKSUM"));

    // Negative: malformed sha256 (not 64 hex characters)
    let mut bad_checksum = make_test_spec("safe-tool");
    bad_checksum.sha256 = Some("not-a-valid-sha256-hash".into());
    let v_bad = policy.evaluate_spec(&bad_checksum);
    assert!(!v_bad.allowed);
    assert!(v_bad.violations.iter().any(|v| v.rule_id == "PP3-INVALID-CHECKSUM"));
}

#[test]
fn test_pp4_transport_protocol_and_repository_security() {
    let policy = PackageSecurityPolicy::default();

    // Negative: insecure plaintext http://
    let mut insecure_http = make_test_spec("web-tool");
    insecure_http.repository_url = Some("http://deb.debian.org/debian".into());
    let v_http = policy.evaluate_spec(&insecure_http);
    assert!(!v_http.allowed);
    assert!(v_http.violations.iter().any(|v| v.rule_id == "PP4-INSECURE-TRANSPORT"));

    // Positive: https:// passes
    let mut secure_https = make_test_spec("web-tool");
    secure_https.repository_url = Some("https://deb.debian.org/debian".into());
    let v_https = policy.evaluate_spec(&secure_https);
    assert!(v_https.allowed);

    // Positive: file:// passes
    let mut local_mirror = make_test_spec("local-tool");
    local_mirror.repository_url = Some("file:///var/cache/aios/repo".into());
    let v_file = policy.evaluate_spec(&local_mirror);
    assert!(v_file.allowed);
}

#[test]
fn test_pp5_architecture_format_and_sizing_limits() {
    let policy = PackageSecurityPolicy::default();

    // Negative: disallowed architecture (mips)
    let mut mips_pkg = make_test_spec("mips-tool");
    mips_pkg.architecture = "mips64el".into();
    let v_arch = policy.evaluate_spec(&mips_pkg);
    assert!(!v_arch.allowed);
    assert!(v_arch.violations.iter().any(|v| v.rule_id == "PP5-DISALLOWED-ARCH"));

    // Negative: package size exceeds ceiling
    let mut huge_pkg = make_test_spec("huge-dataset");
    huge_pkg.installed_size_bytes = 20 * 1024 * 1024 * 1024; // 20 GiB (exceeds default 10 GiB)
    let v_size = policy.evaluate_spec(&huge_pkg);
    assert!(!v_size.allowed);
    assert!(v_size.violations.iter().any(|v| v.rule_id == "PP5-SIZE-EXCEEDED"));
}

#[test]
fn test_pp6_policy_modes_and_transaction_evaluation() {
    let mut policy = PackageSecurityPolicy::default();

    // Mode: Audit
    policy.mode = PackagePolicyMode::Audit;
    let telnet_pkg = make_test_spec("telnet");
    let audit_verdict = policy.evaluate_spec(&telnet_pkg);
    assert!(audit_verdict.allowed);
    assert_eq!(audit_verdict.mode, PackagePolicyMode::Audit);
    assert!(audit_verdict.violations.iter().any(|v| v.rule_id == "PP2-PROHIBITED-PACKAGE"));

    // Mode: Enforcing with Transaction
    policy.mode = PackagePolicyMode::Enforcing;
    let mut store = PackageStore::empty();
    let safe_pkg = make_test_spec("safe-tool");
    let bad_pkg = make_test_spec("telnet");
    store.register_package(safe_pkg).expect("register safe");
    store.register_package(bad_pkg).expect("register bad");

    let tx_actions = vec![
        PackageAction {
            package_name: "safe-tool".into(),
            action: PackageActionType::Install,
            target_version: None,
        },
        PackageAction {
            package_name: "telnet".into(),
            action: PackageActionType::Install,
            target_version: None,
        },
    ];
    let tx = store.plan_transaction(tx_actions, false).expect("plan tx");
    let tx_verdict = policy.evaluate_transaction(&tx, &store);
    assert!(!tx_verdict.allowed);
    assert!(tx_verdict.violations.iter().any(|v| v.rule_id == "PP2-PROHIBITED-PACKAGE"));

    // Serialization / File Roundtrip
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.json");
    let policy_json = serde_json::to_string_pretty(&policy).unwrap();
    std::fs::write(&policy_path, policy_json).unwrap();

    let loaded = PackageSecurityPolicy::from_file(&policy_path).expect("load from file");
    assert_eq!(loaded.mode, PackagePolicyMode::Enforcing);
    assert_eq!(loaded.prohibited_packages, policy.prohibited_packages);
}

#[test]
fn test_pp7_hardening_and_boundary_checks() {
    // 1. Control characters in policy file path
    let err_path = PackageSecurityPolicy::from_file("bad\0policy.json");
    assert!(err_path.is_err());
    assert!(err_path.unwrap_err().contains("control characters"));

    // 2. Oversized policy file (> 64 KiB)
    let tmp = tempfile::tempdir().unwrap();
    let big_path = tmp.path().join("big_policy.json");
    let big_bytes = vec![b' '; (MAX_POLICY_FILE_BYTES + 1024) as usize];
    std::fs::write(&big_path, big_bytes).unwrap();
    let err_size = PackageSecurityPolicy::from_file(&big_path);
    assert!(err_size.is_err());
    assert!(err_size.unwrap_err().contains("exceeds maximum allowable"));

    // 3. Repository URL validation bounds
    let mut policy = PackageSecurityPolicy::default();
    policy.allowed_repositories.push("http://insecure-plain-http.com".into());
    let err_repo = policy.validate();
    assert!(err_repo.is_err());
    assert!(err_repo.unwrap_err().contains("invariant PP4 violated"));
}
