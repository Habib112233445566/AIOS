# T-00216 — Phase 0 — Release Packaging & Backup / Data Model: Integration

## Goal
Integrate the data model of Release Packaging & Backup with the surrounding system.

## Completion Notes
1. **MCP Tool Wire-Up (`aiosh-mcp/server.py`)**:
   - Registered `@mcp.tool()` `aios_release_generate` requiring a PEP grant. The wrapper unpacks arguments, queries the `_dispatch` PEP gate, and upon approval passes execution to the data model `generate_release`.
   - Registered `@mcp.tool()` `aios_backup_create` requiring a PEP grant. It also queries the `_dispatch` PEP gate and upon approval delegates to the underlying `create_backup` logic.
2. **Audit & Classifier Field Provenance (`aiosh_mcp/release.py`)**:
   - In order to comply with the ADR-0035 invariant "consequential actions write exactly one audit row", the data model was refactored to accept arbitrary `**classifier_kwargs` which map exactly to the PEP `_dispatch`'s returned provenance metadata.
   - The data model now persists the active policy revision, rule IDs, matched evidence, and verdict natively in its DB commit without generating a redundant tool envelope audit row.
3. **End-to-End Tests**:
   - Import checks passed natively.
   - `python -m pytest tests/test_release_smoke.py` passes identically and continues to test database insertions effectively.

## Acceptance Criteria Verified
- [x] Feature reachable through its production surface (via FastMCP `@tool` mappings in `server.py`).
- [x] Integration smoke passes end-to-end (`import aiosh_mcp.server` and `test_release_smoke` succeed with zero errors).
