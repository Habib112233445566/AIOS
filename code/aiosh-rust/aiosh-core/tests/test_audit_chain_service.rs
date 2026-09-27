//! Unit tests for AuditChainService (T-02315).

use std::collections::HashMap;
use serde_json::json;

use aiosh_core::audit::{AuditRing, AuditRowInput, ExtendedAuditRowInput};
use aiosh_core::audit_chain_ext::{AuditCausalLink, AuditProvenance, AuditSignature};
use aiosh_core::audit_chain_service::{AuditChainService, AuditQueryFilter};

#[test]
fn test_service_record_and_get_by_hash() {
    let ring = AuditRing::open_in_memory().expect("open memory ring");
    let mut service = AuditChainService::new(ring);

    let mut base = AuditRowInput::default();
    base.actor = "agent_007".into();
    base.tool = "aios.pep.eval".into();
    base.command = "eval".into();

    let mut input = ExtendedAuditRowInput::new(base);
    input.provenance = Some(AuditProvenance {
        session_id: Some("session-alpha".into()),
        pep_grant_id: Some("grant-root-1".into()),
        delegation_depth: 0,
        trace_id: Some("trace-001".into()),
        span_id: Some("span-001".into()),
    });
    let mut ext = HashMap::new();
    ext.insert("metric_score".into(), json!(98.5));
    input.extensions = ext;

    let row = service.record_event(input).expect("record event");
    assert!(!row.hash.is_empty());

    let fetched = service.get_row_by_hash(&row.hash).expect("get by hash");
    assert!(fetched.is_some());
    let fetched_row = fetched.unwrap();
    assert_eq!(fetched_row.actor, "agent_007");
    assert_eq!(fetched_row.provenance.as_ref().unwrap().session_id.as_deref(), Some("session-alpha"));
    assert_eq!(fetched_row.extensions.get("metric_score"), Some(&json!(98.5)));
}

#[test]
fn test_service_query_filtering() {
    let ring = AuditRing::open_in_memory().expect("open memory ring");
    let mut service = AuditChainService::new(ring);

    // Event 1: session-A, trace-1
    let mut base1 = AuditRowInput::default();
    base1.actor = "worker_1".into();
    base1.tool = "aios.fs.read".into();
    let mut input1 = ExtendedAuditRowInput::new(base1);
    input1.provenance = Some(AuditProvenance {
        session_id: Some("session-A".into()),
        pep_grant_id: None,
        delegation_depth: 0,
        trace_id: Some("trace-1".into()),
        span_id: None,
    });
    service.record_event(input1).expect("record 1");

    // Event 2: session-B, trace-2
    let mut base2 = AuditRowInput::default();
    base2.actor = "worker_2".into();
    base2.tool = "aios.fs.write".into();
    let mut input2 = ExtendedAuditRowInput::new(base2);
    input2.provenance = Some(AuditProvenance {
        session_id: Some("session-B".into()),
        pep_grant_id: None,
        delegation_depth: 0,
        trace_id: Some("trace-2".into()),
        span_id: None,
    });
    service.record_event(input2).expect("record 2");

    // Query session-A
    let filter_a = AuditQueryFilter {
        session_id: Some("session-A".into()),
        ..Default::default()
    };
    let res_a = service.query_events(&filter_a).expect("query session A");
    assert_eq!(res_a.len(), 1);
    assert_eq!(res_a[0].actor, "worker_1");

    // Query tool aios.fs.write
    let filter_tool = AuditQueryFilter {
        tool: Some("aios.fs.write".into()),
        ..Default::default()
    };
    let res_tool = service.query_events(&filter_tool).expect("query tool");
    assert_eq!(res_tool.len(), 1);
    assert_eq!(res_tool[0].actor, "worker_2");
}

#[test]
fn test_service_trace_ancestry_dag() {
    let ring = AuditRing::open_in_memory().expect("open memory ring");
    let mut service = AuditChainService::new(ring);

    // 1. Root event
    let mut root_base = AuditRowInput::default();
    root_base.actor = "orchestrator".into();
    root_base.tool = "aios.workflow.start".into();
    let root_row = service.record_event(ExtendedAuditRowInput::new(root_base)).expect("root");

    // 2. Child event (links to root)
    let mut child_base = AuditRowInput::default();
    child_base.actor = "planner".into();
    child_base.tool = "aios.plan.create".into();
    let mut child_input = ExtendedAuditRowInput::new(child_base);
    child_input.causal_links = vec![AuditCausalLink::new(root_row.hash.clone(), "delegation")];
    let child_row = service.record_event(child_input).expect("child");

    // 3. Grandchild event (links to child)
    let mut gc_base = AuditRowInput::default();
    gc_base.actor = "executor".into();
    gc_base.tool = "aios.task.execute".into();
    let mut gc_input = ExtendedAuditRowInput::new(gc_base);
    gc_input.causal_links = vec![AuditCausalLink::new(child_row.hash.clone(), "subtask")];
    let gc_row = service.record_event(gc_input).expect("grandchild");

    // Trace ancestry from grandchild
    let lineage = service.trace_ancestry(&gc_row.hash, 10).expect("trace lineage");
    assert_eq!(lineage.target_hash, gc_row.hash);
    assert_eq!(lineage.ancestors.len(), 2);
    assert_eq!(lineage.ancestors[0].row.hash, child_row.hash);
    assert_eq!(lineage.ancestors[0].relationship, "subtask");
    assert_eq!(lineage.ancestors[1].row.hash, root_row.hash);
    assert_eq!(lineage.ancestors[1].relationship, "delegation");
}

#[test]
fn test_service_signature_verification() {
    let ring = AuditRing::open_in_memory().expect("open memory ring");
    let mut service = AuditChainService::new(ring);

    let mut base = AuditRowInput::default();
    base.actor = "signer".into();
    base.tool = "aios.auth.sign".into();
    let mut input = ExtendedAuditRowInput::new(base);
    input.signature = Some(AuditSignature::new(
        "ed25519",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899",
    ));

    let row = service.record_event(input).expect("record signed");
    let rep = service.verify_event_signature(&row.hash).expect("verify sig");
    assert!(rep.has_signature);
    assert!(rep.is_valid);
    assert_eq!(rep.algorithm.as_deref(), Some("ed25519"));
}

#[test]
fn test_service_record_bounds_enforcement() {
    let ring = AuditRing::open_in_memory().expect("open memory ring");
    let mut service = AuditChainService::new(ring);

    let mut base = AuditRowInput::default();
    base.actor = "attacker".into();
    base.tool = "aios.exploit".into();
    let mut input = ExtendedAuditRowInput::new(base);
    // Invalid session ID with whitespace
    input.provenance = Some(AuditProvenance {
        session_id: Some("invalid session id with spaces".into()),
        pep_grant_id: None,
        delegation_depth: 0,
        trace_id: None,
        span_id: None,
    });

    let err = service.record_event(input).unwrap_err();
    assert!(err.contains("AUDIT_EXT_ERR_VALIDATION"));
}

#[test]
fn test_service_file_backed_persistence_integration() {
    use aiosh_core::audit::OpenOptions;

    let dir = std::env::temp_dir().join(format!("aios_audit_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = std::fs::create_dir_all(&dir);
    let db_path = dir.join("audit.db").to_str().unwrap().to_string();

    let root_hash;
    let child_hash;

    // Phase 1: Open and write
    {
        let ring = AuditRing::open(OpenOptions {
            path: Some(db_path.clone()),
            home: None,
        }).expect("open file ring");
        let mut service = AuditChainService::new(ring);

        let mut root_base = AuditRowInput::default();
        root_base.actor = "system_init".into();
        root_base.tool = "aios.boot".into();
        let r_row = service.record_event(ExtendedAuditRowInput::new(root_base)).expect("record root");
        root_hash = r_row.hash;

        let mut child_base = AuditRowInput::default();
        child_base.actor = "agent_ext".into();
        child_base.tool = "aios.pep.evaluate".into();
        let mut child_input = ExtendedAuditRowInput::new(child_base);
        child_input.provenance = Some(AuditProvenance {
            session_id: Some("session-disk-persisted".into()),
            pep_grant_id: Some("grant-p1".into()),
            delegation_depth: 1,
            trace_id: Some("trace-persisted-99".into()),
            span_id: None,
        });
        child_input.causal_links = vec![AuditCausalLink::new(root_hash.clone(), "trigger")];
        let c_row = service.record_event(child_input).expect("record child");
        child_hash = c_row.hash;

        let verify_res = service.verify_integrity().expect("verify 1");
        assert!(verify_res.ok);
        assert_eq!(verify_res.checked, 2);
    }

    // Phase 2: Re-open existing database and verify persistence & DAG traversal across restarts
    {
        let ring = AuditRing::open(OpenOptions {
            path: Some(db_path.clone()),
            home: None,
        }).expect("reopen file ring");
        let service = AuditChainService::new(ring);

        let fetched = service.get_row_by_hash(&child_hash).expect("get child").expect("found");
        assert_eq!(fetched.provenance.as_ref().unwrap().session_id.as_deref(), Some("session-disk-persisted"));
        assert_eq!(fetched.causal_links.len(), 1);
        assert_eq!(fetched.causal_links[0].parent_event_hash, root_hash);

        let lineage = service.trace_ancestry(&child_hash, 5).expect("trace lineage");
        assert_eq!(lineage.ancestors.len(), 1);
        assert_eq!(lineage.ancestors[0].row.hash, root_hash);
        assert_eq!(lineage.ancestors[0].relationship, "trigger");

        let verify_res = service.verify_integrity().expect("verify 2");
        assert!(verify_res.ok);
        assert_eq!(verify_res.checked, 2);
    }

    let _ = std::fs::remove_dir_all(&dir);
}
