//! Unit tests for System Update Recovery & Validation Subsystem (UVAL1..UVAL6).

use std::fs;
use std::path::Path;
use tempfile::tempdir;

use aiosh_core::system_update::{
    SystemSlotStatus, SystemUpdateStatus, UpdateSlot, UpdateState,
};
use aiosh_core::system_update_recovery::{
    check_update_files, recover_update_files_with_backup, recover_update_state_in_memory,
    validate_update_state, validate_update_store_path, SystemUpdateRecoveryAction,
};

#[test]
fn test_uval1_path_validation() {
    // Valid path
    assert!(validate_update_store_path(Path::new("/var/lib/aiosh/updates/slot_status.json")).is_ok());
    assert!(validate_update_store_path(Path::new("relative/path/state.JSON")).is_ok());

    // Invalid: non-json extension
    let err_ext = validate_update_store_path(Path::new("/var/lib/aiosh/updates/slot_status.txt"));
    assert!(err_ext.is_err());
    assert!(err_ext.unwrap_err().contains(".json"));

    // Invalid: directory traversal
    let err_trav = validate_update_store_path(Path::new("../forbidden/slot.json"));
    assert!(err_trav.is_err());
    assert!(err_trav.unwrap_err().contains("path traversal"));

    // Invalid: control characters
    let err_ctrl = validate_update_store_path(Path::new("/var/lib/aiosh/\x00slot.json"));
    assert!(err_ctrl.is_err());
    assert!(err_ctrl.unwrap_err().contains("control characters"));

    // Invalid: empty
    assert!(validate_update_store_path(Path::new("")).is_err());
}

#[test]
fn test_uval2_in_memory_validation_and_recovery() {
    let dir = tempdir().unwrap();
    let staging_dir = dir.path().join("staging");
    fs::create_dir_all(&staging_dir).unwrap();

    // 1. Healthy state
    let mut slot = SystemSlotStatus::new(UpdateSlot::SlotA, "1.0.0");
    let mut update = SystemUpdateStatus::new("1.0.0", UpdateSlot::SlotA, "2026-09-20T12:00:00Z");

    let report_healthy = validate_update_state(&slot, &update, &staging_dir);
    assert!(report_healthy.healthy);
    assert!(report_healthy.errors.is_empty());
    assert!(!report_healthy.slot_conflict_detected);

    // 2. Corrupt state: slot conflict & stuck downloading
    slot.target_slot = UpdateSlot::SlotA; // Conflict!
    update.state = UpdateState::Downloading;
    update.progress_percent = 45;

    let report_corrupt = validate_update_state(&slot, &update, &staging_dir);
    assert!(!report_corrupt.healthy);
    assert!(report_corrupt.slot_conflict_detected);

    // 3. Self-healing in memory
    let recovery_report = recover_update_state_in_memory(&mut slot, &mut update, &staging_dir);
    assert!(recovery_report.recovered);
    assert!(recovery_report.actions_taken.iter().any(|a| matches!(a, SystemUpdateRecoveryAction::SynchronizedBootSlotPointer { .. })));
    assert!(recovery_report.actions_taken.iter().any(|a| matches!(a, SystemUpdateRecoveryAction::ResetFailedUpdateState { .. })));

    // 4. Verify healed state
    assert_eq!(slot.current_slot, UpdateSlot::SlotA);
    assert_eq!(slot.target_slot, UpdateSlot::SlotB);
    assert_eq!(update.state, UpdateState::Idle);
    assert_eq!(update.progress_percent, 0);

    let report_post_heal = validate_update_state(&slot, &update, &staging_dir);
    assert!(report_post_heal.healthy);
}

#[test]
fn test_uval3_disk_check_and_quarantine_recovery() {
    let dir = tempdir().unwrap();
    let state_dir = dir.path().join("state");
    let staging_dir = dir.path().join("staging");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&staging_dir).unwrap();

    let slot_path = state_dir.join("slot_status.json");
    let update_path = state_dir.join("update_status.json");

    // Write corrupt JSON
    fs::write(&slot_path, b"INVALID_CORRUPT_JSON_DATA{{{").unwrap();
    fs::write(&update_path, b"ALSO_CORRUPT_JSON_DATA<<<").unwrap();

    // Check detects corruption
    let check_report = check_update_files(&state_dir, &staging_dir);
    assert!(!check_report.healthy);
    assert!(!check_report.slot_status_valid);
    assert!(!check_report.update_status_valid);

    // Perform recovery with quarantine backup
    let recovery_res = recover_update_files_with_backup(&state_dir, &staging_dir, "2.0.0", UpdateSlot::SlotA);
    assert!(recovery_res.is_ok());
    let recovery_report = recovery_res.unwrap();
    assert!(recovery_report.recovered);

    // Verify quarantine files were created
    let entries: Vec<_> = fs::read_dir(&state_dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect();
    assert!(entries.iter().any(|name| name.contains(".corrupt.")));

    // Verify fresh files exist and parse cleanly
    let post_check = check_update_files(&state_dir, &staging_dir);
    assert!(post_check.healthy);
    assert!(post_check.slot_status_valid);
    assert!(post_check.update_status_valid);
}

#[test]
fn test_uval4_dangling_artifact_pruning() {
    let dir = tempdir().unwrap();
    let state_dir = dir.path().join("state");
    let staging_dir = dir.path().join("staging");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&staging_dir).unwrap();

    // Populate staging directory with 2 dummy files
    let dummy1 = staging_dir.join("rootfs.raw");
    let dummy2 = staging_dir.join("vmlinuz.efi");
    fs::write(&dummy1, b"DUMMY_STAGED_PAYLOAD_1").unwrap();
    fs::write(&dummy2, b"DUMMY_STAGED_PAYLOAD_2").unwrap();

    // Run recovery
    let recovery_report = recover_update_files_with_backup(&state_dir, &staging_dir, "1.5.0", UpdateSlot::SlotA).unwrap();

    // Verify dangling artifacts were pruned
    assert!(!dummy1.exists());
    assert!(!dummy2.exists());
    assert!(recovery_report.actions_taken.iter().any(|a| matches!(a, SystemUpdateRecoveryAction::PrunedDanglingArtifacts { count: 2, .. })));
}
