# Task Evidence: T-02048 (Capability Model / configuration: Hardening)

## Task Information
- **Task ID**: T-02048
- **Title**: Capability Model / configuration: Hardening
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Hardening Actions
1. **Enforced Symlink Protection on Configuration Loading**:
   - In `CapabilityConfig::from_path`, added `std::fs::symlink_metadata(path)` check.
   - If the configuration file is a symlink, loading is rejected immediately with an explicit error to prevent symlink race or redirection attacks.

2. **Strict Environment Variable Parsing**:
   - In `CapabilityConfig::from_env`, replaced silent `if let Ok(parsed)` pattern with strict error propagation:
     - `AIOS_CAPABILITY_MAX_CAPABILITIES`: Rejects invalid non-numeric strings with descriptive error.
     - `AIOS_CAPABILITY_MAX_STORE_BYTES`: Rejects invalid non-numeric strings with descriptive error.

3. **String Control Character Scrubbing**:
   - Added `is_control()` checks on `CapabilityConfig::version` in `validate()`, preventing log poisoning and terminal escape injections.

4. **File Extension Enforcement**:
   - In `CapabilityConfig::validate()`, enforced that `store_path` must have a `.json` extension, aligning with `validate_service_path` in `CapabilityService`.
