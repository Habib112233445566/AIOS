# T-00575 — Evidence & Audit Trail / security policy: Unit Test

## 1. Unit Test Scope
This task implements and verifies comprehensive unit tests for `check_evidence_policy` in `code/aiosh-rust/aiosh-core/src/evidence_service.rs`.

## 2. Test Cases & Coverage
1. **Read-Only Unauthenticated Invocations**:
   - `aios.evidence.hash`, `aios.evidence.scan`, `aios.evidence.verify`, `evidence.hash`, `evidence.scan`, `evidence.verify` pass with `None` grant token.
2. **Mutating Action Gating**:
   - `aios.evidence.record`, `evidence.record`, `aios.evidence.set`, `evidence.set` return `Err("PermissionDenied")` when grant is `None`.
3. **Whitespace & Empty Token Rejection**:
   - `Some("")` and `Some("   \t\n")` are rejected with `PermissionDenied` error.
4. **Valid PEP Token Authorization**:
   - `Some("gr_valid_token_123")` passes verification for all mutating actions.

## 3. Test Verification Output
```text
running 1 test
test evidence_service::tests::test_check_evidence_policy_enforcement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 151 filtered out; finished in 0.00s
```
