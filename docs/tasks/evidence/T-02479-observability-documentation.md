# Task T-02479 Evidence: Sandbox Observability Documentation

## Goal
Document the observability capabilities of the Sandbox Enforcement subsystem for operators, developers, and autonomous agents.

## Delivered Documentation
- Created `docs/SPEC-SANDBOX-OBSERVABILITY.md` detailing:
  - Six foundational invariants (`SANDBOXOBS1`..`SANDBOXOBS6`).
  - Command-line invocations: `aiosh sandbox stats` and `aiosh sandbox stats --json`.
  - Model Context Protocol (MCP) invocation: `aios.sandbox.stats`.
  - JSON payload schema and example responses.
  - Operational constraints, tail bounds (`tail(1000)`), and platform limitations.
  - Links to evidence files from T-02474 through T-02478.

## Copy-Pasteable Usage Examples
```bash
# Human readable report
aiosh sandbox stats

# Machine readable JSON telemetry
aiosh sandbox stats --json
```

MCP call:
```json
{
  "name": "aios.sandbox.stats",
  "arguments": {}
}
```
