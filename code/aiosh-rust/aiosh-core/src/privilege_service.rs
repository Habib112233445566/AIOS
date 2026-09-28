//! Privilege Escalation Prevention Core Service (`PRIVESC_SRV1..PRIVESC_SRV6`).
//!
//! Provides stateful management of process and session privilege contexts,
//! dynamic elevation gating, and capability verification.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::privilege_data_model::{
    PrivilegeCapability, PrivilegeContext, PrivilegeEscalationVerdict, PrivilegeLevel,
    PrivilegeTransitionRequest, PRIVESC_ERR_INVALID_ACTOR,
    PRIVESC_ERR_KERNEL_TIER_IMMUTABLE, PRIVESC_ERR_UNAUTHORIZED_ELEVATION,
};

/// Minimum allowed maximum active contexts ceiling.
pub const MIN_MAX_ACTIVE_CONTEXTS: usize = 1;

/// Hard upper ceiling for maximum active contexts.
pub const MAX_MAX_ACTIVE_CONTEXTS: usize = 16384;

/// Default maximum number of concurrent active contexts.
pub const DEFAULT_MAX_ACTIVE_CONTEXTS: usize = 1024;

/// Error code: Actor context not registered in service.
pub const PRIVESC_ERR_ACTOR_NOT_FOUND: &str = "PRIVESC_ERR_ACTOR_NOT_FOUND";

/// Error code: Service context registry capacity exceeded.
pub const PRIVESC_ERR_CAPACITY_EXCEEDED: &str = "PRIVESC_ERR_CAPACITY_EXCEEDED";

/// Error code: Context already exists for actor.
pub const PRIVESC_ERR_CONTEXT_EXISTS: &str = "PRIVESC_ERR_CONTEXT_EXISTS";

/// Core service managing active privilege contexts and elevation gating.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeService {
    contexts: HashMap<String, PrivilegeContext>,
    base_levels: HashMap<String, PrivilegeLevel>,
    max_contexts: usize,
}

impl Default for PrivilegeService {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivilegeService {
    /// Creates a new `PrivilegeService` with default capacity.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_MAX_ACTIVE_CONTEXTS)
    }

    /// Creates a new `PrivilegeService` with specified capacity ceiling.
    pub fn with_capacity(max_contexts: usize) -> Self {
        let bounded = max_contexts.clamp(MIN_MAX_ACTIVE_CONTEXTS, MAX_MAX_ACTIVE_CONTEXTS);
        Self {
            contexts: HashMap::new(),
            base_levels: HashMap::new(),
            max_contexts: bounded,
        }
    }

    /// Returns the number of currently active contexts.
    pub fn active_contexts_count(&self) -> usize {
        self.contexts.len()
    }

    /// Checks if an actor is registered.
    pub fn contains_actor(&self, actor_id: &str) -> bool {
        self.contexts.contains_key(actor_id.trim())
    }

    /// Retrieves a reference to an actor's active privilege context.
    pub fn get_context(&self, actor_id: &str) -> Option<&PrivilegeContext> {
        self.contexts.get(actor_id.trim())
    }

    /// Registers a new context. Fails if actor already registered, invalid, or capacity exceeded.
    pub fn register_context(&mut self, context: PrivilegeContext) -> Result<(), String> {
        context.validate()?;
        let actor = context.actor_id.trim().to_string();
        if self.contexts.contains_key(&actor) {
            return Err(format!("{}: context for actor '{}' already registered", PRIVESC_ERR_CONTEXT_EXISTS, actor));
        }
        if self.contexts.len() >= self.max_contexts {
            return Err(format!("{}: maximum active contexts ({}) reached", PRIVESC_ERR_CAPACITY_EXCEEDED, self.max_contexts));
        }

        self.base_levels.insert(actor.clone(), context.active_level);
        self.contexts.insert(actor, context);
        Ok(())
    }

    /// Unregisters an existing context, returning the removed context.
    pub fn unregister_context(&mut self, actor_id: &str) -> Result<PrivilegeContext, String> {
        if actor_id.chars().any(|c| c.is_control()) || actor_id.trim().is_empty() {
            return Err(format!("{}: invalid actor identifier", PRIVESC_ERR_INVALID_ACTOR));
        }
        let actor = actor_id.trim();
        self.base_levels.remove(actor);
        self.contexts.remove(actor).ok_or_else(|| {
            format!("{}: actor '{}' not found", PRIVESC_ERR_ACTOR_NOT_FOUND, actor)
        })
    }

    /// Requests privilege elevation for an actor with transition evaluation.
    pub fn request_elevation(&mut self, req: PrivilegeTransitionRequest) -> Result<PrivilegeContext, String> {
        let actor = req.actor_id.trim();
        let context = self.contexts.get_mut(actor).ok_or_else(|| {
            format!("{}: actor '{}' not found", PRIVESC_ERR_ACTOR_NOT_FOUND, actor)
        })?;

        // Kernel tier cannot be targeted from userspace
        if req.target_level == PrivilegeLevel::SystemKernel && context.active_level != PrivilegeLevel::SystemKernel {
            return Err(format!("{}: transition to SystemKernel is forbidden", PRIVESC_ERR_KERNEL_TIER_IMMUTABLE));
        }

        match req.evaluate() {
            PrivilegeEscalationVerdict::Allowed => {
                if let Some(ref gid) = req.grant_id {
                    context.elevate_with_grant(req.target_level, gid)?;
                } else if req.target_level <= context.active_level {
                    context.drop_to_level(req.target_level)?;
                } else {
                    return Err(format!("{}: elevation mandates grant", PRIVESC_ERR_UNAUTHORIZED_ELEVATION));
                }

                for cap in req.requested_capabilities {
                    context.add_capability(cap)?;
                }
                Ok(context.clone())
            }
            PrivilegeEscalationVerdict::GrantRequired { reason } => Err(reason),
            PrivilegeEscalationVerdict::Denied { reason } => Err(reason),
        }
    }

    /// Drops privilege level for an actor, stripping higher tier capabilities.
    pub fn drop_privilege(&mut self, actor_id: &str, target_level: PrivilegeLevel) -> Result<PrivilegeContext, String> {
        if actor_id.chars().any(|c| c.is_control()) || actor_id.trim().is_empty() {
            return Err(format!("{}: invalid actor identifier", PRIVESC_ERR_INVALID_ACTOR));
        }
        let actor = actor_id.trim();
        let context = self.contexts.get_mut(actor).ok_or_else(|| {
            format!("{}: actor '{}' not found", PRIVESC_ERR_ACTOR_NOT_FOUND, actor)
        })?;
        context.drop_to_level(target_level)?;
        Ok(context.clone())
    }

    /// Revokes active elevation, restoring baseline privilege level.
    pub fn revoke_elevation(&mut self, actor_id: &str) -> Result<PrivilegeContext, String> {
        if actor_id.chars().any(|c| c.is_control()) || actor_id.trim().is_empty() {
            return Err(format!("{}: invalid actor identifier", PRIVESC_ERR_INVALID_ACTOR));
        }
        let actor = actor_id.trim();
        let base_level = *self.base_levels.get(actor).ok_or_else(|| {
            format!("{}: base level not found for actor '{}'", PRIVESC_ERR_ACTOR_NOT_FOUND, actor)
        })?;
        let context = self.contexts.get_mut(actor).ok_or_else(|| {
            format!("{}: actor '{}' not found", PRIVESC_ERR_ACTOR_NOT_FOUND, actor)
        })?;
        context.revoke_elevation(base_level);
        Ok(context.clone())
    }

    /// Checks if an actor currently holds a specified capability in active context.
    pub fn check_capability(&self, actor_id: &str, capability: PrivilegeCapability) -> bool {
        self.contexts.get(actor_id.trim())
            .map(|ctx| ctx.has_capability(&capability))
            .unwrap_or(false)
    }

    /// Returns a sorted list of registered actor IDs.
    pub fn list_actors(&self) -> Vec<String> {
        let mut actors: Vec<String> = self.contexts.keys().cloned().collect();
        actors.sort();
        actors
    }

    /// Returns the baseline privilege level recorded during registration.
    pub fn get_base_level(&self, actor_id: &str) -> Option<PrivilegeLevel> {
        self.base_levels.get(actor_id.trim()).copied()
    }

    /// Clears all registered contexts.
    pub fn clear(&mut self) {
        self.contexts.clear();
        self.base_levels.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_registration_lifecycle() {
        let mut srv = PrivilegeService::with_capacity(3);
        assert_eq!(srv.active_contexts_count(), 0);

        let ctx1 = PrivilegeContext::new("user_1", PrivilegeLevel::User).unwrap();
        assert!(srv.register_context(ctx1).is_ok());
        assert_eq!(srv.active_contexts_count(), 1);
        assert!(srv.contains_actor("user_1"));

        // Duplicate registration fails
        let ctx1_dup = PrivilegeContext::new("user_1", PrivilegeLevel::User).unwrap();
        let dup_res = srv.register_context(ctx1_dup);
        assert!(dup_res.is_err());
        assert!(dup_res.unwrap_err().contains(PRIVESC_ERR_CONTEXT_EXISTS));

        // Register up to capacity
        let ctx2 = PrivilegeContext::new("user_2", PrivilegeLevel::User).unwrap();
        let ctx3 = PrivilegeContext::new("user_3", PrivilegeLevel::User).unwrap();
        assert!(srv.register_context(ctx2).is_ok());
        assert!(srv.register_context(ctx3).is_ok());

        // Overflow capacity fails
        let ctx4 = PrivilegeContext::new("user_4", PrivilegeLevel::User).unwrap();
        let cap_res = srv.register_context(ctx4);
        assert!(cap_res.is_err());
        assert!(cap_res.unwrap_err().contains(PRIVESC_ERR_CAPACITY_EXCEEDED));

        // Unregister succeeds
        let unreg = srv.unregister_context("user_2");
        assert!(unreg.is_ok());
        assert_eq!(srv.active_contexts_count(), 2);
    }

    #[test]
    fn test_elevation_and_revocation_service_flow() {
        let mut srv = PrivilegeService::new();
        let ctx = PrivilegeContext::new("operator_candidate", PrivilegeLevel::User).unwrap();
        srv.register_context(ctx).unwrap();

        // Elevation without grant fails
        let unauth_req = PrivilegeTransitionRequest {
            actor_id: "operator_candidate".to_string(),
            from_level: PrivilegeLevel::User,
            target_level: PrivilegeLevel::Operator,
            requested_capabilities: vec![PrivilegeCapability::NetworkListen],
            grant_id: None,
        };
        assert!(srv.request_elevation(unauth_req).is_err());

        // Elevation with grant succeeds
        let auth_req = PrivilegeTransitionRequest {
            actor_id: "operator_candidate".to_string(),
            from_level: PrivilegeLevel::User,
            target_level: PrivilegeLevel::Operator,
            requested_capabilities: vec![PrivilegeCapability::NetworkListen],
            grant_id: Some("GRANT-PEP-55".to_string()),
        };
        let elevated = srv.request_elevation(auth_req).unwrap();
        assert_eq!(elevated.active_level, PrivilegeLevel::Operator);
        assert!(elevated.is_elevation_active);
        assert!(srv.check_capability("operator_candidate", PrivilegeCapability::NetworkListen));

        // Target to SystemKernel is blocked
        let kernel_req = PrivilegeTransitionRequest {
            actor_id: "operator_candidate".to_string(),
            from_level: PrivilegeLevel::Operator,
            target_level: PrivilegeLevel::SystemKernel,
            requested_capabilities: vec![],
            grant_id: Some("ANY".to_string()),
        };
        assert!(srv.request_elevation(kernel_req).is_err());

        // Revoke elevation restores base level
        let restored = srv.revoke_elevation("operator_candidate").unwrap();
        assert_eq!(restored.active_level, PrivilegeLevel::User);
        assert!(!restored.is_elevation_active);
        assert!(!srv.check_capability("operator_candidate", PrivilegeCapability::NetworkListen));
    }
}
