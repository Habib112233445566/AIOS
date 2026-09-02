# T-00423 — Documentation Index Control / core service: Scaffold

## 1. Scaffold Scope
This task defines the data types and typed function signatures for the Documentation Index Control core service in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs` and exports the module in `lib.rs`.

## 2. Scaffold Interfaces
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrokenDocLink {
    pub source_path: String,
    pub target_link: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocLinkValidationReport {
    pub total_links_checked: usize,
    pub broken_links: Vec<BrokenDocLink>,
    pub is_valid: bool,
}

pub fn parse_markdown_title(_content: &str) -> Option<String> { ... }
pub fn parse_markdown_links(_content: &str) -> Vec<String> { ... }
pub fn validate_doc_links(_repo_root: &Path, _manifest: &DocIndexManifest) -> DocLinkValidationReport { ... }
pub fn build_doc_index_from_paths(_repo_root: &Path, _doc_paths: &[&str]) -> Result<DocIndexManifest, String> { ... }
```

## 3. Test Verification
```text
running 2 tests
test doc_index_service::tests::test_parse_markdown_links_scaffold - should panic ... ok
test doc_index_service::tests::test_parse_markdown_title_scaffold - should panic ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 108 filtered out; finished in 0.09s
```
