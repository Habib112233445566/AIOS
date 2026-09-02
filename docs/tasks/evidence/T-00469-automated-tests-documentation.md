# T-00469 — Documentation Index Control / automated tests: Documentation

## 1. Documentation Scope
This task updates user and operator documentation in `docs/README.md` with usage examples and execution instructions for the Documentation Index Control automated test suites.

## 2. Documentation Updates
- Updated `docs/README.md` under the Documentation Index Control section to add the **Automated Tests** subsection:
  - `python3 tools/test_doc_index_suites.py`: Runs unified criteria test runner D1..D7.
  - `python3 code/aiosh-cli/tests/test_doc_cli_smoke.py`: Standalone CLI smoke suite.
  - `python3 code/aiosh-mcp/tests/test_doc_mcp_smoke.py`: Standalone MCP JSON-RPC smoke suite.
  - `python3 tools/test_doc_index_unit.py`: Behavioral unit tests U01..U13 with runner sensitivity verification (S01).
- Updated evidence links range.

## 3. Verification
Executed `python tools/check_task_docs.py` to confirm that all C1..C6 documentation invariants continue to pass cleanly.
