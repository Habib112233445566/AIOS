# T-02272 Specification: Grant Lifecycle Observability

**Task:** Specify the exact contract for the observability of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Observability  

---

## 1. Scope & Objectives

The Grant Lifecycle Observability Subsystem (`PepGrantObservabilityReport`) defines the exact telemetry contract, state aggregation methods, health metrics, and sanitization boundaries for capability grants in AIOS.

---

## 2. Telemetry Structure Specification

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PepGrantObservabilityReport {
    /// Total count of all grants in the service.
    pub total_grants: usize,
    /// Count of grants currently in Active state.
    pub active_grants: usize,
    /// Count of grants in Suspended state.
    pub suspended_grants: usize,
    /// Count of grants in Revoked state.
    pub revoked_grants: usize,
    /// Count of grants in Expired state.
    pub expired_grants: usize,
    /// Number of top-level root grants (parent_grant_id == None).
    pub root_grants_count: usize,
    /// Number of attenuated descendant grants (parent_grant_id != None).
    pub derived_grants_count: usize,
    /// Distinct recipient subjects.
    pub unique_subjects_count: usize,
    /// Distinct issuing authorities.
    pub unique_issuers_count: usize,
    /// Breakdown of grant counts indexed by state string.
    pub grants_by_state: HashMap<String, usize>,
    /// Breakdown of grants possessing specific capability rights.
    pub grants_by_right: HashMap<String, usize>,
    /// Breakdown of grants by scope category (Filesystem, Network, Ipc, System).
    pub grants_by_scope_type: HashMap<String, usize>,
    /// Configured maximum capacity of the grant registry.
    pub capacity_limit: usize,
    /// Utilization percentage (0..=100).
    pub capacity_utilization_percent: u8,
    /// Operational health indicator: true if utilization < 90%.
    pub is_healthy: bool,
    /// RFC 3339 UTC timestamp when this report was compiled.
    pub generated_at: String,
}
```

---

## 3. Thresholds, Constants & Invariants

| Identifier | Type | Value | Definition |
|---|---|---|---|
| `PEP_GRANT_HEALTH_UTILIZATION_THRESHOLD` | `u8` | `90` | Utilization percentage at/above which health flips to false |
| `PEPOBS_GRANT_ERR_VALIDATION` | `&str` | `"PEPOBS_GRANT_ERR_VALIDATION"` | Invariant failure in telemetry calculation |

### 3.1 Report Invariants
1. `total_grants == active_grants + suspended_grants + revoked_grants + expired_grants`
2. `total_grants == root_grants_count + derived_grants_count`
3. `capacity_utilization_percent == min(100, (total_grants * 100) / capacity_limit)`
4. `is_healthy == (capacity_utilization_percent < PEP_GRANT_HEALTH_UTILIZATION_THRESHOLD)`

---

## 4. Method Signatures & Interfaces

```rust
impl PepGrantService {
    /// Generates a point-in-time observability report across the grant registry.
    pub fn generate_observability_report(&self) -> PepGrantObservabilityReport;
}

/// Sanitizes subject or identifier strings before emitting in telemetry reports.
pub fn sanitize_grant_telemetry_text(s: &str) -> String;
```

---

## 5. Acceptance Verification
- ✅ Inputs, outputs, calculations, invariants, and edge cases specified.
- ✅ Reuses `PepGrant`, `PepGrantState`, `CapabilityRight` without modifying their contracts.
- ✅ Complete and reviewable standalone spec.
