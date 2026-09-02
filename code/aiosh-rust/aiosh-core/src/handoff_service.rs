//! In-memory state store and persistence engine for Agent Handoff Protocol.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::handoff::{
    validate_handoff_record, HandoffPriority, HandoffRecord,
    HandoffReport, HandoffStatus,
};

/// Default maximum allowed store file size (16 MiB).
pub const DEFAULT_MAX_HANDOFF_STORE_BYTES: usize = 16 * 1024 * 1024;

/// In-memory state store managing active and historical handoff requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffStore {
    pub records: HashMap<String, HandoffRecord>,
    pub signature_index: HashMap<String, String>,
}

impl Default for HandoffStore {
    fn default() -> Self {
        Self::new()
    }
}

impl HandoffStore {
    /// Create an empty HandoffStore.
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            signature_index: HashMap::new(),
        }
    }

    /// Return total number of handoff records in the store.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Return true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Initiate or deduplicate a handoff request.
    pub fn initiate_handoff(
        &mut self,
        sender_agent_id: impl Into<String>,
        receiver_agent_id: impl Into<String>,
        task_id: Option<u32>,
        context_summary: impl Into<String>,
        payload_json: impl Into<String>,
        priority: HandoffPriority,
    ) -> HandoffRecord {
        let candidate = HandoffRecord::new(
            sender_agent_id,
            receiver_agent_id,
            task_id,
            context_summary,
            payload_json,
            priority,
        );

        let sig = candidate.signature.clone();

        if let Some(existing_id) = self.signature_index.get(&sig) {
            if let Some(existing) = self.records.get(existing_id) {
                if existing.status == HandoffStatus::Pending || existing.status == HandoffStatus::Accepted {
                    return existing.clone();
                }
            }
        }

        self.signature_index.insert(sig, candidate.id.clone());
        self.records.insert(candidate.id.clone(), candidate.clone());
        candidate
    }

    /// Accept a pending handoff request.
    pub fn accept_handoff(&mut self, id: &str, notes: Option<&str>) -> Result<HandoffRecord, String> {
        let rec = self
            .records
            .get_mut(id)
            .ok_or_else(|| format!("Handoff record '{}' not found", id))?;

        if rec.status != HandoffStatus::Pending {
            return Err(format!(
                "Cannot accept handoff '{}' in status '{:?}' (expected Pending)",
                id, rec.status
            ));
        }

        rec.status = HandoffStatus::Accepted;
        if let Some(n) = notes {
            rec.resolution_notes = Some(n.to_string());
        }

        Ok(rec.clone())
    }

    /// Reject a pending handoff request.
    pub fn reject_handoff(&mut self, id: &str, notes: Option<&str>) -> Result<HandoffRecord, String> {
        let rec = self
            .records
            .get_mut(id)
            .ok_or_else(|| format!("Handoff record '{}' not found", id))?;

        if rec.status != HandoffStatus::Pending {
            return Err(format!(
                "Cannot reject handoff '{}' in status '{:?}' (expected Pending)",
                id, rec.status
            ));
        }

        rec.status = HandoffStatus::Rejected;
        if let Some(n) = notes {
            rec.resolution_notes = Some(n.to_string());
        }

        Ok(rec.clone())
    }

    /// Complete an accepted handoff request.
    pub fn complete_handoff(&mut self, id: &str, notes: Option<&str>) -> Result<HandoffRecord, String> {
        let rec = self
            .records
            .get_mut(id)
            .ok_or_else(|| format!("Handoff record '{}' not found", id))?;

        if rec.status != HandoffStatus::Accepted {
            return Err(format!(
                "Cannot complete handoff '{}' in status '{:?}' (expected Accepted)",
                id, rec.status
            ));
        }

        rec.status = HandoffStatus::Completed;
        if let Some(n) = notes {
            rec.resolution_notes = Some(n.to_string());
        }

        Ok(rec.clone())
    }

    /// Cancel a pending or accepted handoff request.
    pub fn cancel_handoff(&mut self, id: &str, notes: Option<&str>) -> Result<HandoffRecord, String> {
        let rec = self
            .records
            .get_mut(id)
            .ok_or_else(|| format!("Handoff record '{}' not found", id))?;

        if rec.status != HandoffStatus::Pending && rec.status != HandoffStatus::Accepted {
            return Err(format!(
                "Cannot cancel handoff '{}' in terminal status '{:?}'",
                id, rec.status
            ));
        }

        rec.status = HandoffStatus::Cancelled;
        if let Some(n) = notes {
            rec.resolution_notes = Some(n.to_string());
        }

        Ok(rec.clone())
    }

    /// Accept handoff enforcing actor authorization.
    pub fn accept_handoff_as_actor(&mut self, id: &str, actor: &str, notes: Option<&str>) -> Result<HandoffRecord, String> {
        let rec = self.records.get(id).ok_or_else(|| format!("Handoff record '{}' not found", id))?;
        rec.verify_handoff_authorization(actor, "accept")?;
        self.accept_handoff(id, notes)
    }

    /// Reject handoff enforcing actor authorization.
    pub fn reject_handoff_as_actor(&mut self, id: &str, actor: &str, notes: Option<&str>) -> Result<HandoffRecord, String> {
        let rec = self.records.get(id).ok_or_else(|| format!("Handoff record '{}' not found", id))?;
        rec.verify_handoff_authorization(actor, "reject")?;
        self.reject_handoff(id, notes)
    }

    /// Complete handoff enforcing actor authorization.
    pub fn complete_handoff_as_actor(&mut self, id: &str, actor: &str, notes: Option<&str>) -> Result<HandoffRecord, String> {
        let rec = self.records.get(id).ok_or_else(|| format!("Handoff record '{}' not found", id))?;
        rec.verify_handoff_authorization(actor, "complete")?;
        self.complete_handoff(id, notes)
    }

    /// Cancel handoff enforcing actor authorization.
    pub fn cancel_handoff_as_actor(&mut self, id: &str, actor: &str, notes: Option<&str>) -> Result<HandoffRecord, String> {
        let rec = self.records.get(id).ok_or_else(|| format!("Handoff record '{}' not found", id))?;
        rec.verify_handoff_authorization(actor, "cancel")?;
        self.cancel_handoff(id, notes)
    }

    /// Lookup a record by ID.
    pub fn get_by_id(&self, id: &str) -> Option<&HandoffRecord> {
        self.records.get(id)
    }

    /// Lookup a record by SHA-256 signature.
    pub fn get_by_signature(&self, sig: &str) -> Option<&HandoffRecord> {
        self.signature_index.get(sig).and_then(|id| self.records.get(id))
    }

    /// List all active (non-terminal) handoffs.
    pub fn list_active(&self) -> Vec<HandoffRecord> {
        self.records
            .values()
            .filter(|r| r.status == HandoffStatus::Pending || r.status == HandoffStatus::Accepted)
            .cloned()
            .collect()
    }

    /// List all records in the store.
    pub fn list_all(&self) -> Vec<HandoffRecord> {
        self.records.values().cloned().collect()
    }

    /// Compile an aggregated summary report.
    pub fn to_report(&self) -> HandoffReport {
        HandoffReport::new(self.list_all())
    }

    /// Atomically persist store to disk using a temporary file.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent dir: {}", e))?;
        }

        let serialized =
            serde_json::to_string_pretty(self).map_err(|e| format!("Failed to serialize store: {}", e))?;

        let tmp_path = path.with_extension("tmp");
        {
            let mut file = File::create(&tmp_path).map_err(|e| format!("Failed to create tmp file: {}", e))?;
            file.write_all(serialized.as_bytes())
                .map_err(|e| format!("Failed to write tmp file: {}", e))?;
            file.flush().map_err(|e| format!("Failed to flush tmp file: {}", e))?;
        }

        fs::rename(&tmp_path, path).map_err(|e| format!("Failed to atomically rename store file: {}", e))?;
        Ok(())
    }

    /// Load store from disk.
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let default_config = crate::handoff_config::HandoffConfig::default();
        Self::load_from_path_with_config(path, &default_config)
    }

    /// Load store from disk with custom configuration limits.
    pub fn load_from_path_with_config(
        path: &Path,
        config: &crate::handoff_config::HandoffConfig,
    ) -> Result<Self, String> {
        config.validate()?;
        if !path.exists() {
            return Ok(Self::new());
        }

        let meta = fs::metadata(path).map_err(|e| format!("Failed to read metadata: {}", e))?;
        if meta.len() > config.max_store_bytes as u64 {
            return Err(format!(
                "Handoff store {} exceeds max allowed size limit ({} > {} bytes)",
                path.display(),
                meta.len(),
                config.max_store_bytes
            ));
        }

        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read store file: {}", e))?;
        let store: Self =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse handoff store JSON: {}", e))?;

        for record in store.records.values() {
            validate_handoff_record(record)?;
        }

        Ok(store)
    }

    /// Load store from disk or safely recover on corruption with diagnostic error.
    pub fn load_or_recover(path: &Path) -> (Self, Option<String>) {
        let default_config = crate::handoff_config::HandoffConfig::default();
        Self::load_or_recover_with_config(path, &default_config)
    }

    /// Load store from disk with config or safely recover on corruption.
    pub fn load_or_recover_with_config(
        path: &Path,
        config: &crate::handoff_config::HandoffConfig,
    ) -> (Self, Option<String>) {
        match Self::load_from_path_with_config(path, config) {
            Ok(store) => (store, None),
            Err(err) => (
                Self::new(),
                Some(format!("Recovered from error loading {}: {}", path.display(), err)),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_store_lifecycle_flow() {
        let mut store = HandoffStore::new();
        assert!(store.is_empty());

        let rec = store.initiate_handoff(
            "operator",
            "agent_rust_dev",
            Some(924),
            "Build core handoff tests",
            "{\"task\":\"unit_test\"}",
            HandoffPriority::High,
        );
        assert_eq!(store.len(), 1);
        assert_eq!(rec.status, HandoffStatus::Pending);

        // Deduplication returns same record
        let duplicate = store.initiate_handoff(
            "operator",
            "agent_rust_dev",
            Some(924),
            "Build core handoff tests",
            "{\"task\":\"unit_test\"}",
            HandoffPriority::High,
        );
        assert_eq!(store.len(), 1);
        assert_eq!(duplicate.id, rec.id);

        // Accept
        let accepted = store.accept_handoff(&rec.id, Some("Accepted by agent")).expect("accept");
        assert_eq!(accepted.status, HandoffStatus::Accepted);
        assert_eq!(store.list_active().len(), 1);

        // Complete
        let completed = store.complete_handoff(&rec.id, Some("Done successfully")).expect("complete");
        assert_eq!(completed.status, HandoffStatus::Completed);
        assert_eq!(store.list_active().len(), 0);

        let report = store.to_report();
        assert_eq!(report.total_handoffs, 1);
        assert_eq!(report.active_handoffs, 0);
        assert_eq!(report.completed_handoffs, 1);
    }

    #[test]
    fn test_store_persistence_and_recovery() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("handoffs.json");

        let mut store = HandoffStore::new();
        store.initiate_handoff("a1", "a2", None, "ctx", "{}", HandoffPriority::Normal);
        store.save_to_path(&store_path).expect("saved");

        let loaded = HandoffStore::load_from_path(&store_path).expect("loaded");
        assert_eq!(loaded.len(), 1);

        // Test corruption recovery
        let corrupt_path = dir.path().join("corrupt.json");
        fs::write(&corrupt_path, "not json!").unwrap();

        let (recovered, warning) = HandoffStore::load_or_recover(&corrupt_path);
        assert_eq!(recovered.len(), 0);
        assert!(warning.is_some());
    }

    #[test]
    fn test_handoff_automated_edge_cases() {
        let mut store = HandoffStore::new();

        // 1. Rejection path
        let r1 = store.initiate_handoff("sender", "receiver", Some(1), "summary 1", "{}", HandoffPriority::Low);
        let rej = store.reject_handoff(&r1.id, Some("Unable to handle")).expect("reject");
        assert_eq!(rej.status, HandoffStatus::Rejected);

        // Cannot accept or complete after rejection
        assert!(store.accept_handoff(&r1.id, None).is_err());
        assert!(store.complete_handoff(&r1.id, None).is_err());
        assert!(store.cancel_handoff(&r1.id, None).is_err());

        // 2. Cancellation path
        let r2 = store.initiate_handoff("sender", "receiver", Some(2), "summary 2", "{}", HandoffPriority::Urgent);
        let canc = store.cancel_handoff(&r2.id, Some("Cancelled by sender")).expect("cancel");
        assert_eq!(canc.status, HandoffStatus::Cancelled);

        // Cannot accept or complete after cancellation
        assert!(store.accept_handoff(&r2.id, None).is_err());
        assert!(store.complete_handoff(&r2.id, None).is_err());

        // 3. Accepted cancellation
        let r3 = store.initiate_handoff("sender", "receiver", Some(3), "summary 3", "{}", HandoffPriority::Normal);
        store.accept_handoff(&r3.id, None).expect("accept");
        let canc_acc = store.cancel_handoff(&r3.id, Some("Sender revoked in progress")).expect("cancel accepted");
        assert_eq!(canc_acc.status, HandoffStatus::Cancelled);

        // 4. Batch 50 handoffs with distinct signatures
        for i in 100..150 {
            store.initiate_handoff("sender", format!("subagent_{}", i), Some(i), format!("Task #{}", i), "{}", HandoffPriority::Normal);
        }
        assert!(store.len() >= 53);
    }
}
