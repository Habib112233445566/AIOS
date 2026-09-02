# T-00606 — Evidence & Audit Trail / recovery & validation: Integration

## 1. Integration Scope
This task validates cross-substrate integration of Evidence recovery and reconciliation across CLI, MCP, and live invariant checker suites.

## 2. Integrated Suites Executed
1. **`code/aiosh-cli/tests/test_evidence_cli_smoke.py`**:
   - 8/8 smoke tests passed.
2. **`code/aiosh-mcp/tests/test_evidence_mcp_smoke.py`**:
   - 8/8 smoke tests passed.
3. **`tools/check_evidence.py`**:
   - Criteria `E1` (directory health), `E2` (ledger consistency), `E3` (file bounds & valid UTF-8), and `E4` (deterministic SHA-256) all passed.

## 3. Verification Output
```text
All 8 evidence CLI unit and smoke tests passed successfully!
All 8 evidence MCP unit and smoke tests passed successfully!
[+] E1 directory-health: found 1219 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1219 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
```
