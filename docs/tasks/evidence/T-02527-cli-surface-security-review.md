# Security Review: T-02527 Privilege Escalation Prevention CLI Surface

- **Task**: `T-02527`
- **Sub-Epic**: Privilege Escalation Prevention / CLI surface
- **Date**: 2026-09-29
- **Status**: PASSED / APPROVED

## Executive Summary
A comprehensive security review of the Privilege Escalation Prevention CLI Surface (`aiosh privilege`) was conducted, assessing terminal safety, input validation, path hygiene, audit integrity, and elevation control mechanisms.

## Security Controls Audited
1. **Terminal Injection Protection (CWE-150)**:
   - Plain-text error and output messages pass through `sanitize_terminal()`, stripping ANSI escape sequences, cursor manipulation codes, and unprintable ASCII characters.
2. **Path Traversal & Filesystem Hygiene**:
   - The `--store` parameter is validated to reject relative path traversal (`..`), embedded control characters, and lengths exceeding 1024 bytes.
3. **Strict Parameter Gating**:
   - Tiers and capabilities are parsed via strict enum decoders (`PrivilegeLevel::parse_level` and `PrivilegeCapability::parse_capability`), preventing injection of arbitrary rights or undefined levels.
   - Actor IDs are sanitized against control characters and whitespace.
4. **Kernel Tier Isolation**:
   - Attempts to elevate to `SystemKernel` are blocked at CLI evaluation before service mutation occurs.
5. **Audit Provenance**:
   - All subcommands (`status`, `elevate`, `drop`, `revoke`, `check`, `list`) emit classified audit entries via `classify_and_emit` with actor, action, and grant correlation.

## STRIDE Threat Evaluation
| Threat | Mitigation | Status |
|---|---|---|
| **Spoofing** | Actor ID sanitized; requires authenticated session | SECURE |
| **Tampering** | Invariant checking in core engine; grant required for elevation | SECURE |
| **Repudiation** | Hash-chained audit entries emitted for every command | SECURE |
| **Information Disclosure** | Sanitized error responses; token contents masked | SECURE |
| **Denial of Service** | Strict bounds on input arguments, path length, and context counts | SECURE |
| **Elevation of Privilege** | Gating enforces monotonic elevation law and kernel lockout | SECURE |
