//! PEP Grant Store Recovery & Invariant Validation Subsystem (T-02291..T-02300).
//!
//! Provides two-tier validation and recovery mechanisms for PEP authorization grant stores:
//! - Structural and invariant validation: DAG cycle detection, orphan detection, depth consistency,
//!   attenuation containment, and cascade revocation synchronization.
//! - Non-destructive recovery: atomic point-in-time snapshots, quarantine of corrupt/unparseable fragments,
//!   orphan revocation, cascade synchronization, and safe store atomic replacement.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::capability::CapabilityRight;
use crate::pep_grant::{PepGrant, PepGrantState};
use crate::pep_grant_service::{
    PepGrantService, validate_grant_service_path, MAX_GRANT_SERVICE_STORE_SIZE,
};

pub const PEPGRANTRECV_ERR_IO: &str = "PEPGRANTRECV_ERR_IO";
pub const PEPGRANTRECV_ERR_PARSE: &str = "PEPGRANTRECV_ERR_PARSE";
pub const PEPGRANTRECV_ERR_PATH_TRAVERSAL: &str = "PEPGRANTRECV_ERR_PATH_TRAVERSAL";
pub const PEPGRANTRECV_ERR_FILE_SIZE: &str = "PEPGRANTRECV_ERR_FILE_SIZE";
pub const PEPGRANTRECV_ERR_VALIDATION: &str = "PEPGRANTRECV_ERR_VALIDATION";
pub const PEPGRANTRECV_ERR_CORRUPT: &str = "PEPGRANTRECV_ERR_CORRUPT";

/// Severity level for a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepGrantIssueSeverity {
    Fatal,
    Error,
    Warning,
}

/// Standardized diagnostic issue codes for grant store validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PepGrantIssueCode {
    InvalidJson,
    SchemaViolation,
    CycleDetected,
    OrphanGrant,
    DepthInconsistency,
    AttenuationViolation,
    CascadeDesync,
    TimeInversion,
    ExpiredActive,
}

/// Diagnostic issue identified during validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantValidationIssue {
    pub grant_id: Option<String>,
    pub code: PepGrantIssueCode,
    pub severity: PepGrantIssueSeverity,
    pub message: String,
}

/// Summary report returned by validation routines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantValidationReport {
    pub total_grants: usize,
    pub healthy_grants: usize,
    pub issues: Vec<PepGrantValidationIssue>,
    pub is_valid: bool,
    pub can_auto_repair: bool,
}

/// A specific action applied to an individual grant during store recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantRepairAction {
    pub grant_id: String,
    pub action: String,
    pub reason: String,
}

/// Outcome of a store recovery and repair execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantRecoveryResult {
    pub ok: bool,
    pub backup_path: Option<String>,
    pub quarantine_path: Option<String>,
    pub actions: Vec<PepGrantRepairAction>,
    pub repaired_count: usize,
    pub post_validation: PepGrantValidationReport,
}

/// Helper container representing raw loaded grants before service instantiation.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawGrantPayload {
    ServiceFormat { grants: HashMap<String, PepGrant> },
    ArrayFormat(Vec<PepGrant>),
}

/// Grant Recovery Manager implementing validation and salvage operations.
pub struct PepGrantRecoveryManager;

impl PepGrantRecoveryManager {
    /// Validates an in-memory slice of grants against all core hierarchy invariants.
    pub fn validate_grants(grants: &[PepGrant]) -> PepGrantValidationReport {
        let mut issues = Vec::new();
        let grant_map: HashMap<&str, &PepGrant> = grants
            .iter()
            .map(|g| (g.id.as_str(), g))
            .collect();

        let now = Utc::now();

        // 1. Per-grant temporal and self-consistency validation
        for grant in grants {
            if let (Some(nb_str), Some(exp_str)) = (
                grant.constraints.not_before.as_deref(),
                grant.constraints.expires_at.as_deref(),
            ) {
                if let (Ok(nb), Ok(exp)) = (
                    chrono::DateTime::parse_from_rfc3339(nb_str),
                    chrono::DateTime::parse_from_rfc3339(exp_str),
                ) {
                    if exp < nb {
                        issues.push(PepGrantValidationIssue {
                            grant_id: Some(grant.id.clone()),
                            code: PepGrantIssueCode::TimeInversion,
                            severity: PepGrantIssueSeverity::Error,
                            message: format!(
                                "grant {} expires_at ({}) precedes not_before ({})",
                                grant.id, exp_str, nb_str
                            ),
                        });
                    }
                }
            }

            if grant.state == PepGrantState::Active {
                if let Some(exp_str) = grant.constraints.expires_at.as_deref() {
                    if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(exp_str) {
                        if exp < now {
                            issues.push(PepGrantValidationIssue {
                                grant_id: Some(grant.id.clone()),
                                code: PepGrantIssueCode::ExpiredActive,
                                severity: PepGrantIssueSeverity::Warning,
                                message: format!(
                                    "grant {} has expired (expires_at={}) but remains Active",
                                    grant.id, exp_str
                                ),
                            });
                        }
                    }
                }
            }
        }

        // 2. Parent-child relationship, attenuation, and cycle detection
        for grant in grants {
            if let Some(parent_id) = &grant.parent_grant_id {
                // Check orphan
                match grant_map.get(parent_id.as_str()) {
                    None => {
                        let severity = if grant.state == PepGrantState::Revoked {
                            PepGrantIssueSeverity::Warning
                        } else {
                            PepGrantIssueSeverity::Error
                        };
                        issues.push(PepGrantValidationIssue {
                            grant_id: Some(grant.id.clone()),
                            code: PepGrantIssueCode::OrphanGrant,
                            severity,
                            message: format!(
                                "grant {} references non-existent parent '{}'",
                                grant.id, parent_id
                            ),
                        });
                    }
                    Some(parent) => {
                        if grant.state != PepGrantState::Revoked {
                            // Depth consistency
                            if grant.constraints.max_delegation_depth >= parent.constraints.max_delegation_depth && parent.constraints.max_delegation_depth > 0 {
                                issues.push(PepGrantValidationIssue {
                                    grant_id: Some(grant.id.clone()),
                                    code: PepGrantIssueCode::DepthInconsistency,
                                    severity: PepGrantIssueSeverity::Error,
                                    message: format!(
                                        "grant {} max_delegation_depth {} must be strictly less than parent {} depth {}",
                                        grant.id, grant.constraints.max_delegation_depth, parent.id, parent.constraints.max_delegation_depth
                                    ),
                                });
                            }

                            // Attenuation checks: parent must confer Delegate right
                            if !parent.rights.contains(&CapabilityRight::Delegate) {
                                issues.push(PepGrantValidationIssue {
                                    grant_id: Some(grant.id.clone()),
                                    code: PepGrantIssueCode::AttenuationViolation,
                                    severity: PepGrantIssueSeverity::Error,
                                    message: format!(
                                        "parent grant {} does not confer 'delegate' right",
                                        parent.id
                                    ),
                                });
                            }

                            // Attenuation checks: child rights must be a subset of parent rights
                            for r in &grant.rights {
                                if !parent.rights.contains(r) {
                                    issues.push(PepGrantValidationIssue {
                                        grant_id: Some(grant.id.clone()),
                                        code: PepGrantIssueCode::AttenuationViolation,
                                        severity: PepGrantIssueSeverity::Error,
                                        message: format!(
                                            "grant {} right '{}' exceeds parent {} rights",
                                            grant.id, r, parent.id
                                        ),
                                    });
                                }
                            }
                        }

                        // Cascade sync: if parent is Revoked, child must also be Revoked
                        if parent.state == PepGrantState::Revoked && grant.state != PepGrantState::Revoked {
                            issues.push(PepGrantValidationIssue {
                                grant_id: Some(grant.id.clone()),
                                code: PepGrantIssueCode::CascadeDesync,
                                severity: PepGrantIssueSeverity::Error,
                                message: format!(
                                    "parent {} is Revoked, but child {} is still {:?}",
                                    parent.id, grant.id, grant.state
                                ),
                            });
                        }
                    }
                }

                // Cycle detection for this node
                let mut visited = HashSet::new();
                visited.insert(grant.id.as_str());
                let mut curr_parent = grant.parent_grant_id.as_deref();
                while let Some(pid) = curr_parent {
                    if visited.contains(pid) {
                        issues.push(PepGrantValidationIssue {
                            grant_id: Some(grant.id.clone()),
                            code: PepGrantIssueCode::CycleDetected,
                            severity: PepGrantIssueSeverity::Error,
                            message: format!(
                                "cycle detected in delegation graph involving grant '{}' and ancestor '{}'",
                                grant.id, pid
                            ),
                        });
                        break;
                    }
                    visited.insert(pid);
                    curr_parent = grant_map.get(pid).and_then(|p| p.parent_grant_id.as_deref());
                }
            }
        }

        let has_fatal_or_error = issues.iter().any(|i| {
            i.severity == PepGrantIssueSeverity::Fatal || i.severity == PepGrantIssueSeverity::Error
        });
        let has_cycle = issues.iter().any(|i| i.code == PepGrantIssueCode::CycleDetected);

        let healthy_grants = if has_fatal_or_error {
            let issue_grant_ids: HashSet<&str> = issues
                .iter()
                .filter(|i| i.severity != PepGrantIssueSeverity::Warning)
                .filter_map(|i| i.grant_id.as_deref())
                .collect();
            grants.iter().filter(|g| !issue_grant_ids.contains(g.id.as_str())).count()
        } else {
            grants.len()
        };

        PepGrantValidationReport {
            total_grants: grants.len(),
            healthy_grants,
            is_valid: !has_fatal_or_error,
            can_auto_repair: !has_cycle,
            issues,
        }
    }

    /// Validates a grant store file on disk.
    pub fn validate_store_file<P: AsRef<Path>>(path: P) -> Result<PepGrantValidationReport, String> {
        let p = path.as_ref();
        validate_grant_service_path(p)?;

        if !p.exists() {
            return Ok(PepGrantValidationReport {
                total_grants: 0,
                healthy_grants: 0,
                issues: Vec::new(),
                is_valid: true,
                can_auto_repair: true,
            });
        }

        let metadata = fs::metadata(p).map_err(|e| format!("{}: {}", PEPGRANTRECV_ERR_IO, e))?;
        if metadata.len() > MAX_GRANT_SERVICE_STORE_SIZE {
            return Ok(PepGrantValidationReport {
                total_grants: 0,
                healthy_grants: 0,
                issues: vec![PepGrantValidationIssue {
                    grant_id: None,
                    code: PepGrantIssueCode::InvalidJson,
                    severity: PepGrantIssueSeverity::Fatal,
                    message: format!(
                        "store file size {} bytes exceeds maximum {} bytes",
                        metadata.len(),
                        MAX_GRANT_SERVICE_STORE_SIZE
                    ),
                }],
                is_valid: false,
                can_auto_repair: false,
            });
        }

        let content = fs::read_to_string(p).map_err(|e| format!("{}: {}", PEPGRANTRECV_ERR_IO, e))?;
        if content.trim().is_empty() {
            return Ok(PepGrantValidationReport {
                total_grants: 0,
                healthy_grants: 0,
                issues: Vec::new(),
                is_valid: true,
                can_auto_repair: true,
            });
        }

        let grants: Vec<PepGrant> = match serde_json::from_str::<RawGrantPayload>(&content) {
            Ok(RawGrantPayload::ServiceFormat { grants }) => grants.into_values().collect(),
            Ok(RawGrantPayload::ArrayFormat(vec)) => vec,
            Err(e) => {
                return Ok(PepGrantValidationReport {
                    total_grants: 0,
                    healthy_grants: 0,
                    issues: vec![PepGrantValidationIssue {
                        grant_id: None,
                        code: PepGrantIssueCode::InvalidJson,
                        severity: PepGrantIssueSeverity::Fatal,
                        message: format!("corrupt JSON syntax: {}", e),
                    }],
                    is_valid: false,
                    can_auto_repair: false,
                });
            }
        };

        Ok(Self::validate_grants(&grants))
    }

    /// Repairs an existing grant store file by applying safe auto-reconciliation and isolation.
    pub fn recover_store_file<P: AsRef<Path>>(path: P, dry_run: bool) -> Result<PepGrantRecoveryResult, String> {
        let p = path.as_ref();
        validate_grant_service_path(p)?;

        if !p.exists() {
            return Ok(PepGrantRecoveryResult {
                ok: true,
                backup_path: None,
                quarantine_path: None,
                actions: Vec::new(),
                repaired_count: 0,
                post_validation: PepGrantValidationReport {
                    total_grants: 0,
                    healthy_grants: 0,
                    issues: Vec::new(),
                    is_valid: true,
                    can_auto_repair: true,
                },
            });
        }

        let content = fs::read_to_string(p).map_err(|e| format!("{}: {}", PEPGRANTRECV_ERR_IO, e))?;
        let now = Utc::now();
        let now_ts = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(_) => 0,
        };

        let mut backup_path_str = None;
        let mut quarantine_path_str = None;

        let parse_result = serde_json::from_str::<RawGrantPayload>(&content);

        let mut grants: HashMap<String, PepGrant> = match parse_result {
            Ok(RawGrantPayload::ServiceFormat { grants }) => grants,
            Ok(RawGrantPayload::ArrayFormat(vec)) => {
                vec.into_iter().map(|g| (g.id.clone(), g)).collect()
            }
            Err(e) => {
                // If unparseable JSON, quarantine corrupted file and initialize clean store
                if !dry_run {
                    let quarantine_path = p.with_extension(format!("quarantine.{}.json", now_ts));
                    fs::write(&quarantine_path, &content)
                        .map_err(|ioe| format!("{}: {}", PEPGRANTRECV_ERR_IO, ioe))?;
                    quarantine_path_str = Some(quarantine_path.to_string_lossy().to_string());

                    // Overwrite with empty service structure
                    let empty_service = PepGrantService::new();
                    let serialized = serde_json::to_string_pretty(&empty_service)
                        .map_err(|se| format!("{}: {}", PEPGRANTRECV_ERR_PARSE, se))?;
                    fs::write(p, serialized).map_err(|ioe| format!("{}: {}", PEPGRANTRECV_ERR_IO, ioe))?;
                }
                return Ok(PepGrantRecoveryResult {
                    ok: true,
                    backup_path: None,
                    quarantine_path: quarantine_path_str,
                    actions: vec![PepGrantRepairAction {
                        grant_id: "STORE_ROOT".to_string(),
                        action: "quarantined_corrupt_store".to_string(),
                        reason: format!("Unparseable JSON: {}", e),
                    }],
                    repaired_count: 1,
                    post_validation: PepGrantValidationReport {
                        total_grants: 0,
                        healthy_grants: 0,
                        issues: Vec::new(),
                        is_valid: true,
                        can_auto_repair: true,
                    },
                });
            }
        };

        // Create atomic backup before any modification
        if !dry_run {
            let backup_path = p.with_extension(format!("bak.{}", now_ts));
            fs::write(&backup_path, &content).map_err(|e| format!("{}: {}", PEPGRANTRECV_ERR_IO, e))?;
            backup_path_str = Some(backup_path.to_string_lossy().to_string());
        }

        let mut actions = Vec::new();

        // 1. Existing IDs set
        let existing_ids: HashSet<String> = grants.keys().cloned().collect();

        // 2. Fix Orphans: if parent does not exist, revoke orphan grant
        for grant in grants.values_mut() {
            if let Some(parent_id) = &grant.parent_grant_id {
                if !existing_ids.contains(parent_id) && grant.state != PepGrantState::Revoked {
                    grant.state = PepGrantState::Revoked;
                    actions.push(PepGrantRepairAction {
                        grant_id: grant.id.clone(),
                        action: "revoked_orphan".to_string(),
                        reason: format!("Parent grant '{}' missing from store", parent_id),
                    });
                }
            }
        }

        // 3. Fix Cascade Desync: propagate parent revocation down to children
        let mut cascade_changed = true;
        while cascade_changed {
            cascade_changed = false;
            let revoked_ids: HashSet<String> = grants
                .values()
                .filter(|g| g.state == PepGrantState::Revoked)
                .map(|g| g.id.clone())
                .collect();

            for grant in grants.values_mut() {
                if let Some(pid) = &grant.parent_grant_id {
                    if revoked_ids.contains(pid) && grant.state != PepGrantState::Revoked {
                        grant.state = PepGrantState::Revoked;
                        cascade_changed = true;
                        actions.push(PepGrantRepairAction {
                            grant_id: grant.id.clone(),
                            action: "reconciled_cascade".to_string(),
                            reason: format!("Parent grant '{}' is Revoked", pid),
                        });
                    }
                }
            }
        }

        // 4. Auto-expire elapsed grants
        for grant in grants.values_mut() {
            if grant.state == PepGrantState::Active {
                if let Some(exp_str) = grant.constraints.expires_at.as_deref() {
                    if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(exp_str) {
                        if exp < now {
                            grant.state = PepGrantState::Expired;
                            actions.push(PepGrantRepairAction {
                                grant_id: grant.id.clone(),
                                action: "auto_expired".to_string(),
                                reason: format!("Expiration {} has passed", exp_str),
                            });
                        }
                    }
                }
            }
        }

        // 5. If not dry_run and changes were made, write atomically to disk
        let repaired_count = actions.len();
        if !dry_run && repaired_count > 0 {
            let grant_vec: Vec<PepGrant> = grants.values().cloned().collect();
            let service = PepGrantService::from_grants(grant_vec);
            service.save_to_path(p)?;
        }

        let grant_list: Vec<PepGrant> = grants.into_values().collect();
        let post_validation = Self::validate_grants(&grant_list);

        Ok(PepGrantRecoveryResult {
            ok: true,
            backup_path: backup_path_str,
            quarantine_path: quarantine_path_str,
            actions,
            repaired_count,
            post_validation,
        })
    }
}
