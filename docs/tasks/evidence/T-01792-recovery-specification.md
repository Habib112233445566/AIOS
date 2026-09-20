# Task Evidence: T-01792 - Hardware Detection / Recovery & Validation: Specification

## Metadata
- **Task ID:** `T-01792`
- **Sub-Epic:** Sub-Epic 10: Hardware Detection / Recovery & Validation
- **Component:** `aiosh-core::hardware_recovery`
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Technical Specification

### 1. Data Structures & Types

#### `HardwareValidationReport`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareValidationReport {
    pub store_path: String,
    pub total_devices: usize,
    pub valid_devices: usize,
    pub invalid_devices: usize,
    pub stale_paths: Vec<String>,
    pub drift_detected: bool,
    pub summary_mismatches: Vec<String>,
    pub errors: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}
```

#### `HardwareRecoveryAction`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HardwareRecoveryAction {
    NoneRequired,
    QuarantineCorruptedStore { backup_path: String },
    PruneInvalidDevices { pruned_count: usize },
    RecomputeSummary,
    RescanSysfs { scanned_devices: usize },
    RecreateEmptyInventory,
}
```

#### `HardwareRecoveryReport`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareRecoveryReport {
    pub store_path: String,
    pub initial_validation: HardwareValidationReport,
    pub actions_taken: Vec<HardwareRecoveryAction>,
    pub final_validation: HardwareValidationReport,
    pub backup_path: Option<String>,
    pub recovered: bool,
    pub completed_at: String,
}
```

### 2. Validation & Recovery Invariants (HVAL1..HVAL6)
- **HVAL1 (Device Accounting)**: `valid_devices + invalid_devices == total_devices`.
- **HVAL2 (Summary Reconciliation)**: `summary_mismatches.is_empty()` when all class counts match valid devices.
- **HVAL3 (Health State Consistency)**: `healthy` is true if and only if `errors.is_empty()`, `invalid_devices == 0`, `!drift_detected`, and `summary_mismatches.is_empty()`.
- **HVAL4 (Non-Destructive Quarantine)**: Damaged stores on disk are renamed/copied to `.bak.<timestamp>` prior to replacement.
- **HVAL5 (Bounded Resource Limits)**: Maximum store file size is capped at 10 MB (`MAX_STORE_FILE_SIZE`). Maximum device count is capped at 10,000 (`MAX_DEVICES`).
- **HVAL6 (Deterministic Canonical Output)**: Output reports utilize RFC3339 timestamps and sanitized strings.

### 3. API Signatures
- `pub fn validate_inventory(inv: &HardwareInventory, check_paths: bool) -> HardwareValidationReport`
- `pub fn check_inventory_file(path: &Path, check_paths: bool) -> Result<HardwareValidationReport, String>`
- `pub fn recover_inventory_in_memory(inv: &mut HardwareInventory) -> Vec<HardwareRecoveryAction>`
- `pub fn recover_inventory_file(path: &Path, fallback_scan: bool) -> Result<HardwareRecoveryReport, String>`
- `pub fn validate_report_invariants(report: &HardwareValidationReport) -> Result<(), String>`
