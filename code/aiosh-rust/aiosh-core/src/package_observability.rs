//! Observability and telemetry reports for AIOS Package Management subsystem (PO1..PO6).

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use crate::package::{PackageFormat, PackageState};
use crate::package_service::PackageStore;
use crate::package_policy::PackageSecurityPolicy;

/// Canonical string representation for package formats.
pub fn format_to_str(format: PackageFormat) -> &'static str {
    match format {
        PackageFormat::Deb => "deb",
        PackageFormat::Apk => "apk",
        PackageFormat::Flatpak => "flatpak",
        PackageFormat::Tarball => "tarball",
    }
}

/// Canonical string representation for package states.
pub fn state_to_str(state: PackageState) -> &'static str {
    match state {
        PackageState::Available => "available",
        PackageState::Installed => "installed",
        PackageState::Upgradable => "upgradable",
        PackageState::PendingInstall => "pending_install",
        PackageState::PendingRemoval => "pending_removal",
        PackageState::Broken => "broken",
    }
}

/// Comprehensive observability report detailing package store inventory, storage, and policy compliance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageObservabilityReport {
    pub total_packages: usize,
    pub state_breakdown: BTreeMap<String, usize>,
    pub format_breakdown: BTreeMap<String, usize>,
    pub architecture_breakdown: BTreeMap<String, usize>,
    pub total_installed_size_bytes: u64,
    pub average_package_size_bytes: u64,
    pub dependency_distribution: BTreeMap<String, usize>,
    pub policy_compliant_count: usize,
    pub policy_violations_count: usize,
    pub prohibited_packages_found: Vec<String>,
    pub generated_at: String,
}

impl PackageObservabilityReport {
    /// Generates an observability report from the provided PackageStore and optional security policy (PO1..PO6).
    pub fn generate(
        store: &PackageStore,
        policy_opt: Option<&PackageSecurityPolicy>,
    ) -> Self {
        let packages = store.list_packages();
        let total_packages = packages.len();

        let mut state_breakdown = BTreeMap::new();
        let mut format_breakdown = BTreeMap::new();
        let mut architecture_breakdown = BTreeMap::new();

        let mut dependency_distribution = BTreeMap::new();
        dependency_distribution.insert("0".into(), 0);
        dependency_distribution.insert("1-5".into(), 0);
        dependency_distribution.insert("6-10".into(), 0);
        dependency_distribution.insert("11+".into(), 0);

        let mut total_installed_size_bytes: u64 = 0;
        let mut total_all_packages_bytes: u64 = 0;

        let default_policy = PackageSecurityPolicy::default();
        let policy = policy_opt.unwrap_or(&default_policy);

        let mut policy_compliant_count = 0;
        let mut policy_violations_count = 0;
        let mut prohibited_found_set = BTreeSet::new();

        for pkg in &packages {
            // PO2: State, Format, Architecture distributions
            let st = state_to_str(pkg.state).to_string();
            *state_breakdown.entry(st).or_insert(0) += 1;

            let fmt = format_to_str(pkg.format).to_string();
            *format_breakdown.entry(fmt).or_insert(0) += 1;

            *architecture_breakdown.entry(pkg.architecture.clone()).or_insert(0) += 1;

            // PO3: Installed footprint and total footprint
            if pkg.state == PackageState::Installed || pkg.state == PackageState::Upgradable {
                total_installed_size_bytes = total_installed_size_bytes.saturating_add(pkg.installed_size_bytes);
            }
            total_all_packages_bytes = total_all_packages_bytes.saturating_add(pkg.installed_size_bytes);

            // PO4: Dependency distribution histogram
            let dep_count = pkg.dependencies.len();
            let bucket = if dep_count == 0 {
                "0"
            } else if dep_count <= 5 {
                "1-5"
            } else if dep_count <= 10 {
                "6-10"
            } else {
                "11+"
            };
            *dependency_distribution.entry(bucket.to_string()).or_insert(0) += 1;

            // PO5: Security policy evaluation
            let verdict = policy.evaluate_spec(pkg);
            if verdict.allowed {
                policy_compliant_count += 1;
            }
            policy_violations_count += verdict.violations.len();

            // Check if package name or any dependencies are prohibited
            if policy.prohibited_packages.iter().any(|p| p.eq_ignore_ascii_case(&pkg.name)) {
                prohibited_found_set.insert(pkg.name.clone());
            }
            for dep in &pkg.dependencies {
                if policy.prohibited_packages.iter().any(|p| p.eq_ignore_ascii_case(&dep.name)) {
                    prohibited_found_set.insert(dep.name.clone());
                }
            }
        }

        let average_package_size_bytes = if total_packages > 0 {
            total_all_packages_bytes / (total_packages as u64)
        } else {
            0
        };

        // PO6: Read-only deterministic report with ISO timestamp
        Self {
            total_packages,
            state_breakdown,
            format_breakdown,
            architecture_breakdown,
            total_installed_size_bytes,
            average_package_size_bytes,
            dependency_distribution,
            policy_compliant_count,
            policy_violations_count,
            prohibited_packages_found: prohibited_found_set.into_iter().collect(),
            generated_at: "2026-09-04T00:00:00Z".into(),
        }
    }

    /// Serializes the report to pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("failed to serialize observability report: {}", e))
    }

    /// Generates an observability report from optional store and policy file paths with strict input validation.
    pub fn generate_from_paths<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
        store_path_opt: Option<P>,
        policy_path_opt: Option<Q>,
    ) -> Result<Self, String> {
        if let Some(ref sp) = store_path_opt {
            let sp_str = sp.as_ref().to_string_lossy();
            if sp_str.len() > 1024 || sp_str.chars().any(|c| c.is_control()) {
                return Err("store path exceeds 1024 characters or contains control characters".into());
            }
        }
        if let Some(ref pp) = policy_path_opt {
            let pp_str = pp.as_ref().to_string_lossy();
            if pp_str.len() > 1024 || pp_str.chars().any(|c| c.is_control()) {
                return Err("policy path exceeds 1024 characters or contains control characters".into());
            }
        }
        let store = match store_path_opt {
            Some(ref sp) => PackageStore::load_from_path(sp.as_ref())?,
            None => PackageStore::new(),
        };
        let policy = match policy_path_opt {
            Some(ref pp) => Some(PackageSecurityPolicy::from_file(pp.as_ref())?),
            None => None,
        };
        Ok(Self::generate(&store, policy.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observability_empty_store() {
        let store = PackageStore::empty();
        let report = PackageObservabilityReport::generate(&store, None);

        assert_eq!(report.total_packages, 0);
        assert!(report.state_breakdown.is_empty());
        assert!(report.format_breakdown.is_empty());
        assert!(report.architecture_breakdown.is_empty());
        assert_eq!(report.total_installed_size_bytes, 0);
        assert_eq!(report.average_package_size_bytes, 0);
        assert_eq!(report.dependency_distribution.get("0"), Some(&0));
        assert_eq!(report.policy_compliant_count, 0);
        assert_eq!(report.policy_violations_count, 0);
        assert!(report.prohibited_packages_found.is_empty());
    }

    #[test]
    fn test_observability_default_store() {
        let store = PackageStore::new();
        let report = PackageObservabilityReport::generate(&store, None);

        // PO1: Inventory completeness
        assert!(report.total_packages > 0);
        let sum_states: usize = report.state_breakdown.values().sum();
        let sum_formats: usize = report.format_breakdown.values().sum();
        let sum_archs: usize = report.architecture_breakdown.values().sum();
        assert_eq!(sum_states, report.total_packages);
        assert_eq!(sum_formats, report.total_packages);
        assert_eq!(sum_archs, report.total_packages);

        // PO4: Dependency distribution completeness
        let sum_deps: usize = report.dependency_distribution.values().sum();
        assert_eq!(sum_deps, report.total_packages);

        // Serialization check
        let json_res = report.to_json_pretty();
        assert!(json_res.is_ok());
    }
}
