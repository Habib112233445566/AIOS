# T-00383 — Dependency & Toolchain Pinning / observability: Scaffold

## 1. Scaffold Scope
This task defines the data types and function signatures for runtime toolchain telemetry and diagnostics extraction in `aiosh-core`.

## 2. Interface Definitions
- **Type**:
  ```rust
  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  pub struct ToolchainTelemetry {
      pub manifest: ToolchainManifest,
      pub detected_rust: Option<String>,
      pub detected_python: Option<String>,
      pub detected_node: Option<String>,
      pub check_passed: bool,
  }
  ```
- **Function**:
  ```rust
  pub fn collect_toolchain_telemetry(_manifest: &ToolchainManifest) -> Result<ToolchainTelemetry, String>
  ```
- **Scaffold Behavior**: Fails loudly with `unimplemented!("T-00383: collect toolchain telemetry")`.
- **Test**: `toolchain_service::tests::test_collect_toolchain_telemetry_scaffold` asserting `#[should_panic]` behavior.

## 3. Build & Test Output
```text
running 1 test
test toolchain_service::tests::test_collect_toolchain_telemetry_scaffold - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 93 filtered out; finished in 0.02s
```
