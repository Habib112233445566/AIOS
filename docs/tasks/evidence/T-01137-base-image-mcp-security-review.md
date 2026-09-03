# T-01137 — Base Image Build / MCP/API Surface: Security Review

**Date:** 2026-09-03
**Type:** Security Review
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / MCP/API Surface

## 1. Security Analysis
- **Identifier Length & Character Sanitization**: Callers through MCP could submit excessively long strings or non-printable bytes as `id`. We must enforce maximum length (128 chars) and ASCII printable constraints.
- **Store Path Validation**: Custom `store_path` inputs must be length-capped (4096 chars) and handled safely via the 10 MiB bounded reader.
- **PEP Enforcement**: Every tool execution is mediated by `dispatch::recorded_call`, preserving full auditability.

## 2. Hardening Directives for T-01138
- Reject `id` inputs with length $>128$ or containing non-printable ASCII characters.
- Reject `store_path` inputs $>4096$ characters.
- Add negative test assertions in `test_mcp_image_tools`.
