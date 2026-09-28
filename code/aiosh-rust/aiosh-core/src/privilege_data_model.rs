//! Privilege Escalation Prevention Data Model (`PRIVESC1..PRIVESC6`).
//!
//! Provides the core domain types, privilege tiers, capability representations,
//! transition requests, and verdict evaluations for privilege escalation gating.

use std::collections::HashSet;
use serde::{Deserialize, Serialize};

/// Error code: Invalid or empty actor ID.
pub const PRIVESC_ERR_INVALID_ACTOR: &str = "PRIVESC_ERR_INVALID_ACTOR";

/// Error code: Transition to SystemKernel requested from userspace.
pub const PRIVESC_ERR_KERNEL_TIER_IMMUTABLE: &str = "PRIVESC_ERR_KERNEL_TIER_IMMUTABLE";

/// Error code: Capability count exceeds maximum allowable ceiling.
pub const PRIVESC_ERR_CAPABILITY_OVERFLOW: &str = "PRIVESC_ERR_CAPABILITY_OVERFLOW";

/// Error code: Capability requested exceeds active tier ceiling.
pub const PRIVESC_ERR_CAPABILITY_UNAUTHORIZED: &str = "PRIVESC_ERR_CAPABILITY_UNAUTHORIZED";

/// Error code: Unauthorized privilege elevation attempted without grant.
pub const PRIVESC_ERR_UNAUTHORIZED_ELEVATION: &str = "PRIVESC_ERR_UNAUTHORIZED_ELEVATION";

/// Error code: Invalid or malformed grant token provided.
pub const PRIVESC_ERR_INVALID_GRANT: &str = "PRIVESC_ERR_INVALID_GRANT";

/// Maximum length of an actor identifier (128 bytes).
pub const MAX_ACTOR_ID_LEN: usize = 128;

/// Maximum length of a grant identifier (256 bytes).
pub const MAX_GRANT_ID_LEN: usize = 256;

/// Maximum number of distinct capabilities held by a single context.
pub const MAX_CAPABILITIES_COUNT: usize = 32;

/// Discrete privilege tiers governing process execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeLevel {
    Guest = 0,
    User = 10,
    Operator = 20,
    Admin = 30,
    SystemKernel = 40,
}

impl PrivilegeLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrivilegeLevel::Guest => "guest",
            PrivilegeLevel::User => "user",
            PrivilegeLevel::Operator => "operator",
            PrivilegeLevel::Admin => "admin",
            PrivilegeLevel::SystemKernel => "system_kernel",
        }
    }

    pub fn parse_level(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "guest" => Some(PrivilegeLevel::Guest),
            "user" => Some(PrivilegeLevel::User),
            "operator" => Some(PrivilegeLevel::Operator),
            "admin" => Some(PrivilegeLevel::Admin),
            "system_kernel" | "system" | "kernel" => Some(PrivilegeLevel::SystemKernel),
            _ => None,
        }
    }
}

/// Typed capabilities assigned to privilege contexts or requested during execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeCapability {
    ProcessSpawn,
    NetworkConnect,
    NetworkListen,
    FilesystemWrite,
    MemoryInspect,
    AuditLogAdmin,
    SystemReboot,
    KernelModuleLoad,
}

impl PrivilegeCapability {
    /// Returns the minimum required privilege level for this capability.
    pub fn minimum_level(&self) -> PrivilegeLevel {
        match self {
            PrivilegeCapability::NetworkConnect => PrivilegeLevel::Guest,
            PrivilegeCapability::ProcessSpawn => PrivilegeLevel::User,
            PrivilegeCapability::FilesystemWrite => PrivilegeLevel::User,
            PrivilegeCapability::NetworkListen => PrivilegeLevel::Operator,
            PrivilegeCapability::MemoryInspect => PrivilegeLevel::Operator,
            PrivilegeCapability::AuditLogAdmin => PrivilegeLevel::Admin,
            PrivilegeCapability::SystemReboot => PrivilegeLevel::Admin,
            PrivilegeCapability::KernelModuleLoad => PrivilegeLevel::Admin,
        }
    }
}

/// Active execution context carrying privilege tier and assigned capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeContext {
    pub actor_id: String,
    pub active_level: PrivilegeLevel,
    pub capabilities: HashSet<PrivilegeCapability>,
    pub elevation_grant_id: Option<String>,
    pub is_elevation_active: bool,
    pub session_id: Option<String>,
}

impl PrivilegeContext {
    pub fn new(actor_id: impl Into<String>, level: PrivilegeLevel) -> Result<Self, String> {
        let actor = actor_id.into();
        if actor.chars().any(|c| c.is_control()) {
            return Err(format!("{}: actor_id cannot contain control characters", PRIVESC_ERR_INVALID_ACTOR));
        }
        let trimmed = actor.trim();
        if trimmed.is_empty() {
            return Err(format!("{}: actor_id cannot be empty", PRIVESC_ERR_INVALID_ACTOR));
        }
        if trimmed.len() > MAX_ACTOR_ID_LEN {
            return Err(format!("{}: actor_id exceeds maximum length {}", PRIVESC_ERR_INVALID_ACTOR, MAX_ACTOR_ID_LEN));
        }

        Ok(Self {
            actor_id: trimmed.to_string(),
            active_level: level,
            capabilities: HashSet::new(),
            elevation_grant_id: None,
            is_elevation_active: false,
            session_id: None,
        })
    }

    pub fn guest() -> Self {
        Self::new("guest", PrivilegeLevel::Guest).expect("guest context")
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.actor_id.trim().is_empty() {
            return Err(format!("{}: actor_id cannot be empty", PRIVESC_ERR_INVALID_ACTOR));
        }
        if self.capabilities.len() > MAX_CAPABILITIES_COUNT {
            return Err(format!("{}: capabilities count exceeds {}", PRIVESC_ERR_CAPABILITY_OVERFLOW, MAX_CAPABILITIES_COUNT));
        }
        Ok(())
    }

    pub fn add_capability(&mut self, cap: PrivilegeCapability) -> Result<(), String> {
        if self.capabilities.len() >= MAX_CAPABILITIES_COUNT && !self.capabilities.contains(&cap) {
            return Err(format!("{}: capabilities capacity reached", PRIVESC_ERR_CAPABILITY_OVERFLOW));
        }
        self.capabilities.insert(cap);
        Ok(())
    }

    pub fn has_capability(&self, cap: &PrivilegeCapability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Safely drops privilege to a lower or equal level without requiring a grant.
    /// Caps/filters capabilities to only those permissible at or below `new_level`.
    pub fn drop_to_level(&mut self, new_level: PrivilegeLevel) -> Result<(), String> {
        if new_level > self.active_level {
            return Err(format!("{}: cannot drop privilege to a higher level ({:?} > {:?})", 
                PRIVESC_ERR_UNAUTHORIZED_ELEVATION, new_level, self.active_level));
        }

        self.active_level = new_level;
        // Capabilities requiring a higher tier than new_level must be stripped
        self.capabilities.retain(|c| c.minimum_level() <= new_level);
        self.elevation_grant_id = None;
        self.is_elevation_active = false;
        Ok(())
    }

    /// Elevates privilege to target level when accompanied by a valid grant ID.
    pub fn elevate_with_grant(&mut self, target_level: PrivilegeLevel, grant_id: &str) -> Result<(), String> {
        if grant_id.chars().any(|c| c.is_control()) {
            return Err(format!("{}: grant ID cannot contain control characters", PRIVESC_ERR_INVALID_GRANT));
        }
        let trimmed_grant = grant_id.trim();
        if trimmed_grant.is_empty() {
            return Err(format!("{}: elevation requires non-empty grant ID", PRIVESC_ERR_UNAUTHORIZED_ELEVATION));
        }
        if trimmed_grant.len() > MAX_GRANT_ID_LEN {
            return Err(format!("{}: grant ID exceeds maximum length {}", PRIVESC_ERR_INVALID_GRANT, MAX_GRANT_ID_LEN));
        }
        if target_level == PrivilegeLevel::SystemKernel && self.active_level != PrivilegeLevel::SystemKernel {
            return Err(format!("{}: elevation to SystemKernel tier is immutable", PRIVESC_ERR_KERNEL_TIER_IMMUTABLE));
        }

        self.active_level = target_level;
        self.elevation_grant_id = Some(trimmed_grant.to_string());
        self.is_elevation_active = true;
        Ok(())
    }

    /// Revokes active elevation, reverting to base level and clearing grant ID.
    pub fn revoke_elevation(&mut self, base_level: PrivilegeLevel) {
        self.active_level = base_level;
        self.elevation_grant_id = None;
        self.is_elevation_active = false;
        self.capabilities.retain(|c| c.minimum_level() <= base_level);
    }
}

/// Request to transition privilege level or acquire elevated capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeTransitionRequest {
    pub actor_id: String,
    pub from_level: PrivilegeLevel,
    pub target_level: PrivilegeLevel,
    pub requested_capabilities: Vec<PrivilegeCapability>,
    pub grant_id: Option<String>,
}

/// Evaluation verdict for a privilege transition or escalation attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeEscalationVerdict {
    Allowed,
    GrantRequired { reason: String },
    Denied { reason: String },
}

impl PrivilegeTransitionRequest {
    /// Validates the request fields for bounds, control characters, and integrity.
    pub fn validate(&self) -> Result<(), String> {
        if self.actor_id.chars().any(|c| c.is_control()) {
            return Err(format!("{}: actor_id cannot contain control characters", PRIVESC_ERR_INVALID_ACTOR));
        }
        let trimmed_actor = self.actor_id.trim();
        if trimmed_actor.is_empty() {
            return Err(format!("{}: actor_id cannot be empty", PRIVESC_ERR_INVALID_ACTOR));
        }
        if trimmed_actor.len() > MAX_ACTOR_ID_LEN {
            return Err(format!("{}: actor_id exceeds maximum length {}", PRIVESC_ERR_INVALID_ACTOR, MAX_ACTOR_ID_LEN));
        }
        if self.requested_capabilities.len() > MAX_CAPABILITIES_COUNT {
            return Err(format!("{}: requested capabilities count exceeds {}", PRIVESC_ERR_CAPABILITY_OVERFLOW, MAX_CAPABILITIES_COUNT));
        }
        if let Some(ref gid) = self.grant_id {
            if gid.chars().any(|c| c.is_control()) {
                return Err(format!("{}: grant ID cannot contain control characters", PRIVESC_ERR_INVALID_GRANT));
            }
            if gid.trim().len() > MAX_GRANT_ID_LEN {
                return Err(format!("{}: grant ID exceeds maximum length {}", PRIVESC_ERR_INVALID_GRANT, MAX_GRANT_ID_LEN));
            }
        }
        Ok(())
    }

    /// Evaluates the escalation request according to PRIVESC invariants.
    pub fn evaluate(&self) -> PrivilegeEscalationVerdict {
        // Pre-flight check: validate request formatting and bounds
        if let Err(e) = self.validate() {
            return PrivilegeEscalationVerdict::Denied { reason: e };
        }

        // Invariant 2: Cannot transition to SystemKernel from userspace
        if self.target_level == PrivilegeLevel::SystemKernel && self.from_level != PrivilegeLevel::SystemKernel {
            return PrivilegeEscalationVerdict::Denied {
                reason: format!("{}: transition to SystemKernel from userspace is forbidden", PRIVESC_ERR_KERNEL_TIER_IMMUTABLE),
            };
        }

        // Invariant 3: High-risk capabilities check
        for cap in &self.requested_capabilities {
            let min_lvl = cap.minimum_level();
            if self.target_level < min_lvl && self.grant_id.is_none() {
                return PrivilegeEscalationVerdict::GrantRequired {
                    reason: format!("{}: capability {:?} requires minimum tier {:?} or explicit PEP grant", PRIVESC_ERR_CAPABILITY_UNAUTHORIZED, cap, min_lvl),
                };
            }
        }

        // Invariant 1: Monotonic escalation law
        if self.target_level > self.from_level {
            match self.grant_id {
                Some(ref gid) if !gid.trim().is_empty() => PrivilegeEscalationVerdict::Allowed,
                _ => PrivilegeEscalationVerdict::GrantRequired {
                    reason: format!("{}: elevation from {:?} to {:?} mandates explicit PEP authorization grant", PRIVESC_ERR_UNAUTHORIZED_ELEVATION, self.from_level, self.target_level),
                },
            }
        } else {
            PrivilegeEscalationVerdict::Allowed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privilege_level_ordering() {
        assert!(PrivilegeLevel::Guest < PrivilegeLevel::User);
        assert!(PrivilegeLevel::User < PrivilegeLevel::Operator);
        assert!(PrivilegeLevel::Operator < PrivilegeLevel::Admin);
        assert!(PrivilegeLevel::Admin < PrivilegeLevel::SystemKernel);
    }

    #[test]
    fn test_context_validation_and_dropping() {
        let mut ctx = PrivilegeContext::new("alice", PrivilegeLevel::Operator).unwrap();
        ctx.add_capability(PrivilegeCapability::NetworkListen).unwrap();
        ctx.add_capability(PrivilegeCapability::NetworkConnect).unwrap();
        assert_eq!(ctx.capabilities.len(), 2);

        // Dropping to User level must strip NetworkListen (which requires Operator)
        assert!(ctx.drop_to_level(PrivilegeLevel::User).is_ok());
        assert_eq!(ctx.active_level, PrivilegeLevel::User);
        assert!(!ctx.has_capability(&PrivilegeCapability::NetworkListen));
        assert!(ctx.has_capability(&PrivilegeCapability::NetworkConnect));

        // Dropping to higher level must fail
        assert!(ctx.drop_to_level(PrivilegeLevel::Admin).is_err());
    }

    #[test]
    fn test_elevation_with_grant_and_revocation() {
        let mut ctx = PrivilegeContext::new("bob", PrivilegeLevel::User).unwrap();
        assert!(!ctx.is_elevation_active);

        // Cannot elevate without grant
        assert!(ctx.elevate_with_grant(PrivilegeLevel::Admin, "   ").is_err());

        // Elevate with valid grant
        assert!(ctx.elevate_with_grant(PrivilegeLevel::Admin, "GRANT-PEP-9901").is_ok());
        assert_eq!(ctx.active_level, PrivilegeLevel::Admin);
        assert!(ctx.is_elevation_active);
        assert_eq!(ctx.elevation_grant_id.as_deref(), Some("GRANT-PEP-9901"));

        // Cannot elevate to kernel
        assert!(ctx.elevate_with_grant(PrivilegeLevel::SystemKernel, "GRANT-PEP-9902").is_err());

        // Revoke elevation
        ctx.revoke_elevation(PrivilegeLevel::User);
        assert_eq!(ctx.active_level, PrivilegeLevel::User);
        assert!(!ctx.is_elevation_active);
        assert_eq!(ctx.elevation_grant_id, None);
    }

    #[test]
    fn test_transition_request_evaluation() {
        // Monotonic escalation without grant requires grant
        let req = PrivilegeTransitionRequest {
            actor_id: "carol".to_string(),
            from_level: PrivilegeLevel::User,
            target_level: PrivilegeLevel::Admin,
            requested_capabilities: vec![],
            grant_id: None,
        };
        match req.evaluate() {
            PrivilegeEscalationVerdict::GrantRequired { .. } => (),
            other => panic!("expected GrantRequired, got {:?}", other),
        }

        // Monotonic escalation with grant is allowed
        let req_with_grant = PrivilegeTransitionRequest {
            actor_id: "carol".to_string(),
            from_level: PrivilegeLevel::User,
            target_level: PrivilegeLevel::Admin,
            requested_capabilities: vec![],
            grant_id: Some("GRANT-123".to_string()),
        };
        assert_eq!(req_with_grant.evaluate(), PrivilegeEscalationVerdict::Allowed);

        // SystemKernel target is unconditionally denied
        let req_kernel = PrivilegeTransitionRequest {
            actor_id: "root_wanna_be".to_string(),
            from_level: PrivilegeLevel::Admin,
            target_level: PrivilegeLevel::SystemKernel,
            requested_capabilities: vec![],
            grant_id: Some("ANY_GRANT".to_string()),
        };
        match req_kernel.evaluate() {
            PrivilegeEscalationVerdict::Denied { .. } => (),
            other => panic!("expected Denied, got {:?}", other),
        }
    }
}
