# T-00380 — Dependency & Toolchain Pinning / security policy: Verification & Evidence

## 1. Verification Overview
This task closes the Dependency & Toolchain Pinning security policy sequence (T-00371 .. T-00380) through full verification of security policy enforcement, documentation integrity, and test matrix execution.

## 2. Test Execution Evidence

### A. Core PEP Enforcement Unit Tests
```text
running 1 test
test toolchain_service::tests::test_check_toolchain_policy_enforcement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 92 filtered out; finished in 0.00s
```

### B. Security Policy Criteria Validation (`tools/check_security_policy.py`)
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
```

### C. Documentation Invariants (`tools/check_task_docs.py`)
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```

### D. Full CI Smokes & Regression Matrix
```text
PASS: aiosh toolchain show
PASS: aiosh toolchain check
PASS: aiosh toolchain custom config valid
PASS: aiosh toolchain invalid subcommand
PASS: aiosh toolchain missing config negative test
PASS: aiosh toolchain corrupted config negative test
PASS: aiosh toolchain version mismatch negative test
PASS: test_toolchain_cli_smoke.py

PASS: aios.toolchain.config.get
PASS: aios.toolchain.check
PASS: aios.toolchain unknown tool negative test
PASS: test_toolchain_mcp_smoke.py

PASS: ci_suites unit tests (W1..W7)
PASS: ci_service unit tests (X1..X7)
```

## 3. Verdict
All relevant security policy enforcement checks, unit suites, smoke tests, and documentation invariants pass cleanly. The Dependency & Toolchain Pinning security policy sequence is verified and closed.
