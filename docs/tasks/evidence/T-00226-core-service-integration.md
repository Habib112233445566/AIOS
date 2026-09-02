# T-00226 — Phase 0 — Release Packaging & Backup / Core Service: Integration

## Goal
Integrate the core service of Release Packaging & Backup with the surrounding system.

## Completion Notes
1. **Python Integration (`aiosh-mcp/aiosh_mcp/release.py`)**:
   - Integrated `physical_generate_iso` securely into `generate_release`.
   - Integrated `physical_create_zip` securely into `create_backup`.
   - Ensured exact compliance with ADR-0035 (Audit Invariant) by keeping the audit row write sequence intact. 
   - Wrapped the physical function call in a `try/except` block, resolving the exact outcome string (either `"success"` or `"error"`) to commit to the `AuditRing`. 
   - Throws the error only *after* the audit has successfully completed, satisfying fail-open observability.

2. **Rust Integration (`aiosh-core/src/release.rs`)**:
   - Integrated the identically stubbed physical mechanisms into `generate_release` and `create_backup` in Rust for identical cross-substrate parity.
   - Preserved `ctx.ring.write(...)` invariant ensuring no physical action proceeds unnoticed, and any failure generates an `error` outcome audit block.

## Acceptance Criteria Verified
- [x] Feature is fully reachable through the MCP wrapper (invoking `generate_release` will now attempt physical I/O and commit the result).
- [x] Integration smoke passed (`python -m pytest tests/test_release_smoke.py` succeeds without regressions).
