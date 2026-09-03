# T-01151 — Base Image Build / Automated Tests: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Automated Tests

## 1. Automated Testing Strategy Research
- **Scope**: Validate end-to-end cohesion across all Base Image Build components (`base_image`, `base_image_service`, `base_image_config`, `aiosh image` CLI, and `aios.image.*` MCP).
- **Core Automation Pillars**:
  1. **Determinism Testing**: Ensure sequential synthesis of build plans yields identical outputs and hashes.
  2. **Boundary & Malformation Matrix**: Exhaustively test negative inputs (illegal packages, control characters, invalid SemVer, corrupted config files).
  3. **Registry Concurrency & Scale**: Test bulk registry operations with multiple synthetic manifests.
  4. **Integrated Test Suite**: Extend `tools/test_image_suites.py` with criterion `B6` covering end-to-end automated test suites.
