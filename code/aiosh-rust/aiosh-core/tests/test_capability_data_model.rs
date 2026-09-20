//! Focused unit tests for Capability Model Data Model (CAP1..CAP6).

use chrono::{Duration, Utc};
use aiosh_core::capability::{
    Capability, CapabilityConstraints, CapabilityError, CapabilityRight, CapabilityScope,
};

#[test]
fn test_cap1_creation_and_unforgeability() {
    let scope = CapabilityScope::Filesystem {
        path: "/var/log".to_string(),
        recursive: true,
    };
    let rights = vec![CapabilityRight::Read, CapabilityRight::Delegate];
    let constraints = CapabilityConstraints::default();

    let cap = Capability::new("kernel", "agent:sec_audit", scope.clone(), rights.clone(), constraints).unwrap();

    assert!(cap.id.starts_with("cap_"));
    assert_eq!(cap.issuer, "kernel");
    assert_eq!(cap.subject, "agent:sec_audit");
    assert_eq!(cap.rights, rights);
    assert!(!cap.revoked);
    assert!(cap.parent_id.is_none());

    // Validation failures
    assert!(Capability::new("", "subject", scope.clone(), rights.clone(), CapabilityConstraints::default()).is_err());
    assert!(Capability::new("issuer", "", scope.clone(), rights.clone(), CapabilityConstraints::default()).is_err());
    assert!(Capability::new("issuer", "subject", scope, vec![], CapabilityConstraints::default()).is_err());
}

#[test]
fn test_cap2_rights_and_scoping() {
    let fs_cap = Capability::new(
        "kernel",
        "agent:fs_worker",
        CapabilityScope::Filesystem {
            path: "/var/data".to_string(),
            recursive: true,
        },
        vec![CapabilityRight::Read, CapabilityRight::Write],
        CapabilityConstraints::default(),
    ).unwrap();

    assert!(fs_cap.has_right(CapabilityRight::Read));
    assert!(fs_cap.has_right(CapabilityRight::Write));
    assert!(!fs_cap.has_right(CapabilityRight::Delete));
    assert!(fs_cap.check_right(CapabilityRight::Read).is_ok());
    assert_eq!(
        fs_cap.check_right(CapabilityRight::Delete),
        Err(CapabilityError::RightNotGranted(CapabilityRight::Delete))
    );

    // Scope matching
    assert!(fs_cap.matches_scope(&CapabilityScope::Filesystem {
        path: "/var/data/sub/file.txt".to_string(),
        recursive: false,
    }));
    assert!(!fs_cap.matches_scope(&CapabilityScope::Filesystem {
        path: "/etc/shadow".to_string(),
        recursive: false,
    }));

    // Tool scoping
    let tool_cap = Capability::new(
        "kernel",
        "agent:tool_user",
        CapabilityScope::Tool {
            tool_name: "system_update".to_string(),
            allowed_actions: vec!["check".to_string(), "download".to_string()],
        },
        vec![CapabilityRight::Execute],
        CapabilityConstraints::default(),
    ).unwrap();

    assert!(tool_cap.matches_scope(&CapabilityScope::Tool {
        tool_name: "system_update".to_string(),
        allowed_actions: vec!["check".to_string()],
    }));
    assert!(!tool_cap.matches_scope(&CapabilityScope::Tool {
        tool_name: "system_update".to_string(),
        allowed_actions: vec!["apply".to_string()],
    }));
    assert!(!tool_cap.matches_scope(&CapabilityScope::Tool {
        tool_name: "other_tool".to_string(),
        allowed_actions: vec!["check".to_string()],
    }));
}

#[test]
fn test_cap3_monotonic_attenuation() {
    let parent = Capability::new(
        "kernel",
        "agent:orchestrator",
        CapabilityScope::Filesystem {
            path: "/var/data".to_string(),
            recursive: true,
        },
        vec![CapabilityRight::Read, CapabilityRight::Write, CapabilityRight::Delegate],
        CapabilityConstraints::default(),
    ).unwrap();

    // Valid attenuation: narrower rights and subpath
    let child = parent.attenuate(
        "agent:worker_read",
        Some(CapabilityScope::Filesystem {
            path: "/var/data/sub".to_string(),
            recursive: false,
        }),
        vec![CapabilityRight::Read],
        None,
    ).unwrap();

    assert_eq!(child.parent_id, Some(parent.id.clone()));
    assert_eq!(child.issuer, "agent:orchestrator");
    assert_eq!(child.subject, "agent:worker_read");
    assert_eq!(child.rights, vec![CapabilityRight::Read]);

    // Invalid attenuation 1: attempting to grant a right parent lacks (e.g. Delete)
    let err_priv = parent.attenuate(
        "agent:escalator",
        None,
        vec![CapabilityRight::Read, CapabilityRight::Delete],
        None,
    );
    assert!(matches!(err_priv, Err(CapabilityError::InvalidAttenuation(_))));

    // Invalid attenuation 2: attempting to grant scope outside parent
    let err_scope = parent.attenuate(
        "agent:escaped",
        Some(CapabilityScope::Filesystem {
            path: "/etc".to_string(),
            recursive: false,
        }),
        vec![CapabilityRight::Read],
        None,
    );
    assert!(matches!(err_scope, Err(CapabilityError::InvalidAttenuation(_))));

    // Invalid attenuation 3: parent without Delegate right cannot attenuate
    let child_no_delegate = Capability::new(
        "kernel",
        "agent:leaf",
        CapabilityScope::System { subsystem: "all".to_string() },
        vec![CapabilityRight::Read],
        CapabilityConstraints::default(),
    ).unwrap();

    let err_no_del = child_no_delegate.attenuate(
        "agent:sub_leaf",
        None,
        vec![CapabilityRight::Read],
        None,
    );
    assert_eq!(err_no_del, Err(CapabilityError::RightNotGranted(CapabilityRight::Delegate)));
}

#[test]
fn test_cap4_temporal_and_quotas() {
    let now = Utc::now();
    let past = (now - Duration::hours(2)).to_rfc3339();
    let future = (now + Duration::hours(2)).to_rfc3339();

    // Expired capability
    let mut expired_cap = Capability::new(
        "kernel",
        "agent:test",
        CapabilityScope::System { subsystem: "sys".to_string() },
        vec![CapabilityRight::Read],
        CapabilityConstraints {
            expires_at: Some(past),
            ..Default::default()
        },
    ).unwrap();

    assert!(matches!(expired_cap.check_validity_at(now), Err(CapabilityError::Expired(_))));
    assert!(matches!(expired_cap.consume_invocation(), Err(CapabilityError::Expired(_))));

    // Not yet valid capability
    let premature_cap = Capability::new(
        "kernel",
        "agent:test",
        CapabilityScope::System { subsystem: "sys".to_string() },
        vec![CapabilityRight::Read],
        CapabilityConstraints {
            not_before: Some(future),
            ..Default::default()
        },
    ).unwrap();

    assert!(matches!(premature_cap.check_validity_at(now), Err(CapabilityError::NotYetValid(_))));

    // Quotas: max invocations
    let mut quota_cap = Capability::new(
        "kernel",
        "agent:test",
        CapabilityScope::System { subsystem: "sys".to_string() },
        vec![CapabilityRight::Read],
        CapabilityConstraints {
            max_invocations: Some(2),
            ..Default::default()
        },
    ).unwrap();

    assert!(quota_cap.consume_invocation().is_ok());
    assert_eq!(quota_cap.constraints.current_invocations, 1);
    assert!(quota_cap.consume_invocation().is_ok());
    assert_eq!(quota_cap.constraints.current_invocations, 2);
    assert_eq!(quota_cap.consume_invocation(), Err(CapabilityError::InvocationQuotaExceeded));

    // Byte quota on dedicated capability
    let mut byte_cap = Capability::new(
        "kernel",
        "agent:test_bytes",
        CapabilityScope::System { subsystem: "sys".to_string() },
        vec![CapabilityRight::Read],
        CapabilityConstraints {
            quota_bytes: Some(100),
            ..Default::default()
        },
    ).unwrap();

    assert!(byte_cap.consume_bytes(60).is_ok());
    assert_eq!(byte_cap.constraints.consumed_bytes, 60);
    assert_eq!(byte_cap.consume_bytes(50), Err(CapabilityError::ByteQuotaExceeded));
    assert!(byte_cap.consume_bytes(40).is_ok());
    assert_eq!(byte_cap.constraints.consumed_bytes, 100);
}

#[test]
fn test_cap5_revocation() {
    let mut cap = Capability::new(
        "kernel",
        "agent:worker",
        CapabilityScope::System { subsystem: "sys".to_string() },
        vec![CapabilityRight::Read],
        CapabilityConstraints::default(),
    ).unwrap();

    assert!(cap.check_validity_at(Utc::now()).is_ok());
    cap.revoke();
    assert!(cap.revoked);
    assert_eq!(cap.check_validity_at(Utc::now()), Err(CapabilityError::Revoked));
    assert_eq!(cap.consume_invocation(), Err(CapabilityError::Revoked));
    assert_eq!(cap.consume_bytes(10), Err(CapabilityError::Revoked));
}

#[test]
fn test_cap6_json_serialization_roundtrip() {
    let cap = Capability::new(
        "kernel",
        "agent:net_manager",
        CapabilityScope::Network {
            host: "api.aios.local".to_string(),
            port: Some(8443),
            protocol: "https".to_string(),
        },
        vec![CapabilityRight::Read, CapabilityRight::Write, CapabilityRight::Delegate],
        CapabilityConstraints {
            expires_at: Some("2026-12-31T23:59:59Z".to_string()),
            max_invocations: Some(1000),
            quota_bytes: Some(1048576),
            ..Default::default()
        },
    ).unwrap();

    let json_str = serde_json::to_string_pretty(&cap).unwrap();
    let deserialized: Capability = serde_json::from_str(&json_str).unwrap();

    assert_eq!(cap, deserialized);
}
