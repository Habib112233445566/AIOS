# T-01147 — Base Image Build / Configuration: Security Review

**Date:** 2026-09-03
**Type:** Security Review
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Configuration

## 1. Security Analysis
- **Path Traversal & Poisoning**: `build_dir` and `output_dir` must be validated to ensure no null bytes or control characters are embedded.
- **Resource Exhaustion**: Invariants CF4 (timeout max 86400s) and CF5 (artifact size max 100 GiB) prevent infinite build loops and runaway disk allocations.
- **Unbounded I/O Protection**: `from_file` uses `take(10 * 1024 * 1024 + 1)` ensuring DOS prevention against oversized config files.

## 2. Hardening Directives for T-01148
- Enforce check for null bytes and control characters in `build_dir` and `output_dir`.
- Add test assertions for poisoned paths in `base_image_config::tests`.
