//! Unit test suite for Capability Security Policy (T-02065).
//!
//! Tests policy validation, prohibited paths, disallowed rights, temporal bounds,
//! quota ceilings, attenuation depth limits, and policy enforcement modes.

use chrono::{Duration, Utc};

use aiosh_core::capability::{
    CapabilityConstraints, CapabilityRight, CapabilityScope,
};
use aiosh_core::capability_policy::{
    CapabilityPolicyMode, CapabilitySecurityPolicy,
};
use aiosh_core::capability_service::CapabilityService;

#[test]
fn test_policy_validation() {
    let mut policy = CapabilitySecurityPolicy::default();
    assert!(policy.validate().is_ok());

    // Invalid attenuation depth (0)
    policy.max_attenuation_depth = 0;
    assert!(policy.validate().is_err());

    // Invalid attenuation depth (> 128)
    policy.max_attenuation_depth = 129;
    assert!(policy.validate().is_err());

    policy.max_attenuation_depth = 8;

    // Invalid prohibited path (empty)
    policy.prohibited_path_prefixes.push("".into());
    assert!(policy.validate().is_err());
    policy.prohibited_path_prefixes.pop();

    // Invalid prohibited path (path traversal)
    policy.prohibited_path_prefixes.push("/etc/../var".into());
    assert!(policy.validate().is_err());
    policy.prohibited_path_prefixes.pop();

    // Invalid duration (<= 0)
    policy.max_validity_duration_seconds = Some(0);
    assert!(policy.validate().is_err());
    policy.max_validity_duration_seconds = Some(-10);
    assert!(policy.validate().is_err());
}

#[test]
fn test_policy_prohibited_filesystem_paths() {
    let policy = CapabilitySecurityPolicy::default();

    // 1. Prohibited paths
    let prohibited_paths = vec![
        "/etc/shadow",
        "/proc/cpuinfo",
        "/sys/kernel/security",
        "/dev/mem",
        "/root/.ssh/id_rsa",
        "C:\\Windows\\System32\\config\\SAM",
    ];

    for path in prohibited_paths {
        let scope = CapabilityScope::Filesystem {
            path: path.into(),
            recursive: false,
        };
        let verdict = policy.evaluate_issuance(
            "kernel",
            "agent:test",
            &scope,
            &[CapabilityRight::Read],
            &CapabilityConstraints::default(),
        );
        assert!(!verdict.allowed, "path '{}' should be blocked", path);
        assert!(verdict.violations.iter().any(|v| v.rule_id == "CAPSEC_PROHIBITED_PATH"));
    }

    // 2. Allowed path
    let allowed_scope = CapabilityScope::Filesystem {
        path: "/workspace/project/src".into(),
        recursive: true,
    };
    let verdict = policy.evaluate_issuance(
        "kernel",
        "agent:test",
        &allowed_scope,
        &[CapabilityRight::Read, CapabilityRight::Write],
        &CapabilityConstraints::default(),
    );
    assert!(verdict.allowed);
    assert!(verdict.violations.is_empty());

    // 3. Permissive mode
    let mut permissive_policy = CapabilitySecurityPolicy::default();
    permissive_policy.mode = CapabilityPolicyMode::Permissive;
    let verdict = permissive_policy.evaluate_issuance(
        "kernel",
        "agent:test",
        &CapabilityScope::Filesystem {
            path: "/etc/shadow".into(),
            recursive: false,
        },
        &[CapabilityRight::Read],
        &CapabilityConstraints::default(),
    );
    assert!(verdict.allowed);

    // 4. Audit mode
    let mut audit_policy = CapabilitySecurityPolicy::default();
    audit_policy.mode = CapabilityPolicyMode::Audit;
    let verdict = audit_policy.evaluate_issuance(
        "kernel",
        "agent:test",
        &CapabilityScope::Filesystem {
            path: "/etc/shadow".into(),
            recursive: false,
        },
        &[CapabilityRight::Read],
        &CapabilityConstraints::default(),
    );
    assert!(verdict.allowed); // permitted in audit mode
    assert_eq!(verdict.violations.len(), 1);
    assert_eq!(verdict.violations[0].rule_id, "CAPSEC_PROHIBITED_PATH");
}

#[test]
fn test_policy_prohibited_network_hosts() {
    let policy = CapabilitySecurityPolicy::default();

    // Cloud metadata service access blocked
    let scope_cloud_meta = CapabilityScope::Network {
        host: "169.254.169.254".into(),
        port: Some(80),
        protocol: "http".into(),
    };
    let verdict = policy.evaluate_issuance(
        "kernel",
        "agent:test",
        &scope_cloud_meta,
        &[CapabilityRight::Read],
        &CapabilityConstraints::default(),
    );
    assert!(!verdict.allowed);
    assert!(verdict.violations.iter().any(|v| v.rule_id == "CAPSEC_PROHIBITED_HOST"));

    // Legitimate external host allowed
    let scope_api = CapabilityScope::Network {
        host: "api.github.com".into(),
        port: Some(443),
        protocol: "https".into(),
    };
    let verdict = policy.evaluate_issuance(
        "kernel",
        "agent:test",
        &scope_api,
        &[CapabilityRight::Read],
        &CapabilityConstraints::default(),
    );
    assert!(verdict.allowed);
}

#[test]
fn test_policy_prohibited_tools() {
    let policy = CapabilitySecurityPolicy::default();

    // Privileged syscall tool blocked
    let scope_syscall = CapabilityScope::Tool {
        tool_name: "raw_syscall".into(),
        allowed_actions: vec!["call".into()],
    };
    let verdict = policy.evaluate_issuance(
        "kernel",
        "agent:test",
        &scope_syscall,
        &[CapabilityRight::Execute],
        &CapabilityConstraints::default(),
    );
    assert!(!verdict.allowed);
    assert!(verdict.violations.iter().any(|v| v.rule_id == "CAPSEC_PROHIBITED_TOOL"));

    // Normal tool allowed
    let scope_grep = CapabilityScope::Tool {
        tool_name: "grep".into(),
        allowed_actions: vec!["search".into()],
    };
    let verdict = policy.evaluate_issuance(
        "kernel",
        "agent:test",
        &scope_grep,
        &[CapabilityRight::Execute],
        &CapabilityConstraints::default(),
    );
    assert!(verdict.allowed);
}

#[test]
fn test_policy_disallowed_rights_by_subject() {
    let policy = CapabilitySecurityPolicy::default();

    // Untrusted agent requesting Admin / Delegate / Delete
    let scope = CapabilityScope::Filesystem {
        path: "/workspace/temp".into(),
        recursive: true,
    };
    let verdict_admin = policy.evaluate_issuance(
        "kernel",
        "untrusted:agent_1",
        &scope,
        &[CapabilityRight::Read, CapabilityRight::Admin],
        &CapabilityConstraints::default(),
    );
    assert!(!verdict_admin.allowed);
    assert!(verdict_admin.violations.iter().any(|v| v.rule_id == "CAPSEC_DISALLOWED_RIGHT"));

    let verdict_delegate = policy.evaluate_issuance(
        "kernel",
        "untrusted:agent_2",
        &scope,
        &[CapabilityRight::Read, CapabilityRight::Delegate],
        &CapabilityConstraints::default(),
    );
    assert!(!verdict_delegate.allowed);

    // Untrusted agent requesting Read and Write is allowed
    let verdict_read_write = policy.evaluate_issuance(
        "kernel",
        "untrusted:agent_3",
        &scope,
        &[CapabilityRight::Read, CapabilityRight::Write],
        &CapabilityConstraints::default(),
    );
    assert!(verdict_read_write.allowed);

    // Guest subject requesting Write is blocked
    let verdict_guest_write = policy.evaluate_issuance(
        "kernel",
        "guest:visitor",
        &scope,
        &[CapabilityRight::Read, CapabilityRight::Write],
        &CapabilityConstraints::default(),
    );
    assert!(!verdict_guest_write.allowed);
}

#[test]
fn test_policy_temporal_and_quota_constraints() {
    let mut policy = CapabilitySecurityPolicy::default();
    policy.require_temporal_bounds = true;
    policy.max_validity_duration_seconds = Some(3600); // 1 hour max
    policy.max_invocations_ceiling = Some(100);
    policy.max_bytes_ceiling = Some(1_000_000);

    let scope = CapabilityScope::Filesystem {
        path: "/workspace/temp".into(),
        recursive: false,
    };

    // 1. Missing temporal bound
    let verdict_no_exp = policy.evaluate_issuance(
        "kernel",
        "agent:worker",
        &scope,
        &[CapabilityRight::Read],
        &CapabilityConstraints::default(),
    );
    assert!(!verdict_no_exp.allowed);
    assert!(verdict_no_exp.violations.iter().any(|v| v.rule_id == "CAPSEC_MISSING_TEMPORAL_BOUND"));

    // 2. Excessive validity duration (2 hours > 1 hour ceiling)
    let excessive_constraints = CapabilityConstraints {
        not_before: None,
        expires_at: Some((Utc::now() + Duration::hours(2)).to_rfc3339()),
        max_invocations: Some(50),
        current_invocations: 0,
        quota_bytes: Some(500),
        consumed_bytes: 0,
    };
    let verdict_excessive = policy.evaluate_issuance(
        "kernel",
        "agent:worker",
        &scope,
        &[CapabilityRight::Read],
        &excessive_constraints,
    );
    assert!(!verdict_excessive.allowed);
    assert!(verdict_excessive.violations.iter().any(|v| v.rule_id == "CAPSEC_EXCESSIVE_TEMPORAL_BOUND"));

    // 3. Invocation ceiling exceeded (150 > 100)
    let excessive_invocations = CapabilityConstraints {
        not_before: None,
        expires_at: Some((Utc::now() + Duration::minutes(30)).to_rfc3339()),
        max_invocations: Some(150),
        current_invocations: 0,
        quota_bytes: Some(500),
        consumed_bytes: 0,
    };
    let verdict_inv = policy.evaluate_issuance(
        "kernel",
        "agent:worker",
        &scope,
        &[CapabilityRight::Read],
        &excessive_invocations,
    );
    assert!(!verdict_inv.allowed);
    assert!(verdict_inv.violations.iter().any(|v| v.rule_id == "CAPSEC_INVOCATIONS_CEILING_EXCEEDED"));

    // 4. Valid constraints pass
    let valid_constraints = CapabilityConstraints {
        not_before: None,
        expires_at: Some((Utc::now() + Duration::minutes(30)).to_rfc3339()),
        max_invocations: Some(50),
        current_invocations: 0,
        quota_bytes: Some(500),
        consumed_bytes: 0,
    };
    let verdict_valid = policy.evaluate_issuance(
        "kernel",
        "agent:worker",
        &scope,
        &[CapabilityRight::Read],
        &valid_constraints,
    );
    assert!(verdict_valid.allowed);
}

#[test]
fn test_policy_attenuation_depth_limit() {
    let mut custom_policy = CapabilitySecurityPolicy::default();
    custom_policy.max_attenuation_depth = 3;

    let mut service = CapabilityService::new().with_policy(custom_policy);

    let root = service
        .issue_root_capability(
            "kernel",
            "agent:root",
            CapabilityScope::Filesystem {
                path: "/data".into(),
                recursive: true,
            },
            vec![CapabilityRight::Read, CapabilityRight::Delegate],
            CapabilityConstraints::default(),
        )
        .expect("issue root");

    // Depth 1
    let child1 = service
        .attenuate_capability(
            &root.id,
            "agent:child1",
            None,
            vec![CapabilityRight::Read, CapabilityRight::Delegate],
            None,
        )
        .expect("child1 should succeed");
    assert_eq!(service.get_derivation_depth(&child1.id), 1);

    // Depth 2
    let child2 = service
        .attenuate_capability(
            &child1.id,
            "agent:child2",
            None,
            vec![CapabilityRight::Read, CapabilityRight::Delegate],
            None,
        )
        .expect("child2 should succeed");
    assert_eq!(service.get_derivation_depth(&child2.id), 2);

    // Depth 3
    let child3 = service
        .attenuate_capability(
            &child2.id,
            "agent:child3",
            None,
            vec![CapabilityRight::Read, CapabilityRight::Delegate],
            None,
        )
        .expect("child3 should succeed");
    assert_eq!(service.get_derivation_depth(&child3.id), 3);

    // Depth 4: exceeds policy max_attenuation_depth of 3
    let res4 = service.attenuate_capability(
        &child3.id,
        "agent:child4",
        None,
        vec![CapabilityRight::Read],
        None,
    );
    assert!(res4.is_err());
    let err_msg = format!("{}", res4.err().unwrap());
    assert!(err_msg.contains("CAPSEC_DEPTH_EXCEEDED"));
}

#[test]
fn test_capability_service_policy_enforcement() {
    let mut service = CapabilityService::new();

    // 1. Root issuance with prohibited path rejected
    let res_prohibited_root = service.issue_root_capability(
        "kernel",
        "agent:attacker",
        CapabilityScope::Filesystem {
            path: "/etc/shadow".into(),
            recursive: false,
        },
        vec![CapabilityRight::Read],
        CapabilityConstraints::default(),
    );
    assert!(res_prohibited_root.is_err());
    let err = format!("{}", res_prohibited_root.err().unwrap());
    assert!(err.contains("CAPSEC_PROHIBITED_PATH"));

    // 2. Root issuance with disallowed right for untrusted subject rejected
    let res_untrusted_admin = service.issue_root_capability(
        "kernel",
        "untrusted:malicious_agent",
        CapabilityScope::Filesystem {
            path: "/workspace/temp".into(),
            recursive: false,
        },
        vec![CapabilityRight::Read, CapabilityRight::Admin],
        CapabilityConstraints::default(),
    );
    assert!(res_untrusted_admin.is_err());
    let err = format!("{}", res_untrusted_admin.err().unwrap());
    assert!(err.contains("CAPSEC_DISALLOWED_RIGHT"));

    // 3. Valid root capability succeeds
    let root = service
        .issue_root_capability(
            "kernel",
            "agent:trusted",
            CapabilityScope::Filesystem {
                path: "/workspace/data".into(),
                recursive: true,
            },
            vec![CapabilityRight::Read, CapabilityRight::Write, CapabilityRight::Delegate],
            CapabilityConstraints::default(),
        )
        .expect("valid root capability should succeed");

    // 4. Attenuation to untrusted subject granting Admin is blocked
    let res_attenuate_untrusted_admin = service.attenuate_capability(
        &root.id,
        "untrusted:worker",
        None,
        vec![CapabilityRight::Admin],
        None,
    );
    assert!(res_attenuate_untrusted_admin.is_err());

    // 5. Attenuation to untrusted subject granting Read succeeds
    let child = service
        .attenuate_capability(
            &root.id,
            "untrusted:worker",
            None,
            vec![CapabilityRight::Read],
            None,
        )
        .expect("valid attenuation to untrusted worker should succeed");

    assert_eq!(child.rights, vec![CapabilityRight::Read]);
    assert_eq!(child.subject, "untrusted:worker");
}
