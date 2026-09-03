//! Observability and telemetry report for Linux Distro Selection & Justification subsystem.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::distro_service::DistroStore;
use crate::distro_policy::DistroSecurityPolicy;

/// Comprehensive observability report detailing distro store status and metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistroObservabilityReport {
    pub total_profiles: usize,
    pub recommended_profile_id: Option<String>,
    pub production_ready_count: usize,
    pub policy_compliant_count: usize,
    pub average_overall_score: f32,
    pub average_security_score: f32,
    pub average_footprint_score: f32,
    pub average_binary_compatibility_score: f32,
    pub family_breakdown: BTreeMap<String, usize>,
    pub architecture_breakdown: BTreeMap<String, usize>,
    pub generated_at: String,
}

impl DistroObservabilityReport {
    /// Generates an observability report from the provided DistroStore.
    pub fn generate(store: &DistroStore, policy_opt: Option<&DistroSecurityPolicy>) -> Self {
        let profiles = store.list_profiles();
        let total_profiles = profiles.len();
        let recommended_profile_id = store.get_recommended_profile().map(|p| p.id.clone());

        let mut family_breakdown = BTreeMap::new();
        let mut architecture_breakdown = BTreeMap::new();
        let mut production_ready_count = 0;
        let mut total_overall = 0.0f32;
        let mut total_security = 0.0f32;
        let mut total_footprint = 0.0f32;
        let mut total_binary = 0.0f32;

        for p in &profiles {
            *family_breakdown.entry(format!("{:?}", p.family)).or_insert(0) += 1;
            *architecture_breakdown.entry(format!("{:?}", p.arch)).or_insert(0) += 1;

            if let Ok(eval) = store.evaluate_profile(&p.id) {
                if eval.is_production_ready {
                    production_ready_count += 1;
                }
                total_overall += eval.overall_score;
                total_security += eval.security_score;
                total_footprint += eval.footprint_score;
                total_binary += eval.binary_compatibility_score;
            }
        }

        let policy_compliant_count = if let Some(policy) = policy_opt {
            policy.filter_compliant_profiles(store).len()
        } else {
            let default_policy = DistroSecurityPolicy::default();
            default_policy.filter_compliant_profiles(store).len()
        };

        let count_f = if total_profiles > 0 { total_profiles as f32 } else { 1.0 };
        Self {
            total_profiles,
            recommended_profile_id,
            production_ready_count,
            policy_compliant_count,
            average_overall_score: (total_overall / count_f).clamp(0.0, 1.0),
            average_security_score: (total_security / count_f).clamp(0.0, 1.0),
            average_footprint_score: (total_footprint / count_f).clamp(0.0, 1.0),
            average_binary_compatibility_score: (total_binary / count_f).clamp(0.0, 1.0),
            family_breakdown,
            architecture_breakdown,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Validates internal arithmetic invariants (O1..O4).
    pub fn validate(&self) -> Result<(), String> {
        let fam_sum: usize = self.family_breakdown.values().sum();
        if fam_sum != self.total_profiles {
            return Err(format!(
                "invariant O1 violated: family breakdown sum ({}) != total profiles ({})",
                fam_sum, self.total_profiles
            ));
        }

        let arch_sum: usize = self.architecture_breakdown.values().sum();
        if arch_sum != self.total_profiles {
            return Err(format!(
                "invariant O2 violated: architecture breakdown sum ({}) != total profiles ({})",
                arch_sum, self.total_profiles
            ));
        }

        if self.production_ready_count > self.total_profiles {
            return Err(format!(
                "invariant O3 violated: production_ready_count ({}) > total_profiles ({})",
                self.production_ready_count, self.total_profiles
            ));
        }

        if self.policy_compliant_count > self.total_profiles {
            return Err(format!(
                "invariant O3 violated: policy_compliant_count ({}) > total_profiles ({})",
                self.policy_compliant_count, self.total_profiles
            ));
        }

        let scores = [
            ("overall", self.average_overall_score),
            ("security", self.average_security_score),
            ("footprint", self.average_footprint_score),
            ("binary_compatibility", self.average_binary_compatibility_score),
        ];

        for (name, s) in scores {
            if s.is_nan() || !(0.0..=1.0).contains(&s) {
                return Err(format!(
                    "invariant O4 violated: average_{}_score ({}) not in [0.0, 1.0]",
                    name, s
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_distro_observability_generation_and_invariants() {
        let store = DistroStore::new();
        let report = DistroObservabilityReport::generate(&store, None);

        assert!(report.total_profiles >= 2);
        assert_eq!(report.recommended_profile_id, Some("debian-12-minimal-x86_64".into()));
        assert!(report.production_ready_count >= 1);
        assert!(report.policy_compliant_count >= 1);

        // Verify invariants O1..O4 pass
        assert!(report.validate().is_ok());
    }

    #[test]
    fn test_distro_observability_validation_failures() {
        let report = DistroObservabilityReport {
            total_profiles: 10,
            recommended_profile_id: None,
            production_ready_count: 5,
            policy_compliant_count: 5,
            average_overall_score: 0.8,
            average_security_score: 0.8,
            average_footprint_score: 0.8,
            average_binary_compatibility_score: 0.8,
            family_breakdown: BTreeMap::new(), // Sum 0 != 10
            architecture_breakdown: BTreeMap::new(),
            generated_at: "2026-09-03T12:00:00Z".into(),
        };

        // Violates O1
        assert!(report.validate().is_err());
    }

    #[test]
    fn test_distro_observability_with_custom_policy() {
        let store = DistroStore::new();
        let mut policy = DistroSecurityPolicy::default();
        policy.min_security_score = 0.99; // Excessively high

        let report = DistroObservabilityReport::generate(&store, Some(&policy));
        assert_eq!(report.policy_compliant_count, 0);
        assert!(report.validate().is_ok());
    }

    #[test]
    fn test_distro_observability_json_roundtrip() {
        let store = DistroStore::new();
        let report = DistroObservabilityReport::generate(&store, None);

        let json_str = serde_json::to_string(&report).unwrap();
        let deserialized: DistroObservabilityReport = serde_json::from_str(&json_str).unwrap();
        assert_eq!(report, deserialized);
    }

    #[test]
    fn test_distro_observability_empty_store() {
        let store = DistroStore::empty();
        let report = DistroObservabilityReport::generate(&store, None);

        assert_eq!(report.total_profiles, 0);
        assert_eq!(report.recommended_profile_id, None);
        assert_eq!(report.average_overall_score, 0.0);
        assert!(report.validate().is_ok());
    }
}
