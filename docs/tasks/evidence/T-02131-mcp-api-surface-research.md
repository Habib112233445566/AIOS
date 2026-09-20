# T-02131: MCP/API Surface Research — PEP Decision Engine

## Overview
- **Task ID**: `T-02131`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Status**: Completed

## 1. Existing MCP Architecture in AIOS
- **Server Implementation**: `code/aiosh-rust/aiosh-mcp/src/main.rs` implements the JSON-RPC 2.0 Model Context Protocol server over stdio.
- **Registration**: Tools are defined in `Server::tools(&self)` returning a vector of JSON schema definitions.
- **Dispatch**: `Server::call_tool` routes requests to closures wrapped by `dispatch::recorded_call`.
- **Audit Integration**: `recorded_call` automatically creates an audit entry in the SQLite `AuditRing`, capturing tool name, arguments, latency, outcome, and caller identity.

## 2. Authoritative Sources & Standards
1. **Model Context Protocol (MCP) Specification (2024)**:
   - Defines tool discovery (`tools/list`) and invocation (`tools/call`).
   - Strict JSON Schema object definitions for `inputSchema`.
2. **JSON-RPC 2.0 Specification (RFC 7049 / jsonrpc.org)**:
   - Standardized request/response envelopes with `id`, `method`, `params`, `result`, and `error`.
3. **AIOS ADR-0035 §D-2**:
   - MCP is the primary agent tool surface in AIOS. Every security-sensitive capability must be accessible to agents through audited MCP tools.

## 3. Fact vs. Assumption Matrix
| Item | Classification | Description |
|---|---|---|
| MCP Server Binary | Fact | `code/aiosh-rust/aiosh-mcp` is the authoritative Rust MCP implementation. |
| Audit Wrapping | Fact | `dispatch::recorded_call` enforces audit trail creation for every tool invocation. |
| Tool Set Selection | Decision | Expose 5 tools: `aios.pep.evaluate`, `aios.pep.rule_add`, `aios.pep.rule_list`, `aios.pep.rule_remove`, `aios.pep.status`. |
| Store Synchronization | Decision | Tools use `PepDecisionService::load_or_recover` and `save_to_path` against `$AIOSH_HOME/pep_policies.json`. |

## 4. Unknowns & Resolved Decisions
- **Persistent vs. Inline Rules in `evaluate`**: If `rules` array is provided in `arguments`, evaluate against inline rules; otherwise load stored rules from `$AIOSH_HOME/pep_policies.json`.
- **Combining Algorithm Selection**: Default to `deny_overrides`. Support `permit_overrides` and `first_applicable`.
