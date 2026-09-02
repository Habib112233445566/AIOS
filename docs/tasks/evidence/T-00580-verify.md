# T-00580 — Evidence & Audit Trail / security policy: Verification & Evidence

## 1. Verification Goal
Verify and close the **Evidence & Audit Trail / security policy** sub-epic (`T-00571..T-00580`) across all documentation, policy invariants, unit tests, and CI test suites.

## 2. Test Execution Matrix & Results

### A. Security Policy Checker (`tools/check_security_policy.py`)
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
```

### B. Documentation Invariants (`tools/check_task_docs.py`)
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```

### C. Evidence Invariants & Behavioral Unit Tests (`tools/test_check_evidence.py` & `tools/check_evidence.py`)
```text
Running Evidence Checker behavioral unit tests (T-00565)...
Summary: 15/15 passed, 0 failed.
PASS: test_check_evidence_unit (15/15 checks green)

[+] E1 directory-health: found 1141 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1141 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
```

### D. Rust Security Policy Unit Test
```text
running 1 test
test evidence_service::tests::test_check_evidence_policy_enforcement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 151 filtered out; finished in 0.00s
```

## 3. Sub-Epic Closure
The **Evidence & Audit Trail / security policy** sub-epic (`T-00571..T-00580`) is now complete and closed with zero defects or open bypasses.
