# Task Evidence: T-02134 (Implementation)

See [T-02134-mcp-api-surface-implementation.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02134-mcp-api-surface-implementation.md) for full details.
- MCP tools implemented in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - `aios.pep.status`
  - `aios.pep.rule_add`
  - `aios.pep.rule_list`
  - `aios.pep.rule_remove`
  - `aios.pep.evaluate` (extended with persistent store support)
- Dispatch integration: `dispatch::recorded_call` for all operations.
- Path traversal protection: `validate_pep_service_path`.
