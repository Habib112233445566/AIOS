# T-00666 — Repository Health / automated tests: Integration

## 1. Integration Scope
Verified `tools/test_repo_health_suites.py` integrates with existing CI infrastructure and evidence systems.

## 2. Verification Output
```text
[+] H1 data model integrity
[+] H2 git tree hygiene diagnostics
[+] H3 file bounds scanner
[+] H4 security governance audit
[+] H5 CLI surface commands
[+] H6 MCP tool schemas & JSON-RPC
[+] H7 configuration schema & hardening

PASS: repo_health_suites criteria (H1..H7)

[+] E1 directory-health: found 1399 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1399 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)

[+] W1 registry 20/20 == frozen canonical order; bash delegates to ci_run.py; scripts exist
[+] W2 pass record + derived log path
[+] W3 timeout/error force exit_code=null (even when caller passes an int)
[+] W4 all invalid inputs rejected naming the field
[+] W5 atomic write + JSON round-trip, no temp leftovers
[+] W6 failed write leaves no temp files
[+] W7 corrupted registry rejected at import (duplicate suite name in SUITES: 'rust_smoke')
PASS: ci_suites unit tests (W1..W7)
```
