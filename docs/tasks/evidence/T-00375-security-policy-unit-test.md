# T-00375 — Dependency & Toolchain Pinning / security policy: Unit Test

## 1. Unit Test Objectives
Add focused automated tests for the Dependency & Toolchain Pinning security policy enforcement logic, asserting happy path grant verification, missing-grant failure modes, and security policy criteria.

## 2. Test Coverage & Execution
- **Module Under Test**: `code/aiosh-rust/aiosh-core/src/toolchain_service.rs`
- **Test Cases (`test_check_toolchain_policy_enforcement`)**:
  - `aios.toolchain.check`: Passes without a grant (`None`).
  - `aios.toolchain.config.get`: Passes without a grant (`None`).
  - `toolchain.show`: Passes without a grant (`None`).
  - `toolchain.check`: Passes without a grant (`None`).
  - `aios.toolchain.set` without grant: Fails with explicit error `requires an active PEP grant`.
  - `aios.toolchain.set` with empty grant: Fails.
  - `aios.toolchain.set` with valid grant token `Some("gr_12345678")`: Passes `Ok(())`.
- **Policy Invariant Suite (`tools/check_security_policy.py`)**:
  - S1..S5 criteria verified against `SECURITY.md`.

## 3. Verification Output
```text
running 1 test
test toolchain_service::tests::test_check_toolchain_policy_enforcement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 92 filtered out; finished in 0.01s

[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
```
