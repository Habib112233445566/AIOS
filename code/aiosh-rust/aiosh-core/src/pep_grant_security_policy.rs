//! Grant Lifecycle Security Policy Subsystem (T-02261..T-02270).
//!
//! Provides policy-driven governance over capability grant issuance, delegation depth,
//! rights attenuation monotonicity, disallowed delegation rights, and credential lifetimes.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::capability::CapabilityRight;
use crate::pep_grant::PepGrant;

/// Maximum byte size of a persisted PEP grant security policy file (64 KiB).
pub const MAX_GRANT_POLICY_BYTES: u64 = 64 * 1024;

/// Default maximum grant duration in seconds (30 days).
pub const DEFAULT_MAX_GRANT_DURATION_SECS: u64 = 2_592_000;

/// Absolute maximum permissible grant duration in seconds (365 days).
pub const MAX_PERMISSIBLE_DURATION_SECS: u64 = 31_536_000;

/// Default maximum delegation depth under security policy.
pub const DEFAULT_MAX_POLICY_DELEGATION_DEPTH: u32 = 5;

/// Error code: Invalid policy parameter or bounds violation.
pub const GRANTPOL_ERR_VALIDATION: &str = "GRANTPOL_ERR_VALIDATION";

/// Error code: Policy invariant violation (e.g. lifetime exceeded or forbidden subject).
pub const GRANTPOL_ERR_POLICY_VIOLATION: &str = "GRANTPOL_ERR_POLICY_VIOLATION";

/// Error code: Right is prohibited from being delegated.
pub const GRANTPOL_ERR_DELEGATION_REJECTED: &str = "GRANTPOL_ERR_DELEGATION_REJECTED";

/// Error code: Grant validity period exceeds policy limits.
pub const GRANTPOL_ERR_LIFETIME_EXCEEDED: &str = "GRANTPOL_ERR_LIFETIME_EXCEEDED";

/// Error code: Filesystem or serialization I/O failure.
pub const GRANTPOL_ERR_IO: &str = "GRANTPOL_ERR_IO";

/// Operational enforcement mode governing grant policy decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepGrantEnforcementMode {
    /// Strict enforcement: Fail-closed on any policy violation.
    Enforcing,
    /// Audit-only: Violations are logged, but operations proceed.
    Permissive,
    /// Bypassed: Policy checks always return Ok.
    Disabled,
}

impl Default for PepGrantEnforcementMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Security Policy governing Grant Lifecycle operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantSecurityPolicy {
    /// Policy format version.
    pub version: String,
    /// Enforcement mode.
    #[serde(default)]
    pub mode: PepGrantEnforcementMode,
    /// Maximum allowable grant lifetime in seconds.
    pub max_grant_duration_seconds: u64,
    /// Maximum permissible delegation depth.
    pub max_delegation_depth: u32,
    /// Rights prohibited from being included in attenuated child grants.
    #[serde(default)]
    pub disallowed_delegation_rights: Vec<CapabilityRight>,
    /// Whether an explicit expiration timestamp is mandatory for all grants.
    pub require_explicit_expiry: bool,
    /// Substring or glob patterns for subjects forbidden from receiving grants.
    #[serde(default)]
    pub prohibited_subject_patterns: Vec<String>,
    /// Maximum capacity of grants allowed in the registry.
    pub max_store_capacity: usize,
}

impl Default for PepGrantSecurityPolicy {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            mode: PepGrantEnforcementMode::Enforcing,
            max_grant_duration_seconds: DEFAULT_MAX_GRANT_DURATION_SECS,
            max_delegation_depth: DEFAULT_MAX_POLICY_DELEGATION_DEPTH,
            disallowed_delegation_rights: vec![CapabilityRight::Admin],
            require_explicit_expiry: true,
            prohibited_subject_patterns: vec!["*anonymous*".into(), "*nobody*".into(), "*untrusted*".into()],
            max_store_capacity: 5_000,
        }
    }
}

impl PepGrantSecurityPolicy {
    /// Validates internal consistency of the policy struct.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() {
            return Err(format!("{}: version must not be empty", GRANTPOL_ERR_VALIDATION));
        }
        if self.max_grant_duration_seconds < 60 || self.max_grant_duration_seconds > MAX_PERMISSIBLE_DURATION_SECS {
            return Err(format!(
                "{}: max_grant_duration_seconds must be between 60 and {}",
                GRANTPOL_ERR_VALIDATION, MAX_PERMISSIBLE_DURATION_SECS
            ));
        }
        if self.max_delegation_depth < 1 || self.max_delegation_depth > 8 {
            return Err(format!(
                "{}: max_delegation_depth must be between 1 and 8",
                GRANTPOL_ERR_VALIDATION
            ));
        }
        if self.max_store_capacity < 1 || self.max_store_capacity > 50_000 {
            return Err(format!(
                "{}: max_store_capacity must be between 1 and 50,000",
                GRANTPOL_ERR_VALIDATION
            ));
        }
        Ok(())
    }

    /// Evaluates a grant against this security policy.
    pub fn validate_grant(&self, grant: &PepGrant) -> Result<(), String> {
        if self.mode == PepGrantEnforcementMode::Disabled {
            return Ok(());
        }

        // 1. Check prohibited subject patterns
        for pattern in &self.prohibited_subject_patterns {
            let needle = pattern.trim_matches('*');
            if !needle.is_empty() && grant.subject.contains(needle) {
                let err = format!(
                    "{}: Subject '{}' matches prohibited pattern '{}'",
                    GRANTPOL_ERR_POLICY_VIOLATION, grant.subject, pattern
                );
                if self.mode == PepGrantEnforcementMode::Enforcing {
                    return Err(err);
                }
            }
        }

        // 2. Check explicit expiry requirement
        if self.require_explicit_expiry && grant.constraints.expires_at.is_none() {
            let err = format!(
                "{}: Grant '{}' lacks mandatory explicit expiration timestamp",
                GRANTPOL_ERR_POLICY_VIOLATION, grant.id
            );
            if self.mode == PepGrantEnforcementMode::Enforcing {
                return Err(err);
            }
        }

        // 3. Check delegation depth bound
        if grant.constraints.max_delegation_depth > self.max_delegation_depth {
            let err = format!(
                "{}: Grant '{}' delegation depth {} exceeds policy ceiling {}",
                GRANTPOL_ERR_POLICY_VIOLATION,
                grant.id,
                grant.constraints.max_delegation_depth,
                self.max_delegation_depth
            );
            if self.mode == PepGrantEnforcementMode::Enforcing {
                return Err(err);
            }
        }

        // 4. Check lifetime duration if timestamps are present
        if let (Some(nbf_str), Some(exp_str)) = (&grant.constraints.not_before, &grant.constraints.expires_at) {
            if let (Ok(nbf), Ok(exp)) = (
                chrono::DateTime::parse_from_rfc3339(nbf_str),
                chrono::DateTime::parse_from_rfc3339(exp_str),
            ) {
                let diff_secs = (exp - nbf).num_seconds();
                if diff_secs > self.max_grant_duration_seconds as i64 {
                    let err = format!(
                        "{}: Grant duration {}s exceeds policy maximum {}s",
                        GRANTPOL_ERR_LIFETIME_EXCEEDED, diff_secs, self.max_grant_duration_seconds
                    );
                    if self.mode == PepGrantEnforcementMode::Enforcing {
                        return Err(err);
                    }
                }
            }
        }

        Ok(())
    }

    /// Evaluates an attenuation operation (parent -> child) against this security policy.
    pub fn validate_attenuation(&self, parent: &PepGrant, child: &PepGrant) -> Result<(), String> {
        if self.mode == PepGrantEnforcementMode::Disabled {
            return Ok(());
        }

        // Verify child does not introduce disallowed delegation rights
        for right in &child.rights {
            if self.disallowed_delegation_rights.contains(right) {
                let err = format!(
                    "{}: Right '{:?}' is disallowed from being delegated into child grant '{}'",
                    GRANTPOL_ERR_DELEGATION_REJECTED, right, child.id
                );
                if self.mode == PepGrantEnforcementMode::Enforcing {
                    return Err(err);
                }
            }
        }

        // Validate child grant against base policy rules
        self.validate_grant(child)?;

        // Ensure child does not exceed parent delegation depth
        if child.constraints.max_delegation_depth >= parent.constraints.max_delegation_depth {
            let err = format!(
                "{}: Child delegation depth must be strictly less than parent (parent: {}, child: {})",
                GRANTPOL_ERR_POLICY_VIOLATION,
                parent.constraints.max_delegation_depth,
                child.constraints.max_delegation_depth
            );
            if self.mode == PepGrantEnforcementMode::Enforcing {
                return Err(err);
            }
        }

        Ok(())
    }

    /// Loads and validates a policy from a JSON file.
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy();
        if path_str.contains("..") {
            return Err(format!("{}: Path traversal rejected", GRANTPOL_ERR_IO));
        }

        let metadata = fs::metadata(path_ref)
            .map_err(|e| format!("{}: Cannot stat policy file {}: {}", GRANTPOL_ERR_IO, path_str, e))?;
        if metadata.len() > MAX_GRANT_POLICY_BYTES {
            return Err(format!(
                "{}: Policy file size {} exceeds max allowed {} bytes",
                GRANTPOL_ERR_IO, metadata.len(), MAX_GRANT_POLICY_BYTES
            ));
        }

        let content = fs::read_to_string(path_ref)
            .map_err(|e| format!("{}: Cannot read policy file {}: {}", GRANTPOL_ERR_IO, path_str, e))?;
        let policy: Self = serde_json::from_str(&content)
            .map_err(|e| format!("{}: JSON parse failed: {}", GRANTPOL_ERR_VALIDATION, e))?;
        policy.validate()?;
        Ok(policy)
    }

    /// Persists policy to a JSON file atomically.
    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), String> {
        self.validate()?;
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy();
        if path_str.contains("..") {
            return Err(format!("{}: Path traversal rejected", GRANTPOL_ERR_IO));
        }

        if let Some(parent) = path_ref.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("{}: Failed creating directories: {}", GRANTPOL_ERR_IO, e))?;
        }

        let tmp_path = format!("{}.tmp.{}", path_str, std::process::id());
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: Serialization failed: {}", GRANTPOL_ERR_VALIDATION, e))?;

        let mut f = File::create(&tmp_path)
            .map_err(|e| format!("{}: Failed creating tmp file: {}", GRANTPOL_ERR_IO, e))?;
        f.write_all(json_str.as_bytes())
            .map_err(|e| format!("{}: Failed writing tmp file: {}", GRANTPOL_ERR_IO, e))?;
        f.flush()
            .map_err(|e| format!("{}: Failed flushing tmp file: {}", GRANTPOL_ERR_IO, e))?;
        drop(f);

        fs::rename(&tmp_path, path_ref)
            .map_err(|e| format!("{}: Atomic rename failed: {}", GRANTPOL_ERR_IO, e))?;

        Ok(())
    }
}
