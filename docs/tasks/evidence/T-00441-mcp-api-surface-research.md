# T-00441 — Documentation Index Control / MCP/API surface: Research

## 1. Goal
Establish facts, constraints, JSON schemas, tool definitions, and prior art for the MCP/API surface of Documentation Index Control (`aiosh-mcp`).

## 2. Facts vs. Assumptions

### Facts (Empirical Repository Context):
1. **MCP Server Architecture**:
   - `code/aiosh-rust/aiosh-mcp/src/main.rs` exposes tools compliant with the Model Context Protocol (JSON-RPC 2.0 / stdio transport).
2. **Existing MCP Tool Conventions**:
   - Tool names use dot-separated namespacing (e.g., `aios.toolchain.check`, `aios.task.status`, `aios.ci.summary`).
   - Every tool handler verifies PEP policy / grants (if destructive or state-changing), logs audit records, and returns standard JSON payload envelopes.
3. **Read-Only vs Consequential Operations**:
   - Documentation inspection and search tools (`aios.doc.index.get`, `aios.doc.search`, `aios.doc.check`) are read-only diagnostics and safe for autonomous agent consumption without requiring privilege escalation.

### Assumptions:
1. Exposing documentation indexing via MCP allows AI coding subagents and IDE assistants to programmatically navigate repository documentation, verify link consistency, and locate relevant design specs before modifying code.

## 3. Prior Art & Authoritative Specifications
- **Model Context Protocol (MCP) v1.0**: JSON-RPC schema for tool registration (`tools/list`) and invocation (`tools/call`).
- **Language Server Protocol (LSP) Document Symbols**: Structured querying of document hierarchies and references.

## 4. Proposed MCP Tools Matrix
1. `aios.doc.index.get`: Returns the full `DocIndexManifest` catalog.
2. `aios.doc.check`: Runs link verification across indexed markdown files and returns `DocLinkValidationReport`.
3. `aios.doc.search`: Searches indexed documentation entries for keyword query.

## 5. Decisions Needed
1. **Tool Namespace**: Use `aios.doc.*`.
   - *Decision*: Adopt `aios.doc.index.get`, `aios.doc.check`, and `aios.doc.search`.
2. **Schema Parameters**:
   - `aios.doc.check`: Optional `repo_path` (string).
   - `aios.doc.search`: Required `query` (string).

## 6. Next Steps
Advance to Specification (T-00442) to define the input JSON schemas, tool schemas, and response envelopes.
