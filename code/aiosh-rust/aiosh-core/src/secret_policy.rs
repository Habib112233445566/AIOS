//! Security Policy Subsystem for Secrets Handling (SECPOL1..SECPOL6).
//!
//! Provides declarative governance over secret storage, retrieval,
//! rotation, scope boundaries, and payload limits.

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::secret_data_model::{SecretEntry, SecretKind, SecretScope};

/// Error constants for Secrets Security Policy.
pub const SECPOL_ERR_VALIDATION: &str = "SECPOL_ERR_VALIDATION";
pub const SECPOL_ERR_DENIED: &str = "SECPOL_ERR_DENIED";
pub const SECPOL_ERR_GLOBAL_DISALLOWED: &str = "SECPOL_ERR_GLOBAL_DISALLOWED";
pub const SECPOL_ERR_KIND_PROHIBITED: &str = "SECPOL_ERR_KIND_PROHIBITED";
pub const SECPOL_ERR_PAYLOAD_TOO_LARGE: &str = "SECPOL_ERR_PAYLOAD_TOO_LARGE";
pub const SECPOL_ERR_EXPOSE_REQUIRED: &str = "SECPOL_ERR_EXPOSE_REQUIRED";
pub const SECPOL_ERR_IO: &str = "SECPOL_ERR_IO";
pub const SECPOL_ERR_PARSE: &str = "SECPOL_ERR_PARSE";

pub const MAX_SECRET_POLICY_VERSION_LEN: usize = 32;
pub const MAX_SECRET_SECURITY_POLICY_BYTES: u64 = 64 * 1024; // 64 KiB

/// Enforcement mode governing secrets security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretPolicyMode {
    /// Enforcing: Invariant violations fail-closed returning Deny.
    Enforcing,
    /// Permissive: Violations emit warning telemetry but permit access.
    Permissive,
    /// Disabled: Policy evaluation is bypassed.
    Disabled,
}

impl Default for SecretPolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Evaluation verdict for secret operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum SecretPolicyVerdict {
    /// Permitted under policy.
    Permit,
    /// Permitted with an auditable security warning.
    PermitWithWarning { warning: String },
    /// Denied fail-closed under security policy.
    Deny { reason: String, code: String },
}

/// Declarative security policy for Secrets Handling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretSecurityPolicy {
    pub version: String,
    pub mode: SecretPolicyMode,
    pub disallow_global_secrets: bool,
    pub max_payload_bytes: usize,
    pub prohibited_kinds: Vec<SecretKind>,
    pub require_expose_flag: bool,
    pub max_lifetime_seconds: u64,
}

impl Default for SecretSecurityPolicy {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            mode: SecretPolicyMode::Enforcing,
            disallow_global_secrets: false,
            max_payload_bytes: 65536,
            prohibited_kinds: vec![],
            require_expose_flag: true,
            max_lifetime_seconds: 2592000, // 30 days
        }
    }
}

impl SecretSecurityPolicy {
    /// Validates the policy structure and bounds.
    pub fn validate(&self) -> Result<(), String> {
        let v = self.version.trim();
        if v.is_empty() || v.len() > MAX_SECRET_POLICY_VERSION_LEN || !v.starts_with("1.") {
            return Err(format!("{}: version must be '1.x' and <= {} chars", SECPOL_ERR_VALIDATION, MAX_SECRET_POLICY_VERSION_LEN));
        }
        if self.max_payload_bytes == 0 || self.max_payload_bytes > 1048576 {
            return Err(format!("{}: max_payload_bytes must be between 1 and 1048576", SECPOL_ERR_VALIDATION));
        }
        if self.max_lifetime_seconds == 0 || self.max_lifetime_seconds > 86400 * 365 {
            return Err(format!("{}: max_lifetime_seconds must be between 1 and 31536000", SECPOL_ERR_VALIDATION));
        }
        Ok(())
    }

    /// Evaluates storing a secret against the policy.
    pub fn evaluate_store(&self, entry: &SecretEntry) -> SecretPolicyVerdict {
        if self.mode == SecretPolicyMode::Disabled {
            return SecretPolicyVerdict::Permit;
        }

        // 1. Check global secret policy
        if self.disallow_global_secrets && entry.metadata.scope == SecretScope::Global {
            let reason = "global secrets are prohibited by security policy".to_string();
            return match self.mode {
                SecretPolicyMode::Enforcing => SecretPolicyVerdict::Deny { reason, code: SECPOL_ERR_GLOBAL_DISALLOWED.into() },
                SecretPolicyMode::Permissive => SecretPolicyVerdict::PermitWithWarning { warning: reason },
                SecretPolicyMode::Disabled => SecretPolicyVerdict::Permit,
            };
        }

        // 2. Check prohibited kinds
        if self.prohibited_kinds.contains(&entry.metadata.kind) {
            let reason = format!("secret kind '{}' is prohibited by security policy", entry.metadata.kind.as_str());
            return match self.mode {
                SecretPolicyMode::Enforcing => SecretPolicyVerdict::Deny { reason, code: SECPOL_ERR_KIND_PROHIBITED.into() },
                SecretPolicyMode::Permissive => SecretPolicyVerdict::PermitWithWarning { warning: reason },
                SecretPolicyMode::Disabled => SecretPolicyVerdict::Permit,
            };
        }

        // 3. Check payload size
        let val_len = entry.value.as_bytes().len();
        if val_len > self.max_payload_bytes {
            let reason = format!("payload size {} exceeds policy limit {}", val_len, self.max_payload_bytes);
            return match self.mode {
                SecretPolicyMode::Enforcing => SecretPolicyVerdict::Deny { reason, code: SECPOL_ERR_PAYLOAD_TOO_LARGE.into() },
                SecretPolicyMode::Permissive => SecretPolicyVerdict::PermitWithWarning { warning: reason },
                SecretPolicyMode::Disabled => SecretPolicyVerdict::Permit,
            };
        }

        SecretPolicyVerdict::Permit
    }

    /// Evaluates secret retrieval against the policy.
    pub fn evaluate_get(&self, entry: &SecretEntry, caller_scope: &SecretScope, expose: bool) -> SecretPolicyVerdict {
        if self.mode == SecretPolicyMode::Disabled {
            return SecretPolicyVerdict::Permit;
        }

        if !caller_scope.allows(&entry.metadata.scope) {
            let reason = format!("caller scope '{}' denied access to secret scope '{}'", caller_scope.as_str(), entry.metadata.scope.as_str());
            return match self.mode {
                SecretPolicyMode::Enforcing => SecretPolicyVerdict::Deny { reason, code: SECPOL_ERR_DENIED.into() },
                SecretPolicyMode::Permissive => SecretPolicyVerdict::PermitWithWarning { warning: reason },
                SecretPolicyMode::Disabled => SecretPolicyVerdict::Permit,
            };
        }

        if self.require_expose_flag && !expose {
            let reason = "raw secret retrieval requires explicit expose flag under policy".to_string();
            return match self.mode {
                SecretPolicyMode::Enforcing => SecretPolicyVerdict::Deny { reason, code: SECPOL_ERR_EXPOSE_REQUIRED.into() },
                SecretPolicyMode::Permissive => SecretPolicyVerdict::PermitWithWarning { warning: reason },
                SecretPolicyMode::Disabled => SecretPolicyVerdict::Permit,
            };
        }

        SecretPolicyVerdict::Permit
    }

    /// Evaluates rotation against policy payload limits.
    pub fn evaluate_rotate(&self, _entry: &SecretEntry, new_payload_len: usize) -> SecretPolicyVerdict {
        if self.mode == SecretPolicyMode::Disabled {
            return SecretPolicyVerdict::Permit;
        }

        if new_payload_len > self.max_payload_bytes {
            let reason = format!("rotated payload size {} exceeds policy limit {}", new_payload_len, self.max_payload_bytes);
            return match self.mode {
                SecretPolicyMode::Enforcing => SecretPolicyVerdict::Deny { reason, code: SECPOL_ERR_PAYLOAD_TOO_LARGE.into() },
                SecretPolicyMode::Permissive => SecretPolicyVerdict::PermitWithWarning { warning: reason },
                SecretPolicyMode::Disabled => SecretPolicyVerdict::Permit,
            };
        }

        SecretPolicyVerdict::Permit
    }

    /// Loads a SecretSecurityPolicy from a JSON file.
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let p = path.as_ref();
        let path_str = p.to_string_lossy();
        if path_str.contains("..") {
            return Err(format!("{}: path traversal is prohibited", SECPOL_ERR_VALIDATION));
        }

        let metadata = fs::metadata(p).map_err(|e| format!("{}: cannot stat file: {}", SECPOL_ERR_IO, e))?;
        if metadata.len() > MAX_SECRET_SECURITY_POLICY_BYTES {
            return Err(format!("{}: policy file exceeds {} bytes", SECPOL_ERR_VALIDATION, MAX_SECRET_SECURITY_POLICY_BYTES));
        }

        let content = fs::read_to_string(p).map_err(|e| format!("{}: read failed: {}", SECPOL_ERR_IO, e))?;
        let policy: Self = serde_json::from_str(&content).map_err(|e| format!("{}: JSON parse failed: {}", SECPOL_ERR_PARSE, e))?;
        policy.validate()?;
        Ok(policy)
    }

    /// Loads policy with environment variable overrides.
    pub fn load_with_env_overrides() -> Self {
        let mut policy = Self::default();
        if let Ok(mode_str) = std::env::var("AIOS_SECRETS_POLICY_MODE") {
            match mode_str.trim().to_lowercase().as_str() {
                "enforcing" => policy.mode = SecretPolicyMode::Enforcing,
                "permissive" => policy.mode = SecretPolicyMode::Permissive,
                "disabled" => policy.mode = SecretPolicyMode::Disabled,
                _ => {}
            }
        }
        policy
    }

    /// Saves the SecretSecurityPolicy to a JSON file.
    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), String> {
        self.validate()?;
        let p = path.as_ref();
        let path_str = p.to_string_lossy();
        if path_str.contains("..") {
            return Err(format!("{}: path traversal is prohibited", SECPOL_ERR_VALIDATION));
        }

        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let data = serde_json::to_string_pretty(self).map_err(|e| format!("{}: serialize failed: {}", SECPOL_ERR_PARSE, e))?;
        fs::write(p, data).map_err(|e| format!("{}: write failed: {}", SECPOL_ERR_IO, e))?;
        Ok(())
    }
}
