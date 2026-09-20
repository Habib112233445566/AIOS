# T-01694: Kernel Module Management Recovery & Validation Implementation

## Sub-Epic
Kernel Module Management / Recovery & Validation (T-01694)

## Objective
Implement the minimal working behavior for the recovery & validation of Kernel Module Management adhering to invariants KR1..KR6, non-destructive quarantine, conflict resolution, and atomic state updates.

## Implementation Details
1. **Report & Invariant Engine (`KernelModuleValidationReport`)**:
   - Implemented `validate_invariants()` checking KR1 (`valid_rules + invalid_rules == total_rules`), KR2 (`valid_autoload + invalid_autoload == total_autoload`), and KR3 (`healthy == (errors.is_empty() && invalid_rules == 0 && invalid_autoload == 0)`).
2. **Deep Validation (`validate_kernel_module_store`)**:
   - Inspects all modprobe directives (`Blacklist`, `Alias`, `Options`, `Install`, `Remove`, `Softdep`) for validity and dangerous characters.
   - Detects KM3 / KR4 conflicts where an autoloaded module is simultaneously blacklisted or disabled via install `/bin/true` or `/bin/false`.
3. **Read-only Store Evaluation (`check_store_file`)**:
   - Accurately reports missing files, parse errors, or semantic violations without modifying state.
4. **Self-Healing Recovery (`recover_store_file`)**:
   - Auto-creates default store if missing.
   - For corrupted or conflict-bearing stores:
     - Safely generates a timestamped quarantine backup (`<filename>.corrupt.<timestamp>.bak`) before making changes (KR5).
     - Filters out invalid rules and conflicting autoload entries.
     - Atomically writes the repaired store via `tempfile + atomic replace` (KR6).
     - Returns a clean `KernelModuleValidationReport` with `recovered: true` and `backup_path: Some(...)`.
5. **Compilation & Quality**:
   - Compiles cleanly with 0 errors and 0 warnings under `cargo check -p aiosh-core`.
