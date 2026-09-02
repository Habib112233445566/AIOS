//! Repository Health data model (T-00613 / T-00614).
//!
//! Contract: `docs/tasks/evidence/T-00612-data-model-specification.md`.

use serde::{Deserialize, Serialize};

/// Discrete health status level for individual checks and aggregate reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

impl HealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "Pass",
            Self::Warn => "Warn",
            Self::Fail => "Fail",
            Self::Skip => "Skip",
        }
    }
}

/// Domain categories for repository health checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthCategory {
    GitHygiene,
    FileIntegrity,
    SecurityGovernance,
    DependencyHygiene,
    WorkspaceBounds,
}

impl HealthCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GitHygiene => "GitHygiene",
            Self::FileIntegrity => "FileIntegrity",
            Self::SecurityGovernance => "SecurityGovernance",
            Self::DependencyHygiene => "DependencyHygiene",
            Self::WorkspaceBounds => "WorkspaceBounds",
        }
    }
}

/// An individual repository health check evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoHealthCheck {
    pub check_id: String,
    pub name: String,
    pub category: HealthCategory,
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<Vec<String>>,
    pub duration_ms: u64,
}

impl RepoHealthCheck {
    /// Validates an individual RepoHealthCheck record (T-00614).
    pub fn validate(&self) -> Result<(), String> {
        let cid = self.check_id.trim();
        if cid.is_empty() || cid.len() > 64 {
            return Err("check_id must be between 1 and 64 characters".into());
        }
        if !cid.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err("check_id must contain only alphanumeric, underscore, or hyphen characters".into());
        }

        let name = self.name.trim();
        if name.is_empty() || name.len() > 128 {
            return Err("name must be between 1 and 128 characters".into());
        }

        if self.message.len() > 1024 {
            return Err("message exceeds 1024 character limit".into());
        }

        if let Some(ref details) = self.details {
            if details.len() > 100 {
                return Err("details list exceeds maximum length of 100 items".into());
            }
            for (idx, item) in details.iter().enumerate() {
                if item.len() > 512 {
                    return Err(format!("detail item at index {} exceeds 512 character limit", idx));
                }
            }
        }

        Ok(())
    }
}

/// Aggregated repository health assessment report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoHealthReport {
    pub repo_path: String,
    pub timestamp_utc: String,
    pub overall_status: HealthStatus,
    pub total_checks: u32,
    pub passed_checks: u32,
    pub warn_checks: u32,
    pub failed_checks: u32,
    pub skipped_checks: u32,
    pub checks: Vec<RepoHealthCheck>,
}

impl RepoHealthReport {
    /// Constructs a new RepoHealthReport, automatically computing totals and overall status.
    pub fn new(repo_path: String, timestamp_utc: String, checks: Vec<RepoHealthCheck>) -> Result<Self, String> {
        let mut passed = 0;
        let mut warn = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for check in &checks {
            check.validate()?;
            match check.status {
                HealthStatus::Pass => passed += 1,
                HealthStatus::Warn => warn += 1,
                HealthStatus::Fail => failed += 1,
                HealthStatus::Skip => skipped += 1,
            }
        }

        let overall_status = if failed > 0 {
            HealthStatus::Fail
        } else if warn > 0 {
            HealthStatus::Warn
        } else {
            HealthStatus::Pass
        };

        let report = Self {
            repo_path,
            timestamp_utc,
            overall_status,
            total_checks: checks.len() as u32,
            passed_checks: passed,
            warn_checks: warn,
            failed_checks: failed,
            skipped_checks: skipped,
            checks,
        };

        report.validate()?;
        Ok(report)
    }

    /// Validates the full RepoHealthReport structural and mathematical invariants.
    pub fn validate(&self) -> Result<(), String> {
        if self.repo_path.trim().is_empty() || self.repo_path.len() > 1024 {
            return Err("repo_path must be between 1 and 1024 characters".into());
        }

        if self.timestamp_utc.trim().is_empty() {
            return Err("timestamp_utc cannot be empty".into());
        }

        if self.total_checks != self.checks.len() as u32 {
            return Err(format!(
                "total_checks mismatch: field says {}, but checks array has {}",
                self.total_checks,
                self.checks.len()
            ));
        }

        let sum = self.passed_checks + self.warn_checks + self.failed_checks + self.skipped_checks;
        if sum != self.total_checks {
            return Err(format!(
                "sub-status counts sum to {} (passed: {}, warn: {}, fail: {}, skip: {}), but total_checks is {}",
                sum, self.passed_checks, self.warn_checks, self.failed_checks, self.skipped_checks, self.total_checks
            ));
        }

        for (idx, check) in self.checks.iter().enumerate() {
            check.validate().map_err(|e| format!("check item at index {}: {}", idx, e))?;
        }

        let expected_overall = if self.failed_checks > 0 {
            HealthStatus::Fail
        } else if self.warn_checks > 0 {
            HealthStatus::Warn
        } else {
            HealthStatus::Pass
        };

        if self.overall_status != expected_overall {
            return Err(format!(
                "overall_status {:?} does not match expected status {:?} derived from check counts",
                self.overall_status, expected_overall
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_health_check_validation_happy() {
        let check = RepoHealthCheck {
            check_id: "git_status".into(),
            name: "Git Working Tree Cleanliness".into(),
            category: HealthCategory::GitHygiene,
            status: HealthStatus::Pass,
            message: "Working tree clean".into(),
            details: Some(vec!["detail 1".into(), "detail 2".into()]),
            duration_ms: 10,
        };
        assert!(check.validate().is_ok());
    }

    #[test]
    fn test_repo_health_check_validation_errors() {
        // Empty check_id
        let mut check = RepoHealthCheck {
            check_id: "".into(),
            name: "Git Status".into(),
            category: HealthCategory::GitHygiene,
            status: HealthStatus::Pass,
            message: "ok".into(),
            details: None,
            duration_ms: 5,
        };
        assert!(check.validate().is_err());

        // Invalid characters in check_id
        check.check_id = "git status with spaces".into();
        assert!(check.validate().is_err());

        // Empty name
        check.check_id = "git_status".into();
        check.name = "".into();
        assert!(check.validate().is_err());

        // Oversized message (>1024)
        check.name = "Git Status".into();
        check.message = "a".repeat(1025);
        assert!(check.validate().is_err());
    }

    #[test]
    fn test_repo_health_report_happy_and_status_derivation() {
        let c1 = RepoHealthCheck {
            check_id: "git_clean".into(),
            name: "Git Clean".into(),
            category: HealthCategory::GitHygiene,
            status: HealthStatus::Pass,
            message: "Clean".into(),
            details: None,
            duration_ms: 5,
        };
        let c2 = RepoHealthCheck {
            check_id: "security_policy".into(),
            name: "Security Policy".into(),
            category: HealthCategory::SecurityGovernance,
            status: HealthStatus::Pass,
            message: "Valid".into(),
            details: None,
            duration_ms: 2,
        };

        // All Pass -> Pass
        let report = RepoHealthReport::new("/workspace".into(), "2026-08-29T00:00:00Z".into(), vec![c1.clone(), c2.clone()]).unwrap();
        assert_eq!(report.overall_status, HealthStatus::Pass);
        assert_eq!(report.total_checks, 2);
        assert_eq!(report.passed_checks, 2);

        // One Warn -> Warn
        let c_warn = RepoHealthCheck {
            check_id: "disk_usage".into(),
            name: "Disk Usage".into(),
            category: HealthCategory::WorkspaceBounds,
            status: HealthStatus::Warn,
            message: "High usage".into(),
            details: None,
            duration_ms: 3,
        };
        let warn_report = RepoHealthReport::new("/workspace".into(), "2026-08-29T00:00:00Z".into(), vec![c1.clone(), c_warn.clone()]).unwrap();
        assert_eq!(warn_report.overall_status, HealthStatus::Warn);
        assert_eq!(warn_report.warn_checks, 1);

        // One Fail -> Fail (even if Warn present)
        let c_fail = RepoHealthCheck {
            check_id: "security_vuln".into(),
            name: "Security Vulnerability".into(),
            category: HealthCategory::SecurityGovernance,
            status: HealthStatus::Fail,
            message: "Vulnerability found".into(),
            details: None,
            duration_ms: 4,
        };
        let fail_report = RepoHealthReport::new("/workspace".into(), "2026-08-29T00:00:00Z".into(), vec![c1, c_warn, c_fail]).unwrap();
        assert_eq!(fail_report.overall_status, HealthStatus::Fail);
        assert_eq!(fail_report.failed_checks, 1);
        assert_eq!(fail_report.total_checks, 3);
    }

    #[test]
    fn test_repo_health_report_validation_errors() {
        let c1 = RepoHealthCheck {
            check_id: "git_clean".into(),
            name: "Git Clean".into(),
            category: HealthCategory::GitHygiene,
            status: HealthStatus::Pass,
            message: "Clean".into(),
            details: None,
            duration_ms: 5,
        };

        // Empty repo_path
        let mut report = RepoHealthReport::new("/workspace".into(), "2026-08-29T00:00:00Z".into(), vec![c1.clone()]).unwrap();
        report.repo_path = "".into();
        assert!(report.validate().is_err());

        // Empty timestamp_utc
        report.repo_path = "/workspace".into();
        report.timestamp_utc = "".into();
        assert!(report.validate().is_err());

        // Count mismatch
        report.timestamp_utc = "2026-08-29T00:00:00Z".into();
        report.total_checks = 10;
        assert!(report.validate().is_err());

        // Sub-status sum mismatch
        report.total_checks = 1;
        report.passed_checks = 2;
        assert!(report.validate().is_err());

        // Inconsistent overall status (Pass overall when Fail check present)
        let c_fail = RepoHealthCheck {
            check_id: "fail_check".into(),
            name: "Fail Check".into(),
            category: HealthCategory::SecurityGovernance,
            status: HealthStatus::Fail,
            message: "Failed".into(),
            details: None,
            duration_ms: 1,
        };
        let mut report_bad_status = RepoHealthReport::new("/workspace".into(), "2026-08-29T00:00:00Z".into(), vec![c_fail]).unwrap();
        report_bad_status.overall_status = HealthStatus::Pass;
        assert!(report_bad_status.validate().is_err());
    }

    #[test]
    fn test_repo_health_report_json_roundtrip() {
        let c1 = RepoHealthCheck {
            check_id: "git_status".into(),
            name: "Git Status".into(),
            category: HealthCategory::GitHygiene,
            status: HealthStatus::Pass,
            message: "Working tree clean".into(),
            details: Some(vec!["branch: main".into()]),
            duration_ms: 8,
        };
        let report = RepoHealthReport::new("/repo".into(), "2026-08-29T00:00:00Z".into(), vec![c1]).unwrap();

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: RepoHealthReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report, deserialized);
        assert!(deserialized.validate().is_ok());
    }
}

