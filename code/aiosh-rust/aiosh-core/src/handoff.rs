//! Agent Handoff Protocol data model and validation engine.
//!
//! Provides formal, tamper-evident structures for delegating task execution context,
//! capabilities, and status between autonomous agents and human supervisors.

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::canonical::sha256_hex;

/// Maximum allowable bytes for context summary in a single handoff record.
pub const MAX_CONTEXT_SUMMARY_BYTES: usize = 4096;

/// Maximum allowable bytes for JSON payload in a single handoff record.
pub const MAX_PAYLOAD_BYTES: usize = 65536;

/// Lifecycle status of an agent handoff request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffStatus {
    Pending,
    Accepted,
    Rejected,
    Completed,
    Cancelled,
    Expired,
}

impl Default for HandoffStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Execution urgency / priority for handoff requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl Default for HandoffPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Granular handoff record representing an atomic control or context transfer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffRecord {
    pub id: String,
    pub signature: String,
    pub sender_agent_id: String,
    pub receiver_agent_id: String,
    pub task_id: Option<u32>,
    pub context_summary: String,
    pub payload_json: String,
    pub priority: HandoffPriority,
    pub status: HandoffStatus,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub resolution_notes: Option<String>,
}

impl HandoffRecord {
    /// Instantiate a new HandoffRecord with bounded parameters and deterministic signature.
    pub fn new(
        sender_agent_id: impl Into<String>,
        receiver_agent_id: impl Into<String>,
        task_id: Option<u32>,
        context_summary: impl Into<String>,
        payload_json: impl Into<String>,
        priority: HandoffPriority,
    ) -> Self {
        let sender = sender_agent_id.into();
        let receiver = receiver_agent_id.into();
        let mut summary = context_summary.into();
        let mut payload = payload_json.into();

        if summary.len() > MAX_CONTEXT_SUMMARY_BYTES {
            summary.truncate(MAX_CONTEXT_SUMMARY_BYTES);
        }
        if payload.len() > MAX_PAYLOAD_BYTES {
            payload.truncate(MAX_PAYLOAD_BYTES);
        }

        let signature = compute_handoff_signature(&sender, &receiver, task_id, &payload);
        let id = format!("HND-{}", &signature[..8]);
        let now = Utc::now().to_rfc3339();

        Self {
            id,
            signature,
            sender_agent_id: sender,
            receiver_agent_id: receiver,
            task_id,
            context_summary: summary,
            payload_json: payload,
            priority,
            status: HandoffStatus::Pending,
            created_at: now,
            expires_at: None,
            resolution_notes: None,
        }
    }

    /// Verify whether an actor is authorized to perform a specific lifecycle action.
    pub fn can_agent_act(&self, actor_id: &str, action: &str) -> bool {
        let actor = actor_id.trim().to_lowercase();
        if actor == "operator" || actor == "admin" || actor == "root" {
            return true;
        }

        match action.to_lowercase().as_str() {
            "accept" | "reject" | "complete" => {
                actor == self.receiver_agent_id.trim().to_lowercase()
            }
            "cancel" => {
                actor == self.sender_agent_id.trim().to_lowercase()
            }
            "show" | "list" | "inspect" => {
                actor == self.sender_agent_id.trim().to_lowercase()
                    || actor == self.receiver_agent_id.trim().to_lowercase()
            }
            _ => false,
        }
    }

    /// Enforce authorization, returning an explicit error if unauthorized.
    pub fn verify_handoff_authorization(&self, actor_id: &str, action: &str) -> Result<(), String> {
        if self.can_agent_act(actor_id, action) {
            Ok(())
        } else {
            Err(format!(
                "Permission denied: actor '{}' is not authorized to '{}' handoff '{}' (sender: '{}', receiver: '{}')",
                actor_id, action, self.id, self.sender_agent_id, self.receiver_agent_id
            ))
        }
    }
}

/// Compute a normalized deterministic SHA-256 signature for a handoff transfer.
pub fn compute_handoff_signature(
    sender: &str,
    receiver: &str,
    task_id: Option<u32>,
    payload: &str,
) -> String {
    let task_str = task_id.map(|t| t.to_string()).unwrap_or_default();
    let norm_sender = sender.trim().to_lowercase();
    let norm_recv = receiver.trim().to_lowercase();
    let norm_payload = payload.trim().replace("\r\n", "\n");
    let content = format!("{}->{}::{}:{}", norm_sender, norm_recv, task_str, norm_payload);
    sha256_hex(&content)
}

/// Validate structural invariants for a single HandoffRecord.
pub fn validate_handoff_record(record: &HandoffRecord) -> Result<(), String> {
    if record.id.trim().is_empty() {
        return Err("Handoff id cannot be empty".into());
    }
    if !record.id.starts_with("HND-") {
        return Err(format!("Handoff id '{}' must start with 'HND-'", record.id));
    }
    if record.signature.len() != 64 {
        return Err(format!(
            "Signature length ({}) must be exactly 64 hex characters",
            record.signature.len()
        ));
    }
    if record.sender_agent_id.trim().is_empty() {
        return Err("Sender agent id cannot be empty".into());
    }
    if record.receiver_agent_id.trim().is_empty() {
        return Err("Receiver agent id cannot be empty".into());
    }

    Ok(())
}

/// Aggregated report of handoff records and status distributions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffReport {
    pub timestamp_utc: String,
    pub total_handoffs: u32,
    pub active_handoffs: u32,
    pub completed_handoffs: u32,
    pub records: Vec<HandoffRecord>,
}

impl HandoffReport {
    pub fn new(records: Vec<HandoffRecord>) -> Self {
        let mut active_handoffs = 0;
        let mut completed_handoffs = 0;

        for r in &records {
            match r.status {
                HandoffStatus::Completed | HandoffStatus::Rejected | HandoffStatus::Cancelled | HandoffStatus::Expired => {
                    completed_handoffs += 1;
                }
                _ => active_handoffs += 1,
            }
        }

        let total_handoffs = records.len() as u32;

        Self {
            timestamp_utc: Utc::now().to_rfc3339(),
            total_handoffs,
            active_handoffs,
            completed_handoffs,
            records,
        }
    }
}

/// Validate structural and mathematical invariants for a HandoffReport.
pub fn validate_handoff_report(report: &HandoffReport) -> Result<(), String> {
    if report.total_handoffs != report.records.len() as u32 {
        return Err(format!(
            "Total handoffs ({}) does not match records count ({})",
            report.total_handoffs,
            report.records.len()
        ));
    }

    if report.active_handoffs + report.completed_handoffs != report.total_handoffs {
        return Err(format!(
            "Active ({}) + completed ({}) != total ({})",
            report.active_handoffs, report.completed_handoffs, report.total_handoffs
        ));
    }

    for rec in &report.records {
        validate_handoff_record(rec)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_handoff_signature_deterministic() {
        let sig1 = compute_handoff_signature("agent-alpha", "agent-beta", Some(101), "{\"action\":\"build\"}");
        let sig2 = compute_handoff_signature("agent-alpha", "agent-beta", Some(101), "{\"action\":\"build\"}\r\n");
        assert_eq!(sig1, sig2);
        assert_eq!(sig1.len(), 64);
    }

    #[test]
    fn test_handoff_record_creation_and_validation() {
        let rec = HandoffRecord::new(
            "agent_a",
            "agent_b",
            Some(42),
            "Execute compilation check",
            "{\"step\":\"compile\"}",
            HandoffPriority::High,
        );
        assert!(rec.id.starts_with("HND-"));
        assert_eq!(rec.status, HandoffStatus::Pending);
        assert_eq!(rec.priority, HandoffPriority::High);
        assert!(validate_handoff_record(&rec).is_ok());

        let mut invalid_rec = rec.clone();
        invalid_rec.sender_agent_id = "".into();
        assert!(validate_handoff_record(&invalid_rec).is_err());
    }

    #[test]
    fn test_handoff_report_validation_and_serde() {
        let rec1 = HandoffRecord::new("a1", "a2", None, "c1", "p1", HandoffPriority::Normal);
        let mut rec2 = HandoffRecord::new("a2", "a3", Some(10), "c2", "p2", HandoffPriority::Urgent);
        rec2.status = HandoffStatus::Completed;

        let report = HandoffReport::new(vec![rec1, rec2]);
        assert_eq!(report.total_handoffs, 2);
        assert_eq!(report.active_handoffs, 1);
        assert_eq!(report.completed_handoffs, 1);
        assert!(validate_handoff_report(&report).is_ok());

        let json = serde_json::to_string(&report).expect("serde serialization");
        let parsed: HandoffReport = serde_json::from_str(&json).expect("serde deserialization");
        assert_eq!(report, parsed);
    }

    #[test]
    fn test_handoff_authorization_matrix() {
        let rec = HandoffRecord::new("planner", "executor", Some(973), "run task", "{}", HandoffPriority::High);

        // Receiver can accept, reject, complete
        assert!(rec.can_agent_act("executor", "accept"));
        assert!(rec.can_agent_act("executor", "reject"));
        assert!(rec.can_agent_act("executor", "complete"));
        assert!(!rec.can_agent_act("executor", "cancel"));

        // Sender can cancel
        assert!(rec.can_agent_act("planner", "cancel"));
        assert!(!rec.can_agent_act("planner", "accept"));
        assert!(!rec.can_agent_act("planner", "reject"));

        // Operator has universal access
        assert!(rec.can_agent_act("operator", "accept"));
        assert!(rec.can_agent_act("admin", "cancel"));

        // Third-party agent is rejected
        assert!(!rec.can_agent_act("intruder", "accept"));
        assert!(rec.verify_handoff_authorization("intruder", "accept").is_err());
    }
}
