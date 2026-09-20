//! Capability Model Data Model (CAP1..CAP6) for AIOS Security Kernel.
//!
//! Provides unforgeable, scoped, attenuable capabilities with monotonic rights
//! reduction, delegation tracking, temporal bounds, and invocation/byte quota constraints.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MAX_CAPABILITY_ID_LEN: usize = 64;
pub const MAX_SUBJECT_LEN: usize = 128;
pub const MAX_ISSUER_LEN: usize = 128;
pub const MAX_RESOURCE_URI_LEN: usize = 1024;
pub const MAX_ACTIONS_PER_SCOPE: usize = 64;

pub const CAP_ERROR_SCOPE: &str = "CAP_ERROR_SCOPE";
pub const CAP_ERROR_RIGHT: &str = "CAP_ERROR_RIGHT";
pub const CAP_ERROR_EXPIRED: &str = "CAP_ERROR_EXPIRED";
pub const CAP_ERROR_NOT_YET_VALID: &str = "CAP_ERROR_NOT_YET_VALID";
pub const CAP_ERROR_QUOTA_INVOCATIONS: &str = "CAP_ERROR_QUOTA_INVOCATIONS";
pub const CAP_ERROR_QUOTA_BYTES: &str = "CAP_ERROR_QUOTA_BYTES";
pub const CAP_ERROR_REVOKED: &str = "CAP_ERROR_REVOKED";
pub const CAP_ERROR_ATTENUATION: &str = "CAP_ERROR_ATTENUATION";
pub const CAP_ERROR_VALIDATION: &str = "CAP_ERROR_VALIDATION";

/// Specific operation rights granted by a capability (CAP2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityRight {
    Read,
    Write,
    Execute,
    Delete,
    Admin,
    Delegate,
}

impl std::fmt::Display for CapabilityRight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Execute => write!(f, "execute"),
            Self::Delete => write!(f, "delete"),
            Self::Admin => write!(f, "admin"),
            Self::Delegate => write!(f, "delegate"),
        }
    }
}

/// Target resource scoping for capability authorization (CAP2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "details", rename_all = "snake_case")]
pub enum CapabilityScope {
    Filesystem {
        path: String,
        recursive: bool,
    },
    Network {
        host: String,
        port: Option<u16>,
        protocol: String,
    },
    Tool {
        tool_name: String,
        allowed_actions: Vec<String>,
    },
    Process {
        executable: String,
        max_memory_bytes: Option<u64>,
    },
    Ipc {
        channel: String,
    },
    System {
        subsystem: String,
    },
}

/// Temporal and usage constraints bounding capability lifetime (CAP4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilityConstraints {
    pub not_before: Option<String>,
    pub expires_at: Option<String>,
    pub max_invocations: Option<u64>,
    pub current_invocations: u64,
    pub quota_bytes: Option<u64>,
    pub consumed_bytes: u64,
}

/// Core unforgeable authorization token in the Security Kernel (CAP1..CAP6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub parent_id: Option<String>,
    pub issuer: String,
    pub subject: String,
    pub scope: CapabilityScope,
    pub rights: Vec<CapabilityRight>,
    pub constraints: CapabilityConstraints,
    pub revoked: bool,
    pub created_at: String,
}

/// Errors originating from capability operations and enforcement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityError {
    ScopeMismatch(String),
    RightNotGranted(CapabilityRight),
    Expired(String),
    NotYetValid(String),
    InvocationQuotaExceeded,
    ByteQuotaExceeded,
    Revoked,
    InvalidAttenuation(String),
    ValidationError(String),
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScopeMismatch(msg) => write!(f, "{}: {}", CAP_ERROR_SCOPE, msg),
            Self::RightNotGranted(r) => write!(f, "{}: right '{}' not granted", CAP_ERROR_RIGHT, r),
            Self::Expired(exp) => write!(f, "{}: expired at {}", CAP_ERROR_EXPIRED, exp),
            Self::NotYetValid(nb) => write!(f, "{}: not valid before {}", CAP_ERROR_NOT_YET_VALID, nb),
            Self::InvocationQuotaExceeded => write!(f, "{}: invocation limit reached", CAP_ERROR_QUOTA_INVOCATIONS),
            Self::ByteQuotaExceeded => write!(f, "{}: byte quota exhausted", CAP_ERROR_QUOTA_BYTES),
            Self::Revoked => write!(f, "{}: capability has been revoked", CAP_ERROR_REVOKED),
            Self::InvalidAttenuation(msg) => write!(f, "{}: {}", CAP_ERROR_ATTENUATION, msg),
            Self::ValidationError(msg) => write!(f, "{}: {}", CAP_ERROR_VALIDATION, msg),
        }
    }
}

impl std::error::Error for CapabilityError {}

impl Capability {
    /// Creates a new root capability with validated inputs.
    pub fn new(
        issuer: &str,
        subject: &str,
        scope: CapabilityScope,
        rights: Vec<CapabilityRight>,
        constraints: CapabilityConstraints,
    ) -> Result<Self, CapabilityError> {
        if issuer.trim().is_empty() || issuer.len() > MAX_ISSUER_LEN {
            return Err(CapabilityError::ValidationError(format!("invalid issuer: '{}'", issuer)));
        }
        if subject.trim().is_empty() || subject.len() > MAX_SUBJECT_LEN {
            return Err(CapabilityError::ValidationError(format!("invalid subject: '{}'", subject)));
        }
        if rights.is_empty() {
            return Err(CapabilityError::ValidationError("rights list cannot be empty".to_string()));
        }

        let now = Utc::now();
        let mut hasher = Sha256::new();
        hasher.update(issuer.as_bytes());
        hasher.update(subject.as_bytes());
        hasher.update(now.to_rfc3339().as_bytes());
        let hash_hex = format!("{:x}", hasher.finalize());
        let id = format!("cap_{}_{}", now.timestamp_millis(), &hash_hex[..16]);

        Ok(Self {
            id,
            parent_id: None,
            issuer: issuer.to_string(),
            subject: subject.to_string(),
            scope,
            rights,
            constraints,
            revoked: false,
            created_at: now.to_rfc3339(),
        })
    }

    /// Checks if this capability grants a given right.
    pub fn has_right(&self, right: CapabilityRight) -> bool {
        self.rights.contains(&right)
    }

    /// Verifies that the right is granted by this capability.
    pub fn check_right(&self, right: CapabilityRight) -> Result<(), CapabilityError> {
        if self.has_right(right) {
            Ok(())
        } else {
            Err(CapabilityError::RightNotGranted(right))
        }
    }

    /// Checks if this capability is valid at a given time and within quotas (CAP4).
    pub fn check_validity_at(&self, now: chrono::DateTime<Utc>) -> Result<(), CapabilityError> {
        if self.revoked {
            return Err(CapabilityError::Revoked);
        }

        if let Some(ref nb_str) = self.constraints.not_before {
            if let Ok(nb) = chrono::DateTime::parse_from_rfc3339(nb_str) {
                if now < nb {
                    return Err(CapabilityError::NotYetValid(nb_str.clone()));
                }
            }
        }

        if let Some(ref exp_str) = self.constraints.expires_at {
            if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(exp_str) {
                if now > exp {
                    return Err(CapabilityError::Expired(exp_str.clone()));
                }
            }
        }

        if let Some(max_inv) = self.constraints.max_invocations {
            if self.constraints.current_invocations >= max_inv {
                return Err(CapabilityError::InvocationQuotaExceeded);
            }
        }

        if let Some(quota) = self.constraints.quota_bytes {
            if self.constraints.consumed_bytes >= quota {
                return Err(CapabilityError::ByteQuotaExceeded);
            }
        }

        Ok(())
    }

    /// Consumes an invocation against the capability quota (CAP4).
    pub fn consume_invocation(&mut self) -> Result<(), CapabilityError> {
        self.check_validity_at(Utc::now())?;
        if let Some(max_inv) = self.constraints.max_invocations {
            if self.constraints.current_invocations >= max_inv {
                return Err(CapabilityError::InvocationQuotaExceeded);
            }
        }
        self.constraints.current_invocations = self.constraints.current_invocations.saturating_add(1);
        Ok(())
    }

    /// Consumes bytes against the capability quota (CAP4).
    pub fn consume_bytes(&mut self, bytes: u64) -> Result<(), CapabilityError> {
        self.check_validity_at(Utc::now())?;
        if let Some(quota) = self.constraints.quota_bytes {
            if self.constraints.consumed_bytes.saturating_add(bytes) > quota {
                return Err(CapabilityError::ByteQuotaExceeded);
            }
        }
        self.constraints.consumed_bytes = self.constraints.consumed_bytes.saturating_add(bytes);
        Ok(())
    }

    /// Checks whether this capability's scope covers the requested scope (CAP2).
    pub fn matches_scope(&self, requested: &CapabilityScope) -> bool {
        match (&self.scope, requested) {
            (
                CapabilityScope::Filesystem { path: p_path, recursive },
                CapabilityScope::Filesystem { path: r_path, recursive: _ },
            ) => {
                if p_path == r_path {
                    true
                } else if *recursive {
                    let p_norm = if p_path.ends_with('/') || p_path.ends_with('\\') {
                        p_path.clone()
                    } else {
                        format!("{}/", p_path)
                    };
                    r_path.starts_with(&p_norm)
                } else {
                    false
                }
            }
            (
                CapabilityScope::Network { host: p_host, port: p_port, protocol: p_proto },
                CapabilityScope::Network { host: r_host, port: r_port, protocol: r_proto },
            ) => {
                let host_match = p_host == "*" || p_host.eq_ignore_ascii_case(r_host);
                let port_match = p_port.is_none() || *p_port == *r_port;
                let proto_match = p_proto == "*" || p_proto.eq_ignore_ascii_case(r_proto);
                host_match && port_match && proto_match
            }
            (
                CapabilityScope::Tool { tool_name: p_name, allowed_actions: p_actions },
                CapabilityScope::Tool { tool_name: r_name, allowed_actions: r_actions },
            ) => {
                if p_name != r_name && p_name != "*" {
                    return false;
                }
                if p_actions.is_empty() || p_actions.iter().any(|a| a == "*") {
                    true
                } else {
                    r_actions.iter().all(|ra| p_actions.contains(ra))
                }
            }
            (
                CapabilityScope::Process { executable: p_exe, max_memory_bytes: p_mem },
                CapabilityScope::Process { executable: r_exe, max_memory_bytes: r_mem },
            ) => {
                if p_exe != r_exe && p_exe != "*" {
                    return false;
                }
                match (p_mem, r_mem) {
                    (Some(p_limit), Some(r_limit)) => r_limit <= p_limit,
                    (Some(_), None) => false,
                    (None, _) => true,
                }
            }
            (CapabilityScope::Ipc { channel: p_chan }, CapabilityScope::Ipc { channel: r_chan }) => {
                p_chan == "*" || p_chan == r_chan
            }
            (CapabilityScope::System { subsystem: p_sub }, CapabilityScope::System { subsystem: r_sub }) => {
                p_sub == "*" || p_sub == r_sub
            }
            _ => false,
        }
    }

    /// Derives an attenuated child capability with equal or subset rights (CAP3).
    pub fn attenuate(
        &self,
        new_subject: &str,
        narrowed_scope: Option<CapabilityScope>,
        subset_rights: Vec<CapabilityRight>,
        narrowed_constraints: Option<CapabilityConstraints>,
    ) -> Result<Self, CapabilityError> {
        // Enforce parent delegation right
        self.check_right(CapabilityRight::Delegate)?;

        // Enforce parent validity
        self.check_validity_at(Utc::now())?;

        if new_subject.trim().is_empty() || new_subject.len() > MAX_SUBJECT_LEN {
            return Err(CapabilityError::ValidationError(format!("invalid subject: '{}'", new_subject)));
        }

        if subset_rights.is_empty() {
            return Err(CapabilityError::InvalidAttenuation("child rights cannot be empty".to_string()));
        }

        // Monotonic attenuation: all child rights must exist in parent rights
        for right in &subset_rights {
            if !self.has_right(*right) {
                return Err(CapabilityError::InvalidAttenuation(format!(
                    "right '{}' cannot be granted because parent lacks it",
                    right
                )));
            }
        }

        // Scope attenuation
        let child_scope = if let Some(sub_scope) = narrowed_scope {
            if !self.matches_scope(&sub_scope) {
                return Err(CapabilityError::InvalidAttenuation(
                    "child scope exceeds parent scope".to_string(),
                ));
            }
            sub_scope
        } else {
            self.scope.clone()
        };

        // Constraints attenuation
        let child_constraints = if let Some(mut sub_con) = narrowed_constraints {
            // Child cannot outlive parent
            if let Some(ref p_exp) = self.constraints.expires_at {
                if let Some(ref c_exp) = sub_con.expires_at {
                    if let (Ok(p_dt), Ok(c_dt)) = (
                        chrono::DateTime::parse_from_rfc3339(p_exp),
                        chrono::DateTime::parse_from_rfc3339(c_exp),
                    ) {
                        if c_dt > p_dt {
                            return Err(CapabilityError::InvalidAttenuation(
                                "child expiration cannot exceed parent expiration".to_string(),
                            ));
                        }
                    }
                } else {
                    sub_con.expires_at = Some(p_exp.clone());
                }
            }

            // Invocations cannot exceed parent remaining
            if let Some(p_max) = self.constraints.max_invocations {
                let p_remaining = p_max.saturating_sub(self.constraints.current_invocations);
                if let Some(c_max) = sub_con.max_invocations {
                    if c_max > p_remaining {
                        return Err(CapabilityError::InvalidAttenuation(
                            "child invocation quota exceeds parent remaining quota".to_string(),
                        ));
                    }
                } else {
                    sub_con.max_invocations = Some(p_remaining);
                }
            }

            // Quota bytes cannot exceed parent remaining
            if let Some(p_quota) = self.constraints.quota_bytes {
                let p_remaining = p_quota.saturating_sub(self.constraints.consumed_bytes);
                if let Some(c_quota) = sub_con.quota_bytes {
                    if c_quota > p_remaining {
                        return Err(CapabilityError::InvalidAttenuation(
                            "child byte quota exceeds parent remaining quota".to_string(),
                        ));
                    }
                } else {
                    sub_con.quota_bytes = Some(p_remaining);
                }
            }

            sub_con
        } else {
            self.constraints.clone()
        };

        let now = Utc::now();
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(new_subject.as_bytes());
        hasher.update(now.to_rfc3339().as_bytes());
        let hash_hex = format!("{:x}", hasher.finalize());
        let id = format!("cap_{}_{}", now.timestamp_millis(), &hash_hex[..16]);

        Ok(Self {
            id,
            parent_id: Some(self.id.clone()),
            issuer: self.subject.clone(),
            subject: new_subject.to_string(),
            scope: child_scope,
            rights: subset_rights,
            constraints: child_constraints,
            revoked: false,
            created_at: now.to_rfc3339(),
        })
    }

    /// Revokes the capability immediately (CAP5).
    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}
