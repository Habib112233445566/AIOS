# T-00665 — Repository Health / automated tests: Unit Test

## 1. Unit Test Scope
The test runner `tools/test_repo_health_suites.py` IS the automated test suite. This task validates that it:
1. Runs standalone with `python tools/test_repo_health_suites.py`.
2. Returns exit code 0 when all H1..H7 pass.
3. Correctly identifies failures (negative cases verified by scaffold `NotImplementedError` phase).

## 2. Standalone Execution Verification
```text
[+] H1 data model integrity
[+] H2 git tree hygiene diagnostics
[+] H3 file bounds scanner
[+] H4 security governance audit
[+] H5 CLI surface commands
[+] H6 MCP tool schemas & JSON-RPC
[+] H7 configuration schema & hardening

PASS: repo_health_suites criteria (H1..H7)
```

## 3. Negative Case Verification
During scaffold phase (T-00663), all `check_h*` functions raised `NotImplementedError`. The dispatcher correctly caught exceptions and emitted `[-]` prefixed failure lines with `FAIL:` summary.
