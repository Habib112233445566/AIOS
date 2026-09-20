# T-01748: Hardware Detection — Configuration Hardening

## Metadata
- **Task ID**: `T-01748`
- **Sub-Epic**: Sub-Epic 5: Hardware Detection Configuration
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Hardening Remediation Summary
Implemented defenses addressing all vulnerabilities identified in `T-01747`:

1. **Path Traversal Remediation (CFG-SEC-1)**:
   - Added component-level inspection in `HardwareConfig::validate()` checking for `std::path::Component::ParentDir`.
   - Rejects any path containing `..` with error: `"HCFG1 violation: <name> cannot contain parent directory traversal ('..')"`.
2. **Environment Invariant Protection (CFG-SEC-2)**:
   - Added post-validation guard to `HardwareConfig::from_env()`.
   - If environment overrides result in any invariant violation (length, control characters, traversal, bounds), `from_env()` automatically and safely falls back to `HardwareConfig::default()`.
3. **Atomic File Write Protection (CFG-SEC-3)**:
   - In `HardwareConfig::save_to_path()`, data is written to a unique sibling temporary file (`.<filename>.tmp.<pid>`), flushed, and then atomically renamed onto the target file.
   - On error, the temporary file is cleaned up.

---

## 2. Verification
- `test_hcfg1_path_hygiene_traversal`: Confirms rejection of `..` paths.
- `test_hardware_config_from_env_invalid_fallback`: Confirms automatic fallback to safe default when environment variables contain invalid values.
- All 16 unit tests in `test_hardware_config.rs` pass.
