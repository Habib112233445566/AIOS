# T-00374 — Dependency & Toolchain Pinning / security policy: Implementation

## 1. Implementation Scope
This task implements the native security policy checks and PEP integration for Dependency & Toolchain Pinning operations in `aiosh-core`.

## 2. Implementation Details
- **PEP Integration (`code/aiosh-rust/aiosh-core/src/pep.rs`)**:
  - Registered `aios.toolchain.set` and `toolchain.set` under `is_irreversible()`, ensuring that any state-mutating toolchain commands strictly require valid cryptographic PEP grants.
- **Service Policy Enforcement (`code/aiosh-rust/aiosh-core/src/toolchain_service.rs`)**:
  - Implemented `check_toolchain_policy(grant: Option<&str>, action: &str) -> Result<(), String>`.
  - Enforces that read-only actions (`aios.toolchain.check`, `aios.toolchain.config.get`, `toolchain.show`, `toolchain.check`) execute without grant requirements, whereas mutating actions fail closed unless a valid grant is provided.
- **Unit Tests**:
  - Added `test_check_toolchain_policy_enforcement` covering both permitted read-only actions and rejected unauthorized mutation requests.

## 3. Test Output
```text
running 1 test
test toolchain_service::tests::test_check_toolchain_policy_enforcement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 92 filtered out; finished in 0.00s
```
