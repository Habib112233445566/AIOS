//! Capability Model Recovery, Health Check, and Deep Validation Subsystem (CAPREC1..CAPREC6).
//!
//! Provides automated non-destructive self-healing, timestamped quarantine of damaged
//! capability stores, and deep structural validation reports verifying lineage,
//! monotonic attenuation, and capacity constraints.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::capability_service::CapabilityService;

/// Action taken during capability store loading, validation, and corruption recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityRecoveryAction {
    LoadedExisting,
    CreatedDefaultFresh,
    RecoveredFromBackup { backup_path: String, reason: String },
}

/// Comprehensive deep validation report across managed capabilities and delegation lineages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityValidationReport {
    pub store_path: String,
    pub total_capabilities: usize,
    pub valid_capabilities: usize,
    pub invalid_capabilities: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}

impl CapabilityValidationReport {
    /// Validates mathematical and consistency invariants CAPREC1..CAPREC2.
    pub fn validate_invariants(&self) -> Result<(), String> {
        // CAPREC1: valid + invalid == total
        if self.valid_capabilities + self.invalid_capabilities != self.total_capabilities {
            return Err(format!(
                "invariant CAPREC1 violated: valid ({}) + invalid ({}) != total ({})",
                self.valid_capabilities, self.invalid_capabilities, self.total_capabilities
            ));
        }

        // CAPREC2: healthy == (errors.is_empty() && invalid_capabilities == 0)
        let expected_healthy = self.errors.is_empty() && self.invalid_capabilities == 0;
        if self.healthy != expected_healthy {
            return Err(format!(
                "invariant CAPREC2 violated: healthy ({}) != expected ({})",
                self.healthy, expected_healthy
            ));
        }

        Ok(())
    }
}

/// Creates a timestamped backup copy of a damaged or corrupted capability store file (CAPREC5).
pub fn create_backup_file(path: &Path) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%6f").to_string();
    let file_name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "capability_store.json".to_string());
    let parent = path.parent();

    let mut counter = 0;
    let mut backup_path;
    loop {
        let suffix = if counter == 0 {
            format!("{}.bak.{}", file_name, timestamp)
        } else {
            format!("{}.bak.{}.{}", file_name, timestamp, counter)
        };
        backup_path = match parent {
            Some(p) => p.join(&suffix),
            None => PathBuf::from(&suffix),
        };
        if !backup_path.exists() || counter >= 10_000 {
            break;
        }
        counter += 1;
    }

    if path.exists() {
        let _ = fs::copy(path, &backup_path);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&backup_path, fs::Permissions::from_mode(0o600));
        }
    }

    backup_path
}

/// Validates a capability service against CAPREC1..CAPREC4 invariants.
pub fn validate_capability_store(
    service: &CapabilityService,
    store_path: &Path,
) -> CapabilityValidationReport {
    let mut errors = Vec::new();
    let warnings = Vec::new();
    let mut valid_capabilities = 0;
    let mut invalid_capabilities = 0;

    let capabilities = service.capabilities();
    let total_capabilities = capabilities.len();

    // 1. Capacity check
    let max_cap = service.config().max_capabilities;
    if max_cap > 0 && total_capabilities > max_cap {
        errors.push(format!(
            "registry capacity exceeded: {} > {}",
            total_capabilities, max_cap
        ));
    }

    // 2. Individual capability validation & lineage checks
    for cap in capabilities.values() {
        let mut cap_valid = true;

        // ID format check
        if cap.id.trim().is_empty() || !cap.id.starts_with("cap_") || cap.id.len() > 128 {
            errors.push(format!("capability '{}' has invalid id format", cap.id));
            cap_valid = false;
        }

        // Subject & Issuer checks
        if cap.subject.trim().is_empty() || cap.subject.len() > 256 {
            errors.push(format!("capability '{}' has invalid subject", cap.id));
            cap_valid = false;
        }
        if cap.issuer.trim().is_empty() || cap.issuer.len() > 256 {
            errors.push(format!("capability '{}' has invalid issuer", cap.id));
            cap_valid = false;
        }

        // Rights non-empty check
        if cap.rights.is_empty() {
            errors.push(format!("capability '{}' has empty rights set", cap.id));
            cap_valid = false;
        }

        // Temporal constraints checks
        if let Some(ref nb_str) = cap.constraints.not_before {
            if DateTime::parse_from_rfc3339(nb_str).is_err() {
                errors.push(format!("capability '{}' has invalid not_before timestamp", cap.id));
                cap_valid = false;
            }
        }
        if let Some(ref exp_str) = cap.constraints.expires_at {
            if DateTime::parse_from_rfc3339(exp_str).is_err() {
                errors.push(format!("capability '{}' has invalid expires_at timestamp", cap.id));
                cap_valid = false;
            }
        }
        if let (Some(ref nb_str), Some(ref exp_str)) = (&cap.constraints.not_before, &cap.constraints.expires_at) {
            if let (Ok(nb), Ok(exp)) = (DateTime::parse_from_rfc3339(nb_str), DateTime::parse_from_rfc3339(exp_str)) {
                if nb > exp {
                    errors.push(format!("capability '{}' has not_before > expires_at", cap.id));
                    cap_valid = false;
                }
            }
        }

        // Lineage and Monotonic Attenuation checks (CAPREC3, CAPREC4)
        if let Some(ref parent_id) = cap.parent_id {
            match capabilities.get(parent_id) {
                None => {
                    errors.push(format!(
                        "capability '{}' references non-existent parent '{}'",
                        cap.id, parent_id
                    ));
                    cap_valid = false;
                }
                Some(parent) => {
                    // Check cycle in lineage
                    let mut visited = HashSet::new();
                    visited.insert(cap.id.clone());
                    let mut curr = parent_id.clone();
                    let mut cycle_found = false;
                    while let Some(ancestor) = capabilities.get(&curr) {
                        if !visited.insert(curr.clone()) {
                            cycle_found = true;
                            break;
                        }
                        if let Some(ref p) = ancestor.parent_id {
                            curr = p.clone();
                        } else {
                            break;
                        }
                    }
                    if cycle_found {
                        errors.push(format!("cycle detected in lineage for capability '{}'", cap.id));
                        cap_valid = false;
                    }

                    // CAPREC4: Monotonic Attenuation - child rights must be subset of parent rights
                    let parent_rights: HashSet<_> = parent.rights.iter().collect();
                    for r in &cap.rights {
                        if !parent_rights.contains(r) {
                            errors.push(format!(
                                "privilege escalation in '{}': right {:?} not present in parent '{}'",
                                cap.id, r, parent_id
                            ));
                            cap_valid = false;
                        }
                    }

                    // CAPREC4: Scope confinement
                    if !parent.matches_scope(&cap.scope) {
                        errors.push(format!(
                            "scope expansion in '{}': child scope {:?} not confined by parent scope {:?}",
                            cap.id, cap.scope, parent.scope
                        ));
                        cap_valid = false;
                    }
                }
            }
        }

        if cap_valid {
            valid_capabilities += 1;
        } else {
            invalid_capabilities += 1;
        }
    }

    let healthy = errors.is_empty() && invalid_capabilities == 0;
    CapabilityValidationReport {
        store_path: store_path.to_string_lossy().to_string(),
        total_capabilities,
        valid_capabilities,
        invalid_capabilities,
        errors,
        warnings,
        healthy,
        evaluated_at: Utc::now().to_rfc3339(),
    }
}

/// Recovers a capability store from disk, quarantining damaged stores non-destructively (CAPREC5).
pub fn recover_capability_store(
    store_path: &Path,
) -> Result<(CapabilityService, CapabilityRecoveryAction, CapabilityValidationReport), String> {
    if !store_path.exists() {
        let service = CapabilityService::new();
        service.save_to_path(store_path)?;
        let report = validate_capability_store(&service, store_path);
        return Ok((service, CapabilityRecoveryAction::CreatedDefaultFresh, report));
    }

    // Try loading existing store
    match CapabilityService::load_or_create(store_path) {
        Ok(service) => {
            let report = validate_capability_store(&service, store_path);
            if report.healthy {
                Ok((service, CapabilityRecoveryAction::LoadedExisting, report))
            } else {
                // Damaged content in store: quarantine and recreate
                let backup_path = create_backup_file(store_path);
                let fresh_service = CapabilityService::new();
                fresh_service.save_to_path(store_path)?;
                let fresh_report = validate_capability_store(&fresh_service, store_path);
                Ok((
                    fresh_service,
                    CapabilityRecoveryAction::RecoveredFromBackup {
                        backup_path: backup_path.to_string_lossy().to_string(),
                        reason: format!("store failed validation with {} errors", report.errors.len()),
                    },
                    fresh_report,
                ))
            }
        }
        Err(err) => {
            // Corrupt file on disk: quarantine and recreate
            let backup_path = create_backup_file(store_path);
            let fresh_service = CapabilityService::new();
            fresh_service.save_to_path(store_path)?;
            let fresh_report = validate_capability_store(&fresh_service, store_path);
            Ok((
                fresh_service,
                CapabilityRecoveryAction::RecoveredFromBackup {
                    backup_path: backup_path.to_string_lossy().to_string(),
                    reason: format!("corrupted store file: {}", err),
                },
                fresh_report,
            ))
        }
    }
}
