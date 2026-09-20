//! Security Policy Subsystem for PEP Decision Engine (PEPPOL1..PEPPOL6).
//!
//! Provides governance over PEP enforcement modes (Enforcing, Permissive, Disabled),
//! administrative authoring boundaries, obligation criticality, and temporal validity.

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::pep_decision::{PepDecision, PepDecisionEffect, PepObligation, PepPolicyRule};

/// Maximum allowed length for policy version string.
pub const MAX_PEP_POLICY_VERSION_LEN: usize = 32;

/// Maximum allowed length for policy description.
pub const MAX_PEP_POLICY_DESC_LEN: usize = 512;

/// Maximum number of restricted resource prefixes.
pub const MAX_RESTRICTED_PREFIXES: usize = 64;

/// Maximum length for a single resource prefix.
pub const MAX_PREFIX_LEN: usize = 128;

/// Maximum byte size of a persisted PEP security policy file (64 KiB).
pub const MAX_PEP_SECURITY_POLICY_BYTES: u64 = 64 * 1024;

/// Error code: Invalid policy parameter or bounds violation.
pub const PEPPOL_ERR_VALIDATION: &str = "PEPPOL_ERR_VALIDATION";

/// Error code: Unprivileged caller attempting restricted operation.
pub const PEPPOL_ERR_PRIVILEGE: &str = "PEPPOL_ERR_PRIVILEGE";

/// Error code: Temporal validity constraint violated.
pub const PEPPOL_ERR_TEMPORAL: &str = "PEPPOL_ERR_TEMPORAL";

/// Error code: Filesystem or I/O failure.
pub const PEPPOL_ERR_IO: &str = "PEPPOL_ERR_IO";

/// Enforcement mode governing PEP Decision Engine operations (PEPPOL1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepEnforcementMode {
    /// Enforcing: Decisions strictly enforce permits and denies. Fail-closed.
    Enforcing,
    /// Permissive (Audit-Only): Decisions are evaluated, but denied requests are permitted with an audit notice.
    Permissive,
    /// Disabled: Policy evaluation is bypassed entirely.
    Disabled,
}

impl Default for PepEnforcementMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Criticality level for PEP obligations (PEPPOL3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepObligationCriticality {
    /// Strict: If the obligation cannot be fulfilled, the decision must fail-closed (Deny).
    Strict,
    /// BestEffort: Obligation failures are logged but do not revoke a permit decision.
    BestEffort,
}

impl Default for PepObligationCriticality {
    fn default() -> Self {
        Self::Strict
    }
}

/// Security Policy governing the PEP Decision Engine (PEPPOL1..PEPPOL6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepSecurityPolicy {
    pub version: String,
    #[serde(default)]
    pub mode: PepEnforcementMode,
    #[serde(default)]
    pub obligation_criticality: PepObligationCriticality,
    #[serde(default)]
    pub restricted_resource_prefixes: Vec<String>,
    #[serde(default)]
    pub valid_from_epoch_secs: Option<u64>,
    #[serde(default)]
    pub valid_until_epoch_secs: Option<u64>,
    #[serde(default)]
    pub description: String,
}

impl Default for PepSecurityPolicy {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            mode: PepEnforcementMode::Enforcing,
            obligation_criticality: PepObligationCriticality::Strict,
            restricted_resource_prefixes: vec![
                "sys:".to_string(),
                "sec:".to_string(),
                "kernel:".to_string(),
            ],
            valid_from_epoch_secs: None,
            valid_until_epoch_secs: None,
            description: "Default AIOS PEP Security Policy (Enforcing, Strict)".to_string(),
        }
    }
}

impl PepSecurityPolicy {
    /// Validates security policy parameters against invariant bounds.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() || self.version.len() > MAX_PEP_POLICY_VERSION_LEN {
            return Err(format!(
                "{}: version must be between 1 and {} chars",
                PEPPOL_ERR_VALIDATION, MAX_PEP_POLICY_VERSION_LEN
            ));
        }

        if self.description.len() > MAX_PEP_POLICY_DESC_LEN {
            return Err(format!(
                "{}: description length {} exceeds limit {}",
                PEPPOL_ERR_VALIDATION, self.description.len(), MAX_PEP_POLICY_DESC_LEN
            ));
        }

        if self.restricted_resource_prefixes.len() > MAX_RESTRICTED_PREFIXES {
            return Err(format!(
                "{}: restricted prefix count {} exceeds limit {}",
                PEPPOL_ERR_VALIDATION,
                self.restricted_resource_prefixes.len(),
                MAX_RESTRICTED_PREFIXES
            ));
        }

        for prefix in &self.restricted_resource_prefixes {
            if prefix.trim().is_empty() || prefix.len() > MAX_PREFIX_LEN {
                return Err(format!(
                    "{}: prefix {:?} exceeds max length {}",
                    PEPPOL_ERR_VALIDATION, prefix, MAX_PREFIX_LEN
                ));
            }
        }

        if let (Some(from), Some(until)) = (self.valid_from_epoch_secs, self.valid_until_epoch_secs) {
            if from > until {
                return Err(format!(
                    "{}: valid_from ({}) cannot be greater than valid_until ({})",
                    PEPPOL_ERR_VALIDATION, from, until
                ));
            }
        }

        Ok(())
    }

    /// Checks if the policy is active at the given UTC epoch timestamp (PEPPOL4).
    pub fn is_temporally_valid(&self, current_epoch_secs: u64) -> bool {
        if let Some(from) = self.valid_from_epoch_secs {
            if current_epoch_secs < from {
                return false;
            }
        }
        if let Some(until) = self.valid_until_epoch_secs {
            if current_epoch_secs > until {
                return false;
            }
        }
        true
    }

    /// Checks if a resource targets a restricted prefix.
    pub fn is_resource_restricted(&self, resource: &str) -> bool {
        self.restricted_resource_prefixes
            .iter()
            .any(|prefix| resource.starts_with(prefix))
    }

    /// Validates whether an incoming rule addition is permitted under this policy (PEPPOL2).
    pub fn validate_rule_addition(
        &self,
        rule: &PepPolicyRule,
        caller_is_privileged: bool,
    ) -> Result<(), String> {
        if !caller_is_privileged && rule.effect == PepDecisionEffect::Permit {
            if let Some(ref res) = rule.target_resource {
                if self.is_resource_restricted(res) {
                    return Err(format!(
                        "{}: unprivileged caller cannot add Permit rule for restricted resource '{}'",
                        PEPPOL_ERR_PRIVILEGE, res
                    ));
                }
            }
        }
        Ok(())
    }

    /// Enforces the security policy on an evaluated PEP decision (PEPPOL1, PEPPOL4).
    pub fn enforce_decision(
        &self,
        mut decision: PepDecision,
        current_epoch_secs: u64,
    ) -> PepDecision {
        // Check temporal validity
        if !self.is_temporally_valid(current_epoch_secs) {
            return PepDecision::default_deny(
                &decision.request_id,
                format!("{}: security policy is not active at timestamp {}", PEPPOL_ERR_TEMPORAL, current_epoch_secs),
            );
        }

        match self.mode {
            PepEnforcementMode::Enforcing => decision,
            PepEnforcementMode::Permissive => {
                if !decision.allowed {
                    // In permissive mode, permit the action but retain the original effect and log notice
                    decision.allowed = true;
                    decision.obligations.push(PepObligation::AuditLog {
                        level: "warning".to_string(),
                        message: format!(
                            "permissive bypass: decision was {:?} (reason: {})",
                            decision.effect, decision.reason
                        ),
                    });
                }
                decision
            }
            PepEnforcementMode::Disabled => {
                decision.allowed = true;
                decision.effect = PepDecisionEffect::Permit;
                decision.reason = "policy evaluation disabled by administrative mode".to_string();
                decision
            }
        }
    }

    /// Evaluates obligation fulfillment failure under configured criticality (PEPPOL3).
    pub fn handle_obligation_failure(
        &self,
        mut decision: PepDecision,
        failed_obligation: &PepObligation,
        failure_reason: &str,
    ) -> PepDecision {
        match self.obligation_criticality {
            PepObligationCriticality::Strict => {
                decision.allowed = false;
                decision.effect = PepDecisionEffect::Deny;
                decision.reason = format!(
                    "obligation fulfillment failed strictly: {:?} - {}",
                    failed_obligation, failure_reason
                );
                decision
            }
            PepObligationCriticality::BestEffort => {
                decision.obligations.push(PepObligation::AuditLog {
                    level: "error".to_string(),
                    message: format!(
                        "non-fatal obligation delivery failure: {:?} - {}",
                        failed_obligation, failure_reason
                    ),
                });
                decision
            }
        }
    }

    /// Configures the enforcement mode (builder).
    pub fn with_mode(mut self, mode: PepEnforcementMode) -> Self {
        self.mode = mode;
        self
    }

    /// Configures the obligation criticality (builder).
    pub fn with_criticality(mut self, criticality: PepObligationCriticality) -> Self {
        self.obligation_criticality = criticality;
        self
    }

    /// Configures the temporal validity window (builder).
    pub fn with_validity(mut self, from: Option<u64>, until: Option<u64>) -> Self {
        self.valid_from_epoch_secs = from;
        self.valid_until_epoch_secs = until;
        self
    }

    /// Adds a restricted resource prefix (builder).
    pub fn with_restricted_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.restricted_resource_prefixes.push(prefix.into());
        self
    }

    /// Persists the security policy to disk atomically (PEPPOL5).
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        validate_policy_path(path)?;

        if let Ok(meta) = fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() {
                return Err(format!("{}: target path {:?} is a symlink", PEPPOL_ERR_IO, path));
            }
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("{}: failed to create directory {:?}: {}", PEPPOL_ERR_IO, parent, e))?;
        }

        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: serialization failed: {}", PEPPOL_ERR_VALIDATION, e))?;

        let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));
        fs::write(&tmp_path, serialized)
            .map_err(|e| format!("{}: failed to write temporary file {:?}: {}", PEPPOL_ERR_IO, tmp_path, e))?;

        fs::rename(&tmp_path, path)
            .map_err(|e| format!("{}: atomic rename failed: {}", PEPPOL_ERR_IO, e))?;

        Ok(())
    }

    /// Loads a security policy from disk with size and symlink checks (PEPPOL5).
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        validate_policy_path(path)?;

        if !path.exists() {
            return Err(format!("{}: path {:?} does not exist", PEPPOL_ERR_IO, path));
        }

        if let Ok(meta) = fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() {
                return Err(format!("{}: target path {:?} is a symlink", PEPPOL_ERR_IO, path));
            }
            if meta.len() > MAX_PEP_SECURITY_POLICY_BYTES {
                return Err(format!(
                    "{}: file size {} exceeds limit of {}",
                    PEPPOL_ERR_VALIDATION, meta.len(), MAX_PEP_SECURITY_POLICY_BYTES
                ));
            }
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("{}: failed to read {:?}: {}", PEPPOL_ERR_IO, path, e))?;

        let policy: Self = serde_json::from_str(&content)
            .map_err(|e| format!("{}: deserialization failed: {}", PEPPOL_ERR_VALIDATION, e))?;

        policy.validate()?;
        Ok(policy)
    }
}

/// Validates a policy file path against traversal and extension constraints.
pub fn validate_policy_path(path: &Path) -> Result<(), String> {
    let s = path.to_string_lossy();
    if s.is_empty() || s.len() > 1024 {
        return Err(format!(
            "{}: path length {} invalid (must be 1..1024)",
            PEPPOL_ERR_VALIDATION, s.len()
        ));
    }

    if s.contains("..") {
        return Err(format!("{}: path contains directory traversal '..'", PEPPOL_ERR_VALIDATION));
    }

    if s.chars().any(|c| c.is_control()) {
        return Err(format!("{}: path contains invalid control characters", PEPPOL_ERR_VALIDATION));
    }

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => Ok(()),
        _ => Err(format!(
            "{}: policy file must have '.json' extension",
            PEPPOL_ERR_VALIDATION
        )),
    }
}
