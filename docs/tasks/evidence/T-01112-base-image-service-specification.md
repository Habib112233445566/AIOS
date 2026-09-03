# T-01112 — Base Image Build / Core Service: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Core Service

## 1. Data Contracts
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildStage {
    pub name: String,
    pub description: String,
    pub command_template: String,
    pub estimated_duration_secs: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildPlan {
    pub image_id: String,
    pub target_format: ImageFormat,
    pub stages: Vec<BuildStage>,
    pub estimated_artifact_size_bytes: u64,
    pub estimated_total_duration_secs: u32,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageStore {
    images: BTreeMap<String, BaseImageManifest>,
}
```

## 2. Invariants
- `P1 (Plan Non-Emptiness)`: Every generated `BuildPlan` must have at least 4 discrete stages (Bootstrap, Kernel, Config, Packaging).
- `P2 (Duration Invariant)`: `estimated_total_duration_secs` must equal the sum of stage durations.
- `P3 (Size Estimation Invariant)`: `estimated_artifact_size_bytes` must be $> 0$ and $\le size\_budget\_bytes$.
- `P4 (Registry Uniqueness)`: Image IDs must be unique across the registry.
