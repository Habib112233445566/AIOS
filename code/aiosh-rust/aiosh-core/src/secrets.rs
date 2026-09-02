//! Secrets & Access Hygiene data model (T-00711..T-00720).
//!
//! Provides data structures and redaction helpers for scanning,
//! detecting, and cataloging secrets, keys, and credentials without
//! exposing unredacted secret material.

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Severity classification of a detected secret finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Category/family of detected secret or credential pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretPatternKind {
    PrivateKey,
    ApiToken,
    AwsCredentials,
    PasswordInConfig,
    HighEntropyGeneric,
}

/// A single granular secret finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretFinding {
    pub rule_id: String,
    pub path: String,
    pub line_number: usize,
    pub severity: SecretSeverity,
    pub pattern_kind: SecretPatternKind,
    pub description: String,
    pub redacted_snippet: String,
    pub fingerprint: String,
}

/// Aggregated secrets hygiene scan report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretScanReport {
    pub repo_path: String,
    pub timestamp_utc: String,
    pub is_clean: bool,
    pub total_findings: u32,
    pub critical_findings: u32,
    pub high_findings: u32,
    pub medium_findings: u32,
    pub low_findings: u32,
    pub scanned_files_count: u32,
    pub findings: Vec<SecretFinding>,
}

impl SecretScanReport {
    /// Construct a new SecretScanReport automatically computing severity breakdown and cleanliness.
    pub fn new(repo_path: impl Into<String>, findings: Vec<SecretFinding>, scanned_files_count: u32) -> Self {
        let mut critical_findings = 0;
        let mut high_findings = 0;
        let mut medium_findings = 0;
        let mut low_findings = 0;

        for f in &findings {
            match f.severity {
                SecretSeverity::Critical => critical_findings += 1,
                SecretSeverity::High => high_findings += 1,
                SecretSeverity::Medium => medium_findings += 1,
                SecretSeverity::Low | SecretSeverity::Info => low_findings += 1,
            }
        }

        let total_findings = findings.len() as u32;
        let is_clean = total_findings == 0;

        Self {
            repo_path: repo_path.into(),
            timestamp_utc: Utc::now().to_rfc3339(),
            is_clean,
            total_findings,
            critical_findings,
            high_findings,
            medium_findings,
            low_findings,
            scanned_files_count,
            findings,
        }
    }

    /// Return finding counts breakdown as (critical, high, medium, low).
    pub fn severity_counts(&self) -> (u32, u32, u32, u32) {
        (self.critical_findings, self.high_findings, self.medium_findings, self.low_findings)
    }

    /// Formats a single-line observability summary of the scan.
    pub fn summary_line(&self) -> String {
        format!(
            "scanned {} files: {} findings ({} critical, {} high, {} medium, {} low, is_clean: {})",
            self.scanned_files_count,
            self.total_findings,
            self.critical_findings,
            self.high_findings,
            self.medium_findings,
            self.low_findings,
            self.is_clean
        )
    }
}

/// Helper function to safely redact sensitive strings.
///
/// Preserves first 4 and last 4 characters if string is >= 12 chars,
/// replacing the intermediate body with `****`. Strings under 12 chars
/// are replaced completely with `[REDACTED]`.
pub fn redact_secret_value(raw: &str) -> String {
    let trimmed = raw.trim();
    let char_count = trimmed.chars().count();
    if char_count >= 12 {
        let prefix: String = trimmed.chars().take(4).collect();
        let suffix: String = trimmed.chars().skip(char_count - 4).collect();
        format!("{}****{}", prefix, suffix)
    } else {
        "[REDACTED]".to_string()
    }
}

/// Validate invariants on a SecretScanReport.
pub fn validate_secret_report(report: &SecretScanReport) -> Result<(), String> {
    let expected_total = report.critical_findings + report.high_findings + report.medium_findings + report.low_findings;
    if report.total_findings != expected_total {
        return Err(format!(
            "Total findings ({}) does not match sum of severity breakdowns ({})",
            report.total_findings, expected_total
        ));
    }

    if report.total_findings != report.findings.len() as u32 {
        return Err(format!(
            "Total findings ({}) does not match findings array length ({})",
            report.total_findings,
            report.findings.len()
        ));
    }

    if report.is_clean != (report.total_findings == 0) {
        return Err(format!(
            "is_clean flag ({}) contradicts total_findings ({})",
            report.is_clean, report.total_findings
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_secret_value() {
        assert_eq!(redact_secret_value("AKIA1234567890XYZW"), "AKIA****XYZW");
        assert_eq!(redact_secret_value("short_token"), "[REDACTED]");
        assert_eq!(redact_secret_value("   ghp_1234567890abcdef   "), "ghp_****cdef");
        assert_eq!(redact_secret_value("🦀🔑password123456🔒🛡️"), "🦀🔑pa****6🔒🛡️");
    }

    #[test]
    fn test_secret_scan_report_clean() {
        let report = SecretScanReport::new("/workspace/test", vec![], 42);
        assert!(report.is_clean);
        assert_eq!(report.total_findings, 0);
        assert_eq!(report.scanned_files_count, 42);
        assert!(validate_secret_report(&report).is_ok());
    }

    #[test]
    fn test_secret_scan_report_with_findings() {
        let findings = vec![
            SecretFinding {
                rule_id: "SEC-001".into(),
                path: "config/prod.env".into(),
                line_number: 14,
                severity: SecretSeverity::Critical,
                pattern_kind: SecretPatternKind::AwsCredentials,
                description: "AWS Access Key identified".into(),
                redacted_snippet: "AKIA****1234".into(),
                fingerprint: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            },
            SecretFinding {
                rule_id: "SEC-002".into(),
                path: "src/auth.rs".into(),
                line_number: 88,
                severity: SecretSeverity::High,
                pattern_kind: SecretPatternKind::ApiToken,
                description: "GitHub PAT detected".into(),
                redacted_snippet: "ghp_****wxyz".into(),
                fingerprint: "ca978112ca1bbdcaf0643e9ec6cd84a229a28d5c414995922379f8e4e9f783ee".into(),
            },
        ];

        let report = SecretScanReport::new("/workspace/test", findings, 100);
        assert!(!report.is_clean);
        assert_eq!(report.total_findings, 2);
        assert_eq!(report.critical_findings, 1);
        assert_eq!(report.high_findings, 1);
        assert_eq!(report.medium_findings, 0);
        assert_eq!(report.low_findings, 0);
        assert!(validate_secret_report(&report).is_ok());
    }

    #[test]
    fn test_validate_secret_report_invalid() {
        let mut report = SecretScanReport::new("/workspace/test", vec![], 10);
        report.total_findings = 5; // Corrupt invariant
        assert!(validate_secret_report(&report).is_err());
    }

    #[test]
    fn test_secret_scan_report_serde_roundtrip() {
        let findings = vec![SecretFinding {
            rule_id: "SEC-003".into(),
            path: "certs/server.key".into(),
            line_number: 1,
            severity: SecretSeverity::Critical,
            pattern_kind: SecretPatternKind::PrivateKey,
            description: "RSA Private Key detected".into(),
            redacted_snippet: "-----****KEY-----".into(),
            fingerprint: "11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff".into(),
        }];
        let report = SecretScanReport::new("/repo/root", findings, 50);
        let serialized = serde_json::to_string(&report).expect("serialization failed");
        let deserialized: SecretScanReport = serde_json::from_str(&serialized).expect("deserialization failed");
        assert_eq!(report, deserialized);
        assert!(validate_secret_report(&deserialized).is_ok());
    }

    #[test]
    fn test_secret_scan_report_observability() {
        let findings = vec![
            SecretFinding {
                rule_id: "SEC-001".into(),
                path: "keys/id_rsa".into(),
                line_number: 1,
                severity: SecretSeverity::Critical,
                pattern_kind: SecretPatternKind::PrivateKey,
                description: "Private key".into(),
                redacted_snippet: "-----****KEY-----".into(),
                fingerprint: "abc".into(),
            },
            SecretFinding {
                rule_id: "SEC-002".into(),
                path: "config.json".into(),
                line_number: 10,
                severity: SecretSeverity::High,
                pattern_kind: SecretPatternKind::ApiToken,
                description: "API key".into(),
                redacted_snippet: "AKIA****1234".into(),
                fingerprint: "def".into(),
            },
        ];
        let report = SecretScanReport::new("/repo/root", findings, 25);
        assert_eq!(report.severity_counts(), (1, 1, 0, 0));
        let summary = report.summary_line();
        assert!(summary.contains("scanned 25 files"));
        assert!(summary.contains("2 findings"));
        assert!(summary.contains("1 critical"));
    }
}
