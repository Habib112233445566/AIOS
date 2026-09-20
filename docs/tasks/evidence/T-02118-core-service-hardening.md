# T-02118: Core Service Hardening — PEP Decision Engine

## Overview
- **Task ID**: `T-02118`
- **Sub-Epic**: 2 (Core Service)
- **Status**: Completed

## Hardening Safeguards Implemented

1. **Capacity & Size Limits**:
   - `MAX_RULES_IN_SERVICE = 5000`: Hard limit on rule count prevents uncontrolled memory growth.
   - Resource path length capped at 1024 bytes.
   - Subject ID capped at 256 bytes; Action string capped at 64 bytes.
   - Rejection of control characters (`\0`, `\r`, `\n`) across all fields.

2. **Atomic Persistence & Temp File Cleanup**:
   - `PepDecisionService::save_to_path` uses atomic rename pattern (`.tmp.<pid>.<timestamp>`).
   - On write failure, temporary files are immediately removed (`let _ = fs::remove_file(&tmp_path)`), preventing disk leakage.
   - File permissions explicitly set to `0600` on Unix systems.

3. **Non-Destructive Quarantine Recovery**:
   - `load_or_recover` detects corrupted JSON files and moves them to `<path>.bak.<timestamp>`.
   - Never discards or overwrites corrupted policies without creating a secure backup.

4. **Structured Error Envelopes & Audit Guarantees**:
   - Every operation reports structured results through the standard AIOS envelope (`ok`, `tool`, `decision`, `error`).
   - Fail-closed behavior ensures any parsing or invariant error results in an explicit `Deny` or rejection error, never an unrecorded permissive bypass.
