//! Unit tests for Network Bootstrap Documentation Subsystem (NDOC1..NDOC6).

use tempfile::tempdir;

use aiosh_core::network::{
    DnsConfig, InterfaceType, IpAddress, NetworkInterface, NetworkState, OperState, Route,
};
use aiosh_core::network_doc::{
    validate_doc_path, NetworkDocCategory, NetworkDocIndex, MAX_DOC_FILE_BYTES, MAX_DOC_QUERY_LEN,
};

fn create_sample_state() -> NetworkState {
    let mut state = NetworkState::new("test-doc-node");

    let mut lo = NetworkInterface::new("lo", InterfaceType::Loopback);
    lo.operstate = OperState::Unknown;
    lo.ip_addresses.push(IpAddress::new_v4("127.0.0.1", 8));
    state.interfaces.push(lo);

    let mut eth0 = NetworkInterface::new("eth0", InterfaceType::Ethernet);
    eth0.operstate = OperState::Up;
    eth0.mac_address = Some("00:11:22:33:44:55".into());
    eth0.ip_addresses.push(IpAddress::new_v4("192.168.1.50", 24));
    state.interfaces.push(eth0);

    let mut route = Route::new("0.0.0.0/0", 100);
    route.gateway = Some("192.168.1.1".into());
    route.interface = Some("eth0".into());
    state.routes.push(route);

    state.dns = DnsConfig::default().with_nameserver("1.1.1.1");
    state
}

#[test]
fn test_ndoc1_canonical_repository() {
    let index = NetworkDocIndex::new();
    assert!(index.topics.len() >= 6);

    let topic = index.get_topic("net-arch-overview");
    assert!(topic.is_some());
    assert_eq!(topic.unwrap().category, NetworkDocCategory::Architecture);
}

#[test]
fn test_ndoc2_category_loose_matching() {
    assert_eq!(NetworkDocCategory::from_str_loose("arch"), Some(NetworkDocCategory::Architecture));
    assert_eq!(NetworkDocCategory::from_str_loose("probe"), Some(NetworkDocCategory::Discovery));
    assert_eq!(NetworkDocCategory::from_str_loose("sec"), Some(NetworkDocCategory::Security));
    assert_eq!(NetworkDocCategory::from_str_loose("metrics"), Some(NetworkDocCategory::Observability));
    assert_eq!(NetworkDocCategory::from_str_loose("cfg"), Some(NetworkDocCategory::Configuration));
    assert_eq!(NetworkDocCategory::from_str_loose("triage"), Some(NetworkDocCategory::Troubleshooting));
    assert_eq!(NetworkDocCategory::from_str_loose("invalid_category"), None);
}

#[test]
fn test_ndoc2_category_filtering() {
    let index = NetworkDocIndex::new();
    let sec_topics = index.list_topics(Some(NetworkDocCategory::Security));
    assert_eq!(sec_topics.len(), 1);
    assert_eq!(sec_topics[0].id, "net-security-policy");

    let all_topics = index.list_topics(None);
    assert_eq!(all_topics.len(), index.topics.len());
}

#[test]
fn test_ndoc3_search_ranking() {
    let index = NetworkDocIndex::new();
    let results = index.search("promiscuous");
    assert!(!results.is_empty());
    assert_eq!(results[0].topic_id, "net-security-policy");
    assert!(results[0].score >= 25);
    assert!(!results[0].snippet.is_empty());

    let sysfs_res = index.search("sysfs");
    assert!(!sysfs_res.is_empty());
    assert_eq!(sysfs_res[0].topic_id, "net-discovery-sysfs");
}

#[test]
fn test_ndoc3_search_empty_and_bounded() {
    let index = NetworkDocIndex::new();
    assert!(index.search("   ").is_empty());

    let long_query = "a".repeat(MAX_DOC_QUERY_LEN + 10);
    assert!(index.search(&long_query).is_empty());
}

#[test]
fn test_ndoc4_render_topic_markdown() {
    let index = NetworkDocIndex::new();
    let md = index.render_topic_markdown("net-arch-overview").unwrap();
    assert!(md.contains("# Network Bootstrap Architecture"));
    assert!(md.contains("**Category:** `architecture`"));
    assert!(md.contains("## Architecture Principles"));
    assert!(md.contains("### Examples"));
    assert!(md.contains("### References"));

    assert!(index.render_topic_markdown("non-existent").is_none());
}

#[test]
fn test_ndoc5_render_state_markdown() {
    let index = NetworkDocIndex::new();
    let state = create_sample_state();
    let md = index.render_state_markdown(&state);

    assert!(md.contains("# AIOS Host Network State Report: test-doc-node"));
    assert!(md.contains("| Interface | Type | OperState | MAC Address | MTU | IP Addresses |"));
    assert!(md.contains("| `eth0` | `Ethernet` | `Up` | `00:11:22:33:44:55` | 1500 | 192.168.1.50/24 |"));
    assert!(md.contains("| `0.0.0.0/0` | `192.168.1.1` | `eth0` | 100 |"));
    assert!(md.contains("- `1.1.1.1`"));
    assert!(md.contains("## Network Topology Diagram"));
}

#[test]
fn test_ndoc5_render_ascii_topology() {
    let index = NetworkDocIndex::new();
    let state = create_sample_state();
    let ascii = index.render_ascii_topology(&state);

    assert!(ascii.contains("[Host: test-doc-node]"));
    assert!(ascii.contains("[lo]"));
    assert!(ascii.contains("[eth0]"));
    assert!(ascii.contains("[Routes]"));
    assert!(ascii.contains("[DNS Resolvers]"));
}

#[test]
fn test_ndoc6_persistence_atomic_and_path_hygiene() {
    let dir = tempdir().unwrap();
    let doc_path = dir.path().join("net_report.md");
    let index = NetworkDocIndex::new();
    let state = create_sample_state();
    let md = index.render_state_markdown(&state);

    index.save_to_path(&md, &doc_path).unwrap();
    assert!(doc_path.exists());

    let loaded = NetworkDocIndex::load_from_path(&doc_path).unwrap();
    assert_eq!(md, loaded);

    // Path hygiene checks
    assert!(validate_doc_path(&dir.path().join("../bad.md")).is_err());
    assert!(validate_doc_path(&dir.path().join("bad\0null.md")).is_err());
}

#[test]
fn test_ndoc6_oversized_document_rejected() {
    let dir = tempdir().unwrap();
    let doc_path = dir.path().join("big_report.md");
    let index = NetworkDocIndex::new();
    let big_content = "x".repeat((MAX_DOC_FILE_BYTES + 10) as usize);

    let res = index.save_to_path(&big_content, &doc_path);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("exceeds limit"));
}

#[test]
fn test_ndoc_hardening_features() {
    use aiosh_core::network_doc::{sanitize_table_cell, NetworkDocTopic, NetworkDocCategory};

    // 1. Table cell sanitization
    assert_eq!(sanitize_table_cell("eth0|evil\r\n"), "eth0\\|evil");
    assert_eq!(sanitize_table_cell("lo\0control"), "locontrol");

    // 2. UTF-8 multi-byte snippet safety
    let mut index = NetworkDocIndex::new();
    let multibyte_summary = "🦀".repeat(150); // 150 crab emojis (4 bytes each)
    index.topics.push(NetworkDocTopic {
        id: "net-emoji-topic".into(),
        title: "Emoji Topic".into(),
        category: NetworkDocCategory::Troubleshooting,
        summary: multibyte_summary,
        sections: Vec::new(),
        tags: vec!["emoji".into()],
        references: Vec::new(),
        examples: Vec::new(),
    });

    let results = index.search("emoji");
    assert_eq!(results.len(), 1);
    assert!(results[0].snippet.ends_with("..."));

    // 3. Query term bounds (> 16 terms)
    let long_query = "term ".repeat(30);
    let bounded_results = index.search(&long_query);
    // Should execute safely without error
    let _ = bounded_results;
}
