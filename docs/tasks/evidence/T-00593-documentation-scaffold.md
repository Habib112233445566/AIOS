# T-00593 — Evidence & Audit Trail / documentation: Scaffold

## 1. Scaffold Scope
This task scaffolds the `format_evidence_summary` function signature in `code/aiosh-rust/aiosh-core/src/evidence_service.rs`.

## 2. Scaffold Implementation Details
- **`code/aiosh-rust/aiosh-core/src/evidence_service.rs`**:
  ```rust
  pub fn format_evidence_summary(manifest: &TaskEvidenceManifest) -> String {
      let mut out = format!("AIOS Task Evidence Manifest ({}):\n", manifest.task_range);
      out.push_str(&format!("  Epic: {}\n", manifest.epic_name));
      out.push_str(&format!("  Generated At: {}\n", manifest.generated_at));
      if manifest.records.is_empty() {
          out.push_str("  (no evidence records)");
          return out;
      }
      for (i, record) in manifest.records.iter().enumerate() {
          if i > 0 {
              out.push('\n');
          }
          let short_hash = if record.sha256_hash.len() >= 8 {
              &record.sha256_hash[..8]
          } else {
              &record.sha256_hash
          };
          out.push_str(&format!(
              "  [T-{:05} {:?}] {} ({}) - {}",
              record.task_id, record.step, record.file_path, short_hash, record.status
          ));
      }
      out
  }
  ```

## 3. Test Verification
Compiles cleanly across workspace crates.
