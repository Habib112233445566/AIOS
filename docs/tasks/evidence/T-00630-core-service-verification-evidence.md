# T-00630 — Repository Health / core service: Verification & Evidence

## 1. Sub-Epic Closure Summary
- **Component**: Phase 0 / Repository Health (`T-00611..T-00710`)
- **Sub-Epic 2**: `core service` (`T-00621..T-00630`)
- **Status**: 10/10 Tasks COMPLETE
- **Deliverables**:
  - `aiosh-core::repo_health_service`: Diagnostics suite containing `check_git_working_tree`, `check_file_bounds`, `check_security_governance`, and `check_repo_health`.
  - Porcelain v2 git status line parser.
  - Recursive directory scanner with 16 MiB size limit and cache directory exclusion.
  - Complete automated unit tests (11/11 passing).
  - Documentation updated in `docs/README.md`.
  - Security threat model with 0 open bypasses.

## 2. Multi-Suite Test Matrix Run Log
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.44s
     Running unittests src\lib.rs (code\aiosh-rust\target\debug\deps\aiosh_core-04423105fe4b6d57.exe)

running 11 tests
test repo_health::tests::test_repo_health_check_validation_happy ... ok
test repo_health::tests::test_repo_health_report_happy_and_status_derivation ... ok
test repo_health::tests::test_repo_health_check_validation_errors ... ok
test repo_health::tests::test_repo_health_report_json_roundtrip ... ok
test repo_health::tests::test_repo_health_report_validation_errors ... ok
test repo_health_service::tests::test_check_file_bounds_happy ... ok
test repo_health_service::tests::test_check_repo_health_orchestrator ... ok
test repo_health_service::tests::test_check_file_bounds_oversized ... ok
test repo_health_service::tests::test_check_security_governance_missing ... ok
test repo_health_service::tests::test_check_security_governance_todo_markers ... ok
test repo_health_service::tests::test_check_security_governance_valid ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 156 filtered out; finished in 0.02s

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
[+] E1 directory-health: found 1291 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1291 files bounded and valid UTF-8
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
