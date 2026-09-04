//! Observability and telemetry reports for Linux Base Image Build subsystem.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::base_image_service::ImageStore;
use crate::base_image_policy::BaseImageSecurityPolicy;

/// Telemetry and status report detailing base image store status and metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseImageObservabilityReport {
    pub total_images: usize,
    pub format_breakdown: BTreeMap<String, usize>,
    pub architecture_breakdown: BTreeMap<String, usize>,
    pub distro_breakdown: BTreeMap<String, usize>,
    pub policy_compliant_count: usize,
    pub total_size_budget_bytes: u64,
    pub average_size_budget_bytes: u64,
    pub kernel_versions: Vec<String>,
    pub generated_at: String,
}

impl BaseImageObservabilityReport {
    /// Generates an observability report from the provided ImageStore and optional policy.
    pub fn generate(store: &ImageStore, policy_opt: Option<&BaseImageSecurityPolicy>) -> Self {
        let images = store.list_images();
        let total_images = images.len();

        let mut format_breakdown = BTreeMap::new();
        let mut architecture_breakdown = BTreeMap::new();
        let mut distro_breakdown = BTreeMap::new();
        let mut total_size_budget_bytes = 0u64;
        let mut kernel_versions_set = std::collections::BTreeSet::new();

        for img in &images {
            *format_breakdown.entry(img.format.to_string()).or_insert(0) += 1;
            *architecture_breakdown.entry(img.rootfs.architecture.clone()).or_insert(0) += 1;
            *distro_breakdown.entry(img.rootfs.distro_id.clone()).or_insert(0) += 1;
            total_size_budget_bytes = total_size_budget_bytes.saturating_add(img.rootfs.size_budget_bytes);
            kernel_versions_set.insert(img.kernel.version.clone());
        }

        let policy_compliant_count = if let Some(policy) = policy_opt {
            policy.filter_compliant_manifests(store).len()
        } else {
            let default_policy = BaseImageSecurityPolicy::default();
            default_policy.filter_compliant_manifests(store).len()
        };

        let average_size_budget_bytes = if total_images > 0 {
            total_size_budget_bytes / (total_images as u64)
        } else {
            0
        };

        Self {
            total_images,
            format_breakdown,
            architecture_breakdown,
            distro_breakdown,
            policy_compliant_count,
            total_size_budget_bytes,
            average_size_budget_bytes,
            kernel_versions: kernel_versions_set.into_iter().collect(),
            generated_at: "2026-09-04T00:00:00Z".into(),
        }
    }

    /// Validates internal arithmetic invariants (OB1..OB5).
    pub fn validate(&self) -> Result<(), String> {
        let fmt_sum: usize = self.format_breakdown.values().sum();
        if fmt_sum != self.total_images {
            return Err(format!(
                "invariant OB1 violated: format breakdown sum ({}) != total images ({})",
                fmt_sum, self.total_images
            ));
        }

        let arch_sum: usize = self.architecture_breakdown.values().sum();
        if arch_sum != self.total_images {
            return Err(format!(
                "invariant OB2 violated: architecture breakdown sum ({}) != total images ({})",
                arch_sum, self.total_images
            ));
        }

        let distro_sum: usize = self.distro_breakdown.values().sum();
        if distro_sum != self.total_images {
            return Err(format!(
                "invariant OB3 violated: distro breakdown sum ({}) != total images ({})",
                distro_sum, self.total_images
            ));
        }

        if self.policy_compliant_count > self.total_images {
            return Err(format!(
                "invariant OB4 violated: policy_compliant_count ({}) > total_images ({})",
                self.policy_compliant_count, self.total_images
            ));
        }

        if self.total_images > 0 {
            let expected_avg = self.total_size_budget_bytes / (self.total_images as u64);
            if self.average_size_budget_bytes != expected_avg {
                return Err(format!(
                    "invariant OB5 violated: average_size_budget_bytes ({}) != calculated expected ({})",
                    self.average_size_budget_bytes, expected_avg
                ));
            }
        } else if self.average_size_budget_bytes != 0 || self.total_size_budget_bytes != 0 {
            return Err("invariant OB5 violated: empty store must have zero size budgets".into());
        }

        // Hardening & Sanitization: check map sizes and reject control characters
        if self.format_breakdown.len() > 16 {
            return Err("format_breakdown exceeds maximum capacity of 16 entries".into());
        }
        for k in self.format_breakdown.keys() {
            if k.chars().any(|c| c.is_control()) || k.len() > 64 {
                return Err(format!("format_breakdown contains malformed key: '{}'", k));
            }
        }

        if self.architecture_breakdown.len() > 64 {
            return Err("architecture_breakdown exceeds maximum capacity of 64 entries".into());
        }
        for k in self.architecture_breakdown.keys() {
            if k.chars().any(|c| c.is_control()) || k.len() > 64 {
                return Err(format!("architecture_breakdown contains malformed key: '{}'", k));
            }
        }

        if self.distro_breakdown.len() > 256 {
            return Err("distro_breakdown exceeds maximum capacity of 256 entries".into());
        }
        for k in self.distro_breakdown.keys() {
            if k.chars().any(|c| c.is_control()) || k.len() > 128 {
                return Err(format!("distro_breakdown contains malformed key: '{}'", k));
            }
        }

        if self.kernel_versions.len() > 256 {
            return Err("kernel_versions exceeds maximum capacity of 256 entries".into());
        }
        for k in &self.kernel_versions {
            if k.chars().any(|c| c.is_control()) || k.len() > 128 {
                return Err(format!("kernel_versions contains malformed entry: '{}'", k));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_report_generation_reference_store() {
        let store = ImageStore::new();
        let report = BaseImageObservabilityReport::generate(&store, None);
        assert_eq!(report.total_images, 4);
        assert!(report.validate().is_ok());
        assert_eq!(report.format_breakdown.get("raw"), Some(&1));
        assert_eq!(report.format_breakdown.get("qcow2"), Some(&1));
        assert_eq!(report.format_breakdown.get("iso"), Some(&1));
        assert_eq!(report.format_breakdown.get("tarball"), Some(&1));
        assert_eq!(report.policy_compliant_count, 4);
        assert!(report.total_size_budget_bytes > 0);
        assert_eq!(report.average_size_budget_bytes, report.total_size_budget_bytes / 4);
        assert_eq!(report.kernel_versions.len(), 2);
    }

    #[test]
    fn test_report_generation_empty_store() {
        let store = ImageStore::empty();
        let report = BaseImageObservabilityReport::generate(&store, None);
        assert_eq!(report.total_images, 0);
        assert_eq!(report.policy_compliant_count, 0);
        assert_eq!(report.total_size_budget_bytes, 0);
        assert_eq!(report.average_size_budget_bytes, 0);
        assert!(report.validate().is_ok());
    }

    #[test]
    fn test_report_invariants_violations() {
        let store = ImageStore::new();
        let report = BaseImageObservabilityReport::generate(&store, None);

        // OB1 violation
        let mut bad_report = report.clone();
        bad_report.format_breakdown.insert("raw".into(), 99);
        assert!(bad_report.validate().is_err());

        // OB2 violation
        let mut bad_report2 = report.clone();
        bad_report2.architecture_breakdown.insert("x86_64".into(), 0);
        assert!(bad_report2.validate().is_err());

        // OB3 violation
        let mut bad_report3 = report.clone();
        bad_report3.distro_breakdown.clear();
        assert!(bad_report3.validate().is_err());

        // OB4 violation
        let mut bad_report4 = report.clone();
        bad_report4.policy_compliant_count = 100;
        assert!(bad_report4.validate().is_err());

        // OB5 violation
        let mut bad_report5 = report.clone();
        bad_report5.average_size_budget_bytes += 1;
        assert!(bad_report5.validate().is_err());
    }

    #[test]
    fn test_report_serialization_roundtrip() {
        let store = ImageStore::new();
        let report = BaseImageObservabilityReport::generate(&store, None);
        let serialized = serde_json::to_string(&report).expect("serialize");
        let deserialized: BaseImageObservabilityReport = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(report, deserialized);
    }

    #[test]
    fn test_report_hardening_bounds_and_poisoning() {
        let store = ImageStore::new();
        let report = BaseImageObservabilityReport::generate(&store, None);

        // Control char in map key
        let mut bad_key = report.clone();
        bad_key.format_breakdown.insert("raw\x00poison".into(), 1);
        bad_key.format_breakdown.remove("raw");
        assert!(bad_key.validate().is_err());

        // Control char in kernel version
        let mut bad_kernel = report.clone();
        bad_kernel.kernel_versions.push("6.1.0\x07".into());
        assert!(bad_kernel.validate().is_err());

        // Oversized map capacity
        let mut bad_cap = report.clone();
        for i in 0..20 {
            bad_cap.format_breakdown.insert(format!("fmt_{}", i), 0);
        }
        assert!(bad_cap.validate().is_err());
    }
}
