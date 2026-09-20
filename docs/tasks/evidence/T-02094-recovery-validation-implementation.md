# Task Evidence: T-02094 (recovery & validation: Implementation)

## Overview
- **Task ID**: T-02094
- **Sub-Epic**: Sub-Epic 10: Capability Model Recovery & Validation Subsystem
- **Component**: `aiosh-core::capability_recovery`
- **Objective**: Implement deep validation (`validate_capability_store`) and non-destructive quarantine recovery (`recover_capability_store`, `create_backup_file`).

## Implementation Details
1. **Deep Validation (`validate_capability_store`)**:
   - Enforces `CAPREC1..CAPREC4`:
     - Checks individual capability fields (format of `id`, `subject`, `issuer`, non-empty `rights`).
     - Validates temporal constraints (`not_before`, `expires_at`, RFC 3339 format, monotonic time ordering).
     - Verifies capability lineage: parent existence, cycle detection via traversal visited sets (`CAPREC3`).
     - Verifies monotonic attenuation: child rights $\subseteq$ parent rights, child scope $\subseteq$ parent scope (`CAPREC4`).
     - Verifies registry capacity bounds against configured limit.
2. **Non-Destructive Quarantine Recovery (`recover_capability_store`)**:
   - Handles missing stores by initializing a clean default store (`CreatedDefaultFresh`).
   - Validates existing stores: if healthy, returns `LoadedExisting`.
   - If a store file is corrupt (unparseable JSON or fails validation), safely quarantines the damaged file via `create_backup_file` into a timestamped backup (`<store>.bak.<timestamp>[.<counter>]` with mode 0600 on Unix) and initializes a healthy store (`RecoveredFromBackup`).
   - Zero data loss: corrupt or damaged state is never overwritten or deleted.

## Verification
- Validated via `cargo check -p aiosh-core`.
