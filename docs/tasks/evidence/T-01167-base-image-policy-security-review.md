# T-01167 — Base Image Build / Security Policy: Security Review

**Date:** 2026-09-04
**Type:** Security Review
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Security Policy

## 1. Threat Modeling & Abuse Scenarios
- **Scenario A: Kernel Parameter Obfuscation & Bypass**:
  - Attackers might inject prohibited parameters using variations (e.g. `nokaslr=1`, `mitigations=off`, `--nokaslr`, or embedding control characters).
  - Mitigation: Tokenizer splits on whitespace and tests both exact match and prefix `prohibited=` match; control characters in parameters are detected and prohibited.
- **Scenario B: Disallowed Architecture or Filesystem Smuggling**:
  - Attackers might specify unsupported architectures or unjournaled filesystems (e.g. `vfat`, `ntfs`) to induce memory corruption or escape file permission checks.
  - Mitigation: Strict whitelist enforcement (`P5` and `P6`) with case-insensitive ASCII validation.
- **Scenario C: Insecure Remote Daemons**:
  - Malicious manifests attempting to bake `telnet` or `rsh-server` into rootfs.
  - Mitigation: Blacklist matching against `prohibited_packages` with case-insensitive comparison.
- **Scenario D: Policy State Tampering**:
  - In Enforcing mode, any violation halts execution (`allowed: false`).
  - In Audit mode, violations are logged but non-fatal.
  - All executions via CLI and MCP record immutable SHA-256 hash-chained audit events into the SQLite WAL ring.

## 2. Hardening Recommendations for T-01168
- Sanitize and strip whitespace, reject control characters in parameter definitions.
- Cap policy list lengths (prohibited packages, kernel parameters) to prevent unbounded memory allocation.
- Ensure bounded error messages in JSON envelopes.
