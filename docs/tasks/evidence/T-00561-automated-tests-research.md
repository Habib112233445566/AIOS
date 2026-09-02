# T-00561 — Evidence & Audit Trail / automated tests: Research

## 1. Goal
Establish facts, constraints, and prior art for automated end-to-end testing of Evidence & Audit Trail across all AIOS substrates.

## 2. Facts vs. Assumptions

### Facts (Empirical from Current Codebase & Architecture):
1. **Multi-Substrate Test Ecosystem**:
   - Rust unit & integration test suites in `code/aiosh-rust/aiosh-core` and `code/aiosh-rust/aiosh-mcp`.
   - Python CLI smoke tests (`code/aiosh-cli/tests/test_evidence_cli_smoke.py`).
   - Python MCP smoke tests (`code/aiosh-mcp/tests/test_evidence_mcp_smoke.py`).
2. **Deterministic Checksums**:
   - SHA-256 computation produces identical byte sequences across Rust (`sha256_hex_bytes`) and Python (`hashlib.sha256`).
3. **CI Runner Integration**:
   - CI tools (e.g. `check_task_docs.py`, `check_security_policy.py`) provide fast, stdlib-only validation in the pre-commit and CI pipelines.

### Assumptions:
1. Creating `tools/check_evidence.py` will provide automated CI verification of evidence file coverage and SHA-256 consistency.
2. An automated integration test suite in Rust (`code/aiosh-rust/aiosh-core/tests/test_evidence_e2e.rs`) will assert multi-step lifecycle consistency.

## 3. Prior Art & Authoritative Sources
- **`tools/check_task_docs.py`**: Model for deterministic repo validation scripts in AIOS.
- **in-toto verification suites**: Testing provenance chains and artifact digests.

## 4. Decisions Needed
1. Standalone Python CI validator: `tools/check_evidence.py`.
2. Rust end-to-end integration test: `code/aiosh-rust/aiosh-core/tests/test_evidence_e2e.rs`.

## 5. Next Steps
Advance to Specification (T-00562) to formalize test schemas, CI invariants, and execution criteria.
