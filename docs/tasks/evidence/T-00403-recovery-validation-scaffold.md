# T-00403 — Dependency & Toolchain Pinning / recovery & validation: Scaffold

## 1. Scaffold Scope
This task defines the data types and typed function signatures for Recovery & Validation of Dependency & Toolchain Pinning in `aiosh-core`.

## 2. Scaffold Interfaces
- **`ToolchainReconciliationReport` Struct**:
  ```rust
  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  pub struct ToolchainReconciliationReport {
      pub is_conforming: bool,
      pub rust_status: String,
      pub python_status: String,
      pub node_status: String,
      pub remediation_steps: Vec<String>,
  }
  ```
- **Function Signatures**:
  - `pub fn validate_toolchain_manifest(_path: &str) -> Result<ToolchainManifest, String>`
  - `pub fn recover_default_toolchain() -> ToolchainManifest`
  - `pub fn reconcile_toolchain(_manifest: &ToolchainManifest) -> Result<ToolchainReconciliationReport, String>`
- **Test Stubs**: Verified with `#[should_panic]` assertions for fail-loud behavior during scaffolding.

## 3. Test Output
```text
running 1 test
test toolchain_service::tests::test_validate_toolchain_manifest_scaffold - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.00s
```
