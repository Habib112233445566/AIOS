//! PEP Decision Service (PEPSERV1..PEPSERV6) for AIOS Security Kernel.
//!
//! Authoritative in-memory registry, indexer, and evaluation engine for
//! PEP policy rules with atomic disk persistence.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::pep_decision::{
    evaluate_rules, PepCombiningAlgorithm, PepDecision, PepPolicyRule, PepRequest,
};

/// Maximum permissible number of rules in the service registry (PEPSERV6).
pub const MAX_RULES_IN_SERVICE: usize = 5000;

/// Maximum permissible file size for policy service persistence (10 MB).
pub const MAX_PEP_SERVICE_STORE_SIZE: u64 = 10_485_760;

pub const PEPSERV_ERR_CAPACITY: &str = "PEPSERV_ERR_CAPACITY";
pub const PEPSERV_ERR_DUPLICATE_ID: &str = "PEPSERV_ERR_DUPLICATE_ID";
pub const PEPSERV_ERR_IO: &str = "PEPSERV_ERR_IO";
pub const PEPSERV_ERR_VALIDATION: &str = "PEPSERV_ERR_VALIDATION";

/// Validates that a storage path is safe and compliant with policy store rules.
pub fn validate_pep_service_path(path: &Path) -> Result<(), String> {
    let path_str = path.to_string_lossy();
    if path_str.len() > 1024 {
        return Err(format!("{}: path length exceeds 1024 characters", PEPSERV_ERR_VALIDATION));
    }
    if path_str.chars().any(|c| c.is_control()) {
        return Err(format!("{}: path contains control characters", PEPSERV_ERR_VALIDATION));
    }
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(format!("{}: path traversal ('..') is not allowed", PEPSERV_ERR_VALIDATION));
        }
    }
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => Ok(()),
        _ => Err(format!("{}: file must have a .json extension", PEPSERV_ERR_VALIDATION)),
    }
}

/// Authoritative in-memory registry and evaluator for policy rules (PEPSERV1..PEPSERV6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PepDecisionService {
    rules: HashMap<String, PepPolicyRule>,
    by_subject: HashMap<String, HashSet<String>>,
    by_action: HashMap<String, HashSet<String>>,
    default_algorithm: PepCombiningAlgorithm,
    #[serde(skip)]
    storage_path: Option<PathBuf>,
}

impl Default for PepDecisionService {
    fn default() -> Self {
        Self {
            rules: HashMap::new(),
            by_subject: HashMap::new(),
            by_action: HashMap::new(),
            default_algorithm: PepCombiningAlgorithm::DenyOverrides,
            storage_path: None,
        }
    }
}

impl PepDecisionService {
    /// Creates a new, empty PEP decision service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the default combining algorithm.
    pub fn with_algorithm(mut self, algorithm: PepCombiningAlgorithm) -> Self {
        self.default_algorithm = algorithm;
        self
    }

    /// Returns the active default combining algorithm.
    pub fn algorithm(&self) -> PepCombiningAlgorithm {
        self.default_algorithm
    }

    /// Configures the backing storage path for policy persistence.
    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(path);
        self
    }

    /// Total number of rules in the service.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Checks if the rule registry is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Adds a policy rule to the service and updates indexes (PEPSERV3, PEPSERV6).
    pub fn add_rule(&mut self, rule: PepPolicyRule) -> Result<(), String> {
        if self.rules.len() >= MAX_RULES_IN_SERVICE && !self.rules.contains_key(&rule.id) {
            return Err(format!(
                "{}: registry capacity limit reached ({})",
                PEPSERV_ERR_CAPACITY, MAX_RULES_IN_SERVICE
            ));
        }

        let rule_id = rule.id.clone();
        if let Some(ref subj) = rule.target_subject {
            self.by_subject.entry(subj.clone()).or_default().insert(rule_id.clone());
        }
        if let Some(ref act) = rule.target_action {
            self.by_action.entry(act.clone()).or_default().insert(rule_id.clone());
        }

        self.rules.insert(rule_id, rule);
        Ok(())
    }

    /// Removes a rule by its identifier.
    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        if let Some(rule) = self.rules.remove(rule_id) {
            if let Some(ref subj) = rule.target_subject {
                if let Some(set) = self.by_subject.get_mut(subj) {
                    set.remove(rule_id);
                }
            }
            if let Some(ref act) = rule.target_action {
                if let Some(set) = self.by_action.get_mut(act) {
                    set.remove(rule_id);
                }
            }
            true
        } else {
            false
        }
    }

    /// Retrieves a reference to a rule by ID.
    pub fn get_rule(&self, rule_id: &str) -> Option<&PepPolicyRule> {
        self.rules.get(rule_id)
    }

    /// Returns a list of all registered policy rules.
    pub fn list_rules(&self) -> Vec<PepPolicyRule> {
        self.rules.values().cloned().collect()
    }

    /// Evaluates an authorization request against registered rules using default algorithm.
    pub fn evaluate(&self, req: &PepRequest) -> PepDecision {
        self.evaluate_with_algorithm(req, self.default_algorithm)
    }

    /// Evaluates an authorization request against registered rules using specified algorithm.
    pub fn evaluate_with_algorithm(
        &self,
        req: &PepRequest,
        algorithm: PepCombiningAlgorithm,
    ) -> PepDecision {
        let rules_vec: Vec<PepPolicyRule> = self.rules.values().cloned().collect();
        evaluate_rules(&rules_vec, req, algorithm)
    }

    /// Persists policy rules atomically to disk (PEPSERV5).
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        validate_pep_service_path(path)?;

        if let Ok(meta) = fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() {
                return Err(format!("{}: target path {:?} is a symlink", PEPSERV_ERR_IO, path));
            }
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("{}: failed to create directory {:?}: {}", PEPSERV_ERR_IO, parent, e))?;
        }

        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| format!("{}: serialization failed: {}", PEPSERV_ERR_VALIDATION, e))?;

        let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));
        fs::write(&tmp_path, serialized)
            .map_err(|e| format!("{}: failed to write temporary file {:?}: {}", PEPSERV_ERR_IO, tmp_path, e))?;

        fs::rename(&tmp_path, path)
            .map_err(|e| format!("{}: atomic rename failed: {}", PEPSERV_ERR_IO, e))?;

        Ok(())
    }

    /// Loads a policy service from a JSON file on disk.
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        validate_pep_service_path(path)?;

        if !path.exists() {
            return Err(format!("{}: path {:?} does not exist", PEPSERV_ERR_IO, path));
        }

        let meta = fs::metadata(path)
            .map_err(|e| format!("{}: failed to read metadata for {:?}: {}", PEPSERV_ERR_IO, path, e))?;
        if meta.len() > MAX_PEP_SERVICE_STORE_SIZE {
            return Err(format!(
                "{}: file size {} exceeds limit of {}",
                PEPSERV_ERR_VALIDATION, meta.len(), MAX_PEP_SERVICE_STORE_SIZE
            ));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("{}: failed to read {:?}: {}", PEPSERV_ERR_IO, path, e))?;

        let mut service: Self = serde_json::from_str(&content)
            .map_err(|e| format!("{}: deserialization failed: {}", PEPSERV_ERR_VALIDATION, e))?;

        service.storage_path = Some(path.to_path_buf());
        Ok(service)
    }

    /// Loads a policy service from disk or creates a default empty instance if not found.
    pub fn load_or_create(path: &Path) -> Result<Self, String> {
        if path.exists() {
            Self::load_from_path(path)
        } else {
            let mut service = Self::new();
            service.storage_path = Some(path.to_path_buf());
            Ok(service)
        }
    }

    /// Loads a policy service, quarantining damaged or unparseable files non-destructively (PEPSERV5).
    pub fn load_or_recover(path: &Path) -> (Self, bool, Option<String>) {
        if !path.exists() {
            let mut service = Self::new();
            service.storage_path = Some(path.to_path_buf());
            return (service, false, None);
        }

        match Self::load_from_path(path) {
            Ok(service) => (service, false, None),
            Err(_) => {
                // Non-destructive quarantine
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S_%6f").to_string();
                let file_name = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "pep_policy.json".to_string());
                let backup_name = format!("{}.bak.{}", file_name, timestamp);
                let backup_path = path.parent().map(|p| p.join(&backup_name)).unwrap_or_else(|| PathBuf::from(&backup_name));

                if let Ok(meta) = fs::symlink_metadata(path) {
                    if !meta.file_type().is_symlink() {
                        let _ = fs::copy(path, &backup_path);
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let _ = fs::set_permissions(&backup_path, fs::Permissions::from_mode(0o600));
                        }
                    }
                }

                let mut fresh = Self::new();
                fresh.storage_path = Some(path.to_path_buf());
                let _ = fresh.save_to_path(path);
                (fresh, true, Some(backup_path.to_string_lossy().to_string()))
            }
        }
    }
}
