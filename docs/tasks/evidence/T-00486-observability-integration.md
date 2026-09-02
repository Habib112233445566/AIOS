# T-00486 — Documentation Index Control / observability: Integration

## 1. Integration Scope
This task integrates `collect_doc_index_telemetry` into the CLI (`aiosh doc check`) and MCP server (`aios.doc.check`) dispatch layers, providing real-time telemetry summaries in command outputs and audit WAL payloads.

## 2. Integrated Components
1. **CLI Surface (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
   - `aiosh doc check`: Computes `DocIndexTelemetry` via `collect_doc_index_telemetry` and attaches it to the audit event payload and JSON envelope.
2. **MCP Surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - `aios.doc.check`: Returns structured `telemetry` payload containing `total_docs_indexed`, `total_links_checked`, `broken_links_count`, and `is_healthy`.
3. **Audit Ring Invariants**:
   - Telemetry statistics are recorded in WAL SQLite audit entries for every check action.

## 3. Verification Evidence
- `cargo test --workspace` -> 123 unit tests pass in `aiosh_core` + 2 in `aiosh_mcp`.
- `python tools/test_doc_index_suites.py` -> PASS (D1..D7).
- `python code/aiosh-cli/tests/test_doc_cli_smoke.py` -> PASS.
- `python code/aiosh-mcp/tests/test_doc_mcp_smoke.py` -> PASS.
