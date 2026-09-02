# T-00483 — Documentation Index Control / observability: Scaffold

## 1. Scaffold Scope
This task creates the `DocIndexTelemetry` data structure and typed collection signature `collect_doc_index_telemetry` in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs`, verifying compilation with a `#[should_panic]` test stub.

## 2. Scaffold Implementation
- `DocIndexTelemetry` struct:
  ```rust
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct DocIndexTelemetry {
      pub total_docs_indexed: usize,
      pub total_links_checked: usize,
      pub broken_links_count: usize,
      pub is_healthy: bool,
  }
  ```
- `collect_doc_index_telemetry` signature:
  ```rust
  pub fn collect_doc_index_telemetry(
      _manifest: &DocIndexManifest,
      _report: Option<&DocLinkValidationReport>,
  ) -> DocIndexTelemetry {
      todo!("T-00483: collect_doc_index_telemetry scaffold")
  }
  ```

## 3. Test Verification
```text
running 1 test
test doc_index_service::tests::test_collect_doc_index_telemetry_scaffold - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 120 filtered out; finished in 0.02s
```
