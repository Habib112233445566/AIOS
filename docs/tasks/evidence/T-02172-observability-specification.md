# Task Evidence: T-02172 (PEP Decision Engine Observability: Specification)

## Overview
- **Task ID**: `T-02172`
- **Task Name**: observability: Specification
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 8: Observability Subsystem
- **Timestamp**: 2026-09-21T02:48:40+05:00
- **Status**: COMPLETED

## Technical Specification: PEP Decision Observability Subsystem

### 1. Data Model
```rust
/// Comprehensive observability report for the PEP Decision Engine (PEPOBS1..PEPOBS6).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PepObservabilityReport {
    pub total_rules: usize,
    pub rules_by_effect: HashMap<String, usize>,
    pub rules_with_obligations: usize,
    pub obligations_by_type: HashMap<String, usize>,
    pub unique_subjects_count: usize,
    pub unique_resources_count: usize,
    pub unique_actions_count: usize,
    pub default_algorithm: String,
    pub enforcement_mode: String,
    pub obligation_criticality: String,
    pub restricted_prefixes_count: usize,
    pub capacity_limit: usize,
    pub capacity_utilization_percent: u8,
    pub is_healthy: bool,
    pub store_path: Option<String>,
    pub generated_at: String,
}
```

### 2. Health & Capacity Logic (`PEPOBS2`)
- `capacity_limit = MAX_RULES_IN_SERVICE` (5,000 rules).
- `capacity_utilization_percent = ((total_rules * 100) / MAX_RULES_IN_SERVICE).min(100) as u8`.
- `is_healthy = capacity_utilization_percent < 90`.
- If utilization reaches 90% or higher, `is_healthy` transitions to `false` with operational warning.

### 3. Distribution Metrics & Dimensions (`PEPOBS3`, `PEPOBS4`)
- `rules_by_effect`: Map containing keys `"permit"` and `"deny"`.
- `obligations_by_type`: Map containing counts for `"audit_log"`, `"rate_limit"`, `"redact_fields"`, `"custom"`.
- Dimension cardinality: `unique_subjects_count`, `unique_resources_count`, `unique_actions_count`.
- Governance metrics: `enforcement_mode` (`"enforcing"`, `"permissive"`, `"disabled"`), `obligation_criticality` (`"strict"`, `"best_effort"`), `restricted_prefixes_count`.

### 4. Sanitization & Text Hygiene (`PEPOBS5`)
- Function `sanitize_telemetry_text(&str) -> String` strips all ASCII control characters, trims whitespace, and truncates to 256 characters to prevent ANSI injection into terminal and log viewers.

### 5. Report Invariant Validation (`PEPOBS1`, `PEPOBS6`)
- Method `validate(&self) -> Result<(), String>` asserts:
  - `total_rules <= capacity_limit`.
  - `capacity_utilization_percent <= 100`.
  - Sum of counts in `rules_by_effect` equals `total_rules`.
  - `generated_at` is non-empty and sanitized.
