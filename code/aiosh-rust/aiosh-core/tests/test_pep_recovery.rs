//! Unit & Scaffold Tests for PEP Recovery Subsystem (`PEPRECV1..PEPRECV6`).

use aiosh_core::pep_recovery::{
    PepIssueSeverity, PepRecoveryManager, PepRecoveryResult, PepRecoveryStrategy,
    PepStoreValidator, PepValidationIssue, PepValidationReport,
};

#[test]
fn test_pep_recovery_scaffold_compilation_and_types() {
    // 1. Verify default recovery strategy
    let default_strat = PepRecoveryStrategy::default();
    assert_eq!(default_strat, PepRecoveryStrategy::StrictFailClosed);

    // 2. SHA-256 hash check
    let hash = PepStoreValidator::compute_sha256(b"hello aios");
    assert_eq!(hash.len(), 64);

    // 3. Validation of empty rules array
    let empty_store = r#"{"rules": []}"#;
    let report = PepStoreValidator::validate_content(empty_store, None);
    assert!(report.is_valid);
    assert_eq!(report.total_rules_scanned, 0);
    assert_eq!(report.valid_rules_count, 0);
    assert_eq!(report.corrupt_rules_count, 0);
    assert!(report.issues.is_empty());
    assert!(report.sha256_checksum.is_some());

    // 4. Validation of valid rule
    let valid_store = r#"{
        "rules": [
            {
                "id": "r1",
                "target_subject": "agent:worker",
                "target_resource": "fs:/tmp/*",
                "target_action": "read",
                "effect": "permit",
                "obligations": [],
                "description": "allow tmp"
            }
        ]
    }"#;
    let report_valid = PepStoreValidator::validate_content(valid_store, None);
    assert!(report_valid.is_valid);
    assert_eq!(report_valid.total_rules_scanned, 1);
    assert_eq!(report_valid.valid_rules_count, 1);
    assert_eq!(report_valid.corrupt_rules_count, 0);

    // 5. Validation of malformed JSON
    let bad_json = r#"{"rules": [unclosed"#;
    let report_bad: PepValidationReport = PepStoreValidator::validate_content(bad_json, None);
    assert!(!report_bad.is_valid);
    assert_eq!(report_bad.issues.len(), 1);
    let issue: &PepValidationIssue = &report_bad.issues[0];
    assert_eq!(issue.severity, PepIssueSeverity::Error);

    // 6. Test recovery manager dry-run
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("policies.json");
    std::fs::write(&file_path, valid_store).unwrap();
    let res: PepRecoveryResult = PepRecoveryManager::recover_store(&file_path, PepRecoveryStrategy::DryRun).unwrap();
    assert!(res.success);
    assert_eq!(res.rules_salvaged, 1);
}

