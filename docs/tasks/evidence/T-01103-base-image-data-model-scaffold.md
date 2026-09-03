# T-01103 — Base Image Build / Data Model: Scaffold

**Date:** 2026-09-03
**Type:** Scaffold
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Data Model

## 1. Scaffold Deliverables
- Created `code/aiosh-rust/aiosh-core/src/base_image.rs` defining `ImageFormat`, `RootfsSpec`, `KernelSpec`, and `BaseImageManifest`.
- Implemented `validate_base_image_manifest` function enforcing invariants I1..I6.
- Registered `pub mod base_image;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Checked full workspace compilation (`cargo check`).
