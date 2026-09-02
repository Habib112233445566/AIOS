# T-00475 — Documentation Index Control / security policy: Unit Test

## 1. Unit Test Scope
This task adds unit tests for `check_doc_index_policy` in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs` verifying token validation, empty/whitespace boundary conditions, and read-only unauthenticated execution.

## 2. Test Cases & Coverage
1. **Read-Only Unauthenticated Access**:
   - `aios.doc.index.get`, `aios.doc.check`, `aios.doc.search`, `doc.show`, `doc.check`, `doc.search` pass with `None` grant token.
2. **Mutating Action Without Grant Token**:
   - `aios.doc.set` and `doc.set` fail with explicit policy error when grant is `None`.
3. **Empty & Whitespace-Only Grant Tokens**:
   - `Some("")` and `Some("   \t\n")` are rejected with policy violation error.
4. **Valid PEP Token Grant**:
   - `Some("gr_12345678")` passes verification for irreversible actions.

## 3. Test Execution Output
```text
running 1 test
test doc_index_service::tests::test_check_doc_index_policy_enforcement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 119 filtered out; finished in 0.01s
```
