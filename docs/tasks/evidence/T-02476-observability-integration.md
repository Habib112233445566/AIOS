# Task T-02476 Evidence: Sandbox Observability Integration

## Goal
Integrate the observability subsystem of Sandbox Enforcement with the surrounding system across both production surfaces:
1. CLI: `aiosh sandbox stats [--json]`
2. MCP: `aios.sandbox.stats`

## Implementation Details
1. **CLI Surface (`aiosh-cli/src/main.rs`)**:
   - Added subcommand `stats` to `cmd_sandbox`.
   - Supports `--json` flag to return structured `SandboxObservabilityReport` serialised to JSON.
   - Provides human-readable tabular output detailing profile counts, execution tallies, active policy mode, outcome breakdown, and host platform isolation primitives.
2. **MCP Surface (`aiosh-mcp/src/main.rs`)**:
   - Registered tool `aios.sandbox.stats` in tools listing.
   - Handled `aios.sandbox.stats` in `handle_call_tool`, constructing a `SandboxObservabilityReport` by querying the `SandboxService` and local audit records.
   - Emits structured telemetry safely into the tool call response text.
3. **Substrate Consistency**:
   - Both CLI and MCP surfaces consume `SandboxObservabilityReport` from `aiosh_core::sandbox_observability`.
   - Both sanitize telemetry text and guard against boundless allocation via `MAX_OUTCOME_DISTRIBUTION_ENTRIES`.

## Verification Results
- `cargo build -p aiosh-cli -p aiosh-mcp`: Clean compilation with 0 warnings, 0 errors.
- `python -m pytest code/aiosh-mcp/tests/test_sandbox_mcp.py`: 11 passed in 2.14s.
- `aiosh sandbox stats --json`: Emits valid JSON payload with health status, profile distribution, and host capabilities.
- `aiosh sandbox stats`: Emits clean, readable summary to console.
