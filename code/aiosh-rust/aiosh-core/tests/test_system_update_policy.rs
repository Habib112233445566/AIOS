//! Unit tests for System Update Security Policy Subsystem (UPOL1..UPOL6).

use std::fs;
use std::path::{Path, PathBuf};

use aiosh_core::system_update::{
    PartitionTarget, UpdateArtifact, UpdateChannel, UpdateManifest,
};
use aiosh_core::system_update_policy::{
    SystemUpdateSecurityPolicy, UpdatePolicyMode,
    UPOL_VALIDATION_ERROR, validate_policy_path,
};

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

fn sample_manifest(version: &str, channel: UpdateChannel, target: PartitionTarget) -> UpdateManifest {
    UpdateManifest {
        update_id: format!("upd-test-{}", version),
        version: version.to_string(),
        channel,
        min_version: None,
        artifacts: vec![UpdateArtifact {
            target,
            file_name: "rootfs.raw".to_string(),
            sha256: "a".repeat(64),
            size_bytes: 10 * 1024 * 1024,
        }],
        signature: Some("mock_ed25519_sig_valid".to_string()),
        release_notes: "Test notes".to_string(),
        published_at: "2026-09-20T12:00:00Z".to_string(),
    }
}

#[test]
fn test_default_policy_valid() {
    let policy = SystemUpdateSecurityPolicy::default();
    assert!(policy.validate().is_ok());
    assert_eq!(policy.mode, UpdatePolicyMode::Enforcing);
    assert_eq!(policy.allowed_channels, vec![UpdateChannel::Stable]);
    assert!(policy.require_signature);
    assert!(policy.disallow_downgrades);
}

#[test]
fn test_policy_validation_boundaries() {
    let mut policy = SystemUpdateSecurityPolicy::default();

    // 1. Empty allowed_channels
    policy.allowed_channels = vec![];
    assert!(policy.validate().unwrap_err().contains(UPOL_VALIDATION_ERROR));
    policy.allowed_channels = vec![UpdateChannel::Stable];

    // 2. Payload size bounds (< 1 MB)
    policy.max_payload_bytes = 1024;
    assert!(policy.validate().unwrap_err().contains(UPOL_VALIDATION_ERROR));
    // (> 10 GB)
    policy.max_payload_bytes = 11 * 1024 * 1024 * 1024;
    assert!(policy.validate().unwrap_err().contains(UPOL_VALIDATION_ERROR));
    policy.max_payload_bytes = 4 * 1024 * 1024 * 1024;

    // 3. Artifact count bounds (0 or > 32)
    policy.max_artifacts_count = 0;
    assert!(policy.validate().unwrap_err().contains(UPOL_VALIDATION_ERROR));
    policy.max_artifacts_count = 33;
    assert!(policy.validate().unwrap_err().contains(UPOL_VALIDATION_ERROR));
    policy.max_artifacts_count = 8;

    // 4. Empty allowed partition targets
    policy.allowed_partition_targets = vec![];
    assert!(policy.validate().unwrap_err().contains(UPOL_VALIDATION_ERROR));
    policy.allowed_partition_targets = vec![PartitionTarget::Rootfs];

    // 5. Trusted public key hygiene
    policy.trusted_public_keys = vec!["key with spaces".to_string()];
    assert!(policy.validate().unwrap_err().contains(UPOL_VALIDATION_ERROR));
    policy.trusted_public_keys = vec!["k".repeat(257)];
    assert!(policy.validate().unwrap_err().contains(UPOL_VALIDATION_ERROR));
    policy.trusted_public_keys = vec!["valid_key_001".to_string()];
    assert!(policy.validate().is_ok());

    // 6. Revoked version hygiene
    policy.revoked_versions = vec!["v".repeat(65)];
    assert!(policy.validate().unwrap_err().contains(UPOL_VALIDATION_ERROR));
    policy.revoked_versions = vec!["1.0.0".to_string()];
    assert!(policy.validate().is_ok());
}

#[test]
fn test_upol1_channel_evaluation() {
    let policy = SystemUpdateSecurityPolicy::default(); // allowed: Stable

    // Allowed channel
    let stable_manifest = sample_manifest("2.0.0", UpdateChannel::Stable, PartitionTarget::Rootfs);
    let report = policy.evaluate("1.0.0", &stable_manifest);
    assert_eq!(report.verdict, "allow");
    assert!(report.violations.is_empty());

    // Disallowed channel in Enforcing mode
    let beta_manifest = sample_manifest("2.0.0", UpdateChannel::Beta, PartitionTarget::Rootfs);
    let report_deny = policy.evaluate("1.0.0", &beta_manifest);
    assert_eq!(report_deny.verdict, "deny");
    assert_eq!(report_deny.violations[0].rule_id, "UPOL1_CHANNEL_DISALLOWED");

    // Disallowed channel in Audit mode
    let mut audit_policy = policy.clone();
    audit_policy.mode = UpdatePolicyMode::Audit;
    let report_audit = audit_policy.evaluate("1.0.0", &beta_manifest);
    assert_eq!(report_audit.verdict, "audit");
    assert_eq!(report_audit.violations[0].rule_id, "UPOL1_CHANNEL_DISALLOWED");

    // Disallowed channel in Permissive mode
    let mut perm_policy = policy.clone();
    perm_policy.mode = UpdatePolicyMode::Permissive;
    let report_perm = perm_policy.evaluate("1.0.0", &beta_manifest);
    assert_eq!(report_perm.verdict, "allow");
    assert_eq!(report_perm.violations[0].rule_id, "UPOL1_CHANNEL_DISALLOWED");
}

#[test]
fn test_upol2_signature_evaluation() {
    let policy = SystemUpdateSecurityPolicy::default();

    // Missing signature when required
    let mut unsigned_manifest = sample_manifest("2.0.0", UpdateChannel::Stable, PartitionTarget::Rootfs);
    unsigned_manifest.signature = None;
    let report = policy.evaluate("1.0.0", &unsigned_manifest);
    assert_eq!(report.verdict, "deny");
    assert_eq!(report.violations[0].rule_id, "UPOL2_SIGNATURE_MISSING");

    // Untrusted key
    let mut key_policy = policy.clone();
    key_policy.trusted_public_keys = vec!["prod_key_alpha".to_string()];
    let mut foreign_manifest = sample_manifest("2.0.0", UpdateChannel::Stable, PartitionTarget::Rootfs);
    foreign_manifest.signature = Some("untrusted_key_sig".to_string());
    let report_key = key_policy.evaluate("1.0.0", &foreign_manifest);
    assert_eq!(report_key.verdict, "deny");
    assert_eq!(report_key.violations[0].rule_id, "UPOL2_KEY_UNTRUSTED");
}

#[test]
fn test_upol3_anti_rollback_downgrade() {
    let policy = SystemUpdateSecurityPolicy::default();

    // Normal upgrade: 1.0.0 -> 2.0.0
    let upgrade_manifest = sample_manifest("2.0.0", UpdateChannel::Stable, PartitionTarget::Rootfs);
    let report = policy.evaluate("1.0.0", &upgrade_manifest);
    assert_eq!(report.verdict, "allow");

    // Same version: 2.0.0 -> 2.0.0
    let same_manifest = sample_manifest("2.0.0", UpdateChannel::Stable, PartitionTarget::Rootfs);
    let report_same = policy.evaluate("2.0.0", &same_manifest);
    assert_eq!(report_same.verdict, "allow");

    // Downgrade: 2.1.0 -> 2.0.1
    let downgrade_manifest = sample_manifest("2.0.1", UpdateChannel::Stable, PartitionTarget::Rootfs);
    let report_down = policy.evaluate("2.1.0", &downgrade_manifest);
    assert_eq!(report_down.verdict, "deny");
    assert_eq!(report_down.violations[0].rule_id, "UPOL3_DOWNGRADE_ATTEMPT");

    // Downgrade allowed when disallow_downgrades is false
    let mut allow_down_policy = policy.clone();
    allow_down_policy.disallow_downgrades = false;
    let report_allowed = allow_down_policy.evaluate("2.1.0", &downgrade_manifest);
    assert_eq!(report_allowed.verdict, "allow");
}

#[test]
fn test_upol4_partition_target_governance() {
    let mut policy = SystemUpdateSecurityPolicy::default();
    policy.allowed_partition_targets = vec![PartitionTarget::Rootfs, PartitionTarget::Kernel];
    policy.required_partition_targets = vec![PartitionTarget::Rootfs];

    // Disallowed target (Initramfs)
    let bad_target_manifest = sample_manifest("2.0.0", UpdateChannel::Stable, PartitionTarget::Initramfs);
    let report = policy.evaluate("1.0.0", &bad_target_manifest);
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "UPOL4_TARGET_DISALLOWED"));
    // Also missing required Rootfs
    assert!(report.violations.iter().any(|v| v.rule_id == "UPOL4_REQUIRED_TARGET_MISSING"));
}

#[test]
fn test_upol5_quota_and_resource_caps() {
    let mut policy = SystemUpdateSecurityPolicy::default();
    policy.max_payload_bytes = 5 * 1024 * 1024; // 5 MB
    policy.max_artifacts_count = 1;

    // Payload too large (10 MB > 5 MB)
    let large_manifest = sample_manifest("2.0.0", UpdateChannel::Stable, PartitionTarget::Rootfs);
    let report = policy.evaluate("1.0.0", &large_manifest);
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "UPOL5_PAYLOAD_EXCEEDED"));

    // Artifact count exceeded (increase max_payload_bytes so only artifact count triggers)
    policy.max_payload_bytes = 50 * 1024 * 1024;
    let mut multi_manifest = sample_manifest("2.0.0", UpdateChannel::Stable, PartitionTarget::Rootfs);
    multi_manifest.artifacts.push(UpdateArtifact {
        target: PartitionTarget::Kernel,
        file_name: "vmlinuz".to_string(),
        sha256: "b".repeat(64),
        size_bytes: 1024,
    });
    let report_count = policy.evaluate("1.0.0", &multi_manifest);
    assert_eq!(report_count.verdict, "deny");
    assert!(report_count.violations.iter().any(|v| v.rule_id == "UPOL5_ARTIFACT_COUNT_EXCEEDED"));
}

#[test]
fn test_upol6_revocation_denylist() {
    let mut policy = SystemUpdateSecurityPolicy::default();
    policy.revoked_versions = vec!["2.0.1".to_string()];
    policy.revoked_update_ids = vec!["upd-cve-2026-001".to_string()];

    // Revoked version
    let revoked_ver_manifest = sample_manifest("2.0.1", UpdateChannel::Stable, PartitionTarget::Rootfs);
    let report = policy.evaluate("1.0.0", &revoked_ver_manifest);
    assert_eq!(report.verdict, "deny");
    assert_eq!(report.violations[0].rule_id, "UPOL6_VERSION_REVOKED");

    // Revoked update ID
    let mut revoked_id_manifest = sample_manifest("2.0.2", UpdateChannel::Stable, PartitionTarget::Rootfs);
    revoked_id_manifest.update_id = "upd-cve-2026-001".to_string();
    let report_id = policy.evaluate("1.0.0", &revoked_id_manifest);
    assert_eq!(report_id.verdict, "deny");
    assert_eq!(report_id.violations[0].rule_id, "UPOL6_UPDATE_ID_REVOKED");
}

#[test]
fn test_policy_file_persistence_and_hygiene() {
    // Path validation tests
    assert!(validate_policy_path(Path::new("")).is_err());
    assert!(validate_policy_path(Path::new("subdir/../etc")).is_err());
    assert!(validate_policy_path(Path::new("path\nwith\nnewline")).is_err());

    let temp = TestTempDir::new("aiosh_update_policy_test");
    let file_path = temp.path().join("update_policy.json");

    let policy = SystemUpdateSecurityPolicy::default();
    assert!(policy.save_to_file(&file_path).is_ok());
    assert!(file_path.exists());

    let loaded = SystemUpdateSecurityPolicy::from_file(&file_path).unwrap();
    assert_eq!(loaded, policy);
}
