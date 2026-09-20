//! Unit tests for Capability Model Observability Subsystem (T-02075).
//!
//! Tests report generation on empty and populated registries, quota aggregation,
//! depth calculation, scope/rights distributions, health evaluation, and JSON serialization.

use chrono::{Duration, Utc};

use aiosh_core::capability::{
    CapabilityConstraints, CapabilityRight, CapabilityScope,
};
use aiosh_core::capability_observability::{
    sanitize_telemetry_text, CapabilityObservabilityReport,
};
use aiosh_core::capability_policy::{CapabilityPolicyMode, CapabilitySecurityPolicy};
use aiosh_core::capability_service::CapabilityService;

#[test]
fn test_observability_empty_registry() {
    let service = CapabilityService::new();
    let report = CapabilityObservabilityReport::generate(&service, "2026-09-20T18:00:00Z");

    assert_eq!(report.total_capabilities, 0);
    assert_eq!(report.active_capabilities, 0);
    assert_eq!(report.revoked_capabilities, 0);
    assert_eq!(report.expired_capabilities, 0);
    assert_eq!(report.root_capabilities, 0);
    assert_eq!(report.attenuated_capabilities, 0);
    assert_eq!(report.max_derivation_depth, 0);
    assert_eq!(report.unique_subjects_count, 0);
    assert_eq!(report.unique_issuers_count, 0);
    assert_eq!(report.total_invocations_consumed, 0);
    assert_eq!(report.total_bytes_consumed, 0);
    assert!(report.capabilities_by_scope_type.is_empty());
    assert!(report.capabilities_by_right.is_empty());
    assert_eq!(report.capacity_utilization_percent, 0);
    assert!(report.is_healthy);
    assert_eq!(report.generated_at, "2026-09-20T18:00:00Z");
}

#[test]
fn test_observability_populated_registry() {
    let mut service = CapabilityService::new();

    // 1. Issue root filesystem capability
    let root1 = service
        .issue_root_capability(
            "kernel",
            "agent:worker_1",
            CapabilityScope::Filesystem {
                path: "/workspace/data".into(),
                recursive: true,
            },
            vec![CapabilityRight::Read, CapabilityRight::Write, CapabilityRight::Delegate],
            CapabilityConstraints {
                not_before: None,
                expires_at: Some((Utc::now() + Duration::hours(2)).to_rfc3339()),
                max_invocations: Some(100),
                current_invocations: 15,
                quota_bytes: Some(10000),
                consumed_bytes: 2500,
            },
        )
        .expect("issue root1");

    // 2. Issue root network capability
    let _root2 = service
        .issue_root_capability(
            "kernel",
            "agent:worker_2",
            CapabilityScope::Network {
                host: "api.internal".into(),
                port: Some(443),
                protocol: "tcp".into(),
            },
            vec![CapabilityRight::Read],
            CapabilityConstraints {
                not_before: None,
                expires_at: Some((Utc::now() + Duration::hours(1)).to_rfc3339()),
                max_invocations: None,
                current_invocations: 5,
                quota_bytes: None,
                consumed_bytes: 500,
            },
        )
        .expect("issue root2");

    // 3. Attenuate from root1 (depth 1) with fresh counters
    let child1 = service
        .attenuate_capability(
            &root1.id,
            "agent:subworker",
            None,
            vec![CapabilityRight::Read, CapabilityRight::Delegate],
            Some(CapabilityConstraints {
                not_before: None,
                expires_at: None,
                max_invocations: Some(50),
                current_invocations: 0,
                quota_bytes: Some(5000),
                consumed_bytes: 0,
            }),
        )
        .expect("attenuate child1");

    // 4. Attenuate from child1 (depth 2) with fresh counters
    let _child2 = service
        .attenuate_capability(
            &child1.id,
            "agent:leaf",
            None,
            vec![CapabilityRight::Read],
            Some(CapabilityConstraints {
                not_before: None,
                expires_at: None,
                max_invocations: Some(25),
                current_invocations: 0,
                quota_bytes: Some(2500),
                consumed_bytes: 0,
            }),
        )
        .expect("attenuate child2");

    let report = CapabilityObservabilityReport::generate(&service, "");

    assert_eq!(report.total_capabilities, 4);
    assert_eq!(report.root_capabilities, 2);
    assert_eq!(report.attenuated_capabilities, 2);
    assert_eq!(report.max_derivation_depth, 2);
    assert_eq!(report.unique_subjects_count, 4);
    assert_eq!(report.unique_issuers_count, 3);

    // Quota aggregation: 15 + 5 = 20 invocations, 2500 + 500 = 3000 bytes
    assert_eq!(report.total_invocations_consumed, 20);
    assert_eq!(report.total_bytes_consumed, 3000);

    // Scope breakdown: 3 filesystem (root1, child1, child2), 1 network (root2)
    assert_eq!(report.capabilities_by_scope_type.get("filesystem"), Some(&3));
    assert_eq!(report.capabilities_by_scope_type.get("network"), Some(&1));

    // Rights breakdown
    assert!(report.capabilities_by_right.get("read").unwrap_or(&0) >= &4);
    assert_eq!(report.capabilities_by_right.get("write"), Some(&1));
    assert_eq!(report.capabilities_by_right.get("delegate"), Some(&2));

    assert!(report.is_healthy);
}

#[test]
fn test_observability_revocation_and_expiration() {
    let mut service = CapabilityService::new();

    // Active capability
    let cap_active = service
        .issue_root_capability(
            "kernel",
            "agent:active",
            CapabilityScope::System { subsystem: "core".into() },
            vec![CapabilityRight::Read],
            CapabilityConstraints::default(),
        )
        .expect("cap_active");

    // Expired capability
    let _cap_expired = service
        .issue_root_capability(
            "kernel",
            "agent:expired",
            CapabilityScope::System { subsystem: "core".into() },
            vec![CapabilityRight::Read],
            CapabilityConstraints {
                not_before: None,
                expires_at: Some((Utc::now() - Duration::hours(1)).to_rfc3339()),
                max_invocations: None,
                current_invocations: 0,
                quota_bytes: None,
                consumed_bytes: 0,
            },
        )
        .expect("cap_expired");

    // Revoked capability
    let cap_to_revoke = service
        .issue_root_capability(
            "kernel",
            "agent:to_revoke",
            CapabilityScope::System { subsystem: "core".into() },
            vec![CapabilityRight::Read],
            CapabilityConstraints::default(),
        )
        .expect("cap_to_revoke");
    service.revoke_capability(&cap_to_revoke.id).expect("revoke");

    let report = CapabilityObservabilityReport::generate(&service, "");

    assert_eq!(report.total_capabilities, 3);
    assert_eq!(report.active_capabilities, 1);
    assert_eq!(report.expired_capabilities, 1);
    assert_eq!(report.revoked_capabilities, 1);

    // Verify ID matches
    assert_eq!(service.get_capability(&cap_active.id).map(|c| c.revoked), Some(false));
}

#[test]
fn test_observability_health_evaluation() {
    // 1. Healthy service
    let service_healthy = CapabilityService::new();
    let report_healthy = CapabilityObservabilityReport::generate(&service_healthy, "");
    assert!(report_healthy.is_healthy);
    assert_eq!(report_healthy.policy_mode, CapabilityPolicyMode::Enforcing);

    // 2. Unhealthy due to depth exceeding policy
    let mut custom_policy = CapabilitySecurityPolicy::default();
    custom_policy.max_attenuation_depth = 1;
    // Set permissive so depth 2 can be created
    custom_policy.mode = CapabilityPolicyMode::Permissive;

    let mut service_deep = CapabilityService::new().with_policy(custom_policy);
    let root = service_deep
        .issue_root_capability(
            "kernel",
            "agent:root",
            CapabilityScope::Filesystem {
                path: "/data".into(),
                recursive: true,
            },
            vec![CapabilityRight::Read, CapabilityRight::Delegate],
            CapabilityConstraints::default(),
        )
        .expect("root");

    let child1 = service_deep
        .attenuate_capability(
            &root.id,
            "agent:child1",
            None,
            vec![CapabilityRight::Read, CapabilityRight::Delegate],
            None,
        )
        .expect("child1");

    let _child2 = service_deep
        .attenuate_capability(
            &child1.id,
            "agent:child2",
            None,
            vec![CapabilityRight::Read],
            None,
        )
        .expect("child2");

    let report_deep = CapabilityObservabilityReport::generate(&service_deep, "");
    assert_eq!(report_deep.max_derivation_depth, 2);
    // Unhealthy because max_derivation_depth (2) > policy max_attenuation_depth (1)
    assert!(!report_deep.is_healthy);
}

#[test]
fn test_observability_json_serde_and_sanitization() {
    let service = CapabilityService::new();
    let report = CapabilityObservabilityReport::generate(&service, "2026-09-20T18:00:00Z");

    let json_str = report.to_json().expect("serialize to json");
    assert!(json_str.contains("\"total_capabilities\": 0"));
    assert!(json_str.contains("\"is_healthy\": true"));

    let deserialized: CapabilityObservabilityReport =
        serde_json::from_str(&json_str).expect("deserialize from json");
    assert_eq!(report, deserialized);

    // Sanitization test
    let dirty = "  dirty\x00\x07string\r\n  ";
    let cleaned = sanitize_telemetry_text(dirty);
    assert_eq!(cleaned, "dirtystring");

    // Length cap at 256
    let long_str = "a".repeat(500);
    let capped = sanitize_telemetry_text(&long_str);
    assert_eq!(capped.len(), 256);
}

#[test]
fn test_observability_hardening_edge_cases() {
    let service = CapabilityService::new();
    // 1. All-control-character timestamp should fall back to valid RFC3339 now()
    let report_control_ts = CapabilityObservabilityReport::generate(&service, "\x00\x01\x02\t\r\n");
    assert!(!report_control_ts.generated_at.is_empty());
    assert!(chrono::DateTime::parse_from_rfc3339(&report_control_ts.generated_at).is_ok());

    // 2. Whitespace-only timestamp should also fall back
    let report_ws_ts = CapabilityObservabilityReport::generate(&service, "   \t\n   ");
    assert!(!report_ws_ts.generated_at.is_empty());
    assert!(chrono::DateTime::parse_from_rfc3339(&report_ws_ts.generated_at).is_ok());
}

