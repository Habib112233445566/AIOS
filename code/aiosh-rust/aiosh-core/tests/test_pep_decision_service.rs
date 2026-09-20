//! Unit tests for PEP Decision Service (PEPSERV1..PEPSERV6).

use std::fs;
use tempfile::tempdir;

use aiosh_core::pep_decision::{
    PepCombiningAlgorithm, PepDecisionEffect, PepObligation, PepPolicyRule, PepRequest,
};
use aiosh_core::pep_decision_service::{
    validate_pep_service_path, PepDecisionService, MAX_RULES_IN_SERVICE,
};

#[test]
fn test_service_new_empty() {
    let service = PepDecisionService::new();
    assert_eq!(service.len(), 0);
    assert!(service.is_empty());
    assert_eq!(service.algorithm(), PepCombiningAlgorithm::DenyOverrides);
}

#[test]
fn test_service_add_and_get_rule() {
    let mut service = PepDecisionService::new();
    let rule = PepPolicyRule {
        id: "rule_1".to_string(),
        target_subject: Some("agent:tester".to_string()),
        target_resource: Some("fs:/data/*".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: vec![PepObligation::AuditLog {
            level: "info".to_string(),
            message: "rule_1 triggered".to_string(),
        }],
        description: "permit tester read data".to_string(),
    };

    assert!(service.add_rule(rule.clone()).is_ok());
    assert_eq!(service.len(), 1);
    assert!(!service.is_empty());

    let fetched = service.get_rule("rule_1").unwrap();
    assert_eq!(fetched.id, "rule_1");
    assert_eq!(fetched.effect, PepDecisionEffect::Permit);

    let all_rules = service.list_rules();
    assert_eq!(all_rules.len(), 1);
}

#[test]
fn test_service_remove_rule() {
    let mut service = PepDecisionService::new();
    let rule = PepPolicyRule {
        id: "rule_rm".to_string(),
        target_subject: Some("agent:worker".to_string()),
        target_resource: Some("fs:/tmp/*".to_string()),
        target_action: Some("write".to_string()),
        effect: PepDecisionEffect::Deny,
        obligations: Vec::new(),
        description: "deny worker write tmp".to_string(),
    };

    service.add_rule(rule).unwrap();
    assert_eq!(service.len(), 1);

    assert!(service.remove_rule("rule_rm"));
    assert_eq!(service.len(), 0);
    assert!(service.is_empty());
    assert!(!service.remove_rule("rule_rm"));
}

#[test]
fn test_service_evaluation() {
    let mut service = PepDecisionService::new();
    let rule = PepPolicyRule {
        id: "rule_eval".to_string(),
        target_subject: Some("agent:eval".to_string()),
        target_resource: Some("net:api.example.com".to_string()),
        target_action: Some("connect".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: Vec::new(),
        description: "permit connect".to_string(),
    };
    service.add_rule(rule).unwrap();

    // Matching request
    let req = PepRequest::new("agent:eval", "net:api.example.com", "connect", None).unwrap();
    let decision = service.evaluate(&req);
    assert_eq!(decision.effect, PepDecisionEffect::Permit);
    assert!(decision.allowed);
    assert_eq!(decision.matched_rule_id.as_deref(), Some("rule_eval"));

    // Non-matching request defaults to Deny (PEPDEC1)
    let non_matching = PepRequest::new("agent:eval", "net:evil.com", "connect", None).unwrap();
    let denied = service.evaluate(&non_matching);
    assert_eq!(denied.effect, PepDecisionEffect::Deny);
    assert!(!denied.allowed);
}

#[test]
fn test_service_save_and_load() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("pep_policy.json");

    let mut service = PepDecisionService::new().with_algorithm(PepCombiningAlgorithm::FirstApplicable);
    let rule = PepPolicyRule {
        id: "rule_persist".to_string(),
        target_subject: Some("agent:persist".to_string()),
        target_resource: Some("fs:/persisted".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: Vec::new(),
        description: "persisted rule".to_string(),
    };
    service.add_rule(rule).unwrap();

    // Save
    assert!(service.save_to_path(&store_path).is_ok());
    assert!(store_path.exists());

    // Load
    let loaded = PepDecisionService::load_from_path(&store_path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded.algorithm(), PepCombiningAlgorithm::FirstApplicable);
    assert!(loaded.get_rule("rule_persist").is_some());
}

#[test]
fn test_service_load_or_recover_corrupt() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("pep_policy.json");

    // Write corrupt content
    fs::write(&store_path, "{ broken invalid json").unwrap();

    let (recovered_service, recovered, backup_path) = PepDecisionService::load_or_recover(&store_path);
    assert!(recovered, "expected recovered to be true");
    assert!(backup_path.is_some());
    assert_eq!(recovered_service.len(), 0);

    // Verify backup exists and contains original broken content
    let bak = backup_path.unwrap();
    assert!(std::path::Path::new(&bak).exists());
    assert_eq!(fs::read_to_string(&bak).unwrap(), "{ broken invalid json");
}

#[test]
fn test_service_path_traversal_rejected() {
    let bad_path = std::path::Path::new("some/../path/policy.json");
    assert!(validate_pep_service_path(bad_path).is_err());

    let non_json = std::path::Path::new("policy.txt");
    assert!(validate_pep_service_path(non_json).is_err());
}

#[test]
fn test_service_capacity_limit() {
    let mut service = PepDecisionService::new();
    for i in 0..MAX_RULES_IN_SERVICE {
        service.add_rule(PepPolicyRule {
            id: format!("rule_{}", i),
            target_subject: None,
            target_resource: None,
            target_action: None,
            effect: PepDecisionEffect::Permit,
            obligations: Vec::new(),
            description: "test".to_string(),
        }).unwrap();
    }
    assert_eq!(service.len(), MAX_RULES_IN_SERVICE);

    let excess_rule = PepPolicyRule {
        id: "rule_excess".to_string(),
        target_subject: None,
        target_resource: None,
        target_action: None,
        effect: PepDecisionEffect::Permit,
        obligations: Vec::new(),
        description: "excess".to_string(),
    };
    let err = service.add_rule(excess_rule).unwrap_err();
    assert!(err.contains("capacity limit reached"));
}
