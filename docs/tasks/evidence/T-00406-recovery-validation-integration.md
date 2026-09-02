# T-00406 — Dependency & Toolchain Pinning / recovery & validation: Integration

## 1. Integration Scope
This task integrates the toolchain recovery and validation functionality (`validate_toolchain_manifest`, `recover_default_toolchain`, `reconcile_toolchain`) with the core services and CLI/MCP dispatch surfaces.

## 2. Integration Mechanics
1. **Core Service Export (`code/aiosh-rust/aiosh-core/src/toolchain_service.rs`)**:
   - `validate_toolchain_manifest`: Provides fast offline structural validation of toolchain manifests without executing host compiler binaries.
   - `recover_default_toolchain`: Supplies safe in-memory fallback defaults when configuration files are missing or unparseable.
   - `reconcile_toolchain`: Aggregates runtime telemetry and generates status reports and remediation commands for drifted tools.
2. **Cross-Substrate Parity**:
   - Validation failures return standard error envelopes naming the problematic field or malformed JSON syntax.
   - Both CLI and MCP surfaces reflect identical diagnostic strings and error codes.

## 3. Verification
- `cargo test -p aiosh-core toolchain_service::tests` -> PASS (10/10 tests)
- `python code/aiosh-cli/tests/test_toolchain_cli_smoke.py` -> PASS (7/7 tests)
- `python code/aiosh-mcp/tests/test_toolchain_mcp_smoke.py` -> PASS (3/3 tests)
