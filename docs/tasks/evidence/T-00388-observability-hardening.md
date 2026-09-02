# T-00388 — Dependency & Toolchain Pinning / observability: Hardening

## 1. Hardening Scope
This task hardens the toolchain runtime observability and telemetry collectors against output buffer inflation, memory exhaustion, process leaks, and untrusted string formatting.

## 2. Hardening Measures
1. **Output String Clamping (`clamp_str`)**:
   - Diagnostic output from compiler/runtime binaries (`rustc`, `python`, `node`) is clamped to at most 512 bytes with an explicit `...[TRUNCATED]` indicator, preventing memory bloat and audit WAL inflation attacks.
2. **Subprocess Lifecycle & Timeout Enforcement**:
   - `run_with_timeout` enforces bounded 15-second execution with automatic process reaping and error isolation.
3. **Lossless Structured Error Propagation**:
   - Subprocess errors and missing binaries return structured `Err(String)` messages that bubble up cleanly into standard JSON envelopes (`outcome_detail`), preserving the audit-on-failure contract without panicking.

## 3. Verification Output
```text
running 2 tests
test toolchain_service::tests::test_collect_toolchain_telemetry_negative_case ... ok
test toolchain_service::tests::test_collect_toolchain_telemetry_captures_details ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 93 filtered out; finished in 2.89s
```
