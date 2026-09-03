//! Security policy enforcement for Linux Distro Selection & Justification subsystem.

use serde::{Deserialize, Serialize};
use crate::distro::{DistroProfile, DistroEvaluation};
use crate::distro_service::DistroStore;

/// Security policy defining mandatory security criteria for distro profiles.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistroSecurityPolicy {
    /// Minimum required security hardening score (0.0 ..= 1.0). Default: 0.70.
    pub min_security_score: f32,
    /// Minimum required binary compatibility score (0.0 ..= 1.0). Default: 0.70.
    pub min_binary_compatibility_score: f32,
    /// Whether package repositories must enforce HTTPS. Default: true.
    pub require_https_repositories: bool,
    /// Whether package signatures are mandatory. Default: true.
    pub require_signed_packages: bool,
    /// Disallowed distribution families (e.g. experimental/unverified families).
    pub disallowed_distro_families: Vec<String>,
}

impl Default for DistroSecurityPolicy {
    fn default() -> Self {
        Self {
            min_security_score: 0.70,
            min_binary_compatibility_score: 0.70,
            require_https_repositories: true,
            require_signed_packages: true,
            disallowed_distro_families: Vec::new(),
        }
    }
}

/// Result of evaluating a distro profile against a security policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistroPolicyVerdict {
    pub profile_id: String,
    pub allowed: bool,
    pub violations: Vec<String>,
    pub evaluated_at: String,
}

impl DistroSecurityPolicy {
    /// Validates policy parameters against bounds rules.
    pub fn validate(&self) -> Result<(), String> {
        if self.min_security_score.is_nan() || !(0.0..=1.0).contains(&self.min_security_score) {
            return Err("min_security_score must be between 0.0 and 1.0".into());
        }
        if self.min_binary_compatibility_score.is_nan()
            || !(0.0..=1.0).contains(&self.min_binary_compatibility_score)
        {
            return Err("min_binary_compatibility_score must be between 0.0 and 1.0".into());
        }
        Ok(())
    }

    /// Loads security policy with environment variable overrides.
    pub fn from_env() -> Result<Self, String> {
        Self::from_source(|k| std::env::var(k).ok())
    }

    /// Loads security policy from a provider closure.
    pub fn from_source<F>(get: F) -> Result<Self, String>
    where
        F: Fn(&str) -> Option<String>,
    {
        let mut policy = Self::default();

        if let Some(sec_str) = get("AIOSH_DISTRO_MIN_SECURITY_SCORE") {
            match sec_str.trim().parse::<f32>() {
                Ok(sec) if !sec.is_nan() && (0.0..=1.0).contains(&sec) => {
                    policy.min_security_score = sec;
                }
                _ => {
                    return Err(format!(
                        "invalid AIOSH_DISTRO_MIN_SECURITY_SCORE: '{}' must be a valid float between 0.0 and 1.0",
                        sec_str
                    ));
                }
            }
        }
        if let Some(disallowed_str) = get("AIOSH_DISTRO_DISALLOWED_FAMILIES") {
            policy.disallowed_distro_families = disallowed_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        policy.validate()?;
        Ok(policy)
    }

    /// Evaluates a distro profile and its evaluation against this security policy.
    pub fn check_profile(
        &self,
        profile: &DistroProfile,
        evaluation: &DistroEvaluation,
    ) -> DistroPolicyVerdict {
        let mut violations = Vec::new();

        // P1: Minimum security score
        if evaluation.security_score < self.min_security_score {
            violations.push(format!(
                "security score {:.2} below required floor {:.2}",
                evaluation.security_score, self.min_security_score
            ));
        }

        // P2: Minimum binary compatibility score
        if evaluation.binary_compatibility_score < self.min_binary_compatibility_score {
            violations.push(format!(
                "binary compatibility score {:.2} below required floor {:.2}",
                evaluation.binary_compatibility_score, self.min_binary_compatibility_score
            ));
        }

        // P5: Disallowed families
        let family_str = format!("{:?}", profile.family);
        for disallowed in &self.disallowed_distro_families {
            if family_str.eq_ignore_ascii_case(disallowed) {
                violations.push(format!(
                    "distribution family '{}' is disallowed by security policy",
                    family_str
                ));
            }
        }

        let allowed = violations.is_empty();
        DistroPolicyVerdict {
            profile_id: profile.id.clone(),
            allowed,
            violations,
            evaluated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Evaluates all profiles in a DistroStore against this security policy.
    pub fn check_all(&self, store: &DistroStore) -> Vec<DistroPolicyVerdict> {
        let mut verdicts = Vec::new();
        for profile in store.list_profiles() {
            if let Ok(eval) = store.evaluate_profile(&profile.id) {
                verdicts.push(self.check_profile(&profile, &eval));
            }
        }
        verdicts
    }

    /// Returns only profiles that satisfy all criteria of this security policy.
    pub fn filter_compliant_profiles(&self, store: &DistroStore) -> Vec<DistroProfile> {
        let mut compliant = Vec::new();
        for profile in store.list_profiles() {
            if let Ok(eval) = store.evaluate_profile(&profile.id) {
                let verdict = self.check_profile(&profile, &eval);
                if verdict.allowed {
                    compliant.push(profile.clone());
                }
            }
        }
        compliant
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_distro_policy_default_and_validation() {
        let policy = DistroSecurityPolicy::default();
        assert!(policy.validate().is_ok());

        let mut bad_policy = DistroSecurityPolicy::default();
        bad_policy.min_security_score = 1.5;
        assert!(bad_policy.validate().is_err());

        bad_policy.min_security_score = f32::NAN;
        assert!(bad_policy.validate().is_err());
    }

    #[test]
    fn test_distro_policy_check_profile() {
        let store = DistroStore::new();
        let policy = DistroSecurityPolicy::default();

        let verdicts = policy.check_all(&store);
        assert!(!verdicts.is_empty());

        // Debian reference should pass standard policy
        let debian_verdict = verdicts.iter().find(|v| v.profile_id == "debian-12-minimal-x86_64").unwrap();
        assert!(debian_verdict.allowed);

        // Disallow Debian family
        let mut strict_policy = DistroSecurityPolicy::default();
        strict_policy.disallowed_distro_families.push("Debian".into());
        let strict_verdicts = strict_policy.check_all(&store);
        let debian_strict = strict_verdicts.iter().find(|v| v.profile_id == "debian-12-minimal-x86_64").unwrap();
        assert!(!debian_strict.allowed);
        assert!(debian_strict.violations.iter().any(|v| v.contains("disallowed")));
    }

    #[test]
    fn test_distro_policy_filter_compliant() {
        let store = DistroStore::new();
        let mut policy = DistroSecurityPolicy::default();
        policy.min_security_score = 0.99; // Excessively high

        let compliant = policy.filter_compliant_profiles(&store);
        assert!(compliant.is_empty());
    }

    #[test]
    fn test_distro_policy_from_source_overrides() {
        let mut map = std::collections::HashMap::new();
        map.insert("AIOSH_DISTRO_MIN_SECURITY_SCORE", "0.85".to_string());
        map.insert("AIOSH_DISTRO_DISALLOWED_FAMILIES", "Alpine,Arch".to_string());

        let policy = DistroSecurityPolicy::from_source(|k| map.get(k).cloned()).unwrap();
        assert_eq!(policy.min_security_score, 0.85);
        assert_eq!(policy.disallowed_distro_families.len(), 2);
        assert_eq!(policy.disallowed_distro_families[0], "Alpine");
        assert_eq!(policy.disallowed_distro_families[1], "Arch");
    }

    #[test]
    fn test_distro_policy_verdict_serialization() {
        let verdict = DistroPolicyVerdict {
            profile_id: "test-id".into(),
            allowed: false,
            violations: vec!["Violation A".into()],
            evaluated_at: "2026-09-03T12:00:00Z".into(),
        };
        let json_str = serde_json::to_string(&verdict).unwrap();
        let deserialized: DistroPolicyVerdict = serde_json::from_str(&json_str).unwrap();
        assert_eq!(verdict, deserialized);
    }

    #[test]
    fn test_distro_policy_hardening_env_rejection() {
        let mut map = std::collections::HashMap::new();
        map.insert("AIOSH_DISTRO_MIN_SECURITY_SCORE", "not-a-number".to_string());
        assert!(DistroSecurityPolicy::from_source(|k| map.get(k).cloned()).is_err());

        let mut map = std::collections::HashMap::new();
        map.insert("AIOSH_DISTRO_MIN_SECURITY_SCORE", "1.5".to_string());
        assert!(DistroSecurityPolicy::from_source(|k| map.get(k).cloned()).is_err());
    }
}
