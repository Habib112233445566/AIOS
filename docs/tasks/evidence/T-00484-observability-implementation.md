# T-00484 — Documentation Index Control / observability: Implementation

## 1. Implementation Scope
This task implements `collect_doc_index_telemetry` and `DocIndexTelemetry` in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs` to compute structured telemetry summaries from manifest and link validation reports.

## 2. Implementation Details
- `DocIndexTelemetry`:
  ```rust
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct DocIndexTelemetry {
      pub total_docs_indexed: usize,
      pub total_links_checked: usize,
      pub broken_links_count: usize,
      pub is_healthy: bool,
  }
  ```
- `collect_doc_index_telemetry(manifest, report)`:
  - Aggregates `total_docs_indexed` from manifest entries.
  - Computes `total_links_checked`, `broken_links_count`, and `is_healthy` from the optional validation report or manifest link sums.

## 3. Test Verification
```text
running 2 tests
test doc_index_service::tests::test_collect_doc_index_telemetry_with_broken_links ... ok
test doc_index_service::tests::test_collect_doc_index_telemetry_happy ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 120 filtered out; finished in 0.00s
```
