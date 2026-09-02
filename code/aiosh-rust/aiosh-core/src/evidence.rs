//! Evidence & Audit Trail data model (T-00514).
//!
//! Contract: `docs/tasks/evidence/T-00512-data-model-specification.md`.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceStep {
    Research,
    Spec,
    Scaffold,
    Implementation,
    UnitTest,
    Integration,
    SecurityReview,
    Hardening,
    Documentation,
    Verification,
}

impl EvidenceStep {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Research => "Research",
            Self::Spec => "Spec",
            Self::Scaffold => "Scaffold",
            Self::Implementation => "Implementation",
            Self::UnitTest => "UnitTest",
            Self::Integration => "Integration",
            Self::SecurityReview => "SecurityReview",
            Self::Hardening => "Hardening",
            Self::Documentation => "Documentation",
            Self::Verification => "Verification",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub task_id: u32,
    pub step: EvidenceStep,
    pub file_path: String,
    pub sha256_hash: String,
    pub timestamp_utc: String,
    pub status: String,
    pub summary: Option<String>,
}

impl EvidenceRecord {
    /// Validates an individual EvidenceRecord (T-00514).
    pub fn validate(&self) -> Result<(), String> {
        if self.task_id == 0 || self.task_id > 10000 {
            return Err(format!("task_id {} must be between 1 and 10000", self.task_id));
        }

        let path = self.file_path.trim();
        if path.is_empty() {
            return Err("file_path cannot be empty".into());
        }
        if path.len() > 1024 {
            return Err("file_path cannot exceed 1024 characters".into());
        }

        let normalized = path.replace('\\', "/");
        if normalized.starts_with('/')
            || normalized.contains(':')
            || normalized.split('/').any(|part| part == "..")
        {
            return Err(format!("file_path '{}' must be relative and cannot escape repository bounds", path));
        }

        if self.sha256_hash.len() != 64 || !self.sha256_hash.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()) {
            return Err(format!(
                "sha256_hash '{}' must be 64 lowercase hexadecimal characters",
                self.sha256_hash
            ));
        }

        if self.timestamp_utc.trim().is_empty() {
            return Err("timestamp_utc cannot be empty".into());
        }

        if let Some(ref summary) = self.summary {
            if summary.len() > 4096 {
                return Err("summary cannot exceed 4096 characters".into());
            }
        }

        match self.status.as_str() {
            "pass" | "fail" | "pending" => {}
            other => return Err(format!("Invalid evidence status '{}'; must be 'pass', 'fail', or 'pending'", other)),
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskEvidenceManifest {
    pub epic_name: String,
    pub task_range: String,
    pub generated_at: String,
    pub records: Vec<EvidenceRecord>,
}

impl Default for TaskEvidenceManifest {
    fn default() -> Self {
        Self {
            epic_name: "Phase 0 — Governance, Repo, CI, Task Execution / Evidence & Audit Trail".into(),
            task_range: "T-00511..T-00520".into(),
            generated_at: crate::canonical::utcnow_iso(),
            records: Vec::new(),
        }
    }
}

impl TaskEvidenceManifest {
    /// Deserializes and validates a TaskEvidenceManifest from JSON (T-00514).
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let manifest: TaskEvidenceManifest = serde_json::from_str(json_str)
            .map_err(|e| format!("Malformed task evidence manifest JSON: {e}"))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Serializes to canonical JSON (T-00514).
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize task evidence manifest: {e}"))
    }

    /// Validates the entire manifest against schema invariants (T-00514).
    pub fn validate(&self) -> Result<(), String> {
        if self.epic_name.trim().is_empty() || self.epic_name.len() > 256 {
            return Err("epic_name must be non-empty and <= 256 characters".into());
        }

        if self.task_range.trim().is_empty() || self.task_range.len() > 64 {
            return Err("task_range must be non-empty and <= 64 characters".into());
        }

        if self.generated_at.trim().is_empty() {
            return Err("generated_at cannot be empty".into());
        }

        if self.records.len() > 10000 {
            return Err(format!("records count {} exceeds maximum allowed limit of 10000", self.records.len()));
        }

        let mut seen_keys = HashSet::new();
        for record in &self.records {
            record.validate()?;
            let key = (record.task_id, record.step.clone());
            if !seen_keys.insert(key) {
                return Err(format!(
                    "Duplicate evidence record for task {} step {:?}",
                    record.task_id, record.step
                ));
            }
        }

        Ok(())
    }

    /// Finds a record by task ID.
    pub fn get_record(&self, task_id: u32) -> Option<&EvidenceRecord> {
        self.records.iter().find(|r| r.task_id == task_id)
    }

    /// Filters records by step enum.
    pub fn filter_by_step(&self, step: &EvidenceStep) -> Vec<&EvidenceRecord> {
        self.records.iter().filter(|r| &r.step == step).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceVerificationReport {
    pub total_records: usize,
    pub valid_records: usize,
    pub missing_files: Vec<String>,
    pub hash_mismatches: Vec<String>,
    pub is_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceTelemetry {
    pub total_records: usize,
    pub valid_records: usize,
    pub missing_files_count: usize,
    pub hash_mismatches_count: usize,
    pub is_healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_record_valid() {
        let record = EvidenceRecord {
            task_id: 514,
            step: EvidenceStep::Implementation,
            file_path: "docs/tasks/evidence/T-00514-data-model-implementation.md".into(),
            sha256_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            timestamp_utc: "2026-08-28T14:48:00Z".into(),
            status: "pass".into(),
            summary: Some("Implementation of data model".into()),
        };
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_evidence_record_invalid_hash() {
        let record = EvidenceRecord {
            task_id: 514,
            step: EvidenceStep::Implementation,
            file_path: "docs/tasks/evidence/T-00514.md".into(),
            sha256_hash: "UPPERCASE_AND_TOO_SHORT".into(),
            timestamp_utc: "2026-08-28T14:48:00Z".into(),
            status: "pass".into(),
            summary: None,
        };
        assert!(record.validate().is_err());
        assert!(record.validate().unwrap_err().contains("64 lowercase hexadecimal characters"));
    }

    #[test]
    fn test_evidence_record_path_traversal() {
        let record = EvidenceRecord {
            task_id: 514,
            step: EvidenceStep::Implementation,
            file_path: "../../../etc/shadow".into(),
            sha256_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            timestamp_utc: "2026-08-28T14:48:00Z".into(),
            status: "pass".into(),
            summary: None,
        };
        assert!(record.validate().is_err());
        assert!(record.validate().unwrap_err().contains("must be relative and cannot escape repository bounds"));
    }

    #[test]
    fn test_task_evidence_manifest_roundtrip_and_query() {
        let record = EvidenceRecord {
            task_id: 511,
            step: EvidenceStep::Research,
            file_path: "docs/tasks/evidence/T-00511-data-model-research.md".into(),
            sha256_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            timestamp_utc: "2026-08-28T14:48:00Z".into(),
            status: "pass".into(),
            summary: Some("Research notes".into()),
        };
        let mut manifest = TaskEvidenceManifest::default();
        manifest.records.push(record);

        let json = manifest.to_json().unwrap();
        let parsed = TaskEvidenceManifest::from_json(&json).unwrap();
        assert_eq!(parsed.records.len(), 1);
        assert_eq!(parsed.get_record(511).unwrap().task_id, 511);
        assert_eq!(parsed.filter_by_step(&EvidenceStep::Research).len(), 1);
        assert_eq!(parsed.filter_by_step(&EvidenceStep::Verification).len(), 0);
    }

    #[test]
    fn test_evidence_record_task_id_bounds() {
        let mut record = EvidenceRecord {
            task_id: 0,
            step: EvidenceStep::Research,
            file_path: "docs/tasks/evidence/T-00001.md".into(),
            sha256_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            timestamp_utc: "2026-08-28T14:48:00Z".into(),
            status: "pass".into(),
            summary: None,
        };
        assert!(record.validate().is_err());

        record.task_id = 10001;
        assert!(record.validate().is_err());

        record.task_id = 1;
        assert!(record.validate().is_ok());

        record.task_id = 10000;
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_evidence_record_invalid_status() {
        let record = EvidenceRecord {
            task_id: 515,
            step: EvidenceStep::UnitTest,
            file_path: "docs/tasks/evidence/T-00515.md".into(),
            sha256_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            timestamp_utc: "2026-08-28T14:48:00Z".into(),
            status: "unknown_status".into(),
            summary: None,
        };
        assert!(record.validate().is_err());
        assert!(record.validate().unwrap_err().contains("Invalid evidence status"));
    }

    #[test]
    fn test_task_evidence_manifest_duplicate_error() {
        let record1 = EvidenceRecord {
            task_id: 515,
            step: EvidenceStep::UnitTest,
            file_path: "docs/tasks/evidence/T-00515a.md".into(),
            sha256_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            timestamp_utc: "2026-08-28T14:48:00Z".into(),
            status: "pass".into(),
            summary: None,
        };
        let record2 = EvidenceRecord {
            task_id: 515,
            step: EvidenceStep::UnitTest,
            file_path: "docs/tasks/evidence/T-00515b.md".into(),
            sha256_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            timestamp_utc: "2026-08-28T14:48:00Z".into(),
            status: "pass".into(),
            summary: None,
        };
        let mut manifest = TaskEvidenceManifest::default();
        manifest.records.push(record1);
        manifest.records.push(record2);
        assert!(manifest.validate().is_err());
        assert!(manifest.validate().unwrap_err().contains("Duplicate evidence record"));
    }

    #[test]
    fn test_evidence_step_as_str_all_variants() {
        let steps = vec![
            (EvidenceStep::Research, "Research"),
            (EvidenceStep::Spec, "Spec"),
            (EvidenceStep::Scaffold, "Scaffold"),
            (EvidenceStep::Implementation, "Implementation"),
            (EvidenceStep::UnitTest, "UnitTest"),
            (EvidenceStep::Integration, "Integration"),
            (EvidenceStep::SecurityReview, "SecurityReview"),
            (EvidenceStep::Hardening, "Hardening"),
            (EvidenceStep::Documentation, "Documentation"),
            (EvidenceStep::Verification, "Verification"),
        ];
        for (step, expected) in steps {
            assert_eq!(step.as_str(), expected);
        }
    }
}
