# T-00548 — Evidence & Audit Trail / MCP/API surface: Hardening

## 1. Hardening Scope
This task hardens the Model Context Protocol (MCP) server endpoints (`aios.evidence.verify`, `aios.evidence.hash`, `aios.evidence.scan`) against malformed arguments, missing directories, missing arguments, and unhandled errors.

## 2. Hardening Measures
1. **Schema & Argument Guardrails**:
   - `aios.evidence.hash` strictly checks for presence of `file_path`, returning an explicit error message when missing.
   - `aios.evidence.scan` checks `repo_path` directory existence prior to reading entries.
2. **Safe Error Propagation**:
   - All errors return structured JSON envelopes with `ok: false` and are surfaced via JSON-RPC protocol as `isError: true` with human-readable diagnostic messages.
3. **Audit Ring Invariants**:
   - Error states and semantic refusals write honest audit records to SQLite WAL.
4. **Resource Management**:
   - Directory iterators and file handles are safely dropped and bounded by 16 MiB size caps.

## 3. Test Verification
- `cargo test -p aiosh-mcp` -> PASS.
- `python code/aiosh-mcp/tests/test_evidence_mcp_smoke.py` -> 8/8 tests pass (including missing files, missing args, missing directories, and task filtering).
