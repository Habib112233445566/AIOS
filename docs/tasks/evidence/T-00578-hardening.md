# T-00578 — Evidence & Audit Trail / security policy: Hardening

## 1. Hardening Scope
This task verifies and documents the hardening mechanisms protecting Evidence & Audit Trail security policy enforcement against bypasses, memory exhaustion, and silent authorization errors.

## 2. Hardening Measures Implemented
- **Fail-Closed Authorization**:
  - `check_evidence_policy` defaults to denying access whenever mutating actions are attempted without an active, non-empty PEP token.
- **Bounded String Trimming**:
  - Grant tokens are processed using bounded trimming (`g.trim()`) to eliminate whitespace attacks without unbounded string cloning.
- **Honest Refusal Auditing (ADR-0035 §F-2)**:
  - When an authorization check fails, the dispatch layer writes a structured refusal row to SQLite WAL containing the exact failure reason and evaluation timestamp.
- **No Resource Leaks**:
  - Policy evaluations are pure in-memory functions with zero persistent file descriptor or memory handle retention.

## 3. Verification Output
```text
running 1 test
test evidence_service::tests::test_check_evidence_policy_enforcement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 151 filtered out; finished in 0.00s
```
