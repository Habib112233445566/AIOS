# T-02128: CLI Surface Hardening — PEP Decision Engine

## Overview
- **Task ID**: `T-02128`
- **Sub-Epic**: 3 (CLI Surface)
- **Status**: Completed

## Hardening Safeguards Implemented

1. **Size Caps & Length Bounds**:
   - String length caps strictly enforced:
     - Store path: $\le 1024$ bytes.
     - Resource URI: $\le 1024$ bytes.
     - Subject identifier: $\le 256$ bytes.
     - Action identifier: $\le 64$ bytes.
     - Rule ID: $\le 128$ bytes.
   - Rule capacity capped at `MAX_RULES_IN_SERVICE = 5000`.

2. **Standard Result Envelope Enforcement**:
   - In `--json` mode, all outcomes are formatted as:
     `{ "code": i32, "data": Value, "error": { "code": string, "message": string } | null }`
   - Explicit exit codes:
     - `0`: Success / Permit
     - `1`: Failure / Deny / Domain error
     - `2`: Validation / CLI syntax error
   - Never silent failure: all errors are explicitly reported.

3. **Resource & Handle Cleanup**:
   - File writes are atomic: temporary files are cleaned up immediately if writing or renaming fails.
   - Audit database locks and file descriptors are cleanly released at process exit.

4. **Fail-Closed & Audit Integrity**:
   - Missing rules or unmatched requests strictly evaluate to `Deny` (code 1).
   - Any failure emits a structured audit record to the SQLite audit ring.
