# Task Evidence: T-01479 - Session Observability Documentation

## Summary
Documents operator and programmatic interfaces for the User Session Bootstrap Observability Subsystem across CLI and MCP surfaces (`SSO1..SSO6`).

## Documentation Updates
1. **CLI Documentation (`code/aiosh-cli/README.md`)**:
   - Added `aiosh session stats` to the core session subcommands reference table.
   - Added copy-pasteable command invocations:
     ```bash
     aiosh session stats --json
     aiosh session stats --policy policy.json --store /path/to/sessions.json
     ```
   - Specified options: `--policy <path>`, `--store <path>`, `--json`.

2. **MCP Documentation (`code/aiosh-mcp/README.md`)**:
   - Added `aios.session.stats` to tool catalog table.
   - Documented tool schema with `policy_path`, `store_path`, and `grant_id`.
   - Added JSON-RPC 2.0 request example:
     ```json
     {
       "jsonrpc": "2.0",
       "id": 11,
       "method": "tools/call",
       "params": {
         "name": "aios.session.stats",
         "arguments": {
           "store_path": "/path/to/sessions.json"
         }
       }
     }
     ```
   - Added CLI cross-references for security policy and telemetry invocations.
