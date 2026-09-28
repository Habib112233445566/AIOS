//! Security Policy Subsystem for Audit Chain Extensions (AUDITPOL1..AUDITPOL6).
//!
//! Provides declarative governance over event provenance integrity,
//! cryptographic signature requirements, causal fan-out limits,
//! and actor/tool allowlist/denylist rules.

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::audit::{AuditRowInput, ExtendedAuditRowInput};
use crate::audit_chain_ext::AuditCausalLink;

/// Maximum allowed length for policy version string.
pub const MAX_AUDIT_POLICY_VERSION_LEN: usize = 32;

/// Maximum allowed length for policy description.
pub const MAX_AUDIT_POLICY_DESC_LEN: usize = 512;

/// Maximum number of prohibited actors in policy.
pub const MAX_PROHIBITED_ACTORS: usize = 128;

/// Maximum number of prohibited tools in policy.
pub const MAX_PROHIBITED_TOOLS: usize = 128;

/// Maximum number of signature required tool prefixes in policy.
pub const MAX_SIGNATURE_REQUIRED_PREFIXES: usize = 64;

/// Upper bound limit for max allowed causal links per event.
pub const MAX_ALLOWED_CAUSAL_LINKS_UPPER_BOUND: usize = 64;

/// Maximum byte size of a persisted audit chain security policy file (64 KiB).
pub const MAX_AUDIT_SECURITY_POLICY_BYTES: u64 = 64 * 1024;

/// Maximum permissible future timestamp clock skew in seconds (5 minutes).
pub const MAX_PERMISSIBLE_TIMESTAMP_FUTURE_SKEW_SECS: u64 = 300;

/// Error code: Invalid policy parameter or bounds violation.
pub const AUDITPOL_ERR_VALIDATION: &str = "AUDITPOL_ERR_VALIDATION";

/// Error code: Prohibited actor or tool execution rejected.
pub const AUDITPOL_ERR_DENIED: &str = "AUDITPOL_ERR_DENIED";

/// Error code: Cryptographic signature required but missing or invalid.
pub const AUDITPOL_ERR_SIGNATURE_REQUIRED: &str = "AUDITPOL_ERR_SIGNATURE_REQUIRED";

/// Error code: Temporal validity constraint or clock skew violated.
pub const AUDITPOL_ERR_TEMPORAL: &str = "AUDITPOL_ERR_TEMPORAL";

/// Error code: Filesystem or I/O failure.
pub const AUDITPOL_ERR_IO: &str = "AUDITPOL_ERR_IO";

/// Error code: Deserialization or JSON parsing failure.
pub const AUDITPOL_ERR_PARSE: &str = "AUDITPOL_ERR_PARSE";

/// Enforcement mode governing Audit Chain operations (AUDITPOL1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditPolicyMode {
    /// Enforcing: Invariant violations strictly reject event ingestion (fail-closed).
    Enforcing,
    /// Permissive: Invariant violations emit warnings but allow ingestion (audit-only).
    Permissive,
    /// Disabled: Policy evaluation is bypassed.
    Disabled,
}

impl Default for AuditPolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Evaluation verdict for audit events and causal links.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum AuditPolicyVerdict {
    /// Action is permitted under security policy.
    Permit,
    /// Action is permitted with a policy audit warning.
    PermitWithWarning { reason: String },
    /// Action is denied fail-closed.
    Deny { reason: String, error_code: String },
}

impl AuditPolicyVerdict {
    pub fn is_permitted(&self) -> bool {
        matches!(self, Self::Permit | Self::PermitWithWarning { .. })
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, Self::Deny { .. })
    }
}

/// Security Policy governing Audit Chain Extensions (AUDITPOL1..AUDITPOL6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainSecurityPolicy {
    pub version: String,
    #[serde(default)]
    pub mode: AuditPolicyMode,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub disallow_anonymous: bool,
    #[serde(default)]
    pub prohibited_actors: Vec<String>,
    #[serde(default)]
    pub prohibited_tools: Vec<String>,
    #[serde(default)]
    pub signature_required_prefixes: Vec<String>,
    #[serde(default)]
    pub max_allowed_causal_links: usize,
    #[serde(default)]
    pub allow_future_timestamps_max_secs: u64,
    #[serde(default)]
    pub valid_from_epoch_secs: Option<u64>,
    #[serde(default)]
    pub valid_until_epoch_secs: Option<u64>,
}

impl Default for AuditChainSecurityPolicy {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            mode: AuditPolicyMode::Enforcing,
            description: "Default AIOS Audit Chain Security Policy (Enforcing)".to_string(),
            disallow_anonymous: true,
            prohibited_actors: vec![
                "anonymous".to_string(),
                "guest".to_string(),
                "untrusted".to_string(),
            ],
            prohibited_tools: vec![
                "raw_exec_bypass".to_string(),
                "disable_pep".to_string(),
                "drop_audit_chain".to_string(),
            ],
            signature_required_prefixes: vec![
                "kernel:".to_string(),
                "sec:".to_string(),
                "admin:".to_string(),
                "pep:".to_string(),
            ],
            max_allowed_causal_links: 32,
            allow_future_timestamps_max_secs: 300,
            valid_from_epoch_secs: None,
            valid_until_epoch_secs: None,
        }
    }
}

impl AuditChainSecurityPolicy {
    /// Validates policy parameters against invariant limits.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() || self.version.len() > MAX_AUDIT_POLICY_VERSION_LEN {
            return Err(format!(
                "{}: version must be between 1 and {} chars",
                AUDITPOL_ERR_VALIDATION, MAX_AUDIT_POLICY_VERSION_LEN
            ));
        }

        if !self.version.starts_with("1.") {
            return Err(format!(
                "{}: unsupported policy major version (expected 1.x)",
                AUDITPOL_ERR_VALIDATION
            ));
        }

        if self.description.len() > MAX_AUDIT_POLICY_DESC_LEN {
            return Err(format!(
                "{}: description exceeds {} chars",
                AUDITPOL_ERR_VALIDATION, MAX_AUDIT_POLICY_DESC_LEN
            ));
        }

        if self.prohibited_actors.len() > MAX_PROHIBITED_ACTORS {
            return Err(format!(
                "{}: prohibited_actors count exceeds {}",
                AUDITPOL_ERR_VALIDATION, MAX_PROHIBITED_ACTORS
            ));
        }

        if self.prohibited_tools.len() > MAX_PROHIBITED_TOOLS {
            return Err(format!(
                "{}: prohibited_tools count exceeds {}",
                AUDITPOL_ERR_VALIDATION, MAX_PROHIBITED_TOOLS
            ));
        }

        if self.signature_required_prefixes.len() > MAX_SIGNATURE_REQUIRED_PREFIXES {
            return Err(format!(
                "{}: signature_required_prefixes count exceeds {}",
                AUDITPOL_ERR_VALIDATION, MAX_SIGNATURE_REQUIRED_PREFIXES
            ));
        }

        if self.max_allowed_causal_links == 0
            || self.max_allowed_causal_links > MAX_ALLOWED_CAUSAL_LINKS_UPPER_BOUND
        {
            return Err(format!(
                "{}: max_allowed_causal_links must be in range 1..={}",
                AUDITPOL_ERR_VALIDATION, MAX_ALLOWED_CAUSAL_LINKS_UPPER_BOUND
            ));
        }

        if let (Some(from), Some(until)) = (self.valid_from_epoch_secs, self.valid_until_epoch_secs) {
            if from > until {
                return Err(format!(
                    "{}: valid_from ({}) cannot exceed valid_until ({})",
                    AUDITPOL_ERR_VALIDATION, from, until
                ));
            }
        }

        Ok(())
    }

    /// Evaluates an extended event against the security policy.
    pub fn evaluate_event(&self, event: &ExtendedAuditRowInput, current_epoch_secs: u64) -> AuditPolicyVerdict {
        if self.mode == AuditPolicyMode::Disabled {
            return AuditPolicyVerdict::Permit;
        }

        // Base evaluation
        let base_verdict = self.evaluate_base_event(&event.base, current_epoch_secs);
        if base_verdict.is_denied() {
            return base_verdict;
        }

        // Causal links evaluation
        let links_verdict = self.evaluate_causal_links(&event.causal_links);
        if links_verdict.is_denied() {
            return links_verdict;
        }

        // Signature requirement check
        for prefix in &self.signature_required_prefixes {
            if event.base.tool.starts_with(prefix) {
                let has_valid_sig = match &event.signature {
                    Some(sig) => !sig.signature.trim().is_empty() && !sig.public_key.trim().is_empty(),
                    None => false,
                };

                if !has_valid_sig {
                    let msg = format!(
                        "Cryptographic signature required for tool '{}' matching prefix '{}'",
                        event.base.tool, prefix
                    );
                    return self.wrap_violation(msg, AUDITPOL_ERR_SIGNATURE_REQUIRED);
                }
            }
        }

        if let AuditPolicyVerdict::PermitWithWarning { reason } = base_verdict {
            return AuditPolicyVerdict::PermitWithWarning { reason };
        }
        if let AuditPolicyVerdict::PermitWithWarning { reason } = links_verdict {
            return AuditPolicyVerdict::PermitWithWarning { reason };
        }

        AuditPolicyVerdict::Permit
    }

    /// Evaluates a base audit row against policy rules.
    pub fn evaluate_base_event(&self, base: &AuditRowInput, current_epoch_secs: u64) -> AuditPolicyVerdict {
        if self.mode == AuditPolicyMode::Disabled {
            return AuditPolicyVerdict::Permit;
        }

        // Temporal policy checks
        if let Some(valid_from) = self.valid_from_epoch_secs {
            if current_epoch_secs < valid_from {
                let msg = format!("Policy is not yet active (valid from {})", valid_from);
                return self.wrap_violation(msg, AUDITPOL_ERR_TEMPORAL);
            }
        }

        if let Some(valid_until) = self.valid_until_epoch_secs {
            if current_epoch_secs > valid_until {
                let msg = format!("Policy has expired (valid until {})", valid_until);
                return self.wrap_violation(msg, AUDITPOL_ERR_TEMPORAL);
            }
        }

        // Future clock skew check on timestamp if parseable
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&base.ts) {
            let ts_epoch = dt.timestamp().max(0) as u64;
            if ts_epoch > current_epoch_secs + self.allow_future_timestamps_max_secs {
                let msg = format!(
                    "Event timestamp {} exceeds permissible future skew limit (max {} secs)",
                    base.ts, self.allow_future_timestamps_max_secs
                );
                return self.wrap_violation(msg, AUDITPOL_ERR_TEMPORAL);
            }
        }

        // Anonymous provenance check
        if self.disallow_anonymous {
            if base.actor.trim().is_empty() || base.actor.to_lowercase() == "anonymous" {
                let msg = "Anonymous or empty actor provenance is prohibited".to_string();
                return self.wrap_violation(msg, AUDITPOL_ERR_DENIED);
            }
            if base.tool.trim().is_empty() || base.tool.to_lowercase() == "anonymous" {
                let msg = "Anonymous or empty tool identifier is prohibited".to_string();
                return self.wrap_violation(msg, AUDITPOL_ERR_DENIED);
            }
        }

        // Prohibited actor check
        for prohibited in &self.prohibited_actors {
            if base.actor.eq_ignore_ascii_case(prohibited) {
                let msg = format!("Actor '{}' is prohibited by security policy", base.actor);
                return self.wrap_violation(msg, AUDITPOL_ERR_DENIED);
            }
        }

        // Prohibited tool check
        for prohibited in &self.prohibited_tools {
            if base.tool.eq_ignore_ascii_case(prohibited) {
                let msg = format!("Tool '{}' is prohibited by security policy", base.tool);
                return self.wrap_violation(msg, AUDITPOL_ERR_DENIED);
            }
        }

        AuditPolicyVerdict::Permit
    }

    /// Evaluates causal links against policy fanout limits and formatting invariants.
    pub fn evaluate_causal_links(&self, links: &[AuditCausalLink]) -> AuditPolicyVerdict {
        if self.mode == AuditPolicyMode::Disabled {
            return AuditPolicyVerdict::Permit;
        }

        if links.len() > self.max_allowed_causal_links {
            let msg = format!(
                "Causal links count ({}) exceeds policy limit ({})",
                links.len(),
                self.max_allowed_causal_links
            );
            return self.wrap_violation(msg, AUDITPOL_ERR_VALIDATION);
        }

        for (idx, link) in links.iter().enumerate() {
            let h = link.parent_event_hash.trim();
            if h.len() != 64 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
                let msg = format!(
                    "Causal link index {} has invalid parent event hash (expected 64 hex chars, got '{}')",
                    idx, link.parent_event_hash
                );
                return self.wrap_violation(msg, AUDITPOL_ERR_VALIDATION);
            }
        }

        AuditPolicyVerdict::Permit
    }

    fn wrap_violation(&self, reason: String, error_code: &str) -> AuditPolicyVerdict {
        match self.mode {
            AuditPolicyMode::Enforcing => AuditPolicyVerdict::Deny {
                reason,
                error_code: error_code.to_string(),
            },
            AuditPolicyMode::Permissive => AuditPolicyVerdict::PermitWithWarning { reason },
            AuditPolicyMode::Disabled => AuditPolicyVerdict::Permit,
        }
    }

    /// Loads policy from JSON file with safe size bounds checking.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        if !p.exists() {
            return Err(format!("{}: file not found: {}", AUDITPOL_ERR_IO, p.display()));
        }

        let metadata = fs::metadata(p).map_err(|e| {
            format!("{}: failed to read metadata for {}: {}", AUDITPOL_ERR_IO, p.display(), e)
        })?;

        if metadata.len() > MAX_AUDIT_SECURITY_POLICY_BYTES {
            return Err(format!(
                "{}: policy file size {} exceeds maximum permitted limit {}",
                AUDITPOL_ERR_VALIDATION, metadata.len(), MAX_AUDIT_SECURITY_POLICY_BYTES
            ));
        }

        let content = fs::read_to_string(p).map_err(|e| {
            format!("{}: failed to read {}: {}", AUDITPOL_ERR_IO, p.display(), e)
        })?;

        let policy: Self = serde_json::from_str(&content).map_err(|e| {
            format!("{}: invalid JSON in {}: {}", AUDITPOL_ERR_PARSE, p.display(), e)
        })?;

        policy.validate()?;
        Ok(policy)
    }

    /// Saves policy atomically to JSON file.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.validate()?;
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!("{}: failed to create parent dir: {}", AUDITPOL_ERR_IO, e)
            })?;
        }

        let json = serde_json::to_string_pretty(self).map_err(|e| {
            format!("{}: failed to serialize policy: {}", AUDITPOL_ERR_PARSE, e)
        })?;

        let tmp_path = p.with_extension("tmp");
        fs::write(&tmp_path, json).map_err(|e| {
            format!("{}: failed to write temporary policy: {}", AUDITPOL_ERR_IO, e)
        })?;

        fs::rename(&tmp_path, p).map_err(|e| {
            format!("{}: failed to atomically rename policy: {}", AUDITPOL_ERR_IO, e)
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn now_epoch() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    #[test]
    fn test_policy_default_and_validation() {
        let policy = AuditChainSecurityPolicy::default();
        assert!(policy.validate().is_ok());
        assert_eq!(policy.mode, AuditPolicyMode::Enforcing);
    }

    #[test]
    fn test_policy_prohibited_actor() {
        let policy = AuditChainSecurityPolicy::default();
        let mut row = AuditRowInput::default();
        row.actor = "anonymous".to_string();
        row.tool = "test_tool".to_string();

        let epoch = now_epoch();
        let verdict = policy.evaluate_base_event(&row, epoch);
        assert!(verdict.is_denied());

        // In permissive mode, it should permit with warning
        let mut permissive = policy.clone();
        permissive.mode = AuditPolicyMode::Permissive;
        let p_verdict = permissive.evaluate_base_event(&row, epoch);
        assert!(p_verdict.is_permitted());
        assert!(matches!(p_verdict, AuditPolicyVerdict::PermitWithWarning { .. }));
    }

    #[test]
    fn test_policy_signature_required() {
        let policy = AuditChainSecurityPolicy::default();
        let mut row = AuditRowInput::default();
        row.actor = "valid_agent".to_string();
        row.tool = "kernel:reboot".to_string();

        let epoch = now_epoch();
        let input = ExtendedAuditRowInput::new(row.clone());
        let verdict = policy.evaluate_event(&input, epoch);
        assert!(verdict.is_denied());

        if let AuditPolicyVerdict::Deny { error_code, .. } = verdict {
            assert_eq!(error_code, AUDITPOL_ERR_SIGNATURE_REQUIRED);
        }

        // Valid signature provided
        let mut signed_input = ExtendedAuditRowInput::new(row);
        signed_input.signature = Some(crate::audit_chain_ext::AuditSignature::new(
            "ed25519",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ));
        let signed_verdict = policy.evaluate_event(&signed_input, epoch);
        assert!(signed_verdict.is_permitted());
    }

    #[test]
    fn test_policy_persistence_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("audit_policy.json");

        let policy = AuditChainSecurityPolicy::default();
        assert!(policy.save_to_file(&path).is_ok());

        let loaded = AuditChainSecurityPolicy::load_from_file(&path).expect("load policy");
        assert_eq!(policy, loaded);
    }
}
