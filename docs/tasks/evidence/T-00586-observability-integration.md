# T-00586 — Evidence & Audit Trail / observability: Integration

## 1. Integration Scope
This task executes and validates end-to-end integration tests for Evidence & Audit Trail observability across CLI, MCP, and Rust integration suites.

## 2. Integrated Suites Executed
1. **`code/aiosh-cli/tests/test_evidence_cli_smoke.py`**:
   - 8/8 smoke checks testing CLI verification, hashing, scanning, and JSON telemetry payload structures.
2. **`code/aiosh-mcp/tests/test_evidence_mcp_smoke.py`**:
   - 8/8 smoke checks testing MCP tools dispatch, error handling, and structured result verification.
3. **`code/aiosh-rust/aiosh-core/tests/test_evidence_e2e.rs`**:
   - End-to-end manifest lifecycle, tampering detection, and verification reporting.

## 3. Verification Output
```text
All 8 evidence CLI unit and smoke tests passed successfully!
All 8 evidence MCP unit and smoke tests passed successfully!
test test_evidence_manifest_e2e_lifecycle ... ok
test test_evidence_query_helpers ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; finished in 0.01s
```
