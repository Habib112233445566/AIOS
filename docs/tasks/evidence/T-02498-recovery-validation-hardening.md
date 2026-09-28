# Task T-02498 Evidence: Sandbox Recovery & Validation Hardening

## Goal
Harden Sandbox Enforcement Recovery & Validation against directory flooding, path traversal, oversized file parsing, and silent failures.

## Hardening Mechanisms Enforced
1. **Directory Flooding Defense**:
   - `MAX_SCANNED_PROFILES` (256 entries) bounds filesystem iteration in custom directories during validation and quarantine sweeps.
2. **File Size Hard Limits**:
   - `MAX_PROFILE_FILE_BYTES` (64 KiB) rejects oversized or synthetic JSON payloads prior to memory ingestion.
3. **Path Traversal Rejection**:
   - Paths containing `..` or relative parent escapes are rejected fail-closed with `SANDBOXRECV_ERR_TRAVERSAL`.
4. **Structured Error Envelopes**:
   - Result types use typed enums and standard error tags (`SANDBOXRECV_ERR_*`), preventing silent panics.
5. **Evidence Preservation**:
   - Damaged files are quarantined in timestamped directories (`.quarantine_<timestamp>`) instead of being destructively overwritten without trace.

## Verification
- Ran `cargo test -p aiosh-core --test test_sandbox_recovery`: 7/7 tests passed cleanly.
