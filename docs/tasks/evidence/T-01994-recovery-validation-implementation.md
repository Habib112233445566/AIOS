# Task Evidence: T-01994 - System Update / recovery & validation: Implementation (Sub-Epic 10)

## 1. Overview
- **Task ID**: `T-01994`
- **Sub-Epic**: 10 (System Update Recovery & Validation Subsystem)
- **Goal**: Implement complete working behavior for health checking, validation, and automated corruption recovery for System Update state files and in-memory structures.

---

## 2. Implementation Summary
- **Module**: `code/aiosh-rust/aiosh-core/src/system_update_recovery.rs`.
- **Key Functionality Implemented**:
  1. `validate_update_store_path`: Enforces path safety: UTF-8, length $\le 1024$, no control characters, no `..`, must end with `.json`.
  2. `validate_update_state`: Validates in-memory `SystemSlotStatus` and `SystemUpdateStatus`, checking slot disjunction (`current != target`), version presence, and staging directory status.
  3. `check_update_files`: Reads on-disk `slot_status.json` and `update_status.json`, checking file existence, 1 MB size limit, and JSON syntax.
  4. `recover_update_state_in_memory`:
     - Resolves slot conflicts by assigning `target_slot = current_slot.other()`.
     - Resets non-Idle update states to `Idle`.
     - Prunes unreferenced staging artifacts.
  5. `recover_update_files_with_backup`:
     - Quarantines unparseable or corrupt JSON state files by renaming them to `{path}.corrupt.{timestamp}`.
     - Atomically writes fresh coherent `slot_status.json` and `update_status.json` via `.tmp.<pid>` files.
     - Prunes dangling files in the staging directory.
