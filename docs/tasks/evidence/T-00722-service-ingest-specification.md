# T-00722 — Secrets & Access Hygiene / core service: Specification

## 1. Service Specification Overview
This specification defines the core scanning service functions and signature patterns for credential and secret discovery in `code/aiosh-rust/aiosh-core/src/secrets_service.rs`.

## 2. Rule Signatures & Detection Patterns

| Rule ID | Name | Category | Severity | Regex Pattern |
|---|---|---|---|---|
| `SEC-001` | Private Key Pattern | `PrivateKey` | `Critical` | `-----BEGIN (?:[A-Z0-9_-]+ )?PRIVATE KEY-----` |
| `SEC-002` | AWS Access Key ID | `AwsCredentials` | `Critical` | `\bAKIA[0-9A-Z]{16}\b` |
| `SEC-003` | GitHub Personal Access Token | `ApiToken` | `High` | `\b(?:ghp_[0-9a-zA-Z]{36}\|github_pat_[0-9a-zA-Z_]{82})\b` |
| `SEC-004` | Generic API Token | `ApiToken` | `High` | `(?i)\b(?:api_key\|apikey\|bearer_token\|secret_token)\b\s*[:=]\s*["']?([a-zA-Z0-9_\-\.]{20,})["']?` |
| `SEC-005` | Password in Config | `PasswordInConfig` | `High` | `(?i)\b(?:password\|db_pass\|secret_key)\b\s*[:=]\s*["']?([^\s"']{8,})["']?` |

## 3. Function Signatures & Contracts

### A. `scan_file_for_secrets`
```rust
pub fn scan_file_for_secrets(
    path: &Path,
    base_dir: &Path,
    max_file_bytes: u64,
) -> Result<Vec<SecretFinding>, String>
```
- Returns `Ok(Vec<SecretFinding>)` containing any detected secrets with redacted snippets.
- Skips binary files (first 512 bytes containing null bytes `\0`).
- Skips files exceeding `max_file_bytes` (default 16 MiB).
- Truncates scan line parsing at 4096 characters.

### B. `scan_workspace_for_secrets`
```rust
pub fn scan_workspace_for_secrets(
    root: &Path,
    max_file_bytes: u64,
    ignored_dirs: &[&str],
) -> Result<SecretScanReport, String>
```
- Traverses `root` recursively, skipping directories in `ignored_dirs` (`.git`, `target`, `node_modules`, `.venv`).
- Aggregates findings and returns a validated `SecretScanReport`.

## 4. Invariants & Output Guarantees
- Raw unredacted secrets MUST NEVER be retained in the generated `SecretFinding` records.
- Fingerprints are computed via `sha256(rule_id || ":" || rel_path || ":" || line_number || ":" || matched_secret)`.
- If no secrets are found, returns `is_clean: true` and `total_findings: 0`.
