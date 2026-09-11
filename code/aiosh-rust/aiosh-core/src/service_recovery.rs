//! Recovery, health check, and deep validation for the Init & Service Supervision subsystem.
//!
//! Provides automated non-destructive self-healing, timestamped quarantine of damaged
//! stores, and deep validation reports satisfying invariants SR1..SR5.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::service::{validate_service_name, validate_service_spec, validate_service_status};
use crate::service_service::ServiceStore;

/// Action taken during store loading and corruption recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceRecoveryAction {
    LoadedExisting,
    CreatedDefaultFresh,
    RecoveredFromBackup { backup_path: String, reason: String },
}

/// Comprehensive deep validation report across managed service specifications and statuses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceValidationReport {
    pub store_path: String,
    pub total_services: usize,
    pub valid_services: usize,
    pub invalid_services: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}

impl ServiceValidationReport {
    /// Validates mathematical and consistency invariants SR1..SR3.
    pub fn validate_invariants(&self) -> Result<(), String> {
        // SR1: valid + invalid == total
        if self.valid_services + self.invalid_services != self.total_services {
            return Err(format!(
                "invariant SR1 violated: valid ({}) + invalid ({}) != total ({})",
                self.valid_services, self.invalid_services, self.total_services
            ));
        }

        // SR2: healthy == (errors.is_empty() && invalid_services == 0)
        let expected_healthy = self.errors.is_empty() && self.invalid_services == 0;
        if self.healthy != expected_healthy {
            return Err(format!(
                "invariant SR2 violated: healthy ({}) != expected ({})",
                self.healthy, expected_healthy
            ));
        }

        // SR3: invalid_services > 0 => errors.len() >= invalid_services
        if self.invalid_services > 0 && self.errors.len() < self.invalid_services {
            return Err(format!(
                "invariant SR3 violated: error count ({}) < invalid service count ({})",
                self.errors.len(), self.invalid_services
            ));
        }

        Ok(())
    }
}

/// Validates all service specifications, statuses, and dependency graphs in a ServiceStore.
pub fn validate_service_store(store: &ServiceStore, store_path: &Path) -> ServiceValidationReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut valid_services = 0;
    let mut invalid_services = 0;

    let total_services = store.services.len();

    // Capacity check
    if total_services > 10_000 {
        errors.push(format!(
            "store exceeds maximum capacity of 10,000 services (was {})",
            total_services
        ));
    }

    // Validate each service specification in alphabetical order
    let mut sorted_entries: Vec<(&String, &crate::service::ServiceSpec)> = store.services.iter().collect();
    sorted_entries.sort_by(|a, b| a.0.cmp(b.0));

    for (key, spec) in sorted_entries {
        let mut service_errors = Vec::new();

        if key != &spec.name {
            service_errors.push(format!(
                "store key '{}' does not match service specification name '{}'",
                key, spec.name
            ));
        }

        if let Err(name_err) = validate_service_name(&spec.name) {
            service_errors.push(format!("invalid service name syntax: {}", name_err));
        }

        if let Err(spec_errs) = validate_service_spec(spec) {
            service_errors.extend(spec_errs);
        }

        if service_errors.is_empty() {
            valid_services += 1;
        } else {
            invalid_services += 1;
            for err in service_errors {
                errors.push(format!("service '{}': {}", spec.name, err));
            }
        }
    }

    // Validate runtime statuses
    for (key, status) in &store.statuses {
        if key != &status.name {
            errors.push(format!(
                "status key '{}' does not match service status name '{}'",
                key, status.name
            ));
        }
        if let Err(status_errs) = validate_service_status(status) {
            for err in status_errs {
                errors.push(format!("status '{}': {}", status.name, err));
            }
        }
        if !store.services.contains_key(&status.name) {
            warnings.push(format!(
                "status exists for service '{}' which has no registered specification",
                status.name
            ));
        }
    }

    // Topological dependency graph acyclicity validation (CS3)
    for service_name in store.services.keys() {
        if let Err(order_err) = store.plan_service_order(service_name) {
            errors.push(format!("service '{}' dependency planning failure: {}", service_name, order_err));
        }
    }

    let healthy = errors.is_empty() && invalid_services == 0;

    let report = ServiceValidationReport {
        store_path: store_path.to_string_lossy().to_string(),
        total_services,
        valid_services,
        invalid_services,
        errors,
        warnings,
        healthy,
        evaluated_at: chrono::Utc::now().to_rfc3339(),
    };

    let _ = report.validate_invariants();
    report
}

/// Helper function to create a timestamped quarantine backup file (<path>.corrupt.<ts>.bak).
fn create_backup_file(path: &Path) -> PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let base_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("services.json");

    let mut backup_name = format!("{}.corrupt.{}.bak", base_name, ts);
    let mut backup_path = path.with_file_name(&backup_name);

    let mut counter = 1;
    while backup_path.exists() && counter < 10_000 {
        backup_name = format!("{}.corrupt.{}_{}.bak", base_name, ts, counter);
        backup_path = path.with_file_name(&backup_name);
        counter += 1;
    }

    // Attempt rename first; if it fails (e.g. cross-filesystem), copy and remove.
    if std::fs::rename(path, &backup_path).is_err() {
        if std::fs::copy(path, &backup_path).is_ok() {
            let _ = std::fs::remove_file(path);
        }
    }

    backup_path
}

/// Recovers a ServiceStore from disk with non-destructive timestamped quarantine on corruption.
pub fn recover_service_store_with_backup(path: &Path) -> (ServiceStore, Option<PathBuf>) {
    if !path.exists() {
        let store = ServiceStore::new();
        let _ = store.save_to_path(path);
        return (store, None);
    }

    match ServiceStore::load_from_path(path) {
        Ok(store) => {
            let report = validate_service_store(&store, path);
            if report.healthy {
                (store, None)
            } else {
                let backup_path = create_backup_file(path);
                let fresh_store = ServiceStore::new();
                let _ = fresh_store.save_to_path(path);
                (fresh_store, Some(backup_path))
            }
        }
        Err(_) => {
            let backup_path = create_backup_file(path);
            let fresh_store = ServiceStore::new();
            let _ = fresh_store.save_to_path(path);
            (fresh_store, Some(backup_path))
        }
    }
}

/// High-level entrypoint: loads, validates, and optionally repairs a service store.
pub fn load_or_recover(
    path: &Path,
) -> Result<(ServiceStore, ServiceValidationReport, bool, Option<PathBuf>), String> {
    if !path.exists() {
        let store = ServiceStore::new();
        store.save_to_path(path)?;
        let report = validate_service_store(&store, path);
        return Ok((store, report, true, None));
    }

    match ServiceStore::load_from_path(path) {
        Ok(store) => {
            let report = validate_service_store(&store, path);
            if report.healthy {
                Ok((store, report, false, None))
            } else {
                let (recovered_store, backup_path) = recover_service_store_with_backup(path);
                let fresh_report = validate_service_store(&recovered_store, path);
                Ok((recovered_store, fresh_report, true, backup_path))
            }
        }
        Err(_) => {
            let (recovered_store, backup_path) = recover_service_store_with_backup(path);
            let fresh_report = validate_service_store(&recovered_store, path);
            Ok((recovered_store, fresh_report, true, backup_path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::{
        ServiceDependency, ServiceDependencyType, ServiceRestartPolicy, ServiceSpec,
        ServiceStartupMode, ServiceType,
    };
    use std::collections::BTreeMap;

    #[test]
    fn test_validate_default_store_healthy() {
        let store = ServiceStore::new();
        let report = validate_service_store(&store, Path::new("/var/lib/aios/services.json"));

        assert!(report.healthy);
        assert_eq!(report.invalid_services, 0);
        assert!(report.errors.is_empty());
        assert_eq!(report.valid_services, store.services.len());
        assert_eq!(report.total_services, store.services.len());
        assert!(report.validate_invariants().is_ok());
    }

    #[test]
    fn test_validate_store_with_invalid_service() {
        let mut store = ServiceStore::new();
        // Insert invalid spec with forbidden name and traversal
        let bad_spec = ServiceSpec {
            name: "../bad_service".into(),
            description: "Invalid Service".into(),
            exec_start: "../bin/bad".into(),
            exec_stop: None,
            exec_reload: None,
            service_type: ServiceType::Simple,
            restart_policy: ServiceRestartPolicy::No,
            startup_mode: ServiceStartupMode::Enabled,
            user: None,
            group: None,
            working_dir: None,
            environment: BTreeMap::new(),
            dependencies: vec![],
            timeout_start_secs: 30,
            timeout_stop_secs: 30,
        };
        store.services.insert("../bad_service".into(), bad_spec);

        let report = validate_service_store(&store, Path::new("/tmp/test.json"));
        assert!(!report.healthy);
        assert_eq!(report.invalid_services, 1);
        assert!(!report.errors.is_empty());
        assert!(report.validate_invariants().is_ok());
    }

    #[test]
    fn test_validate_store_with_cyclic_dependency() {
        let mut store = ServiceStore::empty();
        let a = ServiceSpec {
            name: "service-a.service".into(),
            description: "Service A".into(),
            exec_start: "/bin/true".into(),
            exec_stop: None,
            exec_reload: None,
            service_type: ServiceType::Simple,
            restart_policy: ServiceRestartPolicy::No,
            startup_mode: ServiceStartupMode::Enabled,
            user: None,
            group: None,
            working_dir: None,
            environment: BTreeMap::new(),
            dependencies: vec![ServiceDependency {
                name: "service-b.service".into(),
                dependency_type: ServiceDependencyType::Requires,
                optional: false,
            }],
            timeout_start_secs: 30,
            timeout_stop_secs: 30,
        };
        let b = ServiceSpec {
            name: "service-b.service".into(),
            description: "Service B".into(),
            exec_start: "/bin/true".into(),
            exec_stop: None,
            exec_reload: None,
            service_type: ServiceType::Simple,
            restart_policy: ServiceRestartPolicy::No,
            startup_mode: ServiceStartupMode::Enabled,
            user: None,
            group: None,
            working_dir: None,
            environment: BTreeMap::new(),
            dependencies: vec![ServiceDependency {
                name: "service-a.service".into(),
                dependency_type: ServiceDependencyType::Requires,
                optional: false,
            }],
            timeout_start_secs: 30,
            timeout_stop_secs: 30,
        };
        store.services.insert("service-a.service".into(), a);
        store.services.insert("service-b.service".into(), b);

        let report = validate_service_store(&store, Path::new("/tmp/cycle.json"));
        assert!(!report.healthy);
        assert!(report.errors.iter().any(|e| e.contains("cyclic dependency")));
        assert!(report.validate_invariants().is_ok());
    }

    #[test]
    fn test_recover_corrupt_store() {
        let temp_dir = std::env::temp_dir().join(format!("aios_rec_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let store_path = temp_dir.join("services.json");

        // Write corrupt JSON
        std::fs::write(&store_path, b"{ this is corrupt JSON!").unwrap();

        let (recovered, backup) = recover_service_store_with_backup(&store_path);
        assert!(backup.is_some());
        let bak = backup.unwrap();
        assert!(bak.exists());
        assert!(bak.to_string_lossy().contains(".corrupt."));

        let report = validate_service_store(&recovered, &store_path);
        assert!(report.healthy);
        assert_eq!(report.invalid_services, 0);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
