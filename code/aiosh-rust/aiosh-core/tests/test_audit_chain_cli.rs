//! Automated CLI tests for Audit Chain Extensions subcommands (T-02325).

use std::path::PathBuf;
use std::process::Command;
use serde_json::Value;

use aiosh_core::audit::{AuditRing, AuditRowInput, ExtendedAuditRowInput, OpenOptions};
use aiosh_core::audit_chain_ext::{AuditCausalLink, AuditProvenance, AuditSignature};

fn get_aiosh_bin() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut bin = manifest_dir.parent().unwrap().join("target").join("debug").join("aiosh.exe");
    if !bin.exists() {
        bin = manifest_dir.parent().unwrap().join("target").join("debug").join("aiosh");
    }
    assert!(bin.exists(), "aiosh binary not found at {}", bin.display());
    bin
}

#[test]
fn test_cli_audit_query_and_inspect() {
    let bin = get_aiosh_bin();
    let temp_dir = std::env::temp_dir().join(format!("aios_cli_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = std::fs::create_dir_all(&temp_dir);

    // Setup an initial audit.db in temp_dir
    let db_path = temp_dir.join("audit.db").to_str().unwrap().to_string();
    let row_hash;
    {
        let mut ring = AuditRing::open(OpenOptions {
            path: Some(db_path.clone()),
            home: None,
        }).expect("open ring");

        let mut base = AuditRowInput::default();
        base.actor = "agent_cli_tester".into();
        base.tool = "aios.pep.eval".into();
        let mut input = ExtendedAuditRowInput::new(base);
        input.provenance = Some(AuditProvenance {
            session_id: Some("session-cli-test-01".into()),
            pep_grant_id: None,
            delegation_depth: 0,
            trace_id: Some("trace-cli-999".into()),
            span_id: None,
        });
        let r = ring.write_extended(input).expect("write");
        row_hash = r.hash;
    }

    // 1. Test aiosh audit query with --session
    let out = Command::new(&bin)
        .env("AIOSH_HOME", &temp_dir)
        .args(["audit", "query", "--session", "session-cli-test-01"])
        .output()
        .expect("exec aiosh audit query");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json_val: Value = serde_json::from_str(&stdout).expect("valid JSON stdout");
    assert_eq!(json_val["ok"], true);
    assert_eq!(json_val["subcommand"], "audit query");
    assert_eq!(json_val["data"]["count"], 1);
    assert_eq!(json_val["data"]["rows"][0]["actor"], "agent_cli_tester");

    // 2. Test aiosh audit inspect
    let out_inspect = Command::new(&bin)
        .env("AIOSH_HOME", &temp_dir)
        .args(["audit", "inspect", &row_hash])
        .output()
        .expect("exec aiosh audit inspect");

    assert!(out_inspect.status.success());
    let stdout_insp = String::from_utf8_lossy(&out_inspect.stdout);
    let json_insp: Value = serde_json::from_str(&stdout_insp).expect("valid JSON stdout");
    assert_eq!(json_insp["ok"], true);
    assert_eq!(json_insp["subcommand"], "audit inspect");
    assert_eq!(json_insp["data"]["hash"], row_hash);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_cli_audit_ancestry_and_sign_verify() {
    let bin = get_aiosh_bin();
    let temp_dir = std::env::temp_dir().join(format!("aios_cli_test_ancestry_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let db_path = temp_dir.join("audit.db").to_str().unwrap().to_string();
    let child_hash;
    {
        let mut ring = AuditRing::open(OpenOptions {
            path: Some(db_path.clone()),
            home: None,
        }).expect("open ring");

        let mut root_base = AuditRowInput::default();
        root_base.actor = "root_agent".into();
        root_base.tool = "aios.root".into();
        let root = ring.write_extended(ExtendedAuditRowInput::new(root_base)).expect("write root");

        let mut child_base = AuditRowInput::default();
        child_base.actor = "child_agent".into();
        child_base.tool = "aios.child".into();
        let mut child_input = ExtendedAuditRowInput::new(child_base);
        child_input.causal_links = vec![AuditCausalLink::new(root.hash, "delegation")];
        child_input.signature = Some(AuditSignature::new(
            "ed25519",
            "pubkey1234567890abcdef",
            "sig1234567890abcdef",
        ));
        let child = ring.write_extended(child_input).expect("write child");
        child_hash = child.hash;
    }

    // 1. Test aiosh audit ancestry
    let out_ancestry = Command::new(&bin)
        .env("AIOSH_HOME", &temp_dir)
        .args(["audit", "ancestry", &child_hash, "--depth", "5"])
        .output()
        .expect("exec aiosh audit ancestry");

    assert!(out_ancestry.status.success());
    let stdout_anc = String::from_utf8_lossy(&out_ancestry.stdout);
    let json_anc: Value = serde_json::from_str(&stdout_anc).expect("valid JSON stdout");
    assert_eq!(json_anc["ok"], true);
    assert_eq!(json_anc["subcommand"], "audit ancestry");
    assert_eq!(json_anc["data"]["ancestors"].as_array().unwrap().len(), 1);

    // 2. Test aiosh audit sign-verify
    let out_sig = Command::new(&bin)
        .env("AIOSH_HOME", &temp_dir)
        .args(["audit", "sign-verify", &child_hash])
        .output()
        .expect("exec aiosh audit sign-verify");

    assert!(out_sig.status.success());
    let stdout_sig = String::from_utf8_lossy(&out_sig.stdout);
    let json_sig: Value = serde_json::from_str(&stdout_sig).expect("valid JSON stdout");
    assert_eq!(json_sig["ok"], true);
    assert_eq!(json_sig["subcommand"], "audit sign-verify");
    assert_eq!(json_sig["data"]["is_valid"], true);
    assert_eq!(json_sig["data"]["has_signature"], true);

    // 3. Negative case: usage refusal on missing hash
    let out_bad = Command::new(&bin)
        .env("AIOSH_HOME", &temp_dir)
        .args(["audit", "ancestry"])
        .output()
        .expect("exec aiosh audit ancestry missing arg");

    assert_eq!(out_bad.status.code(), Some(2));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
