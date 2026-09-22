//! Grant Lifecycle Data Model (PEPGRANT1..PEPGRANT6) for AIOS Security Kernel & PEP Fabric.
//!
//! Provides authoritative grant data models, finite state machine transitions,
//! temporal and quota constraint validation, and delegation attenuation.

use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::capability::{CapabilityRight, CapabilityScope, validate_identifier, validate_scope};

pub const MAX_GRANT_ID_LEN: usize = 128;
pub const MAX_GRANT_SUBJECT_LEN: usize = 128;
pub const MAX_GRANT_ISSUER_LEN: usize = 128;
pub const MAX_GRANT_REASON_LEN: usize = 512;
pub const MAX_METADATA_ENTRIES: usize = 64;
pub const MAX_METADATA_KEY_LEN: usize = 64;
pub const MAX_METADATA_VALUE_LEN: usize = 512;
pub const MAX_DELEGATION_DEPTH_LIMIT: u32 = 8;

pub const PEPGRANT_ERR_INVALID_TRANSITION: &str = "PEPGRANT_ERR_INVALID_TRANSITION";
pub const PEPGRANT_ERR_INVALID_ID: &str = "PEPGRANT_ERR_INVALID_ID";
pub const PEPGRANT_ERR_EXPIRED: &str = "PEPGRANT_ERR_EXPIRED";
pub const PEPGRANT_ERR_NOT_YET_VALID: &str = "PEPGRANT_ERR_NOT_YET_VALID";
pub const PEPGRANT_ERR_QUOTA_EXCEEDED: &str = "PEPGRANT_ERR_QUOTA_EXCEEDED";
pub const PEPGRANT_ERR_REVOKED: &str = "PEPGRANT_ERR_REVOKED";
pub const PEPGRANT_ERR_ATTENUATION: &str = "PEPGRANT_ERR_ATTENUATION";
pub const PEPGRANT_ERR_VALIDATION: &str = "PEPGRANT_ERR_VALIDATION";

/// Lifecycle states of an authorization grant (PEPGRANT1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepGrantState {
    Requested,
    Active,
    Suspended,
    Revoked,
    Expired,
}

impl std::fmt::Display for PepGrantState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Requested => write!(f, "requested"),
            Self::Active => write!(f, "active"),
            Self::Suspended => write!(f, "suspended"),
            Self::Revoked => write!(f, "revoked"),
            Self::Expired => write!(f, "expired"),
        }
    }
}

/// Context recorded upon grant revocation (PEPGRANT5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantRevocation {
    pub revoked_at: String,
    pub revoked_by: String,
    pub reason: String,
}

/// Temporal and volumetric quotas governing grant execution (PEPGRANT4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantConstraints {
    pub not_before: Option<String>,
    pub expires_at: Option<String>,
    pub max_invocations: Option<u64>,
    pub invocations_used: u64,
    pub max_bytes: Option<u64>,
    pub bytes_used: u64,
    pub max_delegation_depth: u32,
}

impl Default for PepGrantConstraints {
    fn default() -> Self {
        Self {
            not_before: None,
            expires_at: None,
            max_invocations: None,
            invocations_used: 0,
            max_bytes: None,
            bytes_used: 0,
            max_delegation_depth: 2,
        }
    }
}

/// Core managed grant entity in the PEP Fabric (PEPGRANT1..PEPGRANT6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrant {
    pub id: String,
    pub parent_grant_id: Option<String>,
    pub issuer: String,
    pub subject: String,
    pub scope: CapabilityScope,
    pub rights: Vec<CapabilityRight>,
    pub state: PepGrantState,
    pub constraints: PepGrantConstraints,
    pub revocation: Option<PepGrantRevocation>,
    pub metadata: HashMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
}

impl PepGrant {
    /// Constructs a new grant in Requested state with default constraints.
    pub fn new(
        id: impl Into<String>,
        issuer: impl Into<String>,
        subject: impl Into<String>,
        scope: CapabilityScope,
        rights: Vec<CapabilityRight>,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: id.into(),
            parent_grant_id: None,
            issuer: issuer.into(),
            subject: subject.into(),
            scope,
            rights,
            state: PepGrantState::Requested,
            constraints: PepGrantConstraints::default(),
            revocation: None,
            metadata: HashMap::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Validates grant fields against format, hygiene, and size constraints (PEPGRANT2).
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() || self.id.len() > MAX_GRANT_ID_LEN {
            return Err(format!("{}: id must be 1..{} chars", PEPGRANT_ERR_INVALID_ID, MAX_GRANT_ID_LEN));
        }
        if self.id.chars().any(|c| c.is_control() || c.is_whitespace()) {
            return Err(format!("{}: id cannot contain control or whitespace characters", PEPGRANT_ERR_INVALID_ID));
        }
        validate_identifier(&self.issuer, "issuer")
            .map_err(|e| format!("{}: {}", PEPGRANT_ERR_VALIDATION, e))?;
        validate_identifier(&self.subject, "subject")
            .map_err(|e| format!("{}: {}", PEPGRANT_ERR_VALIDATION, e))?;
        validate_scope(&self.scope)
            .map_err(|e| format!("{}: {}", PEPGRANT_ERR_VALIDATION, e))?;

        if self.rights.is_empty() {
            return Err(format!("{}: grant must confer at least one right", PEPGRANT_ERR_VALIDATION));
        }
        if self.constraints.max_delegation_depth > MAX_DELEGATION_DEPTH_LIMIT {
            return Err(format!("{}: max_delegation_depth {} exceeds limit of {}", PEPGRANT_ERR_VALIDATION, self.constraints.max_delegation_depth, MAX_DELEGATION_DEPTH_LIMIT));
        }
        if self.metadata.len() > MAX_METADATA_ENTRIES {
            return Err(format!("{}: metadata entries exceed limit of {}", PEPGRANT_ERR_VALIDATION, MAX_METADATA_ENTRIES));
        }
        for (k, v) in &self.metadata {
            if k.len() > MAX_METADATA_KEY_LEN || v.len() > MAX_METADATA_VALUE_LEN {
                return Err(format!("{}: metadata entry exceeds key limit ({}) or value limit ({})", PEPGRANT_ERR_VALIDATION, MAX_METADATA_KEY_LEN, MAX_METADATA_VALUE_LEN));
            }
        }
        Ok(())
    }

    /// Checks if a transition to the target state is allowed by the FSM (PEPGRANT1).
    pub fn can_transition_to(&self, next: PepGrantState) -> bool {
        match self.state {
            PepGrantState::Requested => matches!(next, PepGrantState::Active | PepGrantState::Revoked),
            PepGrantState::Active => matches!(next, PepGrantState::Suspended | PepGrantState::Revoked | PepGrantState::Expired),
            PepGrantState::Suspended => matches!(next, PepGrantState::Active | PepGrantState::Revoked | PepGrantState::Expired),
            PepGrantState::Revoked | PepGrantState::Expired => false, // Terminal
        }
    }

    /// Transitions to the target state if valid, updating the timestamp.
    pub fn transition_to(&mut self, next: PepGrantState) -> Result<(), String> {
        if !self.can_transition_to(next) {
            return Err(format!(
                "{}: cannot transition grant '{}' from {} to {}",
                PEPGRANT_ERR_INVALID_TRANSITION, self.id, self.state, next
            ));
        }
        self.state = next;
        self.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    /// Revokes the grant with an audit reason and operator identification (PEPGRANT5).
    pub fn revoke(&mut self, revoked_by: &str, reason: &str) -> Result<(), String> {
        if self.state == PepGrantState::Revoked {
            return Ok(()); // Idempotent
        }
        if self.state == PepGrantState::Expired {
            return Err(format!("{}: cannot revoke expired grant '{}'", PEPGRANT_ERR_INVALID_TRANSITION, self.id));
        }

        let now = Utc::now().to_rfc3339();
        self.revocation = Some(PepGrantRevocation {
            revoked_at: now.clone(),
            revoked_by: revoked_by.to_string(),
            reason: reason.to_string(),
        });
        self.state = PepGrantState::Revoked;
        self.updated_at = now;
        Ok(())
    }

    /// Evaluates temporal and quota constraints against the current context (PEPGRANT4).
    pub fn is_usable_at(&self, now_iso: &str) -> Result<(), String> {
        if self.state == PepGrantState::Revoked {
            return Err(format!("{}: grant '{}' is revoked", PEPGRANT_ERR_REVOKED, self.id));
        }
        if self.state == PepGrantState::Suspended {
            return Err(format!("{}: grant '{}' is suspended", PEPGRANT_ERR_VALIDATION, self.id));
        }
        if self.state == PepGrantState::Expired {
            return Err(format!("{}: grant '{}' is expired", PEPGRANT_ERR_EXPIRED, self.id));
        }
        if self.state != PepGrantState::Active {
            return Err(format!("{}: grant '{}' is in {} state (not active)", PEPGRANT_ERR_VALIDATION, self.id, self.state));
        }

        let current_time = chrono::DateTime::parse_from_rfc3339(now_iso)
            .map_err(|e| format!("{}: invalid current time format: {}", PEPGRANT_ERR_VALIDATION, e))?;

        if let Some(ref nb) = self.constraints.not_before {
            if let Ok(nb_time) = chrono::DateTime::parse_from_rfc3339(nb) {
                if current_time < nb_time {
                    return Err(format!("{}: grant not yet valid until {}", PEPGRANT_ERR_NOT_YET_VALID, nb));
                }
            }
        }

        if let Some(ref exp) = self.constraints.expires_at {
            if let Ok(exp_time) = chrono::DateTime::parse_from_rfc3339(exp) {
                if current_time >= exp_time {
                    return Err(format!("{}: grant expired at {}", PEPGRANT_ERR_EXPIRED, exp));
                }
            }
        }

        if let Some(max_inv) = self.constraints.max_invocations {
            if self.constraints.invocations_used >= max_inv {
                return Err(format!("{}: invocation quota {} exhausted", PEPGRANT_ERR_QUOTA_EXCEEDED, max_inv));
            }
        }

        if let Some(max_b) = self.constraints.max_bytes {
            if self.constraints.bytes_used >= max_b {
                return Err(format!("{}: byte quota {} exhausted", PEPGRANT_ERR_QUOTA_EXCEEDED, max_b));
            }
        }

        Ok(())
    }

    /// Records usage against the grant's quota (PEPGRANT4).
    pub fn record_invocation(&mut self, bytes: u64) -> Result<(), String> {
        self.constraints.invocations_used = self.constraints.invocations_used.saturating_add(1);
        self.constraints.bytes_used = self.constraints.bytes_used.saturating_add(bytes);
        self.updated_at = Utc::now().to_rfc3339();

        let mut expired = false;
        if let Some(max_inv) = self.constraints.max_invocations {
            if self.constraints.invocations_used >= max_inv {
                expired = true;
            }
        }
        if let Some(max_b) = self.constraints.max_bytes {
            if self.constraints.bytes_used >= max_b {
                expired = true;
            }
        }

        if expired && self.state == PepGrantState::Active {
            self.state = PepGrantState::Expired;
        }

        Ok(())
    }

    /// Derives an attenuated child grant from this parent grant (PEPGRANT3).
    pub fn attenuate(
        &self,
        new_id: &str,
        child_subject: &str,
        delegated_rights: Vec<CapabilityRight>,
    ) -> Result<PepGrant, String> {
        if self.state != PepGrantState::Active {
            return Err(format!("{}: cannot attenuate non-active parent grant", PEPGRANT_ERR_ATTENUATION));
        }
        if !self.rights.contains(&CapabilityRight::Delegate) {
            return Err(format!("{}: parent grant does not confer 'delegate' right", PEPGRANT_ERR_ATTENUATION));
        }
        if self.constraints.max_delegation_depth == 0 {
            return Err(format!("{}: parent grant delegation depth is 0 (delegation limit reached)", PEPGRANT_ERR_ATTENUATION));
        }

        // Assert all delegated rights are present in parent rights
        for r in &delegated_rights {
            if !self.rights.contains(r) {
                return Err(format!("{}: delegated right '{}' exceeds parent rights", PEPGRANT_ERR_ATTENUATION, r));
            }
        }

        let mut child = PepGrant::new(
            new_id,
            &self.subject,
            child_subject,
            self.scope.clone(),
            delegated_rights,
        );
        child.parent_grant_id = Some(self.id.clone());
        child.constraints.max_delegation_depth = self.constraints.max_delegation_depth.saturating_sub(1);
        child.constraints.expires_at = self.constraints.expires_at.clone();
        child.state = PepGrantState::Active;

        child.validate()?;
        Ok(child)
    }
}

pub const MAX_GRANTS_IN_STORE: usize = 5000;
pub const MAX_GRANT_STORE_SIZE: u64 = 10 * 1024 * 1024; // 10 MiB

/// Managed in-memory and persisted collection of grants with subject indexing and cascade revocation (PEPGRANT1..PEPGRANT6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PepGrantStore {
    pub grants: HashMap<String, PepGrant>,
    #[serde(skip)]
    pub storage_path: Option<std::path::PathBuf>,
}

impl PepGrantStore {
    /// Creates a new empty grant store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of grants in the store.
    pub fn len(&self) -> usize {
        self.grants.len()
    }

    /// Checks if store is empty.
    pub fn is_empty(&self) -> bool {
        self.grants.is_empty()
    }

    /// Registers or updates a grant in the store.
    pub fn add_grant(&mut self, grant: PepGrant) -> Result<(), String> {
        grant.validate()?;
        if self.grants.len() >= MAX_GRANTS_IN_STORE && !self.grants.contains_key(&grant.id) {
            return Err(format!("{}: store grant capacity limit {} reached", PEPGRANT_ERR_VALIDATION, MAX_GRANTS_IN_STORE));
        }
        self.grants.insert(grant.id.clone(), grant);
        Ok(())
    }

    /// Retrieves a reference to a grant by ID.
    pub fn get_grant(&self, id: &str) -> Option<&PepGrant> {
        self.grants.get(id)
    }

    /// Retrieves a mutable reference to a grant by ID.
    pub fn get_grant_mut(&mut self, id: &str) -> Option<&mut PepGrant> {
        self.grants.get_mut(id)
    }

    /// Returns a list of all grants.
    pub fn list_grants(&self) -> Vec<PepGrant> {
        self.grants.values().cloned().collect()
    }

    /// Returns references to all grants issued to a specific subject.
    pub fn list_grants_for_subject(&self, subject: &str) -> Vec<&PepGrant> {
        self.grants
            .values()
            .filter(|g| g.subject == subject)
            .collect()
    }

    /// Revokes a grant, optionally cascading revocation to all child grants (PEPGRANT5).
    pub fn revoke_grant(
        &mut self,
        id: &str,
        revoked_by: &str,
        reason: &str,
        cascade: bool,
    ) -> Result<usize, String> {
        if !self.grants.contains_key(id) {
            return Err(format!("{}: grant '{}' not found", PEPGRANT_ERR_VALIDATION, id));
        }

        let mut to_revoke = vec![id.to_string()];
        if cascade {
            let mut i = 0;
            while i < to_revoke.len() {
                let current_parent = to_revoke[i].clone();
                for g in self.grants.values() {
                    if g.parent_grant_id.as_deref() == Some(&current_parent) && !to_revoke.contains(&g.id) {
                        to_revoke.push(g.id.clone());
                    }
                }
                i += 1;
            }
        }

        let mut revoked_count = 0;
        for grant_id in to_revoke {
            if let Some(grant) = self.grants.get_mut(&grant_id) {
                if grant.state != PepGrantState::Revoked {
                    grant.revoke(revoked_by, reason)?;
                    revoked_count += 1;
                }
            }
        }

        Ok(revoked_count)
    }

    /// Evaluates if a grant is valid and authorized for the specified subject, right, and timestamp.
    pub fn validate_grant_for_action(
        &self,
        grant_id: &str,
        subject: &str,
        right: CapabilityRight,
        now_iso: &str,
    ) -> Result<(), String> {
        let grant = self.get_grant(grant_id)
            .ok_or_else(|| format!("{}: grant '{}' not found", PEPGRANT_ERR_VALIDATION, grant_id))?;

        if grant.subject != subject {
            return Err(format!(
                "{}: grant '{}' subject '{}' does not match requester '{}'",
                PEPGRANT_ERR_VALIDATION, grant_id, grant.subject, subject
            ));
        }

        grant.is_usable_at(now_iso)?;

        if !grant.rights.contains(&right) {
            return Err(format!(
                "{}: grant '{}' does not confer right '{}'",
                PEPGRANT_ERR_ATTENUATION, grant_id, right
            ));
        }

        Ok(())
    }

    /// Saves the grant store to a JSON file atomically.
    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{}: failed to create directory {:?}: {}", PEPGRANT_ERR_VALIDATION, parent, e))?;
        }

        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: serialization failed: {}", PEPGRANT_ERR_VALIDATION, e))?;

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp_path = path.with_extension(format!("tmp.{}.{}", std::process::id(), nonce));
        std::fs::write(&tmp_path, serialized)
            .map_err(|e| format!("{}: failed to write temporary file {:?}: {}", PEPGRANT_ERR_VALIDATION, tmp_path, e))?;

        if let Err(e) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("{}: atomic rename failed: {}", PEPGRANT_ERR_VALIDATION, e));
        }

        Ok(())
    }

    /// Loads the grant store from a JSON file.
    pub fn load_from_path(path: &std::path::Path) -> Result<Self, String> {
        if !path.exists() {
            return Err(format!("{}: file {:?} does not exist", PEPGRANT_ERR_VALIDATION, path));
        }

        let meta = std::fs::metadata(path)
            .map_err(|e| format!("{}: failed to read metadata: {}", PEPGRANT_ERR_VALIDATION, e))?;
        if meta.is_dir() {
            return Err(format!("{}: path {:?} is a directory, expected JSON file", PEPGRANT_ERR_VALIDATION, path));
        }
        if meta.len() > MAX_GRANT_STORE_SIZE {
            return Err(format!("{}: file size {} exceeds limit of {}", PEPGRANT_ERR_VALIDATION, meta.len(), MAX_GRANT_STORE_SIZE));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("{}: failed to read file: {}", PEPGRANT_ERR_VALIDATION, e))?;

        let mut store: Self = serde_json::from_str(&content)
            .map_err(|e| format!("{}: deserialization failed: {}", PEPGRANT_ERR_VALIDATION, e))?;

        store.storage_path = Some(path.to_path_buf());
        Ok(store)
    }
}

