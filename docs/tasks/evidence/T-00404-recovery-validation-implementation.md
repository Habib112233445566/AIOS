# T-00404 — Dependency & Toolchain Pinning / recovery & validation: Implementation

## 1. Implementation Scope
This task implements the Recovery & Validation mechanisms for Dependency & Toolchain Pinning in `aiosh-core`.

## 2. Implementation Details
- **`validate_toolchain_manifest(path: &str) -> Result<ToolchainManifest, String>`**:
  - Validates configuration file existence, checks the 64KB size cap, parses JSON, and asserts that version string fields are non-empty without executing compiler binaries.
- **`recover_default_toolchain() -> ToolchainManifest`**:
  - Returns the canonical in-memory `Default` `ToolchainManifest` (`rust_version: "1.99.0"`, `python_version: "3.14"`, `node_version: Some("v24.18")`, `enforce_hashes: false`).
- **`reconcile_toolchain(manifest: &ToolchainManifest) -> Result<ToolchainReconciliationReport, String>`**:
  - Evaluates host environment against desired manifest versions.
  - Categorizes status per runtime (`conforming`, `drifted`, `missing`, `unconstrained`) and generates actionable remediation guidance.

## 3. Unit Test Verification
```text
running 1 test
test toolchain_service::tests::test_validate_toolchain_manifest_happy_and_error ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.19s

running 1 test
test toolchain_service::tests::test_recover_default_toolchain ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.01s

running 1 test
test toolchain_service::tests::test_reconcile_toolchain_report ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 3.68s
```
