//! Tests for Audit Chain Extensions Data Model (T-02304).

use std::collections::HashMap;
use serde_json::json;

use aiosh_core::audit_chain_ext::{
    AuditCausalLink, AuditProvenance, AuditSignature, ExtendedAuditRow,
    AUDIT_EXT_ERR_BOUNDS, AUDIT_EXT_ERR_HASH,
};
use aiosh_core::canonical::{canonical, sha256_hex};
use aiosh_core::types::{AuditRow, CFlags, GENESIS_HASH};

fn make_sample_legacy_row() -> AuditRow {
    let mut row = AuditRow {
        id: 1,
        ts: "2026-09-26T12:00:00Z".to_string(),
        actor: "agent".to_string(),
        actor_id: "agent:test@host".to_string(),
        tool: "aios.pep.grant.inspect".to_string(),
        command: "inspect".to_string(),
        args: json!({"grant_id": "grant-001"}),
        target: Some("grant-001".to_string()),
        outcome: "ok".to_string(),
        outcome_detail: None,
        constitution_rev: Some("rev-1".to_string()),
        grant_token: Some("token-abc".to_string()),
        c_flags: CFlags::default(),
        policy_revision: None,
        classify_rule_ids: None,
        classify_evidence: None,
        classify_overall_verdict: None,
        classify_verdict_reason: None,
        prev_hash: GENESIS_HASH.to_string(),
        hash: String::new(),
    };
    let payload = format!("{}{}", row.prev_hash, canonical(&row.hash_proto()));
    row.hash = sha256_hex(&payload);
    row
}

#[test]
fn test_extended_audit_row_legacy_compatibility() {
    let legacy = make_sample_legacy_row();
    let extended = ExtendedAuditRow::from_legacy_row(legacy.clone());

    // Invariant: hash_proto of an unextended row matches legacy hash_proto
    assert_eq!(extended.hash_proto(), legacy.hash_proto());
    assert_eq!(extended.compute_hash(), legacy.hash);
    assert!(extended.validate().is_ok());

    // Roundtrip back to legacy
    let back_to_legacy = extended.to_legacy_row();
    assert_eq!(back_to_legacy, legacy);
}

#[test]
fn test_extended_audit_row_with_provenance_and_causality() {
    let mut row = ExtendedAuditRow::from_legacy_row(make_sample_legacy_row());
    row.provenance = Some(AuditProvenance {
        session_id: Some("session-xyz-123".to_string()),
        pep_grant_id: Some("grant-root-999".to_string()),
        delegation_depth: 1,
        trace_id: Some("trace-abcdef0123456789".to_string()),
        span_id: Some("span-0123456789abcdef".to_string()),
    });
    row.causal_links = vec![AuditCausalLink::new(
        "1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff",
        "delegation",
    )];
    row.signature = Some(AuditSignature::new(
        "ed25519",
        "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899",
        "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    ));
    let mut ext_map = HashMap::new();
    ext_map.insert("telemetry_runtime_ms".to_string(), json!(42));
    row.extensions = ext_map;

    // Compute and assign hash
    row.hash = row.compute_hash();

    assert!(row.validate().is_ok());
    assert!(row.verify_hash().is_ok());
}

#[test]
fn test_extended_audit_row_tamper_detection() {
    let mut row = ExtendedAuditRow::from_legacy_row(make_sample_legacy_row());
    row.hash = row.compute_hash();
    assert!(row.verify_hash().is_ok());

    // Tamper with command
    row.command = "malicious_command".to_string();
    let err = row.verify_hash().unwrap_err();
    assert!(err.contains(AUDIT_EXT_ERR_HASH));
}

#[test]
fn test_extended_audit_row_bounds_enforcement() {
    let mut row = ExtendedAuditRow::from_legacy_row(make_sample_legacy_row());

    // Exceed max causal links (max 16)
    row.causal_links = (0..20)
        .map(|_| {
            AuditCausalLink::new(
                "0000111122223333444455556666777788889999aaaabbbbccccddddeeeeffff",
                "subtask",
            )
        })
        .collect();

    row.hash = row.compute_hash();
    let err = row.validate().unwrap_err();
    assert!(err.contains(AUDIT_EXT_ERR_BOUNDS));
}

#[test]
fn test_extended_audit_row_json_serde_roundtrip() {
    let mut row = ExtendedAuditRow::from_legacy_row(make_sample_legacy_row());
    row.provenance = Some(AuditProvenance {
        session_id: Some("session-roundtrip-42".to_string()),
        pep_grant_id: Some("grant-roundtrip-99".to_string()),
        delegation_depth: 2,
        trace_id: None,
        span_id: None,
    });
    row.hash = row.compute_hash();

    let json_str = serde_json::to_string(&row).unwrap();
    let deserialized: ExtendedAuditRow = serde_json::from_str(&json_str).unwrap();
    assert_eq!(row, deserialized);
    assert!(deserialized.verify_hash().is_ok());
}

#[test]
fn test_extended_audit_row_invalid_provenance_and_signatures() {
    let mut row = ExtendedAuditRow::from_legacy_row(make_sample_legacy_row());
    row.provenance = Some(AuditProvenance {
        session_id: Some("bad session with spaces".to_string()),
        pep_grant_id: None,
        delegation_depth: 0,
        trace_id: None,
        span_id: None,
    });
    row.hash = row.compute_hash();
    assert!(row.validate().is_err());

    let bad_sig = AuditSignature::new("", "pubkey", "sig");
    assert!(bad_sig.validate().is_err());
}

#[test]
fn test_audit_ring_extended_integration() {
    use aiosh_core::audit::{AuditRing, AuditRowInput, ExtendedAuditRowInput};

    let mut ring = AuditRing::open_in_memory().expect("open_in_memory failed");

    // 1. Write an unextended row using write()
    let mut base_input = AuditRowInput::default();
    base_input.actor = "agent_init".to_string();
    base_input.tool = "aios.system.boot".to_string();
    base_input.command = "boot".to_string();
    let row1 = ring.write(base_input).expect("write failed");

    // 2. Write an extended row using write_extended()
    let mut ext_base = AuditRowInput::default();
    ext_base.actor = "agent_worker".to_string();
    ext_base.tool = "aios.pep.grant.issue".to_string();
    ext_base.command = "issue".to_string();

    let mut ext_input = ExtendedAuditRowInput::new(ext_base);
    ext_input.provenance = Some(AuditProvenance {
        session_id: Some("sess-int-1".to_string()),
        pep_grant_id: Some("grant-int-root".to_string()),
        delegation_depth: 1,
        trace_id: Some("trace-int-1".to_string()),
        span_id: Some("span-int-1".to_string()),
    });
    ext_input.causal_links = vec![AuditCausalLink::new(
        row1.hash.clone(),
        "delegation",
    )];
    ext_input.signature = Some(AuditSignature::new(
        "ed25519",
        "pubkey-test-hex",
        "sig-test-hex",
    ));
    let mut exts = HashMap::new();
    exts.insert("custom_key".to_string(), json!("custom_val"));
    ext_input.extensions = exts;

    let row2 = ring.write_extended(ext_input).expect("write_extended failed");

    assert_eq!(row2.prev_hash, row1.hash);
    assert!(row2.validate().is_ok());

    // 3. Tail extended
    let tailed_ext = ring.tail_extended(10).expect("tail_extended failed");
    assert_eq!(tailed_ext.len(), 2);
    // row 1 has empty extensions
    assert!(tailed_ext[0].provenance.is_none());
    assert!(tailed_ext[0].causal_links.is_empty());
    // row 2 has extensions preserved
    assert_eq!(tailed_ext[1].provenance.as_ref().unwrap().session_id.as_deref(), Some("sess-int-1"));
    assert_eq!(tailed_ext[1].causal_links.len(), 1);
    assert_eq!(tailed_ext[1].causal_links[0].parent_event_hash, row1.hash);
    assert_eq!(tailed_ext[1].signature.as_ref().unwrap().algorithm, "ed25519");
    assert_eq!(tailed_ext[1].extensions.get("custom_key"), Some(&json!("custom_val")));

    // 4. Verify ring integrity across both unextended and extended rows
    let verify_res = ring.verify().expect("verify failed");
    assert!(verify_res.ok);
    assert_eq!(verify_res.checked, 2);
}

