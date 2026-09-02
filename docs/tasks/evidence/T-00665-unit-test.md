# T-00665 — Repository Health / automated tests: Unit Test

## 1. Unit Test Scope
The test runner `tools/test_repo_health_suites.py` IS the automated test suite. This task validates standalone execution with exit code 0 and correct failure detection.

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
