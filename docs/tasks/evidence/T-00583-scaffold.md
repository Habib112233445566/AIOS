# T-00583 — Evidence & Audit Trail / observability: Scaffold

## 1. Scaffold Scope
This task scaffolds the `EvidenceTelemetry` data structure and `collect_evidence_telemetry` function signature in `code/aiosh-rust/aiosh-core`.

## 2. Scaffold Implementation Details
- **`code/aiosh-rust/aiosh-core/src/evidence.rs`**:
  ```rust
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct EvidenceTelemetry {
      pub total_records: usize,
      pub valid_records: usize,
      pub missing_files_count: usize,
      pub hash_mismatches_count: usize,
      pub is_healthy: bool,
  }
  ```
- **`code/aiosh-rust/aiosh-core/src/evidence_service.rs`**:
  ```rust
  pub fn collect_evidence_telemetry(report: &EvidenceVerificationReport) -> EvidenceTelemetry {
      EvidenceTelemetry {
          total_records: report.total_records,
          valid_records: report.valid_records,
          missing_files_count: report.missing_files.len(),
          hash_mismatches_count: report.hash_mismatches.len(),
          is_healthy: report.is_valid,
      }
  }
  ```

## 3. Test Verification
Compiles cleanly across workspace crates.
