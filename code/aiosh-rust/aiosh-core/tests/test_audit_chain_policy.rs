//! Integration tests for Audit Chain Security Policy (T-02364).

use aiosh_core::audit::{AuditRing, AuditRowInput, ExtendedAuditRowInput};
use aiosh_core::audit_chain_ext::{AuditCausalLink, AuditSignature};
use aiosh_core::audit_chain_policy::{
    AuditChainSecurityPolicy, AuditPolicyMode, AUDITPOL_ERR_DENIED, AUDITPOL_ERR_SIGNATURE_REQUIRED,
    AUDITPOL_ERR_VALIDATION,
};
use aiosh_core::audit_chain_service::AuditChainService;

#[test]
fn test_service_with_enforcing_policy_blocks_prohibited_actor() {
    let ring = AuditRing::open_in_memory().expect("open ring");
    let mut service = AuditChainService::new(ring);

    let mut base = AuditRowInput::default();
    base.actor = "untrusted".into();
    base.tool = "aios.read".into();
    base.command = "read".into();

    let input = ExtendedAuditRowInput::new(base);
    let result = service.record_event(input);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains(AUDITPOL_ERR_DENIED));
}

#[test]
fn test_service_with_signature_required_policy() {
    let ring = AuditRing::open_in_memory().expect("open ring");
    let mut service = AuditChainService::new(ring);

    let mut base = AuditRowInput::default();
    base.actor = "authorized_agent".into();
    base.tool = "kernel:reboot".into();
    base.command = "reboot".into();

    let unsigned_input = ExtendedAuditRowInput::new(base.clone());
    let res = service.record_event(unsigned_input);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(err.contains(AUDITPOL_ERR_SIGNATURE_REQUIRED));

    // Now supply valid signature
    let mut signed_input = ExtendedAuditRowInput::new(base);
    signed_input.signature = Some(AuditSignature::new(
        "ed25519",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
    ));

    let record_res = service.record_event(signed_input);
    assert!(record_res.is_ok());
    let row = record_res.unwrap();
    assert_eq!(row.tool, "kernel:reboot");
}

#[test]
fn test_service_with_permissive_policy() {
    let ring = AuditRing::open_in_memory().expect("open ring");
    let mut service = AuditChainService::new(ring);

    let mut policy = AuditChainSecurityPolicy::default();
    policy.mode = AuditPolicyMode::Permissive;
    service.set_policy(policy);

    let mut base = AuditRowInput::default();
    base.actor = "guest".into(); // prohibited in default list
    base.tool = "aios.read".into();
    base.command = "read".into();

    let input = ExtendedAuditRowInput::new(base);
    let record_res = service.record_event(input);
    assert!(record_res.is_ok());
    let row = record_res.unwrap();
    assert_eq!(row.actor, "guest");
}

#[test]
fn test_service_with_causal_links_policy_limit() {
    let ring = AuditRing::open_in_memory().expect("open ring");
    let mut service = AuditChainService::new(ring);

    let mut policy = AuditChainSecurityPolicy::default();
    policy.max_allowed_causal_links = 2;
    service.set_policy(policy);

    let mut base = AuditRowInput::default();
    base.actor = "agent_valid".into();
    base.tool = "aios.task".into();
    base.command = "run".into();

    let mut input = ExtendedAuditRowInput::new(base);
    let valid_hash = "a".repeat(64);
    input.causal_links = vec![
        AuditCausalLink::new(valid_hash.clone(), "cause1"),
        AuditCausalLink::new(valid_hash.clone(), "cause2"),
        AuditCausalLink::new(valid_hash, "cause3"),
    ];

    let result = service.record_event(input);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains(AUDITPOL_ERR_VALIDATION));
}
