//! Privilege Escalation Prevention Store Recovery & Invariant Validation Subsystem (PRIVRECV1..PRIVRECV6).
//!
//! Provides validation and non-destructive recovery mechanisms for privilege store files:
//! - Invariant diagnostics: schema validation, illegal kernel tier detection, grant consistency, and capacity checks.
//! - Non-destructive recovery: point-in-time timestamped backups, quarantine of corrupt JSON,
//!   privilege demotion of corrupted or illegal tiers, and atomic safe store replacement.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::privilege_data_model::{PrivilegeContext, PrivilegeLevel, MAX_ACTOR_ID_LEN};
use crate::privilege_service::PrivilegeService;

pub const PRIVRECV_ERR_IO: &str = "PRIVRECV_ERR_IO";
pub const PRIVRECV_ERR_PARSE: &str = "PRIVRECV_ERR_PARSE";
pub const PRIVRECV_ERR_PATH_TRAVERSAL: &str = "PRIVRECV_ERR_PATH_TRAVERSAL";
pub const PRIVRECV_ERR_FILE_SIZE: &str = "PRIVRECV_ERR_FILE_SIZE";
pub const PRIVRECV_ERR_VALIDATION: &str = "PRIVRECV_ERR_VALIDATION";

/// Maximum allowed file size for a privilege state store (1 MiB).
pub const MAX_PRIVILEGE_STORE_SIZE: u64 = 1024 * 1024;

/// Severity level for a validation diagnostic issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeIssueSeverity {
    Fatal,
    Error,
    Warning,
}

/// Standardized diagnostic issue codes for privilege store validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrivilegeIssueCode {
    InvalidJson,
    SchemaViolation,
    IllegalKernelTier,
    GrantInconsistency,
    InvalidCapability,
    CapacityExceeded,
    CorruptedActorId,
}

/// Diagnostic issue identified during store validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeValidationIssue {
    pub actor_id: Option<String>,
    pub code: PrivilegeIssueCode,
    pub severity: PrivilegeIssueSeverity,
    pub message: String,
}

/// Comprehensive validation report summarizing store health.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeValidationReport {
    pub total_contexts: usize,
    pub healthy_contexts: usize,
    pub issues: Vec<PrivilegeValidationIssue>,
    pub is_valid: bool,
    pub can_auto_repair: bool,
}

/// A specific repair action applied to an actor context during recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeRepairAction {
    pub actor_id: String,
    pub action: String,
    pub reason: String,
}

/// Summary outcome of a store recovery execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeRecoveryResult {
    pub ok: bool,
    pub backup_path: Option<String>,
    pub quarantine_path: Option<String>,
    pub actions: Vec<PrivilegeRepairAction>,
    pub repaired_count: usize,
    pub post_validation: PrivilegeValidationReport,
}

/// Helper container representing raw loaded contexts from disk.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawPrivilegePayload {
    ServiceFormat { contexts: HashMap<String, serde_json::Value> },
    MapFormat(HashMap<String, serde_json::Value>),
    ArrayFormat(Vec<serde_json::Value>),
}

/// Recovery and Invariant Validation Manager for Privilege Escalation Prevention stores.
pub struct PrivilegeRecoveryManager;

impl PrivilegeRecoveryManager {
    /// Validates path hygiene (no traversal, length bounds, no control characters).
    pub fn validate_path(path: &Path) -> Result<(), String> {
        let path_str = path.to_string_lossy();
        if path_str.trim().is_empty() {
            return Err(format!("{}: path cannot be empty", PRIVRECV_ERR_PATH_TRAVERSAL));
        }
        if path_str.contains("..") {
            return Err(format!("{}: path traversal ('..') is prohibited", PRIVRECV_ERR_PATH_TRAVERSAL));
        }
        if path_str.chars().any(|c| c.is_control()) {
            return Err(format!("{}: path contains forbidden control characters", PRIVRECV_ERR_PATH_TRAVERSAL));
        }
        if path_str.len() > 1024 {
            return Err(format!("{}: path exceeds 1024 characters", PRIVRECV_ERR_PATH_TRAVERSAL));
        }
        Ok(())
    }

    /// Evaluates a privilege store file and generates a diagnostic validation report.
    pub fn validate_store_file(path: &Path) -> Result<PrivilegeValidationReport, String> {
        Self::validate_path(path)?;

        if !path.exists() {
            return Ok(PrivilegeValidationReport {
                total_contexts: 0,
                healthy_contexts: 0,
                issues: Vec::new(),
                is_valid: true,
                can_auto_repair: true,
            });
        }

        let meta = fs::symlink_metadata(path).map_err(|e| format!("{}: {}", PRIVRECV_ERR_IO, e))?;
        if meta.file_type().is_symlink() {
            return Err(format!("{}: symbolic links are not permitted for privilege store files", PRIVRECV_ERR_PATH_TRAVERSAL));
        }
        if meta.len() > MAX_PRIVILEGE_STORE_SIZE {
            return Ok(PrivilegeValidationReport {
                total_contexts: 0,
                healthy_contexts: 0,
                issues: vec![PrivilegeValidationIssue {
                    actor_id: None,
                    code: PrivilegeIssueCode::CapacityExceeded,
                    severity: PrivilegeIssueSeverity::Fatal,
                    message: format!("Store file size {} exceeds limit of {} bytes", meta.len(), MAX_PRIVILEGE_STORE_SIZE),
                }],
                is_valid: false,
                can_auto_repair: false,
            });
        }

        let content = fs::read_to_string(path).map_err(|e| format!("{}: {}", PRIVRECV_ERR_IO, e))?;
        Self::validate_raw_json(&content)
    }

    /// Validates raw JSON string content against privilege security invariants.
    pub fn validate_raw_json(content: &str) -> Result<PrivilegeValidationReport, String> {
        let payload: RawPrivilegePayload = match serde_json::from_str(content) {
            Ok(p) => p,
            Err(e) => {
                return Ok(PrivilegeValidationReport {
                    total_contexts: 0,
                    healthy_contexts: 0,
                    issues: vec![PrivilegeValidationIssue {
                        actor_id: None,
                        code: PrivilegeIssueCode::InvalidJson,
                        severity: PrivilegeIssueSeverity::Fatal,
                        message: format!("Malformed JSON payload: {}", e),
                    }],
                    is_valid: false,
                    can_auto_repair: true,
                });
            }
        };

        let raw_contexts: Vec<serde_json::Value> = match payload {
            RawPrivilegePayload::ServiceFormat { contexts } => contexts.into_values().collect(),
            RawPrivilegePayload::MapFormat(map) => map.into_values().collect(),
            RawPrivilegePayload::ArrayFormat(list) => list,
        };

        let total = raw_contexts.len();
        if total > 16384 {
            return Ok(PrivilegeValidationReport {
                total_contexts: total,
                healthy_contexts: 0,
                issues: vec![PrivilegeValidationIssue {
                    actor_id: None,
                    code: PrivilegeIssueCode::CapacityExceeded,
                    severity: PrivilegeIssueSeverity::Fatal,
                    message: format!("Total contexts ({}) exceeds maximum allowable limit (16384)", total),
                }],
                is_valid: false,
                can_auto_repair: false,
            });
        }
        let mut issues = Vec::new();
        let mut healthy = 0;

        for val in raw_contexts {
            let mut context_healthy = true;
            let actor_id = val.get("actor_id").and_then(|v| v.as_str()).unwrap_or("").to_string();

            if actor_id.is_empty() || actor_id.len() > MAX_ACTOR_ID_LEN || actor_id.chars().any(|c| c.is_control()) {
                issues.push(PrivilegeValidationIssue {
                    actor_id: if actor_id.is_empty() { None } else { Some(actor_id.clone()) },
                    code: PrivilegeIssueCode::CorruptedActorId,
                    severity: PrivilegeIssueSeverity::Error,
                    message: "Actor ID is empty, too long, or contains control characters".into(),
                });
                context_healthy = false;
            }

            if let Some(level_str) = val.get("active_level").and_then(|v| v.as_str()) {
                if level_str.to_ascii_lowercase() == "system_kernel" || level_str.to_ascii_lowercase() == "systemkernel" {
                    issues.push(PrivilegeValidationIssue {
                        actor_id: Some(actor_id.clone()),
                        code: PrivilegeIssueCode::IllegalKernelTier,
                        severity: PrivilegeIssueSeverity::Fatal,
                        message: "Actor context illegally set to SystemKernel tier".into(),
                    });
                    context_healthy = false;
                }
            }

            let is_elevated = val.get("is_elevation_active").and_then(|v| v.as_bool()).unwrap_or(false);
            let grant_id = val.get("elevation_grant_id").and_then(|v| v.as_str());

            if is_elevated && grant_id.is_none() {
                issues.push(PrivilegeValidationIssue {
                    actor_id: Some(actor_id.clone()),
                    code: PrivilegeIssueCode::GrantInconsistency,
                    severity: PrivilegeIssueSeverity::Error,
                    message: "Elevation is marked active but elevation_grant_id is missing".into(),
                });
                context_healthy = false;
            }

            if context_healthy {
                healthy += 1;
            }
        }

        let is_valid = issues.is_empty();
        let can_auto_repair = true;

        Ok(PrivilegeValidationReport {
            total_contexts: total,
            healthy_contexts: healthy,
            issues,
            is_valid,
            can_auto_repair,
        })
    }

    /// Repairs and restores a damaged or corrupt privilege store file.
    pub fn repair_store_file(path: &Path) -> Result<PrivilegeRecoveryResult, String> {
        Self::validate_path(path)?;

        if !path.exists() {
            let service = PrivilegeService::new();
            service.save_to_path(path)?;
            let post = Self::validate_store_file(path)?;
            return Ok(PrivilegeRecoveryResult {
                ok: true,
                backup_path: None,
                quarantine_path: None,
                actions: vec![PrivilegeRepairAction {
                    actor_id: "system".into(),
                    action: "initialize".into(),
                    reason: "Store file was missing; created clean empty store".into(),
                }],
                repaired_count: 1,
                post_validation: post,
            });
        }

        let meta = fs::symlink_metadata(path).map_err(|e| format!("{}: {}", PRIVRECV_ERR_IO, e))?;
        if meta.file_type().is_symlink() {
            return Err(format!("{}: symbolic links are not permitted for privilege store files", PRIVRECV_ERR_PATH_TRAVERSAL));
        }
        if meta.len() > MAX_PRIVILEGE_STORE_SIZE {
            return Err(format!("{}: file exceeds max allowed size of 1 MiB", PRIVRECV_ERR_FILE_SIZE));
        }

        // 1. Create timestamped backup
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let backup_path_buf = path.with_extension(format!("bak.{}", timestamp));
        let backup_str = backup_path_buf.to_string_lossy().to_string();
        fs::copy(path, &backup_path_buf).map_err(|e| format!("{}: failed to create backup: {}", PRIVRECV_ERR_IO, e))?;

        let content = fs::read_to_string(path).map_err(|e| format!("{}: {}", PRIVRECV_ERR_IO, e))?;
        let payload_res: Result<RawPrivilegePayload, _> = serde_json::from_str(&content);

        let mut actions = Vec::new();
        let mut quarantine_str = None;

        let raw_contexts: Vec<serde_json::Value> = match payload_res {
            Ok(RawPrivilegePayload::ServiceFormat { contexts }) => contexts.into_values().collect(),
            Ok(RawPrivilegePayload::MapFormat(map)) => map.into_values().collect(),
            Ok(RawPrivilegePayload::ArrayFormat(list)) => list,
            Err(_) => {
                // Corrupted JSON - Quarantine and synthesize clean store
                let q_buf = path.with_extension(format!("quarantine.{}", timestamp));
                let q_s = q_buf.to_string_lossy().to_string();
                fs::copy(path, &q_buf).map_err(|e| format!("{}: failed to quarantine: {}", PRIVRECV_ERR_IO, e))?;
                quarantine_str = Some(q_s);

                let service = PrivilegeService::new();
                service.save_to_path(path)?;
                actions.push(PrivilegeRepairAction {
                    actor_id: "all".into(),
                    action: "quarantine_and_reset".into(),
                    reason: "Store JSON was completely corrupted and unparseable".into(),
                });

                let post = Self::validate_store_file(path)?;
                return Ok(PrivilegeRecoveryResult {
                    ok: true,
                    backup_path: Some(backup_str),
                    quarantine_path: quarantine_str,
                    actions,
                    repaired_count: 1,
                    post_validation: post,
                });
            }
        };

        // 2. Perform field-level sanitization and repair
        if raw_contexts.len() > 16384 {
            return Err(format!("{}: store contains {} contexts, exceeding limit of 16384", PRIVRECV_ERR_VALIDATION, raw_contexts.len()));
        }
        let mut service = PrivilegeService::new();
        for val in raw_contexts {
            let mut actor_id = val.get("actor_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if actor_id.is_empty() || actor_id.chars().any(|c| c.is_control()) {
                actor_id = format!("repaired_actor_{}", actions.len() + 1);
                actions.push(PrivilegeRepairAction {
                    actor_id: actor_id.clone(),
                    action: "sanitize_actor_id".into(),
                    reason: "Corrupted or empty actor identifier sanitized".into(),
                });
            }

            let mut active_level = PrivilegeLevel::User;
            if let Some(level_str) = val.get("active_level").and_then(|v| v.as_str()) {
                if level_str.to_ascii_lowercase() == "system_kernel" || level_str.to_ascii_lowercase() == "systemkernel" {
                    actions.push(PrivilegeRepairAction {
                        actor_id: actor_id.clone(),
                        action: "demote_from_kernel".into(),
                        reason: "Illegal SystemKernel tier demoted to User".into(),
                    });
                } else if let Some(lvl) = PrivilegeLevel::parse_level(level_str) {
                    active_level = lvl;
                }
            }

            if let Ok(mut ctx) = PrivilegeContext::new(&actor_id, active_level) {
                let is_elevated = val.get("is_elevation_active").and_then(|v| v.as_bool()).unwrap_or(false);
                let grant_id = val.get("elevation_grant_id").and_then(|v| v.as_str());

                if is_elevated && grant_id.is_none() {
                    ctx.is_elevation_active = false;
                    ctx.elevation_grant_id = None;
                    actions.push(PrivilegeRepairAction {
                        actor_id: actor_id.clone(),
                        action: "clear_dangling_elevation".into(),
                        reason: "Active elevation without grant ID reset to inactive".into(),
                    });
                } else if is_elevated && grant_id.is_some() {
                    ctx.is_elevation_active = true;
                    ctx.elevation_grant_id = grant_id.map(|s| s.to_string());
                }

                let _ = service.register_context(ctx);
            }
        }

        // 3. Atomically persist repaired service
        service.save_to_path(path)?;
        let post = Self::validate_store_file(path)?;

        Ok(PrivilegeRecoveryResult {
            ok: true,
            backup_path: Some(backup_str),
            quarantine_path: quarantine_str,
            repaired_count: actions.len(),
            actions,
            post_validation: post,
        })
    }
}
