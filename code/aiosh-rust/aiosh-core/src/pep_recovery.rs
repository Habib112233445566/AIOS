//! PEP Decision Engine Policy Recovery & Validation (`PEPRECV1..PEPRECV6`).
//!
//! Provides comprehensive multi-level validation of policy files, deterministic
//! diagnostic reporting, non-destructive quarantine backups, and rule salvage strategies.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::pep_decision_service::{PepDecisionService, validate_pep_service_path, MAX_RULES_IN_SERVICE, MAX_PEP_SERVICE_STORE_SIZE};
use crate::pep_decision::{PepPolicyRule, PepDecisionEffect};

pub const PEPRECV_ERR_IO: &str = "PEPRECV_ERR_IO";
pub const PEPRECV_ERR_PATH_TRAVERSAL: &str = "PEPRECV_ERR_PATH_TRAVERSAL";
pub const PEPRECV_ERR_FILE_SIZE: &str = "PEPRECV_ERR_FILE_SIZE";
pub const PEPRECV_ERR_PARSE: &str = "PEPRECV_ERR_PARSE";
pub const PEPRECV_ERR_RULE_SYNTAX: &str = "PEPRECV_ERR_RULE_SYNTAX";
pub const PEPRECV_ERR_DUPLICATE_ID: &str = "PEPRECV_ERR_DUPLICATE_ID";
pub const PEPRECV_ERR_CAPACITY: &str = "PEPRECV_ERR_CAPACITY";
pub const PEPRECV_ERR_CHECKSUM: &str = "PEPRECV_ERR_CHECKSUM";

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PepIssueSeverity {
    Error,
    Warning,
}

/// A specific diagnostic issue identified during store validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepValidationIssue {
    pub rule_id: Option<String>,
    pub code: String,
    pub message: String,
    pub severity: PepIssueSeverity,
}

/// Comprehensive report generated after validating a PEP policy store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepValidationReport {
    pub is_valid: bool,
    pub store_path: Option<String>,
    pub sha256_checksum: Option<String>,
    pub total_rules_scanned: usize,
    pub valid_rules_count: usize,
    pub corrupt_rules_count: usize,
    pub issues: Vec<PepValidationIssue>,
    pub inspected_at: String,
}

/// Recovery strategy mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PepRecoveryStrategy {
    #[default]
    StrictFailClosed,
    SalvageValidRules,
    DryRun,
}

/// Result of an executed recovery operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepRecoveryResult {
    pub success: bool,
    pub strategy: PepRecoveryStrategy,
    pub original_path: String,
    pub quarantine_path: Option<String>,
    pub rules_salvaged: usize,
    pub rules_dropped: usize,
    pub validation_report: PepValidationReport,
    pub message: String,
}

/// Validates policy store files and JSON content (PEPRECV1).
pub struct PepStoreValidator;

impl PepStoreValidator {
    /// Computes the hex SHA-256 digest of binary content.
    pub fn compute_sha256(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Validates raw JSON string content against PEP schema and invariants (PEPRECV1, PEPRECV2).
    pub fn validate_content(content: &str, store_path: Option<&Path>) -> PepValidationReport {
        let inspected_at = chrono::Utc::now().to_rfc3339();
        let sha256 = Some(Self::compute_sha256(content.as_bytes()));
        let mut issues = Vec::new();

        // Level 2: Syntactic / JSON Parsing
        let parsed: serde_json::Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                issues.push(PepValidationIssue {
                    rule_id: None,
                    code: PEPRECV_ERR_PARSE.to_string(),
                    message: format!("JSON parsing failed: {}", e),
                    severity: PepIssueSeverity::Error,
                });
                return PepValidationReport {
                    is_valid: false,
                    store_path: store_path.map(|p| p.to_string_lossy().to_string()),
                    sha256_checksum: sha256,
                    total_rules_scanned: 0,
                    valid_rules_count: 0,
                    corrupt_rules_count: 0,
                    issues,
                    inspected_at,
                };
            }
        };

        // Extract rules (support both JSON Array and JSON Object map from PepDecisionService)
        let rules_list: Vec<&serde_json::Value> = match parsed.get("rules") {
            Some(serde_json::Value::Array(arr)) => arr.iter().collect(),
            Some(serde_json::Value::Object(map)) => map.values().collect(),
            _ => {
                issues.push(PepValidationIssue {
                    rule_id: None,
                    code: PEPRECV_ERR_PARSE.to_string(),
                    message: "missing or invalid 'rules' field (expected array or object map)".to_string(),
                    severity: PepIssueSeverity::Error,
                });
                return PepValidationReport {
                    is_valid: false,
                    store_path: store_path.map(|p| p.to_string_lossy().to_string()),
                    sha256_checksum: sha256,
                    total_rules_scanned: 0,
                    valid_rules_count: 0,
                    corrupt_rules_count: 0,
                    issues,
                    inspected_at,
                };
            }
        };

        // Level 4: Capacity Check
        let total_rules = rules_list.len();
        if total_rules > MAX_RULES_IN_SERVICE {
            issues.push(PepValidationIssue {
                rule_id: None,
                code: PEPRECV_ERR_CAPACITY.to_string(),
                message: format!("rule count {} exceeds capacity limit of {}", total_rules, MAX_RULES_IN_SERVICE),
                severity: PepIssueSeverity::Error,
            });
        }

        // Level 3: Semantic Rule Validation
        let mut seen_ids = std::collections::HashSet::new();
        let mut valid_rules_count = 0;
        let mut corrupt_rules_count = 0;

        for (idx, rule_val) in rules_list.iter().enumerate() {
            let mut rule_valid = true;
            let rule_id_opt = rule_val.get("id").and_then(|v| v.as_str());

            if let Some(id) = rule_id_opt {
                if id.is_empty() || id.len() > 128 || id.chars().any(|c| c.is_control()) {
                    issues.push(PepValidationIssue {
                        rule_id: Some(id.to_string()),
                        code: PEPRECV_ERR_RULE_SYNTAX.to_string(),
                        message: format!("rule index {} has invalid id '{}' (empty, >128 chars, or contains control chars)", idx, id),
                        severity: PepIssueSeverity::Error,
                    });
                    rule_valid = false;
                } else if !seen_ids.insert(id.to_string()) {
                    issues.push(PepValidationIssue {
                        rule_id: Some(id.to_string()),
                        code: PEPRECV_ERR_DUPLICATE_ID.to_string(),
                        message: format!("duplicate rule id '{}' detected at index {}", id, idx),
                        severity: PepIssueSeverity::Error,
                    });
                    rule_valid = false;
                }
            } else {
                issues.push(PepValidationIssue {
                    rule_id: None,
                    code: PEPRECV_ERR_RULE_SYNTAX.to_string(),
                    message: format!("rule index {} is missing string 'id' field", idx),
                    severity: PepIssueSeverity::Error,
                });
                rule_valid = false;
            }

            // Validate deserializability into PepPolicyRule
            match serde_json::from_value::<PepPolicyRule>((*rule_val).clone()) {
                Ok(rule) => {
                    // Check effect
                    if rule.effect != PepDecisionEffect::Permit && rule.effect != PepDecisionEffect::Deny {
                        issues.push(PepValidationIssue {
                            rule_id: rule_id_opt.map(|s| s.to_string()),
                            code: PEPRECV_ERR_RULE_SYNTAX.to_string(),
                            message: format!("rule '{}' has unsupported decision effect '{:?}'", rule.id, rule.effect),
                            severity: PepIssueSeverity::Error,
                        });
                        rule_valid = false;
                    }
                }
                Err(e) => {
                    issues.push(PepValidationIssue {
                        rule_id: rule_id_opt.map(|s| s.to_string()),
                        code: PEPRECV_ERR_RULE_SYNTAX.to_string(),
                        message: format!("rule index {} deserialization failed: {}", idx, e),
                        severity: PepIssueSeverity::Error,
                    });
                    rule_valid = false;
                }
            }

            if rule_valid {
                valid_rules_count += 1;
            } else {
                corrupt_rules_count += 1;
            }
        }

        let is_valid = issues.is_empty();
        PepValidationReport {
            is_valid,
            store_path: store_path.map(|p| p.to_string_lossy().to_string()),
            sha256_checksum: sha256,
            total_rules_scanned: total_rules,
            valid_rules_count,
            corrupt_rules_count,
            issues,
            inspected_at,
        }
    }

    /// Validates a policy store file from disk (PEPRECV1).
    pub fn validate_path(path: &Path) -> Result<PepValidationReport, String> {
        validate_pep_service_path(path)
            .map_err(|e| format!("{}: {}", PEPRECV_ERR_PATH_TRAVERSAL, e))?;

        if !path.exists() {
            return Err(format!("{}: file {:?} does not exist", PEPRECV_ERR_IO, path));
        }

        if let Ok(meta) = fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() {
                return Err(format!("{}: target path {:?} is a symlink", PEPRECV_ERR_IO, path));
            }
            if meta.len() > MAX_PEP_SERVICE_STORE_SIZE {
                return Err(format!("{}: file size {} exceeds limit {}", PEPRECV_ERR_FILE_SIZE, meta.len(), MAX_PEP_SERVICE_STORE_SIZE));
            }
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("{}: failed to read {:?}: {}", PEPRECV_ERR_IO, path, e))?;

        Ok(Self::validate_content(&content, Some(path)))
    }
}

/// Recovery manager implementing non-destructive quarantine and salvage strategies (PEPRECV3, PEPRECV4).
pub struct PepRecoveryManager;

impl PepRecoveryManager {
    /// Non-destructively copies a file to quarantine backup `<path>.bak.<timestamp>` (PEPRECV4).
    pub fn quarantine_file(path: &Path) -> Result<PathBuf, String> {
        if !path.exists() {
            return Err(format!("{}: path does not exist for quarantine", PEPRECV_ERR_IO));
        }
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S_%6f").to_string();
        let file_name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "pep_policy.json".to_string());
        let backup_name = format!("{}.bak.{}", file_name, timestamp);
        let backup_path = path.parent().map(|p| p.join(&backup_name)).unwrap_or_else(|| PathBuf::from(&backup_name));

        fs::copy(path, &backup_path)
            .map_err(|e| format!("{}: quarantine backup copy failed: {}", PEPRECV_ERR_IO, e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&backup_path, fs::Permissions::from_mode(0o600));
        }

        Ok(backup_path)
    }

    /// Recovers a store using the specified recovery strategy (PEPRECV3).
    pub fn recover_store(path: &Path, strategy: PepRecoveryStrategy) -> Result<PepRecoveryResult, String> {
        let original_path_str = path.to_string_lossy().to_string();
        let report = match PepStoreValidator::validate_path(path) {
            Ok(rep) => rep,
            Err(e) => {
                return Ok(PepRecoveryResult {
                    success: false,
                    strategy,
                    original_path: original_path_str,
                    quarantine_path: None,
                    rules_salvaged: 0,
                    rules_dropped: 0,
                    validation_report: PepValidationReport {
                        is_valid: false,
                        store_path: Some(path.to_string_lossy().to_string()),
                        sha256_checksum: None,
                        total_rules_scanned: 0,
                        valid_rules_count: 0,
                        corrupt_rules_count: 0,
                        issues: vec![PepValidationIssue {
                            rule_id: None,
                            code: PEPRECV_ERR_IO.to_string(),
                            message: e,
                            severity: PepIssueSeverity::Error,
                        }],
                        inspected_at: chrono::Utc::now().to_rfc3339(),
                    },
                    message: "Validation failed due to filesystem or path error".to_string(),
                });
            }
        };

        if report.is_valid {
            return Ok(PepRecoveryResult {
                success: true,
                strategy,
                original_path: original_path_str,
                quarantine_path: None,
                rules_salvaged: report.valid_rules_count,
                rules_dropped: 0,
                validation_report: report,
                message: "Store is fully valid; no recovery needed".to_string(),
            });
        }

        match strategy {
            PepRecoveryStrategy::DryRun => {
                Ok(PepRecoveryResult {
                    success: false,
                    strategy,
                    original_path: original_path_str,
                    quarantine_path: None,
                    rules_salvaged: report.valid_rules_count,
                    rules_dropped: report.corrupt_rules_count,
                    validation_report: report,
                    message: "Dry run completed: corruption detected, no changes written to disk".to_string(),
                })
            }
            PepRecoveryStrategy::StrictFailClosed => {
                let q_path = Self::quarantine_file(path)?;
                let fresh = PepDecisionService::new();
                fresh.save_to_path(path)
                    .map_err(|e| format!("{}: failed to initialize fresh store: {}", PEPRECV_ERR_IO, e))?;

                Ok(PepRecoveryResult {
                    success: true,
                    strategy,
                    original_path: original_path_str,
                    quarantine_path: Some(q_path.to_string_lossy().to_string()),
                    rules_salvaged: 0,
                    rules_dropped: report.total_rules_scanned,
                    validation_report: report,
                    message: "Strict fail-closed: corrupted store quarantined, fresh empty store initialized".to_string(),
                })
            }
            PepRecoveryStrategy::SalvageValidRules => {
                let q_path = Self::quarantine_file(path)?;
                let content = fs::read_to_string(path)
                    .map_err(|e| format!("{}: failed to read for salvage: {}", PEPRECV_ERR_IO, e))?;

                let parsed: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| format!("{}: json parse failed: {}", PEPRECV_ERR_PARSE, e))?;

                let mut salvaged_service = PepDecisionService::new();
                let mut salvaged_count = 0;
                let mut dropped_count = 0;

                let rules_to_salvage: Vec<&serde_json::Value> = match parsed.get("rules") {
                    Some(serde_json::Value::Array(arr)) => arr.iter().collect(),
                    Some(serde_json::Value::Object(map)) => map.values().collect(),
                    _ => Vec::new(),
                };

                let mut seen_ids = std::collections::HashSet::new();
                for rule_val in rules_to_salvage {
                    if let Ok(rule) = serde_json::from_value::<PepPolicyRule>((*rule_val).clone()) {
                            if !rule.id.is_empty()
                                && rule.id.len() <= 128
                                && !rule.id.chars().any(|c| c.is_control())
                                && (rule.effect == PepDecisionEffect::Permit || rule.effect == PepDecisionEffect::Deny)
                                && seen_ids.insert(rule.id.clone())
                            {
                                if salvaged_service.add_rule(rule).is_ok() {
                                    salvaged_count += 1;
                                    continue;
                                }
                            }
                        }
                        dropped_count += 1;
                    }

                salvaged_service.save_to_path(path)
                    .map_err(|e| format!("{}: failed to write salvaged store: {}", PEPRECV_ERR_IO, e))?;

                Ok(PepRecoveryResult {
                    success: true,
                    strategy,
                    original_path: original_path_str,
                    quarantine_path: Some(q_path.to_string_lossy().to_string()),
                    rules_salvaged: salvaged_count,
                    rules_dropped: dropped_count,
                    validation_report: report,
                    message: format!("Salvage completed: {} rules salvaged, {} rules dropped", salvaged_count, dropped_count),
                })
            }
        }
    }
}
