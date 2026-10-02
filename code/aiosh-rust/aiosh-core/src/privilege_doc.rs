//! Privilege Escalation Prevention Documentation Subsystem (PRIVDOC1..PRIVDOC5).
//!
//! Provides an offline, self-contained reference repository and search index for Privilege
//! Escalation Prevention, tiers, grant validation, security policy, and observability.

use serde::{Deserialize, Serialize};

/// Maximum query length for privilege documentation search.
pub const MAX_PRIVILEGE_DOC_QUERY_LEN: usize = 128;

/// Maximum number of search results returned.
pub const MAX_PRIVILEGE_DOC_SEARCH_RESULTS: usize = 10;

/// Maximum length of a snippet in search results.
pub const MAX_PRIVILEGE_DOC_SNIPPET_LEN: usize = 200;

/// Error code: Topic ID not found in documentation index.
pub const PRIVDOC_ERR_NOT_FOUND: &str = "PRIVDOC_ERR_NOT_FOUND";

/// Error code: Search query is empty or exceeds length limits.
pub const PRIVDOC_ERR_QUERY_BOUNDS: &str = "PRIVDOC_ERR_QUERY_BOUNDS";

/// Categories for privilege documentation topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeDocCategory {
    Architecture,
    Lifecycle,
    Policy,
    Observability,
    Security,
    Recovery,
}

impl PrivilegeDocCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrivilegeDocCategory::Architecture => "architecture",
            PrivilegeDocCategory::Lifecycle => "lifecycle",
            PrivilegeDocCategory::Policy => "policy",
            PrivilegeDocCategory::Observability => "observability",
            PrivilegeDocCategory::Security => "security",
            PrivilegeDocCategory::Recovery => "recovery",
        }
    }
}

/// A structured section within a privilege documentation topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeDocSection {
    pub title: String,
    pub content: String,
}

/// A complete privilege documentation topic with metadata, sections, and examples.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeDocTopic {
    pub id: String,
    pub title: String,
    pub category: PrivilegeDocCategory,
    pub summary: String,
    pub sections: Vec<PrivilegeDocSection>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
}

/// Scored search result from `PrivilegeDocIndex::search`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
}

/// Repository and lexical search index for privilege documentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeDocIndex {
    pub topics: Vec<PrivilegeDocTopic>,
}

impl Default for PrivilegeDocIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivilegeDocIndex {
    /// Creates a new index populated with canonical privilege documentation topics.
    pub fn new() -> Self {
        let mut index = Self { topics: Vec::new() };
        index.populate_canonical_topics();
        index
    }

    /// Lists all documentation topics.
    pub fn list_topics(&self) -> &[PrivilegeDocTopic] {
        &self.topics
    }

    /// Looks up a documentation topic by its unique ID.
    pub fn get_topic(&self, id: &str) -> Option<&PrivilegeDocTopic> {
        let trimmed = id.trim();
        if trimmed.is_empty() || trimmed.len() > 64 || trimmed.chars().any(|c| c.is_control()) || trimmed.contains("..") {
            return None;
        }
        self.topics.iter().find(|t| t.id == trimmed)
    }

    /// Searches documentation topics with relevance scoring.
    pub fn search(&self, query: &str) -> Result<Vec<PrivilegeDocSearchResult>, String> {
        let q = query.trim();
        if q.is_empty() {
            return Err(format!("{}: Search query must not be empty", PRIVDOC_ERR_QUERY_BOUNDS));
        }
        if q.chars().any(|c| c.is_control()) {
            return Err(format!("{}: Search query contains forbidden control characters", PRIVDOC_ERR_QUERY_BOUNDS));
        }
        if q.len() > MAX_PRIVILEGE_DOC_QUERY_LEN {
            return Err(format!(
                "{}: Search query length {} exceeds max {}",
                PRIVDOC_ERR_QUERY_BOUNDS,
                q.len(),
                MAX_PRIVILEGE_DOC_QUERY_LEN
            ));
        }

        let q_lower = q.to_ascii_lowercase();
        let tokens: Vec<&str> = q_lower.split_whitespace().collect();

        let mut results = Vec::new();

        for topic in &self.topics {
            let mut score = 0;
            let mut matched_snippet = String::new();

            for token in &tokens {
                for tag in &topic.tags {
                    if tag.to_ascii_lowercase().contains(token) {
                        score += 10;
                    }
                }

                if topic.title.to_ascii_lowercase().contains(token) {
                    score += 5;
                }

                if topic.summary.to_ascii_lowercase().contains(token) {
                    score += 3;
                    if matched_snippet.is_empty() {
                        matched_snippet = topic.summary.clone();
                    }
                }

                for sec in &topic.sections {
                    if sec.title.to_ascii_lowercase().contains(token) {
                        score += 2;
                    }
                    if sec.content.to_ascii_lowercase().contains(token) {
                        score += 1;
                        if matched_snippet.is_empty() {
                            matched_snippet = sec.content.chars().take(MAX_PRIVILEGE_DOC_SNIPPET_LEN).collect();
                        }
                    }
                }
            }

            if score > 0 {
                if matched_snippet.is_empty() {
                    matched_snippet = topic.summary.chars().take(MAX_PRIVILEGE_DOC_SNIPPET_LEN).collect();
                }
                results.push(PrivilegeDocSearchResult {
                    topic_id: topic.id.clone(),
                    title: topic.title.clone(),
                    score,
                    snippet: matched_snippet,
                });
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(MAX_PRIVILEGE_DOC_SEARCH_RESULTS);
        Ok(results)
    }

    /// Formats a documentation topic as a clean Markdown string.
    pub fn render_markdown(&self, topic_id: &str) -> Result<String, String> {
        let topic = self
            .get_topic(topic_id)
            .ok_or_else(|| format!("{}: topic '{}' not found", PRIVDOC_ERR_NOT_FOUND, topic_id))?;

        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", topic.title));
        md.push_str(&format!("**Category:** `{}`  \n", topic.category.as_str()));
        md.push_str(&format!("**Tags:** {}  \n\n", topic.tags.join(", ")));
        md.push_str(&format!("## Summary\n{}\n\n", topic.summary));

        for sec in &topic.sections {
            md.push_str(&format!("## {}\n{}\n\n", sec.title, sec.content));
        }

        if !topic.examples.is_empty() {
            md.push_str("## Examples\n");
            for ex in &topic.examples {
                md.push_str(&format!("```\n{}\n```\n\n", ex));
            }
        }

        Ok(md)
    }

    fn populate_canonical_topics(&mut self) {
        self.topics.push(PrivilegeDocTopic {
            id: "priv-arch".into(),
            title: "Privilege Escalation Prevention Architecture".into(),
            category: PrivilegeDocCategory::Architecture,
            summary: "Core privilege hierarchy, multi-tenant actor contexts, and elevation invariants in AIOS.".into(),
            sections: vec![
                PrivilegeDocSection {
                    title: "Privilege Tiers".into(),
                    content: "Four-tier hierarchy: Guest (lowest, read-only sandboxed), User (standard interactive execution), Operator (administrative actions), SystemKernel (immutable kernel tier).".into(),
                },
                PrivilegeDocSection {
                    title: "Contexts & Actors".into(),
                    content: "Every process or agent is associated with a distinct actor context holding active tier, granted capabilities, and cryptographic grant lineage.".into(),
                },
            ],
            tags: vec!["architecture".into(), "model".into(), "tiers".into(), "actor".into()],
            examples: vec!["aiosh privilege status --actor agent-1".into()],
        });

        self.topics.push(PrivilegeDocTopic {
            id: "priv-lifecycle".into(),
            title: "Privilege Elevation & Dropping Lifecycle".into(),
            category: PrivilegeDocCategory::Lifecycle,
            summary: "Mechanisms for requesting, validating, dropping, and revoking privilege escalations.".into(),
            sections: vec![
                PrivilegeDocSection {
                    title: "Elevation Request Gate".into(),
                    content: "Escalation requires a valid cryptographic PEP grant ID. Elevation without a grant or exceeding policy ceilings is rejected fail-closed.".into(),
                },
                PrivilegeDocSection {
                    title: "Privilege Drop Invariant".into(),
                    content: "Dropping privilege immediately strips elevated capabilities and revokes the active elevation grant.".into(),
                },
            ],
            tags: vec!["lifecycle".into(), "elevation".into(), "drop".into(), "revoke".into()],
            examples: vec!["aiosh privilege elevate --actor alice --to operator --grant g-101".into()],
        });

        self.topics.push(PrivilegeDocTopic {
            id: "priv-kernel-lockout".into(),
            title: "SystemKernel Tier Immutability".into(),
            category: PrivilegeDocCategory::Security,
            summary: "Architectural guarantee: SystemKernel tier cannot be reached through dynamic privilege elevation.".into(),
            sections: vec![
                PrivilegeDocSection {
                    title: "Compiler and Core Gate".into(),
                    content: "Both the data model and the security policy reject elevation targeting SystemKernel with PRIVESC_ERR_KERNEL_TIER_IMMUTABLE.".into(),
                },
            ],
            tags: vec!["security".into(), "kernel".into(), "immutability".into(), "lockout".into()],
            examples: vec!["aiosh privilege elevate --to system_kernel -> DENIED".into()],
        });

        self.topics.push(PrivilegeDocTopic {
            id: "priv-policy".into(),
            title: "Declarative Privilege Security Policy".into(),
            category: PrivilegeDocCategory::Policy,
            summary: "Tri-state policy enforcement modes, actor ceilings, and capability denylists.".into(),
            sections: vec![
                PrivilegeDocSection {
                    title: "Enforcement Modes".into(),
                    content: "Enforcing (violations blocked), Audit (violations recorded, action allowed), Permissive (for development testing only).".into(),
                },
            ],
            tags: vec!["policy".into(), "enforcement".into(), "ceilings".into(), "rules".into()],
            examples: vec!["aiosh privilege policy --json".into()],
        });

        self.topics.push(PrivilegeDocTopic {
            id: "priv-observability".into(),
            title: "Observability, Telemetry & Health Reporting".into(),
            category: PrivilegeDocCategory::Observability,
            summary: "Point-in-time metrics aggregation, outcome tracking, and sanitized health reporting.".into(),
            sections: vec![
                PrivilegeDocSection {
                    title: "Sanitization & Cardinality Bounds".into(),
                    content: "Telemetry text stripped of ANSI/control codes; outcome distribution maps clamped to 128 entries.".into(),
                },
            ],
            tags: vec!["observability".into(), "telemetry".into(), "stats".into(), "health".into()],
            examples: vec!["aiosh privilege stats --json".into()],
        });

        self.topics.push(PrivilegeDocTopic {
            id: "priv-recovery".into(),
            title: "Store Recovery & Validation Diagnostics".into(),
            category: PrivilegeDocCategory::Recovery,
            summary: "Diagnostic self-healing, quarantine of damaged store files, and atomic recovery.".into(),
            sections: vec![
                PrivilegeDocSection {
                    title: "Quarantine & Repair".into(),
                    content: "Unparseable or corrupted store files are safely backed up with timestamped quarantine before clean store synthesis.".into(),
                },
            ],
            tags: vec!["recovery".into(), "validation".into(), "repair".into(), "quarantine".into()],
            examples: vec!["aiosh privilege repair --store /path/to/store.json".into()],
        });
    }
}
