# Task T-02486 Evidence: Sandbox Documentation Integration

## Goal
Integrate the offline documentation and search index of Sandbox Enforcement into the CLI (`aiosh sandbox doc`) and MCP tool interface (`aios.sandbox.doc`).

## Implementation Details
1. **CLI Surface (`aiosh-cli/src/main.rs`)**:
   - Added subcommand `doc` to `cmd_sandbox`.
   - Supports listing all topics: `aiosh sandbox doc [--json]`.
   - Supports retrieving a specific topic: `aiosh sandbox doc <topic_id> [--json]`.
   - Supports keyword search: `aiosh sandbox doc --search <query> [--json]`.
   - Sanitizes outputs and emits classified audit events.
2. **MCP Surface (`aiosh-mcp/src/main.rs`)**:
   - Registered tool `aios.sandbox.doc` with optional `topic` and `search` properties.
   - Handled `aios.sandbox.doc` in `handle_call_tool`, routing through `dispatch::recorded_call`.
   - Returns structured JSON response envelopes with topic content, search hits, or category listings.
3. **Integration Verification**:
   - CLI test: `aiosh sandbox doc --json` returns 6 canonical topics.
   - CLI test: `aiosh sandbox doc --search landlock` returns scored result pointing to `isolation`.
   - Python test suite: `pytest code/aiosh-mcp/tests/test_sandbox_mcp.py` passes 12/12 tests end-to-end.
