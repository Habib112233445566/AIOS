# T-00424 — Documentation Index Control / core service: Implementation

## 1. Implementation Scope
This task implements the core business logic for Documentation Index Control in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs`.

## 2. Implementation Details
- **`parse_markdown_title(content: &str) -> Option<String>`**:
  - Scans lines for the first top-level `# Title` heading.
- **`parse_markdown_links(content: &str) -> Vec<String>`**:
  - Parses in-tree Markdown link destinations `[label](target.md)` while ignoring external schemes (`http://`, `https://`, `mailto:`) and anchor-only hashes.
- **`validate_doc_links(repo_root: &Path, manifest: &DocIndexManifest) -> DocLinkValidationReport`**:
  - Verifies that target links physically exist on disk and do not escape repository boundaries via normalized component checking.
- **`build_doc_index_from_paths(repo_root: &Path, doc_paths: &[&str]) -> Result<DocIndexManifest, String>`**:
  - Ingests multiple markdown files bounded by `MAX_DOC_BYTES` (16 MiB), extracts metadata and links, and returns a validated `DocIndexManifest`.

## 3. Unit Test Verification
```text
running 3 tests
test doc_index_service::tests::test_parse_markdown_title_happy ... ok
test doc_index_service::tests::test_parse_markdown_links_happy ... ok
test doc_index_service::tests::test_validate_doc_links_and_build_index ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 108 filtered out; finished in 0.03s
```
