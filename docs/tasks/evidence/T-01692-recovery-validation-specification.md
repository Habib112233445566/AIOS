# T-01692: Kernel Module Management Recovery & Validation Specification

## Sub-Epic
Kernel Module Management / Recovery & Validation (T-01692)

## Objective
Formally specify the data structures, invariant contracts, verification criteria, and operational interfaces for the Kernel Module Management Recovery & Validation subsystem.

## Formal Specification

### 1. Data Structures & Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelModuleValidationReport {
    pub store_path: String,
    pub total_rules: usize,
    pub valid_rules: usize,
    pub invalid_rules: usize,
    pub total_autoload: usize,
    pub valid_autoload: usize,
    pub invalid_autoload: usize,
    pub errors: Vec<String>,
    pub healthy: bool,
    pub recovered: bool,
    pub backup_path: Option<String>,
    pub evaluated_at: String,
}
```

### 2. Invariant Contracts (KR1..KR6)

| Invariant | Name | Guarantee & Acceptance Criterion |
|---|---|---|
| **KR1** | Rule Counting Parity | `valid_rules + invalid_rules == total_rules` holds identically across all reports. |
| **KR2** | Autoload Counting Parity | `valid_autoload + invalid_autoload == total_autoload` holds identically across all reports. |
| **KR3** | Health Equivalence | `healthy == (errors.is_empty() && invalid_rules == 0 && invalid_autoload == 0)`. |
| **KR4** | Conflict Freedom | Any store passing validation or produced by auto-recovery has zero overlapping blacklist/install-disabled and autoload entries. |
| **KR5** | Non-Destructive Quarantine | When `--auto-recover` repairs an invalid or unparseable store, the original file is preserved at `<path>.corrupt.<iso8601>.bak`. |
| **KR6** | Atomic Recovery Persistence | Store state written during recovery is staged in `<path>.tmp.<pid>` and atomically renamed into place. |

### 3. Recovery Engine Behavior
1. **Unparseable JSON**:
   - Backup `<path>` to `<path>.corrupt.<timestamp>.bak`.
   - Initialize new baseline store with ID `default` and save atomically.
   - Set `recovered = true`, `healthy = true`, and report `backup_path`.
2. **Partially Valid Store**:
   - Filter out invalid rules (e.g. malformed identifiers, dangerous parameter values).
   - Resolve conflicts by dropping blacklisted modules from the autoload list.
   - Deduplicate identical rules.
   - Save cleansed store atomically.
   - Set `recovered = true` and update counts.
3. **Healthy Store**:
   - Leave file intact on disk.
   - Set `recovered = false`, `healthy = true`, `backup_path = None`.

### 4. CLI & MCP Interfaces
- **CLI**:
  - `aiosh mod check [--store <path>] [--auto-recover] [--json]`
- **MCP Tool**: `aios.kernel_module.check`:
  - Arguments:
    - `store_path`: `Option<String>`
    - `auto_recover`: `Option<bool>` (default: `false`)
    - `grant_id`: `Option<String>`
