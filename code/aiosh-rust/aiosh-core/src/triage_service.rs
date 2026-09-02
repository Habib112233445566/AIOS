//! Regression Triage core service (T-00821..T-00830).
//!
//! Manages in-memory and disk-backed triage stores, ingest CI run summaries,
//! deduplicates failures by signature, and updates regression lifecycles.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::ci::RunSummary;
use crate::triage::{TriageRecord, TriageReport, TriageSeverity, TriageStatus};
use crate::triage_config::TriageConfig;

/// Default maximum size cap for triage store JSON file (1 MiB).
pub const MAX_TRIAGE_STORE_BYTES: u64 = 1024 * 1024;

/// Disk-backed and in-memory repository for regression triage records.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriageStore {
    records: HashMap<String, TriageRecord>,
    id_index: HashMap<String, String>,
}

impl TriageStore {
    /// Initialize an empty TriageStore.
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            id_index: HashMap::new(),
        }
    }

    /// Returns the number of triage records in the store.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true if the store contains no records.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Record a failure, either instantiating a new TriageRecord or incrementing occurrence count.
    pub fn record_failure(
        &mut self,
        test_target: &str,
        suite_name: &str,
        error_message: &str,
        repro_cmd: &str,
        severity: TriageSeverity,
    ) -> TriageRecord {
        let candidate = TriageRecord::new(test_target, suite_name, error_message, repro_cmd, severity);
        let sig = candidate.signature.clone();

        if let Some(existing) = self.records.get_mut(&sig) {
            existing.record_occurrence();
            if existing.status == TriageStatus::Resolved {
                existing.status = TriageStatus::Triaged; // Reopened regression!
            }
            existing.clone()
        } else {
            self.id_index.insert(candidate.id.clone(), sig.clone());
            self.records.insert(sig, candidate.clone());
            candidate
        }
    }

    /// Ingest a CI RunSummary and record all failed test suites as triage candidates.
    pub fn ingest_ci_summary(&mut self, summary: &RunSummary) -> usize {
        self.ingest_ci_summary_with_config(summary, &TriageConfig::default())
    }

    /// Ingest a CI RunSummary according to configuration filters and default severity.
    pub fn ingest_ci_summary_with_config(&mut self, summary: &RunSummary, config: &TriageConfig) -> usize {
        let mut processed = 0;
        for r in &summary.results {
            if r.status != "pass" && config.should_ingest_suite(&r.suite) {
                let err_msg = format!("suite {} exited with code {:?}", r.suite, r.exit_code);
                let repro = format!("cargo test --test {}", r.suite);
                self.record_failure(&r.suite, &r.suite, &err_msg, &repro, config.default_severity);
                processed += 1;
            }
        }
        processed
    }

    /// Look up a triage record by its TRG-xxxxxxxx ID.
    pub fn get_by_id(&self, id: &str) -> Option<&TriageRecord> {
        let sig = self.id_index.get(id)?;
        self.records.get(sig)
    }

    /// Look up a triage record by its SHA-256 signature.
    pub fn get_by_signature(&self, signature: &str) -> Option<&TriageRecord> {
        self.records.get(signature)
    }

    /// Update status and optional notes on an existing triage record.
    pub fn update_status(
        &mut self,
        id: &str,
        status: TriageStatus,
        notes: Option<String>,
    ) -> Result<&TriageRecord, String> {
        let sig = self.id_index.get(id).ok_or_else(|| format!("Record ID {} not found", id))?.clone();
        let rec = self.records.get_mut(&sig).ok_or_else(|| format!("Record for ID {} missing", id))?;

        rec.status = status;
        rec.last_observed_at = Utc::now().to_rfc3339();
        if let Some(n) = notes {
            rec.resolution_notes = Some(n);
        }

        Ok(rec)
    }

    /// Mark a triage record as resolved with resolution notes.
    pub fn resolve(&mut self, id: &str, notes: &str) -> Result<&TriageRecord, String> {
        self.update_status(id, TriageStatus::Resolved, Some(notes.to_string()))
    }

    /// Generate an aggregated TriageReport from all stored records.
    pub fn to_report(&self) -> TriageReport {
        let mut list: Vec<TriageRecord> = self.records.values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        TriageReport::new(list)
    }

    /// Save the store to a JSON file at the specified path.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        let serialized = serde_json::to_string_pretty(self).map_err(|e| format!("Serialization error: {}", e))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent dirs: {}", e))?;
        }
        let mut f = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
        f.write_all(serialized.as_bytes()).map_err(|e| format!("Failed to write: {}", e))?;
        Ok(())
    }

    /// Load the store from a JSON file, enforcing the default maximum size cap.
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        Self::load_from_path_with_config(path, &TriageConfig::default())
    }

    /// Load the store from a JSON file, enforcing the configuration maximum size cap.
    pub fn load_from_path_with_config(path: &Path, config: &TriageConfig) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let meta = fs::metadata(path).map_err(|e| format!("Failed to read metadata: {}", e))?;
        let max_bytes = config.max_store_bytes as u64;
        if meta.len() > max_bytes {
            return Err(format!(
                "Triage store {} exceeds size limit ({} > {} bytes)",
                path.display(), meta.len(), max_bytes
            ));
        }

        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let store: Self = serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))?;
        Ok(store)
    }

    /// Load the store from a path or recover to a fresh store with an honest error warning.
    pub fn load_or_recover(path: &Path) -> (Self, Option<String>) {
        match Self::load_from_path(path) {
            Ok(store) => (store, None),
            Err(err) => (Self::new(), Some(format!("Recovered from error loading {}: {}", path.display(), err))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_store_record_and_lookup() {
        let mut store = TriageStore::new();
        let rec = store.record_failure(
            "test_target_a",
            "suite_1",
            "panicked at assertion",
            "cargo test --test suite_1",
            TriageSeverity::Blocker,
        );

        let retrieved = store.get_by_id(&rec.id).expect("found by id");
        assert_eq!(retrieved.occurrences, 1);
        assert_eq!(retrieved.status, TriageStatus::Untriaged);

        // Record again -> occurrences increments
        let updated = store.record_failure(
            "test_target_a",
            "suite_1",
            "panicked at assertion",
            "cargo test --test suite_1",
            TriageSeverity::Blocker,
        );
        assert_eq!(updated.occurrences, 2);
    }

    #[test]
    fn test_store_resolve_and_reopen() {
        let mut store = TriageStore::new();
        let rec = store.record_failure(
            "test_target_b",
            "suite_2",
            "timeout exceeded",
            "cargo test",
            TriageSeverity::Critical,
        );

        store.resolve(&rec.id, "Fixed lock contention").expect("resolved");
        let rec_resolved = store.get_by_id(&rec.id).unwrap();
        assert_eq!(rec_resolved.status, TriageStatus::Resolved);
        assert_eq!(rec_resolved.resolution_notes.as_deref(), Some("Fixed lock contention"));

        // Recurrence reopens it!
        store.record_failure(
            "test_target_b",
            "suite_2",
            "timeout exceeded",
            "cargo test",
            TriageSeverity::Critical,
        );
        let rec_reopened = store.get_by_id(&rec.id).unwrap();
        assert_eq!(rec_reopened.status, TriageStatus::Triaged);
    }

    #[test]
    fn test_store_file_roundtrip() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("triage_store.json");

        let mut store = TriageStore::new();
        store.record_failure("t1", "s1", "err", "cmd", TriageSeverity::Major);
        store.save_to_path(&store_path).expect("saved");

        let loaded = TriageStore::load_from_path(&store_path).expect("loaded");
        assert_eq!(store, loaded);
    }

    #[test]
    fn test_store_config_integration() {
        use crate::ci::ResultRecord;

        let mut store = TriageStore::new();
        let summary = RunSummary {
            tool: "ci_run".into(),
            schema_version: 1,
            started_at: "2026-08-31T00:00:00Z".into(),
            finished_at: "2026-08-31T00:00:01Z".into(),
            total: 3,
            passed: 1,
            failed: 2,
            all_pass: false,
            results: vec![
                ResultRecord {
                    suite: "suite_pass".into(),
                    index: 0,
                    status: "pass".into(),
                    exit_code: Some(0),
                    duration_ms: 10,
                    started_at: "2026-08-31T00:00:00Z".into(),
                    finished_at: "2026-08-31T00:00:00Z".into(),
                    log_path: "logs/0.log".into(),
                },
                ResultRecord {
                    suite: "sec_token_leak".into(),
                    index: 1,
                    status: "fail".into(),
                    exit_code: Some(1),
                    duration_ms: 20,
                    started_at: "2026-08-31T00:00:00Z".into(),
                    finished_at: "2026-08-31T00:00:00Z".into(),
                    log_path: "logs/1.log".into(),
                },
                ResultRecord {
                    suite: "perf_bench".into(),
                    index: 2,
                    status: "fail".into(),
                    exit_code: Some(1),
                    duration_ms: 20,
                    started_at: "2026-08-31T00:00:00Z".into(),
                    finished_at: "2026-08-31T00:00:00Z".into(),
                    log_path: "logs/2.log".into(),
                },
            ],
        };

        let mut cfg = TriageConfig::default();
        cfg.auto_ingest_suites = vec!["sec_*".into()];
        cfg.default_severity = TriageSeverity::Blocker;

        let processed = store.ingest_ci_summary_with_config(&summary, &cfg);
        assert_eq!(processed, 1);

        let report = store.to_report();
        assert_eq!(report.total_records, 1);
        assert_eq!(report.records[0].suite_name, "sec_token_leak");
        assert_eq!(report.records[0].severity, TriageSeverity::Blocker);

        // Test size limit validation with config
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("triage_store.json");
        store.save_to_path(&store_path).expect("saved");

        let mut strict_cfg = TriageConfig::default();
        strict_cfg.max_store_bytes = 16 * 1024;
        assert!(TriageStore::load_from_path_with_config(&store_path, &strict_cfg).is_ok());

        // Artificially small config to trigger size limit error
        strict_cfg.max_store_bytes = 10;
        assert!(TriageStore::load_from_path_with_config(&store_path, &strict_cfg).is_err());
    }

    #[test]
    fn test_store_load_or_recover() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("corrupt_store.json");
        fs::write(&store_path, "{ invalid json corrupt content").expect("wrote corrupt file");

        let (recovered_store, err_msg) = TriageStore::load_or_recover(&store_path);
        assert_eq!(recovered_store.len(), 0);
        assert!(err_msg.is_some());
        assert!(err_msg.unwrap().contains("Recovered from error"));
    }
}
