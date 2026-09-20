//! Unit tests for Hardware Detection Documentation Subsystem (HDOC1..HDOC6).

use aiosh_core::hardware_doc::{
    HardwareDocCategory, HardwareDocIndex, MAX_DOC_QUERY_LEN, MAX_TOPIC_ID_LEN,
};

#[test]
fn test_hdoc1_canonical_topics_prepopulated() {
    let index = HardwareDocIndex::new();
    assert_eq!(index.topics.len(), 6);

    for topic in &index.topics {
        assert!(!topic.id.trim().is_empty());
        assert!(!topic.title.trim().is_empty());
        assert!(!topic.summary.trim().is_empty());
        assert!(!topic.sections.is_empty());
        assert!(!topic.tags.is_empty());
    }
}

#[test]
fn test_hdoc2_get_topic_case_insensitive() {
    let index = HardwareDocIndex::new();

    // Exact case
    let topic1 = index.get_topic("hw-sysfs-topology");
    assert!(topic1.is_some());
    assert_eq!(topic1.unwrap().id, "hw-sysfs-topology");

    // Uppercase
    let topic2 = index.get_topic("HW-SYSFS-TOPOLOGY");
    assert!(topic2.is_some());
    assert_eq!(topic2.unwrap().id, "hw-sysfs-topology");

    // Mixed case
    let topic3 = index.get_topic("Hw-SeCuRiTy-PoLiCy");
    assert!(topic3.is_some());
    assert_eq!(topic3.unwrap().id, "hw-security-policy");

    // Missing topic
    assert!(index.get_topic("non-existent-topic").is_none());

    // Control characters rejected
    assert!(index.get_topic("hw-sysfs\x00-topology").is_none());
    assert!(index.get_topic("hw-sysfs\n-topology").is_none());

    // Excessively long ID rejected
    let long_id = "a".repeat(MAX_TOPIC_ID_LEN + 10);
    assert!(index.get_topic(&long_id).is_none());
}

#[test]
fn test_hdoc3_search_scoring_and_ranking() {
    let index = HardwareDocIndex::new();

    // Exact ID match should score highest
    let results = index.search("hw-sysfs-topology", None);
    assert!(!results.is_empty());
    assert_eq!(results[0].topic_id, "hw-sysfs-topology");
    assert!(results[0].score >= 100);

    // Search by tag
    let results_tag = index.search("telemetry", None);
    assert!(!results_tag.is_empty());
    assert_eq!(results_tag[0].topic_id, "hw-observability-telemetry");

    // Search by content keyword
    let results_content = index.search("rotational", None);
    assert!(!results_content.is_empty());
    assert_eq!(results_content[0].topic_id, "hw-sysfs-topology");
}

#[test]
fn test_hdoc3_search_query_bounds() {
    let index = HardwareDocIndex::new();

    // Empty query
    assert!(index.search("", None).is_empty());
    assert!(index.search("   ", None).is_empty());

    // Excessively long query
    let long_query = "x".repeat(MAX_DOC_QUERY_LEN + 10);
    assert!(index.search(&long_query, None).is_empty());

    // Control characters in query
    assert!(index.search("sysfs\x00topology", None).is_empty());
    assert!(index.search("sysfs\ntopology", None).is_empty());
}

#[test]
fn test_hdoc4_category_filtering() {
    let index = HardwareDocIndex::new();

    // List by category
    let sec_topics = index.list_topics(Some(HardwareDocCategory::Security));
    assert_eq!(sec_topics.len(), 1);
    assert_eq!(sec_topics[0].id, "hw-security-policy");

    let disc_topics = index.list_topics(Some(HardwareDocCategory::Discovery));
    assert_eq!(disc_topics.len(), 1);
    assert_eq!(disc_topics[0].id, "hw-sysfs-topology");

    // Search with category filter
    let results_sec = index.search("policy", Some(HardwareDocCategory::Security));
    assert_eq!(results_sec.len(), 1);
    assert_eq!(results_sec[0].topic_id, "hw-security-policy");

    // Search matching text but wrong category returns empty
    let results_wrong = index.search("sysfs", Some(HardwareDocCategory::Security));
    assert!(results_wrong.is_empty());
}

#[test]
fn test_hdoc5_format_topic_markdown() {
    let index = HardwareDocIndex::new();
    let topic = index.get_topic("hw-security-policy").unwrap();
    let md = HardwareDocIndex::format_topic_markdown(topic);

    assert!(md.contains("# Hardware Detection Security Policy and Gatekeeping"));
    assert!(md.contains("**ID:** `hw-security-policy`"));
    assert!(md.contains("**Category:** `security`"));
    assert!(md.contains("## Policy Modes"));
    assert!(md.contains("## Attribute Redaction"));
    assert!(md.contains("## Examples"));
    assert!(md.contains("## Authoritative References"));
}

#[test]
fn test_hdoc6_memory_footprint_and_bounds() {
    let index = HardwareDocIndex::new();
    let serialized = serde_json::to_string(&index.topics).expect("serialize");
    // Index JSON size must be well under 500 KB (actual is ~10-20 KB)
    assert!(serialized.len() < 500_000);
}

#[test]
fn test_hdoc_hardening() {
    let mut index = HardwareDocIndex::new();

    // 1. Invalid topic ID characters (e.g., path traversal, script tags, whitespace)
    assert!(index.get_topic("../hw-sysfs-topology").is_none());
    assert!(index.get_topic("hw<script>").is_none());
    assert!(index.get_topic("hw topic").is_none());
    assert!(index.get_topic("hw;rm -rf").is_none());

    // 2. Overlong category string rejected
    let long_cat = "a".repeat(40);
    assert!(HardwareDocCategory::from_str_loose(&long_cat).is_none());

    // 3. Multi-byte UTF-8 string slicing safety in search snippet
    index.topics.push(aiosh_core::hardware_doc::HardwareDocTopic {
        id: "hw-unicode-test".into(),
        title: "Unicode Test".into(),
        category: HardwareDocCategory::Troubleshooting,
        summary: "Testing multi-byte emojis and UTF-8 characters 🚀🦀⚡️".into(),
        sections: vec![
            aiosh_core::hardware_doc::HardwareDocSection {
                title: "Section with 🦀".into(),
                content: "Prefix characters 🚀🦀⚡️ and then target search keyword here followed by 🌟✨🎉 suffix.".into(),
            },
        ],
        tags: vec!["unicode".into()],
        references: vec![],
        examples: vec![],
    });

    let results = index.search("target search keyword", None);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].topic_id, "hw-unicode-test");
    assert!(results[0].snippet.contains("target search keyword"));
}

