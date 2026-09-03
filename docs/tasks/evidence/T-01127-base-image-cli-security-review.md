# T-01127 — Base Image Build / CLI Surface: Security Review

**Date:** 2026-09-03
**Type:** Security Review
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / CLI Surface

## 1. Security Review Findings
- **Argument Sanitization**: Checked handling of user-supplied `<id>` in `aiosh image show` and `aiosh image plan`. Identifiers containing control characters, null bytes, or terminal escape sequences should be rejected early.
- **Audit Logging**: Verified that telemetry logs via `classify_and_emit` only record high-level image metadata and counts without leaking internal environment state.
- **Store Path Isolation**: Assessed `--store <path>` loading; validated that store loading enforces 10 MiB limit from T-01118.

## 2. Hardening Recommendations for T-01128
- Enforce strict ASCII printable check on `<id>` parameters in `cmd_image`.
- Add unit test verifying rejection of malicious/non-printable IDs.
