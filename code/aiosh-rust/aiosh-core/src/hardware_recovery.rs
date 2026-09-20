//! Health check, validation, and corruption recovery for Hardware Detection (HVAL1..HVAL6).
//!
//! Provides automated non-destructive self-healing, timestamped quarantine of damaged
//! stores, drift detection against host sysfs paths, and deep validation reports.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::hardware::{
    HardwareInventory, MAX_DEVICES,
};

/// Maximum permissible size for a hardware store/inventory file on disk (10 MB).
pub const MAX_STORE_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Validation report detailing the integrity of a hardware inventory or store file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareValidationReport {
    pub store_path: String,
    pub total_devices: usize,
    pub valid_devices: usize,
    pub invalid_devices: usize,
    pub stale_paths: Vec<String>,
    pub drift_detected: bool,
    pub summary_mismatches: Vec<String>,
    pub errors: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}

impl HardwareValidationReport {
    /// Validates report internal invariants (HVAL1..HVAL3).
    pub fn validate_invariants(&self) -> Result<(), String> {
        if self.valid_devices + self.invalid_devices != self.total_devices {
            return Err(format!(
                "HVAL1 violated: valid_devices ({}) + invalid_devices ({}) != total_devices ({})",
                self.valid_devices, self.invalid_devices, self.total_devices
            ));
        }

        let expected_healthy = self.errors.is_empty()
            && self.invalid_devices == 0
            && !self.drift_detected
            && self.summary_mismatches.is_empty();

        if self.healthy != expected_healthy {
            return Err(format!(
                "HVAL3 violated: healthy ({}) != expected_healthy ({})",
                self.healthy, expected_healthy
            ));
        }

        Ok(())
    }
}

/// Actions performed during hardware inventory recovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HardwareRecoveryAction {
    NoneRequired,
    QuarantineCorruptedStore { backup_path: String },
    PruneInvalidDevices { pruned_count: usize },
    RecomputeSummary,
    RescanSysfs { scanned_devices: usize },
    RecreateEmptyInventory,
}

/// Comprehensive report on an automated recovery operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareRecoveryReport {
    pub store_path: String,
    pub initial_validation: HardwareValidationReport,
    pub actions_taken: Vec<HardwareRecoveryAction>,
    pub final_validation: HardwareValidationReport,
    pub backup_path: Option<String>,
    pub recovered: bool,
    pub completed_at: String,
}

/// Validates an in-memory HardwareInventory against structure, limits, and optional sysfs paths.
pub fn validate_inventory(
    inv: &HardwareInventory,
    store_path: &Path,
    check_paths: bool,
) -> HardwareValidationReport {
    let mut errors = Vec::new();
    let mut valid_devices = 0;
    let mut invalid_devices = 0;
    let mut stale_paths = Vec::new();
    let mut summary_mismatches = Vec::new();

    if inv.hostname.trim().is_empty() {
        errors.push("hostname cannot be empty".into());
    }
    if inv.architecture.trim().is_empty() {
        errors.push("architecture cannot be empty".into());
    }

    if inv.devices.len() > MAX_DEVICES {
        errors.push(format!(
            "device count {} exceeds maximum permitted limit of {}",
            inv.devices.len(),
            MAX_DEVICES
        ));
    }

    let mut seen_ids = std::collections::HashSet::new();
    let mut counted_classes: BTreeMap<String, usize> = BTreeMap::new();

    for d in &inv.devices {
        let mut dev_errors = Vec::new();
        if let Err(e) = d.validate() {
            dev_errors.push(format!("device '{}': {}", d.id, e));
        }

        if !seen_ids.insert(&d.id) {
            dev_errors.push(format!("device '{}': duplicate device id", d.id));
        }

        if check_paths {
            if let Some(ref p) = d.sysfs_path {
                let path = Path::new(p);
                if !path.exists() {
                    stale_paths.push(p.clone());
                }
            }
            if let Some(ref p) = d.dev_path {
                let path = Path::new(p);
                if !path.exists() {
                    stale_paths.push(p.clone());
                }
            }
        }

        if dev_errors.is_empty() {
            valid_devices += 1;
            *counted_classes.entry(d.class.as_str().to_string()).or_insert(0) += 1;
        } else {
            invalid_devices += 1;
            errors.extend(dev_errors);
        }
    }

    // Check summary parity (HVAL2)
    for (class_name, count) in &counted_classes {
        match inv.summary.get(class_name) {
            Some(inv_count) if inv_count == count => {}
            Some(inv_count) => {
                summary_mismatches.push(format!(
                    "class '{}': summary count {} != counted valid devices {}",
                    class_name, inv_count, count
                ));
            }
            None => {
                summary_mismatches.push(format!(
                    "class '{}': missing from summary map (counted {})",
                    class_name, count
                ));
            }
        }
    }
    for (class_name, inv_count) in &inv.summary {
        if !counted_classes.contains_key(class_name) && *inv_count > 0 {
            summary_mismatches.push(format!(
                "class '{}': summary specifies {} devices but 0 valid devices found",
                class_name, inv_count
            ));
        }
    }

    let drift_detected = !stale_paths.is_empty();
    let healthy = errors.is_empty()
        && invalid_devices == 0
        && !drift_detected
        && summary_mismatches.is_empty();

    HardwareValidationReport {
        store_path: store_path.to_string_lossy().to_string(),
        total_devices: inv.devices.len(),
        valid_devices,
        invalid_devices,
        stale_paths,
        drift_detected,
        summary_mismatches,
        errors,
        healthy,
        evaluated_at: Utc::now().to_rfc3339(),
    }
}

/// Checks an existing hardware inventory file on disk in read-only mode.
pub fn check_inventory_file(path: &Path, check_paths: bool) -> Result<HardwareValidationReport, String> {
    if !path.exists() {
        return Ok(HardwareValidationReport {
            store_path: path.to_string_lossy().to_string(),
            total_devices: 0,
            valid_devices: 0,
            invalid_devices: 0,
            stale_paths: Vec::new(),
            drift_detected: false,
            summary_mismatches: Vec::new(),
            errors: vec![format!("store file {:?} does not exist", path)],
            healthy: false,
            evaluated_at: Utc::now().to_rfc3339(),
        });
    }

    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            return Ok(HardwareValidationReport {
                store_path: path.to_string_lossy().to_string(),
                total_devices: 0,
                valid_devices: 0,
                invalid_devices: 0,
                stale_paths: Vec::new(),
                drift_detected: false,
                summary_mismatches: Vec::new(),
                errors: vec![format!("store metadata read failure: {}", e)],
                healthy: false,
                evaluated_at: Utc::now().to_rfc3339(),
            });
        }
    };

    if !meta.is_file() {
        return Ok(HardwareValidationReport {
            store_path: path.to_string_lossy().to_string(),
            total_devices: 0,
            valid_devices: 0,
            invalid_devices: 0,
            stale_paths: Vec::new(),
            drift_detected: false,
            summary_mismatches: Vec::new(),
            errors: vec![format!("store path {:?} is not a regular file", path)],
            healthy: false,
            evaluated_at: Utc::now().to_rfc3339(),
        });
    }

    if meta.len() > MAX_STORE_FILE_SIZE {
        return Ok(HardwareValidationReport {
            store_path: path.to_string_lossy().to_string(),
            total_devices: 0,
            valid_devices: 0,
            invalid_devices: 0,
            stale_paths: Vec::new(),
            drift_detected: false,
            summary_mismatches: Vec::new(),
            errors: vec![format!(
                "store file exceeds maximum permitted size of {} bytes (got {} bytes)",
                MAX_STORE_FILE_SIZE,
                meta.len()
            )],
            healthy: false,
            evaluated_at: Utc::now().to_rfc3339(),
        });
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return Ok(HardwareValidationReport {
                store_path: path.to_string_lossy().to_string(),
                total_devices: 0,
                valid_devices: 0,
                invalid_devices: 0,
                stale_paths: Vec::new(),
                drift_detected: false,
                summary_mismatches: Vec::new(),
                errors: vec![format!("store read failure: {}", e)],
                healthy: false,
                evaluated_at: Utc::now().to_rfc3339(),
            });
        }
    };

    match serde_json::from_str::<HardwareInventory>(&content) {
        Ok(inv) => Ok(validate_inventory(&inv, path, check_paths)),
        Err(e) => Ok(HardwareValidationReport {
            store_path: path.to_string_lossy().to_string(),
            total_devices: 0,
            valid_devices: 0,
            invalid_devices: 0,
            stale_paths: Vec::new(),
            drift_detected: false,
            summary_mismatches: Vec::new(),
            errors: vec![format!("store JSON deserialize failure: {}", e)],
            healthy: false,
            evaluated_at: Utc::now().to_rfc3339(),
        }),
    }
}

/// Surgical in-memory recovery of a HardwareInventory.
///
/// Prunes invalid devices, deduplicates IDs, and recalculates class summary counts.
pub fn recover_inventory_in_memory(inv: &mut HardwareInventory) -> Vec<HardwareRecoveryAction> {
    let mut actions = Vec::new();

    let initial_count = inv.devices.len();
    let mut valid_devices = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for d in inv.devices.drain(..) {
        if d.validate().is_ok() && seen_ids.insert(d.id.clone()) {
            valid_devices.push(d);
        }
    }

    let pruned_count = initial_count.saturating_sub(valid_devices.len());
    inv.devices = valid_devices;

    if pruned_count > 0 {
        actions.push(HardwareRecoveryAction::PruneInvalidDevices { pruned_count });
    }

    // Recompute summary
    let old_summary = inv.summary.clone();
    inv.update_summary();
    if inv.summary != old_summary {
        actions.push(HardwareRecoveryAction::RecomputeSummary);
    }

    if actions.is_empty() {
        actions.push(HardwareRecoveryAction::NoneRequired);
    }

    actions
}

/// Recovers a hardware store file on disk.
///
/// If corrupted or unparseable, creates a timestamped backup (`.bak.<timestamp>`)
/// and initializes a clean inventory. If devices are partially invalid, prunes
/// invalid entries and writes back the healed file.
pub fn recover_inventory_file(
    path: &Path,
    sysfs_path: Option<&Path>,
) -> Result<HardwareRecoveryReport, String> {
    let initial_validation = check_inventory_file(path, false)?;
    let mut actions_taken = Vec::new();
    let mut backup_path = None;

    if initial_validation.healthy {
        return Ok(HardwareRecoveryReport {
            store_path: path.to_string_lossy().to_string(),
            initial_validation: initial_validation.clone(),
            actions_taken: vec![HardwareRecoveryAction::NoneRequired],
            final_validation: initial_validation,
            backup_path: None,
            recovered: false,
            completed_at: Utc::now().to_rfc3339(),
        });
    }

    // Attempt read and parse
    let mut recovered_inv = match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<HardwareInventory>(&content) {
            Ok(mut inv) => {
                let recovery_actions = recover_inventory_in_memory(&mut inv);
                actions_taken.extend(recovery_actions);
                Some(inv)
            }
            Err(_) => None,
        },
        Err(_) => None,
    };

    // If completely unparseable or missing, quarantine if exists and create new
    if recovered_inv.is_none() {
        if path.exists() {
            let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
            let bak_filename = format!(
                "{}.bak.{}",
                path.file_name().unwrap_or_default().to_string_lossy(),
                timestamp
            );
            let bak_path = path.with_file_name(bak_filename);
            fs::copy(path, &bak_path)
                .map_err(|e| format!("failed to quarantine corrupted store: {}", e))?;
            let bak_str = bak_path.to_string_lossy().to_string();
            backup_path = Some(bak_str.clone());
            actions_taken.push(HardwareRecoveryAction::QuarantineCorruptedStore {
                backup_path: bak_str,
            });
        }

        // Initialize fresh inventory (either by live sysfs rescan or empty fallback)
        let mut fresh_opt = None;
        if let Some(sysfs) = sysfs_path {
            if sysfs.exists() {
                let service = crate::hardware_service::HardwareService::with_roots(sysfs, "/proc");
                if let Ok(scanned) = service.scan(&crate::hardware_service::HardwareScanOptions::default()) {
                    actions_taken.push(HardwareRecoveryAction::RescanSysfs {
                        scanned_devices: scanned.devices.len(),
                    });
                    fresh_opt = Some(scanned);
                }
            }
        }

        let fresh = match fresh_opt {
            Some(f) => f,
            None => {
                let mut f = HardwareInventory::new("localhost", std::env::consts::ARCH, "unknown");
                f.update_summary();
                actions_taken.push(HardwareRecoveryAction::RecreateEmptyInventory);
                f
            }
        };
        recovered_inv = Some(fresh);
    }

    let final_inv = recovered_inv.unwrap();

    // Ensure parent dir exists
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let json_bytes = serde_json::to_string_pretty(&final_inv)
        .map_err(|e| format!("failed to serialize recovered inventory: {}", e))?;
    fs::write(path, json_bytes)
        .map_err(|e| format!("failed to write recovered inventory: {}", e))?;

    let final_validation = check_inventory_file(path, false)?;

    Ok(HardwareRecoveryReport {
        store_path: path.to_string_lossy().to_string(),
        initial_validation,
        actions_taken,
        final_validation,
        backup_path,
        recovered: true,
        completed_at: Utc::now().to_rfc3339(),
    })
}
