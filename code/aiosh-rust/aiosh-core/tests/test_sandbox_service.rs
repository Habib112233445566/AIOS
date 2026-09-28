//! Standalone automated unit and integration tests for Sandbox Enforcement Core Service (T-02415).

use aiosh_core::audit::AuditRing;
use aiosh_core::sandbox_data_model::{
    SandboxExecutionRequest, SandboxExecutionStatus, SandboxProfile, SandboxProfileBuilder,
    ERR_SANDBOX_EMPTY_COMMAND, ERR_SANDBOX_INVALID_PATH,
};
use aiosh_core::sandbox_service::{
    SandboxConfig, SandboxService, ERR_SANDBOX_CANNOT_DELETE_DEFAULT,
    ERR_SANDBOX_CAPACITY_EXCEEDED, ERR_SANDBOX_PEP_UNAUTHORIZED, ERR_SANDBOX_PROFILE_EXISTS,
    MAX_PROFILES_IN_SERVICE,
};

fn echo_cmd() -> (String, Vec<String>) {
    if cfg!(windows) {
        ("cmd".into(), vec!["/C".into(), "echo aios-sandbox-test".into()])
    } else {
        ("echo".into(), vec!["aios-sandbox-test".into()])
    }
}

#[test]
fn test_service_lifecycle_and_catalog() {
    let mut svc = SandboxService::with_default_profiles(None);

    // Initial default catalog contains 3 profiles
    let profiles = svc.list_profiles();
    assert_eq!(profiles.len(), 3);
    assert_eq!(profiles[0].name, "permissive");
    assert_eq!(profiles[1].name, "standard");
    assert_eq!(profiles[2].name, "strict");

    // Re-registering existing profile rejected
    let res_dup = svc.register_profile(SandboxProfile::standard());
    assert!(res_dup.unwrap_err().contains(ERR_SANDBOX_PROFILE_EXISTS));

    // Deleting default profile rejected
    let res_del_std = svc.remove_profile("standard");
    assert!(res_del_std.unwrap_err().contains(ERR_SANDBOX_CANNOT_DELETE_DEFAULT));

    // Registering and deleting custom profile succeeds
    let custom = SandboxProfileBuilder::new("ephemeral-worker")
        .max_memory_bytes(1_000_000_000)
        .build()
        .unwrap();
    assert!(svc.register_profile(custom).is_ok());
    assert!(svc.get_profile("ephemeral-worker").is_some());
    assert_eq!(svc.list_profiles().len(), 4);

    assert!(svc.remove_profile("ephemeral-worker").unwrap());
    assert_eq!(svc.list_profiles().len(), 3);
    assert!(svc.get_profile("ephemeral-worker").is_none());
}

#[test]
fn test_service_validation_rejections() {
    let mut svc = SandboxService::with_default_profiles(None);

    // Empty command rejected
    let req_empty = SandboxExecutionRequest {
        command: "".into(),
        args: vec![],
        cwd: None,
        profile: SandboxProfile::standard(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };
    assert!(svc.execute(&req_empty).unwrap_err().contains(ERR_SANDBOX_EMPTY_COMMAND));

    // Traversal CWD rejected
    let req_traversal = SandboxExecutionRequest {
        command: "echo".into(),
        args: vec![],
        cwd: Some("/path/../sensitive".into()),
        profile: SandboxProfile::standard(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };
    assert!(svc.execute(&req_traversal).unwrap_err().contains(ERR_SANDBOX_INVALID_PATH));
}

#[test]
fn test_service_command_execution_success() {
    let mut svc = SandboxService::with_default_profiles(None);
    let (bin, args) = echo_cmd();

    let req = SandboxExecutionRequest {
        command: bin,
        args,
        cwd: None,
        profile: SandboxProfile::standard(),
        session_id: Some("session-alpha".into()),
        pep_grant_id: None,
        stdin_data: None,
    };

    let result = svc.execute(&req).expect("execute");
    assert!(result.is_success());
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("aios-sandbox-test"));
    assert!(result.components_applied.iter().any(|c| c.component == "landlock"));
    assert!(result.components_applied.iter().any(|c| c.component == "seccomp"));
}

#[test]
fn test_service_missing_executable_failure() {
    let mut svc = SandboxService::with_default_profiles(None);

    let req = SandboxExecutionRequest {
        command: "non_existent_binary_123456789".into(),
        args: vec![],
        cwd: None,
        profile: SandboxProfile::standard(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };

    let result = svc.execute(&req).expect("exec attempted");
    assert!(!result.is_success());
    assert_eq!(result.exit_code, 127);
    assert!(matches!(result.status, SandboxExecutionStatus::Error(_)));
}

#[test]
fn test_service_pep_gating_behavior() {
    let mut config = SandboxConfig::default();
    config.enforce_pep_grants = true;
    let mut svc = SandboxService::new(None, config);
    let (bin, args) = echo_cmd();

    // Fails when pep_grant_id is omitted
    let req_no_grant = SandboxExecutionRequest {
        command: bin.clone(),
        args: args.clone(),
        cwd: None,
        profile: SandboxProfile::standard(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };
    let err = svc.execute(&req_no_grant).unwrap_err();
    assert!(err.contains(ERR_SANDBOX_PEP_UNAUTHORIZED));

    // Passes when pep_grant_id is supplied
    let req_with_grant = SandboxExecutionRequest {
        command: bin,
        args,
        cwd: None,
        profile: SandboxProfile::standard(),
        session_id: Some("sess-1".into()),
        pep_grant_id: Some("grant-token-999".into()),
        stdin_data: None,
    };
    let res = svc.execute(&req_with_grant).expect("execute with grant");
    assert!(res.is_success());
}

#[test]
fn test_service_audit_trail_emission() {
    let ring = AuditRing::open_in_memory().expect("open memory ring");
    let mut svc = SandboxService::with_default_profiles(Some(ring));
    let (bin, args) = echo_cmd();

    let req = SandboxExecutionRequest {
        command: bin,
        args,
        cwd: None,
        profile: SandboxProfile::standard(),
        session_id: Some("sess-audit-test".into()),
        pep_grant_id: Some("pep-grant-456".into()),
        stdin_data: None,
    };

    let result = svc.execute(&req).expect("execute with audit");
    assert!(result.is_success());
    assert!(result.audit_hash.is_some());

    // Verify row was committed to SQLite audit ring
    let tail = svc.ring().unwrap().tail(1).expect("tail audit");
    assert_eq!(tail.len(), 1);
    let row = &tail[0];
    assert_eq!(row.tool, "sandbox");
    assert_eq!(row.outcome, "ok");
    assert_eq!(row.grant_token, Some("pep-grant-456".into()));
    assert_eq!(Some(row.hash.clone()), result.audit_hash);
}

#[test]
fn test_service_output_truncation_cap() {
    let mut config = SandboxConfig::default();
    config.max_output_capture_bytes = 10; // Cap at 10 bytes
    let mut svc = SandboxService::new(None, config);
    let (bin, args) = echo_cmd();

    let req = SandboxExecutionRequest {
        command: bin,
        args,
        cwd: None,
        profile: SandboxProfile::standard(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };

    let result = svc.execute(&req).expect("execute");
    assert!(result.is_success());
    assert!(result.stdout.len() <= 10);
}

#[test]
fn test_service_capacity_limit() {
    let mut svc = SandboxService::with_default_profiles(None);
    // Already has 3 profiles (standard, strict, permissive)
    for i in 4..=MAX_PROFILES_IN_SERVICE {
        let prof = SandboxProfileBuilder::new(format!("prof-{}", i))
            .build()
            .unwrap();
        assert!(svc.register_profile(prof).is_ok());
    }
    assert_eq!(svc.list_profiles().len(), MAX_PROFILES_IN_SERVICE);

    // Registering 257th profile must fail with capacity exceeded
    let over_prof = SandboxProfileBuilder::new("overflow-profile")
        .build()
        .unwrap();
    let err = svc.register_profile(over_prof).unwrap_err();
    assert!(err.contains(ERR_SANDBOX_CAPACITY_EXCEEDED));
}

