//! Unit tests for Sandbox Documentation Subsystem (T-02484).

use aiosh_core::sandbox_doc::*;

#[test]
fn test_sandbox_doc_list_topics() {
    let index = SandboxDocIndex::new();
    let topics = index.list_topics();
    assert!(topics.len() >= 6, "Expected at least 6 canonical topics");

    let ids: Vec<&str> = topics.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains(&"overview"));
    assert!(ids.contains(&"profiles"));
    assert!(ids.contains(&"isolation"));
    assert!(ids.contains(&"policy"));
    assert!(ids.contains(&"observability"));
    assert!(ids.contains(&"reference"));
}

#[test]
fn test_sandbox_doc_get_topic() {
    let index = SandboxDocIndex::new();
    let topic = index.get_topic("profiles").expect("topic exists");
    assert_eq!(topic.id, "profiles");
    assert_eq!(topic.category, SandboxDocCategory::Profiles);
    assert!(!topic.sections.is_empty());
    assert!(!topic.tags.is_empty());
    assert!(!topic.examples.is_empty());

    // Case-insensitivity and trim
    let topic_upper = index.get_topic("   PROFILES   ").expect("lookup case-insensitive");
    assert_eq!(topic_upper.id, "profiles");

    // Missing topic
    assert!(index.get_topic("unknown_topic_xyz").is_none());
}

#[test]
fn test_sandbox_doc_search() {
    let index = SandboxDocIndex::new();

    // Query matching isolation (Landlock)
    let results = index.search("landlock");
    assert!(!results.is_empty(), "Expected results for 'landlock'");
    assert_eq!(results[0].topic_id, "isolation");
    assert!(results[0].score > 0);
    assert!(!results[0].snippet.is_empty());

    // Query matching policy (denylist)
    let policy_res = index.search("denylist");
    assert!(!policy_res.is_empty());
    assert_eq!(policy_res[0].topic_id, "policy");

    // Empty query returns empty results
    let empty_res = index.search("   ");
    assert!(empty_res.is_empty());
}
