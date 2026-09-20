//! Unit tests for PEP Decision Engine Security Policy Subsystem (PEPPOL1..PEPPOL6).

use tempfile::tempdir;

use aiosh_core::pep_decision::{
    PepDecision, PepDecisionEffect, PepObligation, PepPolicyRule,
};
use aiosh_core::pep_security_policy::{
    validate_policy_path, PepEnforcementMode, PepObligationCriticality,
    PepSecurityPolicy, MAX_PEP_POLICY_DESC_LEN, MAX_PEP_POLICY_VERSION_LEN,
    MAX_PREFIX_LEN, MAX_RESTRICTED_PREFIXES, PEPPOL_ERR_PRIVILEGE, PEPPOL_ERR_TEMPORAL,
    PEPPOL_ERR_VALIDATION,
};

#[test]
fn test_pep_security_policy_default() {
    let policy = PepSecurityPolicy::default();
    assert_eq!(policy.version, "1.0.0");
    assert_eq!(policy.mode, PepEnforcementMode::Enforcing);
    assert_eq!(policy.obligation_criticality, PepObligationCriticality::Strict);
    assert!(policy.restricted_resource_prefixes.contains(&"sys:".to_string()));
    assert!(policy.validate().is_ok());
}

#[test]
fn test_pep_security_policy_validation_bounds() {
    // Empty version
    let mut bad = PepSecurityPolicy::default();
    bad.version = "".to_string();
    assert!(bad.validate().unwrap_err().contains(PEPPOL_ERR_VALIDATION));

    // Version too long
    bad.version = "a".repeat(MAX_PEP_POLICY_VERSION_LEN + 1);
    assert!(bad.validate().unwrap_err().contains(PEPPOL_ERR_VALIDATION));

    // Description too long
    let mut bad_desc = PepSecurityPolicy::default();
    bad_desc.description = "d".repeat(MAX_PEP_POLICY_DESC_LEN + 1);
    assert!(bad_desc.validate().unwrap_err().contains(PEPPOL_ERR_VALIDATION));

    // Too many restricted prefixes
    let mut bad_prefixes = PepSecurityPolicy::default();
    bad_prefixes.restricted_resource_prefixes = (0..=MAX_RESTRICTED_PREFIXES)
        .map(|i| format!("res_{}:", i))
        .collect();
    assert!(bad_prefixes.validate().unwrap_err().contains(PEPPOL_ERR_VALIDATION));

    // Prefix too long
    let mut bad_single_prefix = PepSecurityPolicy::default();
    bad_single_prefix.restricted_resource_prefixes = vec!["x".repeat(MAX_PREFIX_LEN + 1)];
    assert!(bad_single_prefix.validate().unwrap_err().contains(PEPPOL_ERR_VALIDATION));

    // Inverted temporal window
    let mut bad_temporal = PepSecurityPolicy::default();
    bad_temporal.valid_from_epoch_secs = Some(2000);
    bad_temporal.valid_until_epoch_secs = Some(1000);
    assert!(bad_temporal.validate().unwrap_err().contains(PEPPOL_ERR_VALIDATION));
}

#[test]
fn test_pep_security_policy_enforcement_modes() {
    let permit_dec = PepDecision::permit(
        "req_1",
        Some("rule_1".to_string()),
        "permitted",
        Vec::new(),
        10,
    );
    let deny_dec = PepDecision::deny(
        "req_2",
        Some("rule_2".to_string()),
        "denied",
        Vec::new(),
        10,
    );

    // 1. Enforcing mode
    let enforcing = PepSecurityPolicy::default().with_mode(PepEnforcementMode::Enforcing);
    let out1 = enforcing.enforce_decision(permit_dec.clone(), 100);
    assert!(out1.allowed);
    assert_eq!(out1.effect, PepDecisionEffect::Permit);

    let out2 = enforcing.enforce_decision(deny_dec.clone(), 100);
    assert!(!out2.allowed);
    assert_eq!(out2.effect, PepDecisionEffect::Deny);

    // 2. Permissive mode
    let permissive = PepSecurityPolicy::default().with_mode(PepEnforcementMode::Permissive);
    let out3 = permissive.enforce_decision(permit_dec.clone(), 100);
    assert!(out3.allowed);

    let out4 = permissive.enforce_decision(deny_dec.clone(), 100);
    assert!(out4.allowed, "permissive mode must allow denied requests");
    assert_eq!(out4.effect, PepDecisionEffect::Deny, "original effect preserved");
    assert!(out4.obligations.iter().any(|ob| match ob {
        PepObligation::AuditLog { level, message } => {
            level == "warning" && message.contains("permissive bypass")
        }
        _ => false,
    }));

    // 3. Disabled mode
    let disabled = PepSecurityPolicy::default().with_mode(PepEnforcementMode::Disabled);
    let out5 = disabled.enforce_decision(deny_dec.clone(), 100);
    assert!(out5.allowed);
    assert_eq!(out5.effect, PepDecisionEffect::Permit);
}

#[test]
fn test_pep_security_policy_temporal_validity() {
    let policy = PepSecurityPolicy::default().with_validity(Some(1000), Some(2000));
    assert!(policy.validate().is_ok());

    assert!(!policy.is_temporally_valid(999));
    assert!(policy.is_temporally_valid(1000));
    assert!(policy.is_temporally_valid(1500));
    assert!(policy.is_temporally_valid(2000));
    assert!(!policy.is_temporally_valid(2001));

    let dec = PepDecision::permit("req_t", None, "ok", Vec::new(), 5);
    let expired_dec = policy.enforce_decision(dec, 2500);
    assert!(!expired_dec.allowed);
    assert_eq!(expired_dec.effect, PepDecisionEffect::Deny);
    assert!(expired_dec.reason.contains(PEPPOL_ERR_TEMPORAL));
}

#[test]
fn test_pep_security_policy_privilege_governance() {
    let policy = PepSecurityPolicy::default();

    let sys_permit = PepPolicyRule {
        id: "sys_permit".to_string(),
        target_subject: None,
        target_resource: Some("sys:kernel:module".to_string()),
        target_action: Some("load".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: Vec::new(),
        description: "permit sys".to_string(),
    };

    let sys_deny = PepPolicyRule {
        id: "sys_deny".to_string(),
        target_subject: None,
        target_resource: Some("sys:kernel:module".to_string()),
        target_action: Some("load".to_string()),
        effect: PepDecisionEffect::Deny,
        obligations: Vec::new(),
        description: "deny sys".to_string(),
    };

    let app_permit = PepPolicyRule {
        id: "app_permit".to_string(),
        target_subject: None,
        target_resource: Some("app:workload:run".to_string()),
        target_action: Some("start".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: Vec::new(),
        description: "permit app".to_string(),
    };

    // Privileged caller can add anything
    assert!(policy.validate_rule_addition(&sys_permit, true).is_ok());

    // Unprivileged caller cannot add Permit on restricted resource
    let err = policy.validate_rule_addition(&sys_permit, false).unwrap_err();
    assert!(err.contains(PEPPOL_ERR_PRIVILEGE));

    // Unprivileged caller can add Deny rule on restricted resource
    assert!(policy.validate_rule_addition(&sys_deny, false).is_ok());

    // Unprivileged caller can add Permit rule on non-restricted resource
    assert!(policy.validate_rule_addition(&app_permit, false).is_ok());
}

#[test]
fn test_pep_security_policy_obligation_criticality() {
    let initial_permit = PepDecision::permit("req_ob", None, "allowed", Vec::new(), 10);
    let failed_ob = PepObligation::RateLimit {
        key: "agent:test".to_string(),
        cost: 10,
    };

    // 1. Strict criticality
    let strict_policy = PepSecurityPolicy::default().with_criticality(PepObligationCriticality::Strict);
    let strict_out = strict_policy.handle_obligation_failure(
        initial_permit.clone(),
        &failed_ob,
        "rate limit redis connection timeout",
    );
    assert!(!strict_out.allowed, "strict failure must convert permit to deny");
    assert_eq!(strict_out.effect, PepDecisionEffect::Deny);
    assert!(strict_out.reason.contains("obligation fulfillment failed strictly"));

    // 2. BestEffort criticality
    let best_effort_policy = PepSecurityPolicy::default().with_criticality(PepObligationCriticality::BestEffort);
    let be_out = best_effort_policy.handle_obligation_failure(
        initial_permit.clone(),
        &failed_ob,
        "rate limit redis connection timeout",
    );
    assert!(be_out.allowed, "best effort must preserve permit");
    assert_eq!(be_out.effect, PepDecisionEffect::Permit);
    assert!(be_out.obligations.iter().any(|o| match o {
        PepObligation::AuditLog { level, message } => {
            level == "error" && message.contains("non-fatal obligation delivery failure")
        }
        _ => false,
    }));
}

#[test]
fn test_pep_security_policy_persistence_roundtrip() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("pep_security_policy.json");

    let policy = PepSecurityPolicy::default()
        .with_mode(PepEnforcementMode::Permissive)
        .with_criticality(PepObligationCriticality::BestEffort)
        .with_validity(Some(100), Some(500))
        .with_restricted_prefix("custom_sec:");

    assert!(policy.save_to_path(&file_path).is_ok());
    assert!(file_path.exists());

    let loaded = PepSecurityPolicy::load_from_path(&file_path).expect("load policy");
    assert_eq!(loaded.mode, PepEnforcementMode::Permissive);
    assert_eq!(loaded.obligation_criticality, PepObligationCriticality::BestEffort);
    assert_eq!(loaded.valid_from_epoch_secs, Some(100));
    assert_eq!(loaded.valid_until_epoch_secs, Some(500));
    assert!(loaded.restricted_resource_prefixes.contains(&"custom_sec:".to_string()));
}

#[test]
fn test_pep_security_policy_path_traversal_and_errors() {
    let bad_path = std::path::Path::new("some/../escape/policy.json");
    assert!(validate_policy_path(bad_path).is_err());

    let bad_ext = std::path::Path::new("policy.yaml");
    assert!(validate_policy_path(bad_ext).is_err());

    let nonexistent = std::path::Path::new("nonexistent_pep_policy.json");
    assert!(PepSecurityPolicy::load_from_path(nonexistent).is_err());
}
