# T-00315 — Dependency & Toolchain Pinning: Data Model Unit Tests

## Overview
We expanded the unit test coverage for `toolchain_config.rs` to comprehensively assert boundary values and negative states, fulfilling the specification requirements from T-00312.

## Test Coverage Added
1. `test_load_toolchain_config_malformed_json`: Validates that invalid JSON syntax is caught during bounded reading, returning a clear `Malformed toolchain config` error rather than panicking.
2. `test_load_toolchain_config_missing_field`: Validates that `serde` correctly rejects objects missing strictly required fields (like `python_version`), ensuring the struct invariants hold.
3. `test_to_json_with_sources`: Validates that the runtime `json!()` exporter accurately maps source provenance ("env" vs "default") based on what was parsed.

## Verification
- All tests are isolated unit tests operating inside `toolchain_config::tests`.
- The tests run and pass alongside the remaining 83+ tests in `cargo test -p aiosh-core`.
- The tests assert strictly against observable String errors (`err.contains(...)`) rather than implementation details.
