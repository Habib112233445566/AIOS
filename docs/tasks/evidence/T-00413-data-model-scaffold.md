# T-00413 — Documentation Index Control / data model: Scaffold

## 1. Scaffold Scope
This task creates the data model skeleton and types for Documentation Index Control in `code/aiosh-rust/aiosh-core/src/doc_index.rs` and exports the module in `lib.rs`.

## 2. Scaffold Interfaces
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocIndexEntry {
    pub path: String,
    pub title: String,
    pub section: String,
    pub task_range: Option<String>,
    pub links: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocIndexManifest {
    pub version: String,
    pub entries: Vec<DocIndexEntry>,
}

impl DocIndexManifest {
    pub fn from_json(_json_str: &str) -> Result<Self, String> { ... }
    pub fn to_json(&self) -> Result<String, String> { ... }
    pub fn find_entry_by_path(&self, _path: &str) -> Option<&DocIndexEntry> { ... }
    pub fn find_entries_by_section(&self, _section: &str) -> Vec<&DocIndexEntry> { ... }
    pub fn validate(&self) -> Result<(), String> { ... }
}
```

## 3. Test Verification
```text
running 2 tests
test doc_index::tests::test_doc_index_manifest_from_json_scaffold - should panic ... ok
test doc_index::tests::test_doc_index_manifest_to_json_scaffold - should panic ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.03s
```
