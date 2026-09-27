# T-02324: Audit Chain Extensions CLI Surface Implementation

## Overview
This task completes the implementation of the Audit Chain Extensions subcommands in `aiosh-cli`:
- `aiosh audit query`
- `aiosh audit inspect`
- `aiosh audit ancestry`
- `aiosh audit sign-verify`

## Implementation Verification Evidence

### 1. `aiosh audit query --limit 1`
```json
{
  "audit_id": -1,
  "data": {
    "count": 1,
    "rows": [
      {
        "actor": "agent",
        "actor_id": "agent:mcp@aiosh-mcp",
        "args": {
          "grant_id_param": "g-nonexistent-phantom",
          "store_path": "C:\\Users\\OBSESS~1\\AppData\\Local\\Temp\\tmpposdmgat\\test_pep_grants_integration.json"
        },
        "c_flags": {
          "c1": false,
          "c2": false,
          "c3": false,
          "c4": true
        },
        "causal_links": [],
        "command": "Inspect PEP authorization grant",
        "hash": "113a2172513295bb517663045f541006c9638ad9fc012434babbd46dc10c8df8",
        "id": 18991,
        "outcome": "error",
        "tool": "aios.pep.grant.inspect",
        "ts": "2026-09-22T21:29:51.277364Z"
      }
    ]
  },
  "ok": true,
  "outcome": "ok",
  "subcommand": "audit query"
}
```

### 2. `aiosh audit inspect <hash>`
```json
{
  "audit_id": -1,
  "data": {
    "actor": "agent",
    "actor_id": "agent:mcp@aiosh-mcp",
    "hash": "113a2172513295bb517663045f541006c9638ad9fc012434babbd46dc10c8df8",
    "id": 18991,
    "outcome": "error",
    "tool": "aios.pep.grant.inspect"
  },
  "ok": true,
  "outcome": "ok",
  "subcommand": "audit inspect"
}
```

## Standards & Invariants
- Consequential invocations emit honest audit rows to SQLite WAL.
- Standard JSON envelope (`ok`, `subcommand`, `outcome`, `audit_id`, `data`) preserved across all commands.
