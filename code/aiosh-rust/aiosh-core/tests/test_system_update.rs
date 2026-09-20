//! Unit tests for System Update Mechanism Data Model (UPD1..UPD6).

use aiosh_core::system_update::{
    PartitionTarget, SystemSlotStatus, SystemUpdateStatus, UpdateArtifact,
    UpdateChannel, UpdateManifest, UpdateSlot, UpdateState,
    MAX_UPDATE_ID_LEN, MAX_UPDATE_PAYLOAD_SIZE, MAX_UPDATE_VERSION_LEN,
    UPD_DIGEST_ERROR, UPD_SLOT_ERROR, UPD_STATE_ERROR, UPD_VALIDATION_ERROR,
};

#[test]
fn test_upd1_slot_toggle_and_parsing() {
    // Other slot toggle
    assert_eq!(UpdateSlot::SlotA.other(), UpdateSlot::SlotB);
    assert_eq!(UpdateSlot::SlotB.other(), UpdateSlot::SlotA);
    assert_eq!(UpdateSlot::SlotA.as_str(), "slot_a");
    assert_eq!(UpdateSlot::SlotB.as_str(), "slot_b");

    // Loose string parsing
    assert_eq!(UpdateSlot::from_str_loose("slot_a"), Some(UpdateSlot::SlotA));
    assert_eq!(UpdateSlot::from_str_loose("A"), Some(UpdateSlot::SlotA));
    assert_eq!(UpdateSlot::from_str_loose("slota"), Some(UpdateSlot::SlotA));
    assert_eq!(UpdateSlot::from_str_loose("slot0"), Some(UpdateSlot::SlotA));

    assert_eq!(UpdateSlot::from_str_loose("slot_b"), Some(UpdateSlot::SlotB));
    assert_eq!(UpdateSlot::from_str_loose("B"), Some(UpdateSlot::SlotB));
    assert_eq!(UpdateSlot::from_str_loose("slotb"), Some(UpdateSlot::SlotB));
    assert_eq!(UpdateSlot::from_str_loose("slot1"), Some(UpdateSlot::SlotB));

    assert_eq!(UpdateSlot::from_str_loose("slot_c"), None);
    assert_eq!(UpdateSlot::from_str_loose(""), None);
}

#[test]
fn test_upd1_slot_status_lifecycle() {
    let mut slots = SystemSlotStatus::new(UpdateSlot::SlotA, "1.0.0");
    assert_eq!(slots.current_slot, UpdateSlot::SlotA);
    assert_eq!(slots.target_slot, UpdateSlot::SlotB);
    assert_eq!(slots.rollback_slot, Some(UpdateSlot::SlotA));
    assert_eq!(slots.slot_a_version, "1.0.0");
    assert_eq!(slots.slot_b_version, "none");
    assert!(slots.slot_a_successful);
    assert!(!slots.slot_b_successful);
    assert!(slots.validate().is_ok());

    // Switch slot
    slots.switch_slot();
    assert_eq!(slots.current_slot, UpdateSlot::SlotB);
    assert_eq!(slots.target_slot, UpdateSlot::SlotA);
    assert_eq!(slots.rollback_slot, Some(UpdateSlot::SlotA));

    // Mark slot B successful with new version
    slots.mark_slot_success(UpdateSlot::SlotB, "1.1.0");
    assert!(slots.slot_b_successful);
    assert_eq!(slots.slot_b_version, "1.1.0");

    // Invariant violation: identical current and target slot
    slots.target_slot = UpdateSlot::SlotB;
    let err = slots.validate().unwrap_err();
    assert!(err.contains(UPD_SLOT_ERROR));
}

#[test]
fn test_upd2_channel_parsing_and_serde() {
    assert_eq!(UpdateChannel::from_str_loose("stable"), Some(UpdateChannel::Stable));
    assert_eq!(UpdateChannel::from_str_loose("prod"), Some(UpdateChannel::Stable));
    assert_eq!(UpdateChannel::from_str_loose("beta"), Some(UpdateChannel::Beta));
    assert_eq!(UpdateChannel::from_str_loose("nightly"), Some(UpdateChannel::Nightly));
    assert_eq!(UpdateChannel::from_str_loose("dev"), Some(UpdateChannel::Development));
    assert_eq!(UpdateChannel::from_str_loose("unknown"), None);

    assert_eq!(UpdateChannel::Stable.as_str(), "stable");
    assert_eq!(UpdateChannel::Beta.as_str(), "beta");
    assert_eq!(UpdateChannel::Nightly.as_str(), "nightly");
    assert_eq!(UpdateChannel::Development.as_str(), "development");

    let json = serde_json::to_string(&UpdateChannel::Beta).unwrap();
    assert_eq!(json, "\"beta\"");
    let decoded: UpdateChannel = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, UpdateChannel::Beta);
}

#[test]
fn test_upd3_artifact_validation() {
    let valid_sha = "a".repeat(64);
    let valid_artifact = UpdateArtifact {
        target: PartitionTarget::Rootfs,
        file_name: "rootfs.img".to_string(),
        sha256: valid_sha.clone(),
        size_bytes: 1024 * 1024 * 500, // 500 MB
    };
    assert!(valid_artifact.validate().is_ok());

    // Empty file name
    let mut bad_artifact = valid_artifact.clone();
    bad_artifact.file_name = "".to_string();
    assert!(bad_artifact.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));

    // Control characters in file name
    bad_artifact.file_name = "rootfs\n.img".to_string();
    assert!(bad_artifact.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));

    // Path traversal in file name
    bad_artifact.file_name = "../rootfs.img".to_string();
    assert!(bad_artifact.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));
    bad_artifact.file_name = "subdir/rootfs.img".to_string();
    assert!(bad_artifact.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));
    bad_artifact.file_name = "subdir\\rootfs.img".to_string();
    assert!(bad_artifact.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));

    // Hidden file name
    bad_artifact.file_name = ".rootfs.img".to_string();
    assert!(bad_artifact.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));

    // File name with whitespace
    bad_artifact.file_name = "rootfs image.img".to_string();
    assert!(bad_artifact.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));

    // Invalid SHA-256 digest (length != 64)
    bad_artifact.file_name = "rootfs.img".to_string();
    bad_artifact.sha256 = "abc".to_string();
    assert!(bad_artifact.validate().unwrap_err().contains(UPD_DIGEST_ERROR));

    // Invalid SHA-256 digest (non-hex character)
    bad_artifact.sha256 = format!("{}z", &valid_sha[..63]);
    assert!(bad_artifact.validate().unwrap_err().contains(UPD_DIGEST_ERROR));

    // Zero size
    bad_artifact.sha256 = valid_sha.clone();
    bad_artifact.size_bytes = 0;
    assert!(bad_artifact.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));

    // Exceeds max payload size (10 GB)
    bad_artifact.size_bytes = MAX_UPDATE_PAYLOAD_SIZE + 1;
    assert!(bad_artifact.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));
}

#[test]
fn test_upd3_manifest_validation_and_helpers() {
    let valid_sha = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string();
    let rootfs_artifact = UpdateArtifact {
        target: PartitionTarget::Rootfs,
        file_name: "rootfs.raw".to_string(),
        sha256: valid_sha.clone(),
        size_bytes: 400 * 1024 * 1024,
    };
    let kernel_artifact = UpdateArtifact {
        target: PartitionTarget::Kernel,
        file_name: "vmlinuz".to_string(),
        sha256: valid_sha.clone(),
        size_bytes: 12 * 1024 * 1024,
    };

    let manifest = UpdateManifest {
        update_id: "upd-2026-09-20-01".to_string(),
        version: "2.1.0".to_string(),
        channel: UpdateChannel::Stable,
        min_version: Some("2.0.0".to_string()),
        artifacts: vec![rootfs_artifact.clone(), kernel_artifact.clone()],
        signature: Some("sig_mock".to_string()),
        release_notes: "Routine security fixes".to_string(),
        published_at: "2026-09-20T00:00:00Z".to_string(),
    };

    assert!(manifest.validate().is_ok());
    assert_eq!(manifest.total_bytes(), 412 * 1024 * 1024);
    assert!(manifest.has_target(PartitionTarget::Rootfs));
    assert!(manifest.has_target(PartitionTarget::Kernel));
    assert!(!manifest.has_target(PartitionTarget::Initramfs));
    assert_eq!(manifest.find_artifact(PartitionTarget::Kernel).unwrap().file_name, "vmlinuz");

    // Invalid: empty update_id
    let mut bad_manifest = manifest.clone();
    bad_manifest.update_id = "".to_string();
    assert!(bad_manifest.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));

    // Invalid: oversized update_id
    bad_manifest.update_id = "x".repeat(MAX_UPDATE_ID_LEN + 1);
    assert!(bad_manifest.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));

    // Invalid: empty version
    bad_manifest.update_id = "upd-1".to_string();
    bad_manifest.version = "".to_string();
    assert!(bad_manifest.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));

    // Invalid: oversized version
    bad_manifest.version = "v".repeat(MAX_UPDATE_VERSION_LEN + 1);
    assert!(bad_manifest.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));

    // Invalid: empty artifacts
    bad_manifest.version = "2.1.0".to_string();
    bad_manifest.artifacts = vec![];
    assert!(bad_manifest.validate().unwrap_err().contains(UPD_VALIDATION_ERROR));

    // Invalid: duplicate filenames
    let mut dup_art = rootfs_artifact.clone();
    dup_art.target = PartitionTarget::Initramfs;
    bad_manifest.artifacts = vec![rootfs_artifact.clone(), dup_art];
    assert!(bad_manifest.validate().unwrap_err().contains("duplicate artifact filename"));

    // Invalid: duplicate partition target
    let mut dup_tgt = kernel_artifact.clone();
    dup_tgt.file_name = "vmlinuz-alt".to_string();
    dup_tgt.target = PartitionTarget::Rootfs;
    bad_manifest.artifacts = vec![rootfs_artifact.clone(), dup_tgt];
    assert!(bad_manifest.validate().unwrap_err().contains("duplicate partition target"));
}

#[test]
fn test_upd4_state_machine_transitions() {
    let mut status = SystemUpdateStatus::new("1.0.0", UpdateSlot::SlotA, "2026-09-20T12:00:00Z");
    assert_eq!(status.state, UpdateState::Idle);

    // Standard happy-path sequence
    assert!(status.transition(UpdateState::Checking).is_ok());
    assert_eq!(status.state, UpdateState::Checking);

    assert!(status.transition(UpdateState::Downloading).is_ok());
    status.set_progress(35);
    assert_eq!(status.progress_percent, 35);

    assert!(status.transition(UpdateState::Verifying).is_ok());
    status.set_progress(80);

    assert!(status.transition(UpdateState::Applying).is_ok());
    status.set_progress(100);

    assert!(status.transition(UpdateState::ReadyToReboot).is_ok());

    assert!(status.transition(UpdateState::Verified).is_ok());

    assert!(status.transition(UpdateState::Idle).is_ok());
    assert_eq!(status.progress_percent, 0);

    // Rollback branch
    assert!(status.transition(UpdateState::Downloading).is_ok());
    assert!(status.transition(UpdateState::Verifying).is_ok());
    assert!(status.transition(UpdateState::Applying).is_ok());
    assert!(status.transition(UpdateState::ReadyToReboot).is_ok());
    assert!(status.transition(UpdateState::RolledBack).is_ok());
    assert!(status.transition(UpdateState::Idle).is_ok());

    // Failure transition from anywhere
    assert!(status.transition(UpdateState::Checking).is_ok());
    status.set_error("Connection timeout");
    assert_eq!(status.state, UpdateState::Failed);
    assert_eq!(status.last_error.as_deref(), Some("Connection timeout"));
    assert!(status.transition(UpdateState::Idle).is_ok());

    // Invalid transition checks
    assert_eq!(status.state, UpdateState::Idle);
    let bad_transition = status.transition(UpdateState::ReadyToReboot);
    assert!(bad_transition.is_err());
    assert!(bad_transition.unwrap_err().contains(UPD_STATE_ERROR));
}

#[test]
fn test_upd6_status_serde_parity() {
    let status = SystemUpdateStatus::new("1.0.0", UpdateSlot::SlotA, "2026-09-20T12:00:00Z");
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"state\":\"idle\""));
    assert!(json.contains("\"current_version\":\"1.0.0\""));
    assert!(json.contains("\"active_slot\":\"slot_a\""));

    let decoded: SystemUpdateStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, status);
}
