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

    // Non-matching query
    let no_match = index.search("zzzznonexistentterm999");
    assert!(no_match.is_empty());
}

#[test]
fn test_sandbox_doc_categories_and_completeness() {
    let index = SandboxDocIndex::new();
    let topics = index.list_topics();

    for summary in &topics {
        let topic = index.get_topic(&summary.id).expect("topic exists");
        assert!(!topic.title.trim().is_empty(), "Topic title cannot be empty");
        assert!(!topic.summary.trim().is_empty(), "Topic summary cannot be empty");
        assert!(!topic.sections.is_empty(), "Topic must have at least one section");
        for sec in &topic.sections {
            assert!(!sec.title.trim().is_empty());
            assert!(!sec.content.trim().is_empty());
        }
        assert!(!topic.category.as_str().is_empty());
    }
}

#[test]
fn test_sandbox_doc_serialization() {
    let index = SandboxDocIndex::new();
    let topic = index.get_topic("overview").unwrap();

    let json = serde_json::to_string(&topic).expect("serialize topic");
    assert!(json.contains("overview"));

    let deserialized: SandboxDocTopic = serde_json::from_str(&json).expect("deserialize topic");
    assert_eq!(deserialized.id, "overview");
    assert_eq!(deserialized.category, SandboxDocCategory::Architecture);
}

#[test]
fn test_sandbox_doc_hardening() {
    let index = SandboxDocIndex::new();

    // Query with control characters
    let dirty_query = "\x00\t\r\nlandlock\x00\x07";
    let results = index.search(dirty_query);
    assert!(!results.is_empty(), "Should sanitize control chars and find matches");
    assert_eq!(results[0].topic_id, "isolation");

    // Oversized query is clamped without panic
    let long_query = "a".repeat(500);
    let clamped_res = index.search(&long_query);
    assert!(clamped_res.is_empty());

    // Non-existent control-only topic lookup
    assert!(index.get_topic("\x00\x01\x02").is_none());
}


