# T-00445 — Documentation Index Control / MCP/API surface: Unit Test

## 1. Unit Test Scope
This task tests the MCP/API surface of Documentation Index Control (`aiosh-mcp`) using both in-crate tests and end-to-end Python smoke tests (`test_doc_mcp_smoke.py`).

## 2. Test Cases & Coverage
1. `test_mcp_tools_list`: Asserts that `tools/list` enumerates `aios.doc.index.get`, `aios.doc.check`, and `aios.doc.search`.
2. `test_mcp_doc_index_get`: Invokes `aios.doc.index.get` and validates `ok: true` with entries catalog.
3. `test_mcp_doc_check`: Invokes `aios.doc.check` and validates `ok: true` with `is_valid: true`.
4. `test_mcp_doc_search`: Invokes `aios.doc.search` with `{"query": "task"}` and verifies returned matches.
5. `test_mcp_doc_search_missing_query_negative`: Calls `aios.doc.search` omitting `query` parameter and validates error response.

## 3. Test Execution Output
```text
PASS: aios.doc tools present in tools/list
PASS: aios.doc.index.get
PASS: aios.doc.check
PASS: aios.doc.search
PASS: aios.doc.search missing query negative test
PASS: test_doc_mcp_smoke.py
```
