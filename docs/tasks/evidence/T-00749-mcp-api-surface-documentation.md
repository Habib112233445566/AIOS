# T-00749 — Secrets & Access Hygiene / MCP/API surface: Documentation

## 1. Operator & Agent Documentation
Documented MCP tools `aios.secrets.scan` and `aios.secrets.check` in `docs/README.md` under `## Secrets & Access Hygiene (T-00711..T-00810)`.

### JSON-RPC Example
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.secrets.scan",
    "arguments": {
      "repo_path": ".",
      "max_bytes": 16777216
    }
  }
}
```

## 2. Invariant Validation
- Ran `python tools/check_task_docs.py` to confirm full compliance with documentation invariants C1..C6.
