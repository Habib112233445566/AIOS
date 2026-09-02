# T-00373 — Dependency & Toolchain Pinning / security policy: Scaffold

## 1. Scaffold Scope
This task defines the interface and typed signatures for the Dependency & Toolchain Pinning security policy validation function within `code/aiosh-rust/aiosh-core/src/toolchain_service.rs`.

## 2. Interface Definition
- **Function Signature**:
  ```rust
  pub fn check_toolchain_policy(grant: Option<&str>, action: &str) -> Result<(), String>
  ```
- **Scaffold Behavior**: Fails loudly via `unimplemented!("T-00373: toolchain security policy check")`.
- **Test Stub**: `toolchain_service::tests::test_check_toolchain_policy_scaffold` asserting `#[should_panic]` behavior.

## 3. Verification Output
```text
running 1 test
test toolchain_service::tests::test_check_toolchain_policy_scaffold - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 92 filtered out; finished in 0.01s
```
