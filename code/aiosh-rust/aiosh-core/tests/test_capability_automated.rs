//! Automated end-to-end integration tests for Capability Model (Sub-Epic 6, CAPTEST1..CAPTEST6).

use std::fs;
use std::path::PathBuf;
use chrono::{Duration, Utc};
use tempfile::{tempdir, TempDir};

use aiosh_core::capability::{
    CapabilityConstraints, CapabilityError, CapabilityRight, CapabilityScope,
    CAP_ERROR_ATTENUATION, CAP_ERROR_QUOTA_BYTES, CAP_ERROR_QUOTA_INVOCATIONS,
};
use aiosh_core::capability_service::{
    CapabilityService, CSERV_VALIDATION_ERROR,
};

/// Helper fixture for generating hermetic mock capability environments.
pub struct MockCapabilityEnv {
    pub dir: TempDir,
    pub store_path: PathBuf,
}

impl MockCapabilityEnv {
    pub fn new() -> Self {
        let dir = tempdir().expect("create temp dir");
        let store_path = dir.path().join("capability_store.json");
        Self { dir, store_path }
    }

    pub fn populate_standard_fixtures(&self) -> Result<CapabilityService, CapabilityError> {
        let mut service = CapabilityService::new().with_storage_path(self.store_path.clone());

        // 1. Root Admin Capability
        service.issue_root_capability(
            "kernel",
            "agent:admin",
            CapabilityScope::System {
                subsystem: "all".into(),
            },
            vec![
                CapabilityRight::Read,
                CapabilityRight::Write,
                CapabilityRight::Execute,
                CapabilityRight::Delete,
                CapabilityRight::Admin,
                CapabilityRight::Delegate,
            ],
            CapabilityConstraints::default(),
        )?;

        // 2. Scoped Filesystem Worker Capability
        service.issue_root_capability(
            "kernel",
            "agent:worker",
            CapabilityScope::Filesystem {
                path: "/var/data".into(),
                recursive: true,
            },
            vec![CapabilityRight::Read, CapabilityRight::Write, CapabilityRight::Delegate],
            CapabilityConstraints {
                not_before: None,
                expires_at: Some((Utc::now() + Duration::hours(1)).to_rfc3339()),
                max_invocations: Some(50),
                current_invocations: 0,
                quota_bytes: Some(1024 * 1024),
                consumed_bytes: 0,
            },
        )?;

        // 3. Expired Capability
        service.issue_root_capability(
            "kernel",
            "agent:expired",
            CapabilityScope::Network {
                host: "api.internal".into(),
                port: Some(443),
                protocol: "tcp".into(),
            },
            vec![CapabilityRight::Read],
            CapabilityConstraints {
                not_before: None,
                expires_at: Some((Utc::now() - Duration::hours(1)).to_rfc3339()),
                max_invocations: None,
                current_invocations: 0,
                quota_bytes: None,
                consumed_bytes: 0,
            },
        )?;

        Ok(service)
    }
}

#[test]
fn test_automated_mock_env_initialization() {
    let mock = MockCapabilityEnv::new();
    let service = mock.populate_standard_fixtures().expect("fixtures populated");
    assert_eq!(service.len(), 3);
    assert_eq!(service.get_capabilities_for_subject("agent:admin").len(), 1);
    assert_eq!(service.get_capabilities_for_subject("agent:worker").len(), 1);
    // Expired capability should not appear in active capabilities for subject
    assert_eq!(service.get_capabilities_for_subject("agent:expired").len(), 0);
}

#[test]
fn test_automated_capability_lifecycle_matrix() {
    let mock = MockCapabilityEnv::new();
    let mut service = CapabilityService::new().with_storage_path(mock.store_path.clone());

    // 1. Root Issue
    let root = service
        .issue_root_capability(
            "kernel",
            "agent:tier0",
            CapabilityScope::Filesystem {
                path: "/var".into(),
                recursive: true,
            },
            vec![
                CapabilityRight::Read,
                CapabilityRight::Write,
                CapabilityRight::Delete,
                CapabilityRight::Delegate,
            ],
            CapabilityConstraints::default(),
        )
        .expect("issue root");

    // 2. Multi-tier Attenuation: Tier 0 -> Tier 1 -> Tier 2 -> Tier 3
    let tier1 = service
        .attenuate_capability(
            &root.id,
            "agent:tier1",
            Some(CapabilityScope::Filesystem {
                path: "/var/data".into(),
                recursive: true,
            }),
            vec![CapabilityRight::Read, CapabilityRight::Write, CapabilityRight::Delegate],
            None,
        )
        .expect("attenuate tier1");

    let tier2 = service
        .attenuate_capability(
            &tier1.id,
            "agent:tier2",
            Some(CapabilityScope::Filesystem {
                path: "/var/data/sub".into(),
                recursive: true,
            }),
            vec![CapabilityRight::Read, CapabilityRight::Delegate],
            None,
        )
        .expect("attenuate tier2");

    let tier3 = service
        .attenuate_capability(
            &tier2.id,
            "agent:tier3",
            None,
            vec![CapabilityRight::Read],
            None,
        )
        .expect("attenuate tier3");

    assert_eq!(service.len(), 4);

    // Verify access rights at each tier
    let scope_tier3 = CapabilityScope::Filesystem {
        path: "/var/data/sub/file.txt".into(),
        recursive: false,
    };
    assert!(service.has_active_capability("agent:tier3", &scope_tier3, CapabilityRight::Read));
    assert!(!service.has_active_capability("agent:tier3", &scope_tier3, CapabilityRight::Write));

    // 3. Cascade Revocation at Tier 1
    let revoked = service.revoke_capability(&tier1.id).expect("revoke tier1");
    assert_eq!(revoked.len(), 3);
    assert!(revoked.contains(&tier1.id));
    assert!(revoked.contains(&tier2.id));
    assert!(revoked.contains(&tier3.id));

    // Tier 1, 2, 3 must now be denied
    assert!(!service.has_active_capability("agent:tier1", &scope_tier3, CapabilityRight::Read));
    assert!(!service.has_active_capability("agent:tier2", &scope_tier3, CapabilityRight::Read));
    assert!(!service.has_active_capability("agent:tier3", &scope_tier3, CapabilityRight::Read));

    // Root (Tier 0) must remain active
    let scope_root = CapabilityScope::Filesystem {
        path: "/var/other".into(),
        recursive: true,
    };
    assert!(service.has_active_capability("agent:tier0", &scope_root, CapabilityRight::Read));
}

#[test]
fn test_automated_capability_attenuation_invariants() {
    let mock = MockCapabilityEnv::new();
    let mut service = mock.populate_standard_fixtures().expect("fixtures");

    let worker_caps = service.get_capabilities_for_subject("agent:worker");
    let worker_cap = &worker_caps[0];

    // 1. Right expansion: requesting Admin when worker only has Read, Write, Delegate
    let err_right = service.attenuate_capability(
        &worker_cap.id,
        "agent:child",
        None,
        vec![CapabilityRight::Admin],
        None,
    );
    assert!(err_right.is_err());
    assert!(err_right.unwrap_err().to_string().contains(CAP_ERROR_ATTENUATION));

    // 2. Scope expansion: requesting wider path /var when worker only has /var/data
    let err_scope = service.attenuate_capability(
        &worker_cap.id,
        "agent:child",
        Some(CapabilityScope::Filesystem {
            path: "/var".into(),
            recursive: true,
        }),
        vec![CapabilityRight::Read],
        None,
    );
    assert!(err_scope.is_err());
    assert!(err_scope.unwrap_err().to_string().contains(CAP_ERROR_ATTENUATION));

    // 3. Quota expansion: worker has max_invocations = 50, child requests 100
    let err_quota = service.attenuate_capability(
        &worker_cap.id,
        "agent:child",
        None,
        vec![CapabilityRight::Read],
        Some(CapabilityConstraints {
            not_before: None,
            expires_at: None,
            max_invocations: Some(100),
            current_invocations: 0,
            quota_bytes: None,
            consumed_bytes: 0,
        }),
    );
    assert!(err_quota.is_err());
    assert!(err_quota.unwrap_err().to_string().contains(CAP_ERROR_ATTENUATION));

    // 4. Expiration extension: worker expires in 1 hour, child requests 2 hours
    let err_exp = service.attenuate_capability(
        &worker_cap.id,
        "agent:child",
        None,
        vec![CapabilityRight::Read],
        Some(CapabilityConstraints {
            not_before: None,
            expires_at: Some((Utc::now() + Duration::hours(2)).to_rfc3339()),
            max_invocations: None,
            current_invocations: 0,
            quota_bytes: None,
            consumed_bytes: 0,
        }),
    );
    assert!(err_exp.is_err());
    assert!(err_exp.unwrap_err().to_string().contains(CAP_ERROR_ATTENUATION));
}

#[test]
fn test_automated_capability_quota_and_consumption() {
    let mock = MockCapabilityEnv::new();
    let mut service = CapabilityService::new().with_storage_path(mock.store_path.clone());

    // 1. Test invocation quota exhaustion
    let cap_inv = service
        .issue_root_capability(
            "kernel",
            "agent:metered_inv",
            CapabilityScope::Filesystem {
                path: "/data1".into(),
                recursive: true,
            },
            vec![CapabilityRight::Read],
            CapabilityConstraints {
                not_before: None,
                expires_at: None,
                max_invocations: Some(3),
                current_invocations: 0,
                quota_bytes: None,
                consumed_bytes: 0,
            },
        )
        .expect("issue metered_inv");

    let id_inv = cap_inv.id.clone();
    assert!(service.consume_invocation_on_capability(&id_inv).is_ok());
    assert!(service.consume_invocation_on_capability(&id_inv).is_ok());
    assert!(service.consume_invocation_on_capability(&id_inv).is_ok());

    // 4th invocation must fail with quota error
    let inv_err = service.consume_invocation_on_capability(&id_inv);
    assert!(inv_err.is_err());
    assert!(inv_err.unwrap_err().to_string().contains(CAP_ERROR_QUOTA_INVOCATIONS));

    // 2. Test byte quota exhaustion on separate capability
    let cap_bytes = service
        .issue_root_capability(
            "kernel",
            "agent:metered_bytes",
            CapabilityScope::Filesystem {
                path: "/data2".into(),
                recursive: true,
            },
            vec![CapabilityRight::Read],
            CapabilityConstraints {
                not_before: None,
                expires_at: None,
                max_invocations: None,
                current_invocations: 0,
                quota_bytes: Some(1000),
                consumed_bytes: 0,
            },
        )
        .expect("issue metered_bytes");

    let id_bytes = cap_bytes.id.clone();
    assert!(service.consume_bytes_on_capability(&id_bytes, 500).is_ok());
    assert!(service.consume_bytes_on_capability(&id_bytes, 500).is_ok());

    // Consuming 1 more byte exceeds 1000 quota
    let byte_err = service.consume_bytes_on_capability(&id_bytes, 1);
    assert!(byte_err.is_err());
    assert!(byte_err.unwrap_err().to_string().contains(CAP_ERROR_QUOTA_BYTES));
}

#[test]
fn test_automated_capability_persistence_and_reload() {
    let mock = MockCapabilityEnv::new();
    let service = mock.populate_standard_fixtures().expect("fixtures");

    // Persist to disk
    service.save_to_path(&mock.store_path).expect("save to disk");
    assert!(mock.store_path.exists());

    // Reload from disk
    let loaded = CapabilityService::load_from_path(&mock.store_path).expect("load from disk");
    assert_eq!(loaded.len(), service.len());
    assert_eq!(
        loaded.get_capabilities_for_subject("agent:admin").len(),
        service.get_capabilities_for_subject("agent:admin").len()
    );
    assert_eq!(
        loaded.get_capabilities_for_subject("agent:worker").len(),
        service.get_capabilities_for_subject("agent:worker").len()
    );
}

#[test]
fn test_automated_capability_pruning_and_temporal() {
    let mock = MockCapabilityEnv::new();
    let mut service = CapabilityService::new().with_storage_path(mock.store_path.clone());

    // 1. Valid parent capability
    let parent = service
        .issue_root_capability(
            "kernel",
            "agent:parent",
            CapabilityScope::System { subsystem: "core".into() },
            vec![CapabilityRight::Read, CapabilityRight::Delegate],
            CapabilityConstraints::default(),
        )
        .expect("issue parent");

    // 2. Child capability that has expired
    let _child = service
        .attenuate_capability(
            &parent.id,
            "agent:expired_child",
            None,
            vec![CapabilityRight::Read],
            Some(CapabilityConstraints {
                not_before: None,
                expires_at: Some((Utc::now() - Duration::hours(1)).to_rfc3339()),
                max_invocations: None,
                current_invocations: 0,
                quota_bytes: None,
                consumed_bytes: 0,
            }),
        )
        .expect("attenuate child");

    // 3. Another expired leaf capability
    let _leaf = service
        .issue_root_capability(
            "kernel",
            "agent:exp_leaf",
            CapabilityScope::System { subsystem: "leaf".into() },
            vec![CapabilityRight::Read],
            CapabilityConstraints {
                not_before: None,
                expires_at: Some((Utc::now() - Duration::hours(1)).to_rfc3339()),
                max_invocations: None,
                current_invocations: 0,
                quota_bytes: None,
                consumed_bytes: 0,
            },
        )
        .expect("issue leaf");

    assert_eq!(service.len(), 3);

    // Pruning: expired child and expired leaf are pruned, non-expired parent is preserved
    let pruned = service.prune_expired(Utc::now());
    assert_eq!(pruned, 2);
    assert_eq!(service.len(), 1);

    // Parent is still in registry because it was not expired
    assert!(service.get_capability(&parent.id).is_some());
}

#[test]
fn test_automated_capability_fault_injection() {
    let mock = MockCapabilityEnv::new();

    // 1. Corrupted file content
    fs::write(&mock.store_path, "not-valid-json{{{{").unwrap();
    let err_corrupt = CapabilityService::load_from_path(&mock.store_path);
    assert!(err_corrupt.is_err());
    assert!(err_corrupt.unwrap_err().contains(CSERV_VALIDATION_ERROR));

    // 2. Non-json extension
    let bad_ext = mock.dir.path().join("store.txt");
    let err_ext = CapabilityService::load_or_create(&bad_ext);
    assert!(err_ext.is_err());
    assert!(err_ext.unwrap_err().contains(CSERV_VALIDATION_ERROR));

    // 3. Path traversal in path
    let bad_path = mock.dir.path().join("../evil.json");
    let err_trav = CapabilityService::load_or_create(&bad_path);
    assert!(err_trav.is_err());
    assert!(err_trav.unwrap_err().contains(CSERV_VALIDATION_ERROR));
}

#[test]
fn test_automated_capability_deep_hierarchy_stress() {
    let mock = MockCapabilityEnv::new();
    let mut service = CapabilityService::new().with_storage_path(mock.store_path.clone());

    // Root capability
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

    // Attenuate 50 levels deep
    let mut current_id = root.id.clone();
    let mut level1_id = String::new();
    for i in 1..=50 {
        let child = service
            .attenuate_capability(
                &current_id,
                &format!("agent:level_{}", i),
                None,
                vec![CapabilityRight::Read, CapabilityRight::Delegate],
                None,
            )
            .unwrap_or_else(|e| panic!("failed at level {}: {}", i, e));
        if i == 1 {
            level1_id = child.id.clone();
        }
        current_id = child.id;
    }

    assert_eq!(service.len(), 51);

    // Verify access at level 50
    let test_scope = CapabilityScope::Filesystem {
        path: "/data/file.txt".into(),
        recursive: false,
    };
    assert!(service.has_active_capability("agent:level_50", &test_scope, CapabilityRight::Read));

    // Cascade revoke level 1 -> must revoke all 50 descendants
    let revoked = service.revoke_capability(&level1_id).expect("revoke level 1");
    assert_eq!(revoked.len(), 50);

    // Level 50 must now be denied
    assert!(!service.has_active_capability("agent:level_50", &test_scope, CapabilityRight::Read));

    // Root must remain active
    assert!(service.has_active_capability("agent:root", &test_scope, CapabilityRight::Read));
}

