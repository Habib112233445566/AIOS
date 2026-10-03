//! Secrets Handling Core Service (SECSVC1..SECSVC7).
//!
//! Provides the runtime vault service, scoped authorization gate,
//! lifecycle operations, and atomic persistence.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::secret_data_model::{
    SecretEntry, SecretKind, SecretMetadata, SecretScope, SecretValue,
};
use crate::secret_config::SecretConfig;
use crate::secret_policy::{SecretPolicyVerdict, SecretSecurityPolicy};

pub const SECSVC_ERR_NOT_FOUND: &str = "SECSVC_ERR_NOT_FOUND";
pub const SECSVC_ERR_ACCESS_DENIED: &str = "SECSVC_ERR_ACCESS_DENIED";
pub const SECSVC_ERR_INACCESSIBLE: &str = "SECSVC_ERR_INACCESSIBLE";
pub const SECSVC_ERR_CAPACITY_EXCEEDED: &str = "SECSVC_ERR_CAPACITY_EXCEEDED";
pub const SECSVC_ERR_FILE_SIZE: &str = "SECSVC_ERR_FILE_SIZE";
pub const SECSVC_ERR_PATH_TRAVERSAL: &str = "SECSVC_ERR_PATH_TRAVERSAL";
pub const SECSVC_ERR_IO: &str = "SECSVC_ERR_IO";
pub const SECSVC_ERR_PARSE: &str = "SECSVC_ERR_PARSE";

pub const MAX_SECRETS_VAULT_CAPACITY: usize = 1024;
pub const MAX_SECRETS_STORE_SIZE: u64 = 1024 * 1024; // 1 MiB
pub const DEFAULT_SECRETS_VAULT_PATH: &str = "docs/secrets_vault.json";

/// On-disk serialized secret record pairing metadata with hex-encoded payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredSecretRecord {
    pub metadata: SecretMetadata,
    pub payload_hex: String,
}

/// On-disk vault storage wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultPayload {
    pub version: String,
    pub secrets: HashMap<String, StoredSecretRecord>,
}

/// Helper encoding functions.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("hex string length must be even".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| format!("invalid hex character at {}: {}", i, e))
        })
        .collect()
}

/// Core runtime vault service.
pub struct SecretService {
    entries: HashMap<String, SecretEntry>,
    config: SecretConfig,
    policy: SecretSecurityPolicy,
}

impl Default for SecretService {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SecretService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretService")
            .field("entries_count", &self.entries.len())
            .field("config", &self.config)
            .field("policy", &self.policy)
            .finish()
        }
}

impl SecretService {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            config: SecretConfig::default(),
            policy: SecretSecurityPolicy::default(),
        }
    }

    pub fn new_with_config(config: SecretConfig) -> Self {
        Self {
            entries: HashMap::new(),
            config,
            policy: SecretSecurityPolicy::default(),
        }
    }

    pub fn config(&self) -> &SecretConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut SecretConfig {
        &mut self.config
    }

    pub fn policy(&self) -> &SecretSecurityPolicy {
        &self.policy
    }

    pub fn policy_mut(&mut self) -> &mut SecretSecurityPolicy {
        &mut self.policy
    }

    pub fn set_policy(&mut self, policy: SecretSecurityPolicy) {
        self.policy = policy;
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn validate_path(path: &Path) -> Result<(), String> {
        let path_str = path.to_string_lossy();
        if path_str.trim().is_empty() {
            return Err(format!("{}: path cannot be empty", SECSVC_ERR_PATH_TRAVERSAL));
        }
        if path_str.contains("..") {
            return Err(format!("{}: path traversal ('..') is prohibited", SECSVC_ERR_PATH_TRAVERSAL));
        }
        if path_str.chars().any(|c| c.is_control()) {
            return Err(format!("{}: path contains control characters", SECSVC_ERR_PATH_TRAVERSAL));
        }
        Ok(())
    }

    /// Stores a new secret or updates existing, verifying capacity bounds and policy.
    pub fn store_secret(&mut self, entry: SecretEntry) -> Result<(), String> {
        entry.metadata.validate()?;
        match self.policy.evaluate_store(&entry) {
            SecretPolicyVerdict::Deny { reason, code } => {
                return Err(format!("{}: {}", code, reason));
            }
            _ => {}
        }
        let val_len = entry.value.as_bytes().len();
        if val_len > self.config.max_payload_bytes {
            return Err(format!(
                "{}: secret payload size {} exceeds configured maximum {}",
                SECSVC_ERR_FILE_SIZE, val_len, self.config.max_payload_bytes
            ));
        }
        if !self.entries.contains_key(&entry.metadata.id) && self.entries.len() >= self.config.max_secrets_capacity {
            return Err(format!(
                "{}: vault capacity of {} reached",
                SECSVC_ERR_CAPACITY_EXCEEDED, self.config.max_secrets_capacity
            ));
        }
        self.entries.insert(entry.metadata.id.clone(), entry);
        Ok(())
    }

    /// Retrieves the secret payload, checking caller scope and accessibility.
    pub fn get_secret(&self, id: &str, caller_scope: &SecretScope) -> Result<SecretValue, String> {
        let entry = self.entries.get(id).ok_or_else(|| {
            format!("{}: secret with id '{}' not found", SECSVC_ERR_NOT_FOUND, id)
        })?;

        if !caller_scope.allows(&entry.metadata.scope) {
            return Err(format!(
                "{}: caller scope '{}' does not grant access to secret scope '{}'",
                SECSVC_ERR_ACCESS_DENIED,
                caller_scope.as_str(),
                entry.metadata.scope.as_str()
            ));
        }

        if !entry.metadata.state.is_accessible() || entry.metadata.is_expired() {
            return Err(format!(
                "{}: secret '{}' is in non-accessible state '{}'",
                SECSVC_ERR_INACCESSIBLE,
                id,
                entry.metadata.state.as_str()
            ));
        }

        SecretValue::new(entry.value.as_bytes())
    }

    pub fn contains(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }

    pub fn remove_secret(&mut self, id: &str) -> Result<SecretMetadata, String> {
        self.entries.remove(id)
            .map(|e| e.metadata)
            .ok_or_else(|| format!("{}: secret with id '{}' not found", SECSVC_ERR_NOT_FOUND, id))
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn get_secret_with_privilege(
        &self,
        id: &str,
        priv_ctx: &crate::privilege_data_model::PrivilegeContext,
    ) -> Result<SecretValue, String> {
        let entry = self.entries.get(id).ok_or_else(|| {
            format!("{}: secret with id '{}' not found", SECSVC_ERR_NOT_FOUND, id)
        })?;

        let allowed = match priv_ctx.active_level {
            crate::privilege_data_model::PrivilegeLevel::SystemKernel |
            crate::privilege_data_model::PrivilegeLevel::Admin => true,
            crate::privilege_data_model::PrivilegeLevel::Operator => {
                match &entry.metadata.scope {
                    SecretScope::Global | SecretScope::Environment(_) => true,
                    SecretScope::Actor(a) => a == &priv_ctx.actor_id,
                    SecretScope::Session(_) => true,
                }
            }
            crate::privilege_data_model::PrivilegeLevel::User => {
                match &entry.metadata.scope {
                    SecretScope::Global => true,
                    SecretScope::Actor(a) => a == &priv_ctx.actor_id,
                    SecretScope::Session(_) => true,
                    SecretScope::Environment(_) => false,
                }
            }
            crate::privilege_data_model::PrivilegeLevel::Guest => false,
        };

        if !allowed {
            return Err(format!(
                "{}: actor '{}' (tier {}) denied access to secret '{}' with scope '{}'",
                SECSVC_ERR_ACCESS_DENIED,
                priv_ctx.actor_id,
                priv_ctx.active_level.as_str(),
                id,
                entry.metadata.scope.as_str()
            ));
        }

        if !entry.metadata.state.is_accessible() || entry.metadata.is_expired() {
            return Err(format!(
                "{}: secret '{}' is in non-accessible state '{}'",
                SECSVC_ERR_INACCESSIBLE,
                id,
                entry.metadata.state.as_str()
            ));
        }

        SecretValue::new(entry.value.as_bytes())
    }

    /// Retrieves secret metadata without exposing payload.
    pub fn get_metadata(&self, id: &str) -> Result<SecretMetadata, String> {
        self.entries.get(id)
            .map(|e| e.metadata.clone())
            .ok_or_else(|| format!("{}: secret with id '{}' not found", SECSVC_ERR_NOT_FOUND, id))
    }

    /// Lists secret metadata with optional filtering.
    pub fn list_metadata(
        &self,
        filter_kind: Option<SecretKind>,
        filter_scope: Option<&SecretScope>,
    ) -> Vec<SecretMetadata> {
        let mut results: Vec<SecretMetadata> = self.entries.values()
            .filter(|e| {
                if let Some(k) = filter_kind {
                    if e.metadata.kind != k {
                        return false;
                    }
                }
                if let Some(s) = filter_scope {
                    if !s.allows(&e.metadata.scope) {
                        return false;
                    }
                }
                true
            })
            .map(|e| e.metadata.clone())
            .collect();
        results.sort_by(|a, b| a.id.cmp(&b.id));
        results
    }

    /// Rotates the secret value for target id, verifying security policy.
    pub fn rotate_secret(&mut self, id: &str, new_value: &[u8]) -> Result<(), String> {
        let entry = self.entries.get_mut(id).ok_or_else(|| {
            format!("{}: secret with id '{}' not found", SECSVC_ERR_NOT_FOUND, id)
        })?;
        match self.policy.evaluate_rotate(entry, new_value.len()) {
            SecretPolicyVerdict::Deny { reason, code } => {
                return Err(format!("{}: {}", code, reason));
            }
            _ => {}
        }
        entry.rotate(new_value)
    }

    /// Revokes target secret, wiping its content.
    pub fn revoke_secret(&mut self, id: &str) -> Result<(), String> {
        let entry = self.entries.get_mut(id).ok_or_else(|| {
            format!("{}: secret with id '{}' not found", SECSVC_ERR_NOT_FOUND, id)
        })?;
        entry.revoke()
    }

    /// Saves vault state to disk atomically.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        Self::validate_path(path)?;

        if path.exists() {
            let meta = fs::symlink_metadata(path).map_err(|e| format!("{}: {}", SECSVC_ERR_IO, e))?;
            if meta.file_type().is_symlink() {
                return Err(format!("{}: symbolic links are not permitted for secrets store files", SECSVC_ERR_PATH_TRAVERSAL));
            }
        }

        let mut stored_map = HashMap::new();
        for (id, entry) in &self.entries {
            stored_map.insert(id.clone(), StoredSecretRecord {
                metadata: entry.metadata.clone(),
                payload_hex: hex_encode(entry.value.as_bytes()),
            });
        }

        let payload = VaultPayload {
            version: "1.0.0".into(),
            secrets: stored_map,
        };

        let json_data = serde_json::to_string_pretty(&payload)
            .map_err(|e| format!("{}: {}", SECSVC_ERR_PARSE, e))?;

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("{}: failed to create parent dir: {}", SECSVC_ERR_IO, e))?;
            }
        }

        let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));
        fs::write(&tmp_path, json_data)
            .map_err(|e| format!("{}: failed to write temp file: {}", SECSVC_ERR_IO, e))?;

        if let Err(e) = fs::rename(&tmp_path, path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(format!("{}: atomic rename failed: {}", SECSVC_ERR_IO, e));
        }

        Ok(())
    }

    /// Loads vault state from disk using default configuration.
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        Self::load_from_path_with_config(path, SecretConfig::default())
    }

    /// Loads vault state from disk with size bounds checking governed by `config`.
    pub fn load_from_path_with_config(path: &Path, config: SecretConfig) -> Result<Self, String> {
        Self::validate_path(path)?;

        if !path.exists() {
            return Ok(Self::new_with_config(config));
        }

        let meta = fs::symlink_metadata(path).map_err(|e| format!("{}: {}", SECSVC_ERR_IO, e))?;
        if meta.file_type().is_symlink() {
            return Err(format!("{}: symbolic links are not permitted for secrets store files", SECSVC_ERR_PATH_TRAVERSAL));
        }
        if meta.len() > config.max_store_file_bytes {
            return Err(format!(
                "{}: vault file size {} exceeds limit of {} bytes",
                SECSVC_ERR_FILE_SIZE, meta.len(), config.max_store_file_bytes
            ));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("{}: {}", SECSVC_ERR_IO, e))?;

        let payload: VaultPayload = serde_json::from_str(&content)
            .map_err(|e| format!("{}: {}", SECSVC_ERR_PARSE, e))?;

        let mut service = Self::new_with_config(config);
        for (_, rec) in payload.secrets {
            let raw_bytes = hex_decode(&rec.payload_hex)
                .map_err(|e| format!("{}: {}", SECSVC_ERR_PARSE, e))?;
            let val = SecretValue::new(&raw_bytes)?;
            service.entries.insert(rec.metadata.id.clone(), SecretEntry {
                metadata: rec.metadata,
                value: val,
            });
        }

        Ok(service)
    }
}
