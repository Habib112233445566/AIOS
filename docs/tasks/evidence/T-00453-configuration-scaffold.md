# T-00453 — Documentation Index Control / configuration: Scaffold

## 1. Scaffold Scope
This task creates the `DocIndexConfig` struct in `code/aiosh-rust/aiosh-core/src/doc_index_config.rs`, exports the module in `lib.rs`, and tests compilation with `#[should_panic]` test stubs.

## 2. Scaffold Implementation
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocIndexConfig {
    pub version: String,
    pub root_dirs: Vec<String>,
    pub include_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub enforce_strict_links: bool,
}

impl Default for DocIndexConfig { ... }
impl DocIndexConfig {
    pub fn from_json(_json_str: &str) -> Result<Self, String> { ... }
    pub fn from_path(_path: &Path) -> Result<Self, String> { ... }
    pub fn from_env() -> Result<Self, String> { ... }
    pub fn validate(&self) -> Result<(), String> { ... }
}
```

## 3. Test Verification
```text
running 2 tests
test doc_index_config::tests::test_doc_index_config_default_scaffold - should panic ... ok
test doc_index_config::tests::test_doc_index_config_from_json_scaffold - should panic ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 114 filtered out; finished in 0.00s
```
