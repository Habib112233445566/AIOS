# T-00713 — Secrets & Access Hygiene / data model: Scaffold

## 1. Scaffold Deliverables
- Created `code/aiosh-rust/aiosh-core/src/secrets.rs` containing typed declarations for:
  - `SecretSeverity`: Enum with variants `Critical`, `High`, `Medium`, `Low`, `Info`.
  - `SecretPatternKind`: Enum with variants `PrivateKey`, `ApiToken`, `AwsCredentials`, `PasswordInConfig`, `HighEntropyGeneric`.
  - `SecretFinding`: Struct defining `rule_id`, `path`, `line_number`, `severity`, `pattern_kind`, `description`, `redacted_snippet`, and `fingerprint`.
  - `SecretScanReport`: Aggregated report struct tracking `repo_path`, `timestamp_utc`, `is_clean`, findings counts, and findings list.
  - `redact_secret_value`: Safe redaction helper preserving boundary characters.
- Registered `pub mod secrets;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.

## 2. Compilation Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml` compiled cleanly.
