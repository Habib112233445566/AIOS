# T-00346 — Dependency & Toolchain Pinning / MCP/API surface: Integration

## Integration Test Results

Sent a JSON-RPC payload simulating an MCP tool invocation for `aios.toolchain.check` to `aiosh-mcp` via standard input.

**Input Payload:**
```json
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
        "name": "aios.toolchain.check",
        "arguments": {}
    }
}
```

**Output from `aiosh-mcp`:**
```json
{
    "id": 1,
    "jsonrpc": "2.0",
    "result": {
        "content": [
            {
                "text": "{\"audit_id\":12,\"classifier_policy_revision\":\"sprint-2-rule-pack-v1\",\"message\":\"Toolchain validated successfully.\",\"ok\":true,\"tool\":\"aios.toolchain.check\"}",
                "type": "text"
            }
        ],
        "isError": false,
        "structuredContent": {
            "result": {
                "audit_id": 12,
                "classifier_policy_revision": "sprint-2-rule-pack-v1",
                "message": "Toolchain validated successfully.",
                "ok": true,
                "tool": "aios.toolchain.check"
            }
        }
    }
}
```

## Validation Points
- **JSON-RPC Format**: Valid `jsonrpc: 2.0` response.
- **Audit ID**: The tool invocation correctly triggered an audit row (ID: 12) through the `dispatch::recorded_call()` PEP gate.
- **Outcome**: The environment was successfully checked against the active manifest, returning `ok: true`.
