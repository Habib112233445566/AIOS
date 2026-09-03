# T-01092 — Distro Selection & Justification / Recovery & Validation: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Recovery & Validation

## 1. Data Contract Specification
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistroHealthReport {
    pub healthy: bool,
    pub profile_count: usize,
    pub checked_profiles: Vec<String>,
    pub recommended_profile_valid: bool,
    pub errors: Vec<String>,
    pub evaluated_at: String,
}
```

## 2. Invariant Rules
- **V1 (Health Determinism)**: `healthy` is `true` if and only if `errors` is empty.
- **V2 (Recommended Profile Invariant)**: `recommended_profile_valid` is `true` if and only if a valid recommended profile ID is found in the registry and passes evaluation.
- **V3 (Non-Destructive Corruption Handling)**: When `recover_with_backup` encounters an unparseable or corrupted store file, the invalid file is preserved as `<path>.corrupt.<timestamp>.bak` prior to overwriting.
