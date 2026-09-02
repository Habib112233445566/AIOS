use aiosh_core::evidence::{EvidenceStep, TaskEvidenceManifest};
use aiosh_core::evidence_service::{build_evidence_record, verify_evidence_manifest};

#[test]
fn test_evidence_full_lifecycle_e2e() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_root = temp_dir.path();

    let evidence_dir = repo_root.join("docs/tasks/evidence");
    std::fs::create_dir_all(&evidence_dir).unwrap();

    let mut manifest = TaskEvidenceManifest {
        epic_name: "Test Epic".into(),
        task_range: "T-00501..T-00510".into(),
        generated_at: "2026-08-28T12:00:00Z".into(),
        records: Vec::new(),
    };

    // 1. Create mock files and records for all 10 steps
    let steps = [
        (501, EvidenceStep::Research, "T-00501-research.md"),
        (502, EvidenceStep::Spec, "T-00502-spec.md"),
        (503, EvidenceStep::Scaffold, "T-00503-scaffold.md"),
        (504, EvidenceStep::Implementation, "T-00504-impl.md"),
        (505, EvidenceStep::UnitTest, "T-00505-unit-test.md"),
        (506, EvidenceStep::Integration, "T-00506-integration.md"),
        (507, EvidenceStep::SecurityReview, "T-00507-security.md"),
        (508, EvidenceStep::Hardening, "T-00508-hardening.md"),
        (509, EvidenceStep::Documentation, "T-00509-documentation.md"),
        (510, EvidenceStep::Verification, "T-00510-verify.md"),
    ];

    for (tid, step, fname) in &steps {
        let fpath = evidence_dir.join(fname);
        let content = format!("# Evidence for Task {}\nContent for step {:?}", tid, step);
        std::fs::write(&fpath, content).unwrap();

        let rel_path = format!("docs/tasks/evidence/{}", fname);
        let record = build_evidence_record(repo_root, &rel_path, *tid, step.clone(), None).unwrap();
        manifest.records.push(record);
    }

    // 2. Initial verification -> must be fully valid
    let report = verify_evidence_manifest(repo_root, &manifest).unwrap();
    assert!(report.is_valid);
    assert_eq!(report.total_records, 10);
    assert_eq!(report.valid_records, 10);
    assert!(report.missing_files.is_empty());
    assert!(report.hash_mismatches.is_empty());

    // 3. Tamper with one file -> must detect mismatch
    let tampered_file = evidence_dir.join("T-00504-impl.md");
    std::fs::write(&tampered_file, "# Tampered content").unwrap();

    let tampered_report = verify_evidence_manifest(repo_root, &manifest).unwrap();
    assert!(!tampered_report.is_valid);
    assert_eq!(tampered_report.valid_records, 9);
    assert_eq!(tampered_report.hash_mismatches.len(), 1);

    // 4. Delete one file -> must detect missing
    let deleted_file = evidence_dir.join("T-00510-verify.md");
    std::fs::remove_file(&deleted_file).unwrap();

    let missing_report = verify_evidence_manifest(repo_root, &manifest).unwrap();
    assert!(!missing_report.is_valid);
    assert_eq!(missing_report.valid_records, 8);
    assert_eq!(missing_report.missing_files.len(), 1);
    assert_eq!(missing_report.hash_mismatches.len(), 1);
}

#[test]
fn test_evidence_manifest_query_and_filter_e2e() {
    let mut manifest = TaskEvidenceManifest {
        epic_name: "Query Epic".into(),
        task_range: "T-00511..T-00520".into(),
        generated_at: "2026-08-28T12:00:00Z".into(),
        records: Vec::new(),
    };

    manifest.records.push(aiosh_core::evidence::EvidenceRecord {
        task_id: 511,
        step: EvidenceStep::Research,
        file_path: "docs/tasks/evidence/T-00511-research.md".into(),
        sha256_hash: "a".repeat(64),
        timestamp_utc: "2026-08-28T12:00:00Z".into(),
        status: "pass".into(),
        summary: Some("Research summary".into()),
    });

    manifest.records.push(aiosh_core::evidence::EvidenceRecord {
        task_id: 512,
        step: EvidenceStep::Spec,
        file_path: "docs/tasks/evidence/T-00512-spec.md".into(),
        sha256_hash: "b".repeat(64),
        timestamp_utc: "2026-08-28T12:05:00Z".into(),
        status: "pass".into(),
        summary: None,
    });

    // Query by task ID
    assert!(manifest.get_record(511).is_some());
    assert_eq!(manifest.get_record(511).unwrap().step, EvidenceStep::Research);
    assert!(manifest.get_record(999).is_none());

    // Filter by step
    let spec_records = manifest.filter_by_step(&EvidenceStep::Spec);
    assert_eq!(spec_records.len(), 1);
    assert_eq!(spec_records[0].task_id, 512);

    let verify_records = manifest.filter_by_step(&EvidenceStep::Verification);
    assert!(verify_records.is_empty());
}
