# T-01168 — Base Image Build / Security Policy: Hardening

**Date:** 2026-09-04
**Type:** Hardening
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Security Policy

## 1. Hardening Measures Implemented
- **Control Character & Input Poisoning Check**:
  - Implemented `P0_MALFORMED_INPUT` in `evaluate()` to reject kernel command lines or package names containing control characters or null bytes.
- **Resource Limits & Size Bounds**:
  - Enforced strict upper bounds in `validate()`:
    - `allowed_architectures`: max 64 entries, max 64 chars each.
    - `allowed_filesystems`: max 64 entries, max 64 chars each.
    - `prohibited_packages`: max 1024 entries, max 128 chars each.
    - `prohibited_kernel_params`: max 1024 entries, max 256 chars each.
- **Automated Validation**:
  - Added unit test `test_base_image_policy_hardening_bounds_and_poisoning`.
  - All 7 unit tests pass cleanly (`test result: ok. 7 passed; 0 failed`).
