//! Capability Model Documentation and Reference Subsystem (CAPDOC1..CAPDOC6).
//!
//! Provides an offline, self-contained documentation repository for the AIOS
//! Capability Model, covering zero ambient authority architecture, rights and scopes,
//! monotonic attenuation, constraints, cascade revocation, security policy,
//! observability, and MCP tool interfaces.

use serde::{Deserialize, Serialize};

/// Categories for capability documentation topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDocCategory {
    Architecture,
    Lifecycle,
    Security,
    Observability,
    Reference,
}

impl CapabilityDocCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            CapabilityDocCategory::Architecture => "architecture",
            CapabilityDocCategory::Lifecycle => "lifecycle",
            CapabilityDocCategory::Security => "security",
            CapabilityDocCategory::Observability => "observability",
            CapabilityDocCategory::Reference => "reference",
        }
    }
}

/// A structured section within a capability documentation topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDocSection {
    pub title: String,
    pub content: String,
}

/// A complete capability documentation topic with metadata, sections, examples, and references.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDocTopic {
    pub id: String,
    pub title: String,
    pub category: CapabilityDocCategory,
    pub summary: String,
    pub sections: Vec<CapabilityDocSection>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub examples: Vec<String>,
}

/// Scored search result returned from `CapabilityDocIndex::search`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
    pub matched_tags: Vec<String>,
}

/// Maximum allowed length for search queries (256 characters).
pub const MAX_DOC_QUERY_LEN: usize = 256;

/// Maximum number of search results returned (50).
pub const MAX_DOC_SEARCH_RESULTS: usize = 50;

/// Maximum length for topic identifiers (64 characters).
pub const MAX_TOPIC_ID_LEN: usize = 64;

/// Maximum length for extracted contextual snippets (160 characters).
pub const MAX_SNIPPET_LEN: usize = 160;

/// Safely extracts a snippet around byte position `byte_idx` in `content`, respecting UTF-8 char boundaries (CAPDOC4).
pub fn extract_utf8_snippet(content: &str, byte_idx: usize, query_char_len: usize) -> String {
    let char_indices: Vec<(usize, char)> = content.char_indices().collect();
    if char_indices.is_empty() {
        return String::new();
    }

    let match_char_pos = char_indices
        .iter()
        .position(|(b, _)| *b >= byte_idx)
        .unwrap_or(char_indices.len() - 1);

    let start_char_pos = match_char_pos.saturating_sub(25);
    let end_char_pos = (match_char_pos + query_char_len + 55).min(char_indices.len());

    let start_byte = char_indices[start_char_pos].0;
    let end_byte = if end_char_pos < char_indices.len() {
        char_indices[end_char_pos].0
    } else {
        content.len()
    };

    let slice = &content[start_byte..end_byte];
    let prefix = if start_byte > 0 { "..." } else { "" };
    let suffix = if end_byte < content.len() { "..." } else { "" };
    format!("{}{}{}", prefix, slice, suffix)
}

/// Offline in-memory index of Capability Model documentation.
#[derive(Debug, Clone)]
pub struct CapabilityDocIndex {
    pub topics: Vec<CapabilityDocTopic>,
}

impl Default for CapabilityDocIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityDocIndex {
    /// Creates a new capability documentation index pre-populated with canonical topics (CAPDOC1).
    pub fn new() -> Self {
        let mut idx = CapabilityDocIndex {
            topics: Vec::new(),
        };
        idx.register_canonical_topics();
        idx
    }

    /// Registers canonical capability documentation topics into the index (CAPDOC1).
    pub fn register_canonical_topics(&mut self) {
        self.topics.push(CapabilityDocTopic {
            id: "cap-overview".to_string(),
            title: "Capability Model Overview & Architecture".to_string(),
            category: CapabilityDocCategory::Architecture,
            summary: "Zero ambient authority security architecture, unforgeable capability tokens, and core invariants CAP1..CAP6.".to_string(),
            sections: vec![
                CapabilityDocSection {
                    title: "Zero Ambient Authority".to_string(),
                    content: "In AIOS, entities possess zero ambient authority. Access to filesystem paths, network endpoints, tools, or processes strictly requires presenting an explicit, unforgeable capability token.".to_string(),
                },
                CapabilityDocSection {
                    title: "Core Invariants (CAP1..CAP6)".to_string(),
                    content: "CAP1 (Unforgeability): SHA-256 tokens. CAP2 (Scoping): Explicit targets & rights. CAP3 (Monotonic Attenuation): Rights & scopes can only shrink. CAP4 (Constraints): Saturated temporal and byte quotas. CAP5 (Revocation): Immediate cascade revocation. CAP6 (Serialization): Deterministic JSON.".to_string(),
                },
            ],
            tags: vec!["overview".into(), "architecture".into(), "zero-ambient-authority".into(), "cap1".into(), "cap2".into(), "cap3".into(), "cap4".into(), "cap5".into(), "cap6".into()],
            references: vec!["docs/capability_model.md#1-executive-summary--architectural-overview".into(), "docs/capability_model.md#2-capability-model-invariants-cap1---cap6".into()],
            examples: vec!["aios.capability.issue".into(), "aios.capability.check".into()],
        });

        self.topics.push(CapabilityDocTopic {
            id: "cap-rights-scopes".to_string(),
            title: "Capability Rights and Scopes".to_string(),
            category: CapabilityDocCategory::Architecture,
            summary: "Formal specification of CapabilityRight variants and CapabilityScope resource targets.".to_string(),
            sections: vec![
                CapabilityDocSection {
                    title: "Rights (CapabilityRight)".to_string(),
                    content: "CapabilityRight variants: Read (inspect/read), Write (create/modify), Execute (execute binary or tool), Delete (remove file or terminate process), Admin (manage policy or audit), Delegate (attenuate child capabilities).".to_string(),
                },
                CapabilityDocSection {
                    title: "Scopes (CapabilityScope)".to_string(),
                    content: "CapabilityScope variants: Filesystem (path, recursive), Network (host, port, protocol), Tool (tool_name, allowed_actions), Process (executable, max_memory_bytes), Ipc (channel), System (subsystem).".to_string(),
                },
            ],
            tags: vec!["rights".into(), "scopes".into(), "filesystem".into(), "network".into(), "tool".into(), "process".into(), "ipc".into(), "system".into(), "delegate".into()],
            references: vec!["docs/capability_model.md#3-data-model-specifications".into()],
            examples: vec!["CapabilityScope::Filesystem { path: \"/workspace\".into(), recursive: true }".into()],
        });

        self.topics.push(CapabilityDocTopic {
            id: "cap-attenuation".to_string(),
            title: "Monotonic Attenuation & Delegation".to_string(),
            category: CapabilityDocCategory::Lifecycle,
            summary: "Rules and algorithms for deriving child capabilities from parent capabilities without privilege amplification.".to_string(),
            sections: vec![
                CapabilityDocSection {
                    title: "Attenuation Invariant (CAP3)".to_string(),
                    content: "When deriving a child capability, child rights must be a subset of parent rights, child scope must be confined within parent scope, and child quotas cannot exceed parent quotas.".to_string(),
                },
                CapabilityDocSection {
                    title: "Delegation Right Requirement".to_string(),
                    content: "The parent capability must hold CapabilityRight::Delegate. Without Delegate, any attenuation attempt is rejected immediately with Unauthorized.".to_string(),
                },
            ],
            tags: vec!["attenuation".into(), "delegation".into(), "monotonic".into(), "privilege-reduction".into(), "cap3".into(), "subset".into()],
            references: vec!["docs/capability_model.md#4-operational-semantics--attenuation".into()],
            examples: vec!["service.attenuate_capability(parent_id, child_subject, None, vec![CapabilityRight::Read], None)".into()],
        });

        self.topics.push(CapabilityDocTopic {
            id: "cap-constraints".to_string(),
            title: "Temporal Bounds and Resource Quotas".to_string(),
            category: CapabilityDocCategory::Lifecycle,
            summary: "Enforcement of not_before, expires_at, max_invocations, and quota_bytes with saturated arithmetic.".to_string(),
            sections: vec![
                CapabilityDocSection {
                    title: "Temporal Constraints".to_string(),
                    content: "Capabilities support not_before and expires_at RFC 3339 timestamps. Operations before not_before or after expires_at fail with Expired or NotYetValid.".to_string(),
                },
                CapabilityDocSection {
                    title: "Resource Quotas (CAP4)".to_string(),
                    content: "Capabilities enforce max_invocations and quota_bytes. Consumptions use saturating arithmetic to prevent integer overflow. Exceeding limits returns QuotaExceeded.".to_string(),
                },
            ],
            tags: vec!["constraints".into(), "temporal".into(), "quota".into(), "expires_at".into(), "invocations".into(), "bytes".into(), "cap4".into()],
            references: vec!["docs/capability_model.md#33-constraints--lifecycle-capabilityconstraints".into()],
            examples: vec!["CapabilityConstraints { max_invocations: Some(10), quota_bytes: Some(1024), ..Default::default() }".into()],
        });

        self.topics.push(CapabilityDocTopic {
            id: "cap-revocation".to_string(),
            title: "Capability Revocation & Lineage Tracking".to_string(),
            category: CapabilityDocCategory::Lifecycle,
            summary: "Immediate revocation and transitive cascade revocation across derivation hierarchies.".to_string(),
            sections: vec![
                CapabilityDocSection {
                    title: "Explicit Revocation (CAP5)".to_string(),
                    content: "Revoking a capability immediately marks it revoked. Any subsequent validity check, invocation, or byte consumption returns Revoked.".to_string(),
                },
                CapabilityDocSection {
                    title: "Cascade Revocation".to_string(),
                    content: "Cascade revocation traverses the delegation graph via parent_id lineage, revoking all child and descendant capabilities recursively.".to_string(),
                },
            ],
            tags: vec!["revocation".into(), "cascade".into(), "lineage".into(), "parent_id".into(), "cap5".into(), "lifecycle".into()],
            references: vec!["docs/capability_model.md#5-revocation--lineage".into()],
            examples: vec!["service.revoke_capability(&cap_id)".into(), "service.cascade_revoke(&cap_id)".into()],
        });

        self.topics.push(CapabilityDocTopic {
            id: "cap-policy".to_string(),
            title: "Capability Security Policy & MAC Invariants".to_string(),
            category: CapabilityDocCategory::Security,
            summary: "Mandatory Access Control rules, policy modes (Enforcing, Audit, Permissive), and CAPSEC1..CAPSEC6 invariants.".to_string(),
            sections: vec![
                CapabilityDocSection {
                    title: "Policy Modes (CAPSEC1)".to_string(),
                    content: "Enforcing mode blocks policy violations immediately. Audit mode logs warnings but permits issuance. Permissive mode disables checks for testing.".to_string(),
                },
                CapabilityDocSection {
                    title: "Path & Host Prohibitions (CAPSEC3)".to_string(),
                    content: "Normalizes paths against path traversal (..) and blocks access to sensitive system paths (/etc, /proc, C:\\Windows) and cloud metadata hosts (169.254.169.254).".to_string(),
                },
            ],
            tags: vec!["policy".into(), "security".into(), "capsec1".into(), "capsec2".into(), "capsec3".into(), "capsec4".into(), "capsec5".into(), "capsec6".into(), "mac".into()],
            references: vec!["docs/capability_model.md#12-capability-security-policy-capsec1capsec6".into()],
            examples: vec!["CapabilitySecurityPolicy::default().with_mode(CapabilityPolicyMode::Enforcing)".into()],
        });

        self.topics.push(CapabilityDocTopic {
            id: "cap-observability".to_string(),
            title: "Capability Observability & Telemetry".to_string(),
            category: CapabilityDocCategory::Observability,
            summary: "Point-in-time state aggregation, memoized lineage depth calculation, quota metrics, and health evaluation (CAPOBS1..CAPOBS6).".to_string(),
            sections: vec![
                CapabilityDocSection {
                    title: "Observability Invariants (CAPOBS1..CAPOBS6)".to_string(),
                    content: "Aggregates total, active, revoked, expired, root, and attenuated counts. Calculates max derivation depth in O(N) via memoization. Tracks quota totals and evaluates health.".to_string(),
                },
                CapabilityDocSection {
                    title: "Telemetry Sanitization (CAPOBS6)".to_string(),
                    content: "Strips control characters, bounds string lengths to 256 chars, and provides RFC 3339 UTC fallback for empty timestamps.".to_string(),
                },
            ],
            tags: vec!["observability".into(), "telemetry".into(), "metrics".into(), "health".into(), "capobs1".into(), "capobs2".into(), "capobs3".into(), "capobs4".into(), "capobs5".into(), "capobs6".into()],
            references: vec!["docs/capability_model.md#13-capability-observability-subsystem-capobs1capobs6".into()],
            examples: vec!["CapabilityObservabilityReport::generate(&service, \"\")".into()],
        });

        self.topics.push(CapabilityDocTopic {
            id: "cap-mcp-tools".to_string(),
            title: "Capability MCP Tool Surface".to_string(),
            category: CapabilityDocCategory::Reference,
            summary: "Complete reference for MCP tools exposing capability management over JSON-RPC.".to_string(),
            sections: vec![
                CapabilityDocSection {
                    title: "Core Management Tools".to_string(),
                    content: "aios.capability.issue (issue root), aios.capability.attenuate (derive child), aios.capability.check (validate token), aios.capability.revoke (revoke single/tree), aios.capability.list (query registry).".to_string(),
                },
                CapabilityDocSection {
                    title: "Observability & Documentation Tools".to_string(),
                    content: "aios.capability.observability (generate telemetry report), aios.capability.doc (list, get, and search capability reference documentation).".to_string(),
                },
            ],
            tags: vec!["mcp".into(), "tools".into(), "jsonrpc".into(), "aios.capability.issue".into(), "aios.capability.check".into(), "aios.capability.doc".into()],
            references: vec!["docs/capability_model.md#10-mcp-interface-specification".into(), "docs/capability_model.md#133-mcp-integration-aioscapabilityobservability".into()],
            examples: vec!["{\"method\": \"tools/call\", \"params\": {\"name\": \"aios.capability.doc\", \"arguments\": {\"action\": \"list\"}}}".into()],
        });
    }

    /// Looks up a topic by ID (case-insensitive) with defensive bounds (CAPDOC2).
    pub fn get_topic(&self, id: &str) -> Option<&CapabilityDocTopic> {
        if id.chars().any(|c| c.is_control()) {
            return None;
        }
        let id_clean = id.trim();
        if id_clean.is_empty() || id_clean.len() > MAX_TOPIC_ID_LEN {
            return None;
        }
        self.topics.iter().find(|t| t.id.eq_ignore_ascii_case(id_clean))
    }

    /// Lists all topics in the index (CAPDOC1).
    pub fn list_topics(&self) -> Vec<&CapabilityDocTopic> {
        self.topics.iter().collect()
    }

    /// Lists topics belonging to a specific category.
    pub fn list_by_category(&self, category: CapabilityDocCategory) -> Vec<&CapabilityDocTopic> {
        self.topics.iter().filter(|t| t.category == category).collect()
    }

    /// Searches documentation topics with scored ranking, defensive bounds, and UTF-8 safe snippets (CAPDOC3..CAPDOC5).
    pub fn search(&self, query: &str) -> Vec<CapabilityDocSearchResult> {
        if query.chars().any(|c| c.is_control()) {
            return Vec::new();
        }
        let trimmed = query.trim();
        if trimmed.is_empty() || trimmed.len() > MAX_DOC_QUERY_LEN {
            return Vec::new();
        }
        let query_clean = trimmed.to_ascii_lowercase();
        let query_char_len = query_clean.chars().count();

        let mut results = Vec::new();
        for topic in &self.topics {
            let mut score = 0;
            let mut matched_tags = Vec::new();
            let mut snippet = String::new();

            // 1. Exact or partial ID match
            let topic_id_lower = topic.id.to_ascii_lowercase();
            if topic_id_lower == query_clean {
                score += 100;
            } else if topic_id_lower.contains(&query_clean) {
                score += 40;
            }

            // 2. Tag matches
            for tag in &topic.tags {
                let tag_lower = tag.to_ascii_lowercase();
                if tag_lower == query_clean {
                    score += 50;
                    matched_tags.push(tag.clone());
                } else if tag_lower.contains(&query_clean) {
                    score += 20;
                    matched_tags.push(tag.clone());
                }
            }

            // 3. Title match
            let title_lower = topic.title.to_ascii_lowercase();
            if title_lower.contains(&query_clean) {
                score += 25;
            }

            // 4. Summary match
            let summary_lower = topic.summary.to_ascii_lowercase();
            if let Some(idx) = summary_lower.find(&query_clean) {
                score += 15;
                if snippet.is_empty() {
                    snippet = extract_utf8_snippet(&topic.summary, idx, query_char_len);
                }
            }

            // 5. Section matches
            for sec in &topic.sections {
                if sec.title.to_ascii_lowercase().contains(&query_clean) {
                    score += 10;
                }
                let content_lower = sec.content.to_ascii_lowercase();
                if let Some(idx) = content_lower.find(&query_clean) {
                    score += 10;
                    if snippet.is_empty() {
                        snippet = extract_utf8_snippet(&sec.content, idx, query_char_len);
                    }
                }
            }

            if score > 0 {
                if snippet.is_empty() {
                    snippet = topic.summary.clone();
                }
                results.push(CapabilityDocSearchResult {
                    topic_id: topic.id.clone(),
                    title: topic.title.clone(),
                    score,
                    snippet,
                    matched_tags,
                });
            }
        }

        // Sort descending by score, then ascending by topic ID for determinism
        results.sort_by(|a, b| {
            b.score.cmp(&a.score).then_with(|| a.topic_id.cmp(&b.topic_id))
        });

        results.truncate(MAX_DOC_SEARCH_RESULTS);
        results
    }

    /// Formats a topic as readable Markdown.
    pub fn format_topic_markdown(topic: &CapabilityDocTopic) -> String {
        let mut out = String::new();
        out.push_str(&format!("# {}\n\n", topic.title));
        out.push_str(&format!(
            "**ID:** `{}` | **Category:** `{}`\n\n",
            topic.id,
            topic.category.as_str()
        ));
        out.push_str(&format!("{}\n\n", topic.summary));

        for section in &topic.sections {
            out.push_str(&format!("## {}\n\n", section.title));
            out.push_str(&format!("{}\n\n", section.content.trim()));
        }

        if !topic.examples.is_empty() {
            out.push_str("## Examples\n\n```json\n");
            for ex in &topic.examples {
                out.push_str(&format!("{}\n", ex));
            }
            out.push_str("```\n\n");
        }

        if !topic.references.is_empty() {
            out.push_str("## Authoritative References\n\n");
            for r in &topic.references {
                out.push_str(&format!("- {}\n", r));
            }
            out.push_str("\n");
        }

        out
    }
}
