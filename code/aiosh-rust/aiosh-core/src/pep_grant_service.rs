//! PEP Grant Core Service (GSVC1..GSVC6) for AIOS Security Kernel & PEP Fabric.
//!
//! Provides authoritative multi-indexed grant lifecycle coordination, active grant
//! caching, delegation attenuation, temporal/quota metering, cascade revocation,
//! and atomic disk persistence.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::capability::CapabilityRight;
use crate::pep_grant::{
    PepGrant, PepGrantState,
    PEPGRANT_ERR_EXPIRED, PEPGRANT_ERR_NOT_YET_VALID,
    PEPGRANT_ERR_QUOTA_EXCEEDED, PEPGRANT_ERR_REVOKED,
};

pub const MAX_GRANTS_IN_SERVICE: usize = 5000;
pub const MAX_GRANT_SERVICE_STORE_SIZE: u64 = 10 * 1024 * 1024; // 10 MiB

pub const GSVC_ERR_CAPACITY: &str = "GSVC_ERR_CAPACITY";
pub const GSVC_ERR_NOT_FOUND: &str = "GSVC_ERR_NOT_FOUND";
pub const GSVC_ERR_INVALID_TRANSITION: &str = "GSVC_ERR_INVALID_TRANSITION";
pub const GSVC_ERR_ATTENUATION: &str = "GSVC_ERR_ATTENUATION";
pub const GSVC_ERR_QUOTA_EXCEEDED: &str = "GSVC_ERR_QUOTA_EXCEEDED";
pub const GSVC_ERR_EXPIRED: &str = "GSVC_ERR_EXPIRED";
pub const GSVC_ERR_NOT_YET_VALID: &str = "GSVC_ERR_NOT_YET_VALID";
pub const GSVC_ERR_VALIDATION: &str = "GSVC_ERR_VALIDATION";
pub const GSVC_ERR_IO: &str = "GSVC_ERR_IO";

/// Validates that a grant service storage path is clean, canonical, and has a .json extension.
pub fn validate_grant_service_path(path: &Path) -> Result<(), String> {
    let path_str = path.to_string_lossy();
    if path_str.len() > 1024 {
        return Err(format!("{}: path length exceeds 1024 characters", GSVC_ERR_VALIDATION));
    }
    if path_str.chars().any(|c| c.is_control()) {
        return Err(format!("{}: path contains control characters", GSVC_ERR_VALIDATION));
    }
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(format!("{}: path traversal ('..') is not allowed", GSVC_ERR_VALIDATION));
        }
    }
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => Ok(()),
        _ => Err(format!("{}: file must have a .json extension", GSVC_ERR_VALIDATION)),
    }
}

/// Authoritative multi-indexed coordinator for grant lifecycle and execution (GSVC1..GSVC6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PepGrantService {
    grants: HashMap<String, PepGrant>,
    #[serde(skip)]
    by_subject: HashMap<String, HashSet<String>>,
    #[serde(skip)]
    by_parent: HashMap<String, HashSet<String>>,
    #[serde(skip)]
    by_state: HashMap<PepGrantState, HashSet<String>>,
    #[serde(skip)]
    storage_path: Option<PathBuf>,
}

impl PepGrantService {
    /// Creates a new empty grant service with initialized indexes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the storage path for persistence.
    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(path);
        self
    }

    /// Returns the backing storage path if configured.
    pub fn storage_path(&self) -> Option<&Path> {
        self.storage_path.as_deref()
    }

    /// Total number of grants in the service.
    pub fn len(&self) -> usize {
        self.grants.len()
    }

    /// Checks if service holds zero grants.
    pub fn is_empty(&self) -> bool {
        self.grants.is_empty()
    }

    /// Internal indexer for a grant into secondary multi-maps (GSVC1).
    fn index_grant(&mut self, grant: &PepGrant) {
        self.by_subject
            .entry(grant.subject.clone())
            .or_default()
            .insert(grant.id.clone());

        if let Some(ref p) = grant.parent_grant_id {
            self.by_parent
                .entry(p.clone())
                .or_default()
                .insert(grant.id.clone());
        }

        self.by_state
            .entry(grant.state)
            .or_default()
            .insert(grant.id.clone());
    }

    /// Internal un-indexer for a grant from secondary multi-maps (GSVC1).
    fn unindex_grant(&mut self, grant: &PepGrant) {
        if let Some(set) = self.by_subject.get_mut(&grant.subject) {
            set.remove(&grant.id);
        }
        if let Some(ref p) = grant.parent_grant_id {
            if let Some(set) = self.by_parent.get_mut(p) {
                set.remove(&grant.id);
            }
        }
        if let Some(set) = self.by_state.get_mut(&grant.state) {
            set.remove(&grant.id);
        }
    }

    /// Rebuilds all secondary indices from scratch (GSVC1).
    pub fn rebuild_indexes(&mut self) {
        self.by_subject.clear();
        self.by_parent.clear();
        self.by_state.clear();

        for grant in self.grants.values() {
            self.by_subject
                .entry(grant.subject.clone())
                .or_default()
                .insert(grant.id.clone());

            if let Some(ref p) = grant.parent_grant_id {
                self.by_parent
                    .entry(p.clone())
                    .or_default()
                    .insert(grant.id.clone());
            }

            self.by_state
                .entry(grant.state)
                .or_default()
                .insert(grant.id.clone());
        }
    }

    /// Registers a new grant in the service and updates secondary indices (GSVC1, GSVC2).
    pub fn issue_grant(&mut self, grant: PepGrant) -> Result<(), String> {
        grant.validate().map_err(|e| format!("{}: {}", GSVC_ERR_VALIDATION, e))?;

        if self.grants.len() >= MAX_GRANTS_IN_SERVICE && !self.grants.contains_key(&grant.id) {
            return Err(format!(
                "{}: service capacity limit {} reached",
                GSVC_ERR_CAPACITY, MAX_GRANTS_IN_SERVICE
            ));
        }

        if let Some(old) = self.grants.remove(&grant.id) {
            self.unindex_grant(&old);
        }

        self.index_grant(&grant);
        self.grants.insert(grant.id.clone(), grant);
        Ok(())
    }

    /// Retrieves a reference to a grant by its identifier.
    pub fn get_grant(&self, id: &str) -> Option<&PepGrant> {
        self.grants.get(id)
    }

    /// Retrieves a mutable reference to a grant by its identifier.
    pub fn get_grant_mut(&mut self, id: &str) -> Option<&mut PepGrant> {
        self.grants.get_mut(id)
    }

    /// Lists all grants registered in the service.
    pub fn list_grants(&self) -> Vec<PepGrant> {
        self.grants.values().cloned().collect()
    }

    /// Retrieves references to all grants issued to a specific subject (GSVC1).
    pub fn list_grants_for_subject(&self, subject: &str) -> Vec<&PepGrant> {
        match self.by_subject.get(subject) {
            Some(ids) => ids.iter().filter_map(|id| self.grants.get(id)).collect(),
            None => Vec::new(),
        }
    }

    /// Retrieves references to all grants in a specific lifecycle state (GSVC1).
    pub fn list_grants_by_state(&self, state: PepGrantState) -> Vec<&PepGrant> {
        match self.by_state.get(&state) {
            Some(ids) => ids.iter().filter_map(|id| self.grants.get(id)).collect(),
            None => Vec::new(),
        }
    }

    /// Transitions a grant to a new lifecycle state, maintaining index synchronization (GSVC1, GSVC2).
    pub fn transition_grant(&mut self, id: &str, target_state: PepGrantState) -> Result<(), String> {
        let grant = self
            .grants
            .get_mut(id)
            .ok_or_else(|| format!("{}: grant '{}' not found", GSVC_ERR_NOT_FOUND, id))?;

        let old_state = grant.state;
        grant
            .transition_to(target_state)
            .map_err(|e| format!("{}: {}", GSVC_ERR_INVALID_TRANSITION, e))?;

        if let Some(set) = self.by_state.get_mut(&old_state) {
            set.remove(id);
        }
        self.by_state
            .entry(target_state)
            .or_default()
            .insert(id.to_string());

        Ok(())
    }

    /// Derives and registers an attenuated child grant from an active parent grant (GSVC3).
    pub fn attenuate_grant(
        &mut self,
        parent_id: &str,
        child_id: &str,
        child_subject: &str,
        delegated_rights: Vec<CapabilityRight>,
    ) -> Result<PepGrant, String> {
        let parent = self
            .grants
            .get(parent_id)
            .ok_or_else(|| format!("{}: parent grant '{}' not found", GSVC_ERR_NOT_FOUND, parent_id))?;

        let child = parent
            .attenuate(child_id, child_subject, delegated_rights)
            .map_err(|e| format!("{}: {}", GSVC_ERR_ATTENUATION, e))?;

        self.issue_grant(child.clone())?;
        Ok(child)
    }

    /// Evaluates if a grant authorizes an action for a subject, right, and time window (GSVC4).
    pub fn evaluate_grant(
        &self,
        grant_id: &str,
        subject: &str,
        right: CapabilityRight,
        now_iso: &str,
    ) -> Result<(), String> {
        let grant = self
            .grants
            .get(grant_id)
            .ok_or_else(|| format!("{}: grant '{}' not found", GSVC_ERR_NOT_FOUND, grant_id))?;

        if grant.subject != subject {
            return Err(format!(
                "{}: grant '{}' subject '{}' does not match requester '{}'",
                GSVC_ERR_VALIDATION, grant_id, grant.subject, subject
            ));
        }

        grant.is_usable_at(now_iso).map_err(|e| {
            if e.contains(PEPGRANT_ERR_EXPIRED) {
                format!("{}: {}", GSVC_ERR_EXPIRED, e)
            } else if e.contains(PEPGRANT_ERR_NOT_YET_VALID) {
                format!("{}: {}", GSVC_ERR_NOT_YET_VALID, e)
            } else if e.contains(PEPGRANT_ERR_QUOTA_EXCEEDED) {
                format!("{}: {}", GSVC_ERR_QUOTA_EXCEEDED, e)
            } else if e.contains(PEPGRANT_ERR_REVOKED) {
                format!("{}: {}", GSVC_ERR_INVALID_TRANSITION, e)
            } else {
                format!("{}: {}", GSVC_ERR_VALIDATION, e)
            }
        })?;

        if !grant.rights.contains(&right) {
            return Err(format!(
                "{}: grant '{}' does not confer right '{}'",
                GSVC_ERR_ATTENUATION, grant_id, right
            ));
        }

        Ok(())
    }

    /// Meters resource consumption (invocations and bytes) against a grant (GSVC4).
    pub fn record_grant_usage(&mut self, grant_id: &str, bytes: u64) -> Result<(), String> {
        let grant = self
            .grants
            .get_mut(grant_id)
            .ok_or_else(|| format!("{}: grant '{}' not found", GSVC_ERR_NOT_FOUND, grant_id))?;

        let old_state = grant.state;
        grant.record_invocation(bytes)?;

        if grant.state != old_state {
            if let Some(set) = self.by_state.get_mut(&old_state) {
                set.remove(grant_id);
            }
            self.by_state
                .entry(grant.state)
                .or_default()
                .insert(grant_id.to_string());
        }

        Ok(())
    }

    /// Revokes a grant, optionally cascading revocation to all derived descendants (GSVC5).
    pub fn revoke_grant(
        &mut self,
        id: &str,
        revoked_by: &str,
        reason: &str,
        cascade: bool,
    ) -> Result<usize, String> {
        if !self.grants.contains_key(id) {
            return Err(format!("{}: grant '{}' not found", GSVC_ERR_NOT_FOUND, id));
        }

        let mut to_revoke = vec![id.to_string()];
        if cascade {
            let mut i = 0;
            while i < to_revoke.len() {
                let current_parent = &to_revoke[i];
                if let Some(children) = self.by_parent.get(current_parent) {
                    for child in children {
                        if !to_revoke.contains(child) {
                            to_revoke.push(child.clone());
                        }
                    }
                }
                i += 1;
            }
        }

        let mut revoked_count = 0;
        for grant_id in to_revoke {
            if let Some(grant) = self.grants.get_mut(&grant_id) {
                if grant.state != PepGrantState::Revoked {
                    let old_state = grant.state;
                    grant
                        .revoke(revoked_by, reason)
                        .map_err(|e| format!("{}: {}", GSVC_ERR_INVALID_TRANSITION, e))?;

                    if let Some(set) = self.by_state.get_mut(&old_state) {
                        set.remove(&grant_id);
                    }
                    self.by_state
                        .entry(PepGrantState::Revoked)
                        .or_default()
                        .insert(grant_id.clone());

                    revoked_count += 1;
                }
            }
        }

        Ok(revoked_count)
    }

    /// Sweeps active/suspended grants and transitions expired grants to Expired (GSVC6).
    pub fn sweep_expired(&mut self, now_iso: &str) -> Result<usize, String> {
        let mut candidates = Vec::new();
        if let Some(active) = self.by_state.get(&PepGrantState::Active) {
            candidates.extend(active.clone());
        }
        if let Some(suspended) = self.by_state.get(&PepGrantState::Suspended) {
            candidates.extend(suspended.clone());
        }

        let mut swept_count = 0;
        for grant_id in candidates {
            if let Some(grant) = self.grants.get_mut(&grant_id) {
                let mut should_expire = false;

                if let Some(ref exp) = grant.constraints.expires_at {
                    if let (Ok(now_dt), Ok(exp_dt)) = (
                        chrono::DateTime::parse_from_rfc3339(now_iso),
                        chrono::DateTime::parse_from_rfc3339(exp),
                    ) {
                        if now_dt >= exp_dt {
                            should_expire = true;
                        }
                    }
                }

                if let Some(max_inv) = grant.constraints.max_invocations {
                    if grant.constraints.invocations_used >= max_inv {
                        should_expire = true;
                    }
                }

                if let Some(max_b) = grant.constraints.max_bytes {
                    if grant.constraints.bytes_used >= max_b {
                        should_expire = true;
                    }
                }

                if should_expire {
                    let old_state = grant.state;
                    grant.state = PepGrantState::Expired;
                    grant.updated_at = Utc::now().to_rfc3339();

                    if let Some(set) = self.by_state.get_mut(&old_state) {
                        set.remove(&grant_id);
                    }
                    self.by_state
                        .entry(PepGrantState::Expired)
                        .or_default()
                        .insert(grant_id.clone());

                    swept_count += 1;
                }
            }
        }

        Ok(swept_count)
    }

    /// Saves the grant service state to disk atomically (GSVC6).
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        validate_grant_service_path(path)?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{}: failed to create directory {:?}: {}", GSVC_ERR_IO, parent, e))?;
        }

        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: serialization failed: {}", GSVC_ERR_VALIDATION, e))?;

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp_path = path.with_extension(format!("tmp.{}.{}", std::process::id(), nonce));
        std::fs::write(&tmp_path, serialized)
            .map_err(|e| format!("{}: failed to write temporary file {:?}: {}", GSVC_ERR_IO, tmp_path, e))?;

        if let Err(e) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("{}: atomic rename failed: {}", GSVC_ERR_IO, e));
        }

        Ok(())
    }

    /// Loads the grant service state from disk, rebuilding all secondary indices (GSVC1, GSVC6).
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        validate_grant_service_path(path)?;

        if !path.exists() {
            return Err(format!("{}: file {:?} does not exist", GSVC_ERR_NOT_FOUND, path));
        }

        let meta = std::fs::metadata(path)
            .map_err(|e| format!("{}: failed to read metadata: {}", GSVC_ERR_IO, e))?;
        if meta.is_dir() {
            return Err(format!("{}: path {:?} is a directory, expected JSON file", GSVC_ERR_VALIDATION, path));
        }
        if meta.len() > MAX_GRANT_SERVICE_STORE_SIZE {
            return Err(format!(
                "{}: file size {} exceeds limit of {}",
                GSVC_ERR_CAPACITY, meta.len(), MAX_GRANT_SERVICE_STORE_SIZE
            ));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("{}: failed to read file: {}", GSVC_ERR_IO, e))?;

        let mut service: Self = serde_json::from_str(&content)
            .map_err(|e| format!("{}: deserialization failed: {}", GSVC_ERR_VALIDATION, e))?;

        service.rebuild_indexes();
        service.storage_path = Some(path.to_path_buf());
        Ok(service)
    }

    /// Flushes state to the configured storage path.
    pub fn sync(&self) -> Result<(), String> {
        match &self.storage_path {
            Some(p) => self.save_to_path(p),
            None => Err(format!("{}: no storage path configured on service", GSVC_ERR_IO)),
        }
    }
}
