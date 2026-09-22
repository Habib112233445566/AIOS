//! Automated end-to-end integration tests for PEP Grant Lifecycle (Sub-Epic 6, AUTOGRANT1..AUTOGRANT8).

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;
use chrono::{Duration, Utc};
use tempfile::{tempdir, TempDir};

use aiosh_core::capability::{CapabilityRight, CapabilityScope};
use aiosh_core::pep_grant::{
    PepGrant, PepGrantConstraints, PepGrantState, PepGrantStore,
    PEPGRANT_ERR_INVALID_ID, PEPGRANT_ERR_VALIDATION,
};
use aiosh_core::pep_grant_service::{
    PepGrantService, GSVC_ERR_ATTENUATION, GSVC_ERR_INVALID_TRANSITION,
    GSVC_ERR_NOT_FOUND, GSVC_ERR_VALIDATION,
};

/// Hermetic environment fixture for automated PEP grant testing.
pub struct MockPepGrantEnv {
    pub dir: TempDir,
    pub store_path: PathBuf,
}

impl MockPepGrantEnv {
    pub fn new() -> Self {
        let dir = tempdir().expect("create temp dir");
        let store_path = dir.path().join("pep_grants_automated.json");
        Self { dir, store_path }
    }

    pub fn populate_standard_fixtures(&self) -> Result<PepGrantService, String> {
        let mut service = PepGrantService::new().with_storage_path(self.store_path.clone());

        // 1. Root Admin Grant
        let mut root = PepGrant::new(
            "grant-root",
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
        );
        root.constraints.max_delegation_depth = 8;
        service.issue_grant(root)?;
        service.transition_grant("grant-root", PepGrantState::Active)?;

        // 2. Scoped Filesystem Worker Grant
        let mut worker = PepGrant::new(
            "grant-worker",
            "kernel",
            "agent:worker",
            CapabilityScope::Filesystem {
                path: "/var/data".into(),
                recursive: true,
            },
            vec![
                CapabilityRight::Read,
                CapabilityRight::Write,
                CapabilityRight::Delegate,
            ],
        );
        worker.constraints = PepGrantConstraints {
            not_before: None,
            expires_at: Some((Utc::now() + Duration::hours(1)).to_rfc3339()),
            max_invocations: Some(100),
            invocations_used: 0,
            max_bytes: Some(1024 * 1024),
            bytes_used: 0,
            max_delegation_depth: 4,
        };
        service.issue_grant(worker)?;
        service.transition_grant("grant-worker", PepGrantState::Active)?;

        // 3. Pre-Expired Grant
        let mut expired = PepGrant::new(
            "grant-expired",
            "kernel",
            "agent:expired",
            CapabilityScope::Network {
                host: "api.internal".into(),
                port: Some(443),
                protocol: "tcp".into(),
            },
            vec![CapabilityRight::Read],
        );
        expired.constraints = PepGrantConstraints {
            not_before: None,
            expires_at: Some((Utc::now() - Duration::hours(2)).to_rfc3339()),
            max_invocations: None,
            invocations_used: 0,
            max_bytes: None,
            bytes_used: 0,
            max_delegation_depth: 0,
        };
        service.issue_grant(expired)?;
        service.transition_grant("grant-expired", PepGrantState::Active)?;

        Ok(service)
    }
}

#[test]
fn test_automated_mock_env_initialization() {
    let mock = MockPepGrantEnv::new();
    let service = mock.populate_standard_fixtures().expect("populate fixtures");
    assert_eq!(service.len(), 3);
    assert_eq!(service.list_grants_for_subject("agent:admin").len(), 1);
    assert_eq!(service.list_grants_for_subject("agent:worker").len(), 1);
    assert_eq!(service.list_grants_for_subject("agent:expired").len(), 1);
}

#[test]
fn test_automated_grant_scale_and_indexing() {
    let mock = MockPepGrantEnv::new();
    let mut service = PepGrantService::new().with_storage_path(mock.store_path.clone());

    // Scale test: Issue 1,000 synthetic grants across 50 subjects
    for i in 0..1000 {
        let subject = format!("agent:worker_{}", i % 50);
        let grant = PepGrant::new(
            format!("g-scale-{:04}", i),
            "kernel",
            &subject,
            CapabilityScope::System {
                subsystem: format!("subsystem_{}", i % 10),
            },
            vec![CapabilityRight::Read],
        );
        service.issue_grant(grant).expect("issue grant");
    }

    assert_eq!(service.len(), 1000);

    // Verify index coherence for subject
    for s_idx in 0..50 {
        let subject = format!("agent:worker_{}", s_idx);
        let grants = service.list_grants_for_subject(&subject);
        assert_eq!(grants.len(), 20, "subject {} expected 20 grants", subject);
    }

    // Verify index coherence for state
    let requested = service.list_grants_by_state(PepGrantState::Requested);
    assert_eq!(requested.len(), 1000);

    // Fast point-lookup
    assert!(service.get_grant("g-scale-0000").is_some());
    assert!(service.get_grant("g-scale-0999").is_some());
    assert!(service.get_grant("g-scale-1000").is_none());
}

#[test]
fn test_automated_grant_attenuation_depth_and_invariants() {
    let mock = MockPepGrantEnv::new();
    let mut service = PepGrantService::new().with_storage_path(mock.store_path.clone());

    // Root grant with max_delegation_depth = 7
    let mut root = PepGrant::new(
        "g-hier-root",
        "kernel",
        "agent:tier0",
        CapabilityScope::Filesystem {
            path: "/var".into(),
            recursive: true,
        },
        vec![
            CapabilityRight::Read,
            CapabilityRight::Write,
            CapabilityRight::Execute,
            CapabilityRight::Delete,
            CapabilityRight::Admin,
            CapabilityRight::Delegate,
        ],
    );
    root.constraints.max_delegation_depth = 7;
    service.issue_grant(root).expect("issue root");
    service
        .transition_grant("g-hier-root", PepGrantState::Active)
        .expect("activate root");

    // Attenuate down 7 tiers: T1 to T7
    let mut current_parent_id = "g-hier-root".to_string();
    for depth in 1..=7 {
        let child_id = format!("g-hier-tier{}", depth);
        let subject = format!("agent:tier{}", depth);

        // Tiers 1..6 retain Delegate to permit subsequent attenuation; Tier 7 is leaf
        let rights = if depth == 7 {
            vec![CapabilityRight::Read]
        } else {
            vec![CapabilityRight::Read, CapabilityRight::Delegate]
        };

        service
            .attenuate_grant(&current_parent_id, &child_id, &subject, rights)
            .expect(&format!("attenuate tier {}", depth));

        current_parent_id = child_id;
    }

    assert_eq!(service.len(), 8);

    // Attempting depth 8 must fail because delegation limit reached on tier 7 (no Delegate right and max_delegation_depth == 0)
    let err_depth = service.attenuate_grant(
        &current_parent_id,
        "g-hier-tier8",
        "agent:tier8",
        vec![CapabilityRight::Read],
    );
    assert!(err_depth.is_err());
    assert!(err_depth.unwrap_err().contains(GSVC_ERR_ATTENUATION));

    // Attenuation invariant check: Right escalation attempt (tier 1 requesting Admin from root)
    let err_esc = service.attenuate_grant(
        "g-hier-tier1",
        "g-hier-escalate",
        "agent:evil",
        vec![CapabilityRight::Admin],
    );
    assert!(err_esc.is_err());
    assert!(err_esc.unwrap_err().contains(GSVC_ERR_ATTENUATION));
}

#[test]
fn test_automated_grant_cascade_revocation_branching() {
    let mock = MockPepGrantEnv::new();
    let mut service = PepGrantService::new().with_storage_path(mock.store_path.clone());

    // Root
    let mut root = PepGrant::new(
        "g-tree-root",
        "kernel",
        "agent:root",
        CapabilityScope::System {
            subsystem: "root".into(),
        },
        vec![CapabilityRight::Read, CapabilityRight::Delegate],
    );
    root.constraints.max_delegation_depth = 4;
    service.issue_grant(root).expect("issue root");
    service.transition_grant("g-tree-root", PepGrantState::Active).unwrap();

    // Branch A: root -> A1 -> (A2a, A2b)
    service
        .attenuate_grant(
            "g-tree-root",
            "g-tree-a1",
            "agent:a1",
            vec![CapabilityRight::Read, CapabilityRight::Delegate],
        )
        .unwrap();

    service
        .attenuate_grant(
            "g-tree-a1",
            "g-tree-a2a",
            "agent:a2a",
            vec![CapabilityRight::Read],
        )
        .unwrap();

    service
        .attenuate_grant(
            "g-tree-a1",
            "g-tree-a2b",
            "agent:a2b",
            vec![CapabilityRight::Read],
        )
        .unwrap();

    // Branch B: root -> B1 -> (B2a, B2b)
    service
        .attenuate_grant(
            "g-tree-root",
            "g-tree-b1",
            "agent:b1",
            vec![CapabilityRight::Read, CapabilityRight::Delegate],
        )
        .unwrap();

    service
        .attenuate_grant(
            "g-tree-b1",
            "g-tree-b2a",
            "agent:b2a",
            vec![CapabilityRight::Read],
        )
        .unwrap();

    service
        .attenuate_grant(
            "g-tree-b1",
            "g-tree-b2b",
            "agent:b2b",
            vec![CapabilityRight::Read],
        )
        .unwrap();

    assert_eq!(service.len(), 7);
    assert_eq!(service.list_grants_by_state(PepGrantState::Active).len(), 7);

    // Revoke Branch A1 with cascade = true
    let revoked_count = service
        .revoke_grant("g-tree-a1", "admin", "security cascade test", true)
        .expect("revoke a1");
    assert_eq!(revoked_count, 3);

    // Verify states
    assert_eq!(service.get_grant("g-tree-a1").unwrap().state, PepGrantState::Revoked);
    assert_eq!(service.get_grant("g-tree-a2a").unwrap().state, PepGrantState::Revoked);
    assert_eq!(service.get_grant("g-tree-a2b").unwrap().state, PepGrantState::Revoked);

    // Verify unlinked branch B and root are still Active
    assert_eq!(service.get_grant("g-tree-root").unwrap().state, PepGrantState::Active);
    assert_eq!(service.get_grant("g-tree-b1").unwrap().state, PepGrantState::Active);
    assert_eq!(service.get_grant("g-tree-b2a").unwrap().state, PepGrantState::Active);
    assert_eq!(service.get_grant("g-tree-b2b").unwrap().state, PepGrantState::Active);

    let active = service.list_grants_by_state(PepGrantState::Active);
    assert_eq!(active.len(), 4);
    let revoked_list = service.list_grants_by_state(PepGrantState::Revoked);
    assert_eq!(revoked_list.len(), 3);
}

#[test]
fn test_automated_grant_mass_expiration_sweep() {
    let mock = MockPepGrantEnv::new();
    let mut service = PepGrantService::new().with_storage_path(mock.store_path.clone());

    // 100 Expired Grants
    for i in 0..100 {
        let mut g = PepGrant::new(
            format!("g-exp-{:03}", i),
            "kernel",
            format!("agent:exp-{}", i),
            CapabilityScope::System {
                subsystem: "temporal".into(),
            },
            vec![CapabilityRight::Read],
        );
        g.constraints = PepGrantConstraints {
            not_before: None,
            expires_at: Some((Utc::now() - Duration::hours(1 + (i as i64) % 10)).to_rfc3339()),
            max_invocations: None,
            invocations_used: 0,
            max_bytes: None,
            bytes_used: 0,
            max_delegation_depth: 0,
        };
        service.issue_grant(g).unwrap();
        service.transition_grant(&format!("g-exp-{:03}", i), PepGrantState::Active).unwrap();
    }

    // 100 Active Grants
    for i in 0..100 {
        let mut g = PepGrant::new(
            format!("g-act-{:03}", i),
            "kernel",
            format!("agent:act-{}", i),
            CapabilityScope::System {
                subsystem: "temporal".into(),
            },
            vec![CapabilityRight::Read],
        );
        g.constraints = PepGrantConstraints {
            not_before: None,
            expires_at: Some((Utc::now() + Duration::hours(1 + (i as i64) % 10)).to_rfc3339()),
            max_invocations: None,
            invocations_used: 0,
            max_bytes: None,
            bytes_used: 0,
            max_delegation_depth: 0,
        };
        service.issue_grant(g).unwrap();
        service.transition_grant(&format!("g-act-{:03}", i), PepGrantState::Active).unwrap();
    }

    assert_eq!(service.len(), 200);
    assert_eq!(service.list_grants_by_state(PepGrantState::Active).len(), 200);

    // Perform mass sweep
    let now_iso = Utc::now().to_rfc3339();
    let swept = service.sweep_expired(&now_iso).expect("sweep expired grants");
    assert_eq!(swept, 100);

    // Verify counts and states
    let active = service.list_grants_by_state(PepGrantState::Active);
    assert_eq!(active.len(), 100);

    let expired = service.list_grants_by_state(PepGrantState::Expired);
    assert_eq!(expired.len(), 100);

    // Second sweep must be idempotent
    let swept_again = service.sweep_expired(&now_iso).expect("idempotent sweep");
    assert_eq!(swept_again, 0);
}

#[test]
fn test_automated_grant_persistence_and_reload_integrity() {
    let mock = MockPepGrantEnv::new();
    {
        let mut service = PepGrantService::new().with_storage_path(mock.store_path.clone());

        for i in 0..50 {
            let mut g = PepGrant::new(
                format!("g-persist-{:02}", i),
                "kernel",
                format!("agent:p-{}", i % 5),
                CapabilityScope::System {
                    subsystem: "io".into(),
                },
                vec![CapabilityRight::Read, CapabilityRight::Write],
            );
            g.constraints.max_bytes = Some(1024 * (i as u64 + 1));
            service.issue_grant(g).unwrap();

            if i % 2 == 0 {
                service
                    .transition_grant(&format!("g-persist-{:02}", i), PepGrantState::Active)
                    .unwrap();
            }
        }

        // Revoke a subset: index 10 was activated by the even-index branch
        service
            .revoke_grant("g-persist-10", "admin", "test persistence revoke", false)
            .unwrap();

        // Save to disk explicitly
        service.save_to_path(&mock.store_path).expect("save to path");
    }

    // Verify file exists
    assert!(mock.store_path.exists());

    // Reload from disk
    let loaded = PepGrantService::load_from_path(&mock.store_path).expect("reload from disk");
    assert_eq!(loaded.len(), 50);

    // Verify revoked state persisted
    assert_eq!(
        loaded.get_grant("g-persist-10").unwrap().state,
        PepGrantState::Revoked
    );

    // Verify active grants count
    let active = loaded.list_grants_by_state(PepGrantState::Active);
    assert_eq!(active.len(), 24); // 25 even indices minus index 10

    // Verify subject index reconstructed
    let p0_grants = loaded.list_grants_for_subject("agent:p-0");
    assert_eq!(p0_grants.len(), 10);
}

#[test]
fn test_automated_grant_security_boundaries_and_fuzzing() {
    let mock = MockPepGrantEnv::new();
    let mut service = PepGrantService::new().with_storage_path(mock.store_path.clone());

    // 1. Invalid identifier with control characters or whitespace
    let invalid_id_grant = PepGrant::new(
        "bad\x00grant id",
        "kernel",
        "agent:test",
        CapabilityScope::System {
            subsystem: "test".into(),
        },
        vec![CapabilityRight::Read],
    );
    assert!(invalid_id_grant.validate().unwrap_err().contains(PEPGRANT_ERR_INVALID_ID));
    assert!(service.issue_grant(invalid_id_grant).is_err());

    // 2. Oversized identifier (> 128 chars)
    let long_id = "a".repeat(129);
    let long_id_grant = PepGrant::new(
        &long_id,
        "kernel",
        "agent:test",
        CapabilityScope::System {
            subsystem: "test".into(),
        },
        vec![CapabilityRight::Read],
    );
    assert!(long_id_grant.validate().unwrap_err().contains(PEPGRANT_ERR_INVALID_ID));
    assert!(service.issue_grant(long_id_grant).is_err());

    // 3. Subject with illegal path traversal / characters
    let bad_subject_grant = PepGrant::new(
        "g-bad-subj",
        "kernel",
        "agent/../../root",
        CapabilityScope::System {
            subsystem: "test".into(),
        },
        vec![CapabilityRight::Read],
    );
    assert!(bad_subject_grant.validate().unwrap_err().contains(PEPGRANT_ERR_VALIDATION));

    // 4. Excessive max_delegation_depth (> 8)
    let mut deep_grant = PepGrant::new(
        "g-deep-test",
        "kernel",
        "agent:test",
        CapabilityScope::System {
            subsystem: "test".into(),
        },
        vec![CapabilityRight::Read],
    );
    deep_grant.constraints.max_delegation_depth = 50;
    assert!(deep_grant.validate().unwrap_err().contains(PEPGRANT_ERR_VALIDATION));

    // 5. Empty rights
    let empty_rights_grant = PepGrant::new(
        "g-empty-rights",
        "kernel",
        "agent:test",
        CapabilityScope::System {
            subsystem: "test".into(),
        },
        vec![],
    );
    assert!(empty_rights_grant.validate().unwrap_err().contains(PEPGRANT_ERR_VALIDATION));

    // 6. Invalid state transition: Requested -> Expired directly
    let g_req = PepGrant::new(
        "g-req-test",
        "kernel",
        "agent:test",
        CapabilityScope::System {
            subsystem: "test".into(),
        },
        vec![CapabilityRight::Read],
    );
    service.issue_grant(g_req).unwrap();
    let err_trans = service.transition_grant("g-req-test", PepGrantState::Expired);
    assert!(err_trans.is_err());
    assert!(err_trans.unwrap_err().contains(GSVC_ERR_INVALID_TRANSITION));

    // 7. Revoking non-existent grant
    let err_nonexist = service.revoke_grant("g-does-not-exist", "admin", "reason", false);
    assert!(err_nonexist.is_err());
    assert!(err_nonexist.unwrap_err().contains(GSVC_ERR_NOT_FOUND));

    // 8. Path validation in load_from_path with path traversal
    let bad_path = mock.dir.path().join("../traversal.json");
    let err_path = PepGrantService::load_from_path(&bad_path);
    assert!(err_path.is_err());
    assert!(err_path.unwrap_err().contains(GSVC_ERR_VALIDATION));
}

#[test]
fn test_automated_grant_concurrent_eval_and_mutation() {
    let mock = MockPepGrantEnv::new();
    let service = Arc::new(RwLock::new(
        mock.populate_standard_fixtures().expect("populate fixtures"),
    ));

    let mut handles = Vec::new();

    // Spawn 4 reader threads
    for thread_idx in 0..4 {
        let s = Arc::clone(&service);
        handles.push(thread::spawn(move || {
            for _ in 0..50 {
                let guard = s.read().unwrap();
                let grants = guard.list_grants_for_subject("agent:admin");
                assert!(!grants.is_empty(), "thread {} should find admin grant", thread_idx);
                let _active = guard.list_grants_by_state(PepGrantState::Active);
            }
        }));
    }

    // Spawn 2 writer threads
    for writer_idx in 0..2 {
        let s = Arc::clone(&service);
        handles.push(thread::spawn(move || {
            for iter in 0..25 {
                let grant_id = format!("g-thread-{}-{}", writer_idx, iter);
                let g = PepGrant::new(
                    grant_id,
                    "kernel",
                    format!("agent:worker_{}", writer_idx),
                    CapabilityScope::System {
                        subsystem: "thread".into(),
                    },
                    vec![CapabilityRight::Read],
                );
                let mut guard = s.write().unwrap();
                guard.issue_grant(g).unwrap();
            }
        }));
    }

    for h in handles {
        h.join().expect("thread joined successfully");
    }

    let final_guard = service.read().unwrap();
    // 3 initial + 2 * 25 added = 53
    assert_eq!(final_guard.len(), 53);
}

#[test]
fn test_automated_grant_cli_mcp_cross_substrate() {
    let mock = MockPepGrantEnv::new();
    let service = mock.populate_standard_fixtures().expect("populate fixtures");

    // Serialize service grants via canonical/store representation
    let store_json = serde_json::to_string(&service).expect("serialize service");
    let parsed: serde_json::Value = serde_json::from_str(&store_json).expect("parse json");

    // Assert JSON representation has expected structure
    let grants_obj = parsed.get("grants").and_then(|v| v.as_object()).expect("grants map");
    assert!(grants_obj.contains_key("grant-root"));
    assert!(grants_obj.contains_key("grant-worker"));
    assert!(grants_obj.contains_key("grant-expired"));

    // Verify fields in grant-root
    let root_val = &grants_obj["grant-root"];
    assert_eq!(root_val["id"], "grant-root");
    assert_eq!(root_val["issuer"], "kernel");
    assert_eq!(root_val["subject"], "agent:admin");
    assert_eq!(root_val["state"], "active");
    assert_eq!(root_val["constraints"]["max_delegation_depth"], 8);

    // Verify deserialization into PepGrantStore
    let mut store = PepGrantStore::new();
    let root_grant = service.get_grant("grant-root").unwrap().clone();
    let worker_grant = service.get_grant("grant-worker").unwrap().clone();
    store.add_grant(root_grant).expect("add root grant");
    store.add_grant(worker_grant).expect("add worker grant");

    let store_path = mock.dir.path().join("store_test.json");
    store.save_to_path(&store_path).expect("save PepGrantStore");

    let loaded_store = PepGrantStore::load_from_path(&store_path).expect("load PepGrantStore");
    assert_eq!(loaded_store.len(), 2);
    assert!(loaded_store.get_grant("grant-root").is_some());
    assert!(loaded_store.get_grant("grant-worker").is_some());
}
