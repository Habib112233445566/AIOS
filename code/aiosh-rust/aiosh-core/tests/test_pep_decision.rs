//! Unit tests for PEP Decision Engine Data Model (PEPDEC1..PEPDEC6).

use std::collections::HashMap;

use aiosh_core::pep_decision::{
    evaluate_rules, match_pattern, PepCombiningAlgorithm, PepDecision,
    PepDecisionEffect, PepDecisionError, PepEnvironmentContext, PepObligation, PepPolicyRule,
    PepRequest, MAX_PEP_ACTION_LEN, MAX_PEP_RESOURCE_LEN,
};

#[test]
fn test_pep_request_valid_construction() {
    let mut attrs = HashMap::new();
    attrs.insert("auth_level".to_string(), "mfa".to_string());
    let env = PepEnvironmentContext {
        timestamp: "2026-09-20T18:00:00Z".to_string(),
        session_id: Some("sess_12345".to_string()),
        client_ip: Some("127.0.0.1".to_string()),
        grant_id: Some("grant_abc".to_string()),
        attributes: attrs,
    };

    let req = PepRequest::new("agent:researcher", "fs:/data/reports", "read", Some(env)).unwrap();
    assert!(req.id.starts_with("pep_req_"));
    assert_eq!(req.subject, "agent:researcher");
    assert_eq!(req.resource, "fs:/data/reports");
    assert_eq!(req.action, "read");
    assert_eq!(req.environment.session_id.as_deref(), Some("sess_12345"));
}

#[test]
fn test_pep_request_validation_failures() {
    // Empty subject
    let err1 = PepRequest::new("", "fs:/data", "read", None).unwrap_err();
    assert!(matches!(err1, PepDecisionError::InvalidSubject(_)));

    // Whitespace-only subject
    let err2 = PepRequest::new("   ", "fs:/data", "read", None).unwrap_err();
    assert!(matches!(err2, PepDecisionError::InvalidSubject(_)));

    // Control characters in subject
    let err3 = PepRequest::new("agent:\0evil", "fs:/data", "read", None).unwrap_err();
    assert!(matches!(err3, PepDecisionError::InvalidSubject(_)));

    // Resource too long
    let long_resource = "a".repeat(MAX_PEP_RESOURCE_LEN + 1);
    let err4 = PepRequest::new("agent:root", long_resource, "read", None).unwrap_err();
    assert!(matches!(err4, PepDecisionError::InvalidResource(_)));

    // Action too long
    let long_action = "a".repeat(MAX_PEP_ACTION_LEN + 1);
    let err5 = PepRequest::new("agent:root", "fs:/data", long_action, None).unwrap_err();
    assert!(matches!(err5, PepDecisionError::InvalidAction(_)));
}

#[test]
fn test_pep_decision_invariants() {
    // Permit decision satisfies PEPDEC1
    let permit = PepDecision::permit(
        "req_1",
        Some("rule_1".to_string()),
        "allowed",
        vec![PepObligation::AuditLog {
            level: "info".to_string(),
            message: "read performed".to_string(),
        }],
        42,
    );
    assert_eq!(permit.effect, PepDecisionEffect::Permit);
    assert!(permit.allowed);
    assert!(permit.validate_invariants().is_ok());

    // Deny decision satisfies PEPDEC1
    let deny = PepDecision::deny(
        "req_2",
        Some("rule_2".to_string()),
        "prohibited",
        Vec::new(),
        15,
    );
    assert_eq!(deny.effect, PepDecisionEffect::Deny);
    assert!(!deny.allowed);
    assert!(deny.validate_invariants().is_ok());

    // Default deny satisfies PEPDEC1
    let def_deny = PepDecision::default_deny("req_3", "no match");
    assert_eq!(def_deny.effect, PepDecisionEffect::Deny);
    assert!(!def_deny.allowed);
    assert!(def_deny.validate_invariants().is_ok());

    // Invariant failure detection
    let mut corrupted = permit.clone();
    corrupted.allowed = false; // Contradiction with Permit effect
    assert!(corrupted.validate_invariants().is_err());
}

#[test]
fn test_pep_pattern_matching() {
    assert!(match_pattern("*", "anything"));
    assert!(match_pattern("agent:*", "agent:researcher"));
    assert!(!match_pattern("agent:*", "user:admin"));
    assert!(match_pattern("*.json", "store.json"));
    assert!(!match_pattern("*.json", "store.txt"));
    assert!(match_pattern("read", "READ")); // case-insensitive
    assert!(!match_pattern("read", "write"));
}

#[test]
fn test_pep_combining_deny_overrides() {
    let req = PepRequest::new("agent:worker", "fs:/secret/keys", "read", None).unwrap();

    let rule_permit = PepPolicyRule {
        id: "rule_permit_all".to_string(),
        target_subject: Some("agent:*".to_string()),
        target_resource: Some("fs:*".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: vec![PepObligation::AuditLog {
            level: "info".to_string(),
            message: "permit matched".to_string(),
        }],
        description: "permit read to all fs".to_string(),
    };

    let rule_deny_secret = PepPolicyRule {
        id: "rule_deny_secrets".to_string(),
        target_subject: Some("agent:*".to_string()),
        target_resource: Some("fs:/secret/*".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Deny,
        obligations: vec![PepObligation::AuditLog {
            level: "warn".to_string(),
            message: "attempted secret access".to_string(),
        }],
        description: "deny secret access".to_string(),
    };

    // With DenyOverrides, the deny rule MUST override permit
    let rules = vec![rule_permit.clone(), rule_deny_secret.clone()];
    let decision = evaluate_rules(&rules, &req, PepCombiningAlgorithm::DenyOverrides);
    assert_eq!(decision.effect, PepDecisionEffect::Deny);
    assert!(!decision.allowed);
    assert_eq!(decision.matched_rule_id.as_deref(), Some("rule_deny_secrets"));
    assert!(decision.validate_invariants().is_ok());

    // For a non-secret path, permit should succeed
    let safe_req = PepRequest::new("agent:worker", "fs:/public/readme", "read", None).unwrap();
    let safe_decision = evaluate_rules(&rules, &safe_req, PepCombiningAlgorithm::DenyOverrides);
    assert_eq!(safe_decision.effect, PepDecisionEffect::Permit);
    assert!(safe_decision.allowed);
    assert_eq!(safe_decision.matched_rule_id.as_deref(), Some("rule_permit_all"));
    assert!(safe_decision.validate_invariants().is_ok());
}

#[test]
fn test_pep_combining_permit_overrides() {
    let req = PepRequest::new("agent:worker", "fs:/secret/keys", "read", None).unwrap();

    let rule_permit = PepPolicyRule {
        id: "rule_permit_override".to_string(),
        target_subject: Some("agent:*".to_string()),
        target_resource: Some("fs:*".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: Vec::new(),
        description: "broad permit".to_string(),
    };

    let rule_deny = PepPolicyRule {
        id: "rule_deny_keys".to_string(),
        target_subject: Some("agent:*".to_string()),
        target_resource: Some("fs:/secret/*".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Deny,
        obligations: Vec::new(),
        description: "deny keys".to_string(),
    };

    let rules = vec![rule_deny, rule_permit];
    let decision = evaluate_rules(&rules, &req, PepCombiningAlgorithm::PermitOverrides);
    assert_eq!(decision.effect, PepDecisionEffect::Permit);
    assert!(decision.allowed);
    assert!(decision.validate_invariants().is_ok());
}

#[test]
fn test_pep_combining_first_applicable() {
    let req = PepRequest::new("agent:worker", "fs:/data", "read", None).unwrap();

    let rule1 = PepPolicyRule {
        id: "rule_first_deny".to_string(),
        target_subject: Some("agent:*".to_string()),
        target_resource: Some("fs:/data".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Deny,
        obligations: Vec::new(),
        description: "first deny".to_string(),
    };

    let rule2 = PepPolicyRule {
        id: "rule_second_permit".to_string(),
        target_subject: Some("agent:*".to_string()),
        target_resource: Some("fs:/data".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: Vec::new(),
        description: "second permit".to_string(),
    };

    // First rule is Deny -> Decision is Deny
    let rules1 = vec![rule1.clone(), rule2.clone()];
    let dec1 = evaluate_rules(&rules1, &req, PepCombiningAlgorithm::FirstApplicable);
    assert_eq!(dec1.effect, PepDecisionEffect::Deny);
    assert_eq!(dec1.matched_rule_id.as_deref(), Some("rule_first_deny"));

    // First rule is Permit -> Decision is Permit
    let rules2 = vec![rule2, rule1];
    let dec2 = evaluate_rules(&rules2, &req, PepCombiningAlgorithm::FirstApplicable);
    assert_eq!(dec2.effect, PepDecisionEffect::Permit);
    assert_eq!(dec2.matched_rule_id.as_deref(), Some("rule_second_permit"));
}

#[test]
fn test_pep_empty_rules_fail_closed_default_deny() {
    let req = PepRequest::new("agent:worker", "fs:/data", "read", None).unwrap();
    let empty_rules: Vec<PepPolicyRule> = Vec::new();

    // PEPDEC1: Fail-closed default deny across all combining algorithms
    for algo in [
        PepCombiningAlgorithm::DenyOverrides,
        PepCombiningAlgorithm::PermitOverrides,
        PepCombiningAlgorithm::FirstApplicable,
    ] {
        let dec = evaluate_rules(&empty_rules, &req, algo);
        assert_eq!(dec.effect, PepDecisionEffect::Deny);
        assert!(!dec.allowed);
        assert!(dec.reason.contains("default deny"));
        assert!(dec.validate_invariants().is_ok());
    }
}

#[test]
fn test_pep_hardening_controls() {
    // Path traversal in resource rejected
    let traversal_err = PepRequest::new("agent:root", "fs:/secret/../keys", "read", None).unwrap_err();
    assert!(matches!(traversal_err, PepDecisionError::InvalidResource(_)));

    // Rule count bound enforced (> 1000 rules triggers default deny)
    let req = PepRequest::new("agent:worker", "fs:/data", "read", None).unwrap();
    let mut excessive_rules = Vec::new();
    for i in 0..1005 {
        excessive_rules.push(PepPolicyRule {
            id: format!("rule_{}", i),
            target_subject: Some("agent:*".to_string()),
            target_resource: Some("fs:*".to_string()),
            target_action: Some("read".to_string()),
            effect: PepDecisionEffect::Permit,
            obligations: Vec::new(),
            description: "permit".to_string(),
        });
    }
    let dec = evaluate_rules(&excessive_rules, &req, PepCombiningAlgorithm::DenyOverrides);
    assert_eq!(dec.effect, PepDecisionEffect::Deny);
    assert!(!dec.allowed);
    assert!(dec.reason.contains("exceeds maximum allowed"));
    assert!(dec.validate_invariants().is_ok());
}
