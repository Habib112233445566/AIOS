# T-00473 — Documentation Index Control / security policy: Scaffold

## 1. Scaffold Scope
This task creates the security policy validation signature `check_doc_index_policy` in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs`, registers irreversible actions in `code/aiosh-rust/aiosh-core/src/pep.rs`, and tests compilation with a `#[should_panic]` test stub.

## 2. Scaffold Implementation
- `code/aiosh-rust/aiosh-core/src/doc_index_service.rs`:
  ```rust
  pub fn check_doc_index_policy(_grant: Option<&str>, _tool_name: &str) -> Result<(), String> {
      todo!("T-00473: check_doc_index_policy scaffold")
  }
  ```
- `code/aiosh-rust/aiosh-core/src/pep.rs`:
  - Added `aios.doc.set` and `doc.set` to `is_irreversible`.

## 3. Test Verification
```text
running 1 test
test doc_index_service::tests::test_check_doc_index_policy_scaffold - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 119 filtered out; finished in 0.00s
```
