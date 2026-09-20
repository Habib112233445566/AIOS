//! Unit tests for Kernel Module Documentation Subsystem (KD1..KD6).

use aiosh_core::kernel_module_doc::{DocCategory, DocTopic, KernelModuleDocIndex};

#[test]
fn test_kd1_index_initialization_and_canonical_topics() {
    let index = KernelModuleDocIndex::new();
    let topics = index.list_topics();
    assert!(topics.len() >= 7, "must have at least 7 canonical topics");

    let expected_ids = [
        "modprobe-directives",
        "cis-benchmark-hardening",
        "lifecycle-workflows",
        "observability-and-procfs",
        "security-policy-and-pep",
        "container-isolation",
        "wireless-pentest",
    ];

    for expected_id in expected_ids {
        assert!(
            topics.iter().any(|t| t.id == expected_id),
            "missing canonical topic '{}'",
            expected_id
        );
    }
}

#[test]
fn test_kd2_topic_lookup_and_case_insensitivity() {
    let index = KernelModuleDocIndex::new();

    // Exact lowercase
    let t1 = index.get_topic("modprobe-directives");
    assert!(t1.is_some());
    let t1 = t1.unwrap();
    assert_eq!(t1.id, "modprobe-directives");
    assert_eq!(t1.category, DocCategory::Directive);

    // Uppercase
    let t2 = index.get_topic("MODPROBE-DIRECTIVES");
    assert!(t2.is_some());
    assert_eq!(t2.unwrap().id, "modprobe-directives");

    // Nonexistent topic
    let missing = index.get_topic("nonexistent-kernel-module-topic");
    assert!(missing.is_none());
}

#[test]
fn test_kd3_search_scoring_and_ranking() {
    let index = KernelModuleDocIndex::new();

    // Search by exact ID keyword
    let results = index.search("modprobe-directives");
    assert!(!results.is_empty());
    assert_eq!(results[0].topic_id, "modprobe-directives");
    assert!(results[0].score >= 100);

    // Search by tag keyword
    let cis_results = index.search("cramfs");
    assert!(!cis_results.is_empty());
    assert_eq!(cis_results[0].topic_id, "cis-benchmark-hardening");

    // Search with empty query returns empty list
    let empty_results = index.search("   ");
    assert!(empty_results.is_empty());
}

#[test]
fn test_kd4_category_filtering() {
    let index = KernelModuleDocIndex::new();

    let sec_topics = index.list_by_category(DocCategory::Security);
    assert!(!sec_topics.is_empty());
    for t in sec_topics {
        assert_eq!(t.category, DocCategory::Security);
    }

    let base_topics = index.list_by_category(DocCategory::Baseline);
    assert_eq!(base_topics.len(), 2); // container-isolation and wireless-pentest
}

#[test]
fn test_kd5_markdown_rendering_quality() {
    let index = KernelModuleDocIndex::new();
    let topic = index.get_topic("cis-benchmark-hardening").expect("cis topic");

    let md = KernelModuleDocIndex::format_topic_markdown(topic);
    assert!(md.contains("# CIS Linux Benchmark Kernel Module Hardening"));
    assert!(md.contains("**ID:** `cis-benchmark-hardening`"));
    assert!(md.contains("## Legacy Filesystem Disabling"));
    assert!(md.contains("```bash"));
    assert!(md.contains("install cramfs /bin/true"));
    assert!(md.contains("## Authoritative References"));
    assert!(md.contains("CIS Distribution Benchmark"));
    assert!(md.contains("**Tags:**"));
}

#[test]
fn test_kd6_serialization_and_json_export() {
    let index = KernelModuleDocIndex::new();
    let topic = index.get_topic("container-isolation").expect("container topic");

    let serialized = serde_json::to_string(&topic).expect("serialize topic");
    assert!(serialized.contains("container-isolation"));
    assert!(serialized.contains("metacopy=on"));

    let deserialized: DocTopic = serde_json::from_str(&serialized).expect("deserialize topic");
    assert_eq!(deserialized, *topic);

    // Test search results serialization
    let results = index.search("wireless");
    assert!(!results.is_empty());
    let res_json = serde_json::to_string(&results).expect("serialize results");
    assert!(res_json.contains("wireless-pentest"));
}
