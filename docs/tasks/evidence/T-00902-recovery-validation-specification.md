# T-00902 — Regression Triage / Recovery & Validation: Specification

## 1. Validation & Recovery Contracts

```rust
/// Validate structural invariants of a single TriageRecord.
pub fn validate_triage_record(record: &TriageRecord) -> Result<(), String>;

impl TriageStore {
    /// Attempt to load from path; on corrupted or invalid JSON, return a fresh store and an honest error description.
    pub fn load_or_recover(path: &Path) -> (Self, Option<String>);
}
```

## 2. Invariants & Guarantees
- `validate_triage_record` checks:
  1. `id` starts with `TRG-` and is non-empty.
  2. `signature` is exactly 64 hexadecimal characters.
  3. `test_target` and `error_message` are non-empty after trimming.
  4. `occurrences >= 1`.
- `load_or_recover` guarantees that operators and automated test runners never experience unhandled panics on corrupted store files.
