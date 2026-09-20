# Task Evidence: T-01892 - Network Bootstrap / recovery & validation: Specification

## 1. Overview
- **Task ID**: `T-01892`
- **Sub-Epic**: 10 (Network Bootstrap Recovery & Validation)
- **Goal**: Formally specify data structures, recovery actions, and verification interfaces for Network Bootstrap Recovery & Validation.

---

## 2. Formal Specification

### 2.1 Constants & Error Codes
```rust
pub const MAX_NETWORK_STORE_SIZE: u64 = 1_048_576; // 1 MB
pub const MAX_RECOVERY_ISSUES: usize = 100;

pub const NVAL_PATH_ERROR: &str = "NVAL_PATH_ERROR";
pub const NVAL_IO_ERROR: &str = "NVAL_IO_ERROR";
pub const NVAL_VALIDATION_ERROR: &str = "NVAL_VALIDATION_ERROR";
pub const NVAL_PARSE_ERROR: &str = "NVAL_PARSE_ERROR";
```

### 2.2 Invariant Validation Rules
- **`NVAL1`**: `report.valid_interfaces + report.invalid_interfaces == report.total_interfaces`
- **`NVAL2`**: All routes must reference a declared interface. Interfaces not found in `interfaces` are marked as dangling routes.
- **`NVAL3`**: DNS must contain at least one valid IPv4/IPv6 address.
- **`NVAL4`**: `healthy == true` iff `errors.is_empty() && invalid_interfaces == 0 && dangling_routes.is_empty() && !missing_loopback && dns_configured`.
- **`NVAL5`**: Corrupt file quarantine creates timestamped sibling `<file>.bak.<timestamp>` before recreation.
- **`NVAL6`**: Path hygiene ($\le 1024$ chars, `.json` extension, no `..`, no nulls/controls) and 1 MB max size.

### 2.3 Recovery Actions Enum
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkRecoveryAction {
    NoneRequired,
    QuarantineCorruptedStore { backup_path: String },
    PruneDanglingRoutes { pruned_count: usize },
    RestoreLoopback,
    SetDefaultDnsFallback { fallback_servers: Vec<String> },
    RecreateEmptyConfig,
}
```

### 2.4 Reports & API Signatures
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkValidationReport {
    pub store_path: String,
    pub total_interfaces: usize,
    pub valid_interfaces: usize,
    pub invalid_interfaces: usize,
    pub dangling_routes: Vec<String>,
    pub missing_default_route: bool,
    pub missing_loopback: bool,
    pub dns_configured: bool,
    pub errors: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkRecoveryReport {
    pub store_path: String,
    pub initial_validation: NetworkValidationReport,
    pub actions_taken: Vec<NetworkRecoveryAction>,
    pub final_validation: NetworkValidationReport,
    pub backup_path: Option<String>,
    pub recovered: bool,
    pub completed_at: String,
}

pub fn validate_network_state(state: &NetworkState, store_path: &Path) -> NetworkValidationReport;
pub fn check_network_file(path: &Path) -> NetworkValidationReport;
pub fn recover_network_state_in_memory(state: &mut NetworkState, store_path: &Path) -> NetworkRecoveryReport;
pub fn recover_network_file(path: &Path) -> Result<NetworkRecoveryReport, String>;
```

Status: Specification completed. Ready for scaffolding in `T-01893`.
