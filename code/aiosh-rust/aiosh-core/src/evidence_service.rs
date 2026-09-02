//! Evidence & Audit Trail core service (T-00524).
//!
//! Contract: `docs/tasks/evidence/T-00522-core-service-specification.md`.

use std::path::Path;

use crate::canonical::{sha256_hex_bytes, utcnow_iso};
use crate::evidence::{
    EvidenceRecord, EvidenceStep, EvidenceTelemetry, EvidenceVerificationReport, TaskEvidenceManifest,
};
use crate::evidence_config::EvidenceConfig;

/// Maximum file size read during evidence checksum computation (16 MiB).
pub const MAX_DOC_BYTES: u64 = 16 * 1024 * 1024;

/// Computes the SHA-256 hex checksum of a file using EvidenceConfig limits.
pub fn compute_file_sha256_with_config(path: &Path, config: &EvidenceConfig) -> Result<String, String> {
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Failed to read metadata for {}: {e}", path.display()))?;

    if metadata.len() > config.max_file_bytes {
        return Err(format!(
            "File {} exceeds max size cap of {} bytes ({} bytes)",
            path.display(),
            config.max_file_bytes,
            metadata.len()
        ));
    }

    let bytes = std::fs::read(path)
        .map_err(|e| format!("Failed to read file {}: {e}", path.display()))?;

    Ok(sha256_hex_bytes(&bytes))
}

/// Computes the SHA-256 hex checksum of a file up to 16 MiB (T-00524).
pub fn compute_file_sha256(path: &Path) -> Result<String, String> {
    let config = EvidenceConfig::from_env().unwrap_or_default();
    compute_file_sha256_with_config(path, &config)
}

/// Constructs a validated EvidenceRecord from a file on disk (T-00524).
pub fn build_evidence_record(
    repo_root: &Path,
    rel_path: &str,
    task_id: u32,
    step: EvidenceStep,
    summary: Option<String>,
) -> Result<EvidenceRecord, String> {
    let clean_path = rel_path.trim();
    if clean_path.is_empty() {
        return Err("rel_path cannot be empty".into());
    }

    let normalized = clean_path.replace('\\', "/");
    if normalized.starts_with('/')
        || normalized.contains(':')
        || normalized.split('/').any(|part| part == "..")
    {
        return Err(format!("rel_path '{}' must be relative and cannot escape repository bounds", rel_path));
    }

    let full_path = repo_root.join(clean_path);
    let hash = compute_file_sha256(&full_path)?;

    let record = EvidenceRecord {
        task_id,
        step,
        file_path: clean_path.to_string(),
        sha256_hash: hash,
        timestamp_utc: utcnow_iso(),
        status: "pass".into(),
        summary,
    };

    record.validate()?;
    Ok(record)
}

/// Verifies all artifacts in a TaskEvidenceManifest against disk state (T-00524).
pub fn verify_evidence_manifest(
    repo_root: &Path,
    manifest: &TaskEvidenceManifest,
) -> Result<EvidenceVerificationReport, String> {
    manifest.validate()?;

    let total_records = manifest.records.len();
    let mut valid_records = 0;
    let mut missing_files = Vec::new();
    let mut hash_mismatches = Vec::new();

    for record in &manifest.records {
        let full_path = repo_root.join(&record.file_path);
        if !full_path.exists() {
            missing_files.push(record.file_path.clone());
            continue;
        }

        match compute_file_sha256(&full_path) {
            Ok(actual_hash) => {
                if actual_hash == record.sha256_hash {
                    valid_records += 1;
                } else {
                    hash_mismatches.push(format!(
                        "{}: expected {}, found {}",
                        record.file_path, record.sha256_hash, actual_hash
                    ));
                }
            }
            Err(e) => {
                hash_mismatches.push(format!("{}: read error ({})", record.file_path, e));
            }
        }
    }

    let is_valid = missing_files.is_empty() && hash_mismatches.is_empty();

    Ok(EvidenceVerificationReport {
        total_records,
        valid_records,
        missing_files,
        hash_mismatches,
        is_valid,
    })
}

/// Checks security policy for Evidence & Audit Trail actions (T-00524).
pub fn check_evidence_policy(grant: Option<&str>, tool_name: &str) -> Result<(), String> {
    match tool_name {
        "aios.evidence.record" | "evidence.record" | "aios.evidence.set" | "evidence.set" => {
            match grant {
                Some(g) if !g.trim().is_empty() => Ok(()),
                _ => Err("PermissionDenied: mutating evidence actions require a valid PEP grant".into()),
            }
        }
        _ => Ok(()),
    }
}

/// Collects aggregate telemetry metrics for Evidence & Audit Trail (T-00584).
pub fn collect_evidence_telemetry(report: &EvidenceVerificationReport) -> EvidenceTelemetry {
    EvidenceTelemetry {
        total_records: report.total_records,
        valid_records: report.valid_records,
        missing_files_count: report.missing_files.len(),
        hash_mismatches_count: report.hash_mismatches.len(),
        is_healthy: report.is_valid,
    }
}

/// Formats a human-readable text summary of a TaskEvidenceManifest (T-00594).
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

/// Parses the EvidenceStep from a task evidence filename loosely.
fn step_from_filename(filename: &str) -> EvidenceStep {
    let lower = filename.to_lowercase();
    if lower.contains("research") {
        EvidenceStep::Research
    } else if lower.contains("spec") {
        EvidenceStep::Spec
    } else if lower.contains("scaffold") {
        EvidenceStep::Scaffold
    } else if lower.contains("implementation") {
        EvidenceStep::Implementation
    } else if lower.contains("unit") || lower.contains("test") {
        EvidenceStep::UnitTest
    } else if lower.contains("integration") {
        EvidenceStep::Integration
    } else if lower.contains("security") {
        EvidenceStep::SecurityReview
    } else if lower.contains("hardening") {
        EvidenceStep::Hardening
    } else if lower.contains("documentation") || lower.contains("doc") {
        EvidenceStep::Documentation
    } else {
        EvidenceStep::Verification
    }
}

/// Scans the evidence directory on disk and returns discovered EvidenceRecords (T-00604).
pub fn scan_evidence_directory(
    repo_root: &Path,
    task_filter: Option<u32>,
) -> Result<Vec<EvidenceRecord>, String> {
    let evidence_dir = repo_root.join("docs/tasks/evidence");
    if !evidence_dir.exists() {
        return Err(format!("Evidence directory not found: {}", evidence_dir.display()));
    }

    let mut records = Vec::new();
    let entries = std::fs::read_dir(&evidence_dir)
        .map_err(|e| format!("Failed to read evidence dir {}: {e}", evidence_dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            if file_name.starts_with('T') && file_name.contains('-') {
                let parts: Vec<&str> = file_name.split('-').collect();
                if parts.len() >= 2 {
                    if let Ok(tid) = parts[1].parse::<u32>() {
                        if task_filter.map_or(true, |target| target == tid) {
                            let rel_path = format!("docs/tasks/evidence/{}", file_name);
                            if let Ok(hash) = compute_file_sha256(&path) {
                                records.push(EvidenceRecord {
                                    task_id: tid,
                                    step: step_from_filename(&file_name),
                                    file_path: rel_path,
                                    sha256_hash: hash,
                                    timestamp_utc: utcnow_iso(),
                                    status: "pass".into(),
                                    summary: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    records.sort_by_key(|r| (r.task_id, r.file_path.clone()));
    Ok(records)
}

/// Recovers the canonical in-memory default EvidenceConfig (T-00604).
pub fn recover_default_evidence_config() -> EvidenceConfig {
    EvidenceConfig::default()
}

/// Reconstructs a TaskEvidenceManifest from live on-disk evidence files (T-00604).
pub fn reconstruct_evidence_manifest(
    repo_path: &Path,
    task_range: &str,
    epic_name: &str,
) -> Result<TaskEvidenceManifest, String> {
    let records = scan_evidence_directory(repo_path, None)?;
    Ok(TaskEvidenceManifest {
        epic_name: epic_name.to_string(),
        task_range: task_range.to_string(),
        generated_at: utcnow_iso(),
        records,
    })
}

/// Reconciles an evidence manifest by executing full verification and computing telemetry (T-00604).
pub fn reconcile_evidence_manifest(
    repo_path: &Path,
    manifest: &TaskEvidenceManifest,
) -> Result<(EvidenceVerificationReport, EvidenceTelemetry), String> {
    let report = verify_evidence_manifest(repo_path, manifest)?;
    let telemetry = collect_evidence_telemetry(&report);
    Ok((report, telemetry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_file_sha256_happy_and_missing() {
        let temp_dir = std::env::temp_dir().join("aios_test_evidence_sha");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.txt");
        std::fs::write(&file_path, "hello evidence\n").unwrap();

        let hash_res = compute_file_sha256(&file_path);
        assert!(hash_res.is_ok());
        let hash = hash_res.unwrap();
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, sha256_hex_bytes(b"hello evidence\n"));

        let missing_res = compute_file_sha256(&temp_dir.join("missing.txt"));
        assert!(missing_res.is_err());
        assert!(missing_res.unwrap_err().contains("File not found"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_build_and_verify_evidence_manifest_happy() {
        let temp_dir = std::env::temp_dir().join("aios_test_evidence_verify_happy");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("docs/tasks/evidence")).unwrap();

        let rel_file = "docs/tasks/evidence/T-00524-test.md";
        std::fs::write(temp_dir.join(rel_file), "# Evidence for T-00524\n\nPASS\n").unwrap();

        let record_res = build_evidence_record(
            &temp_dir,
            rel_file,
            524,
            EvidenceStep::Implementation,
            Some("Implementation verified".into()),
        );
        assert!(record_res.is_ok());
        let record = record_res.unwrap();
        assert_eq!(record.task_id, 524);

        let mut manifest = TaskEvidenceManifest::default();
        manifest.records.push(record);

        let verify_res = verify_evidence_manifest(&temp_dir, &manifest);
        assert!(verify_res.is_ok());
        let report = verify_res.unwrap();
        assert!(report.is_valid);
        assert_eq!(report.total_records, 1);
        assert_eq!(report.valid_records, 1);
        assert!(report.missing_files.is_empty());
        assert!(report.hash_mismatches.is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_verify_evidence_manifest_mismatch_and_missing() {
        let temp_dir = std::env::temp_dir().join("aios_test_evidence_verify_err");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("docs/tasks/evidence")).unwrap();

        let rel_file = "docs/tasks/evidence/T-00524-mismatch.md";
        std::fs::write(temp_dir.join(rel_file), "Original content").unwrap();

        let record_res = build_evidence_record(
            &temp_dir,
            rel_file,
            524,
            EvidenceStep::Implementation,
            None,
        );
        assert!(record_res.is_ok());
        let record = record_res.unwrap();

        // Mutate file on disk to trigger mismatch
        std::fs::write(temp_dir.join(rel_file), "Tampered content").unwrap();

        let missing_record = EvidenceRecord {
            task_id: 525,
            step: EvidenceStep::UnitTest,
            file_path: "docs/tasks/evidence/T-00525-missing.md".into(),
            sha256_hash: "0000000000000000000000000000000000000000000000000000000000000000".into(),
            timestamp_utc: utcnow_iso(),
            status: "pass".into(),
            summary: None,
        };

        let mut manifest = TaskEvidenceManifest::default();
        manifest.records.push(record);
        manifest.records.push(missing_record);

        let report = verify_evidence_manifest(&temp_dir, &manifest).unwrap();
        assert!(!report.is_valid);
        assert_eq!(report.total_records, 2);
        assert_eq!(report.valid_records, 0);
        assert_eq!(report.missing_files.len(), 1);
        assert_eq!(report.hash_mismatches.len(), 1);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_check_evidence_policy_enforcement() {
        // 1. Read-only unauthenticated tools pass with None
        assert!(check_evidence_policy(None, "aios.evidence.hash").is_ok());
        assert!(check_evidence_policy(None, "aios.evidence.scan").is_ok());
        assert!(check_evidence_policy(None, "aios.evidence.verify").is_ok());
        assert!(check_evidence_policy(None, "evidence.hash").is_ok());
        assert!(check_evidence_policy(None, "evidence.scan").is_ok());
        assert!(check_evidence_policy(None, "evidence.verify").is_ok());

        // 2. Mutating tools fail with None
        assert!(check_evidence_policy(None, "aios.evidence.record").is_err());
        assert!(check_evidence_policy(None, "evidence.record").is_err());
        assert!(check_evidence_policy(None, "aios.evidence.set").is_err());
        assert!(check_evidence_policy(None, "evidence.set").is_err());

        // 3. Whitespace-only tokens fail
        assert!(check_evidence_policy(Some(""), "aios.evidence.record").is_err());
        assert!(check_evidence_policy(Some("   \t\n"), "aios.evidence.record").is_err());
        assert!(check_evidence_policy(Some("   \t\n"), "evidence.set").is_err());

        // 4. Valid PEP grant token passes
        assert!(check_evidence_policy(Some("gr_valid_token_123"), "aios.evidence.record").is_ok());
        assert!(check_evidence_policy(Some("gr_valid_token_123"), "evidence.record").is_ok());
        assert!(check_evidence_policy(Some("gr_valid_token_123"), "aios.evidence.set").is_ok());
        assert!(check_evidence_policy(Some("gr_valid_token_123"), "evidence.set").is_ok());
    }

    #[test]
    fn test_build_evidence_record_invalid_paths_error() {
        let temp_dir = std::env::temp_dir().join("aios_test_evidence_invalid_path");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Empty path
        let err_empty = build_evidence_record(&temp_dir, "   ", 525, EvidenceStep::UnitTest, None);
        assert!(err_empty.is_err());
        assert!(err_empty.unwrap_err().contains("cannot be empty"));

        // Path traversal
        let err_traversal = build_evidence_record(&temp_dir, "../outside.md", 525, EvidenceStep::UnitTest, None);
        assert!(err_traversal.is_err());
        assert!(err_traversal.unwrap_err().contains("cannot escape repository bounds"));

        // Absolute path
        let err_abs = build_evidence_record(&temp_dir, "/etc/shadow", 525, EvidenceStep::UnitTest, None);
        assert!(err_abs.is_err());
        assert!(err_abs.unwrap_err().contains("cannot escape repository bounds"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_compute_file_sha256_with_config_size_limit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.bin");
        std::fs::write(&file_path, vec![b'A'; 2048]).unwrap();

        let mut config = EvidenceConfig::default();
        config.max_file_bytes = 1024; // 1 KiB limit

        let err = compute_file_sha256_with_config(&file_path, &config).unwrap_err();
        assert!(err.contains("exceeds max size cap"));

        config.max_file_bytes = 4096; // 4 KiB limit
        let ok = compute_file_sha256_with_config(&file_path, &config);
        assert!(ok.is_ok());
    }

    #[test]
    fn test_collect_evidence_telemetry() {
        // 1. Happy path report
        let healthy_report = EvidenceVerificationReport {
            total_records: 10,
            valid_records: 10,
            missing_files: vec![],
            hash_mismatches: vec![],
            is_valid: true,
        };
        let telemetry = collect_evidence_telemetry(&healthy_report);
        assert_eq!(telemetry.total_records, 10);
        assert_eq!(telemetry.valid_records, 10);
        assert_eq!(telemetry.missing_files_count, 0);
        assert_eq!(telemetry.hash_mismatches_count, 0);
        assert!(telemetry.is_healthy);

        // 2. Degraded report with missing and mismatched records
        let degraded_report = EvidenceVerificationReport {
            total_records: 10,
            valid_records: 7,
            missing_files: vec!["missing1.md".into(), "missing2.md".into()],
            hash_mismatches: vec!["mismatch.md: expected X found Y".into()],
            is_valid: false,
        };
        let degraded_telemetry = collect_evidence_telemetry(&degraded_report);
        assert_eq!(degraded_telemetry.total_records, 10);
        assert_eq!(degraded_telemetry.valid_records, 7);
        assert_eq!(degraded_telemetry.missing_files_count, 2);
        assert_eq!(degraded_telemetry.hash_mismatches_count, 1);
        assert!(!degraded_telemetry.is_healthy);

        // 3. Empty report boundary
        let empty_report = EvidenceVerificationReport {
            total_records: 0,
            valid_records: 0,
            missing_files: vec![],
            hash_mismatches: vec![],
            is_valid: true,
        };
        let empty_telemetry = collect_evidence_telemetry(&empty_report);
        assert_eq!(empty_telemetry.total_records, 0);
        assert_eq!(empty_telemetry.valid_records, 0);
        assert!(empty_telemetry.is_healthy);

        // 4. All missing boundary
        let all_missing_report = EvidenceVerificationReport {
            total_records: 3,
            valid_records: 0,
            missing_files: vec!["a.md".into(), "b.md".into(), "c.md".into()],
            hash_mismatches: vec![],
            is_valid: false,
        };
        let all_missing_telemetry = collect_evidence_telemetry(&all_missing_report);
        assert_eq!(all_missing_telemetry.missing_files_count, 3);
        assert!(!all_missing_telemetry.is_healthy);

        // 5. JSON serialization & deserialization roundtrip
        let json_str = serde_json::to_string(&telemetry).unwrap();
        let decoded: EvidenceTelemetry = serde_json::from_str(&json_str).unwrap();
        assert_eq!(telemetry, decoded);
    }

    #[test]
    fn test_format_evidence_summary() {
        // 1. Populated manifest with multiple records
        let manifest = TaskEvidenceManifest {
            epic_name: "Evidence & Audit Trail".into(),
            task_range: "T-00511..T-00520".into(),
            generated_at: "2026-08-29T00:00:00Z".into(),
            records: vec![
                EvidenceRecord {
                    task_id: 511,
                    step: EvidenceStep::Research,
                    file_path: "docs/tasks/evidence/T-00511-research.md".into(),
                    sha256_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
                    timestamp_utc: "2026-08-29T00:00:00Z".into(),
                    status: "pass".into(),
                    summary: None,
                },
                EvidenceRecord {
                    task_id: 512,
                    step: EvidenceStep::Spec,
                    file_path: "docs/tasks/evidence/T-00512-spec.md".into(),
                    sha256_hash: "abcd".into(), // short hash boundary
                    timestamp_utc: "2026-08-29T00:00:00Z".into(),
                    status: "pass".into(),
                    summary: None,
                },
            ],
        };
        let formatted = format_evidence_summary(&manifest);
        assert!(formatted.contains("AIOS Task Evidence Manifest (T-00511..T-00520)"));
        assert!(formatted.contains("Epic: Evidence & Audit Trail"));
        assert!(formatted.contains("[T-00511 Research] docs/tasks/evidence/T-00511-research.md (e3b0c442) - pass"));
        assert!(formatted.contains("[T-00512 Spec] docs/tasks/evidence/T-00512-spec.md (abcd) - pass"));

        // 2. Empty manifest
        let empty_manifest = TaskEvidenceManifest {
            epic_name: "Evidence & Audit Trail".into(),
            task_range: "T-00511..T-00520".into(),
            generated_at: "2026-08-29T00:00:00Z".into(),
            records: vec![],
        };
        let empty_formatted = format_evidence_summary(&empty_manifest);
        assert!(empty_formatted.contains("(no evidence records)"));
    }

    #[test]
    fn test_recover_default_evidence_config() {
        let config = recover_default_evidence_config();
        assert_eq!(config.evidence_dir, "docs/tasks/evidence");
        assert_eq!(config.max_file_bytes, 16 * 1024 * 1024);
        assert!(config.enforce_checksum);
        assert_eq!(config.allowed_extensions, vec![".md", ".json"]);
    }

    #[test]
    fn test_reconstruct_and_reconcile_evidence_manifest() {
        let temp_dir = std::env::temp_dir().join("aios_test_reconstruct_evidence");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let evidence_dir = temp_dir.join("docs/tasks/evidence");
        std::fs::create_dir_all(&evidence_dir).unwrap();

        std::fs::write(evidence_dir.join("T-00511-research.md"), "test research evidence").unwrap();
        std::fs::write(evidence_dir.join("T-00512-spec.md"), "test spec evidence").unwrap();

        // Reconstruct manifest from disk
        let manifest = reconstruct_evidence_manifest(&temp_dir, "T-00511..T-00520", "Evidence & Audit Trail").unwrap();
        assert_eq!(manifest.records.len(), 2);
        assert_eq!(manifest.records[0].task_id, 511);
        assert_eq!(manifest.records[0].step, EvidenceStep::Research);
        assert_eq!(manifest.records[1].task_id, 512);
        assert_eq!(manifest.records[1].step, EvidenceStep::Spec);

        // Reconcile manifest
        let (report, telemetry) = reconcile_evidence_manifest(&temp_dir, &manifest).unwrap();
        assert!(report.is_valid);
        assert_eq!(report.total_records, 2);
        assert_eq!(report.valid_records, 2);
        assert!(telemetry.is_healthy);
        assert_eq!(telemetry.total_records, 2);

        // Filtered scan
        let filtered_records = scan_evidence_directory(&temp_dir, Some(511)).unwrap();
        assert_eq!(filtered_records.len(), 1);
        assert_eq!(filtered_records[0].task_id, 511);

        // Degraded reconciliation: tamper with one file
        std::fs::write(evidence_dir.join("T-00511-research.md"), "tampered research evidence").unwrap();
        let (degraded_report, degraded_telemetry) = reconcile_evidence_manifest(&temp_dir, &manifest).unwrap();
        assert!(!degraded_report.is_valid);
        assert_eq!(degraded_report.valid_records, 1);
        assert_eq!(degraded_report.hash_mismatches.len(), 1);
        assert!(!degraded_telemetry.is_healthy);
        assert_eq!(degraded_telemetry.hash_mismatches_count, 1);

        // Non-existent directory error
        let non_existent = temp_dir.join("non_existent_path");
        let err_res = scan_evidence_directory(&non_existent, None);
        assert!(err_res.is_err());
        assert!(err_res.unwrap_err().contains("Evidence directory not found"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
