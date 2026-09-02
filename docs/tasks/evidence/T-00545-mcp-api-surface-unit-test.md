# T-00545 — Evidence & Audit Trail / MCP/API surface: Unit Test

## 1. Unit Test Scope
This task tests the JSON-RPC 2.0 MCP interface for Evidence & Audit Trail (`aios.evidence.verify`, `aios.evidence.hash`, `aios.evidence.scan`), asserting tool listing, valid execution, missing file errors, missing argument errors, task filtering, and non-existent directory errors.

## 2. Test Cases & Coverage
1. `test_mcp_tools_list`:
   - Verifies `aios.evidence.verify`, `aios.evidence.hash`, and `aios.evidence.scan` are listed in `tools/list`.
2. `test_mcp_evidence_hash`:
   - Checks SHA-256 hash generation for `docs/README.md` via JSON-RPC `tools/call`.
3. `test_mcp_evidence_hash_missing_file_error`:
   - Asserts non-existent file produces `"ok": false` and `"error"` message.
4. `test_mcp_evidence_hash_missing_arg_error`:
   - Asserts missing required `file_path` argument produces `"ok": false`.
5. `test_mcp_evidence_verify`:
   - Executes evidence verification against default manifest.
6. `test_mcp_evidence_scan`:
   - Scans evidence directory returning all discovered evidence records.
7. `test_mcp_evidence_scan_filtered`:
   - Filters evidence scan by `task_id: 501`.
8. `test_mcp_evidence_scan_missing_dir_error`:
   - Asserts scanning non-existent directory returns `"ok": false`.

## 3. Test Execution Output
```text
PASS: aios.evidence tools present in tools/list
PASS: aios.evidence.hash execution
PASS: aios.evidence.hash missing file error
PASS: aios.evidence.hash missing arg error
PASS: aios.evidence.verify execution
PASS: aios.evidence.scan execution
PASS: aios.evidence.scan filtered by task
PASS: aios.evidence.scan missing dir error
All 8 evidence MCP unit and smoke tests passed successfully!
```
