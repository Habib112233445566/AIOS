# T-01042 — Distro Selection & Justification / Configuration: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Configuration

## 1. Rust Data Model Specification (`aiosh_core::distro_config`)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistroConfig {
    pub store_path: String,
    pub pinned_reference_id: String,
    pub min_recommendation_score: f64,
    pub weights: ScoreWeightProfile,
    pub auto_evaluate: bool,
}

impl Default for DistroConfig {
    fn default() -> Self {
        Self {
            store_path: "config/distros.json".into(),
            pinned_reference_id: "debian-13".into(),
            min_recommendation_score: 70.0,
            weights: ScoreWeightProfile::default(),
            auto_evaluate: true,
        }
    }
}
```

## 2. Interface Methods & Error Invariants

```rust
impl DistroConfig {
    pub fn from_env() -> Result<Self, String>;
    pub fn from_path(path: &str) -> Result<Self, String>;
    pub fn from_source(get: &dyn Fn(&str) -> Option<String>) -> Result<Self, String>;
    pub fn to_json_with_sources(&self) -> serde_json::Value;
    pub fn to_json_with_sources_from(&self, is_set: &dyn Fn(&str) -> bool) -> serde_json::Value;
    pub fn save_to_file(&self, path: &str) -> Result<(), String>;
    pub fn validate(&self) -> Result<(), String>;
}
```

## 3. Validation Rules
- **V1 (Store Path)**: `store_path` must not be empty or whitespace-only.
- **V2 (Reference ID)**: `pinned_reference_id` must not be empty.
- **V3 (Score Bound)**: `min_recommendation_score` must satisfy `0.0 <= score <= 100.0`.
- **V4 (Weights Bound)**: `weights.security`, `stability`, `footprint`, `package_availability`, `hardware_support` must all be `>= 0.0`, and their sum must be positive (`> 0.0`).
- **V5 (Size Cap)**: Input files are capped at 65,536 bytes (`take(65_536)`).

## 4. Acceptance Criteria
- [x] Full serde roundtrip with default, valid, and overridden configurations.
- [x] Provenance mapping indicates `env` vs `default` for each property.
- [x] Strict error reporting on invalid JSON or out-of-range thresholds.
