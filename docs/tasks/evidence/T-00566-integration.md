# T-00566 — Evidence & Audit Trail / automated tests: Integration

## 1. Integration Scope
This task integrates the Evidence & Audit Trail automated tests into the centralized CI runner registry (`tools/ci_suites.py`) and updates the canonical suite order in `tools/test_ci_suites.py`.

## 2. Integrated Suites
The following four test suites were registered in `tools/ci_suites.py` at the end of the `SUITES` registry:
1. `evidence_cli_smoke`: `code/aiosh-cli/tests/test_evidence_cli_smoke.py` (8/8 CLI smoke tests)
2. `evidence_mcp_smoke`: `code/aiosh-mcp/tests/test_evidence_mcp_smoke.py` (8/8 MCP JSON-RPC smoke tests)
3. `evidence_checker`: `tools/check_evidence.py` (E1..E4 criteria validation)
4. `evidence_unit`: `tools/test_check_evidence.py` (15/15 unit and sensitivity tests)

## 3. Registry & Order Verification
- Updated `tools/test_ci_suites.py` `CANONICAL_ORDER` to pin all 29 registered suites.
- Verified `test_ci_suites.py` W1..W7 pass with exit code 0.

## 4. Verification Output
```text
[+] W1 registry 20/20 == frozen canonical order; bash delegates to ci_run.py; scripts exist
[+] W2 pass record + derived log path
[+] W3 timeout/error force exit_code=null (even when caller passes an int)
[+] W4 all invalid inputs rejected naming the field
[+] W5 atomic write + JSON round-trip, no temp leftovers
[+] W6 failed write leaves no temp files
[+] W7 corrupted registry rejected at import (duplicate suite name in SUITES: 'rust_smoke')
PASS: ci_suites unit tests (W1..W7)
```
```text
All 8 evidence CLI unit and smoke tests passed successfully!
All 8 evidence MCP unit and smoke tests passed successfully!
PASS: evidence integrity criteria (E1..E4)
PASS: test_check_evidence_unit (15/15 checks green)
```
