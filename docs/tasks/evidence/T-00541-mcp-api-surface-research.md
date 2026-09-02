# T-00541 — Evidence & Audit Trail / MCP/API surface: Research

## 1. Goal
Establish facts, constraints, and prior art for the Model Context Protocol (MCP) and JSON-RPC API surface of Evidence & Audit Trail in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Current Codebase & Architecture):
1. **MCP Server Architecture**: The Rust server in `code/aiosh-rust/aiosh-mcp/src/main.rs` serves JSON-RPC 2.0 requests over stdin/stdout, handling `initialize`, `ping`, `tools/list`, and `tools/call`.
2. **Standard Result Envelopes**: Tool calls return standardized envelopes (`content: [{"type": "text", "text": "..."}]`, `structuredContent: {"result": ...}`, `isError: bool`).
3. **Audit Integration**: All MCP tool calls route through `dispatch::recorded_call` and emit cryptographic rows to SQLite WAL.
4. **Existing Evidence Endpoints**:
   - `aios.evidence.verify`: Validates manifests and checksums.
   - `aios.evidence.hash`: Computes SHA-256 for a file on disk.

### Assumptions:
1. Adding `aios.evidence.scan` to MCP will allow AI agents to programmatically query and discover evidence artifacts for any task range without raw disk access.
2. A standalone smoke test `code/aiosh-mcp/tests/test_evidence_mcp_smoke.py` will validate JSON-RPC protocol interactions end-to-end.

## 3. Prior Art & Authoritative Sources
- **Model Context Protocol (MCP) v1.0**: JSON-RPC schema definitions for `tools/list` and `tools/call`.
- **JSON-RPC 2.0 Specification**: Request framing, response formatting, and standard error codes.
- **ADR-0035 §F-2**: Autonomous agent audit invariants and PEP grant gating.

## 4. Decisions Needed
1. **Tools to Expose in `tools/list`**:
   - `aios.evidence.verify`
   - `aios.evidence.hash`
   - `aios.evidence.scan`
2. **Smoke Test Placement**: `code/aiosh-mcp/tests/test_evidence_mcp_smoke.py`.

## 5. Next Steps
Advance to Specification (T-00542) to formalize JSON schemas, input parameters, and response structures.
