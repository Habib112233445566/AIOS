# Specification: Capability Model Observability (T-02072)

## 1. Scope & Purpose
The **Capability Observability** subsystem (`CAPOBS1..CAPOBS6`) provides point-in-time state aggregation, lineage depth metrics, quota consumption telemetry, resource distributions, and registry health analysis for the AIOS Security Kernel.

---

## 2. Architecture & Data Structures

### 2.1 Struct Definitions
```rust
/// Comprehensive observability and telemetry report for the Capability Model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityObservabilityReport {
    // Registry counts
    pub total_capabilities: usize,
    pub active_capabilities: usize,
    pub revoked_capabilities: usize,
    pub expired_capabilities: usize,
    pub root_capabilities: usize,
    pub attenuated_capabilities: usize,

    // Lineage and depth
    pub max_derivation_depth: usize,
    pub unique_subjects_count: usize,
    pub unique_issuers_count: usize,

    // Quota consumption
    pub total_invocations_consumed: u64,
    pub total_bytes_consumed: u64,

    // Scope and rights distributions
    pub capabilities_by_scope_type: HashMap<String, usize>,
    pub capabilities_by_right: HashMap<String, usize>,

    // Policy and capacity health
    pub policy_mode: CapabilityPolicyMode,
    pub max_capabilities_capacity: usize,
    pub capacity_utilization_percent: u8,
    pub is_healthy: bool,

    // Metadata
    pub generated_at: String,
}
```

---

## 3. Operational Specification

### 3.1 Input / Output Contract
- **Input**: Reference to active `CapabilityService` (`&CapabilityService`), optional timestamp (`&str` or RFC3339 default).
- **Output**: `CapabilityObservabilityReport` struct containing non-negative counters and distributions.
- **Fail-Safe Guarantees**:
  - Never panics even on empty registry or cyclic derivation trees.
  - Saturated arithmetic (`saturating_add`, `min(100)`) prevents overflow.
  - Text fields sanitized via `sanitize_telemetry_text`.

### 3.2 Health Assessment Logic (`is_healthy`)
The `is_healthy` flag evaluates to `true` if and only if:
1. `capacity_utilization_percent < 95`.
2. `max_derivation_depth <= service.policy().max_attenuation_depth`.
3. Registry internal indices (`by_subject`, `by_parent`) are consistent with `capabilities.len()`.

---

## 4. MCP Tool Specification

### Tool: `aios.capability.observability`
- **Description**: "Generate a comprehensive observability and telemetry report for the capability registry."
- **Parameters**:
  - `store_path` (optional string): Path to custom capability store. Defaults to `.aios/capability_store.json`.
- **Response Schema**:
  ```json
  {
    "ok": true,
    "tool": "aios.capability.observability",
    "report": {
      "total_capabilities": 12,
      "active_capabilities": 10,
      "revoked_capabilities": 2,
      "expired_capabilities": 0,
      "root_capabilities": 2,
      "attenuated_capabilities": 10,
      "max_derivation_depth": 3,
      "unique_subjects_count": 5,
      "unique_issuers_count": 1,
      "total_invocations_consumed": 45,
      "total_bytes_consumed": 1048576,
      "capabilities_by_scope_type": {
        "filesystem": 8,
        "network": 2,
        "tool": 2
      },
      "capabilities_by_right": {
        "read": 10,
        "write": 4,
        "execute": 2
      },
      "policy_mode": "enforcing",
      "max_capabilities_capacity": 10000,
      "capacity_utilization_percent": 0,
      "is_healthy": true,
      "generated_at": "2026-09-20T18:20:00Z"
    }
  }
  ```

---

## 5. Audit Effects
- Calls to `aios.capability.observability` execute through `dispatch::recorded_call`, writing an immutable row to the SHA-256 hash-chained audit ring.
