# T-00612 — Repository Health / data model: Specification

## 1. Goal & Architecture
This specification defines the formal data structures, JSON schemas, field validation rules, and error handling invariants for the **Repository Health** component (`T-00611..T-00710`) implemented in `aiosh-core::repo_health`.

## 2. Type Definitions

### 2.1 HealthStatus & HealthCategory Enums
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthCategory {
    GitHygiene,
    FileIntegrity,
    SecurityGovernance,
    DependencyHygiene,
    WorkspaceBounds,
}
```

### 2.2 RepoHealthCheck
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoHealthCheck {
    pub check_id: String,
    pub name: String,
    pub category: HealthCategory,
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<Vec<String>>,
    pub duration_ms: u64,
}
```

**Field Validation Invariants:**
- `check_id`: Must be non-empty, 1..64 characters, containing only ASCII alphanumeric characters, hyphens, or underscores (`[a-zA-Z0-9_-]+`).
- `name`: Must be non-empty, 1..128 characters.
- `message`: Must be bounded to $\le 1024$ characters.
- `details`: If present, list length $\le 100$, each detail entry $\le 512$ characters.

### 2.3 RepoHealthReport
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoHealthReport {
    pub repo_path: String,
    pub timestamp_utc: String,
    pub overall_status: HealthStatus,
    pub total_checks: u32,
    pub passed_checks: u32,
    pub warn_checks: u32,
    pub failed_checks: u32,
    pub skipped_checks: u32,
    pub checks: Vec<RepoHealthCheck>,
}
```

**Report Validation Invariants:**
- `repo_path`: Must be a non-empty string bounded to $\le 1024$ characters.
- `timestamp_utc`: Must be a non-empty ISO 8601 string.
- `total_checks`: Must strictly equal `checks.len() as u32`.
- `passed_checks + warn_checks + failed_checks + skipped_checks`: Must strictly equal `total_checks`.
- **Overall Status Derivation**:
  - If any check has `HealthStatus::Fail` $\implies$ `overall_status == HealthStatus::Fail`.
  - Else if any check has `HealthStatus::Warn` $\implies$ `overall_status == HealthStatus::Warn`.
  - Else $\implies$ `overall_status == HealthStatus::Pass`.

## 3. JSON Serialization Schemas

### Example Valid Report JSON:
```json
{
  "repo_path": "/workspace/AIOS",
  "timestamp_utc": "2026-08-29T12:00:00Z",
  "overall_status": "Pass",
  "total_checks": 2,
  "passed_checks": 2,
  "warn_checks": 0,
  "failed_checks": 0,
  "skipped_checks": 0,
  "checks": [
    {
      "check_id": "git_status",
      "name": "Git Working Tree Cleanliness",
      "category": "GitHygiene",
      "status": "Pass",
      "message": "Working tree clean, no uncommitted modifications",
      "details": null,
      "duration_ms": 12
    },
    {
      "check_id": "security_policy",
      "name": "Security Policy Invariant",
      "category": "SecurityGovernance",
      "status": "Pass",
      "message": "SECURITY.md conforms to OpenSSF standards",
      "details": null,
      "duration_ms": 4
    }
  ]
}
```

## 4. Error Handling & Boundary Guarantees
- Validation methods return standard `Result<(), String>` indicating the exact failed invariant without panicking.
- Formatting helpers provide human-readable summary projections with safe truncation.
