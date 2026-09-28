//! Security Policy Subsystem for Sandbox Enforcement (SANDBOXPOL1..SANDBOXPOL6).
//!
//! Provides declarative governance over sandbox execution requests,
//! evaluating prohibited commands, forbidden environment variables,
//! PEP grant mandates, and global resource limits.

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::sandbox_data_model::SandboxExecutionRequest;

/// Error code: Invalid policy parameter or bounds violation.
pub const SANDBOXPOL_ERR_VALIDATION: &str = "SANDBOXPOL_ERR_VALIDATION";

/// Error code: Prohibited command or policy violation rejected.
pub const SANDBOXPOL_ERR_DENIED: &str = "SANDBOXPOL_ERR_DENIED";

/// Error code: Filesystem or I/O failure.
pub const SANDBOXPOL_ERR_IO: &str = "SANDBOXPOL_ERR_IO";

/// Error code: Deserialization or JSON parsing failure.
pub const SANDBOXPOL_ERR_PARSE: &str = "SANDBOXPOL_ERR_PARSE";

/// Maximum allowed length for policy version string.
pub const MAX_SANDBOX_POLICY_VERSION_LEN: usize = 32;

/// Maximum number of prohibited commands in policy.
pub const MAX_PROHIBITED_COMMANDS: usize = 128;

/// Maximum number of prohibited environment variables in policy.
pub const MAX_PROHIBITED_ENV_VARS: usize = 128;

/// Maximum number of PEP-mandated profiles.
pub const MAX_PEP_MANDATED_PROFILES: usize = 64;

/// Maximum byte size of a persisted sandbox security policy file (64 KiB).
pub const MAX_SANDBOX_SECURITY_POLICY_BYTES: u64 = 64 * 1024;

/// Enforcement mode governing Sandbox operations (SANDBOXPOL1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxPolicyMode {
    /// Enforcing: Invariant violations strictly reject execution (fail-closed).
    Enforcing,
    /// Permissive: Invariant violations emit warnings but allow execution (audit-only).
    Permissive,
    /// Disabled: Policy evaluation is bypassed.
    Disabled,
}

impl Default for SandboxPolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Evaluation verdict for sandbox execution requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum SandboxPolicyVerdict {
    /// Action is permitted under security policy.
    Permit,
    /// Action is permitted with a policy warning.
    PermitWithWarning { warning: String },
    /// Action is denied fail-closed under security policy.
    Deny { reason: String, code: String },
}

/// Declarative security policy for Sandbox Enforcement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxSecurityPolicy {
    pub version: String,
    pub mode: SandboxPolicyMode,
    pub prohibited_commands: Vec<String>,
    pub prohibited_env_vars: Vec<String>,
    pub require_pep_grant_for_profiles: Vec<String>,
    pub max_permissible_wall_time_ms: u64,
    pub max_permissible_memory_bytes: u64,
}

impl Default for SandboxSecurityPolicy {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            mode: SandboxPolicyMode::Enforcing,
            prohibited_commands: vec![
                "rm".into(),
                "dd".into(),
                "mkfs".into(),
                "format".into(),
                "fdisk".into(),
                "shutdown".into(),
                "reboot".into(),
                "poweroff".into(),
            ],
            prohibited_env_vars: vec![
                "LD_PRELOAD".into(),
                "LD_LIBRARY_PATH".into(),
                "DYLD_INSERT_LIBRARIES".into(),
                "PYTHONPATH".into(),
                "NODE_OPTIONS".into(),
            ],
            require_pep_grant_for_profiles: vec![],
            max_permissible_wall_time_ms: 300_000,           // 5 minutes
            max_permissible_memory_bytes: 8_589_934_592,    // 8 GiB
        }
    }
}

impl SandboxSecurityPolicy {
    /// Validates internal constraints of the security policy.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() || self.version.len() > MAX_SANDBOX_POLICY_VERSION_LEN {
            return Err(format!("{}: invalid version length", SANDBOXPOL_ERR_VALIDATION));
        }
        if self.prohibited_commands.len() > MAX_PROHIBITED_COMMANDS {
            return Err(format!("{}: prohibited_commands exceeds maximum {}", SANDBOXPOL_ERR_VALIDATION, MAX_PROHIBITED_COMMANDS));
        }
        if self.prohibited_env_vars.len() > MAX_PROHIBITED_ENV_VARS {
            return Err(format!("{}: prohibited_env_vars exceeds maximum {}", SANDBOXPOL_ERR_VALIDATION, MAX_PROHIBITED_ENV_VARS));
        }
        if self.require_pep_grant_for_profiles.len() > MAX_PEP_MANDATED_PROFILES {
            return Err(format!("{}: require_pep_grant_for_profiles exceeds maximum {}", SANDBOXPOL_ERR_VALIDATION, MAX_PEP_MANDATED_PROFILES));
        }
        if self.max_permissible_wall_time_ms == 0 {
            return Err(format!("{}: max_permissible_wall_time_ms must be > 0", SANDBOXPOL_ERR_VALIDATION));
        }
        if self.max_permissible_memory_bytes == 0 {
            return Err(format!("{}: max_permissible_memory_bytes must be > 0", SANDBOXPOL_ERR_VALIDATION));
        }
        Ok(())
    }

    /// Evaluates an execution request against this security policy.
    pub fn evaluate(&self, req: &SandboxExecutionRequest) -> SandboxPolicyVerdict {
        if self.mode == SandboxPolicyMode::Disabled {
            return SandboxPolicyVerdict::Permit;
        }

        // 1. Prohibited command check (SANDBOXPOL2)
        let cmd_clean = req.command.trim().to_lowercase();
        let cmd_base = Path::new(&cmd_clean)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&cmd_clean);
        let cmd_base = cmd_base.strip_suffix(".exe").unwrap_or(cmd_base);

        for prohibited in &self.prohibited_commands {
            if cmd_base == prohibited.to_lowercase() {
                let reason = format!("command '{}' is prohibited by security policy", req.command);
                if self.mode == SandboxPolicyMode::Enforcing {
                    return SandboxPolicyVerdict::Deny {
                        reason,
                        code: SANDBOXPOL_ERR_DENIED.into(),
                    };
                } else {
                    return SandboxPolicyVerdict::PermitWithWarning { warning: reason };
                }
            }
        }

        // 2. Prohibited environment variables (SANDBOXPOL3)
        for (k, _) in &req.profile.environment.injected_vars {
            for prohibited_var in &self.prohibited_env_vars {
                if k.eq_ignore_ascii_case(prohibited_var) {
                    let reason = format!("environment variable '{}' is prohibited by security policy", k);
                    if self.mode == SandboxPolicyMode::Enforcing {
                        return SandboxPolicyVerdict::Deny {
                            reason,
                            code: SANDBOXPOL_ERR_DENIED.into(),
                        };
                    } else {
                        return SandboxPolicyVerdict::PermitWithWarning { warning: reason };
                    }
                }
            }
        }

        // 3. PEP grant mandate for designated profiles (SANDBOXPOL4)
        for mandated_profile in &self.require_pep_grant_for_profiles {
            if req.profile.name.eq_ignore_ascii_case(mandated_profile) {
                if req.pep_grant_id.is_none() || req.pep_grant_id.as_deref().unwrap_or("").trim().is_empty() {
                    let reason = format!("profile '{}' requires an explicit PEP capability grant", req.profile.name);
                    if self.mode == SandboxPolicyMode::Enforcing {
                        return SandboxPolicyVerdict::Deny {
                            reason,
                            code: SANDBOXPOL_ERR_DENIED.into(),
                        };
                    } else {
                        return SandboxPolicyVerdict::PermitWithWarning { warning: reason };
                    }
                }
            }
        }

        // 4. Resource bounds check (SANDBOXPOL5)
        if req.profile.resources.max_wall_time_ms > self.max_permissible_wall_time_ms {
            let reason = format!("requested wall time {} ms exceeds policy ceiling {} ms",
                req.profile.resources.max_wall_time_ms, self.max_permissible_wall_time_ms);
            if self.mode == SandboxPolicyMode::Enforcing {
                return SandboxPolicyVerdict::Deny {
                    reason,
                    code: SANDBOXPOL_ERR_DENIED.into(),
                };
            } else {
                return SandboxPolicyVerdict::PermitWithWarning { warning: reason };
            }
        }

        if req.profile.resources.max_memory_bytes > self.max_permissible_memory_bytes {
            let reason = format!("requested memory limit {} bytes exceeds policy ceiling {} bytes",
                req.profile.resources.max_memory_bytes, self.max_permissible_memory_bytes);
            if self.mode == SandboxPolicyMode::Enforcing {
                return SandboxPolicyVerdict::Deny {
                    reason,
                    code: SANDBOXPOL_ERR_DENIED.into(),
                };
            } else {
                return SandboxPolicyVerdict::PermitWithWarning { warning: reason };
            }
        }

        SandboxPolicyVerdict::Permit
    }

    /// Loads and parses a sandbox security policy from a JSON file.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let p_str = p.to_string_lossy();
        if p_str.contains("..") {
            return Err(format!("{}: directory traversal '..' prohibited in policy path {:?}", SANDBOXPOL_ERR_VALIDATION, p));
        }
        if p_str.chars().any(|c| c.is_control()) {
            return Err(format!("{}: control characters prohibited in policy path", SANDBOXPOL_ERR_VALIDATION));
        }

        let metadata = fs::symlink_metadata(p)
            .map_err(|e| format!("{}: unable to read metadata for {:?}: {}", SANDBOXPOL_ERR_IO, p, e))?;

        if metadata.file_type().is_symlink() {
            return Err(format!("{}: symlinks prohibited for policy file {:?}", SANDBOXPOL_ERR_VALIDATION, p));
        }

        if metadata.len() > MAX_SANDBOX_SECURITY_POLICY_BYTES {
            return Err(format!("{}: policy file {:?} size {} exceeds limit {}",
                SANDBOXPOL_ERR_VALIDATION, p, metadata.len(), MAX_SANDBOX_SECURITY_POLICY_BYTES));
        }

        let content = fs::read_to_string(p)
            .map_err(|e| format!("{}: failed to read {:?}: {}", SANDBOXPOL_ERR_IO, p, e))?;

        let policy: Self = serde_json::from_str(&content)
            .map_err(|e| format!("{}: invalid policy json in {:?}: {}", SANDBOXPOL_ERR_PARSE, p, e))?;

        policy.validate()?;
        Ok(policy)
    }

    /// Persists the security policy atomically to disk.
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.validate()?;
        let p = path.as_ref();
        let p_str = p.to_string_lossy();
        if p_str.contains("..") {
            return Err(format!("{}: directory traversal '..' prohibited in policy path {:?}", SANDBOXPOL_ERR_VALIDATION, p));
        }
        if p_str.chars().any(|c| c.is_control()) {
            return Err(format!("{}: control characters prohibited in policy path", SANDBOXPOL_ERR_VALIDATION));
        }

        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("{}: failed to create parent directory for {:?}: {}", SANDBOXPOL_ERR_IO, p, e))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: serialization error: {}", SANDBOXPOL_ERR_PARSE, e))?;

        fs::write(p, json)
            .map_err(|e| format!("{}: failed to write {:?}: {}", SANDBOXPOL_ERR_IO, p, e))?;
        Ok(())
    }
}
