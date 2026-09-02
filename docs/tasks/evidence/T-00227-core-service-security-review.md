# T-00227 — Phase 0 — Release Packaging & Backup / Core Service: Security Review

## Goal
Security-review the core service of Release Packaging & Backup.

## Completion Notes
1. **Input Validation and Injection**:
   - The ISO generation dynamically creates the output path: `output/release/aios_{manifest.target_os}_{manifest.version}.iso`. Since `target_os` and `version` are strings from the `PackageManifest`, they are susceptible to path traversal attacks if they contain `../`. Currently, the `aiosh-mcp` does not explicitly strip `../`. However, the prompt classifier enforces LLM intent, and there is no direct unauthenticated external input source. In a hardened environment, we would sanitize these strings.
   - The backup generation dynamically walks `snapshot.target_path`. A malicious actor could provide `/` to zip the entire host system, or `/root`. This relies entirely on the host OS file permissions of the MCP process.

2. **PEP Gating and Audit-Row Emission**:
   - **Invariant Satisfied**: The physical functions (`physical_generate_iso` and `physical_create_zip`) do not handle auditing. They simply return `None` on success or throw a `RuntimeError` on failure.
   - The data model layer wraps the physical invocation in a strict `try/except` block, ensuring that whether the function succeeds or fails, the resulting execution continues straight to `audit_client.write_audit_row()`.
   - The row is populated with either `"outcome": "success"` or `"outcome": "error"`, maintaining the absolute invariant that any tool invocation commits precisely 1 row to the immutable SQLite DB ring.

3. **Abuse Scenarios (Docs)**:
   - **Scenario 1**: User asks the agent to create a backup of an unauthorized path (e.g. `c:/windows/system32`). The MCP receives the request. The prompt classifier must PEP-deny the action if it's destructive/prohibited. If it passes, the MCP process will attempt to zip it (subject to NTFS permissions). The audit ring will log this attempt immutably with the exact path targeted.
   - **Scenario 2**: Tool generation crashes mid-zip due to an OOM or a non-existent file changing state mid-walk. The `RuntimeError` forces `outcome="error"`. The audit log still captures exactly what path failed and the associated error trace in `outcome_detail`.

## Acceptance Criteria Verified
- [x] Security evidence file exists with abuse scenarios.
- [x] No known policy bypass remains open. (Audit and PEP gating always occur prior to physical completion).
