# T-00590 — Evidence & Audit Trail / observability: Verification & Evidence

## 1. Verification Goal
Verify and close the **Evidence & Audit Trail / observability** sub-epic (`T-00581..T-00590`) across all documentation, policy invariants, unit tests, and cross-substrate test suites.

## 2. Test Execution Matrix & Results

### A. Security Policy & Documentation Invariants
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
```

### B. Evidence Integrity & Behavioral Unit Tests
```text
Running Evidence Checker behavioral unit tests (T-00565)...
Summary: 15/15 passed, 0 failed.
PASS: test_check_evidence_unit (15/15 checks green)

[+] E1 directory-health: found 1171 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1171 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
```

### C. CLI & MCP Observability Smoke Tests
```text
PASS: aiosh evidence hash prose
PASS: aiosh evidence hash --json
PASS: aiosh evidence hash missing file error
PASS: aiosh evidence hash missing arg error
PASS: aiosh evidence verify --json
PASS: aiosh evidence scan --json
PASS: aiosh evidence scan filtered by task
PASS: aiosh evidence unknown subcommand error
All 8 evidence CLI unit and smoke tests passed successfully!

PASS: aios.evidence tools present in tools/list
PASS: aios.evidence.hash execution
PASS: aios.evidence.hash missing file error
PASS: aios.evidence.hash missing arg error
PASS: aios.evidence.verify execution
PASS: aios.evidence.scan execution
PASS: aios.evidence.scan filtered by task
PASS: aios.evidence.scan missing dir error
All 8 evidence MCP unit and smoke tests passed successfully!
```

### D. Rust Telemetry & End-to-End Suite
```text
running 1 test
test evidence_service::tests::test_collect_evidence_telemetry ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 152 filtered out; finished in 0.01s

running 2 tests
test test_evidence_manifest_query_and_filter_e2e ... ok
test test_evidence_full_lifecycle_e2e ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

## 3. Sub-Epic Closure
The **Evidence & Audit Trail / observability** sub-epic (`T-00581..T-00590`) is now complete and closed with zero defects or regressions.
