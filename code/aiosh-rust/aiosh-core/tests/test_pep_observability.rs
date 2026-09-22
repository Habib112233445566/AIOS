//! Unit tests for PEP Decision Engine Observability Subsystem (PEPOBS1..PEPOBS6).

use aiosh_core::pep_decision::{
    PepCombiningAlgorithm, PepDecisionEffect, PepObligation, PepPolicyRule,
};
use aiosh_core::pep_decision_service::PepDecisionService;
use aiosh_core::pep_observability::{
    sanitize_telemetry_text, PepObservabilityReport, PEPOBS_ERR_VALIDATION,
    PEP_HEALTH_UTILIZATION_THRESHOLD,
};
use aiosh_core::pep_security_policy::{PepEnforcementMode, PepObligationCriticality, PepSecurityPolicy};

#[test]
fn test_pep_observability_empty_service() {
    let service = PepDecisionService::new().with_algorithm(PepCombiningAlgorithm::DenyOverrides);
    let policy = PepSecurityPolicy::default();

    let report = PepObservabilityReport::generate(&service, &policy, "2026-09-21T12:00:00Z");

    assert_eq!(report.total_rules, 0);
    assert_eq!(report.rules_with_obligations, 0);
    assert_eq!(report.unique_subjects_count, 0);
    assert_eq!(report.unique_resources_count, 0);
    assert_eq!(report.unique_actions_count, 0);
    assert_eq!(report.capacity_utilization_percent, 0);
    assert!(report.is_healthy);
    assert_eq!(report.default_algorithm, "denyoverrides");
    assert_eq!(report.enforcement_mode, "enforcing");
    assert_eq!(report.obligation_criticality, "strict");
    assert_eq!(report.generated_at, "2026-09-21T12:00:00Z");

    assert_eq!(*report.rules_by_effect.get("permit").unwrap_or(&99), 0);
    assert_eq!(*report.rules_by_effect.get("deny").unwrap_or(&99), 0);

    assert!(report.validate().is_ok());
}

#[test]
fn test_pep_observability_populated_service() {
    let mut service = PepDecisionService::new().with_algorithm(PepCombiningAlgorithm::PermitOverrides);

    // Rule 1: Permit with AuditLog and RateLimit
    let r1 = PepPolicyRule {
        id: "rule-1".to_string(),
        description: "Permit admin read".to_string(),
        target_subject: Some("admin".to_string()),
        target_resource: Some("resource:logs".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: vec![
            PepObligation::AuditLog {
                level: "info".to_string(),
                message: "admin_read".to_string(),
            },
            PepObligation::RateLimit {
                key: "admin".to_string(),
                cost: 100,
            },
        ],
    };
    service.add_rule(r1).unwrap();

    // Rule 2: Deny with RedactFields
    let r2 = PepPolicyRule {
        id: "rule-2".to_string(),
        description: "Deny guest write".to_string(),
        target_subject: Some("guest".to_string()),
        target_resource: Some("resource:confidential".to_string()),
        target_action: Some("write".to_string()),
        effect: PepDecisionEffect::Deny,
        obligations: vec![PepObligation::RedactFields {
            fields: vec!["ssn".to_string()],
        }],
    };
    service.add_rule(r2).unwrap();

    // Rule 3: Permit with Custom obligation
    let r3 = PepPolicyRule {
        id: "rule-3".to_string(),
        description: "Permit worker execute".to_string(),
        target_subject: Some("worker".to_string()),
        target_resource: Some("resource:queue".to_string()),
        target_action: Some("execute".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: vec![PepObligation::Custom {
            name: "metric_counter".to_string(),
            payload: serde_json::json!({"k": "v"}),
        }],
    };
    service.add_rule(r3).unwrap();

    let mut policy = PepSecurityPolicy::default();
    policy.mode = PepEnforcementMode::Permissive;
    policy.obligation_criticality = PepObligationCriticality::BestEffort;

    let report = PepObservabilityReport::generate(&service, &policy, "2026-09-21T12:30:00Z");

    assert_eq!(report.total_rules, 3);
    assert_eq!(report.rules_with_obligations, 3);
    assert_eq!(report.unique_subjects_count, 3);
    assert_eq!(report.unique_resources_count, 3);
    assert_eq!(report.unique_actions_count, 3);
    assert_eq!(*report.rules_by_effect.get("permit").unwrap(), 2);
    assert_eq!(*report.rules_by_effect.get("deny").unwrap(), 1);

    assert_eq!(*report.obligations_by_type.get("audit_log").unwrap(), 1);
    assert_eq!(*report.obligations_by_type.get("rate_limit").unwrap(), 1);
    assert_eq!(*report.obligations_by_type.get("redact_fields").unwrap(), 1);
    assert_eq!(*report.obligations_by_type.get("custom").unwrap(), 1);

    assert_eq!(report.enforcement_mode, "permissive");
    assert_eq!(report.obligation_criticality, "besteffort");
    assert!(report.is_healthy);
    assert!(report.validate().is_ok());
}

#[test]
fn test_pep_observability_health_utilization_threshold() {
    let service = PepDecisionService::new().with_algorithm(PepCombiningAlgorithm::DenyOverrides);
    let policy = PepSecurityPolicy::default();

    let mut report = PepObservabilityReport::generate(&service, &policy, "");
    assert!(report.is_healthy);
    assert_eq!(PEP_HEALTH_UTILIZATION_THRESHOLD, 90);

    // Below threshold (89%) -> healthy
    report.capacity_utilization_percent = 89;
    report.is_healthy = report.capacity_utilization_percent < PEP_HEALTH_UTILIZATION_THRESHOLD;
    assert!(report.is_healthy);

    // At threshold (90%) -> unhealthy
    report.capacity_utilization_percent = 90;
    report.is_healthy = report.capacity_utilization_percent < PEP_HEALTH_UTILIZATION_THRESHOLD;
    assert!(!report.is_healthy);

    // Above threshold (95%) -> unhealthy
    report.capacity_utilization_percent = 95;
    report.is_healthy = report.capacity_utilization_percent < PEP_HEALTH_UTILIZATION_THRESHOLD;
    assert!(!report.is_healthy);
}

#[test]
fn test_pep_observability_sanitization() {
    // Strips control characters
    let raw = "user\x00name\x07\r\n\t_test";
    let cleaned = sanitize_telemetry_text(raw);
    assert_eq!(cleaned, "username_test");

    // Limits length to 256 characters
    let long_str = "x".repeat(500);
    let cleaned_long = sanitize_telemetry_text(&long_str);
    assert_eq!(cleaned_long.len(), 256);

    // Trims whitespace
    let padded = "   valid_timestamp_string   ";
    assert_eq!(sanitize_telemetry_text(padded), "valid_timestamp_string");
}

#[test]
fn test_pep_observability_validation_invariants() {
    let service = PepDecisionService::new().with_algorithm(PepCombiningAlgorithm::DenyOverrides);
    let policy = PepSecurityPolicy::default();

    let report = PepObservabilityReport::generate(&service, &policy, "2026-09-21T12:00:00Z");
    assert!(report.validate().is_ok());

    // Invariant: total_rules > capacity_limit
    let mut bad_rules = report.clone();
    bad_rules.total_rules = bad_rules.capacity_limit + 1;
    let err = bad_rules.validate().unwrap_err();
    assert!(err.contains(PEPOBS_ERR_VALIDATION));
    assert!(err.contains("exceeds capacity_limit"));

    // Invariant: capacity_utilization_percent > 100
    let mut bad_util = report.clone();
    bad_util.capacity_utilization_percent = 101;
    let err = bad_util.validate().unwrap_err();
    assert!(err.contains(PEPOBS_ERR_VALIDATION));
    assert!(err.contains("exceeds 100"));

    // Invariant: sum of effects != total_rules
    let mut bad_effects = report.clone();
    bad_effects.total_rules = 5;
    let err = bad_effects.validate().unwrap_err();
    assert!(err.contains(PEPOBS_ERR_VALIDATION));
    assert!(err.contains("sum of rules by effect"));

    // Invariant: rules_with_obligations > total_rules
    let mut bad_ob_count = report.clone();
    bad_ob_count.rules_with_obligations = 10;
    let err = bad_ob_count.validate().unwrap_err();
    assert!(err.contains(PEPOBS_ERR_VALIDATION));
    assert!(err.contains("rules_with_obligations"));

    // Invariant: generated_at empty
    let mut bad_ts = report.clone();
    bad_ts.generated_at = "   ".to_string();
    let err = bad_ts.validate().unwrap_err();
    assert!(err.contains(PEPOBS_ERR_VALIDATION));
    assert!(err.contains("generated_at cannot be empty"));
}

#[test]
fn test_pep_observability_json_roundtrip() {
    let service = PepDecisionService::new().with_algorithm(PepCombiningAlgorithm::DenyOverrides);
    let policy = PepSecurityPolicy::default();

    let report = PepObservabilityReport::generate(&service, &policy, "2026-09-21T14:00:00Z");
    let json = report.to_json().expect("Serialization should succeed");

    assert!(json.contains("\"total_rules\": 0"));
    assert!(json.contains("\"is_healthy\": true"));
    assert!(json.contains("\"generated_at\": \"2026-09-21T14:00:00Z\""));

    let deserialized = PepObservabilityReport::from_json(&json).expect("Deserialization should succeed");
    assert_eq!(report, deserialized);
}

#[test]
fn test_pep_observability_timestamp_fallback() {
    let service = PepDecisionService::new().with_algorithm(PepCombiningAlgorithm::DenyOverrides);
    let policy = PepSecurityPolicy::default();

    // Empty timestamp should generate an automatic timestamp
    let report = PepObservabilityReport::generate(&service, &policy, "");
    assert!(!report.generated_at.is_empty());
    assert!(report.validate().is_ok());
}
