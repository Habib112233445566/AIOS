# T-01178 — Base Image Build / Observability: Hardening

**Date:** 2026-09-04
**Type:** Hardening
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Observability

## 1. Hardening Deliverables
- Enforced capacity boundaries on all categorical maps and collections:
  - `format_breakdown`: max 16 entries.
  - `architecture_breakdown`: max 64 entries.
  - `distro_breakdown`: max 256 entries.
  - `kernel_versions`: max 256 entries (max 128 chars each).
- Added input sanitization rejecting control characters and null bytes (`\0`) in map keys and version strings.
- Added negative unit test `test_report_hardening_bounds_and_poisoning`.
- Verified all 5 tests pass cleanly (`5 passed; 0 failed`).
