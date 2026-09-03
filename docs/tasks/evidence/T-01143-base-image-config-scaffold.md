# T-01143 — Base Image Build / Configuration: Scaffold

**Date:** 2026-09-03
**Type:** Scaffold
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Configuration

## 1. Scaffold Deliverables
- Scaffolded `code/aiosh-rust/aiosh-core/src/base_image_config.rs`.
- Registered `pub mod base_image_config;` in `aiosh-core/src/lib.rs`.
- Implemented `ImageBuildConfig`, `validate` with invariants CF1..CF6, `from_env`, and `from_file`.
- Checked full workspace compilation (`cargo check`).
