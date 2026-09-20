//! Unit tests for Capability Documentation Subsystem (CAPDOC1..CAPDOC6).

use aiosh_core::capability_doc::{
    extract_utf8_snippet, CapabilityDocCategory, CapabilityDocIndex, CapabilityDocSearchResult,
    CapabilityDocTopic, MAX_DOC_QUERY_LEN, MAX_TOPIC_ID_LEN,
};

#[test]
fn test_doc_index_canonical_topics_present() {
    let index = CapabilityDocIndex::new();
    let topics = index.list_topics();

    assert_eq!(topics.len(), 8);

    let expected_ids = [
        "cap-overview",
        "cap-rights-scopes",
        "cap-attenuation",
        "cap-constraints",
        "cap-revocation",
        "cap-policy",
        "cap-observability",
        "cap-mcp-tools",
    ];

    for id in &expected_ids {
        let topic = index.get_topic(id);
        assert!(topic.is_some(), "Missing canonical topic: {}", id);
        let t = topic.unwrap();
        assert!(!t.title.is_empty());
        assert!(!t.summary.is_empty());
        assert!(!t.sections.is_empty());
        assert!(!t.tags.is_empty());
        assert!(!t.references.is_empty());
    }
}

#[test]
fn test_doc_index_get_topic() {
    let index = CapabilityDocIndex::new();

    // 1. Exact match
    let topic = index.get_topic("cap-overview");
    assert!(topic.is_some());
    assert_eq!(topic.unwrap().id, "cap-overview");

    // 2. Case-insensitivity & whitespace trimming
    let topic_upper = index.get_topic("  CAP-OVERVIEW  ");
    assert!(topic_upper.is_some());
    assert_eq!(topic_upper.unwrap().id, "cap-overview");

    // 3. Non-existent ID
    assert!(index.get_topic("non-existent-topic").is_none());

    // 4. Oversized ID
    let long_id = "a".repeat(MAX_TOPIC_ID_LEN + 1);
    assert!(index.get_topic(&long_id).is_none());

    // 5. Control characters
    assert!(index.get_topic("cap-overview\x00").is_none());
    assert!(index.get_topic("cap-overview\r\n").is_none());
}

#[test]
fn test_doc_index_list_by_category() {
    let index = CapabilityDocIndex::new();

    let arch_topics = index.list_by_category(CapabilityDocCategory::Architecture);
    assert_eq!(arch_topics.len(), 2); // cap-overview, cap-rights-scopes

    let life_topics = index.list_by_category(CapabilityDocCategory::Lifecycle);
    assert_eq!(life_topics.len(), 3); // cap-attenuation, cap-constraints, cap-revocation

    let sec_topics = index.list_by_category(CapabilityDocCategory::Security);
    assert_eq!(sec_topics.len(), 1); // cap-policy

    let obs_topics = index.list_by_category(CapabilityDocCategory::Observability);
    assert_eq!(obs_topics.len(), 1); // cap-observability

    let ref_topics = index.list_by_category(CapabilityDocCategory::Reference);
    assert_eq!(ref_topics.len(), 1); // cap-mcp-tools
}

#[test]
fn test_doc_index_search_scoring() {
    let index = CapabilityDocIndex::new();

    // Exact ID match should score highest (>= 100)
    let results_id = index.search("cap-attenuation");
    assert!(!results_id.is_empty());
    assert_eq!(results_id[0].topic_id, "cap-attenuation");
    assert!(results_id[0].score >= 100);

    // Tag match
    let results_tag = index.search("zero-ambient-authority");
    assert!(!results_tag.is_empty());
    assert_eq!(results_tag[0].topic_id, "cap-overview");
    assert!(results_tag[0].score >= 50);

    // Search query matches multiple topics, verify sorting
    let results_cap = index.search("capability");
    assert!(results_cap.len() >= 2);
    for i in 0..results_cap.len() - 1 {
        assert!(results_cap[i].score >= results_cap[i + 1].score);
    }
}

#[test]
fn test_doc_index_search_defensive_bounds() {
    let index = CapabilityDocIndex::new();

    // Empty query
    assert!(index.search("").is_empty());
    assert!(index.search("   ").is_empty());

    // Query with control chars
    assert!(index.search("cap\x00overview").is_empty());
    assert!(index.search("test\t\n").is_empty());

    // Oversized query
    let long_query = "a".repeat(MAX_DOC_QUERY_LEN + 1);
    assert!(index.search(&long_query).is_empty());
}

#[test]
fn test_doc_index_utf8_snippet_safety() {
    // Test safe snippet extraction with multi-byte UTF-8 sequences (emojis, CJK, accents)
    let text = "Security 🔐 Kernel with 🚀 high-throughput and 安全 guarantees. Special chars: ääääääääääääääää.";
    
    // Find byte position of emoji or non-ASCII char
    let idx = text.find("🚀").unwrap();
    let snippet = extract_utf8_snippet(text, idx, 1);
    assert!(!snippet.is_empty());
    assert!(snippet.contains("🚀"));

    // Find byte position of CJK
    let idx_cjk = text.find("安全").unwrap();
    let snippet_cjk = extract_utf8_snippet(text, idx_cjk, 2);
    assert!(!snippet_cjk.is_empty());
    assert!(snippet_cjk.contains("安全"));

    // Boundary at start and end
    let snippet_start = extract_utf8_snippet(text, 0, 5);
    assert!(!snippet_start.is_empty());
    let snippet_end = extract_utf8_snippet(text, text.len() - 1, 1);
    assert!(!snippet_end.is_empty());
}

#[test]
fn test_doc_index_markdown_formatting() {
    let index = CapabilityDocIndex::new();
    let topic = index.get_topic("cap-overview").expect("get topic");
    let md = CapabilityDocIndex::format_topic_markdown(topic);

    assert!(md.contains("# Capability Model Overview & Architecture"));
    assert!(md.contains("**ID:** `cap-overview`"));
    assert!(md.contains("**Category:** `architecture`"));
    assert!(md.contains("## Zero Ambient Authority"));
    assert!(md.contains("## Core Invariants (CAP1..CAP6)"));
    assert!(md.contains("## Authoritative References"));
}

#[test]
fn test_doc_index_serde() {
    let index = CapabilityDocIndex::new();
    let topic = index.get_topic("cap-policy").expect("get topic");

    let json = serde_json::to_string(topic).expect("serialize");
    assert!(json.contains("\"category\":\"security\""));

    let deserialized: CapabilityDocTopic = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(topic, &deserialized);

    let search_res = CapabilityDocSearchResult {
        topic_id: "cap-policy".into(),
        title: "Capability Security Policy".into(),
        score: 100,
        snippet: "snippet".into(),
        matched_tags: vec!["policy".into()],
    };
    let res_json = serde_json::to_string(&search_res).expect("serialize search result");
    let res_de: CapabilityDocSearchResult = serde_json::from_str(&res_json).expect("deserialize search result");
    assert_eq!(search_res, res_de);
}
