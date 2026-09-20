# T-02038: Hardening — Capability Model MCP/API Surface

See complete hardening documentation at [T-02038-mcp-api-surface-hardening.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02038-mcp-api-surface-hardening.md).

- **Hardening Applied**:
  - String length caps and control character rejection via `validate_mcp_string`.
  - Quota and duration range validation.
  - Path traversal defense on `store_path`.
  - Honest audit row emission on all error and rejection paths.
