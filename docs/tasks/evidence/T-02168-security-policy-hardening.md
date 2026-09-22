# Task Evidence: T-02168 (PEP Decision Engine Security Policy: Hardening)

## Overview
- **Task ID**: `T-02168`
- **Task Name**: security policy: Hardening
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 7: Security Policy Subsystem
- **Timestamp**: 2026-09-21T02:45:35+05:00
- **Status**: COMPLETED

## Hardening Controls Implemented

### 1. File & Memory Bounds
- Enforced strict upper bounds across all fields of `PepSecurityPolicy`:
  - `MAX_PEP_POLICY_VERSION_LEN = 32` chars.
  - `MAX_PEP_POLICY_DESC_LEN = 512` chars.
  - `MAX_RESTRICTED_PREFIXES = 64` prefixes.
  - `MAX_PREFIX_LEN = 128` chars per prefix.
  - `MAX_PEP_SECURITY_POLICY_BYTES = 65,536` bytes (64 KiB cap on disk policy reads).

### 2. Path Hygiene & Symlink Rejection
- `validate_policy_path()` enforces:
  - Length bounded within $[1, 1024]$ bytes.
  - Absence of parent directory traversal (`..`).
  - Absence of control characters.
  - Mandatory `.json` extension.
- `fs::symlink_metadata()` checks strictly reject symlinks on both read and write paths (`PEPPOL_ERR_IO`), eliminating TOCTOU symlink redirection attacks.

### 3. Atomic File Persistence
- Policy files are written to a process-unique temporary file (`.tmp.<pid>`) and flushed before calling atomic `fs::rename()`. Prevents partially written or corrupted configuration files from lingering.

### 4. Honest Audit Reporting
- Violations of the security policy during rule additions trigger honest audit records in the SQLite audit ring:
  - In `aiosh-cli`: Recorded via `classify_and_emit` with outcome `"failure"`.
  - In `aiosh-mcp`: Recorded via `dispatch::recorded_call` with error metadata.
- Permissive mode actions preserve `effect: Deny` and inject explicit warning obligations into the audit payload.
