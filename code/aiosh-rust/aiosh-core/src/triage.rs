//! Regression Triage data model (T-00811..T-00820).
//!
//! Provides data structures, failure signatures, severity ratings,
//! and validation helpers for managing test regressions and triage records.

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::canonical::sha256_hex;

/// Lifecycle status of a regression triage record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriageStatus {
    Untriaged,
    Triaged,
    FixPending,
    Resolved,
    WontFix,
}

/// Severity classification of a regression finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriageSeverity {
    Blocker,
    Critical,
    Major,
    Minor,
}

/// Maximum size of an error message string (64 KiB).
pub const MAX_ERROR_MSG_BYTES: usize = 65536;

/// Maximum size of a repro command string (4 KiB).
pub const MAX_REPRO_CMD_BYTES: usize = 4096;

/// Maximum size of a test target identifier (512 bytes).
pub const MAX_TEST_TARGET_BYTES: usize = 512;

/// Granular triage record for an observed regression or test failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriageRecord {
    pub id: String,
    pub signature: String,
    pub test_target: String,
    pub suite_name: String,
    pub error_message: String,
    pub repro_command: String,
    pub first_observed_at: String,
    pub last_observed_at: String,
    pub occurrences: u32,
    pub status: TriageStatus,
    pub severity: TriageSeverity,
    pub blame_task_id: Option<u32>,
    pub blame_commit: Option<String>,
    pub resolution_notes: Option<String>,
}

impl TriageRecord {
    /// Create a new triage record from test failure information.
    pub fn new(
        test_target: impl Into<String>,
        suite_name: impl Into<String>,
        error_message: impl Into<String>,
        repro_command: impl Into<String>,
        severity: TriageSeverity,
    ) -> Self {
        let mut test_target = test_target.into();
        if test_target.len() > MAX_TEST_TARGET_BYTES {
            test_target.truncate(MAX_TEST_TARGET_BYTES);
        }

        let mut error_message = error_message.into();
        if error_message.len() > MAX_ERROR_MSG_BYTES {
            error_message.truncate(MAX_ERROR_MSG_BYTES);
        }

        let mut repro_command = repro_command.into();
        if repro_command.len() > MAX_REPRO_CMD_BYTES {
            repro_command.truncate(MAX_REPRO_CMD_BYTES);
        }

        let signature = compute_failure_signature(&test_target, &error_message);
        let id = format!("TRG-{}", &signature[..8]);
        let now = Utc::now().to_rfc3339();

        Self {
            id,
            signature,
            test_target,
            suite_name: suite_name.into(),
            error_message,
            repro_command: repro_command.into(),
            first_observed_at: now.clone(),
            last_observed_at: now,
            occurrences: 1,
            status: TriageStatus::Untriaged,
            severity,
            blame_task_id: None,
            blame_commit: None,
            resolution_notes: None,
        }
    }

    /// Record a recurring occurrence of this regression.
    pub fn record_occurrence(&mut self) {
        self.occurrences = self.occurrences.saturating_add(1);
        self.last_observed_at = Utc::now().to_rfc3339();
    }
}

/// Aggregated triage report for regressions across workspaces or CI runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriageReport {
    pub timestamp_utc: String,
    pub total_records: u32,
    pub open_records: u32,
    pub resolved_records: u32,
    pub records: Vec<TriageRecord>,
}

impl TriageReport {
    /// Construct a new TriageReport from a list of records.
    pub fn new(records: Vec<TriageRecord>) -> Self {
        let mut open_records = 0;
        let mut resolved_records = 0;

        for r in &records {
            match r.status {
                TriageStatus::Resolved | TriageStatus::WontFix => resolved_records += 1,
                _ => open_records += 1,
            }
        }

        let total_records = records.len() as u32;

        Self {
            timestamp_utc: Utc::now().to_rfc3339(),
            total_records,
            open_records,
            resolved_records,
            records,
        }
    }

    /// Returns breakdown by lifecycle status: (untriaged, triaged, fix_pending, resolved, wont_fix).
    pub fn status_counts(&self) -> (usize, usize, usize, usize, usize) {
        let mut untriaged = 0;
        let mut triaged = 0;
        let mut fix_pending = 0;
        let mut resolved = 0;
        let mut wont_fix = 0;

        for r in &self.records {
            match r.status {
                TriageStatus::Untriaged => untriaged += 1,
                TriageStatus::Triaged => triaged += 1,
                TriageStatus::FixPending => fix_pending += 1,
                TriageStatus::Resolved => resolved += 1,
                TriageStatus::WontFix => wont_fix += 1,
            }
        }

        (untriaged, triaged, fix_pending, resolved, wont_fix)
    }

    /// Returns breakdown by severity: (blocker, critical, major, minor).
    pub fn severity_counts(&self) -> (usize, usize, usize, usize) {
        let mut blocker = 0;
        let mut critical = 0;
        let mut major = 0;
        let mut minor = 0;

        for r in &self.records {
            match r.severity {
                TriageSeverity::Blocker => blocker += 1,
                TriageSeverity::Critical => critical += 1,
                TriageSeverity::Major => major += 1,
                TriageSeverity::Minor => minor += 1,
            }
        }

        (blocker, critical, major, minor)
    }

    /// Formats a standardized human-readable single-line summary string.
    pub fn summary_line(&self) -> String {
        let (blocker, critical, major, minor) = self.severity_counts();
        format!(
            "Triage Report: {} total ({} blocker, {} critical, {} major, {} minor | {} open, {} resolved)",
            self.total_records, blocker, critical, major, minor, self.open_records, self.resolved_records
        )
    }
}

/// Compute a normalized deterministic SHA-256 signature for a test failure.
pub fn compute_failure_signature(test_target: &str, error_message: &str) -> String {
    let normalized_target = test_target.trim().to_lowercase();
    let normalized_err = error_message.trim().replace("\r\n", "\n");
    let payload = format!("{}::{}", normalized_target, normalized_err);
    sha256_hex(&payload)
}

/// Validate structural invariants of a single TriageRecord.
pub fn validate_triage_record(record: &TriageRecord) -> Result<(), String> {
    if record.id.trim().is_empty() {
        return Err("Record id cannot be empty".into());
    }
    if !record.id.starts_with("TRG-") {
        return Err(format!("Record id '{}' must start with 'TRG-'", record.id));
    }
    if record.signature.len() != 64 {
        return Err(format!(
            "Signature length ({}) must be exactly 64 hex characters",
            record.signature.len()
        ));
    }
    if record.test_target.trim().is_empty() {
        return Err("Test target cannot be empty".into());
    }
    if record.error_message.trim().is_empty() {
        return Err("Error message cannot be empty".into());
    }
    if record.occurrences == 0 {
        return Err("Occurrences must be at least 1".into());
    }

    Ok(())
}

/// Validate invariant consistency for a TriageReport.
pub fn validate_triage_report(report: &TriageReport) -> Result<(), String> {
    if report.total_records != report.records.len() as u32 {
        return Err(format!(
            "Total records ({}) does not match records array length ({})",
            report.total_records,
            report.records.len()
        ));
    }

    if report.open_records + report.resolved_records != report.total_records {
        return Err(format!(
            "Open ({}) + resolved ({}) does not equal total ({})",
            report.open_records, report.resolved_records, report.total_records
        ));
    }

    for record in &report.records {
        validate_triage_record(record)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_failure_signature_deterministic() {
        let sig1 = compute_failure_signature("secrets::tests::test_key", "assertion failed: expected true, got false");
        let sig2 = compute_failure_signature("secrets::tests::test_key", "assertion failed: expected true, got false\r\n");
        assert_eq!(sig1, sig2);
        assert_eq!(sig1.len(), 64);
    }

    #[test]
    fn test_triage_record_creation_and_recurrence() {
        let mut rec = TriageRecord::new(
            "ci::tests::test_run",
            "ci_unit",
            "process exited with status 1",
            "cargo test --lib ci::tests",
            TriageSeverity::Critical,
        );
        assert_eq!(rec.occurrences, 1);
        assert_eq!(rec.status, TriageStatus::Untriaged);
        assert!(rec.id.starts_with("TRG-"));

        rec.record_occurrence();
        assert_eq!(rec.occurrences, 2);
    }

    #[test]
    fn test_triage_report_validation() {
        let rec1 = TriageRecord::new(
            "t1", "s1", "err1", "cmd1", TriageSeverity::Major,
        );
        let mut rec2 = TriageRecord::new(
            "t2", "s1", "err2", "cmd2", TriageSeverity::Minor,
        );
        rec2.status = TriageStatus::Resolved;

        let report = TriageReport::new(vec![rec1, rec2]);
        assert_eq!(report.total_records, 2);
        assert_eq!(report.open_records, 1);
        assert_eq!(report.resolved_records, 1);
        assert!(validate_triage_report(&report).is_ok());

        let mut invalid_report = report.clone();
        invalid_report.total_records = 5;
        assert!(validate_triage_report(&invalid_report).is_err());
    }

    #[test]
    fn test_triage_report_serde_roundtrip() {
        let rec = TriageRecord::new(
            "t1", "s1", "err1", "cmd1", TriageSeverity::Blocker,
        );
        let report = TriageReport::new(vec![rec]);
        let serialized = serde_json::to_string(&report).expect("serialization failed");
        let deserialized: TriageReport = serde_json::from_str(&serialized).expect("deserialization failed");
        assert_eq!(report, deserialized);
        assert!(validate_triage_report(&deserialized).is_ok());
    }

    #[test]
    fn test_triage_report_observability() {
        let mut rec1 = TriageRecord::new("t1", "s1", "err1", "cmd1", TriageSeverity::Blocker);
        rec1.status = TriageStatus::Untriaged;
        let mut rec2 = TriageRecord::new("t2", "s1", "err2", "cmd2", TriageSeverity::Critical);
        rec2.status = TriageStatus::Resolved;

        let report = TriageReport::new(vec![rec1, rec2]);
        let (untriaged, triaged, fix_pending, resolved, wont_fix) = report.status_counts();
        assert_eq!(untriaged, 1);
        assert_eq!(triaged, 0);
        assert_eq!(fix_pending, 0);
        assert_eq!(resolved, 1);
        assert_eq!(wont_fix, 0);

        let (blocker, critical, major, minor) = report.severity_counts();
        assert_eq!(blocker, 1);
        assert_eq!(critical, 1);
        assert_eq!(major, 0);
        assert_eq!(minor, 0);

        let summary = report.summary_line();
        assert!(summary.contains("2 total"));
        assert!(summary.contains("1 blocker"));
        assert!(summary.contains("1 critical"));
    }

    #[test]
    fn test_validate_triage_record() {
        let valid_rec = TriageRecord::new("t1", "s1", "err1", "cmd1", TriageSeverity::Major);
        assert!(validate_triage_record(&valid_rec).is_ok());

        let mut invalid_rec = valid_rec.clone();
        invalid_rec.id = "INVALID_ID".into();
        assert!(validate_triage_record(&invalid_rec).is_err());

        let mut empty_target_rec = valid_rec.clone();
        empty_target_rec.test_target = "".into();
        assert!(validate_triage_record(&empty_target_rec).is_err());
    }
}
