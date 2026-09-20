# T-01691: Kernel Module Management Recovery & Validation Research

## Sub-Epic
Kernel Module Management / Recovery & Validation (T-01691)

## Objective
Research store corruption mechanisms, self-healing algorithms, non-destructive quarantine strategies, and validation invariants for the Kernel Module Management subsystem.

## Research Findings

### 1. Store Failure & Corruption Modes
- **Truncted/Malformed JSON**: Power loss or interrupted process writes leaving syntactically invalid JSON on disk.
- **Syntactic Violations**: Invalid module identifiers (containing slashes, dots, dashes, shell control characters) or overlong values exceeding memory bounds.
- **Internal Semantic Contradictions**: Conflicting configuration directives, specifically a module present in both `autoload_modules` and disabled via `blacklist` or `install <module> /bin/true` (violating invariant KM3).
- **Rule Redundancy & Duplication**: Repeated identical rules causing configuration bloat or conflicting options for the same module.
- **Unbounded Store Growth**: Store files exceeding the maximum allowed size ceiling (`MAX_MODULE_DOC_BYTES = 10 MiB`).

### 2. Self-Healing & Recovery Architecture
- **Component**: `aiosh_core::kernel_module_recovery`
- **Validation Report**: `KernelModuleValidationReport`:
  - `store_path: String`
  - `healthy: bool`
  - `total_rules: usize`, `valid_rules: usize`, `invalid_rules: usize`
  - `total_autoload: usize`, `valid_autoload: usize`, `invalid_autoload: usize`
  - `errors: Vec<String>`
  - `recovered: bool`
  - `backup_path: Option<String>`
  - `evaluated_at: String`
- **Non-Destructive Quarantine**:
  - Before repairing or re-initializing a corrupted store file, the original file is preserved by renaming to `<path>.corrupt.<timestamp>.bak`.
- **Sanitization Pipeline**:
  - Drops invalid rules and logs specific reasons in the report.
  - Resolves conflict by removing blacklisted modules from the autoload list.
  - Deduplicates identical rules while preserving first-seen directive order.
  - Atomically writes repaired store via temporary file replacement.

### 3. Invariant Contracts (KR1..KR6)
- **KR1**: Structural Consistency (`valid_rules + invalid_rules == total_rules`).
- **KR2**: Health Equivalence (`healthy == (errors.is_empty() && invalid_rules == 0 && invalid_autoload == 0)`).
- **KR3**: Conflict-Free Post-Recovery (No recovered store contains conflicting blacklist and autoload entries).
- **KR4**: Non-Destructive Backup (Corrupted files are never deleted; quarantined to timestamped `.bak`).
- **KR5**: Atomic Persistence (All store recoveries use atomic rename to prevent half-written files).
- **KR6**: Read-Only Non-Mutating Validation (Validation without `--auto-recover` never modifies disk state).
