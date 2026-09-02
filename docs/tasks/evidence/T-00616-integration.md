# T-00616 — Repository Health / data model: Integration

## 1. Integration Scope
This task validates the integration of the `aiosh-core::repo_health` data structures into the repository's cross-substrate validation harnesses.

## 2. Integrated Suites Executed
1. **`tools/check_security_policy.py`**:
   - Criteria `S1`..`S5` passed.
2. **`tools/check_task_docs.py`**:
   - Criteria `C1`..`C6` passed.
3. **`tools/check_evidence.py`**:
   - Criteria `E1`..`E4` passed across all 1,249 evidence documents.
4. **`tools/test_ci_suites.py`**:
   - CI registry invariants `W1`..`W7` passed.

## 3. Verification Output
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
[+] E1 directory-health: found 1249 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1249 files bounded and valid UTF-8
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
