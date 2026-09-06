//! Unit test suite for AIOS Init & Service Supervision Security Policy Subsystem.
//! Validates criteria SP1..SP6.

use std::collections::BTreeMap;
use aiosh_core::service::*;
use aiosh_core::service_policy::*;
use aiosh_core::service_service::*;

fn make_test_spec(name: &str) -> ServiceSpec {
    let mut env = BTreeMap::new();
    env.insert("AIOS_ENV".into(), "production".into());
    env.insert("LOG_LEVEL".into(), "info".into());

    ServiceSpec {
        name: name.into(),
        description: format!("Supervised service for {}", name),
        exec_start: "/usr/bin/aios-agent --daemon".into(),
        exec_stop: Some("/usr/bin/aios-agent --stop".into()),
        exec_reload: Some("/usr/bin/aios-agent --reload".into()),
        service_type: ServiceType::Simple,
        restart_policy: ServiceRestartPolicy::Always,
        startup_mode: ServiceStartupMode::Enabled,
        user: Some("aios".into()),
        group: Some("aios".into()),
        working_dir: Some("/var/lib/aios".into()),
        environment: env,
        dependencies: vec![],
        timeout_start_secs: 30,
        timeout_stop_secs: 30,
    }
}

#[test]
fn test_sp1_policy_configuration_bounds_and_defaults() {
    let mut policy = ServiceSecurityPolicy::default();
    assert!(policy.validate().is_ok());
    assert_eq!(policy.mode, ServicePolicyMode::Enforcing);

    // Negative: empty allowed service types
    policy.allowed_service_types.clear();
    let err1 = policy.validate();
    assert!(err1.is_err());
    assert!(err1.unwrap_err().contains("allowed_service_types cannot be empty"));

    policy.allowed_service_types = vec![ServiceType::Simple];

    // Negative: max_env_vars boundary (0 or > 1024)
    policy.max_env_vars = 0;
    let err2 = policy.validate();
    assert!(err2.is_err());
    assert!(err2.unwrap_err().contains("max_env_vars out of range"));

    policy.max_env_vars = 256;

    // Negative: max_timeout_secs boundary (> 86400)
    policy.max_timeout_secs = 100_000;
    let err3 = policy.validate();
    assert!(err3.is_err());
    assert!(err3.unwrap_err().contains("max_timeout_secs out of range"));

    policy.max_timeout_secs = 3600;

    // Negative: invalid prohibited exec path (relative path)
    policy.prohibited_exec_paths.push("relative/path".into());
    let err4 = policy.validate();
    assert!(err4.is_err());
    assert!(err4.unwrap_err().contains("must be absolute paths"));

    policy.prohibited_exec_paths.pop();

    // Negative: invalid disallowed env var (contains '=')
    policy.disallow_env_vars.push("INVALID=VAR".into());
    let err5 = policy.validate();
    assert!(err5.is_err());
    assert!(err5.unwrap_err().contains("invalid disallowed environment variable"));
}

#[test]
fn test_sp2_prohibited_service_blocking() {
    let policy = ServiceSecurityPolicy::default();

    // Positive: legitimate daemon allowed
    let valid_spec = make_test_spec("aios-agent.service");
    let v_valid = policy.evaluate_spec(&valid_spec);
    assert!(v_valid.allowed);
    assert!(v_valid.violations.is_empty());

    // Negative: telnet.service prohibited
    let telnet_spec = make_test_spec("telnet.service");
    let v_telnet = policy.evaluate_spec(&telnet_spec);
    assert!(!v_telnet.allowed);
    assert!(v_telnet.violations.iter().any(|v| v.rule_id == "SP2-PROHIBITED-SERVICE"));

    // Negative: bare name telnet matches prohibited
    let telnet_bare = make_test_spec("telnet");
    let v_bare = policy.evaluate_spec(&telnet_bare);
    assert!(!v_bare.allowed);
    assert!(v_bare.violations.iter().any(|v| v.rule_id == "SP2-PROHIBITED-SERVICE"));

    // Negative: rsh.service and tftp.service prohibited
    let rsh_spec = make_test_spec("rsh.service");
    assert!(!policy.evaluate_spec(&rsh_spec).allowed);

    let tftp_spec = make_test_spec("tftp.service");
    assert!(!policy.evaluate_spec(&tftp_spec).allowed);
}

#[test]
fn test_sp3_executable_path_and_working_dir_hygiene() {
    let policy = ServiceSecurityPolicy::default();

    // Negative: relative binary in exec_start
    let mut relative_spec = make_test_spec("relative-app.service");
    relative_spec.exec_start = "bin/my-daemon --start".into();
    let v_rel = policy.evaluate_spec(&relative_spec);
    assert!(!v_rel.allowed);
    assert!(v_rel.violations.iter().any(|v| v.rule_id == "SP3-RELATIVE-PATH"));

    // Negative: binary resides in prohibited directory /tmp/
    let mut tmp_spec = make_test_spec("tmp-app.service");
    tmp_spec.exec_start = "/tmp/malicious-daemon".into();
    let v_tmp = policy.evaluate_spec(&tmp_spec);
    assert!(!v_tmp.allowed);
    assert!(v_tmp.violations.iter().any(|v| v.rule_id == "SP3-PROHIBITED-PATH"));

    // Negative: directory traversal in exec command
    let mut trav_spec = make_test_spec("trav-app.service");
    trav_spec.exec_start = "/usr/bin/../../bin/sh".into();
    let v_trav = policy.evaluate_spec(&trav_spec);
    assert!(!v_trav.allowed);
    assert!(v_trav.violations.iter().any(|v| v.rule_id == "SP3-PATH-TRAVERSAL"));

    // Negative: directory traversal in working_dir
    let mut trav_wd = make_test_spec("trav-wd.service");
    trav_wd.working_dir = Some("/var/lib/aios/../../etc".into());
    let v_trav_wd = policy.evaluate_spec(&trav_wd);
    assert!(!v_trav_wd.allowed);
    assert!(v_trav_wd.violations.iter().any(|v| v.rule_id == "SP3-PATH-TRAVERSAL"));

    // Negative: working_dir resides in /dev/shm
    let mut shm_wd = make_test_spec("shm-wd.service");
    shm_wd.working_dir = Some("/dev/shm/app".into());
    let v_shm_wd = policy.evaluate_spec(&shm_wd);
    assert!(!v_shm_wd.allowed);
    assert!(v_shm_wd.violations.iter().any(|v| v.rule_id == "SP3-PROHIBITED-PATH"));
}

#[test]
fn test_sp4_user_privilege_and_root_hygiene() {
    let mut policy = ServiceSecurityPolicy::default();

    // 1. require_service_user enforcement
    policy.require_service_user = true;

    let mut no_user_spec = make_test_spec("orphan-daemon.service");
    no_user_spec.user = None;
    let v_no_user = policy.evaluate_spec(&no_user_spec);
    assert!(!v_no_user.allowed);
    assert!(v_no_user.violations.iter().any(|v| v.rule_id == "SP4-UNPRIVILEGED-USER-REQUIRED"));

    // Exemption for allowed_root_services
    let mut root_exempt = make_test_spec("systemd-journald.service");
    root_exempt.user = None;
    let v_exempt = policy.evaluate_spec(&root_exempt);
    assert!(v_exempt.allowed);

    // 2. disallow_root enforcement
    policy.disallow_root = true;

    let mut root_user_spec = make_test_spec("root-daemon.service");
    root_user_spec.user = Some("root".into());
    let v_root = policy.evaluate_spec(&root_user_spec);
    assert!(!v_root.allowed);
    assert!(v_root.violations.iter().any(|v| v.rule_id == "SP4-ROOT-DISALLOWED"));

    // Allowed root service exemption
    let mut securityd_spec = make_test_spec("aios-securityd.service");
    securityd_spec.user = Some("root".into());
    let v_sec = policy.evaluate_spec(&securityd_spec);
    assert!(v_sec.allowed);
}

#[test]
fn test_sp5_environment_and_parameter_sanitization() {
    let policy = ServiceSecurityPolicy::default();

    // Negative: dangerous LD_PRELOAD injection
    let mut preload_spec = make_test_spec("injected.service");
    preload_spec.environment.insert("LD_PRELOAD".into(), "/lib/libhook.so".into());
    let v_preload = policy.evaluate_spec(&preload_spec);
    assert!(!v_preload.allowed);
    assert!(v_preload.violations.iter().any(|v| v.rule_id == "SP5-DANGEROUS-ENV-VAR"));

    // Negative: timeout exceeds policy limit
    let mut timeout_spec = make_test_spec("slow.service");
    timeout_spec.timeout_start_secs = 7200; // default max is 3600
    let v_timeout = policy.evaluate_spec(&timeout_spec);
    assert!(v_timeout.violations.iter().any(|v| v.rule_id == "SP5-TIMEOUT-EXCEEDED"));

    // Negative: disallowed service type
    let mut restricted_policy = policy.clone();
    restricted_policy.allowed_service_types = vec![ServiceType::Simple];
    let mut forking_spec = make_test_spec("forker.service");
    forking_spec.service_type = ServiceType::Forking;
    let v_fork = restricted_policy.evaluate_spec(&forking_spec);
    assert!(!v_fork.allowed);
    assert!(v_fork.violations.iter().any(|v| v.rule_id == "SP5-DISALLOWED-TYPE"));
}

#[test]
fn test_sp6_policy_modes_store_evaluation_and_file_roundtrip() {
    // Mode: Audit
    let mut audit_policy = ServiceSecurityPolicy::default();
    audit_policy.mode = ServicePolicyMode::Audit;
    let bad_spec = make_test_spec("telnet.service");
    let audit_verdict = audit_policy.evaluate_spec(&bad_spec);
    assert!(audit_verdict.allowed);
    assert_eq!(audit_verdict.mode, ServicePolicyMode::Audit);
    assert!(audit_verdict.violations.iter().any(|v| v.rule_id == "SP2-PROHIBITED-SERVICE"));

    // Mode: Permissive (suppresses non-fatal violations, but blocks prohibited service)
    let mut perm_policy = ServiceSecurityPolicy::default();
    perm_policy.mode = ServicePolicyMode::Permissive;
    let perm_telnet = perm_policy.evaluate_spec(&bad_spec);
    assert!(!perm_telnet.allowed); // Prohibited service is still blocked in Permissive

    // Store evaluation
    let mut store = ServiceStore::empty();
    store.register_service(make_test_spec("safe-service.service")).unwrap();
    store.register_service(make_test_spec("telnet.service")).unwrap();

    let verdicts = ServiceSecurityPolicy::default().evaluate_store(&store);
    assert_eq!(verdicts.len(), 2);
    assert!(verdicts.iter().any(|v| v.service_name == "safe-service.service" && v.allowed));
    assert!(verdicts.iter().any(|v| v.service_name == "telnet.service" && !v.allowed));

    // Serialization & File Roundtrip
    let tmp = tempfile::tempdir().unwrap();
    let policy_file = tmp.path().join("service_policy.json");
    let policy_json = serde_json::to_string_pretty(&audit_policy).unwrap();
    std::fs::write(&policy_file, policy_json).unwrap();

    let loaded = ServiceSecurityPolicy::from_file(&policy_file).expect("load from file");
    assert_eq!(loaded.mode, ServicePolicyMode::Audit);
    assert_eq!(loaded.prohibited_services, audit_policy.prohibited_services);

    // Oversized policy file (> 64 KiB)
    let big_file = tmp.path().join("huge_policy.json");
    let big_data = vec![b' '; (MAX_POLICY_FILE_BYTES + 1024) as usize];
    std::fs::write(&big_file, big_data).unwrap();
    let err_size = ServiceSecurityPolicy::from_file(&big_file);
    assert!(err_size.is_err());
    assert!(err_size.unwrap_err().contains("exceeds maximum allowable"));
}

#[test]
fn test_sp7_hardening_and_boundary_checks() {
    // 1. Control character in policy file path
    let err_path = ServiceSecurityPolicy::from_file("bad\0service_policy.json");
    assert!(err_path.is_err());
    assert!(err_path.unwrap_err().contains("control characters"));

    // 2. Oversized policy file path (> 1024 chars)
    let long_path = "a/".repeat(600) + "policy.json";
    let err_long = ServiceSecurityPolicy::from_file(&long_path);
    assert!(err_long.is_err());
    assert!(err_long.unwrap_err().contains("exceeds 1024 characters"));

    // 3. Fail-closed semantics under Enforcing mode
    let policy = ServiceSecurityPolicy::default();
    let mut bad_spec = make_test_spec("bad-daemon.service");
    bad_spec.exec_start = "/var/tmp/bad-daemon".into();
    let verdict = policy.evaluate_spec(&bad_spec);
    assert!(!verdict.allowed);
    assert_eq!(verdict.mode, ServicePolicyMode::Enforcing);

    // 4. Multiple command arguments handling
    let mut args_spec = make_test_spec("multi-arg.service");
    args_spec.exec_start = "/usr/bin/aios-agent  --flag1   --flag2=value  --daemon".into();
    let v_args = policy.evaluate_spec(&args_spec);
    assert!(v_args.allowed);
    assert!(v_args.violations.is_empty());
}

