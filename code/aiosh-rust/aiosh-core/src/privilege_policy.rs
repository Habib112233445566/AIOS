//! Security Policy Subsystem for Privilege Escalation Prevention (PRIVESCPOL1..PRIVESCPOL6).
//!
//! Provides declarative governance over privilege transition requests,
//! evaluating disallowed elevation tiers, prohibited capabilities,
//! actor tier ceilings, and mandatory grant requirements.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::privilege_data_model::{
    PrivilegeCapability, PrivilegeLevel, PrivilegeTransitionRequest,
    PRIVESC_ERR_KERNEL_TIER_IMMUTABLE, PRIVESC_ERR_UNAUTHORIZED_ELEVATION,
};

/// Error code: Invalid policy parameter or bounds violation.
pub const PRIVESCPOL_ERR_VALIDATION: &str = "PRIVESCPOL_ERR_VALIDATION";

/// Error code: Privilege elevation or capability request denied under policy.
pub const PRIVESCPOL_ERR_DENIED: &str = "PRIVESCPOL_ERR_DENIED";

/// Error code: Filesystem or I/O failure.
pub const PRIVESCPOL_ERR_IO: &str = "PRIVESCPOL_ERR_IO";

/// Error code: Deserialization or JSON parsing failure.
pub const PRIVESCPOL_ERR_PARSE: &str = "PRIVESCPOL_ERR_PARSE";

/// Maximum allowed length for policy version string.
pub const MAX_PRIVILEGE_POLICY_VERSION_LEN: usize = 32;

/// Maximum number of disallowed elevation targets in policy.
pub const MAX_DISALLOWED_TARGETS: usize = 16;

/// Maximum number of prohibited capabilities in policy.
pub const MAX_PROHIBITED_CAPABILITIES: usize = 32;

/// Maximum number of actor tier ceilings.
pub const MAX_ACTOR_CEILINGS: usize = 256;

/// Maximum byte size of a persisted privilege security policy file (64 KiB).
pub const MAX_PRIVILEGE_SECURITY_POLICY_BYTES: u64 = 64 * 1024;

/// Enforcement mode governing Privilege Escalation Prevention (PRIVESCPOL1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegePolicyMode {
    /// Enforcing: Policy violations strictly reject transitions fail-closed.
    Enforcing,
    /// Permissive: Policy violations emit warnings but allow transition for auditing.
    Permissive,
    /// Disabled: Policy evaluation is bypassed.
    Disabled,
}

impl Default for PrivilegePolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Evaluation verdict for privilege transition requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum PrivilegePolicyVerdict {
    /// Action is permitted under security policy.
    Permit,
    /// Action is permitted with an auditable security warning.
    PermitWithWarning { warning: String },
    /// Action is denied fail-closed under security policy.
    Deny { reason: String, code: String },
}

/// Declarative security policy for Privilege Escalation Prevention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeSecurityPolicy {
    pub version: String,
    pub mode: PrivilegePolicyMode,
    pub disallowed_elevation_targets: Vec<PrivilegeLevel>,
    pub prohibited_capabilities: Vec<PrivilegeCapability>,
    pub require_grant_token: bool,
    pub max_grant_duration_seconds: u64,
    pub actor_tier_ceilings: HashMap<String, PrivilegeLevel>,
}

impl Default for PrivilegeSecurityPolicy {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            mode: PrivilegePolicyMode::Enforcing,
            disallowed_elevation_targets: vec![PrivilegeLevel::SystemKernel],
            prohibited_capabilities: vec![PrivilegeCapability::KernelModuleLoad],
            require_grant_token: true,
            max_grant_duration_seconds: 3600,
            actor_tier_ceilings: HashMap::new(),
        }
    }
}

impl PrivilegeSecurityPolicy {
    /// Validates the policy against structural invariants, capacity ceilings, and rules.
    pub fn validate(&self) -> Result<(), String> {
        let v = self.version.trim();
        if v.is_empty() || v.len() > MAX_PRIVILEGE_POLICY_VERSION_LEN {
            return Err(format!("{}: version must be between 1 and {} chars", PRIVESCPOL_ERR_VALIDATION, MAX_PRIVILEGE_POLICY_VERSION_LEN));
        }

        if self.disallowed_elevation_targets.len() > MAX_DISALLOWED_TARGETS {
            return Err(format!("{}: disallowed elevation targets count exceeds {}", PRIVESCPOL_ERR_VALIDATION, MAX_DISALLOWED_TARGETS));
        }

        // PRIVESCPOL1: SystemKernel tier MUST always be disallowed
        if !self.disallowed_elevation_targets.contains(&PrivilegeLevel::SystemKernel) {
            return Err(format!("{}: SystemKernel tier cannot be removed from disallowed elevation targets", PRIVESC_ERR_KERNEL_TIER_IMMUTABLE));
        }

        if self.prohibited_capabilities.len() > MAX_PROHIBITED_CAPABILITIES {
            return Err(format!("{}: prohibited capabilities count exceeds {}", PRIVESCPOL_ERR_VALIDATION, MAX_PROHIBITED_CAPABILITIES));
        }

        if self.actor_tier_ceilings.len() > MAX_ACTOR_CEILINGS {
            return Err(format!("{}: actor tier ceilings count exceeds {}", PRIVESCPOL_ERR_VALIDATION, MAX_ACTOR_CEILINGS));
        }

        for (actor, ceiling) in &self.actor_tier_ceilings {
            if actor.trim().is_empty() || actor.chars().any(|c| c.is_control()) {
                return Err(format!("{}: invalid actor identifier in ceiling map", PRIVESCPOL_ERR_VALIDATION));
            }
            if *ceiling == PrivilegeLevel::SystemKernel {
                return Err(format!("{}: actor ceiling cannot be set to SystemKernel", PRIVESC_ERR_KERNEL_TIER_IMMUTABLE));
            }
        }

        if self.max_grant_duration_seconds == 0 || self.max_grant_duration_seconds > 86400 * 30 {
            return Err(format!("{}: max_grant_duration_seconds must be between 1 and 2592000", PRIVESCPOL_ERR_VALIDATION));
        }

        Ok(())
    }

    /// Evaluates a privilege transition request against this security policy.
    pub fn evaluate_transition(&self, req: &PrivilegeTransitionRequest) -> PrivilegePolicyVerdict {
        if self.mode == PrivilegePolicyMode::Disabled {
            return PrivilegePolicyVerdict::Permit;
        }

        // 1. Check disallowed elevation targets
        if self.disallowed_elevation_targets.contains(&req.target_level) {
            let reason = format!("{}: target level {:?} is explicitly disallowed by privilege policy", PRIVESC_ERR_KERNEL_TIER_IMMUTABLE, req.target_level);
            return match self.mode {
                PrivilegePolicyMode::Enforcing => PrivilegePolicyVerdict::Deny {
                    reason,
                    code: PRIVESCPOL_ERR_DENIED.into(),
                },
                PrivilegePolicyMode::Permissive => PrivilegePolicyVerdict::PermitWithWarning { warning: reason },
                PrivilegePolicyMode::Disabled => PrivilegePolicyVerdict::Permit,
            };
        }

        // 2. Check actor tier ceiling
        if let Some(&ceiling) = self.actor_tier_ceilings.get(req.actor_id.trim()) {
            if req.target_level > ceiling {
                let reason = format!("target level {:?} exceeds actor '{}' ceiling {:?}", req.target_level, req.actor_id, ceiling);
                return match self.mode {
                    PrivilegePolicyMode::Enforcing => PrivilegePolicyVerdict::Deny {
                        reason,
                        code: PRIVESCPOL_ERR_DENIED.into(),
                    },
                    PrivilegePolicyMode::Permissive => PrivilegePolicyVerdict::PermitWithWarning { warning: reason },
                    PrivilegePolicyMode::Disabled => PrivilegePolicyVerdict::Permit,
                };
            }
        }

        // 3. Check grant token mandate
        if self.require_grant_token && req.target_level > req.from_level {
            if req.grant_id.as_ref().map_or(true, |g| g.trim().is_empty()) {
                let reason = format!("{}: privilege elevation requires an explicit PEP authorization grant token", PRIVESC_ERR_UNAUTHORIZED_ELEVATION);
                return match self.mode {
                    PrivilegePolicyMode::Enforcing => PrivilegePolicyVerdict::Deny {
                        reason,
                        code: PRIVESCPOL_ERR_DENIED.into(),
                    },
                    PrivilegePolicyMode::Permissive => PrivilegePolicyVerdict::PermitWithWarning { warning: reason },
                    PrivilegePolicyMode::Disabled => PrivilegePolicyVerdict::Permit,
                };
            }
        }

        // 4. Check prohibited capabilities
        for cap in &req.requested_capabilities {
            if self.prohibited_capabilities.contains(cap) {
                let reason = format!("capability {:?} is prohibited by security policy", cap);
                return match self.mode {
                    PrivilegePolicyMode::Enforcing => PrivilegePolicyVerdict::Deny {
                        reason,
                        code: PRIVESCPOL_ERR_DENIED.into(),
                    },
                    PrivilegePolicyMode::Permissive => PrivilegePolicyVerdict::PermitWithWarning { warning: reason },
                    PrivilegePolicyMode::Disabled => PrivilegePolicyVerdict::Permit,
                };
            }
        }

        PrivilegePolicyVerdict::Permit
    }

    /// Loads a PrivilegeSecurityPolicy from a JSON file with size bounds and traversal protection.
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let p = path.as_ref();
        let path_str = p.to_string_lossy();
        if path_str.contains("..") {
            return Err(format!("{}: path traversal is prohibited", PRIVESCPOL_ERR_VALIDATION));
        }

        let metadata = fs::metadata(p).map_err(|e| format!("{}: cannot stat file: {}", PRIVESCPOL_ERR_IO, e))?;
        if metadata.len() > MAX_PRIVILEGE_SECURITY_POLICY_BYTES {
            return Err(format!("{}: policy file exceeds {} bytes", PRIVESCPOL_ERR_VALIDATION, MAX_PRIVILEGE_SECURITY_POLICY_BYTES));
        }

        let content = fs::read_to_string(p).map_err(|e| format!("{}: read failed: {}", PRIVESCPOL_ERR_IO, e))?;
        let policy: Self = serde_json::from_str(&content).map_err(|e| format!("{}: JSON parse failed: {}", PRIVESCPOL_ERR_PARSE, e))?;
        policy.validate()?;
        Ok(policy)
    }

    /// Loads default policy and applies environment overrides (e.g. AIOS_PRIVILEGE_POLICY_MODE).
    pub fn load_with_env_overrides() -> Self {
        let mut policy = Self::default();
        if let Ok(mode_str) = std::env::var("AIOS_PRIVILEGE_POLICY_MODE") {
            match mode_str.trim().to_lowercase().as_str() {
                "enforcing" => policy.mode = PrivilegePolicyMode::Enforcing,
                "permissive" => policy.mode = PrivilegePolicyMode::Permissive,
                "disabled" => policy.mode = PrivilegePolicyMode::Disabled,
                _ => {}
            }
        }
        policy
    }

    /// Saves the PrivilegeSecurityPolicy to a JSON file with path validation.
    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), String> {
        self.validate()?;
        let p = path.as_ref();
        let path_str = p.to_string_lossy();
        if path_str.contains("..") {
            return Err(format!("{}: path traversal is prohibited", PRIVESCPOL_ERR_VALIDATION));
        }

        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let data = serde_json::to_string_pretty(self).map_err(|e| format!("{}: serialize failed: {}", PRIVESCPOL_ERR_PARSE, e))?;
        fs::write(p, data).map_err(|e| format!("{}: write failed: {}", PRIVESCPOL_ERR_IO, e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_default_validation() {
        let policy = PrivilegeSecurityPolicy::default();
        assert_eq!(policy.mode, PrivilegePolicyMode::Enforcing);
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_policy_rejects_removal_of_kernel_target() {
        let mut policy = PrivilegeSecurityPolicy::default();
        policy.disallowed_elevation_targets.clear();
        let err = policy.validate().unwrap_err();
        assert!(err.contains(PRIVESC_ERR_KERNEL_TIER_IMMUTABLE));
    }

    #[test]
    fn test_policy_evaluation_enforcing_and_permissive() {
        let mut policy = PrivilegeSecurityPolicy::default();
        policy.actor_tier_ceilings.insert("intern".into(), PrivilegeLevel::User);

        let req = PrivilegeTransitionRequest {
            actor_id: "intern".into(),
            from_level: PrivilegeLevel::User,
            target_level: PrivilegeLevel::Operator,
            requested_capabilities: vec![],
            grant_id: Some("grant-123".into()),
        };

        // Enforcing -> Deny
        policy.mode = PrivilegePolicyMode::Enforcing;
        match policy.evaluate_transition(&req) {
            PrivilegePolicyVerdict::Deny { reason, code } => {
                assert_eq!(code, PRIVESCPOL_ERR_DENIED);
                assert!(reason.contains("ceiling"));
            }
            other => panic!("expected Deny, got {:?}", other),
        }

        // Permissive -> PermitWithWarning
        policy.mode = PrivilegePolicyMode::Permissive;
        match policy.evaluate_transition(&req) {
            PrivilegePolicyVerdict::PermitWithWarning { warning } => {
                assert!(warning.contains("ceiling"));
            }
            other => panic!("expected PermitWithWarning, got {:?}", other),
        }

        // Disabled -> Permit
        policy.mode = PrivilegePolicyMode::Disabled;
        assert_eq!(policy.evaluate_transition(&req), PrivilegePolicyVerdict::Permit);
    }
}
