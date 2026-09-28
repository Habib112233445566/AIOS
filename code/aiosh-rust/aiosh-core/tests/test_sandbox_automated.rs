//! Formal Automated Test Suite for Sandbox Enforcement (T-02451..T-02454).
//!
//! Formal Vectors:
//! - AUTOSANDBOX1: Profile Lifecycle & Invariant Validation
//! - AUTOSANDBOX2: Filesystem Policy Conflict & Traversal Stress
//! - AUTOSANDBOX3: Syscall & Network Isolation Modes
//! - AUTOSANDBOX4: Resource Limits Boundary Conditions
//! - AUTOSANDBOX5: Supervised Execution Lifecycle & Exit Propagation
//! - AUTOSANDBOX6: Output Capture Truncation & Memory Bounding
//! - AUTOSANDBOX7: PEP Grant Authorization Gating
//! - AUTOSANDBOX8: Thread-Safe Concurrent Invocations & Audit Logging

use aiosh_core::audit::{AuditRing, OpenOptions};
use aiosh_core::sandbox_config::*;
use aiosh_core::sandbox_data_model::*;
use aiosh_core::sandbox_service::*;
use std::sync::{Arc, Mutex};
use std::thread;
use tempfile::tempdir;

#[test]
fn test_autosandbox1_profile_lifecycle_and_invariants() {
    let mut svc = SandboxService::with_default_profiles(None);

    // Verify 3 factory profiles
    let profiles = svc.list_profiles();
    assert_eq!(profiles.len(), 3);
    assert!(svc.get_profile("standard").is_some());
    assert!(svc.get_profile("strict").is_some());
    assert!(svc.get_profile("permissive").is_some());

    // Protected profiles cannot be deleted
    assert!(svc.remove_profile("standard").is_err());
    assert!(svc.remove_profile("strict").is_err());
    assert!(svc.remove_profile("permissive").is_err());

    // Register custom profile
    let mut custom = SandboxProfile::standard();
    custom.name = "custom_analyst".into();
    assert!(svc.register_profile(custom).is_ok());
    assert_eq!(svc.list_profiles().len(), 4);

    // Re-registering duplicate name fails
    let mut dupe = SandboxProfile::standard();
    dupe.name = "custom_analyst".into();
    let res = svc.register_profile(dupe);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains(ERR_SANDBOX_PROFILE_EXISTS));

    // Custom profile can be removed
    assert_eq!(svc.remove_profile("custom_analyst").unwrap(), true);
    assert_eq!(svc.list_profiles().len(), 3);
}

#[test]
fn test_autosandbox2_filesystem_policy_conflict_and_traversal() {
    // 1. Conflict detection: same path in read_only and read_write
    let mut p = SandboxProfile::standard();
    p.filesystem.paths_ro.push("/opt/data".into());
    p.filesystem.paths_rw.push("/opt/data".into());
    let err = p.validate().unwrap_err();
    assert!(err.contains(ERR_SANDBOX_POLICY_CONFLICT));

    // 2. Directory traversal rejection
    let mut p_trav = SandboxProfile::standard();
    p_trav.filesystem.paths_ro.push("/opt/../etc/passwd".into());
    let err_trav = p_trav.validate().unwrap_err();
    assert!(err_trav.contains(ERR_SANDBOX_INVALID_PATH));

    // 3. Excessive path length
    let mut p_len = SandboxProfile::standard();
    p_len.filesystem.paths_ro.push("a".repeat(MAX_PATH_LEN + 1));
    let err_len = p_len.validate().unwrap_err();
    assert!(err_len.contains(ERR_SANDBOX_BOUNDS_EXCEEDED));
}

#[test]
fn test_autosandbox3_syscall_and_network_modes() {
    let mut profile = SandboxProfile::strict();
    assert_eq!(profile.network, NetworkIsolationMode::Disabled);
    assert_eq!(profile.isolation_level, IsolationLevel::FullLandlockSeccomp);

    profile.network = NetworkIsolationMode::LoopbackOnly;
    assert_eq!(profile.network, NetworkIsolationMode::LoopbackOnly);

    profile.syscall.default_action = SyscallAction::ReturnErrno(1);
    assert!(profile.validate().is_ok());
}

#[test]
fn test_autosandbox4_resource_limits_boundary_conditions() {
    let mut limits = ResourceLimits::default();
    assert!(limits.validate().is_ok());

    // Zero memory fails
    limits.max_memory_bytes = 0;
    assert!(limits.validate().is_err());

    // Exceeding 64 GiB fails
    limits.max_memory_bytes = 69 * 1024 * 1024 * 1024;
    assert!(limits.validate().is_err());

    // Zero wall timeout fails
    limits = ResourceLimits::default();
    limits.max_wall_time_ms = 0;
    assert!(limits.validate().is_err());
}

#[test]
fn test_autosandbox5_supervised_execution_lifecycle() {
    let mut svc = SandboxService::with_default_profiles(None);
    let profile = svc.get_profile("permissive").unwrap();

    // 1. Success execution (exit code 0)
    let req_ok = SandboxExecutionRequest {
        command: "python".into(),
        args: vec!["-c".into(), "print('lifecycle_success')".into()],
        cwd: None,
        profile: profile.clone(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };
    let res_ok = svc.execute(&req_ok).expect("exec ok");
    assert_eq!(res_ok.exit_code, 0);
    assert!(res_ok.stdout.contains("lifecycle_success"));

    // 2. Non-zero exit code propagation
    let req_fail = SandboxExecutionRequest {
        command: "python".into(),
        args: vec!["-c".into(), "import sys; sys.exit(42)".into()],
        cwd: None,
        profile: profile.clone(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };
    let res_fail = svc.execute(&req_fail).expect("exec fail");
    assert_eq!(res_fail.exit_code, 42);

    // 3. Nonexistent binary (returns 127)
    let req_missing = SandboxExecutionRequest {
        command: "nonexistent_executable_binary_12345".into(),
        args: vec![],
        cwd: None,
        profile,
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };
    let res_missing = svc.execute(&req_missing).expect("exec missing");
    assert_eq!(res_missing.exit_code, 127);
}

#[test]
fn test_autosandbox6_output_capture_truncation() {
    let mut config = SandboxConfig::default();
    config.max_output_capture_bytes = 1024; // Clamp capture to 1 KiB
    let mut svc = SandboxService::new(None, config);
    let profile = svc.get_profile("permissive").unwrap();

    let req = SandboxExecutionRequest {
        command: "python".into(),
        args: vec!["-c".into(), "print('A' * 10000)".into()],
        cwd: None,
        profile,
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };

    let res = svc.execute(&req).expect("exec output truncation");
    assert_eq!(res.exit_code, 0);
    assert_eq!(res.stdout.len(), 1024);
}

#[test]
fn test_autosandbox7_pep_grant_authorization_gating() {
    let mut config = SandboxConfig::default();
    config.enforce_pep_grants = true;
    let mut svc = SandboxService::new(None, config);
    let profile = svc.get_profile("permissive").unwrap();

    // Missing grant -> fails closed
    let req_no_grant = SandboxExecutionRequest {
        command: "python".into(),
        args: vec!["-c".into(), "print('no_grant')".into()],
        cwd: None,
        profile: profile.clone(),
        session_id: None,
        pep_grant_id: None,
        stdin_data: None,
    };
    let res_err = svc.execute(&req_no_grant);
    assert!(res_err.is_err());
    assert!(res_err.unwrap_err().contains(ERR_SANDBOX_PEP_UNAUTHORIZED));

    // Valid grant supplied -> succeeds
    let req_with_grant = SandboxExecutionRequest {
        command: "python".into(),
        args: vec!["-c".into(), "print('with_grant')".into()],
        cwd: None,
        profile,
        session_id: None,
        pep_grant_id: Some("grant_authorized_123".into()),
        stdin_data: None,
    };
    let res_ok = svc.execute(&req_with_grant).expect("authorized exec");
    assert_eq!(res_ok.exit_code, 0);
}

#[test]
fn test_autosandbox8_thread_safe_concurrent_invocations() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("autosandbox8.db");
    let ring = AuditRing::open(OpenOptions {
        path: Some(db_path.to_str().unwrap().to_string()),
        home: None,
    }).expect("open ring");

    let svc = SandboxService::with_default_profiles(Some(ring));
    let shared_svc = Arc::new(Mutex::new(svc));

    let mut handles = Vec::new();
    for thread_idx in 0..4 {
        let svc_clone = Arc::clone(&shared_svc);
        let h = thread::spawn(move || {
            for i in 0..5 {
                let profile = {
                    let s = svc_clone.lock().unwrap();
                    s.get_profile("permissive").unwrap()
                };
                let req = SandboxExecutionRequest {
                    command: "python".into(),
                    args: vec!["-c".into(), format!("print('thread_{}_iter_{}')", thread_idx, i)],
                    cwd: None,
                    profile,
                    session_id: Some(format!("sess_{}", thread_idx)),
                    pep_grant_id: None,
                    stdin_data: None,
                };
                let mut s = svc_clone.lock().unwrap();
                let res = s.execute(&req).expect("concurrent exec");
                assert_eq!(res.exit_code, 0);
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.join().expect("thread join");
    }

    // Inspect audit rows
    let s = shared_svc.lock().unwrap();
    if let Some(r) = s.ring() {
        let rows = r.tail(50).expect("tail audit rows");
        assert_eq!(rows.len(), 20); // 4 threads * 5 iterations
    }
}
