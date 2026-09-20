//! Unit tests for System Update Observability Subsystem (UOBS1..UOBS6).

use tempfile::tempdir;
use sha2::{Digest, Sha256};

use aiosh_core::system_update::{
    PartitionTarget, UpdateArtifact, UpdateChannel, UpdateManifest, UpdateSlot, UpdateState,
};
use aiosh_core::system_update_service::{SystemUpdateService, SystemUpdateServiceConfig};
use aiosh_core::system_update_policy::{SystemUpdateSecurityPolicy, UpdatePolicyMode};
use aiosh_core::system_update_observability::{
    sanitize_telemetry_text, SystemUpdateObservabilityReport,
};

fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

fn create_sample_manifest(rootfs_data: &[u8], kernel_data: &[u8]) -> UpdateManifest {
    UpdateManifest {
        update_id: "upd-obs-test-001".to_string(),
        version: "2.5.0".to_string(),
        channel: UpdateChannel::Stable,
        min_version: Some("2.0.0".to_string()),
        artifacts: vec![
            UpdateArtifact {
                target: PartitionTarget::Rootfs,
                file_name: "rootfs.raw".to_string(),
                sha256: compute_sha256(rootfs_data),
                size_bytes: rootfs_data.len() as u64,
            },
            UpdateArtifact {
                target: PartitionTarget::Kernel,
                file_name: "vmlinuz.efi".to_string(),
                sha256: compute_sha256(kernel_data),
                size_bytes: kernel_data.len() as u64,
            },
        ],
        signature: Some("ed25519:test_sig".to_string()),
        release_notes: "Observability test release".to_string(),
        published_at: "2026-09-20T12:00:00Z".to_string(),
    }
}

#[test]
fn test_uobs1_default_report_generation() {
    let service = SystemUpdateService::with_defaults("1.0.0", UpdateSlot::SlotA, "2026-09-20T12:00:00Z");
    let report = SystemUpdateObservabilityReport::generate(&service, None, "2026-09-20T14:00:00Z");

    assert_eq!(report.current_slot, UpdateSlot::SlotA);
    assert_eq!(report.target_slot, UpdateSlot::SlotB);
    assert_eq!(report.rollback_slot, Some(UpdateSlot::SlotA));
    assert_eq!(report.slot_a_version, "1.0.0");
    assert_eq!(report.slot_b_version, "none");
    assert!(report.slot_a_successful);
    assert!(!report.slot_b_successful);
    assert_eq!(report.state, UpdateState::Idle);
    assert_eq!(report.progress_percent, 0);
    assert_eq!(report.current_version, "1.0.0");
    assert_eq!(report.target_version, None);
    assert_eq!(report.last_error, None);
    assert_eq!(report.update_id, None);
    assert_eq!(report.channel, None);
    assert_eq!(report.staged_artifacts_count, 0);
    assert_eq!(report.staged_payload_bytes, 0);
    assert_eq!(report.manifest_total_bytes, None);
    assert_eq!(report.policy_verdict, None);
    assert_eq!(report.policy_violations_count, 0);
    assert_eq!(report.policy_mode, None);
    assert!(report.is_healthy);
    assert_eq!(report.generated_at, "2026-09-20T14:00:00Z");
}

#[test]
fn test_uobs2_report_with_active_manifest_and_staged_payload() {
    let dir = tempdir().unwrap();
    let state_dir = dir.path().join("state");
    let staging_dir = dir.path().join("staging");

    let config = SystemUpdateServiceConfig {
        state_dir,
        staging_dir,
        max_payload_bytes: 10 * 1024 * 1024,
        auto_rollback_on_failure: true,
    };

    let mut service = SystemUpdateService::new(
        "2.0.0",
        UpdateSlot::SlotA,
        config,
        "2026-09-20T12:00:00Z",
    );

    let rootfs_data = b"MOCK_ROOTFS_OBS_PAYLOAD";
    let kernel_data = b"MOCK_KERNEL_OBS_PAYLOAD";
    let manifest = create_sample_manifest(rootfs_data, kernel_data);
    let total_bytes = manifest.total_bytes();

    service.check_manifest(manifest).unwrap();
    service.stage_artifact(PartitionTarget::Rootfs, rootfs_data).unwrap();
    service.stage_artifact(PartitionTarget::Kernel, kernel_data).unwrap();

    let report = SystemUpdateObservabilityReport::generate(&service, None, "2026-09-20T14:00:00Z");

    assert_eq!(report.state, UpdateState::Downloading);
    assert_eq!(report.target_version, Some("2.5.0".to_string()));
    assert_eq!(report.update_id, Some("upd-obs-test-001".to_string()));
    assert_eq!(report.channel, Some(UpdateChannel::Stable));
    assert_eq!(report.staged_artifacts_count, 2);
    assert_eq!(report.staged_payload_bytes, total_bytes);
    assert_eq!(report.manifest_total_bytes, Some(total_bytes));
    assert!(report.is_healthy);
}

#[test]
fn test_uobs3_report_with_security_policy_evaluated() {
    let service = SystemUpdateService::with_defaults("2.0.0", UpdateSlot::SlotA, "2026-09-20T12:00:00Z");
    let mut policy = SystemUpdateSecurityPolicy::default();
    policy.mode = UpdatePolicyMode::Enforcing;

    // With no manifest active in service
    let report_no_manifest = SystemUpdateObservabilityReport::generate(&service, Some(&policy), "2026-09-20T14:00:00Z");
    assert_eq!(report_no_manifest.policy_verdict, Some("not_evaluated".to_string()));
    assert_eq!(report_no_manifest.policy_violations_count, 0);
    assert_eq!(report_no_manifest.policy_mode, Some(UpdatePolicyMode::Enforcing));

    // Now with active manifest in service
    let dir = tempdir().unwrap();
    let config = SystemUpdateServiceConfig {
        state_dir: dir.path().join("state"),
        staging_dir: dir.path().join("staging"),
        max_payload_bytes: 10 * 1024 * 1024,
        auto_rollback_on_failure: true,
    };
    let mut service_with_manifest = SystemUpdateService::new(
        "2.0.0",
        UpdateSlot::SlotA,
        config,
        "2026-09-20T12:00:00Z",
    );

    let rootfs_data = b"ROOTFS";
    let kernel_data = b"KERNEL";
    let manifest = create_sample_manifest(rootfs_data, kernel_data);
    service_with_manifest.active_manifest = Some(manifest);

    let report_with_manifest = SystemUpdateObservabilityReport::generate(&service_with_manifest, Some(&policy), "2026-09-20T14:00:00Z");
    assert_eq!(report_with_manifest.policy_verdict, Some("allow".to_string()));
    assert_eq!(report_with_manifest.policy_violations_count, 0);
    assert_eq!(report_with_manifest.policy_mode, Some(UpdatePolicyMode::Enforcing));
}

#[test]
fn test_uobs4_telemetry_sanitization() {
    // Direct sanitization checks
    assert_eq!(sanitize_telemetry_text("  clean_string  "), "clean_string");
    assert_eq!(sanitize_telemetry_text("line1\nline2\rline3\x00null\x1b[31mcolor"), "line1line2line3null[31mcolor");

    let long_str = "A".repeat(500);
    let sanitized = sanitize_telemetry_text(&long_str);
    assert_eq!(sanitized.len(), 256);

    // Sanitization via report generation
    let mut service = SystemUpdateService::with_defaults("1.0.0\r\n", UpdateSlot::SlotA, "2026-09-20T12:00:00Z");
    service.update_status.last_error = Some("Failed with error:\x00\x1b[31mbad\x1b[0m\n".to_string());
    service.update_status.target_version = Some("  2.0.0-beta\t".to_string());

    let report = SystemUpdateObservabilityReport::generate(&service, None, "2026-09-20T14:00:00Z\n");
    assert_eq!(report.current_version, "1.0.0");
    assert_eq!(report.last_error, Some("Failed with error:[31mbad[0m".to_string()));
    assert_eq!(report.target_version, Some("2.0.0-beta".to_string()));
    assert_eq!(report.generated_at, "2026-09-20T14:00:00Z");
}

#[test]
fn test_uobs5_health_computation() {
    // Healthy slot A
    let mut service = SystemUpdateService::with_defaults("1.0.0", UpdateSlot::SlotA, "2026-09-20T12:00:00Z");
    let report = SystemUpdateObservabilityReport::generate(&service, None, "2026-09-20T14:00:00Z");
    assert!(report.is_healthy);

    // Unhealthy slot A (marked not successful)
    service.slot_status.slot_a_successful = false;
    let report_unhealthy_slot = SystemUpdateObservabilityReport::generate(&service, None, "2026-09-20T14:00:00Z");
    assert!(!report_unhealthy_slot.is_healthy);

    // Current slot B successful vs unsuccessful
    service.slot_status.current_slot = UpdateSlot::SlotB;
    service.slot_status.slot_b_successful = false;
    let report_b_fail = SystemUpdateObservabilityReport::generate(&service, None, "2026-09-20T14:00:00Z");
    assert!(!report_b_fail.is_healthy);

    service.slot_status.slot_b_successful = true;
    let report_b_pass = SystemUpdateObservabilityReport::generate(&service, None, "2026-09-20T14:00:00Z");
    assert!(report_b_pass.is_healthy);

    // UpdateState::Failed makes it unhealthy regardless of slot
    service.update_status.state = UpdateState::Failed;
    let report_failed_state = SystemUpdateObservabilityReport::generate(&service, None, "2026-09-20T14:00:00Z");
    assert!(!report_failed_state.is_healthy);
}

#[test]
fn test_uobs6_json_serialization_roundtrip() {
    let service = SystemUpdateService::with_defaults("1.2.3", UpdateSlot::SlotA, "2026-09-20T12:00:00Z");
    let report = SystemUpdateObservabilityReport::generate(&service, None, "2026-09-20T14:00:00Z");

    let json_str = report.to_json().expect("serialization should succeed");
    assert!(json_str.contains("\"current_slot\": \"slot_a\""));
    assert!(json_str.contains("\"is_healthy\": true"));

    let deserialized: SystemUpdateObservabilityReport =
        serde_json::from_str(&json_str).expect("deserialization should succeed");
    assert_eq!(report, deserialized);
}

#[test]
fn test_uobs7_progress_clamping() {
    let mut service = SystemUpdateService::with_defaults("1.0.0", UpdateSlot::SlotA, "2026-09-20T12:00:00Z");
    service.update_status.progress_percent = 250;

    let report = SystemUpdateObservabilityReport::generate(&service, None, "2026-09-20T14:00:00Z");
    assert_eq!(report.progress_percent, 100);
}
