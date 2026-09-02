# T-00596 — Evidence & Audit Trail / documentation: Integration

## 1. Integration Scope
This task verifies that Evidence & Audit Trail documentation passes mechanical invariant checks and aligns with live CLI and MCP execution paths.

## 2. Integrated Suites Executed
1. **`tools/check_task_docs.py`**:
   - `C1 (spec-health)`: Verified all task specifications and ledger sections exist.
   - `C2 (component sections)`: Validated component structure.
   - `C3 (referenced paths)`: Confirmed all backticked in-tree paths exist.
   - `C4 (phase map)`: Verified Phase 0 integrity.
   - `C5 (index health)`: Confirmed index completeness.
   - `C6 (no volatile counts)`: Verified static invariance.
2. **`code/aiosh-cli/tests/test_evidence_cli_smoke.py`**:
   - 8/8 smoke tests passed.
3. **`code/aiosh-mcp/tests/test_evidence_mcp_smoke.py`**:
   - 8/8 smoke tests passed.

## 3. Verification Output
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
All 8 evidence CLI unit and smoke tests passed successfully!
All 8 evidence MCP unit and smoke tests passed successfully!
```
