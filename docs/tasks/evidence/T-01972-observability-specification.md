# Task Evidence: T-01972 (System Update / observability: Specification)

## 1. Specification Overview
This document specifies the exact contract, data structures, invariants, and serialization rules for the AIOS System Update Observability Subsystem (`code/aiosh-rust/aiosh-core/src/system_update_observability.rs`).

## 2. Invariants Enforced (UOBS1 - UOBS6)

| Invariant | Name | Description | Assertion Criteria |
|---|---|---|---|
| **UOBS1** | Slot Telemetry Integrity | Accurate reporting of dual-slot state machine. | `current_slot`, `target_slot`, `rollback_slot`, `slot_a_version`, `slot_b_version`, and success flags match `SystemSlotStatus`. |
| **UOBS2** | Progress Bounding | Progress percentage clamping. | `0 <= progress_percent <= 100`. |
| **UOBS3** | Payload Accounting | Accurate byte and artifact counting. | `staged_artifacts_count == service.staged_artifacts.len()`; `staged_payload_bytes` equals cumulative bytes on disk. |
| **UOBS4** | Telemetry Text Sanitization | Stripping control chars from text outputs. | All text fields (`last_error`, `current_version`, `target_version`, `update_id`) stripped of control characters and length $\le 256$. |
| **UOBS5** | Policy Compliance Integration | Seamless inclusion of policy evaluation. | If policy is provided, `policy_verdict`, `policy_violations_count`, and `policy_mode` reflect evaluation without altering service state. |
| **UOBS6** | Cross-Substrate Parity | JSON schema parity. | Serializes to canonical JSON matching Python and MCP telemetry schemas. |

## 3. Data Model (`SystemUpdateObservabilityReport`)
```rust
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
```

## 4. Operational Methods
- `generate(service: &SystemUpdateService, policy_opt: Option<&SystemUpdateSecurityPolicy>, timestamp: &str) -> SystemUpdateObservabilityReport`:
  Pure inspection function generating report without side effects. Computes `is_healthy` as `service.update_status.state != UpdateState::Failed && (service.slot_status.current_slot == UpdateSlot::SlotA ? service.slot_status.slot_a_successful : service.slot_status.slot_b_successful)`.
- `sanitize_telemetry_text(s: &str) -> String`:
  Removes ASCII control characters, trims whitespace, and limits length to 256 chars.
- `to_json(&self) -> Result<String, String>`:
  Canonical JSON serialization.
