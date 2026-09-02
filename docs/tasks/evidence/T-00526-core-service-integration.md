# T-00526 — Evidence & Audit Trail / core service: Integration

## 1. Integration Scope
This task integrates the evidence service into `aiosh-cli` and `aiosh-mcp`, providing operational commands and MCP tools for computing SHA-256 hashes and verifying evidence manifests against disk state.

## 2. Integrated Surfaces
1. **CLI Commands (`aiosh evidence`)**:
   - `aiosh evidence verify [--repo <path>] [--manifest <path>] [--json]`: Verifies all records in the evidence manifest against on-disk files.
   - `aiosh evidence hash <path> [--json]`: Computes deterministic SHA-256 hash for any bounded file.
2. **MCP Tool Endpoints (`aiosh-mcp`)**:
   - `aios.evidence.verify`: Validates evidence manifest artifacts via JSON-RPC.
   - `aios.evidence.hash`: Computes SHA-256 checksum for specified target file.

## 3. Verification
- `cargo test --workspace` -> 143 unit tests in `aiosh_core` + 2 in `aiosh_mcp` pass green.
