# Task Evidence: T-01793 - Hardware Detection / Recovery & Validation: Scaffold

## Metadata
- **Task ID:** `T-01793`
- **Sub-Epic:** Sub-Epic 10: Hardware Detection / Recovery & Validation
- **Component:** `aiosh-core::hardware_recovery`, `aiosh-core::lib`
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Scaffolding Summary
1. Created `code/aiosh-rust/aiosh-core/src/hardware_recovery.rs`:
   - Defined `HardwareValidationReport`, `HardwareRecoveryAction`, and `HardwareRecoveryReport`.
   - Implemented `validate_invariants` on `HardwareValidationReport` checking `HVAL1` and `HVAL3`.
   - Implemented `validate_inventory` for in-memory invariant and drift validation.
   - Implemented `check_inventory_file` for disk-level size, existence, permissions, and deserialization checking.
   - Implemented `recover_inventory_in_memory` for surgical pruning of invalid devices and summary recalculation.
   - Implemented `recover_inventory_file` for non-destructive quarantine (`.bak.<timestamp>`) and recovery.
2. Exported `pub mod hardware_recovery;` and public re-exports in `code/aiosh-rust/aiosh-core/src/lib.rs`.
