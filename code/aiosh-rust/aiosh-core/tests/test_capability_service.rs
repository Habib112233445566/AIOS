//! Focused unit tests for CapabilityService (CSERV1..CSERV6).

use chrono::{Duration, Utc};
use tempfile::tempdir;

use aiosh_core::capability::{
    CapabilityConstraints, CapabilityError, CapabilityRight, CapabilityScope,
};
use aiosh_core::capability_service::CapabilityService;

#[test]
fn test_cserv1_root_issuance_and_indexing() {
    let mut service = CapabilityService::new();
    assert!(service.is_empty());

    let cap = service.issue_root_capability(
        "kernel",
        "agent:primary",
        CapabilityScope::Filesystem {
            path: "/var/data".to_string(),
            recursive: true,
        },
        vec![CapabilityRight::Read, CapabilityRight::Write, CapabilityRight::Delegate],
        CapabilityConstraints::default(),
    ).unwrap();

    assert_eq!(service.len(), 1);
    assert_eq!(service.get_capability(&cap.id).unwrap().subject, "agent:primary");

    let subject_caps = service.get_capabilities_for_subject("agent:primary");
    assert_eq!(subject_caps.len(), 1);
    assert_eq!(subject_caps[0].id, cap.id);

    assert!(service.get_capabilities_for_subject("agent:other").is_empty());
}

#[test]
fn test_cserv2_attenuation_and_lineage() {
    let mut service = CapabilityService::new();

    let root = service.issue_root_capability(
        "kernel",
        "agent:orchestrator",
        CapabilityScope::Filesystem {
            path: "/var/log".to_string(),
            recursive: true,
        },
        vec![CapabilityRight::Read, CapabilityRight::Delegate],
        CapabilityConstraints::default(),
    ).unwrap();

    let child = service.attenuate_capability(
        &root.id,
        "agent:sub_reader",
        Some(CapabilityScope::Filesystem {
            path: "/var/log/audit.log".to_string(),
            recursive: false,
        }),
        vec![CapabilityRight::Read],
        None,
    ).unwrap();

    assert_eq!(service.len(), 2);
    assert_eq!(child.parent_id, Some(root.id.clone()));
    assert_eq!(child.issuer, "agent:orchestrator");
    assert_eq!(child.subject, "agent:sub_reader");

    let sub_caps = service.get_capabilities_for_subject("agent:sub_reader");
    assert_eq!(sub_caps.len(), 1);
    assert_eq!(sub_caps[0].id, child.id);
}

#[test]
fn test_cserv3_cascade_revocation() {
    let mut service = CapabilityService::new();

    // 3-level tree: Root -> Child -> Grandchild
    let root = service.issue_root_capability(
        "kernel",
        "agent:orchestrator",
        CapabilityScope::System { subsystem: "all".to_string() },
        vec![CapabilityRight::Read, CapabilityRight::Delegate],
        CapabilityConstraints::default(),
    ).unwrap();

    let child = service.attenuate_capability(
        &root.id,
        "agent:manager",
        None,
        vec![CapabilityRight::Read, CapabilityRight::Delegate],
        None,
    ).unwrap();

    let grandchild = service.attenuate_capability(
        &child.id,
        "agent:worker",
        None,
        vec![CapabilityRight::Read],
        None,
    ).unwrap();

    assert_eq!(service.len(), 3);

    // Revoke root -> should revoke root, child, and grandchild
    let revoked = service.revoke_capability(&root.id).unwrap();
    assert_eq!(revoked.len(), 3);
    assert!(revoked.contains(&root.id));
    assert!(revoked.contains(&child.id));
    assert!(revoked.contains(&grandchild.id));

    // Verify all are now marked revoked
    assert!(service.get_capability(&root.id).unwrap().revoked);
    assert!(service.get_capability(&child.id).unwrap().revoked);
    assert!(service.get_capability(&grandchild.id).unwrap().revoked);

    // Verify get_capabilities_for_subject excludes them
    assert!(service.get_capabilities_for_subject("agent:orchestrator").is_empty());
    assert!(service.get_capabilities_for_subject("agent:manager").is_empty());
    assert!(service.get_capabilities_for_subject("agent:worker").is_empty());
}

#[test]
fn test_cserv4_check_access_and_quotas() {
    let mut service = CapabilityService::new();

    let cap = service.issue_root_capability(
        "kernel",
        "agent:tool_runner",
        CapabilityScope::Tool {
            tool_name: "backup".to_string(),
            allowed_actions: vec!["run".to_string()],
        },
        vec![CapabilityRight::Execute],
        CapabilityConstraints {
            max_invocations: Some(2),
            ..Default::default()
        },
    ).unwrap();

    let scope = CapabilityScope::Tool {
        tool_name: "backup".to_string(),
        allowed_actions: vec!["run".to_string()],
    };

    // Initial access check succeeds
    assert!(service.check_access("agent:tool_runner", &scope, CapabilityRight::Execute).is_ok());
    assert!(service.has_active_capability("agent:tool_runner", &scope, CapabilityRight::Execute));

    // Consume 1 invocation
    assert!(service.consume_invocation_on_capability(&cap.id).is_ok());
    assert!(service.has_active_capability("agent:tool_runner", &scope, CapabilityRight::Execute));

    // Consume 2nd invocation -> exhausts quota
    assert!(service.consume_invocation_on_capability(&cap.id).is_ok());

    // 3rd invocation fails
    assert_eq!(
        service.consume_invocation_on_capability(&cap.id),
        Err(CapabilityError::InvocationQuotaExceeded)
    );

    // check_access now fails because quota is exhausted
    assert_eq!(
        service.check_access("agent:tool_runner", &scope, CapabilityRight::Execute),
        Err(CapabilityError::RightNotGranted(CapabilityRight::Execute))
    );
}

#[test]
fn test_cserv5_persistence_atomic_roundtrip() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("capabilities.json");

    let mut service = CapabilityService::new().with_storage_path(file_path.clone());

    let root = service.issue_root_capability(
        "kernel",
        "agent:db_admin",
        CapabilityScope::Filesystem {
            path: "/var/lib/db".to_string(),
            recursive: true,
        },
        vec![CapabilityRight::Read, CapabilityRight::Write, CapabilityRight::Delegate],
        CapabilityConstraints::default(),
    ).unwrap();

    let _child = service.attenuate_capability(
        &root.id,
        "agent:db_reader",
        Some(CapabilityScope::Filesystem {
            path: "/var/lib/db/data".to_string(),
            recursive: true,
        }),
        vec![CapabilityRight::Read],
        None,
    ).unwrap();

    // Save to disk
    service.save_to_path(&file_path).unwrap();
    assert!(file_path.exists());

    // Reload from disk
    let loaded = CapabilityService::load_from_path(&file_path).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded.get_capabilities_for_subject("agent:db_admin").len(), 1);
    assert_eq!(loaded.get_capabilities_for_subject("agent:db_reader").len(), 1);
}

#[test]
fn test_cserv6_prune_expired() {
    let mut service = CapabilityService::new();
    let now = Utc::now();
    let past = (now - Duration::hours(1)).to_rfc3339();

    // Expired leaf capability
    service.issue_root_capability(
        "kernel",
        "agent:temp",
        CapabilityScope::System { subsystem: "tmp".to_string() },
        vec![CapabilityRight::Read],
        CapabilityConstraints {
            expires_at: Some(past),
            ..Default::default()
        },
    ).unwrap();

    // Valid capability
    service.issue_root_capability(
        "kernel",
        "agent:perm",
        CapabilityScope::System { subsystem: "perm".to_string() },
        vec![CapabilityRight::Read],
        CapabilityConstraints::default(),
    ).unwrap();

    assert_eq!(service.len(), 2);

    let pruned = service.prune_expired(now);
    assert_eq!(pruned, 1);
    assert_eq!(service.len(), 1);
    assert!(service.get_capabilities_for_subject("agent:temp").is_empty());
    assert_eq!(service.get_capabilities_for_subject("agent:perm").len(), 1);
}
