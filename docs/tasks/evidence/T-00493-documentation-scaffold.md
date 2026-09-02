# T-00493 — Documentation Index Control / documentation: Scaffold

## 1. Scaffold Scope
This task creates the `format_doc_index_summary` function signature in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs` and tests compilation with a `#[should_panic]` test stub.

## 2. Scaffold Implementation
- `format_doc_index_summary` signature:
  ```rust
  pub fn format_doc_index_summary(_manifest: &DocIndexManifest) -> String {
      todo!("T-00493: format_doc_index_summary scaffold")
  }
  ```

## 3. Test Verification
```text
running 1 test
test doc_index_service::tests::test_format_doc_index_summary_scaffold - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 123 filtered out; finished in 0.02s
```
