# T-02133: MCP/API Surface Scaffold — PEP Decision Engine

## Overview
- **Task ID**: `T-02133`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Status**: Completed

## Scaffolding Implementation
1. **Tool Schema Registration**:
   - Registered `aios.pep.rule_add`, `aios.pep.rule_list`, `aios.pep.rule_remove`, and `aios.pep.status` in `Server::tools(&self)` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Included full JSON schema definitions with properties, types, required flags, and descriptions.
2. **Dispatch Arm Wiring**:
   - Wired dispatch arms in `Server::call_tool` routing through `dispatch::recorded_call`.
   - Guaranteed that all calls emit audit rows into the SQLite audit ring.
3. **Compilation Verification**:
   - Ran `cargo check -p aiosh-mcp`.
   - Zero compilation errors.
