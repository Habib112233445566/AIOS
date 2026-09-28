//! Unit and integration tests for Audit Chain Documentation Subsystem (T-02385).

use aiosh_core::audit_chain_doc::{
    AuditChainDocCategory, AuditChainDocIndex, AUDITDOC_ERR_NOT_FOUND, AUDITDOC_ERR_QUERY_BOUNDS,
    MAX_AUDIT_DOC_QUERY_LEN,
};

#[test]
fn test_audit_doc_index_canonical_topics_count() {
    let index = AuditChainDocIndex::new();
    let topics = index.list_topics();
    assert_eq!(topics.len(), 7);

    let topic_ids: Vec<&str> = topics.iter().map(|t| t.id.as_str()).collect();
    assert!(topic_ids.contains(&"audit-arch"));
    assert!(topic_ids.contains(&"audit-lineage"));
    assert!(topic_ids.contains(&"audit-crypto"));
    assert!(topic_ids.contains(&"audit-policy"));
    assert!(topic_ids.contains(&"audit-observability"));
    assert!(topic_ids.contains(&"audit-recovery"));
    assert!(topic_ids.contains(&"audit-reference"));
}

#[test]
fn test_audit_doc_get_topic_positive_and_negative() {
    let index = AuditChainDocIndex::new();

    // Positive
    let topic = index.get_topic("audit-arch").expect("find audit-arch");
    assert_eq!(topic.id, "audit-arch");
    assert!(topic.title.contains("Architecture"));
    assert!(!topic.sections.is_empty());
    assert_eq!(topic.category, AuditChainDocCategory::Architecture);

    // Negative
    assert!(index.get_topic("audit-nonexistent-topic").is_none());
}

#[test]
fn test_audit_doc_search_relevance() {
    let index = AuditChainDocIndex::new();

    // Search for "lineage"
    let results = index.search("lineage").expect("search lineage");
    assert!(!results.is_empty());
    assert_eq!(results[0].topic_id, "audit-lineage");
    assert!(results[0].score >= 8);
    assert!(!results[0].snippet.is_empty());

    // Search for "ed25519"
    let sig_results = index.search("ed25519").expect("search ed25519");
    assert!(!sig_results.is_empty());
    assert_eq!(sig_results[0].topic_id, "audit-crypto");
}

#[test]
fn test_audit_doc_search_bounds() {
    let index = AuditChainDocIndex::new();

    // Empty query rejected
    let err_empty = index.search("   ").unwrap_err();
    assert!(err_empty.contains(AUDITDOC_ERR_QUERY_BOUNDS));

    // Overlong query rejected
    let long_q = "x".repeat(MAX_AUDIT_DOC_QUERY_LEN + 10);
    let err_long = index.search(&long_q).unwrap_err();
    assert!(err_long.contains(AUDITDOC_ERR_QUERY_BOUNDS));
}

#[test]
fn test_audit_doc_render_markdown() {
    let index = AuditChainDocIndex::new();

    // Positive rendering
    let md = index.render_markdown("audit-policy").expect("render markdown");
    assert!(md.contains("# Audit Chain Security Policy"));
    assert!(md.contains("**Category:** `policy`"));
    assert!(md.contains("## Summary"));
    assert!(md.contains("## Examples"));

    // Negative rendering for non-existent topic
    let err = index.render_markdown("ghost-topic").unwrap_err();
    assert!(err.contains(AUDITDOC_ERR_NOT_FOUND));
}

#[test]
fn test_audit_doc_category_filter() {
    let index = AuditChainDocIndex::new();

    let obs_topics = index.topics_by_category(AuditChainDocCategory::Observability);
    assert_eq!(obs_topics.len(), 1);
    assert_eq!(obs_topics[0].id, "audit-observability");

    let rec_topics = index.topics_by_category(AuditChainDocCategory::Recovery);
    assert_eq!(rec_topics.len(), 1);
    assert_eq!(rec_topics[0].id, "audit-recovery");
}
