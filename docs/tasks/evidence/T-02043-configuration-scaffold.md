# Task Evidence: T-02043 (Capability Model / configuration: Scaffold)

## Task Information
- **Task ID**: T-02043
- **Title**: Capability Model / configuration: Scaffold
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Scaffold
1. **Module Creation**:
   - Created `code/aiosh-rust/aiosh-core/src/capability_config.rs`.
   - Exported `pub mod capability_config;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.

2. **Scaffolded Constants & Types**:
   - `MAX_CONFIG_BYTES: u64 = 64 * 1024`
   - `DEFAULT_CAPABILITY_STORE_PATH: &str = ".aios/capability_store.json"`
   - `MIN_STORE_BYTES: u64 = 1024`
   - `MAX_STORE_BYTES: u64 = 104_857_600`
   - `DEFAULT_MAX_STORE_BYTES: u64 = 10_485_760`
   - `MAX_CAPABILITY_COUNT: usize = 1_000_000`
   - `DEFAULT_MAX_CAPABILITY_COUNT: usize = 10_000`
   - `MAX_EXPIRES_SECS: u64 = 315_360_000`
   - `CapabilityConfig` struct with full serde derive and field definitions.
   - Initial method stubs for `from_json`, `to_json`, `from_path`, `from_env`, and `validate`.

3. **Compilation**:
   - Verified that `cargo check -p aiosh-core` compiles cleanly.
