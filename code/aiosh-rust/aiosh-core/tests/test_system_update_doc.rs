//! Unit tests for System Update Documentation Subsystem (UDOC1..UDOC6).

use tempfile::tempdir;

use aiosh_core::system_update::UpdateSlot;
use aiosh_core::system_update_service::SystemUpdateService;
use aiosh_core::system_update_observability::SystemUpdateObservabilityReport;
use aiosh_core::system_update_doc::{
    SystemUpdateDocCategory, SystemUpdateDocIndex,
};

#[test]
fn test_udoc1_canonical_index_population() {
    let index = SystemUpdateDocIndex::new();
    assert!(index.topics.len() >= 6);

    let topic = index.get_by_id("arch-overview").expect("topic should exist");
    assert_eq!(topic.category, SystemUpdateDocCategory::Architecture);
    assert!(topic.content.contains("A/B dual-partition"));
    assert_eq!(index.get_by_id("nonexistent-topic"), None);
}

#[test]
fn test_udoc2_category_navigation_and_loose_parsing() {
    let index = SystemUpdateDocIndex::new();

    // Category loose string parsing
    assert_eq!(SystemUpdateDocCategory::from_str_loose("arch"), Some(SystemUpdateDocCategory::Architecture));
    assert_eq!(SystemUpdateDocCategory::from_str_loose("slots"), Some(SystemUpdateDocCategory::ABPartitioning));
    assert_eq!(SystemUpdateDocCategory::from_str_loose("policy"), Some(SystemUpdateDocCategory::Security));
    assert_eq!(SystemUpdateDocCategory::from_str_loose("telemetry"), Some(SystemUpdateDocCategory::Observability));
    assert_eq!(SystemUpdateDocCategory::from_str_loose("config"), Some(SystemUpdateDocCategory::Configuration));
    assert_eq!(SystemUpdateDocCategory::from_str_loose("debug"), Some(SystemUpdateDocCategory::Troubleshooting));
    assert_eq!(SystemUpdateDocCategory::from_str_loose("invalid_cat"), None);

    // List by category
    let sec_topics = index.list_by_category(SystemUpdateDocCategory::Security);
    assert_eq!(sec_topics.len(), 1);
    assert_eq!(sec_topics[0].id, "security-policy");

    let trouble_topics = index.list_by_category(SystemUpdateDocCategory::Troubleshooting);
    assert_eq!(trouble_topics.len(), 1);
    assert_eq!(trouble_topics[0].id, "troubleshooting-rollback");
}

#[test]
fn test_udoc3_ranked_search() {
    let index = SystemUpdateDocIndex::new();

    // Query matching ID and tags
    let results = index.search("security-policy");
    assert!(!results.is_empty());
    assert_eq!(results[0].topic_id, "security-policy");
    assert!(results[0].score >= 100);

    // Query matching multiple topics
    let dual_slot_results = index.search("dual-slot");
    assert!(!dual_slot_results.is_empty());
    assert!(dual_slot_results.iter().any(|r| r.topic_id == "arch-overview"));

    // Empty and nonexistent queries
    assert!(index.search("").is_empty());
    assert!(index.search("   ").is_empty());
    assert!(index.search("quantum_flux_capacitor_xyz").is_empty());
}

#[test]
fn test_udoc4_markdown_export() {
    let index = SystemUpdateDocIndex::new();
    let topic = index.get_by_id("ab-slots").unwrap();
    let md = topic.to_markdown();

    assert!(md.contains("# A/B Dual-Slot Partition Layout & Switching"));
    assert!(md.contains("- **Category**: `ab_partitioning`"));
    assert!(md.contains("### See Also"));

    let full_md = index.render_full_index_markdown();
    assert!(full_md.contains("# AIOS System Update Documentation Index"));
    assert!(full_md.contains("## Category: `architecture`"));
    assert!(full_md.contains("## Category: `troubleshooting`"));
}

#[test]
fn test_udoc5_dynamic_rendering() {
    let service = SystemUpdateService::with_defaults("2.1.0", UpdateSlot::SlotA, "2026-09-20T12:00:00Z");
    let status_md = SystemUpdateDocIndex::render_status_markdown(&service);
    eprintln!("DEBUG status_md:\n{}", status_md);
    assert!(status_md.contains("# Live System Update Status Report"));
    assert!(status_md.contains("Slot A: [ACTIVE]"));
    assert!(status_md.contains("Slot B: [INACTIVE]"));
    assert!(status_md.contains("**Current Running Version**: `2.1.0`"));

    let report = SystemUpdateObservabilityReport::generate(&service, None, "2026-09-20T14:30:00Z");
    let obs_md = SystemUpdateDocIndex::render_observability_markdown(&report);

    assert!(obs_md.contains("# System Update Observability Telemetry Summary"));
    assert!(obs_md.contains("- **Health Status**: `HEALTHY`"));
    assert!(obs_md.contains("- **Current Slot**: `SlotA`"));
}

#[test]
fn test_udoc6_file_export_and_hygiene() {
    let dir = tempdir().unwrap();
    let export_path = dir.path().join("docs").join("update_docs.md");

    let index = SystemUpdateDocIndex::new();
    assert!(index.export_to_file(&export_path).is_ok());
    assert!(export_path.exists());

    let content = std::fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("# AIOS System Update Documentation Index"));

    #[cfg(unix)]
    {
        let symlink_path = dir.path().join("symlink.md");
        std::os::unix::fs::symlink(&export_path, &symlink_path).unwrap();
        let res = index.export_to_file(&symlink_path);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("symlink"));
    }
}
