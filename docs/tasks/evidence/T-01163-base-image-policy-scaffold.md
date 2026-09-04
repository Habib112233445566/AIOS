# T-01163 — Base Image Build / Security Policy: Scaffold

**Date:** 2026-09-04
**Type:** Scaffold
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Security Policy

## 1. Scaffold Deliverables
- Created `code/aiosh-rust/aiosh-core/src/base_image_policy.rs` defining:
  - `BaseImagePolicyMode` (Enforcing, Audit, Permissive)
  - `BaseImageSecurityPolicy`
  - `BaseImagePolicyViolation`
  - `BaseImagePolicyVerdict`
  - Method stubs: `validate()`, `evaluate()`
- Registered module in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Validated clean compilation with `cargo check`.
