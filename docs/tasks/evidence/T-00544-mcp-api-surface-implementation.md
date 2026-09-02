# T-00544 — Evidence & Audit Trail / MCP/API surface: Implementation

## 1. Implementation Scope
This task implements the complete JSON-RPC 2.0 MCP tool suite for Evidence & Audit Trail (`aios.evidence.verify`, `aios.evidence.hash`, `aios.evidence.scan`) in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. MCP Implementation Details
- `aios.evidence.verify`:
  - Parses optional `repo_path` and `manifest_path` parameters, validates files, and returns verification report.
- `aios.evidence.hash`:
  - Computes deterministic SHA-256 string for specified target file.
- `aios.evidence.scan`:
  - Scans `docs/tasks/evidence/` directory, filters by optional `task_id`, and returns discovered evidence records.
- Invocations are recorded in the SQLite WAL audit ring with deterministic hash chains.

## 3. Test Verification
- `cargo test -p aiosh-mcp` -> 2/2 unit tests pass.
- `python code/aiosh-mcp/tests/test_evidence_mcp_smoke.py` -> 4/4 smoke tests pass.
