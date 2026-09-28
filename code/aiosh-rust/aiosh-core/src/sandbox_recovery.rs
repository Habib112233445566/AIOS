//! Sandbox Enforcement Recovery & Validation Subsystem (`SANDBOXRECV1..SANDBOXRECV6`).
//!
//! Provides comprehensive validation, diagnostic issue identification,
//! quarantine backups, and state restoration for Sandbox Enforcement.

use std::fs;
use std::path::Path;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::sandbox_data_model::SandboxProfile;
use crate::sandbox_service::SandboxService;

pub const SANDBOXRECV_ERR_IO: &str = "SANDBOXRECV_ERR_IO";
pub const SANDBOXRECV_ERR_TRAVERSAL: &str = "SANDBOXRECV_ERR_TRAVERSAL";
pub const SANDBOXRECV_ERR_CORRUPT: &str = "SANDBOXRECV_ERR_CORRUPT";
pub const SANDBOXRECV_ERR_MISSING_FACTORY: &str = "SANDBOXRECV_ERR_MISSING_FACTORY";
pub const SANDBOXRECV_ERR_LIMIT_BOUNDS: &str = "SANDBOXRECV_ERR_LIMIT_BOUNDS";

/// Maximum size for custom profile manifest files (64 KiB).
pub const MAX_PROFILE_FILE_BYTES: u64 = 64 * 1024;

/// Maximum number of profile files scanned in a custom directory.
pub const MAX_SCANNED_PROFILES: usize = 256;


/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SandboxValidationSeverity {
    Error,
    Warning,
}

/// A specific diagnostic issue identified during validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxValidationIssue {
    pub profile_name: Option<String>,
    pub code: String,
    pub message: String,
    pub severity: SandboxValidationSeverity,
}

/// Comprehensive report generated after validating Sandbox Enforcement profiles and state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxValidationReport {
    pub is_healthy: bool,
    pub factory_profiles_intact: bool,
    pub total_profiles_checked: usize,
    pub valid_profiles_count: usize,
    pub corrupt_profiles_count: usize,
    pub issues: Vec<SandboxValidationIssue>,
    pub timestamp_utc: String,
}

/// Recovery strategy mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SandboxRecoveryStrategy {
    #[default]
    DryRun,
    RestoreFactoryDefaults,
    QuarantineAndReset,
}

/// Result of an executed recovery operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxRecoveryResult {
    pub success: bool,
    pub strategy: SandboxRecoveryStrategy,
    pub quarantine_path: Option<String>,
    pub profiles_restored: usize,
    pub issues_resolved: usize,
    pub message: String,
}

/// Manager orchestrating validation and repair for Sandbox Enforcement.
pub struct SandboxRecoveryManager;

impl SandboxRecoveryManager {
    /// Validates the current state of profiles in the sandbox service and optional custom directory.
    pub fn validate(service: &SandboxService, custom_dir: Option<&Path>) -> SandboxValidationReport {
        let mut issues = Vec::new();
        let profiles = service.list_profiles();

        // 1. Check factory profiles
        let has_standard = service.get_profile("standard").is_some();
        let has_strict = service.get_profile("strict").is_some();
        let has_permissive = service.get_profile("permissive").is_some();

        if !has_standard {
            issues.push(SandboxValidationIssue {
                profile_name: Some("standard".into()),
                code: SANDBOXRECV_ERR_MISSING_FACTORY.into(),
                message: "Factory profile 'standard' is missing".into(),
                severity: SandboxValidationSeverity::Error,
            });
        }
        if !has_strict {
            issues.push(SandboxValidationIssue {
                profile_name: Some("strict".into()),
                code: SANDBOXRECV_ERR_MISSING_FACTORY.into(),
                message: "Factory profile 'strict' is missing".into(),
                severity: SandboxValidationSeverity::Error,
            });
        }
        if !has_permissive {
            issues.push(SandboxValidationIssue {
                profile_name: Some("permissive".into()),
                code: SANDBOXRECV_ERR_MISSING_FACTORY.into(),
                message: "Factory profile 'permissive' is missing".into(),
                severity: SandboxValidationSeverity::Error,
            });
        }

        let factory_intact = has_standard && has_strict && has_permissive;
        let mut valid_count = 0;
        let mut corrupt_count = 0;

        for p in &profiles {
            if let Err(e) = p.validate() {
                corrupt_count += 1;
                issues.push(SandboxValidationIssue {
                    profile_name: Some(p.name.clone()),
                    code: SANDBOXRECV_ERR_LIMIT_BOUNDS.into(),
                    message: format!("Profile validation failed: {}", e),
                    severity: SandboxValidationSeverity::Error,
                });
            } else {
                valid_count += 1;
            }
        }

        // 2. Validate custom directory if provided
        if let Some(dir) = custom_dir {
            if dir.exists() && dir.is_dir() {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten().take(MAX_SCANNED_PROFILES) {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("json") {
                            match Self::validate_profile_file(&path) {
                                Ok(_) => valid_count += 1,
                                Err(file_issues) => {
                                    corrupt_count += 1;
                                    issues.extend(file_issues);
                                }
                            }
                        }
                    }
                }
            }
        }

        let has_errors = issues.iter().any(|i| i.severity == SandboxValidationSeverity::Error);
        SandboxValidationReport {
            is_healthy: factory_intact && !has_errors,
            factory_profiles_intact: factory_intact,
            total_profiles_checked: valid_count + corrupt_count,
            valid_profiles_count: valid_count,
            corrupt_profiles_count: corrupt_count,
            issues,
            timestamp_utc: Utc::now().to_rfc3339(),
        }
    }

    /// Validates an individual profile file on disk.
    pub fn validate_profile_file(path: &Path) -> Result<SandboxProfile, Vec<SandboxValidationIssue>> {
        let mut issues = Vec::new();
        let path_str = path.to_string_lossy().to_string();

        if path_str.contains("..") {
            issues.push(SandboxValidationIssue {
                profile_name: None,
                code: SANDBOXRECV_ERR_TRAVERSAL.into(),
                message: format!("Path traversal forbidden in profile path: {}", path_str),
                severity: SandboxValidationSeverity::Error,
            });
            return Err(issues);
        }

        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                issues.push(SandboxValidationIssue {
                    profile_name: None,
                    code: SANDBOXRECV_ERR_IO.into(),
                    message: format!("Failed to read metadata for {}: {}", path_str, e),
                    severity: SandboxValidationSeverity::Error,
                });
                return Err(issues);
            }
        };

        if meta.len() > MAX_PROFILE_FILE_BYTES {
            issues.push(SandboxValidationIssue {
                profile_name: None,
                code: SANDBOXRECV_ERR_CORRUPT.into(),
                message: format!("Profile file {} exceeds size limit ({} > {})", path_str, meta.len(), MAX_PROFILE_FILE_BYTES),
                severity: SandboxValidationSeverity::Error,
            });
            return Err(issues);
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                issues.push(SandboxValidationIssue {
                    profile_name: None,
                    code: SANDBOXRECV_ERR_IO.into(),
                    message: format!("Failed to read {}: {}", path_str, e),
                    severity: SandboxValidationSeverity::Error,
                });
                return Err(issues);
            }
        };

        let profile: SandboxProfile = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                issues.push(SandboxValidationIssue {
                    profile_name: None,
                    code: SANDBOXRECV_ERR_CORRUPT.into(),
                    message: format!("Malformed JSON in {}: {}", path_str, e),
                    severity: SandboxValidationSeverity::Error,
                });
                return Err(issues);
            }
        };

        if let Err(e) = profile.validate() {
            issues.push(SandboxValidationIssue {
                profile_name: Some(profile.name.clone()),
                code: SANDBOXRECV_ERR_LIMIT_BOUNDS.into(),
                message: format!("Invalid profile limits: {}", e),
                severity: SandboxValidationSeverity::Error,
            });
            return Err(issues);
        }

        Ok(profile)
    }

    /// Executes a recovery strategy.
    pub fn recover(
        service: &mut SandboxService,
        strategy: SandboxRecoveryStrategy,
        custom_dir: Option<&Path>,
    ) -> Result<SandboxRecoveryResult, String> {
        let report = Self::validate(service, custom_dir);

        match strategy {
            SandboxRecoveryStrategy::DryRun => Ok(SandboxRecoveryResult {
                success: report.is_healthy,
                strategy,
                quarantine_path: None,
                profiles_restored: 0,
                issues_resolved: 0,
                message: format!("Dry-run validation complete: {} issues found", report.issues.len()),
            }),
            SandboxRecoveryStrategy::RestoreFactoryDefaults => {
                let default_svc = SandboxService::with_default_profiles(None);
                let mut restored = 0;
                for p in default_svc.list_profiles() {
                    let _ = service.upsert_profile(p);
                    restored += 1;
                }
                Ok(SandboxRecoveryResult {
                    success: true,
                    strategy,
                    quarantine_path: None,
                    profiles_restored: restored,
                    issues_resolved: report.issues.len(),
                    message: "Factory default profiles successfully re-registered".into(),
                })
            }
            SandboxRecoveryStrategy::QuarantineAndReset => {
                let mut q_path_str = None;
                if let Some(dir) = custom_dir {
                    if dir.exists() && dir.is_dir() {
                        let q_dir = dir.join(format!(".quarantine_{}", Utc::now().timestamp()));
                        if let Ok(()) = fs::create_dir_all(&q_dir) {
                            if let Ok(entries) = fs::read_dir(dir) {
                                for entry in entries.flatten() {
                                    let p = entry.path();
                                    if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("json") {
                                        if Self::validate_profile_file(&p).is_err() {
                                            if let Some(fname) = p.file_name() {
                                                let target = q_dir.join(fname);
                                                let _ = fs::rename(&p, target);
                                            }
                                        }
                                    }
                                }
                            }
                            q_path_str = Some(q_dir.to_string_lossy().to_string());
                        }
                    }
                }

                // Restore factory defaults
                let default_svc = SandboxService::with_default_profiles(None);
                let mut restored = 0;
                for p in default_svc.list_profiles() {
                    let _ = service.upsert_profile(p);
                    restored += 1;
                }

                Ok(SandboxRecoveryResult {
                    success: true,
                    strategy,
                    quarantine_path: q_path_str,
                    profiles_restored: restored,
                    issues_resolved: report.issues.len(),
                    message: "Corrupt manifests quarantined and factory defaults restored".into(),
                })
            }
        }
    }
}
