//! Comprehensive Unit & Boundary Tests for PEP Recovery & Validation (`PEPRECV1..PEPRECV6`).

use std::fs;
use aiosh_core::pep_recovery::{
    PepRecoveryManager, PepRecoveryStrategy, PepStoreValidator,
    PEPRECV_ERR_CAPACITY, PEPRECV_ERR_DUPLICATE_ID, PEPRECV_ERR_IO,
    PEPRECV_ERR_PATH_TRAVERSAL, PEPRECV_ERR_RULE_SYNTAX,
};
use aiosh_core::pep_decision_service::MAX_RULES_IN_SERVICE;

#[test]
fn test_pep_recovery_valid_store_array_and_object() {
    // 1. Array format
    let array_json = r#"{
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
    let rep_arr = PepStoreValidator::validate_content(array_json, None);
    assert!(rep_arr.is_valid);
    assert_eq!(rep_arr.valid_rules_count, 1);
    assert_eq!(rep_arr.corrupt_rules_count, 0);

    // 2. Object map format (native PepDecisionService format)
    let map_json = r#"{
        "rules": {
            "r1": {
                "id": "r1",
                "target_subject": "agent:worker",
                "target_resource": "fs:/var/*",
                "target_action": "write",
                "effect": "deny",
                "obligations": [],
                "description": "deny var"
            }
        }
    }"#;
    let rep_map = PepStoreValidator::validate_content(map_json, None);
    assert!(rep_map.is_valid);
    assert_eq!(rep_map.valid_rules_count, 1);
    assert_eq!(rep_map.corrupt_rules_count, 0);
}

#[test]
fn test_pep_recovery_duplicate_id_detection() {
    let dup_json = r#"{
        "rules": [
            {
                "id": "dup_rule",
                "target_subject": "agent:a",
                "target_resource": "res:a",
                "target_action": "read",
                "effect": "permit",
                "obligations": [],
                "description": "rule 1"
            },
            {
                "id": "dup_rule",
                "target_subject": "agent:b",
                "target_resource": "res:b",
                "target_action": "write",
                "effect": "deny",
                "obligations": [],
                "description": "rule 2"
            }
        ]
    }"#;
    let rep = PepStoreValidator::validate_content(dup_json, None);
    assert!(!rep.is_valid);
    assert_eq!(rep.total_rules_scanned, 2);
    assert_eq!(rep.valid_rules_count, 1);
    assert_eq!(rep.corrupt_rules_count, 1);
    assert!(rep.issues.iter().any(|i| i.code == PEPRECV_ERR_DUPLICATE_ID));
}

#[test]
fn test_pep_recovery_semantic_target_validation() {
    // 1. Path traversal in resource
    let bad_resource = r#"{
        "rules": [
            {
                "id": "r_trav",
                "target_subject": "agent:ok",
                "target_resource": "fs:/var/../etc/passwd",
                "target_action": "read",
                "effect": "permit",
                "obligations": [],
                "description": "traversal attempt"
            }
        ]
    }"#;
    let rep1 = PepStoreValidator::validate_content(bad_resource, None);
    assert!(!rep1.is_valid);
    assert!(rep1.issues.iter().any(|i| i.code == PEPRECV_ERR_RULE_SYNTAX && i.message.contains("traversal")));

    // 2. Control characters in subject
    let bad_subject = r#"{
        "rules": [
            {
                "id": "r_ctrl",
                "target_subject": "agent:bad\nnewline",
                "target_resource": "res:ok",
                "target_action": "read",
                "effect": "permit",
                "obligations": [],
                "description": "ctrl char"
            }
        ]
    }"#;
    let rep2 = PepStoreValidator::validate_content(bad_subject, None);
    assert!(!rep2.is_valid);
    assert!(rep2.issues.iter().any(|i| i.code == PEPRECV_ERR_RULE_SYNTAX && i.message.contains("control chars")));

    // 3. Action length > 64
    let long_action = "a".repeat(65);
    let bad_action = format!(r#"{{
        "rules": [
            {{
                "id": "r_long_act",
                "target_subject": "agent:ok",
                "target_resource": "res:ok",
                "target_action": "{}",
                "effect": "permit",
                "obligations": [],
                "description": "oversized action"
            }}
        ]
    }}"#, long_action);
    let rep3 = PepStoreValidator::validate_content(&bad_action, None);
    assert!(!rep3.is_valid);
    assert!(rep3.issues.iter().any(|i| i.code == PEPRECV_ERR_RULE_SYNTAX && i.message.contains("length > 64")));
}

#[test]
fn test_pep_recovery_capacity_bounds() {
    let mut rules = Vec::new();
    for i in 0..(MAX_RULES_IN_SERVICE + 1) {
        rules.push(format!(
            r#"{{"id":"r_{}","target_subject":"agent:u","target_resource":"res:x","target_action":"read","effect":"permit"}}"#,
            i
        ));
    }
    let overflow_json = format!(r#"{{"rules": [{}]}}"#, rules.join(","));
    let rep = PepStoreValidator::validate_content(&overflow_json, None);
    assert!(!rep.is_valid);
    assert!(rep.issues.iter().any(|i| i.code == PEPRECV_ERR_CAPACITY));
}

#[test]
fn test_pep_recovery_file_path_hygiene() {
    // 1. Non-existent file
    let rep_err = PepStoreValidator::validate_path(std::path::Path::new("nonexistent_pep_store.json"));
    assert!(rep_err.is_err());
    assert!(rep_err.unwrap_err().contains(PEPRECV_ERR_IO));

    // 2. Traversal path
    let rep_trav = PepStoreValidator::validate_path(std::path::Path::new("../secret.json"));
    assert!(rep_trav.is_err());
    assert!(rep_trav.unwrap_err().contains(PEPRECV_ERR_PATH_TRAVERSAL));
}

#[test]
fn test_pep_recovery_salvage_and_quarantine_strategy() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let store_path = tmp_dir.path().join("policies.json");

    // Construct store with 2 valid rules and 2 corrupt rules
    let mixed_store = r#"{
        "rules": [
            {
                "id": "valid_1",
                "target_subject": "agent:valid",
                "target_resource": "res:valid",
                "target_action": "read",
                "effect": "permit",
                "obligations": [],
                "description": "good 1"
            },
            {
                "id": "corrupt_traversal",
                "target_subject": "agent:bad",
                "target_resource": "fs:/../etc/shadow",
                "target_action": "read",
                "effect": "permit",
                "obligations": [],
                "description": "bad resource"
            },
            {
                "id": "valid_2",
                "target_subject": "agent:valid2",
                "target_resource": "res:valid2",
                "target_action": "write",
                "effect": "deny",
                "obligations": [],
                "description": "good 2"
            },
            {
                "id": "valid_2",
                "target_subject": "agent:dup",
                "target_resource": "res:dup",
                "target_action": "read",
                "effect": "permit",
                "obligations": [],
                "description": "duplicate id"
            }
        ]
    }"#;
    fs::write(&store_path, mixed_store).unwrap();

    // Execute salvage recovery
    let res = PepRecoveryManager::recover_store(&store_path, PepRecoveryStrategy::SalvageValidRules).unwrap();
    assert!(res.success);
    assert_eq!(res.rules_salvaged, 2);
    assert_eq!(res.rules_dropped, 2);
    assert!(res.quarantine_path.is_some());

    // Verify quarantine file exists and contains original mixed store
    let q_path = std::path::PathBuf::from(res.quarantine_path.unwrap());
    assert!(q_path.exists());
    let q_content = fs::read_to_string(&q_path).unwrap();
    assert!(q_content.contains("corrupt_traversal"));

    // Verify rewritten store is now fully valid
    let check = PepStoreValidator::validate_path(&store_path).unwrap();
    assert!(check.is_valid);
    assert_eq!(check.valid_rules_count, 2);
    assert_eq!(check.corrupt_rules_count, 0);
}

#[test]
fn test_pep_recovery_strict_fail_closed() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let store_path = tmp_dir.path().join("policies.json");

    let corrupt_json = r#"{"rules": [{"id": "bad", "target_resource": "fs:/../root"}]}"#;
    fs::write(&store_path, corrupt_json).unwrap();

    let res = PepRecoveryManager::recover_store(&store_path, PepRecoveryStrategy::StrictFailClosed).unwrap();
    assert!(res.success);
    assert_eq!(res.rules_salvaged, 0);
    assert_eq!(res.rules_dropped, 1);
    assert!(res.quarantine_path.is_some());

    // Clean fresh store has 0 rules
    let check = PepStoreValidator::validate_path(&store_path).unwrap();
    assert!(check.is_valid);
    assert_eq!(check.valid_rules_count, 0);
}

#[test]
fn test_pep_recovery_dry_run() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let store_path = tmp_dir.path().join("policies.json");

    let corrupt_json = r#"{"rules": [{"id": "bad", "target_resource": "fs:/../root"}]}"#;
    fs::write(&store_path, corrupt_json).unwrap();

    let res = PepRecoveryManager::recover_store(&store_path, PepRecoveryStrategy::DryRun).unwrap();
    assert!(!res.success); // DryRun on invalid store returns success=false
    assert!(res.quarantine_path.is_none()); // No quarantine created

    // Disk content remains unchanged
    let after = fs::read_to_string(&store_path).unwrap();
    assert_eq!(after, corrupt_json);
}
