# T-00830 — Regression Triage / core service: Verification & Evidence

## 1. Verification Deliverables
- Fully validated core service for Regression Triage in `aiosh-core::triage_service`.
- Automated test suites passing:
  - `tools/check_security_policy.py` (S1..S5 PASS).
  - `tools/check_task_docs.py` (C1..C6 PASS).
  - `tools/test_triage_suites.py` (T1..T2 PASS).
  - `tools/check_evidence.py` (E1..E4 PASS across 1900+ files).

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
[+] T1 triage data model integrity & failure signatures
[+] T2 triage store, CI summary ingestion & persistence

PASS: triage_suites criteria (T1..T2)
[+] E1 directory-health: found 1900 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1900 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
```
