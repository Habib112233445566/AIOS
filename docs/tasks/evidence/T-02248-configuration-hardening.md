# Task Evidence: T-02248 (Grant Lifecycle Configuration: Hardening)

## Overview
- **Task ID**: `T-02248`
- **Task Name**: Grant Lifecycle Configuration: Hardening
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Grant Lifecycle
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Timestamp**: 2026-09-23T01:29:00+05:00
- **Status**: COMPLETED

## Hardening Measures Implemented

1. **Size Caps & Resource Bounds**:
   - Configuration file loading strictly capped at 64 KiB (`MAX_CONFIG_BYTES`) via `File::take()`.
   - Store path length bounded to $\le 1024$ characters.
   - Registry grant capacity clamped to $[1, 50\,000]$ (`MIN_GRANTS_COUNT` .. `MAX_GRANTS_COUNT`).
   - Store size bounded within $[1\,024, 104\,857\,600]$ bytes (1 KiB to 100 MiB).
   - Delegation depth clamped to $[1, 10]$ (`MIN_DELEGATION_DEPTH` .. `MAX_DELEGATION_DEPTH`).

2. **Standard Error Envelopes & Diagnostics**:
   - Explicit error code categorization:
     - `GRANTCONF_ERR_IO`: Filesystem IO errors (file not found, read failures).
     - `GRANTCONF_ERR_PARSE`: JSON syntax and deserialization errors.
     - `GRANTCONF_ERR_VALIDATION`: Path traversal, control characters, empty version, symlink rejection.
     - `GRANTCONF_ERR_BOUNDS`: Numerical boundary violations.
   - Zero silent drops or unhandled panics.

3. **Atomic Persistence & Symlink Protection**:
   - `save_to_path()` stages data into `.pep_grant_config.tmp.<pid>` before atomic `std::fs::rename()`.
   - Clean failure cleanup: temporary staging file is deleted if write or rename fails.
   - `from_path()` inspects `std::fs::symlink_metadata()` to refuse symlinks.

4. **Hardening Unit Tests**:
   - Added `test_pep_grant_config_hardening_checks` to `code/aiosh-rust/aiosh-core/tests/test_pep_grant_config.rs`:
     - Asserts rejection of control characters in `store_path` (`\n`).
     - Asserts rejection of oversized paths ($> 1024$ characters).
     - Asserts fail-closed handling on non-existent configuration files.
     - Asserts clean error reporting on malformed JSON.
   - 9/9 unit tests passing cleanly.
