# T-01072 — Distro Selection & Justification / Observability: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Observability

## 1. Observability Data Contract
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistroObservabilityReport {
    pub total_profiles: usize,
    pub recommended_profile_id: Option<String>,
    pub production_ready_count: usize,
    pub policy_compliant_count: usize,
    pub average_overall_score: f32,
    pub average_security_score: f32,
    pub average_footprint_score: f32,
    pub average_binary_compatibility_score: f32,
    pub family_breakdown: std::collections::BTreeMap<String, usize>,
    pub architecture_breakdown: std::collections::BTreeMap<String, usize>,
    pub generated_at: String,
}
```

## 2. Invariant Validation Rules (O1..O4)
- **O1 (Family Partition Integrity)**: Sum of all counts in `family_breakdown` must equal `total_profiles`.
- **O2 (Architecture Partition Integrity)**: Sum of all counts in `architecture_breakdown` must equal `total_profiles`.
- **O3 (Cardinality Bounds)**: Both `production_ready_count` and `policy_compliant_count` must not exceed `total_profiles`.
- **O4 (Score Domain Invariant)**: All score averages must fall within `[0.0, 1.0]` and must not be `NaN`.
