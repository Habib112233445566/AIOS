//! Capability Service (CSERV1..CSERV6) for AIOS Security Kernel.
//!
//! Provides registry management, indexing, root capability issuance,
//! managed monotonic attenuation, cascade revocation, and persistence.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::capability::{
    Capability, CapabilityConstraints, CapabilityError, CapabilityRight, CapabilityScope,
};

/// Maximum permissible file size for capability registry persistence (10 MB).
pub const MAX_CAPABILITY_STORE_SIZE: u64 = 10_485_760;

/// Maximum number of capabilities allowed in the in-memory registry.
pub const MAX_CAPABILITIES_IN_REGISTRY: usize = 10_000;

pub const CSERV_IO_ERROR: &str = "CSERV_IO_ERROR";
pub const CSERV_VALIDATION_ERROR: &str = "CSERV_VALIDATION_ERROR";
pub const CSERV_NOT_FOUND: &str = "CSERV_NOT_FOUND";

/// Validates that a storage path is safe and compliant with registry storage policies.
pub fn validate_service_path(path: &Path) -> Result<(), String> {
    let path_str = path.to_string_lossy();
    if path_str.len() > 1024 {
        return Err(format!("{}: path length exceeds 1024 characters", CSERV_VALIDATION_ERROR));
    }
    if path_str.chars().any(|c| c.is_control()) {
        return Err(format!("{}: path contains control characters", CSERV_VALIDATION_ERROR));
    }
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(format!("{}: path traversal ('..') is not allowed", CSERV_VALIDATION_ERROR));
        }
    }
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => Ok(()),
        _ => Err(format!("{}: file must have a .json extension", CSERV_VALIDATION_ERROR)),
    }
}

/// Authoritative in-memory registry and lifecycle manager for capabilities (CSERV1..CSERV6).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityService {
    capabilities: HashMap<String, Capability>,
    by_subject: HashMap<String, HashSet<String>>,
    by_parent: HashMap<String, HashSet<String>>,
    #[serde(skip)]
    storage_path: Option<PathBuf>,
}

impl CapabilityService {
    /// Creates a new, empty capability service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the backing storage path for capability persistence.
    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(path);
        self
    }

    /// Total number of capabilities in the registry.
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Checks if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    /// Issues a new root capability with validated inputs (CSERV2).
    pub fn issue_root_capability(
        &mut self,
        issuer: &str,
        subject: &str,
        scope: CapabilityScope,
        rights: Vec<CapabilityRight>,
        constraints: CapabilityConstraints,
    ) -> Result<Capability, CapabilityError> {
        if self.capabilities.len() >= MAX_CAPABILITIES_IN_REGISTRY {
            return Err(CapabilityError::ValidationError(format!(
                "{}: registry capacity limit reached ({})",
                CSERV_VALIDATION_ERROR, MAX_CAPABILITIES_IN_REGISTRY
            )));
        }

        if issuer != "kernel" && !issuer.starts_with("admin:") {
            return Err(CapabilityError::ValidationError(format!(
                "{}: root capabilities can only be issued by 'kernel' or 'admin:*', got '{}'",
                CSERV_VALIDATION_ERROR, issuer
            )));
        }

        let cap = Capability::new(issuer, subject, scope, rights, constraints)?;
        self.register_capability(cap.clone());
        Ok(cap)
    }

    /// Retrieves a reference to a capability by its ID (CSERV1).
    pub fn get_capability(&self, id: &str) -> Option<&Capability> {
        self.capabilities.get(id)
    }

    /// Retrieves a mutable reference to a capability by its ID.
    pub fn get_capability_mut(&mut self, id: &str) -> Option<&mut Capability> {
        self.capabilities.get_mut(id)
    }

    /// Retrieves all active, non-revoked capabilities granted to a subject (CSERV1).
    pub fn get_capabilities_for_subject(&self, subject: &str) -> Vec<Capability> {
        let now = Utc::now();
        self.by_subject
            .get(subject)
            .into_iter()
            .flatten()
            .filter_map(|id| self.capabilities.get(id))
            .filter(|cap| !cap.revoked && cap.check_validity_at(now).is_ok())
            .cloned()
            .collect()
    }

    /// Derives an attenuated child capability and indexes its lineage (CSERV3).
    pub fn attenuate_capability(
        &mut self,
        parent_id: &str,
        new_subject: &str,
        narrowed_scope: Option<CapabilityScope>,
        subset_rights: Vec<CapabilityRight>,
        narrowed_constraints: Option<CapabilityConstraints>,
    ) -> Result<Capability, CapabilityError> {
        if self.capabilities.len() >= MAX_CAPABILITIES_IN_REGISTRY {
            return Err(CapabilityError::ValidationError(format!(
                "{}: registry capacity limit reached ({})",
                CSERV_VALIDATION_ERROR, MAX_CAPABILITIES_IN_REGISTRY
            )));
        }

        let parent = self
            .capabilities
            .get(parent_id)
            .ok_or_else(|| CapabilityError::ValidationError(format!("{}: capability '{}' not found", CSERV_NOT_FOUND, parent_id)))?;

        let child = parent.attenuate(new_subject, narrowed_scope, subset_rights, narrowed_constraints)?;
        self.register_capability(child.clone());
        Ok(child)
    }

    /// Revokes a capability and transitively revokes all derived child capabilities (CSERV4).
    pub fn revoke_capability(&mut self, id: &str) -> Result<Vec<String>, CapabilityError> {
        if !self.capabilities.contains_key(id) {
            return Err(CapabilityError::ValidationError(format!("{}: capability '{}' not found", CSERV_NOT_FOUND, id)));
        }

        let mut revoked_ids = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = vec![id.to_string()];

        while let Some(current_id) = queue.pop() {
            if !visited.insert(current_id.clone()) {
                continue;
            }
            if let Some(cap) = self.capabilities.get_mut(&current_id) {
                if !cap.revoked {
                    cap.revoke();
                    revoked_ids.push(current_id.clone());
                }
            }
            if let Some(children) = self.by_parent.get(&current_id) {
                for child_id in children {
                    if !visited.contains(child_id) {
                        queue.push(child_id.clone());
                    }
                }
            }
        }

        Ok(revoked_ids)
    }

    /// Checks if a subject possesses a valid capability covering the requested scope and right.
    pub fn check_access(
        &self,
        subject: &str,
        requested_scope: &CapabilityScope,
        required_right: CapabilityRight,
    ) -> Result<&Capability, CapabilityError> {
        let now = Utc::now();
        let candidate = self
            .by_subject
            .get(subject)
            .into_iter()
            .flatten()
            .filter_map(|id| self.capabilities.get(id))
            .find(|cap| {
                !cap.revoked
                    && cap.check_right(required_right).is_ok()
                    && cap.matches_scope(requested_scope)
                    && cap.check_validity_at(now).is_ok()
            });

        candidate.ok_or_else(|| CapabilityError::RightNotGranted(required_right))
    }

    /// Fast boolean check for whether a subject possesses a valid capability.
    pub fn has_active_capability(
        &self,
        subject: &str,
        requested_scope: &CapabilityScope,
        required_right: CapabilityRight,
    ) -> bool {
        self.check_access(subject, requested_scope, required_right).is_ok()
    }

    /// Consumes an invocation against a specified capability ID in the store.
    pub fn consume_invocation_on_capability(&mut self, id: &str) -> Result<(), CapabilityError> {
        let cap = self
            .capabilities
            .get_mut(id)
            .ok_or_else(|| CapabilityError::ValidationError(format!("{}: capability '{}' not found", CSERV_NOT_FOUND, id)))?;
        cap.consume_invocation()
    }

    /// Consumes bytes against a specified capability ID in the store.
    pub fn consume_bytes_on_capability(&mut self, id: &str, bytes: u64) -> Result<(), CapabilityError> {
        let cap = self
            .capabilities
            .get_mut(id)
            .ok_or_else(|| CapabilityError::ValidationError(format!("{}: capability '{}' not found", CSERV_NOT_FOUND, id)))?;
        cap.consume_bytes(bytes)
    }

    /// Returns all currently active, non-revoked capabilities in the store.
    pub fn get_active_capabilities(&self) -> Vec<Capability> {
        let now = Utc::now();
        self.capabilities
            .values()
            .filter(|cap| !cap.revoked && cap.check_validity_at(now).is_ok())
            .cloned()
            .collect()
    }

    /// Helper to register and index a capability into internal collections.
    fn register_capability(&mut self, cap: Capability) {
        let id = cap.id.clone();
        let subject = cap.subject.clone();
        if let Some(ref parent_id) = cap.parent_id {
            self.by_parent
                .entry(parent_id.clone())
                .or_default()
                .insert(id.clone());
        }
        self.by_subject
            .entry(subject)
            .or_default()
            .insert(id.clone());
        self.capabilities.insert(id, cap);
    }

    /// Prunes expired capabilities that have no active children (CSERV6).
    pub fn prune_expired(&mut self, now: DateTime<Utc>) -> usize {
        let expired_ids: Vec<String> = self
            .capabilities
            .values()
            .filter(|cap| {
                if let Some(ref exp_str) = cap.constraints.expires_at {
                    if let Ok(exp) = DateTime::parse_from_rfc3339(exp_str) {
                        return now > exp;
                    }
                }
                false
            })
            .map(|c| c.id.clone())
            .collect();

        let mut pruned = 0;
        for id in expired_ids {
            // Only prune if it has no children
            let has_children = self.by_parent.get(&id).map(|c| !c.is_empty()).unwrap_or(false);
            if !has_children {
                if let Some(cap) = self.capabilities.remove(&id) {
                    if let Some(set) = self.by_subject.get_mut(&cap.subject) {
                        set.remove(&id);
                    }
                    if let Some(ref parent_id) = cap.parent_id {
                        if let Some(set) = self.by_parent.get_mut(parent_id) {
                            set.remove(&id);
                        }
                    }
                    pruned += 1;
                }
            }
        }
        pruned
    }

    /// Persists registry atomically to disk (CSERV5).
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        validate_service_path(path)?;

        if let Ok(meta) = fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() {
                return Err(format!("{}: target path {:?} is a symlink", CSERV_IO_ERROR, path));
            }
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("{}: failed to create directory {:?}: {}", CSERV_IO_ERROR, parent, e))?;
        }

        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: serialization failed: {}", CSERV_VALIDATION_ERROR, e))?;

        if serialized.len() as u64 > MAX_CAPABILITY_STORE_SIZE {
            return Err(format!(
                "{}: serialized capability registry exceeds maximum size {} bytes",
                CSERV_VALIDATION_ERROR, MAX_CAPABILITY_STORE_SIZE
            ));
        }

        let tmp_path = format!("{}.tmp.{}", path.to_string_lossy(), std::process::id());
        let tmp_buf = PathBuf::from(&tmp_path);

        if let Err(e) = fs::write(&tmp_buf, serialized.as_bytes()) {
            let _ = fs::remove_file(&tmp_buf);
            return Err(format!("{}: failed to write temporary file {:?}: {}", CSERV_IO_ERROR, tmp_buf, e));
        }

        if let Err(e) = fs::rename(&tmp_buf, path) {
            let _ = fs::remove_file(&tmp_buf);
            return Err(format!("{}: failed to rename {:?} to {:?}: {}", CSERV_IO_ERROR, tmp_buf, path, e));
        }

        Ok(())
    }

    /// Loads and rebuilds registry from disk (CSERV5).
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        validate_service_path(path)?;

        if let Ok(meta) = fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() {
                return Err(format!("{}: path {:?} is a symlink", CSERV_IO_ERROR, path));
            }
            if meta.len() > MAX_CAPABILITY_STORE_SIZE {
                return Err(format!(
                    "{}: capability file {:?} exceeds maximum size {} bytes",
                    CSERV_VALIDATION_ERROR, path, MAX_CAPABILITY_STORE_SIZE
                ));
            }
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("{}: failed to read {:?}: {}", CSERV_IO_ERROR, path, e))?;

        let mut service: CapabilityService = serde_json::from_str(&content)
            .map_err(|e| format!("{}: failed to deserialize {:?}: {}", CSERV_VALIDATION_ERROR, path, e))?;

        if service.capabilities.len() > MAX_CAPABILITIES_IN_REGISTRY {
            return Err(format!(
                "{}: capability file contains {} entries, exceeding maximum limit {}",
                CSERV_VALIDATION_ERROR, service.capabilities.len(), MAX_CAPABILITIES_IN_REGISTRY
            ));
        }

        // Rebuild indexes
        service.by_subject.clear();
        service.by_parent.clear();
        for (id, cap) in &service.capabilities {
            service.by_subject.entry(cap.subject.clone()).or_default().insert(id.clone());
            if let Some(ref p_id) = cap.parent_id {
                service.by_parent.entry(p_id.clone()).or_default().insert(id.clone());
            }
        }

        service.storage_path = Some(path.to_path_buf());
        Ok(service)
    }
}
