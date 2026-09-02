use std::path::Path;
use std::process::Command;
use std::time::Instant;
use crate::repo_health::{HealthCategory, HealthStatus, RepoHealthCheck, RepoHealthReport};
use crate::repo_health_config::RepoHealthConfig;

/// Default file size limit: 16 MiB (16,777,216 bytes)
pub const DEFAULT_MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// Check git working tree cleanliness via `git status --porcelain=v2`
pub fn check_git_working_tree(repo_root: &Path) -> RepoHealthCheck {
    let start = Instant::now();
    let mut check = RepoHealthCheck {
        check_id: "git_working_tree".into(),
        name: "Git Working Tree Cleanliness".into(),
        category: HealthCategory::GitHygiene,
        status: HealthStatus::Pass,
        message: "Working tree clean".into(),
        details: None,
        duration_ms: 0,
    };

    let git_dir = repo_root.join(".git");
    if !git_dir.exists() {
        check.status = HealthStatus::Warn;
        check.message = "No .git directory found at repository root".into();
        check.duration_ms = start.elapsed().as_millis() as u64;
        return check;
    }

    let output_res = Command::new("git")
        .args(["status", "--porcelain=v2"])
        .current_dir(repo_root)
        .output();

    match output_res {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut changes = Vec::new();
                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    changes.push(trimmed.to_string());
                }

                if changes.is_empty() {
                    check.status = HealthStatus::Pass;
                    check.message = "Working tree is clean".into();
                } else {
                    check.status = HealthStatus::Warn;
                    check.message = format!("Working tree contains {} uncommitted or untracked change(s)", changes.len());
                    check.details = Some(changes.into_iter().take(50).collect());
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                check.status = HealthStatus::Warn;
                check.message = format!("git status exited with non-zero code: {}", stderr.trim());
            }
        }
        Err(e) => {
            check.status = HealthStatus::Warn;
            check.message = format!("Failed to execute git status: {}", e);
        }
    }

    check.duration_ms = start.elapsed().as_millis() as u64;
    check
}

/// Recursive directory walker helper for size checking
fn scan_directory_file_sizes(dir: &Path, max_bytes: u64, oversized: &mut Vec<String>) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if path.is_dir() {
            if file_name_str == ".git" || file_name_str == "target" || file_name_str == "node_modules" || file_name_str == ".venv" {
                continue;
            }
            scan_directory_file_sizes(&path, max_bytes, oversized)?;
        } else if path.is_file() {
            if let Ok(meta) = entry.metadata() {
                if meta.len() > max_bytes {
                    oversized.push(format!("{}: {} bytes (limit: {} bytes)", path.display(), meta.len(), max_bytes));
                }
            }
        }
    }

    Ok(())
}

/// Check file size bounds across repository workspace
pub fn check_file_bounds(repo_root: &Path, max_bytes: u64) -> RepoHealthCheck {
    let start = Instant::now();
    let mut check = RepoHealthCheck {
        check_id: "file_bounds".into(),
        name: "File Size Bounds Integrity".into(),
        category: HealthCategory::FileIntegrity,
        status: HealthStatus::Pass,
        message: format!("All files within size limit ({} bytes)", max_bytes),
        details: None,
        duration_ms: 0,
    };

    let mut oversized = Vec::new();
    match scan_directory_file_sizes(repo_root, max_bytes, &mut oversized) {
        Ok(()) => {
            if oversized.is_empty() {
                check.status = HealthStatus::Pass;
                check.message = format!("All monitored files are within size limit ({} bytes)", max_bytes);
            } else {
                check.status = HealthStatus::Fail;
                check.message = format!("Found {} file(s) exceeding size limit of {} bytes", oversized.len(), max_bytes);
                check.details = Some(oversized.into_iter().take(50).collect());
            }
        }
        Err(e) => {
            check.status = HealthStatus::Warn;
            check.message = format!("Failed to complete file size scan: {}", e);
        }
    }

    check.duration_ms = start.elapsed().as_millis() as u64;
    check
}

/// Check repository security governance policy
pub fn check_security_governance(repo_root: &Path) -> RepoHealthCheck {
    let start = Instant::now();
    let mut check = RepoHealthCheck {
        check_id: "security_governance".into(),
        name: "Security Governance Policy Verification".into(),
        category: HealthCategory::SecurityGovernance,
        status: HealthStatus::Pass,
        message: "SECURITY.md policy verified".into(),
        details: None,
        duration_ms: 0,
    };

    let sec_path = repo_root.join("SECURITY.md");
    if !sec_path.exists() {
        check.status = HealthStatus::Fail;
        check.message = "Root SECURITY.md policy document is missing".into();
        check.duration_ms = start.elapsed().as_millis() as u64;
        return check;
    }

    match std::fs::read_to_string(&sec_path) {
        Ok(content) => {
            if content.trim().len() < 100 {
                check.status = HealthStatus::Fail;
                check.message = "SECURITY.md is too short (< 100 characters)".into();
            } else if content.contains("TODO") {
                check.status = HealthStatus::Fail;
                check.message = "SECURITY.md contains uncompleted TODO markers".into();
            } else {
                check.status = HealthStatus::Pass;
                check.message = "SECURITY.md exists and meets governance policy requirements".into();
            }
        }
        Err(e) => {
            check.status = HealthStatus::Fail;
            check.message = format!("Failed to read SECURITY.md: {}", e);
        }
    }

    check.duration_ms = start.elapsed().as_millis() as u64;
    check
}

/// Orchestrate all repository health checks and return aggregate report
pub fn check_repo_health(repo_root: &Path) -> Result<RepoHealthReport, String> {
    let repo_path_str = repo_root.to_string_lossy().to_string();
    let timestamp_utc = chrono::Utc::now().to_rfc3339();

    let checks = vec![
        check_git_working_tree(repo_root),
        check_file_bounds(repo_root, DEFAULT_MAX_FILE_BYTES),
        check_security_governance(repo_root),
    ];

    RepoHealthReport::new(repo_path_str, timestamp_utc, checks)
}

/// Formats a human-readable text summary of a RepoHealthReport (T-00693/T-00694).
pub fn format_repo_health_summary(report: &RepoHealthReport) -> String {
    let mut out = format!("AIOS Repository Health Report ([{}]):\n", report.overall_status.as_str());
    out.push_str(&format!("  Repository Root: {}\n", report.repo_path));
    out.push_str(&format!("  Timestamp (UTC): {}\n", report.timestamp_utc));
    if report.checks.is_empty() {
        out.push_str("  (no health checks performed)\n");
    } else {
        for check in &report.checks {
            out.push_str(&format!(
                "  [{}] {} ({}) - {} ({}ms)\n",
                check.status.as_str(),
                check.name,
                check.check_id,
                check.message,
                check.duration_ms
            ));
            if let Some(ref details) = check.details {
                for detail in details.iter().take(50) {
                    out.push_str(&format!("    - {}\n", detail));
                }
                if details.len() > 50 {
                    out.push_str(&format!("    ... ({} additional items truncated)\n", details.len() - 50));
                }
            }
        }
    }
    out.push_str(&format!(
        "  Summary: {} total ({} pass, {} warn, {} fail, {} skip)",
        report.total_checks,
        report.passed_checks,
        report.warn_checks,
        report.failed_checks,
        report.skipped_checks
    ));
    out
}

/// Recovers the canonical default RepoHealthConfig in memory (T-00703/T-00704).
pub fn recover_default_repo_health_config() -> RepoHealthConfig {
    RepoHealthConfig::default()
}

/// Reconstructs a full RepoHealthReport using specified or recovered configuration (T-00703/T-00704).
pub fn reconstruct_repo_health_report(
    repo_root: &Path,
    config: &RepoHealthConfig,
) -> Result<RepoHealthReport, String> {
    if !repo_root.exists() {
        return Err(format!("Repository root does not exist: {}", repo_root.display()));
    }
    let repo_path_str = repo_root.to_string_lossy().to_string();
    let timestamp_utc = chrono::Utc::now().to_rfc3339();

    let checks = vec![
        check_git_working_tree(repo_root),
        check_file_bounds(repo_root, config.max_file_bytes),
        check_security_governance(repo_root),
    ];

    RepoHealthReport::new(repo_path_str, timestamp_utc, checks)
}

/// Validates the structural and mathematical integrity of a RepoHealthReport (T-00703/T-00704).
pub fn validate_repo_health_report(report: &RepoHealthReport) -> Result<(), String> {
    report.validate()
}

/// Fully reconciles repository health with automatic fallback and validation (T-00703/T-00704).
pub fn reconcile_repo_health(repo_root: &Path) -> Result<RepoHealthReport, String> {
    let config = RepoHealthConfig::from_env().unwrap_or_else(|_| recover_default_repo_health_config());
    let report = reconstruct_repo_health_report(repo_root, &config)?;
    validate_repo_health_report(&report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_check_file_bounds_happy() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut f = File::create(file_path).unwrap();
        f.write_all(b"Hello world").unwrap();
        drop(f);

        let check = check_file_bounds(dir.path(), 1024);
        assert_eq!(check.status, HealthStatus::Pass);
        assert_eq!(check.category, HealthCategory::FileIntegrity);
        assert!(check.details.is_none());
    }

    #[test]
    fn test_check_file_bounds_oversized() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.bin");
        let mut f = File::create(file_path).unwrap();
        let data = vec![0u8; 2048];
        f.write_all(&data).unwrap();
        drop(f);

        let check = check_file_bounds(dir.path(), 1024);
        assert_eq!(check.status, HealthStatus::Fail);
        assert!(check.details.is_some());
        assert_eq!(check.details.unwrap().len(), 1);
    }

    #[test]
    fn test_check_security_governance_missing() {
        let dir = tempdir().unwrap();
        let check = check_security_governance(dir.path());
        assert_eq!(check.status, HealthStatus::Fail);
        assert!(check.message.contains("missing"));
    }

    #[test]
    fn test_check_security_governance_valid() {
        let dir = tempdir().unwrap();
        let sec_path = dir.path().join("SECURITY.md");
        let mut f = File::create(sec_path).unwrap();
        f.write_all(b"# Security Policy\n\nWe take security seriously. Please report vulnerabilities to security@example.com.\nAll disclosures are acknowledged within 24 hours and addressed within 90 days.\n").unwrap();
        drop(f);

        let check = check_security_governance(dir.path());
        assert_eq!(check.status, HealthStatus::Pass);
    }

    #[test]
    fn test_check_security_governance_todo_markers() {
        let dir = tempdir().unwrap();
        let sec_path = dir.path().join("SECURITY.md");
        let mut f = File::create(sec_path).unwrap();
        f.write_all(b"# Security Policy\n\nTODO: add contact details here.\nPlease report vulnerabilities.\nThis is a long policy text for tests.\n").unwrap();
        drop(f);

        let check = check_security_governance(dir.path());
        assert_eq!(check.status, HealthStatus::Fail);
        assert!(check.message.contains("TODO markers"));
    }

    #[test]
    fn test_check_repo_health_orchestrator() {
        let dir = tempdir().unwrap();
        let sec_path = dir.path().join("SECURITY.md");
        let mut f = File::create(sec_path).unwrap();
        f.write_all(b"# Security Policy\n\nWe take security seriously. Please report vulnerabilities to security@example.com.\nAll disclosures are acknowledged within 24 hours.\n").unwrap();
        drop(f);

        let report = check_repo_health(dir.path()).unwrap();
        assert_eq!(report.total_checks, 3);
        assert!(report.passed_checks >= 2);
    }

    #[test]
    fn test_format_repo_health_summary() {
        let checks = vec![
            RepoHealthCheck {
                check_id: "git_working_tree".into(),
                name: "Git Working Tree Cleanliness".into(),
                category: HealthCategory::GitHygiene,
                status: HealthStatus::Pass,
                message: "Working tree clean".into(),
                details: None,
                duration_ms: 12,
            },
            RepoHealthCheck {
                check_id: "file_bounds".into(),
                name: "File Size Bounds Integrity".into(),
                category: HealthCategory::FileIntegrity,
                status: HealthStatus::Warn,
                message: "Found oversized file".into(),
                details: Some(vec!["oversized.bin: 20000000 bytes".into()]),
                duration_ms: 45,
            },
        ];

        let report = RepoHealthReport::new(
            "/workspace/aios".into(),
            "2026-08-30T12:00:00Z".into(),
            checks,
        ).unwrap();

        let formatted = format_repo_health_summary(&report);
        assert!(formatted.contains("AIOS Repository Health Report ([Warn]):"));
        assert!(formatted.contains("Repository Root: /workspace/aios"));
        assert!(formatted.contains("Timestamp (UTC): 2026-08-30T12:00:00Z"));
        assert!(formatted.contains("[Pass] Git Working Tree Cleanliness (git_working_tree) - Working tree clean (12ms)"));
        assert!(formatted.contains("[Warn] File Size Bounds Integrity (file_bounds) - Found oversized file (45ms)"));
        assert!(formatted.contains("- oversized.bin: 20000000 bytes"));
        assert!(formatted.contains("Summary: 2 total (1 pass, 1 warn, 0 fail, 0 skip)"));
    }

    #[test]
    fn test_format_repo_health_summary_empty_and_fail() {
        let empty_report = RepoHealthReport::new(
            "/repo".into(),
            "2026-08-30T00:00:00Z".into(),
            vec![],
        ).unwrap();
        let formatted_empty = format_repo_health_summary(&empty_report);
        assert!(formatted_empty.contains("AIOS Repository Health Report ([Pass]):"));
        assert!(formatted_empty.contains("(no health checks performed)"));
        assert!(formatted_empty.contains("Summary: 0 total (0 pass, 0 warn, 0 fail, 0 skip)"));

        let fail_report = RepoHealthReport::new(
            "/repo".into(),
            "2026-08-30T00:00:00Z".into(),
            vec![
                RepoHealthCheck {
                    check_id: "sec_gov".into(),
                    name: "Security Governance".into(),
                    category: HealthCategory::SecurityGovernance,
                    status: HealthStatus::Fail,
                    message: "SECURITY.md missing".into(),
                    details: None,
                    duration_ms: 1,
                },
                RepoHealthCheck {
                    check_id: "bounds".into(),
                    name: "Bounds".into(),
                    category: HealthCategory::WorkspaceBounds,
                    status: HealthStatus::Skip,
                    message: "Skipped".into(),
                    details: None,
                    duration_ms: 0,
                },
            ],
        ).unwrap();
        let formatted_fail = format_repo_health_summary(&fail_report);
        assert!(formatted_fail.contains("AIOS Repository Health Report ([Fail]):"));
        assert!(formatted_fail.contains("[Fail] Security Governance (sec_gov) - SECURITY.md missing (1ms)"));
        assert!(formatted_fail.contains("[Skip] Bounds (bounds) - Skipped (0ms)"));
        assert!(formatted_fail.contains("Summary: 2 total (0 pass, 0 warn, 1 fail, 1 skip)"));
    }

    #[test]
    fn test_format_repo_health_summary_truncation() {
        let details: Vec<String> = (0..60).map(|i| format!("item {}", i)).collect();
        let report = RepoHealthReport::new(
            "/repo".into(),
            "2026-08-30T00:00:00Z".into(),
            vec![
                RepoHealthCheck {
                    check_id: "many_details".into(),
                    name: "Many Details".into(),
                    category: HealthCategory::GitHygiene,
                    status: HealthStatus::Warn,
                    message: "Many items".into(),
                    details: Some(details),
                    duration_ms: 10,
                },
            ],
        ).unwrap();
        let formatted = format_repo_health_summary(&report);
        assert!(formatted.contains("item 49"));
        assert!(!formatted.contains("item 50\n"));
        assert!(formatted.contains("... (10 additional items truncated)"));
    }

    #[test]
    fn test_recover_default_repo_health_config() {
        let config = recover_default_repo_health_config();
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.max_file_bytes, 16 * 1024 * 1024);
        assert!(!config.require_clean_git);
        assert_eq!(config.security_policy_path, "SECURITY.md");
        assert_eq!(config.min_security_policy_bytes, 100);
    }

    #[test]
    fn test_reconstruct_and_reconcile_repo_health() {
        let dir = tempdir().unwrap();
        let sec_path = dir.path().join("SECURITY.md");
        let mut f = File::create(sec_path).unwrap();
        f.write_all(b"# Security Policy\n\nWe take security seriously. Please report vulnerabilities to security@example.com.\nAll disclosures are acknowledged within 24 hours and addressed within 90 days.\n").unwrap();
        drop(f);

        let config = recover_default_repo_health_config();
        let reconstructed = reconstruct_repo_health_report(dir.path(), &config).unwrap();
        assert!(validate_repo_health_report(&reconstructed).is_ok());
        assert_eq!(reconstructed.total_checks, 3);
        assert!(reconstructed.passed_checks >= 2);

        let reconciled = reconcile_repo_health(dir.path()).unwrap();
        assert!(validate_repo_health_report(&reconciled).is_ok());
        assert_eq!(reconciled.total_checks, 3);

        // Non-existent path error
        let bad_path = dir.path().join("non_existent_subdir");
        assert!(reconstruct_repo_health_report(&bad_path, &config).is_err());
    }

    #[test]
    fn test_validate_repo_health_report_corrupt_invariants() {
        let mut report = RepoHealthReport::new(
            "/repo".into(),
            "2026-08-30T00:00:00Z".into(),
            vec![
                RepoHealthCheck {
                    check_id: "test".into(),
                    name: "Test".into(),
                    category: HealthCategory::GitHygiene,
                    status: HealthStatus::Pass,
                    message: "Ok".into(),
                    details: None,
                    duration_ms: 1,
                },
            ],
        ).unwrap();

        // Alter total_checks to induce arithmetic invariant failure
        report.total_checks = 5;
        assert!(validate_repo_health_report(&report).is_err());
    }
}
