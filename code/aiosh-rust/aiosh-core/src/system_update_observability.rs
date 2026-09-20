//! System Update Observability Subsystem (UOBS1..UOBS6).
//!
//! Synthesizes dual-slot state, staging progress, cryptographic verification status,
//! policy compliance, and health metrics into a single unified telemetry report.

use serde::{Deserialize, Serialize};

use crate::system_update::{UpdateChannel, UpdateSlot, UpdateState};
use crate::system_update_policy::{SystemUpdateSecurityPolicy, UpdatePolicyMode};
use crate::system_update_service::SystemUpdateService;

/// Sanitizes a string for safe inclusion in telemetry outputs (removes control characters, trims, limits to 256 chars).
pub fn sanitize_telemetry_text(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(256)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Comprehensive observability and telemetry report for the system update mechanism.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemUpdateObservabilityReport {
    pub current_slot: UpdateSlot,
    pub target_slot: UpdateSlot,
    pub rollback_slot: Option<UpdateSlot>,
    pub slot_a_version: String,
    pub slot_b_version: String,
    pub slot_a_successful: bool,
    pub slot_b_successful: bool,
    pub state: UpdateState,
    pub progress_percent: u8,
    pub current_version: String,
    pub target_version: Option<String>,
    pub last_error: Option<String>,
    pub update_id: Option<String>,
    pub channel: Option<UpdateChannel>,
    pub staged_artifacts_count: usize,
    pub staged_payload_bytes: u64,
    pub manifest_total_bytes: Option<u64>,
    pub policy_verdict: Option<String>,
    pub policy_violations_count: usize,
    pub policy_mode: Option<UpdatePolicyMode>,
    pub is_healthy: bool,
    pub generated_at: String,
}

impl SystemUpdateObservabilityReport {
    /// Generates an observability report from the provided update service and optional policy (UOBS1..UOBS6).
    pub fn generate(
        service: &SystemUpdateService,
        policy_opt: Option<&SystemUpdateSecurityPolicy>,
        timestamp: &str,
    ) -> Self {
        let slot_status = &service.slot_status;
        let update_status = &service.update_status;

        let current_slot = slot_status.current_slot;
        let target_slot = slot_status.target_slot;
        let rollback_slot = slot_status.rollback_slot;

        let slot_a_version = sanitize_telemetry_text(&slot_status.slot_a_version);
        let slot_b_version = sanitize_telemetry_text(&slot_status.slot_b_version);
        let slot_a_successful = slot_status.slot_a_successful;
        let slot_b_successful = slot_status.slot_b_successful;

        let state = update_status.state;
        let progress_percent = update_status.progress_percent.min(100);

        let current_version = sanitize_telemetry_text(&update_status.current_version);
        let target_version = update_status.target_version.as_deref().map(sanitize_telemetry_text);
        let last_error = update_status.last_error.as_deref().map(sanitize_telemetry_text);

        let (update_id, channel, manifest_total_bytes) = if let Some(ref m) = service.active_manifest {
            (
                Some(sanitize_telemetry_text(&m.update_id)),
                Some(m.channel),
                Some(m.total_bytes()),
            )
        } else {
            (None, None, None)
        };

        let staged_artifacts_count = service.staged_artifacts.len();
        let staged_payload_bytes: u64 = service
            .staged_artifacts
            .values()
            .filter_map(|p| std::fs::metadata(p).ok().map(|meta| meta.len()))
            .sum();

        let (policy_verdict, policy_violations_count, policy_mode) = if let Some(policy) = policy_opt {
            if let Some(ref m) = service.active_manifest {
                let report = policy.evaluate(&current_version, m);
                (Some(report.verdict), report.violations.len(), Some(policy.mode))
            } else {
                (Some("not_evaluated".to_string()), 0, Some(policy.mode))
            }
        } else {
            (None, 0, None)
        };

        let is_healthy = state != UpdateState::Failed
            && match current_slot {
                UpdateSlot::SlotA => slot_a_successful,
                UpdateSlot::SlotB => slot_b_successful,
            };

        Self {
            current_slot,
            target_slot,
            rollback_slot,
            slot_a_version,
            slot_b_version,
            slot_a_successful,
            slot_b_successful,
            state,
            progress_percent,
            current_version,
            target_version,
            last_error,
            update_id,
            channel,
            staged_artifacts_count,
            staged_payload_bytes,
            manifest_total_bytes,
            policy_verdict,
            policy_violations_count,
            policy_mode,
            is_healthy,
            generated_at: sanitize_telemetry_text(timestamp),
        }
    }

    /// Serializes report to canonical JSON string.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize observability report: {}", e))
    }
}
