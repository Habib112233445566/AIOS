# T-02433: Sandbox Enforcement MCP/API Surface Scaffold

## 1. Scaffold Overview
This document records verification evidence for task T-02433: Sandbox Enforcement MCP/API Surface Scaffold.
The MCP tool `aios.sandbox.exec` has been scaffolded and wired into `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. Interface Registration & Routing
1. **Schema Registration in `tools/list`**:
   `aios.sandbox.exec` is published with JSON Schema Draft-07 properties:
   - `command`: (string, required) Target binary path
   - `args`: (array of strings, optional) Process arguments
   - `profile`: (string, optional) Containment profile name (defaults to "standard")
   - `cwd`: (string, optional) Working directory
   - `grant_token`: (string, optional) PEP authorization capability token
2. **Dispatch Arm in `handle_tool_call`**:
   - Routes `aios.sandbox.exec` through `dispatch::recorded_call`.
   - Binds closure parameters for command execution, input path traversal verification, profile lookup, and execution response framing.

## 3. Compilation Verification
`cargo check -p aiosh-mcp` and `cargo check --workspace` compile cleanly with zero errors and zero warnings.
