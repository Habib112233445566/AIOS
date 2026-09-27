# T-02334: Audit Chain Extensions MCP/API Surface Implementation

## Overview
This task completes the implementation of the Audit Chain Extensions Model Context Protocol (MCP) tools in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
- `aios.audit.query`
- `aios.audit.inspect`
- `aios.audit.ancestry`
- `aios.audit.sign_verify`

## Implementation Verification Evidence

### 1. Tool Call: `aios.audit.query`
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"audit_id\":18994,\"count\":1,\"ok\":true,\"rows\":[{\"actor\":\"user\",\"command\":\"aiosh audit inspect ...\",\"hash\":\"cfb9e5ac33103badca61c78b2eefa61bc5fcb351a757091b7bd478979aad4e1a\",\"id\":18993,\"outcome\":\"ok\",\"tool\":\"audit.inspect\"}],\"tool\":\"aios.audit.query\"}"
      }
    ],
    "isError": false,
    "structuredContent": {
      "result": {
        "audit_id": 18994,
        "count": 1,
        "ok": true,
        "tool": "aios.audit.query"
      }
    }
  }
}
```

### 2. Tool Call: `aios.audit.inspect`
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"audit_id\":18995,\"ok\":true,\"row\":{\"actor\":\"user\",\"hash\":\"cfb9e5ac33103badca61c78b2eefa61bc5fcb351a757091b7bd478979aad4e1a\",\"id\":18993,\"outcome\":\"ok\",\"tool\":\"audit.inspect\"},\"tool\":\"aios.audit.inspect\"}"
      }
    ],
    "isError": false,
    "structuredContent": {
      "result": {
        "audit_id": 18995,
        "ok": true,
        "tool": "aios.audit.inspect"
      }
    }
  }
}
```

## Security & Architectural Invariants
- Every MCP invocation triggers `dispatch::recorded_call`, inserting an immutable C-4 audit row before executing and returning results.
- Parameterized SQLite access prevents injection or memory leakages across requests.
- Backward compatibility preserved for legacy audit ring inspection.
