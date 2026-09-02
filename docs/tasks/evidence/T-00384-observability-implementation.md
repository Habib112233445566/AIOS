# T-00384 — Dependency & Toolchain Pinning / observability: Implementation

## 1. Implementation Scope
This task implements runtime toolchain telemetry gathering, binary probe stdout capture, and error diagnostic aggregation in `aiosh-core`.

## 2. Implementation Details
- **`ToolchainTelemetry` Struct (`code/aiosh-rust/aiosh-core/src/toolchain_service.rs`)**:
  - Encapsulates target manifest, detected compiler/runtime outputs (`detected_rust`, `detected_python`, `detected_node`), and execution check status (`check_passed`).
- **`collect_toolchain_telemetry` Function**:
  - Dispatches bounded 15-second subprocess commands (`rustc -V`, `python3 -V`/`python -V`, `node -v`).
  - Captures and trims stdout text losslessly into telemetry properties.
  - Executes `enforce_toolchain(manifest)` to compute overall conformance boolean.
- **Unit Test**:
  - `test_collect_toolchain_telemetry_captures_details` validates that telemetry correctly extracts host versions for active runtimes.

## 3. Test Output
```text
running 1 test
test toolchain_service::tests::test_collect_toolchain_telemetry_captures_details ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 93 filtered out; finished in 5.61s
```
