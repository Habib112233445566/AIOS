# T-01113 — Base Image Build / Core Service: Scaffold

**Date:** 2026-09-03
**Type:** Scaffold
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Core Service

## 1. Scaffold Deliverables
- Created `code/aiosh-rust/aiosh-core/src/base_image_service.rs` defining `BuildStage`, `BuildPlan`, and `ImageStore`.
- Implemented `BuildPlan::validate()` for invariants P1..P3.
- Registered `pub mod base_image_service;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Checked full workspace compilation (`cargo check`).
