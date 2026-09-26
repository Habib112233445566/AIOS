//! Unit tests for PEP Grant Lifecycle Documentation Subsystem (T-02285).

use aiosh_core::pep_grant_doc::{
    PepGrantDocIndex, GRANTDOC_ERR_NOT_FOUND, GRANTDOC_ERR_QUERY_BOUNDS,
};

#[test]
fn test_grant_doc_index_canonical_topics_count() {
    let index = PepGrantDocIndex::new();
    let topics = index.list_topics();
    assert_eq!(topics.len(), 7);

    let topic_ids: Vec<&str> = topics.iter().map(|t| t.id.as_str()).collect();
    assert!(topic_ids.contains(&"grant-arch"));
    assert!(topic_ids.contains(&"grant-lifecycle"));
    assert!(topic_ids.contains(&"grant-attenuation"));
    assert!(topic_ids.contains(&"grant-revocation"));
    assert!(topic_ids.contains(&"grant-policy"));
    assert!(topic_ids.contains(&"grant-observability"));
    assert!(topic_ids.contains(&"grant-mcp"));
}

#[test]
fn test_grant_doc_get_topic_positive_and_negative() {
    let index = PepGrantDocIndex::new();

    // Positive
    let topic = index.get_topic("grant-arch").expect("find grant-arch");
    assert_eq!(topic.id, "grant-arch");
    assert!(topic.title.contains("Architecture"));
    assert!(!topic.sections.is_empty());

    // Negative
    assert!(index.get_topic("grant-nonexistent-topic").is_none());
}

#[test]
fn test_grant_doc_search_relevance() {
    let index = PepGrantDocIndex::new();

    // Search for "attenuation"
    let results = index.search("attenuation").expect("search attenuation");
    assert!(!results.is_empty());
    assert_eq!(results[0].topic_id, "grant-attenuation");
    assert!(results[0].score >= 10);
    assert!(!results[0].snippet.is_empty());

    // Search for "cascade"
    let rev_results = index.search("cascade").expect("search cascade");
    assert!(!rev_results.is_empty());
    assert_eq!(rev_results[0].topic_id, "grant-revocation");
}

#[test]
fn test_grant_doc_search_bounds() {
    let index = PepGrantDocIndex::new();

    // Empty query rejected
    let err_empty = index.search("   ").unwrap_err();
    assert!(err_empty.contains(GRANTDOC_ERR_QUERY_BOUNDS));

    // Overlong query rejected
    let long_q = "x".repeat(200);
    let err_long = index.search(&long_q).unwrap_err();
    assert!(err_long.contains(GRANTDOC_ERR_QUERY_BOUNDS));
}

#[test]
fn test_grant_doc_render_markdown() {
    let index = PepGrantDocIndex::new();

    // Positive rendering
    let md = index.render_markdown("grant-policy").expect("render markdown");
    assert!(md.contains("# Grant Security Policy Governance"));
    assert!(md.contains("**Category:** `policy`"));
    assert!(md.contains("## Summary"));
    assert!(md.contains("## Examples"));

    // Negative rendering for non-existent topic
    let err = index.render_markdown("ghost-topic").unwrap_err();
    assert!(err.contains(GRANTDOC_ERR_NOT_FOUND));
}
