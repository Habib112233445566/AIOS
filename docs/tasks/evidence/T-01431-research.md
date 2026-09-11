# T-01431: User Session Bootstrap - MCP/API Surface: Research

See primary artifact: [T-01431-mcp-api-surface-research.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01431-mcp-api-surface-research.md)

## Summary of Findings & Decisions Needed

1. **Protocol Standards**: JSON-RPC 2.0 over stdio adhering to the Model Context Protocol specification (Anthropic 2024-11-05), ADR-0035 §D-2, and AI_CONSTITUTION §1.4 C-1..C-3.
2. **Existing MCP Surface**: `aios.session.validate`, `aios.session.list`, `aios.session.get`, and `aios.session.action` exist in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
3. **Identified Gap**: `aios.session.create` is missing from the MCP surface, leaving agents unable to bootstrap sessions programmatically.
4. **Hardening Needed**: Parameter bounds on `limit` ($1 \le n \le 10,000$), string lengths, control character rejection, and 1 MiB spec payload ceiling.
5. **Decisions for T-01432**:
   - Specify `aios.session.create` schema, arguments, and return types.
   - Add optional `reason` parameter to `aios.session.action`.
   - Specify cross-substrate smoke test `test_session_mcp_smoke.py`.
