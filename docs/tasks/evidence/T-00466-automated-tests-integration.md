# T-00466 — Documentation Index Control / automated tests: Integration

## 1. Integration Scope
This task integrates the Documentation Index Control automated test suites (`doc_cli_smoke`, `doc_mcp_smoke`, `doc_index_suites`) into the master CI suite registry in `tools/ci_suites.py` and verifies full end-to-end execution.

## 2. Integrated Components
1. **Registry Integration (`tools/ci_suites.py`)**:
   - `doc_cli_smoke`: Runs `code/aiosh-cli/tests/test_doc_cli_smoke.py`
   - `doc_mcp_smoke`: Runs `code/aiosh-mcp/tests/test_doc_mcp_smoke.py`
   - `doc_index_suites`: Runs `tools/test_doc_index_suites.py` (criteria D1..D7)
   - Suite count updated from 22 to 25 preserving deterministic sequential order.
2. **Registry Verification (`tools/test_ci_suites.py`)**:
   - Updated `CANONICAL_ORDER` and suite count assertion to 25.
   - Verified that all scripts exist on disk and meet non-zero timeout invariants.

## 3. Verification & Execution Evidence
- `python tools/test_doc_index_suites.py` -> PASS (D1..D7)
- `python tools/test_doc_index_unit.py` -> PASS (U01..U13, S01)
- `python code/aiosh-cli/tests/test_doc_cli_smoke.py` -> PASS
- `python code/aiosh-mcp/tests/test_doc_mcp_smoke.py` -> PASS
- `python tools/test_ci_suites.py` -> PASS (W1..W7)
- `python tools/test_ci_service.py` -> PASS (X1..X7)
- `python tools/test_ci_config.py` -> PASS
- `python tools/test_ci_run.py` -> PASS
