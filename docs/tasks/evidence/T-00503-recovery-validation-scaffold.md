# T-00503 — Documentation Index Control / recovery & validation: Scaffold

## 1. Scaffold Scope
This task creates the recovery and validation function signatures in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs` and tests compilation with `#[should_panic]` test stubs.

## 2. Scaffold Implementation
- Function signatures:
  ```rust
  pub fn recover_default_doc_index_config() -> crate::doc_index_config::DocIndexConfig {
      todo!("T-00503: recover_default_doc_index_config scaffold")
  }

  pub fn validate_doc_index_catalog(
      _repo_root: &Path,
      _manifest: &DocIndexManifest,
  ) -> Result<DocIndexTelemetry, String> {
      todo!("T-00503: validate_doc_index_catalog scaffold")
  }

  pub fn reconcile_doc_index(
      _repo_root: &Path,
      _doc_paths: &[&str],
  ) -> Result<(DocIndexManifest, DocLinkValidationReport, DocIndexTelemetry), String> {
      todo!("T-00503: reconcile_doc_index scaffold")
  }
  ```

## 3. Test Verification
```text
running 1 test
test doc_index_service::tests::test_recover_default_doc_index_config_scaffold - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 128 filtered out; finished in 0.00s
```
