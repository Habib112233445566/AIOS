//! Automated tests for Sandbox Enforcement Data Model (T-02405).

use aiosh_core::sandbox_data_model::{
    FilesystemPolicy, IsolationLevel, NetworkIsolationMode,
    ResourceLimits, SandboxExecutionRequest, SandboxExecutionResult, SandboxExecutionStatus,
    SandboxProfile, SandboxProfileBuilder, SandboxProfileType,
    ERR_SANDBOX_BOUNDS_EXCEEDED, ERR_SANDBOX_EMPTY_COMMAND, ERR_SANDBOX_INVALID_LIMIT, ERR_SANDBOX_INVALID_PATH,
    ERR_SANDBOX_POLICY_CONFLICT, MAX_MEMORY_BYTES, MAX_OPEN_FILES_LIMIT, MAX_PROCESSES_LIMIT,
    MAX_PROFILE_NAME_LEN, MAX_WALL_TIME_MS, MIN_MEMORY_BYTES, MIN_WALL_TIME_MS,
};

#[test]
fn test_valid_profiles_default_construction() {
    let standard = SandboxProfile::standard();
    assert!(standard.validate().is_ok());
    assert_eq!(standard.profile_type, SandboxProfileType::Standard);
    assert_eq!(standard.isolation_level, IsolationLevel::FullLandlockSeccomp);
    assert_eq!(standard.network, NetworkIsolationMode::LoopbackOnly);

    let strict = SandboxProfile::strict();
    assert!(strict.validate().is_ok());
    assert_eq!(strict.profile_type, SandboxProfileType::Strict);
    assert_eq!(strict.network, NetworkIsolationMode::Disabled);
    assert!(strict.environment.clean_env);

    let permissive = SandboxProfile::permissive();
    assert!(permissive.validate().is_ok());
    assert_eq!(permissive.profile_type, SandboxProfileType::Permissive);
    assert_eq!(permissive.network, NetworkIsolationMode::Unrestricted);
}

#[test]
fn test_resource_limits_boundary_conditions() {
    let mut limits = ResourceLimits::default();

    // Exact min memory
    limits.max_memory_bytes = MIN_MEMORY_BYTES;
    assert!(limits.validate().is_ok());
    // Below min memory
    limits.max_memory_bytes = MIN_MEMORY_BYTES - 1;
    assert!(limits.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_LIMIT));

    // Exact max memory
    limits.max_memory_bytes = MAX_MEMORY_BYTES;
    assert!(limits.validate().is_ok());
    // Above max memory
    limits.max_memory_bytes = MAX_MEMORY_BYTES + 1;
    assert!(limits.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_LIMIT));
    limits.max_memory_bytes = 512 * 1024 * 1024;

    // Exact min wall time
    limits.max_wall_time_ms = MIN_WALL_TIME_MS;
    assert!(limits.validate().is_ok());
    // Below min wall time
    limits.max_wall_time_ms = MIN_WALL_TIME_MS - 1;
    assert!(limits.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_LIMIT));

    // Exact max wall time
    limits.max_wall_time_ms = MAX_WALL_TIME_MS;
    assert!(limits.validate().is_ok());
    // Above max wall time
    limits.max_wall_time_ms = MAX_WALL_TIME_MS + 1;
    assert!(limits.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_LIMIT));
    limits.max_wall_time_ms = 30_000;

    // Process limits (1..4096)
    limits.max_processes = 1;
    assert!(limits.validate().is_ok());
    limits.max_processes = MAX_PROCESSES_LIMIT;
    assert!(limits.validate().is_ok());
    limits.max_processes = 0;
    assert!(limits.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_LIMIT));
    limits.max_processes = MAX_PROCESSES_LIMIT + 1;
    assert!(limits.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_LIMIT));
    limits.max_processes = 32;

    // Open file limits (16..65536)
    limits.max_open_files = 16;
    assert!(limits.validate().is_ok());
    limits.max_open_files = MAX_OPEN_FILES_LIMIT;
    assert!(limits.validate().is_ok());
    limits.max_open_files = 15;
    assert!(limits.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_LIMIT));
    limits.max_open_files = MAX_OPEN_FILES_LIMIT + 1;
    assert!(limits.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_LIMIT));
}

#[test]
fn test_filesystem_policy_traversal_and_empty() {
    let mut fs = FilesystemPolicy::default();
    fs.paths_ro.push("/etc".into());
    assert!(fs.validate().is_ok());

    // Empty path rejected
    fs.paths_ro.push("".into());
    assert!(fs.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_PATH));
    fs.paths_ro.pop();

    // Traversal .. rejected
    fs.paths_ro.push("/etc/../shadow".into());
    assert!(fs.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_PATH));
    fs.paths_ro.pop();

    fs.paths_rw.push("/var/run/../../root".into());
    assert!(fs.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_PATH));
    fs.paths_rw.pop();

    fs.paths_denied.push("..".into());
    assert!(fs.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_PATH));
}

#[test]
fn test_filesystem_policy_conflicts() {
    // Conflict RO and RW
    let mut fs = FilesystemPolicy::default();
    fs.paths_ro.push("/data/shared".into());
    fs.paths_rw.push("/data/shared".into());
    assert!(fs.validate().unwrap_err().contains(ERR_SANDBOX_POLICY_CONFLICT));

    // Conflict RO and Denied
    let mut fs2 = FilesystemPolicy::default();
    fs2.paths_ro.push("/data/shared".into());
    fs2.paths_denied.push("/data/shared".into());
    assert!(fs2.validate().unwrap_err().contains(ERR_SANDBOX_POLICY_CONFLICT));

    // Conflict RW and Denied
    let mut fs3 = FilesystemPolicy::default();
    fs3.paths_rw.push("/data/shared".into());
    fs3.paths_denied.push("/data/shared".into());
    assert!(fs3.validate().unwrap_err().contains(ERR_SANDBOX_POLICY_CONFLICT));
}

#[test]
fn test_custom_profile_builder_workflow() {
    let profile = SandboxProfileBuilder::new("ci-builder")
        .profile_type(SandboxProfileType::Custom)
        .isolation_level(IsolationLevel::RestrictedNamespaces)
        .max_memory_bytes(2_000_000_000)
        .max_wall_time_ms(60_000)
        .add_ro_path("/usr/lib")
        .add_rw_path("/tmp/build")
        .add_execute_path("/usr/bin/cargo")
        .add_denied_path("/etc/shadow")
        .network_mode(NetworkIsolationMode::FilteredEgress { allowed_ports: vec![80, 443] })
        .add_denied_syscall("ptrace")
        .set_env_var("RUST_LOG", "info")
        .build();

    assert!(profile.is_ok());
    let p = profile.unwrap();
    assert_eq!(p.name, "ci-builder");
    assert_eq!(p.filesystem.paths_execute, vec!["/usr/bin/cargo"]);
    assert_eq!(p.filesystem.paths_denied, vec!["/etc/shadow"]);
    assert_eq!(p.environment.injected_vars.get("RUST_LOG").unwrap(), "info");
}

#[test]
fn test_execution_request_and_result() {
    // Empty command rejected
    let req_bad = SandboxExecutionRequest {
        command: "   ".into(),
        args: vec![],
        cwd: None,
        profile: SandboxProfile::standard(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };
    assert!(req_bad.validate().unwrap_err().contains(ERR_SANDBOX_EMPTY_COMMAND));

    // Traversal CWD rejected
    let req_traversal = SandboxExecutionRequest {
        command: "/bin/ls".into(),
        args: vec![],
        cwd: Some("/home/../root".into()),
        profile: SandboxProfile::standard(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };
    assert!(req_traversal.validate().unwrap_err().contains(ERR_SANDBOX_INVALID_PATH));

    // Valid execution request
    let req_good = SandboxExecutionRequest {
        command: "/usr/bin/find".into(),
        args: vec![".cargo".into(), "-type".into(), "f".into()],
        cwd: Some("/workspace".into()),
        profile: SandboxProfile::standard(),
        session_id: Some("sess-001".into()),
        pep_grant_id: Some("grant-abc".into()),
        stdin_data: Some("test-stdin".into()),
    };
    assert!(req_good.validate().is_ok());

    // Result evaluation
    let res_success = SandboxExecutionResult {
        exit_code: 0,
        status: SandboxExecutionStatus::Success,
        stdout: "output".into(),
        stderr: "".into(),
        duration_ms: 125,
        components_applied: vec![],
        audit_hash: Some("abcdef123456".into()),
    };
    assert!(res_success.is_success());

    let res_failed = SandboxExecutionResult {
        exit_code: 137,
        status: SandboxExecutionStatus::Signaled(9),
        stdout: "".into(),
        stderr: "Killed".into(),
        duration_ms: 30000,
        components_applied: vec![],
        audit_hash: None,
    };
    assert!(!res_failed.is_success());
}

#[test]
fn test_json_roundtrip_and_canonical_hash() {
    let p = SandboxProfile::standard();
    let json_str = serde_json::to_string(&p).expect("serialize");
    let deserialized: SandboxProfile = serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(p, deserialized);

    let hash1 = p.canonical_hash().expect("hash 1");
    let hash2 = deserialized.canonical_hash().expect("hash 2");
    assert_eq!(hash1, hash2);
}

#[test]
fn test_hardening_bounds_exceeded() {
    let mut p = SandboxProfile::standard();
    p.name = "a".repeat(MAX_PROFILE_NAME_LEN + 1);
    let res = p.validate();
    assert!(res.is_err());
    assert!(res.unwrap_err().contains(ERR_SANDBOX_BOUNDS_EXCEEDED));

    let mut fs = FilesystemPolicy::default();
    fs.paths_ro.push("/".to_string() + &"b".repeat(4097));
    let res2 = fs.validate();
    assert!(res2.is_err());
    assert!(res2.unwrap_err().contains(ERR_SANDBOX_BOUNDS_EXCEEDED));
}

