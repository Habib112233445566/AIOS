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
