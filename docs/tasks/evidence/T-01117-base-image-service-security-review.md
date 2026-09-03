# T-01117 — Base Image Build / Core Service: Security Review

**Date:** 2026-09-03
**Type:** Security Review
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Core Service

## 1. Security Review Analysis
- **Unbounded File Reading**: `ImageStore::load_from_path` reads the entire file directly into memory using `std::fs::read`. A malicious or damaged store file could cause excessive memory consumption.
- **Command Template Injection**: Reviewed `generate_build_plan` command construction. Confirmed that all embedded variables (`architecture`, `hostname`, `packages`, `cmdline`) pass strict validation in `validate_base_image_manifest`.
- **Atomic File Writing**: Evaluated whether store saving can suffer partial writes on power failure.

## 2. Hardening Directives for T-01118
- Enforce strict 10 MiB limit on store file loading via `std::io::Read::take(10 * 1024 * 1024)`.
- Set restrictive file permissions on Unix when saving image stores.
- Add unit test asserting rejection of oversized store files.
