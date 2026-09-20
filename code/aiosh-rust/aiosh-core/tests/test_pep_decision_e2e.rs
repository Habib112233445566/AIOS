//! End-to-End Automated Test Suite for PEP Decision Engine (T-02151..T-02156).
//!
//! Validates: PEPE2E1 - PEPE2E6
//! - PEPE2E1: Full combining algorithm matrix (DenyOverrides, PermitOverrides, FirstApplicable)
//! - PEPE2E2: Obligation delivery & integrity
//! - PEPE2E3: Capacity stress & boundary limits (up to 5,000 rules)
//! - PEPE2E4: Corrupt store fault injection & non-destructive quarantine
//! - PEPE2E5: Input fuzzing & path traversal defense
//! - PEPE2E6: Cross-surface persistence & JSON parity

use std::fs;
use std::path::{Path, PathBuf};

use aiosh_core::pep_decision::{
    PepCombiningAlgorithm, PepDecisionEffect, PepObligation, PepPolicyRule, PepRequest,
};
use aiosh_core::pep_decision_service::{
    validate_pep_service_path, PepDecisionService, MAX_RULES_IN_SERVICE, PEPSERV_ERR_CAPACITY,
    PEPSERV_ERR_VALIDATION,
};

/// RAII Temporary Directory Guard ensuring zero residual artifacts even upon test panic.
struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(prefix: &str) -> Self {
        let base = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = base.join(format!("{}_{}_{}", prefix, std::process::id(), nanos));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("failed to create test temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestTempDir {
    fn drop(&mut self) {
        let temp = std::env::temp_dir();
        if self.path.starts_with(&temp) && self.path != temp {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[test]
fn test_pepe2e1_combining_algorithm_matrix() {
    let mut service = PepDecisionService::new();

    // 1. Add permit rule
    service.add_rule(PepPolicyRule {
        id: "rule_01_permit".to_string(),
        target_subject: Some("agent:worker".to_string()),
        target_resource: Some("fs:/data/*".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: Vec::new(),
        description: "Permit worker read".to_string(),
    }).expect("add permit rule");

    // 2. Add deny rule
    service.add_rule(PepPolicyRule {
        id: "rule_02_deny".to_string(),
        target_subject: Some("agent:worker".to_string()),
        target_resource: Some("fs:/data/secret.key".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Deny,
        obligations: Vec::new(),
        description: "Deny secret read".to_string(),
    }).expect("add deny rule");

    let req_secret = PepRequest::new("agent:worker", "fs:/data/secret.key", "read", None)
        .expect("valid request");

    // DenyOverrides: deny takes precedence
    let dec_deny_overrides = service.evaluate_with_algorithm(&req_secret, PepCombiningAlgorithm::DenyOverrides);
    assert_eq!(dec_deny_overrides.effect, PepDecisionEffect::Deny);
    assert!(!dec_deny_overrides.allowed);
    assert_eq!(dec_deny_overrides.matched_rule_id.as_deref(), Some("rule_02_deny"));

    // PermitOverrides: permit takes precedence
    let dec_permit_overrides = service.evaluate_with_algorithm(&req_secret, PepCombiningAlgorithm::PermitOverrides);
    assert_eq!(dec_permit_overrides.effect, PepDecisionEffect::Permit);
    assert!(dec_permit_overrides.allowed);
    assert_eq!(dec_permit_overrides.matched_rule_id.as_deref(), Some("rule_01_permit"));

    // FirstApplicable: in this service, permit is first
    let dec_first_app = service.evaluate_with_algorithm(&req_secret, PepCombiningAlgorithm::FirstApplicable);
    assert_eq!(dec_first_app.effect, PepDecisionEffect::Permit);
    assert!(dec_first_app.allowed);
    assert_eq!(dec_first_app.matched_rule_id.as_deref(), Some("rule_01_permit"));

    // Unmatched request: default deny
    let req_unmatched = PepRequest::new("agent:guest", "fs:/data/public.txt", "read", None)
        .expect("valid request");
    let dec_unmatched = service.evaluate(&req_unmatched);
    assert_eq!(dec_unmatched.effect, PepDecisionEffect::Deny);
    assert!(!dec_unmatched.allowed);
    assert!(dec_unmatched.matched_rule_id.is_none());
}

#[test]
fn test_pepe2e2_obligation_delivery() {
    let mut service = PepDecisionService::new();

    let obligations = vec![
        PepObligation::AuditLog {
            level: "info".to_string(),
            message: "read performed".to_string(),
        },
        PepObligation::RateLimit {
            key: "agent:auditor".to_string(),
            cost: 5,
        },
    ];

    service.add_rule(PepPolicyRule {
        id: "rule_with_obligations".to_string(),
        target_subject: Some("agent:auditor".to_string()),
        target_resource: Some("audit:logs".to_string()),
        target_action: Some("inspect".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: obligations.clone(),
        description: "Auditor inspection with obligations".to_string(),
    }).expect("add rule");

    let req = PepRequest::new("agent:auditor", "audit:logs", "inspect", None).expect("request");
    let dec = service.evaluate(&req);

    assert!(dec.allowed);
    assert_eq!(dec.obligations.len(), 2);
    match &dec.obligations[0] {
        PepObligation::AuditLog { level, message } => {
            assert_eq!(level, "info");
            assert_eq!(message, "read performed");
        }
        _ => panic!("expected AuditLog obligation"),
    }
    match &dec.obligations[1] {
        PepObligation::RateLimit { key, cost } => {
            assert_eq!(key, "agent:auditor");
            assert_eq!(*cost, 5);
        }
        _ => panic!("expected RateLimit obligation"),
    }
}

#[test]
fn test_pepe2e3_capacity_stress_and_boundary_limits() {
    let mut service = PepDecisionService::new();

    // Populate up to MAX_RULES_IN_SERVICE (5,000)
    for i in 0..MAX_RULES_IN_SERVICE {
        let rule = PepPolicyRule {
            id: format!("rule_{:04}", i),
            target_subject: Some(format!("agent:user_{}", i % 50)),
            target_resource: Some(format!("res:item_{}", i % 100)),
            target_action: Some("read".to_string()),
            effect: PepDecisionEffect::Permit,
            obligations: Vec::new(),
            description: format!("Stress rule {}", i),
        };
        service.add_rule(rule).expect("insert within capacity");
    }

    assert_eq!(service.len(), MAX_RULES_IN_SERVICE);

    // Verify fast indexed evaluation
    let req = PepRequest::new("agent:user_10", "res:item_10", "read", None).expect("valid request");
    let dec = service.evaluate(&req);
    assert!(dec.allowed);

    // Rule 5001 must be rejected with PEPSERV_ERR_CAPACITY
    let overflow_rule = PepPolicyRule {
        id: "rule_overflow".to_string(),
        target_subject: Some("agent:overflow".to_string()),
        target_resource: Some("res:overflow".to_string()),
        target_action: Some("read".to_string()),
        effect: PepDecisionEffect::Deny,
        obligations: Vec::new(),
        description: "Overflow rule".to_string(),
    };
    let err = service.add_rule(overflow_rule).unwrap_err();
    assert!(err.contains(PEPSERV_ERR_CAPACITY));
}

#[test]
fn test_pepe2e4_corrupt_store_fault_injection_and_quarantine() {
    let temp_dir = TestTempDir::new("pep_fault_test");
    let store_path = temp_dir.path().join("policies.json");

    // Write corrupt / truncated JSON
    fs::write(&store_path, b"{\"rules\": [{\"id\": \"unclosed").expect("write corrupt JSON");

    let (recovered_service, recovered, quarantine_path) =
        PepDecisionService::load_or_recover(&store_path);

    assert!(recovered, "expected recovered=true on corrupted store");
    assert!(quarantine_path.is_some(), "expected quarantine path to be returned");

    let q_path = PathBuf::from(quarantine_path.unwrap());
    assert!(q_path.exists(), "quarantine backup file must exist");
    let q_bytes = fs::read(&q_path).expect("read quarantine file");
    assert_eq!(q_bytes, b"{\"rules\": [{\"id\": \"unclosed");

    // Recovered service must be fresh with 0 rules
    assert_eq!(recovered_service.len(), 0);
}

#[test]
fn test_pepe2e5_input_fuzzing_and_path_traversal() {
    // 1. Path traversal in storage paths
    let bad_paths = [
        "../secret.json",
        "foo/../../bar.json",
        "nested/path/../../../etc/passwd.json",
        "null\0byte.json",
        "control\nnewline.json",
        "not_json.txt",
        "not_json.yaml",
        "no_extension",
    ];

    for path_str in &bad_paths {
        let err = validate_pep_service_path(Path::new(path_str));
        assert!(err.is_err(), "expected rejection for '{}'", path_str);
        assert!(err.unwrap_err().contains(PEPSERV_ERR_VALIDATION));
    }

    // 2. Oversized path > 1024 chars
    let long_path = format!("{}.json", "a".repeat(1025));
    let err = validate_pep_service_path(Path::new(&long_path));
    assert!(err.is_err());
    assert!(err.unwrap_err().contains(PEPSERV_ERR_VALIDATION));

    // 3. Request input validation
    assert!(PepRequest::new("", "fs:/path", "read", None).is_err());
    assert!(PepRequest::new("agent:ok", "", "read", None).is_err());
    assert!(PepRequest::new("agent:ok", "fs:/path", "", None).is_err());
    assert!(PepRequest::new("agent:\nnewline", "fs:/path", "read", None).is_err());
    assert!(PepRequest::new("agent:ok", "fs:/../traversal", "read", None).is_err());
}

#[test]
fn test_pepe2e6_cross_surface_persistence_and_json_parity() {
    let temp_dir = TestTempDir::new("pep_persistence_test");
    let store_path = temp_dir.path().join("policies.json");

    let mut service = PepDecisionService::new();
    service.add_rule(PepPolicyRule {
        id: "rule_alpha".to_string(),
        target_subject: Some("agent:alpha".to_string()),
        target_resource: Some("net:tcp:443".to_string()),
        target_action: Some("connect".to_string()),
        effect: PepDecisionEffect::Permit,
        obligations: vec![PepObligation::AuditLog {
            level: "info".to_string(),
            message: "tls connect".to_string(),
        }],
        description: "Permit alpha connect".to_string(),
    }).expect("add rule");

    service.add_rule(PepPolicyRule {
        id: "rule_beta".to_string(),
        target_subject: Some("agent:beta".to_string()),
        target_resource: Some("net:tcp:80".to_string()),
        target_action: Some("connect".to_string()),
        effect: PepDecisionEffect::Deny,
        obligations: Vec::new(),
        description: "Deny beta plaintext".to_string(),
    }).expect("add rule");

    // Save to path
    service.save_to_path(&store_path).expect("save to path");
    assert!(store_path.exists());

    // Load back
    let loaded = PepDecisionService::load_from_path(&store_path).expect("load from path");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded.get_rule("rule_alpha").unwrap().effect, PepDecisionEffect::Permit);
    assert_eq!(loaded.get_rule("rule_beta").unwrap().effect, PepDecisionEffect::Deny);

    // Verify evaluation matches
    let req = PepRequest::new("agent:alpha", "net:tcp:443", "connect", None).expect("request");
    let dec = loaded.evaluate(&req);
    assert!(dec.allowed);
    assert_eq!(dec.matched_rule_id.as_deref(), Some("rule_alpha"));
}
