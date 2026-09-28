# Task T-02492 Evidence: Sandbox Recovery & Validation Specification

## Goal
Specify the exact contract, data structures, invariants, and algorithms for Sandbox Enforcement Recovery & Validation.

## 1. Domain Entities & Schemas

### `SandboxValidationSeverity`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SandboxValidationSeverity {
    Error,
    Warning,
}
```

### `SandboxValidationIssue`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxValidationIssue {
    pub profile_name: Option<String>,
    pub code: String,
    pub message: String,
    pub severity: SandboxValidationSeverity,
}
```

### `SandboxValidationReport`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxValidationReport {
    pub is_healthy: bool,
    pub factory_profiles_intact: bool,
    pub total_profiles_checked: usize,
    pub valid_profiles_count: usize,
    pub corrupt_profiles_count: usize,
    pub issues: Vec<SandboxValidationIssue>,
    pub timestamp_utc: String,
}
```

### `SandboxRecoveryStrategy`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SandboxRecoveryStrategy {
    #[default]
    DryRun,
    RestoreFactoryDefaults,
    QuarantineAndReset,
}
```

### `SandboxRecoveryResult`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxRecoveryResult {
    pub success: bool,
    pub strategy: SandboxRecoveryStrategy,
    pub quarantine_path: Option<String>,
    pub profiles_restored: usize,
    pub issues_resolved: usize,
    pub message: String,
}
```

## 2. Invariants & Rulesets
- `SANDBOXRECV1`: Factory profiles (`standard`, `strict`, `permissive`) must be present in service; missing factory profiles trigger `SANDBOXRECV_ERR_MISSING_FACTORY`.
- `SANDBOXRECV2`: File validation rejects files exceeding 64 KiB or containing directory traversal (`..`).
- `SANDBOXRECV3`: `QuarantineAndReset` strategy atomically moves invalid files into a timestamped directory before re-initializing factory profiles.
- `SANDBOXRECV4`: Diagnostic issues record profile name, exact failure reason, and severity code.
- `SANDBOXRECV5`: Operations are auditable and reproducible across CLI and MCP surfaces.

## 3. Standard Error Codes
- `SANDBOXRECV_ERR_IO`
- `SANDBOXRECV_ERR_TRAVERSAL`
- `SANDBOXRECV_ERR_CORRUPT`
- `SANDBOXRECV_ERR_MISSING_FACTORY`
- `SANDBOXRECV_ERR_LIMIT_BOUNDS`
