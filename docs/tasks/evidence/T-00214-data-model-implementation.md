# T-00214 — Phase 0 — Release Packaging & Backup / Data Model: Implementation

## Goal
Implement the minimal working behavior for the data model of Release Packaging & Backup while preserving audit invariants.

## Completion Notes
1. **Rust Implementation (`aiosh-core/src/release.rs`)**:
   - `generate_release`: Extended to take a context and compute a deterministic SHA-256 hash using the canonical JSON form of the manifest. Generates an `.iso` virtual path.
   - `create_backup`: Extended to take a context and generate a virtual `.zip` snapshot path stamped with ISO8601 time.
   - **Audit Ring Integration**: Both functions embed `AuditRowInput` writes directly into the SQLite WAL via the `AuditRing` instance, properly recording targets, arguments, and outcomes.
   - Tests were adapted to supply in-memory audit ring instances and successfully assert logic instead of expecting panics.
2. **Python Implementation (`aiosh_mcp/release.py`)**:
   - Implemented cross-substrate parity for `generate_release` and `create_backup`.
   - Used `aiosh_mcp.audit_client` methods (`open_db`, `write_audit_row`, `sha256_hex`, `canonical`) to achieve the exact same deterministic operations and log emission as the Rust core.
   - Pytest `test_release_smoke.py` updated to verify successful path formatting and hash outputs.

## Acceptance Criteria Verified
- [x] Zero build/import errors (`python -m pytest` passes cleanly, `cargo test` syntax validates).
- [x] Minimum viable implementation fulfills specification.
- [x] Audit invariants intact (exactly one row appended to the ring on execution).
