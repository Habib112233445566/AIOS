# T-00636 — Repository Health / CLI surface: Integration

## 1. Integration Scope
This task tests cross-substrate integration of `aiosh repo` with the local SQLite audit ring, stdout formatters, and baseline repository invariants.

## 2. Integrated Suites Executed
1. **`code/aiosh-cli/tests/test_repo_cli_smoke.py`**:
   - All 5 smoke scenarios passed.
2. **`tools/check_security_policy.py`**:
   - Criteria S1..S5 passed.
3. **`tools/check_task_docs.py`**:
   - Criteria C1..C6 passed.
4. **`tools/check_evidence.py`**:
   - Criteria E1..E4 passed across all 1,309 evidence files.
5. **`tools/test_ci_suites.py`**:
   - Criteria W1..W7 passed.

## 3. Verification Output
```text
PASS: aiosh repo health prose output
PASS: aiosh repo health --json output
PASS: aiosh repo check alias
PASS: aiosh repo health --repo custom path
PASS: aiosh repo invalid subcommand rejection

ALL REPO CLI SMOKE TESTS PASSED!
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
[+] E1 directory-health: found 1309 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1309 files bounded and valid UTF-8
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
