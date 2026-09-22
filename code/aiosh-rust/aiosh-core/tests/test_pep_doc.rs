//! Unit tests for PEP Decision Engine Documentation Subsystem (PEPDOC1..PEPDOC6).

use aiosh_core::pep_doc::{
    extract_utf8_snippet, PepDocCategory, PepDocIndex, PepDocSearchResult, PepDocTopic,
    MAX_DOC_QUERY_LEN, MAX_SNIPPET_LEN,
};

#[test]
fn test_pep_doc_index_canonical_topics() {
    let index = PepDocIndex::new();
    let topics = index.list_topics();

    assert!(topics.len() >= 6);

    let expected_ids = [
        "pep-algorithms",
        "pep-arch",
        "pep-cli-mcp",
        "pep-obligations",
        "pep-observability",
        "pep-secpolicy",
    ];

    for id in &expected_ids {
        let topic = index.get_topic(id);
        assert!(topic.is_some(), "Expected canonical topic {} to be present", id);
        let t = topic.unwrap();
        assert!(!t.title.is_empty());
        assert!(!t.summary.is_empty());
        assert!(!t.sections.is_empty());
        assert!(!t.tags.is_empty());
        assert!(!t.examples.is_empty());
    }
}

#[test]
fn test_pep_doc_get_topic_lookup() {
    let index = PepDocIndex::new();

    // Exact match
    assert!(index.get_topic("pep-arch").is_some());

    // Case-insensitive match
    assert!(index.get_topic("PEP-ARCH").is_some());
    assert!(index.get_topic("Pep-Algorithms").is_some());

    // Non-existent ID returns None
    assert!(index.get_topic("nonexistent-topic").is_none());
    assert!(index.get_topic("").is_none());
}

#[test]
fn test_pep_doc_list_topics_sorted() {
    let index = PepDocIndex::new();
    let topics = index.list_topics();

    // Must be sorted lexicographically by ID
    for i in 1..topics.len() {
        assert!(
            topics[i - 1].id <= topics[i].id,
            "Topics must be sorted: {} vs {}",
            topics[i - 1].id,
            topics[i].id
        );
    }
}

#[test]
fn test_pep_doc_list_by_category() {
    let index = PepDocIndex::new();

    let sec_topics = index.list_by_category(PepDocCategory::Security);
    assert!(!sec_topics.is_empty());
    for t in sec_topics {
        assert_eq!(t.category, PepDocCategory::Security);
    }

    let eval_topics = index.list_by_category(PepDocCategory::Evaluation);
    assert!(!eval_topics.is_empty());
    for t in eval_topics {
        assert_eq!(t.category, PepDocCategory::Evaluation);
    }

    let obs_topics = index.list_by_category(PepDocCategory::Observability);
    assert!(!obs_topics.is_empty());
    for t in obs_topics {
        assert_eq!(t.category, PepDocCategory::Observability);
    }
}

#[test]
fn test_pep_doc_search_ranked_scoring() {
    let index = PepDocIndex::new();

    // "Architecture" should match title of pep-arch (+10) and rank first
    let res = index.search("Architecture");
    assert!(!res.is_empty());
    assert_eq!(res[0].topic_id, "pep-arch");
    assert!(res[0].score >= 10);

    // "DenyOverrides" should match in algorithms
    let res_algo = index.search("DenyOverrides");
    assert!(!res_algo.is_empty());
    assert_eq!(res_algo[0].topic_id, "pep-algorithms");

    // Results must be ordered by score descending
    for i in 1..res.len() {
        assert!(res[i - 1].score >= res[i].score);
    }
}

#[test]
fn test_pep_doc_search_empty_and_control_chars() {
    let index = PepDocIndex::new();

    // Empty query returns empty results
    assert!(index.search("").is_empty());
    assert!(index.search("    ").is_empty());

    // Query with control characters is sanitized
    let res_ctrl = index.search("\x00\r\n\tarch\x07");
    assert!(!res_ctrl.is_empty());
    assert_eq!(res_ctrl[0].topic_id, "pep-arch");

    // Over-long query (> 256 chars) is truncated without panic
    let long_query = "a".repeat(MAX_DOC_QUERY_LEN + 100);
    let res_long = index.search(&long_query);
    assert!(res_long.is_empty()); // No matching topics, but succeeds cleanly
}

#[test]
fn test_pep_doc_utf8_snippet_safety() {
    // Multi-byte characters: German umlauts, Japanese Kanji, Emojis
    let content = "AIOS 安全性ポリシー 🚀: Invarianten für PEP-Entscheidungen mit hoher Zuverlässigkeit und Präzision.";

    // Byte index at multi-byte character boundary
    let byte_idx = content.find("ポリシー").unwrap();
    let snippet = extract_utf8_snippet(content, byte_idx, "ポリシー".len());

    assert!(!snippet.is_empty());
    assert!(snippet.len() <= MAX_SNIPPET_LEN + 10);
    assert!(snippet.contains("ポリシー"));

    // Empty content handles safely
    let empty_snippet = extract_utf8_snippet("", 0, 0);
    assert!(empty_snippet.is_empty());
}

#[test]
fn test_pep_doc_json_serialization() {
    let index = PepDocIndex::new();
    let topic = index.get_topic("pep-secpolicy").unwrap();

    let json_str = serde_json::to_string(topic).expect("Topic must serialize to JSON");
    assert!(json_str.contains("\"id\":\"pep-secpolicy\""));
    assert!(json_str.contains("\"category\":\"security\""));

    let deserialized: PepDocTopic = serde_json::from_str(&json_str).expect("Must deserialize");
    assert_eq!(deserialized.id, topic.id);
    assert_eq!(deserialized.category, topic.category);

    let search_res = PepDocSearchResult {
        topic_id: "pep-arch".to_string(),
        title: "PEP Arch".to_string(),
        score: 15,
        snippet: "...overview...".to_string(),
        matched_tags: vec!["pdp".to_string()],
    };
    let res_json = serde_json::to_string(&search_res).expect("Search result must serialize");
    assert!(res_json.contains("\"score\":15"));
}
