# T-00641 — Repository Health / MCP/API surface: Research

## 1. Context & Prior Art
Per ADR-0035 §D-2, the Model Context Protocol (MCP) serves as the primary tool surface exposed to external agents and autonomous models. The Repository Health diagnostic routines implemented in `aiosh-core::repo_health_service` need to be exposed via MCP tools so that automated reasoning agents can evaluate codebase hygiene before proposing code edits.

## 2. MCP Tool Design & Schema Analysis

### A. Tool Identity & Semantics
- **Tool Name**: `aios.repo.health`
- **Description**: "Assess repository health, Git working tree cleanliness, file bounds, and security governance policies."
- **Read-only vs Consequential**: Read-only diagnostic tool. Does not modify disk state; requires no elevated PEP grant token.

### B. Input Schema
```json
{
  "type": "object",
  "properties": {
    "repo_path": {
      "type": "string",
      "description": "Path to the repository root directory (defaults to current working directory if omitted)."
    }
  },
  "additionalProperties": false
}
```

### C. Output Envelope
```json
{
  "ok": true,
  "tool": "aios.repo.health",
  "report": {
    "repo_path": "/workspace",
    "timestamp_utc": "2026-08-29T12:00:00Z",
    "overall_status": "Pass",
    "total_checks": 3,
    "passed_checks": 3,
    "warn_checks": 0,
    "failed_checks": 0,
    "skipped_checks": 0,
    "checks": [ ... ]
  }
}
```

## 3. Facts vs. Assumptions

| Domain | Verified Fact | Working Assumption |
| :--- | :--- | :--- |
| **MCP Dispatch** | `aiosh-mcp/src/main.rs` routes tool calls through `call_tool(&mut self, tool, args)`. | Adding `aios.repo.health` into `tool_manifest` automatically makes it discoverable in `tools/list`. |
| **PEP & Audit** | `dispatch::eval_and_run` wraps MCP tools with classification and audit logging. | Read-only inspection requires standard audit logging with tool `"aios.repo.health"`. |
| **Serialization** | `RepoHealthReport` implements `serde::Serialize` and `serde::Deserialize`. | Report can be directly embedded in the MCP result dictionary. |

## 4. Key Design Decisions for Implementation
1. Add `aios.repo.health` to `tool_manifest()` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
2. Implement tool handler in `call_tool()` using `aiosh_core::repo_health_service::check_repo_health`.
3. Add unit tests for `tools/list` schema validation and `tools/call` execution in `aiosh-mcp`.
