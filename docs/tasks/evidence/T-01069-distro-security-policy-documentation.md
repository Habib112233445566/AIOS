# T-01069 — Distro Selection & Justification / Security Policy: Documentation

**Date:** 2026-09-03
**Type:** Documentation
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Security Policy

## 1. Documentation Updates
- Updated `docs/README.md` to document the Distro Security Policy Subsystem (`aiosh-core::distro_policy`).
- Documented CLI command `aiosh distro policy [<id>] [--json] [--store <path>]`.
- Documented MCP tool `aios.distro.policy` and environment variables (`AIOSH_DISTRO_MIN_SECURITY_SCORE`, `AIOSH_DISTRO_DISALLOWED_FAMILIES`).
- Verified zero documentation rot with `python tools/check_task_docs.py` (PASS C1..C6).
