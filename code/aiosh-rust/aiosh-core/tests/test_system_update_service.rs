//! Unit tests for System Update Core Service (USVC1..USVC6).

use std::fs;
use tempfile::tempdir;
use sha2::{Digest, Sha256};

use aiosh_core::system_update::{
    PartitionTarget, UpdateArtifact, UpdateChannel, UpdateManifest, UpdateSlot, UpdateState,
    UPD_DIGEST_ERROR, UPD_VALIDATION_ERROR,
};
use aiosh_core::system_update_service::{SystemUpdateService, SystemUpdateServiceConfig};

fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

fn create_sample_manifest(rootfs_data: &[u8], kernel_data: &[u8]) -> UpdateManifest {
    UpdateManifest {
        update_id: "upd-2026-09-20-001".to_string(),
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
        signature: Some("ed25519:sigmock".to_string()),
        release_notes: "Routine security and performance update".to_string(),
        published_at: "2026-09-20T12:00:00Z".to_string(),
    }
}

#[test]
fn test_usvc1_initialization_and_defaults() {
    let service = SystemUpdateService::with_defaults("2.0.0", UpdateSlot::SlotA, "2026-09-20T12:00:00Z");
    assert_eq!(service.slot_status.current_slot, UpdateSlot::SlotA);
    assert_eq!(service.slot_status.target_slot, UpdateSlot::SlotB);
    assert_eq!(service.slot_status.rollback_slot, Some(UpdateSlot::SlotA));
    assert_eq!(service.update_status.state, UpdateState::Idle);
    assert_eq!(service.update_status.current_version, "2.0.0");
    assert!(service.active_manifest.is_none());
}

#[test]
fn test_usvc2_happy_path_update_lifecycle() {
    let dir = tempdir().unwrap();
    let state_dir = dir.path().join("state");
    let staging_dir = dir.path().join("staging");

    let config = SystemUpdateServiceConfig {
        state_dir: state_dir.clone(),
        staging_dir: staging_dir.clone(),
        max_payload_bytes: 10 * 1024 * 1024,
        auto_rollback_on_failure: true,
    };

    let mut service = SystemUpdateService::new("2.0.0", UpdateSlot::SlotA, config, "2026-09-20T12:00:00Z");

    let rootfs_data = b"AIOS ROOTFS V2.5.0 SIMULATED PAYLOAD DATA";
    let kernel_data = b"AIOS LINUX KERNEL 6.12.0 SIMULATED BINARY";
    let manifest = create_sample_manifest(rootfs_data, kernel_data);

    // 1. Check manifest
    assert!(service.check_manifest(manifest).is_ok());
    assert_eq!(service.update_status.state, UpdateState::Downloading);
    assert_eq!(service.update_status.target_version.as_deref(), Some("2.5.0"));

    // 2. Stage rootfs
    let staged_rootfs = service.stage_artifact(PartitionTarget::Rootfs, rootfs_data);
    assert!(staged_rootfs.is_ok());
    assert!(staged_rootfs.unwrap().exists());

    // 3. Stage kernel
    let staged_kernel = service.stage_artifact(PartitionTarget::Kernel, kernel_data);
    assert!(staged_kernel.is_ok());
    assert!(staged_kernel.unwrap().exists());

    // 4. Verify staged
    assert!(service.verify_staged().is_ok());
    assert_eq!(service.update_status.state, UpdateState::Verifying);

    // 5. Apply update
    let next_slot = service.apply_update().unwrap();
    assert_eq!(next_slot, UpdateSlot::SlotB);
    assert_eq!(service.slot_status.current_slot, UpdateSlot::SlotB);
    assert_eq!(service.slot_status.rollback_slot, Some(UpdateSlot::SlotA));
    assert_eq!(service.update_status.state, UpdateState::ReadyToReboot);
    assert_eq!(service.update_status.progress_percent, 100);

    // 6. Confirm boot
    assert!(service.confirm_boot("2.5.0").is_ok());
    assert_eq!(service.update_status.state, UpdateState::Idle);
    assert_eq!(service.update_status.current_version, "2.5.0");
    assert!(service.slot_status.slot_b_successful);
    assert_eq!(service.slot_status.slot_b_version, "2.5.0");
    assert!(service.active_manifest.is_none());
}

#[test]
fn test_usvc2_digest_mismatch_fails_and_halts() {
    let dir = tempdir().unwrap();
    let config = SystemUpdateServiceConfig {
        state_dir: dir.path().join("state"),
        staging_dir: dir.path().join("staging"),
        max_payload_bytes: 10 * 1024 * 1024,
        auto_rollback_on_failure: true,
    };

    let mut service = SystemUpdateService::new("2.0.0", UpdateSlot::SlotA, config, "2026-09-20T12:00:00Z");

    let rootfs_data = b"VALID ROOTFS DATA";
    let kernel_data = b"VALID KERNEL DATA";
    let manifest = create_sample_manifest(rootfs_data, kernel_data);

    assert!(service.check_manifest(manifest).is_ok());

    // Corrupted payload with same byte length
    let corrupted_rootfs = b"TAMPERED DATA!!!!";
    let err = service.stage_artifact(PartitionTarget::Rootfs, corrupted_rootfs).unwrap_err();
    assert!(err.contains(UPD_DIGEST_ERROR));
    assert_eq!(service.update_status.state, UpdateState::Failed);
    assert!(service.update_status.last_error.unwrap().contains(UPD_DIGEST_ERROR));
}

#[test]
fn test_usvc2_size_mismatch_rejected() {
    let dir = tempdir().unwrap();
    let config = SystemUpdateServiceConfig {
        state_dir: dir.path().join("state"),
        staging_dir: dir.path().join("staging"),
        max_payload_bytes: 10 * 1024 * 1024,
        auto_rollback_on_failure: true,
    };

    let mut service = SystemUpdateService::new("2.0.0", UpdateSlot::SlotA, config, "2026-09-20T12:00:00Z");

    let rootfs_data = b"VALID ROOTFS DATA";
    let kernel_data = b"VALID KERNEL DATA";
    let manifest = create_sample_manifest(rootfs_data, kernel_data);

    assert!(service.check_manifest(manifest).is_ok());

    let wrong_size_data = b"SHORT";
    let err = service.stage_artifact(PartitionTarget::Rootfs, wrong_size_data).unwrap_err();
    assert!(err.contains(UPD_VALIDATION_ERROR));
}

#[test]
fn test_usvc3_incomplete_staging_cannot_verify() {
    let dir = tempdir().unwrap();
    let config = SystemUpdateServiceConfig {
        state_dir: dir.path().join("state"),
        staging_dir: dir.path().join("staging"),
        max_payload_bytes: 10 * 1024 * 1024,
        auto_rollback_on_failure: true,
    };

    let mut service = SystemUpdateService::new("2.0.0", UpdateSlot::SlotA, config, "2026-09-20T12:00:00Z");

    let rootfs_data = b"VALID ROOTFS DATA";
    let kernel_data = b"VALID KERNEL DATA";
    let manifest = create_sample_manifest(rootfs_data, kernel_data);

    assert!(service.check_manifest(manifest).is_ok());

    // Only stage rootfs, skip kernel
    assert!(service.stage_artifact(PartitionTarget::Rootfs, rootfs_data).is_ok());

    let verify_err = service.verify_staged().unwrap_err();
    assert!(verify_err.contains("missing staged artifact"));
}

#[test]
fn test_usvc4_atomic_state_persistence_and_reload() {
    let dir = tempdir().unwrap();
    let state_dir = dir.path().join("state");

    let config = SystemUpdateServiceConfig {
        state_dir: state_dir.clone(),
        staging_dir: dir.path().join("staging"),
        max_payload_bytes: 10 * 1024 * 1024,
        auto_rollback_on_failure: true,
    };

    let mut service = SystemUpdateService::new("2.0.0", UpdateSlot::SlotA, config.clone(), "2026-09-20T12:00:00Z");
    service.slot_status.switch_slot();
    service.slot_status.mark_slot_success(UpdateSlot::SlotB, "2.1.0");

    assert!(service.save_state_to_dir(&state_dir).is_ok());
    assert!(state_dir.join("slot_status.json").exists());
    assert!(state_dir.join("update_status.json").exists());

    // Load state back
    let loaded = SystemUpdateService::load_state_from_dir(&state_dir, config).unwrap();
    assert_eq!(loaded.slot_status.current_slot, UpdateSlot::SlotB);
    assert_eq!(loaded.slot_status.target_slot, UpdateSlot::SlotA);
    assert_eq!(loaded.slot_status.slot_b_version, "2.1.0");
}

#[test]
fn test_usvc5_rollback_orchestration() {
    let dir = tempdir().unwrap();
    let config = SystemUpdateServiceConfig {
        state_dir: dir.path().join("state"),
        staging_dir: dir.path().join("staging"),
        max_payload_bytes: 10 * 1024 * 1024,
        auto_rollback_on_failure: true,
    };

    let mut service = SystemUpdateService::new("2.0.0", UpdateSlot::SlotA, config, "2026-09-20T12:00:00Z");
    let rootfs_data = b"VALID ROOTFS DATA";
    let kernel_data = b"VALID KERNEL DATA";
    let manifest = create_sample_manifest(rootfs_data, kernel_data);

    assert!(service.check_manifest(manifest).is_ok());
    assert!(service.stage_artifact(PartitionTarget::Rootfs, rootfs_data).is_ok());
    assert!(service.stage_artifact(PartitionTarget::Kernel, kernel_data).is_ok());
    assert!(service.verify_staged().is_ok());
    assert!(service.apply_update().is_ok());
    assert_eq!(service.slot_status.current_slot, UpdateSlot::SlotB);

    // Boot fails, trigger rollback
    let rolled_back_slot = service.rollback().unwrap();
    assert_eq!(rolled_back_slot, UpdateSlot::SlotA);
    assert_eq!(service.slot_status.current_slot, UpdateSlot::SlotA);
    assert_eq!(service.update_status.state, UpdateState::Idle);
}

#[test]
fn test_usvc6_clean_staging() {
    let dir = tempdir().unwrap();
    let staging_dir = dir.path().join("staging");
    fs::create_dir_all(&staging_dir).unwrap();
    fs::write(staging_dir.join("leftover.bin"), b"test").unwrap();

    let config = SystemUpdateServiceConfig {
        state_dir: dir.path().join("state"),
        staging_dir: staging_dir.clone(),
        max_payload_bytes: 10 * 1024 * 1024,
        auto_rollback_on_failure: true,
    };

    let mut service = SystemUpdateService::new("2.0.0", UpdateSlot::SlotA, config, "2026-09-20T12:00:00Z");
    assert!(service.clean_staging().is_ok());
    assert!(!staging_dir.join("leftover.bin").exists());
    assert!(staging_dir.exists());
}
