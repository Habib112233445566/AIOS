# T-00494 — Documentation Index Control / documentation: Implementation

## 1. Implementation Scope
This task implements `format_doc_index_summary` in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs` to generate standardized, human-readable CLI summary text from a `DocIndexManifest`.

## 2. Implementation Details
- `format_doc_index_summary(manifest: &DocIndexManifest) -> String`:
  - Formats header `AIOS Documentation Index (v<version>):\n`.
  - Iterates over manifest entries formatting `  [<section>] <title> (<path>)`.
  - Handles empty manifests gracefully with `  (no documents indexed)`.

## 3. Test Verification
```text
running 1 test
test doc_index_service::tests::test_format_doc_index_summary_happy ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 123 filtered out; finished in 0.00s
```
