# T-01172 — Base Image Build / Observability: Specification

**Date:** 2026-09-04
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Observability

## 1. Data Model Specification: `BaseImageObservabilityReport`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseImageObservabilityReport {
    pub total_images: usize,
    pub format_breakdown: BTreeMap<String, usize>,
    pub architecture_breakdown: BTreeMap<String, usize>,
    pub distro_breakdown: BTreeMap<String, usize>,
    pub policy_compliant_count: usize,
    pub total_size_budget_bytes: u64,
    pub average_size_budget_bytes: u64,
    pub kernel_versions: Vec<String>,
    pub generated_at: String,
}
```

## 2. Invariant Specifications (OB1..OB5)
- **OB1 (Format Sum Invariant)**: `format_breakdown.values().sum() == total_images`.
- **OB2 (Architecture Sum Invariant)**: `architecture_breakdown.values().sum() == total_images`.
- **OB3 (Distro Sum Invariant)**: `distro_breakdown.values().sum() == total_images`.
- **OB4 (Compliance Bound Invariant)**: `policy_compliant_count <= total_images`.
- **OB5 (Size Budget Calculation Invariant)**:
  - If `total_images > 0`: `average_size_budget_bytes == total_size_budget_bytes / (total_images as u64)`.
  - If `total_images == 0`: `average_size_budget_bytes == 0` and `total_size_budget_bytes == 0`.

## 3. Operations & Interface Contracts
- `BaseImageObservabilityReport::generate(store: &ImageStore, policy_opt: Option<&BaseImageSecurityPolicy>) -> Self`
- `BaseImageObservabilityReport::validate(&self) -> Result<(), String>`
- CLI contract: `aiosh image report [--json] [--store <path>]`
- MCP contract: `aios.image.report` returning structured report JSON.
