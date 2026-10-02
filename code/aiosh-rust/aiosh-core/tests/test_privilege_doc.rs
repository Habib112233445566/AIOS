//! Unit and Integration Tests for Privilege Escalation Prevention Documentation (T-02585).

use aiosh_core::privilege_doc::*;

#[test]
fn test_privilege_doc_canonical_topics_present() {
    let index = PrivilegeDocIndex::new();
    let topics = index.list_topics();

    assert!(topics.len() >= 6);
    let ids: Vec<&str> = topics.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains(&"priv-arch"));
    assert!(ids.contains(&"priv-lifecycle"));
    assert!(ids.contains(&"priv-kernel-lockout"));
    assert!(ids.contains(&"priv-policy"));
    assert!(ids.contains(&"priv-observability"));
    assert!(ids.contains(&"priv-recovery"));
}

#[test]
fn test_privilege_doc_get_topic() {
    let index = PrivilegeDocIndex::new();

    // Positive
    let arch = index.get_topic("priv-arch").expect("find priv-arch");
    assert_eq!(arch.category, PrivilegeDocCategory::Architecture);
    assert!(!arch.summary.is_empty());
    assert!(!arch.sections.is_empty());

    // Negative
    assert!(index.get_topic("non-existent-topic").is_none());
}

#[test]
fn test_privilege_doc_search_scoring() {
    let index = PrivilegeDocIndex::new();

    // Search for kernel
    let results = index.search("kernel").expect("search kernel");
    assert!(!results.is_empty());
    assert_eq!(results[0].topic_id, "priv-kernel-lockout");
    assert!(results[0].score > 0);

    // Multi-term search
    let results2 = index.search("policy enforcement").expect("search policy");
    assert!(!results2.is_empty());
    assert_eq!(results2[0].topic_id, "priv-policy");
}

#[test]
fn test_privilege_doc_search_boundaries() {
    let index = PrivilegeDocIndex::new();

    // Empty query fails
    let err_empty = index.search("   ").unwrap_err();
    assert!(err_empty.contains(PRIVDOC_ERR_QUERY_BOUNDS));

    // Oversized query fails
    let long_q = "a".repeat(MAX_PRIVILEGE_DOC_QUERY_LEN + 1);
    let err_long = index.search(&long_q).unwrap_err();
    assert!(err_long.contains(PRIVDOC_ERR_QUERY_BOUNDS));
}

#[test]
fn test_privilege_doc_markdown_rendering() {
    let index = PrivilegeDocIndex::new();

    // Positive render
    let md = index.render_markdown("priv-arch").expect("render markdown");
    assert!(md.contains("# Privilege Escalation Prevention Architecture"));
    assert!(md.contains("**Category:** `architecture`"));
    assert!(md.contains("## Summary"));
    assert!(md.contains("## Examples"));

    // Negative render
    let err = index.render_markdown("invalid-id").unwrap_err();
    assert!(err.contains(PRIVDOC_ERR_NOT_FOUND));
}

#[test]
fn test_privilege_doc_category_strings() {
    assert_eq!(PrivilegeDocCategory::Architecture.as_str(), "architecture");
    assert_eq!(PrivilegeDocCategory::Lifecycle.as_str(), "lifecycle");
    assert_eq!(PrivilegeDocCategory::Policy.as_str(), "policy");
    assert_eq!(PrivilegeDocCategory::Observability.as_str(), "observability");
    assert_eq!(PrivilegeDocCategory::Security.as_str(), "security");
    assert_eq!(PrivilegeDocCategory::Recovery.as_str(), "recovery");
}

#[test]
fn test_privilege_doc_hardening_bounds() {
    let index = PrivilegeDocIndex::new();

    // Query with control characters rejected
    let bad_q = "kernel\x00injection";
    let res = index.search(bad_q);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("control characters"));

    // Traversal or oversized topic ID rejected
    assert!(index.get_topic("../priv-arch").is_none());
    assert!(index.get_topic("priv-arch\x00").is_none());
    assert!(index.get_topic(&"a".repeat(70)).is_none());
    assert!(index.render_markdown("../priv-arch").is_err());
}
