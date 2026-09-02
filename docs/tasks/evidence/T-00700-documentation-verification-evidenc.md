# T-00700 — Repository Health / documentation: Verification & Evidence

## 1. Verification Deliverables
- Implemented `format_repo_health_summary` with bounded output and explicit detail truncation notification.
- Formatted human-readable summaries integrated into unit tests and verified cleanly in `repo_health_service::tests`.
- Operator reference manual updated in `docs/README.md` passing all C1..C6 structural invariants.
- All baseline CI and invariant checkers green.

## 2. Test Execution & Evidence Log
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
[+] E1 directory-health: found 1497 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1497 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
[+] H1 data model integrity
[+] H2 git tree hygiene diagnostics
[+] H3 file bounds scanner
[+] H4 security governance audit
[+] H5 CLI surface commands
[+] H6 MCP tool schemas & JSON-RPC
[+] H7 configuration schema & hardening

PASS: repo_health_suites criteria (H1..H7)
[+] W1 registry 20/20 == frozen canonical order; bash delegates to ci_run.py; scripts exist
[+] W2 pass record + derived log path
[+] W3 timeout/error force exit_code=null (even when caller passes an int)
[+] W4 all invalid inputs rejected naming the field
[+] W5 atomic write + JSON round-trip, no temp leftovers
[+] W6 failed write leaves no temp files
[+] W7 corrupted registry rejected at import (duplicate suite name in SUITES: 'rust_smoke')
PASS: ci_suites unit tests (W1..W7)
```
