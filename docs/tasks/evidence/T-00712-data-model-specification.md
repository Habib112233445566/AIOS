# T-00712 — Secrets & Access Hygiene / data model: Specification

## 1. Data Model Specification Overview
This specification formalizes the data structures, severity classifications, redaction functions, and validation invariants for Secrets & Access Hygiene in AIOS (`code/aiosh-rust/aiosh-core/src/secrets.rs`).

## 2. Type & Struct Definitions

### A. SecretSeverity
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecretSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}
```

### B. SecretPatternKind
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecretPatternKind {
    PrivateKey,
    ApiToken,
    AwsCredentials,
    PasswordInConfig,
    HighEntropyGeneric,
}
```

### C. SecretFinding
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretFinding {
    pub rule_id: String,
    pub path: String,
    pub line_number: usize,
    pub severity: SecretSeverity,
    pub pattern_kind: SecretPatternKind,
    pub description: String,
    pub redacted_snippet: String,
    pub fingerprint: String,
}
```

### D. SecretScanReport
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretScanReport {
    pub repo_path: String,
    pub timestamp_utc: String,
    pub is_clean: bool,
    pub total_findings: u32,
    pub critical_findings: u32,
    pub high_findings: u32,
    pub medium_findings: u32,
    pub low_findings: u32,
    pub scanned_files_count: u32,
    pub findings: Vec<SecretFinding>,
}
```

### E. Secret Redaction Helper
```rust
pub fn redact_secret_value(raw: &str) -> String
```
- Preserves first 4 characters and last 4 characters if `raw.len() >= 12`.
- Masks intermediate characters with `****` (e.g., `AKIA****WXYZ`).
- If `raw.len() < 12`, masks entire string with `[REDACTED]`.

## 3. Mathematical & Structural Invariants
- `total_findings == critical_findings + high_findings + medium_findings + low_findings`.
- `total_findings == findings.len() as u32`.
- `is_clean == (total_findings == 0)`.
- `fingerprint` MUST be a valid 64-character lowercase hexadecimal SHA-256 string.
- Raw unmasked secret strings must never be serialized or outputted.
