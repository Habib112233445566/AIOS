# T-01043 — Distro Selection & Justification / Configuration: Scaffold

**Date:** 2026-09-03
**Type:** Scaffold
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Configuration

## 1. Scaffold Deliverables
- Created module `code/aiosh-rust/aiosh-core/src/distro_config.rs`.
- Declared and exported `pub mod distro_config;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Defined struct `DistroConfig` with fields `store_path`, `pinned_reference_id`, `min_recommendation_score`, `weights`, and `auto_evaluate`.
- Defined struct `DistroEvaluationWeights` with binary compatibility, security, and footprint weights.
- Implemented `Default` and basic structural validation rules V1..V5.
- Cargo check passes with zero errors and zero warnings across the workspace.
