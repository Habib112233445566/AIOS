# T-01131 — Base Image Build / MCP/API Surface: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / MCP/API Surface

## 1. MCP Tools Specification & AI Agent Ergonomics
- **Tool Names**:
  - `aios.image.list`: Returns list of available base image manifests, optionally filtered by `format` or `distro_id`.
  - `aios.image.get`: Fetches full `BaseImageManifest` by `id`.
  - `aios.image.plan`: Generates discrete 4-stage `BuildPlan` for specified `id`.
- **Policy Enforcement Point (PEP)**:
  - All calls authenticated through PEP with resource attribution and audit logging.
- **Fail-Closed Validation**:
  - Invalid parameters or missing IDs return structured error responses with descriptive messages.
