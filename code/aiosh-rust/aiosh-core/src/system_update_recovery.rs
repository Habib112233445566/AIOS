//! Health check, validation, and corruption recovery for System Update (UVAL1..UVAL6).
//!
//! Provides automated non-destructive self-healing, timestamped quarantine of damaged
//! state files, dangling artifact pruning, state machine reset, and slot synchronization.

use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::system_update::{
    SystemSlotStatus, SystemUpdateStatus, UpdateSlot, UpdateState,
};

/// Maximum permissible size for a system update state file on disk (1 MB).
pub const MAX_UPDATE_STORE_SIZE: u64 = 1_048_576;

pub const UVAL_PATH_ERROR: &str = "UVAL_PATH_ERROR";
pub const UVAL_IO_ERROR: &str = "UVAL_IO_ERROR";
pub const UVAL_VALIDATION_ERROR: &str = "UVAL_VALIDATION_ERROR";
pub const UVAL_PARSE_ERROR: &str = "UVAL_PARSE_ERROR";

/// Validates that an update state path is safe, bounded, free from directory traversal, and ends with .json.
pub fn validate_update_store_path(path: &Path) -> Result<(), String> {
    let path_str = path.to_str().ok_or_else(|| format!("{}: path must be valid UTF-8", UVAL_PATH_ERROR))?;
    if path_str.trim().is_empty() {
        return Err(format!("{}: path cannot be empty", UVAL_PATH_ERROR));
    }
    if path_str.len() > 1024 {
        return Err(format!("{}: path exceeds maximum length of 1024 characters", UVAL_PATH_ERROR));
    }
    if path_str.chars().any(|c| c.is_control() || c == '\0') {
        return Err(format!("{}: path cannot contain control characters", UVAL_PATH_ERROR));
    }
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(format!("{}: path traversal ('..') is not permitted", UVAL_PATH_ERROR));
    }
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("json") => Ok(()),
        _ => Err(format!("{}: path must have a '.json' extension", UVAL_PATH_ERROR)),
    }
}

/// Validation report detailing the integrity of the system update state and files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemUpdateValidationReport {
    pub state_dir: String,
    pub slot_status_valid: bool,
    pub update_status_valid: bool,
    pub staging_dir_valid: bool,
    pub slot_conflict_detected: bool,
    pub dangling_artifacts: Vec<String>,
    pub errors: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}

/// Actions taken during system update self-healing recovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SystemUpdateRecoveryAction {
    QuarantinedCorruptFile { original_path: String, quarantine_path: String },
    RestoredDefaultSlotStatus { active_slot: UpdateSlot, version: String },
    ResetFailedUpdateState { previous_state: UpdateState },
    PrunedDanglingArtifacts { count: usize, bytes_freed: u64 },
    SynchronizedBootSlotPointer { current_slot: UpdateSlot, target_slot: UpdateSlot },
}

/// Report summarizing all recovery actions performed during self-healing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemUpdateRecoveryReport {
    pub state_dir: String,
    pub recovered: bool,
    pub actions_taken: Vec<SystemUpdateRecoveryAction>,
    pub timestamp: String,
}

/// Validates in-memory slot status and update status integrity (UVAL1).
pub fn validate_update_state(
    slot_status: &SystemSlotStatus,
    update_status: &SystemUpdateStatus,
    staging_dir: &Path,
) -> SystemUpdateValidationReport {
    let mut errors = Vec::new();
    let mut slot_conflict_detected = false;

    // Slot validation
    let mut slot_status_valid = true;
    if slot_status.current_slot == slot_status.target_slot {
        slot_status_valid = false;
        slot_conflict_detected = true;
        errors.push("current_slot and target_slot cannot be identical".to_string());
    }

    if slot_status.slot_a_version.trim().is_empty() || slot_status.slot_b_version.trim().is_empty() {
        slot_status_valid = false;
        errors.push("slot versions cannot be empty".to_string());
    }

    // Update status validation
    let mut update_status_valid = true;
    if update_status.current_version.trim().is_empty() {
        update_status_valid = false;
        errors.push("current_version cannot be empty".to_string());
    }

    // Staging directory validation and dangling artifact detection
    let mut staging_dir_valid = true;
    let mut dangling_artifacts = Vec::new();

    if staging_dir.exists() {
        if !staging_dir.is_dir() {
            staging_dir_valid = false;
            errors.push(format!("staging path {:?} is not a directory", staging_dir));
        } else if let Ok(entries) = fs::read_dir(staging_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        dangling_artifacts.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    let healthy = slot_status_valid && update_status_valid && staging_dir_valid && errors.is_empty();

    SystemUpdateValidationReport {
        state_dir: staging_dir.parent().unwrap_or(staging_dir).to_string_lossy().to_string(),
        slot_status_valid,
        update_status_valid,
        staging_dir_valid,
        slot_conflict_detected,
        dangling_artifacts,
        errors,
        healthy,
        evaluated_at: Utc::now().to_rfc3339(),
    }
}

/// Inspects update state files on disk for existence, format, and consistency (UVAL1).
pub fn check_update_files(state_dir: &Path, staging_dir: &Path) -> SystemUpdateValidationReport {
    let mut errors = Vec::new();
    let mut slot_status_valid = true;
    let mut update_status_valid = true;
    let mut slot_conflict_detected = false;

    let slot_path = state_dir.join("slot_status.json");
    let update_path = state_dir.join("update_status.json");

    // Check slot status file
    if !slot_path.exists() {
        slot_status_valid = false;
        errors.push("slot_status.json does not exist".to_string());
    } else if let Ok(meta) = fs::symlink_metadata(&slot_path) {
        if meta.file_type().is_symlink() {
            slot_status_valid = false;
            errors.push("slot_status.json is a symlink (rejected)".to_string());
        } else if meta.len() > MAX_UPDATE_STORE_SIZE {
            slot_status_valid = false;
            errors.push(format!("slot_status.json exceeds size limit: {} bytes", meta.len()));
        } else {
            match fs::read_to_string(&slot_path) {
                Ok(content) => match serde_json::from_str::<SystemSlotStatus>(&content) {
                    Ok(slot) => {
                        if slot.current_slot == slot.target_slot {
                            slot_status_valid = false;
                            slot_conflict_detected = true;
                            errors.push("slot_status.json has current_slot == target_slot".to_string());
                        }
                    }
                    Err(e) => {
                        slot_status_valid = false;
                        errors.push(format!("failed to parse slot_status.json: {}", e));
                    }
                },
                Err(e) => {
                    slot_status_valid = false;
                    errors.push(format!("failed to read slot_status.json: {}", e));
                }
            }
        }
    }

    // Check update status file
    if !update_path.exists() {
        update_status_valid = false;
        errors.push("update_status.json does not exist".to_string());
    } else if let Ok(meta) = fs::symlink_metadata(&update_path) {
        if meta.file_type().is_symlink() {
            update_status_valid = false;
            errors.push("update_status.json is a symlink (rejected)".to_string());
        } else if meta.len() > MAX_UPDATE_STORE_SIZE {
            update_status_valid = false;
            errors.push(format!("update_status.json exceeds size limit: {} bytes", meta.len()));
        } else {
            match fs::read_to_string(&update_path) {
                Ok(content) => {
                    if let Err(e) = serde_json::from_str::<SystemUpdateStatus>(&content) {
                        update_status_valid = false;
                        errors.push(format!("failed to parse update_status.json: {}", e));
                    }
                }
                Err(e) => {
                    update_status_valid = false;
                    errors.push(format!("failed to read update_status.json: {}", e));
                }
            }
        }
    }

    // Staging artifacts check
    let mut staging_dir_valid = true;
    let mut dangling_artifacts = Vec::new();
    if staging_dir.exists() {
        if !staging_dir.is_dir() {
            staging_dir_valid = false;
            errors.push("staging_dir is not a directory".to_string());
        } else if let Ok(entries) = fs::read_dir(staging_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        dangling_artifacts.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    let healthy = slot_status_valid && update_status_valid && staging_dir_valid && errors.is_empty();

    SystemUpdateValidationReport {
        state_dir: state_dir.to_string_lossy().to_string(),
        slot_status_valid,
        update_status_valid,
        staging_dir_valid,
        slot_conflict_detected,
        dangling_artifacts,
        errors,
        healthy,
        evaluated_at: Utc::now().to_rfc3339(),
    }
}

/// Recovers in-memory update state by synchronizing slots and resetting invalid states (UVAL4, UVAL5).
pub fn recover_update_state_in_memory(
    slot_status: &mut SystemSlotStatus,
    update_status: &mut SystemUpdateStatus,
    staging_dir: &Path,
) -> SystemUpdateRecoveryReport {
    let mut actions_taken = Vec::new();

    // 1. Resolve slot conflicts (UVAL5)
    if slot_status.current_slot == slot_status.target_slot {
        slot_status.target_slot = slot_status.current_slot.other();
        slot_status.rollback_slot = Some(slot_status.current_slot);
        actions_taken.push(SystemUpdateRecoveryAction::SynchronizedBootSlotPointer {
            current_slot: slot_status.current_slot,
            target_slot: slot_status.target_slot,
        });
    }

    // 2. Reset invalid or unrecoverable update state (UVAL4)
    if update_status.state != UpdateState::Idle {
        let prev = update_status.state;
        update_status.state = UpdateState::Idle;
        update_status.progress_percent = 0;
        update_status.target_version = None;
        update_status.last_error = None;
        actions_taken.push(SystemUpdateRecoveryAction::ResetFailedUpdateState {
            previous_state: prev,
        });
    }

    // 3. Prune dangling artifacts (UVAL3)
    if staging_dir.exists() && staging_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(staging_dir) {
            let mut count = 0;
            let mut bytes_freed = 0u64;
            for entry in entries.flatten() {
                let p = entry.path();
                if let Ok(meta) = fs::symlink_metadata(&p) {
                    if meta.file_type().is_file() {
                        bytes_freed = bytes_freed.saturating_add(meta.len());
                        if fs::remove_file(&p).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
            if count > 0 {
                actions_taken.push(SystemUpdateRecoveryAction::PrunedDanglingArtifacts {
                    count,
                    bytes_freed,
                });
            }
        }
    }

    SystemUpdateRecoveryReport {
        state_dir: staging_dir.parent().unwrap_or(staging_dir).to_string_lossy().to_string(),
        recovered: !actions_taken.is_empty(),
        actions_taken,
        timestamp: Utc::now().to_rfc3339(),
    }
}

/// Quarantines corrupt state files on disk and restores coherent default state (UVAL2, UVAL6).
pub fn recover_update_files_with_backup(
    state_dir: &Path,
    staging_dir: &Path,
    fallback_version: &str,
    fallback_slot: UpdateSlot,
) -> Result<SystemUpdateRecoveryReport, String> {
    fs::create_dir_all(state_dir)
        .map_err(|e| format!("{}: failed to create state directory {:?}: {}", UVAL_IO_ERROR, state_dir, e))?;

    let mut actions_taken = Vec::new();
    let ts = Utc::now().format("%Y%m%d_%H%M%S").to_string();

    let slot_path = state_dir.join("slot_status.json");
    let update_path = state_dir.join("update_status.json");

    // Helper to quarantine and replace
    let quarantine_if_invalid = |file_path: &Path, is_slot: bool| -> Result<Option<SystemUpdateRecoveryAction>, String> {
        if !file_path.exists() {
            return Ok(None);
        }

        let is_valid = match fs::symlink_metadata(file_path) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err(format!("{}: file {:?} is a symlink, refusing to read or quarantine", UVAL_PATH_ERROR, file_path));
                }
                if meta.len() > MAX_UPDATE_STORE_SIZE {
                    false
                } else {
                    match fs::read_to_string(file_path) {
                        Ok(content) => {
                            if is_slot {
                                serde_json::from_str::<SystemSlotStatus>(&content).is_ok()
                            } else {
                                serde_json::from_str::<SystemUpdateStatus>(&content).is_ok()
                            }
                        }
                        Err(_) => false,
                    }
                }
            }
            Err(_) => false,
        };

        if !is_valid {
            let quarantine_path = format!("{}.corrupt.{}", file_path.to_string_lossy(), ts);
            let quarantine_buf = PathBuf::from(&quarantine_path);
            fs::rename(file_path, &quarantine_buf)
                .map_err(|e| format!("{}: failed to quarantine {:?} to {:?}: {}", UVAL_IO_ERROR, file_path, quarantine_buf, e))?;

            Ok(Some(SystemUpdateRecoveryAction::QuarantinedCorruptFile {
                original_path: file_path.to_string_lossy().to_string(),
                quarantine_path,
            }))
        } else {
            Ok(None)
        }
    };

    if let Some(action) = quarantine_if_invalid(&slot_path, true)? {
        actions_taken.push(action);
    }
    if let Some(action) = quarantine_if_invalid(&update_path, false)? {
        actions_taken.push(action);
    }

    // If slot_status.json is missing or was quarantined, write coherent fresh state
    if !slot_path.exists() {
        let fresh_slot = SystemSlotStatus::new(fallback_slot, fallback_version);
        let data = serde_json::to_string_pretty(&fresh_slot)
            .map_err(|e| format!("{}: failed to serialize slot status: {}", UVAL_VALIDATION_ERROR, e))?;
        
        let tmp_slot = format!("{}.tmp.{}", slot_path.to_string_lossy(), std::process::id());
        if let Err(e) = fs::write(&tmp_slot, data.as_bytes()) {
            let _ = fs::remove_file(&tmp_slot);
            return Err(format!("{}: failed to write {:?}: {}", UVAL_IO_ERROR, tmp_slot, e));
        }
        if let Err(e) = fs::rename(&tmp_slot, &slot_path) {
            let _ = fs::remove_file(&tmp_slot);
            return Err(format!("{}: failed to rename {:?} to {:?}: {}", UVAL_IO_ERROR, tmp_slot, slot_path, e));
        }

        actions_taken.push(SystemUpdateRecoveryAction::RestoredDefaultSlotStatus {
            active_slot: fallback_slot,
            version: fallback_version.to_string(),
        });
    }

    // If update_status.json is missing or was quarantined, write coherent fresh state
    if !update_path.exists() {
        let fresh_update = SystemUpdateStatus::new(fallback_version, fallback_slot, Utc::now().to_rfc3339());
        let data = serde_json::to_string_pretty(&fresh_update)
            .map_err(|e| format!("{}: failed to serialize update status: {}", UVAL_VALIDATION_ERROR, e))?;

        let tmp_update = format!("{}.tmp.{}", update_path.to_string_lossy(), std::process::id());
        if let Err(e) = fs::write(&tmp_update, data.as_bytes()) {
            let _ = fs::remove_file(&tmp_update);
            return Err(format!("{}: failed to write {:?}: {}", UVAL_IO_ERROR, tmp_update, e));
        }
        if let Err(e) = fs::rename(&tmp_update, &update_path) {
            let _ = fs::remove_file(&tmp_update);
            return Err(format!("{}: failed to rename {:?} to {:?}: {}", UVAL_IO_ERROR, tmp_update, update_path, e));
        }

        actions_taken.push(SystemUpdateRecoveryAction::ResetFailedUpdateState {
            previous_state: UpdateState::Failed,
        });
    }

    // Prune staging directory dangling artifacts
    if staging_dir.exists() && staging_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(staging_dir) {
            let mut count = 0;
            let mut bytes_freed = 0u64;
            for entry in entries.flatten() {
                let p = entry.path();
                if let Ok(meta) = fs::symlink_metadata(&p) {
                    if meta.file_type().is_symlink() {
                        if fs::remove_file(&p).is_ok() {
                            count += 1;
                        }
                    } else if meta.file_type().is_file() {
                        bytes_freed = bytes_freed.saturating_add(meta.len());
                        if fs::remove_file(&p).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
            if count > 0 {
                actions_taken.push(SystemUpdateRecoveryAction::PrunedDanglingArtifacts {
                    count,
                    bytes_freed,
                });
            }
        }
    }

    Ok(SystemUpdateRecoveryReport {
        state_dir: state_dir.to_string_lossy().to_string(),
        recovered: !actions_taken.is_empty(),
        actions_taken,
        timestamp: Utc::now().to_rfc3339(),
    })
}
