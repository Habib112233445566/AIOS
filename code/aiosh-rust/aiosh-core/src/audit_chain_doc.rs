//! Audit Chain Extensions Documentation Subsystem (T-02381..T-02390).
//!
//! Provides an offline, self-contained reference repository and lexical search index
//! for Audit Chain hash rings, causal lineage DAGs, digital signatures,
//! security policies, observability telemetry, and MCP tool references.

use serde::{Deserialize, Serialize};

/// Maximum query length for audit documentation search.
pub const MAX_AUDIT_DOC_QUERY_LEN: usize = 128;

/// Maximum number of search results returned.
pub const MAX_AUDIT_DOC_SEARCH_RESULTS: usize = 10;

/// Maximum length of a snippet in search results.
pub const MAX_AUDIT_DOC_SNIPPET_LEN: usize = 200;

/// Error code: Topic ID not found in documentation index.
pub const AUDITDOC_ERR_NOT_FOUND: &str = "AUDITDOC_ERR_NOT_FOUND";

/// Error code: Search query is empty or exceeds length limits.
pub const AUDITDOC_ERR_QUERY_BOUNDS: &str = "AUDITDOC_ERR_QUERY_BOUNDS";

/// Maximum number of tokens parsed in a single search query.
pub const MAX_DOC_SEARCH_TOKENS: usize = 16;

/// Sanitizes query string by stripping control characters and trimming.
pub fn sanitize_doc_query(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(MAX_AUDIT_DOC_QUERY_LEN)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Categories for audit documentation topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditChainDocCategory {
    Architecture,
    Lineage,
    Signatures,
    Policy,
    Observability,
    Recovery,
    Reference,
}

impl AuditChainDocCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditChainDocCategory::Architecture => "architecture",
            AuditChainDocCategory::Lineage => "lineage",
            AuditChainDocCategory::Signatures => "signatures",
            AuditChainDocCategory::Policy => "policy",
            AuditChainDocCategory::Observability => "observability",
            AuditChainDocCategory::Recovery => "recovery",
            AuditChainDocCategory::Reference => "reference",
        }
    }
}

/// A structured section within an audit documentation topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainDocSection {
    pub title: String,
    pub content: String,
}

/// A complete audit documentation topic with metadata, sections, and examples.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainDocTopic {
    pub id: String,
    pub title: String,
    pub category: AuditChainDocCategory,
    pub summary: String,
    pub sections: Vec<AuditChainDocSection>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
}

/// Scored search result from `AuditChainDocIndex::search`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
}

/// Repository and lexical search index for audit documentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainDocIndex {
    pub topics: Vec<AuditChainDocTopic>,
}

impl Default for AuditChainDocIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditChainDocIndex {
    /// Creates a new index populated with canonical audit chain documentation topics.
    pub fn new() -> Self {
        let mut index = Self { topics: Vec::new() };
        index.populate_canonical_topics();
        index
    }

    /// Lists all documentation topics.
    pub fn list_topics(&self) -> &[AuditChainDocTopic] {
        &self.topics
    }

    /// Retrieves a specific topic by its unique identifier.
    pub fn get_topic(&self, id: &str) -> Option<&AuditChainDocTopic> {
        self.topics.iter().find(|t| t.id == id)
    }

    /// Searches documentation topics for query keywords.
    pub fn search(&self, query: &str) -> Result<Vec<AuditChainDocSearchResult>, String> {
        let cleaned = sanitize_doc_query(query);
        if cleaned.is_empty() || query.trim().chars().count() > MAX_AUDIT_DOC_QUERY_LEN {
            return Err(format!(
                "{}: query length must be between 1 and {} chars",
                AUDITDOC_ERR_QUERY_BOUNDS, MAX_AUDIT_DOC_QUERY_LEN
            ));
        }

        let query_lower = cleaned.to_lowercase();
        let tokens: Vec<&str> = query_lower.split_whitespace().take(MAX_DOC_SEARCH_TOKENS).collect();
        let mut results = Vec::new();

        for topic in &self.topics {
            let mut score = 0;
            let mut best_snippet = String::new();

            // Match title
            let title_lower = topic.title.to_lowercase();
            for token in &tokens {
                if title_lower.contains(token) {
                    score += 10;
                }
            }

            // Match tags
            for tag in &topic.tags {
                let tag_lower = tag.to_lowercase();
                for token in &tokens {
                    if tag_lower.contains(token) {
                        score += 8;
                    }
                }
            }

            // Match summary
            let summary_lower = topic.summary.to_lowercase();
            for token in &tokens {
                if summary_lower.contains(token) {
                    score += 5;
                    if best_snippet.is_empty() {
                        best_snippet = topic.summary.clone();
                    }
                }
            }

            // Match sections
            for section in &topic.sections {
                let content_lower = section.content.to_lowercase();
                for token in &tokens {
                    if content_lower.contains(token) {
                        score += 3;
                        if best_snippet.is_empty() {
                            best_snippet = format!("{}: {}", section.title, section.content);
                        }
                    }
                }
            }

            if score > 0 {
                let snippet = if best_snippet.is_empty() {
                    topic.summary.clone()
                } else if best_snippet.chars().count() > MAX_AUDIT_DOC_SNIPPET_LEN {
                    format!("{}...", best_snippet.chars().take(MAX_AUDIT_DOC_SNIPPET_LEN).collect::<String>())
                } else {
                    best_snippet
                };

                results.push(AuditChainDocSearchResult {
                    topic_id: topic.id.clone(),
                    title: topic.title.clone(),
                    score,
                    snippet,
                });
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(MAX_AUDIT_DOC_SEARCH_RESULTS);
        Ok(results)
    }

    /// Formats a documentation topic as a clean Markdown string.
    pub fn render_markdown(&self, topic_id: &str) -> Result<String, String> {
        let trimmed_id = topic_id.trim();
        if trimmed_id.is_empty() || trimmed_id.len() > 64 || !trimmed_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(format!("{}: invalid topic ID format", AUDITDOC_ERR_NOT_FOUND));
        }
        let topic = self
            .get_topic(trimmed_id)
            .ok_or_else(|| format!("{}: topic '{}' not found", AUDITDOC_ERR_NOT_FOUND, trimmed_id))?;

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
                md.push_str(&format!("- `{}`\n", ex));
            }
            md.push('\n');
        }

        Ok(md)
    }

    /// Populates standard canonical audit documentation topics.
    fn populate_canonical_topics(&mut self) {
        self.topics.push(AuditChainDocTopic {
            id: "audit-arch".into(),
            title: "Audit Chain Architecture & Storage".into(),
            category: AuditChainDocCategory::Architecture,
            summary: "Cryptographic hash chaining and SQLite persistence for immutable event logging.".into(),
            sections: vec![
                AuditChainDocSection {
                    title: "Hash Chaining".into(),
                    content: "Every audit row records a previous_hash linking sequentially to the prior row with SHA-256 integrity.".into(),
                },
                AuditChainDocSection {
                    title: "Ring Storage".into(),
                    content: "Audit rings are stored in SQLite databases with automatic index optimization and WAL journaling.".into(),
                },
            ],
            tags: vec!["architecture".into(), "hash".into(), "sha256".into(), "sqlite".into()],
            examples: vec!["aiosh audit stats".into()],
        });

        self.topics.push(AuditChainDocTopic {
            id: "audit-lineage".into(),
            title: "Causal DAG Lineage Tracking".into(),
            category: AuditChainDocCategory::Lineage,
            summary: "Directed Acyclic Graph ancestry resolution and diamond deduplication for event lineage.".into(),
            sections: vec![
                AuditChainDocSection {
                    title: "Causal Links".into(),
                    content: "Events record causal links to parent events, enabling topological ancestor traversal and blast-radius analysis.".into(),
                },
                AuditChainDocSection {
                    title: "Cycle Immunity".into(),
                    content: "DAG traversal engines detect loops and enforce depth clamping to prevent recursion overflow.".into(),
                },
            ],
            tags: vec!["lineage".into(), "dag".into(), "causal".into(), "ancestry".into()],
            examples: vec!["aiosh audit tree <id>".into()],
        });

        self.topics.push(AuditChainDocTopic {
            id: "audit-crypto".into(),
            title: "Ed25519 Cryptographic Signatures".into(),
            category: AuditChainDocCategory::Signatures,
            summary: "Digital signatures on audit events ensuring non-repudiation and tamper evidence.".into(),
            sections: vec![
                AuditChainDocSection {
                    title: "Key Management".into(),
                    content: "Signatures utilize Ed25519 public key pairs associated with specific actors or agents.".into(),
                },
                AuditChainDocSection {
                    title: "Verification".into(),
                    content: "Payload canonicalization ensures consistent signature verification across Rust and Python runtimes.".into(),
                },
            ],
            tags: vec!["crypto".into(), "ed25519".into(), "signature".into(), "verification".into()],
            examples: vec!["aiosh audit verify".into()],
        });

        self.topics.push(AuditChainDocTopic {
            id: "audit-policy".into(),
            title: "Audit Chain Security Policy".into(),
            category: AuditChainDocCategory::Policy,
            summary: "Enforcement modes and gating rules for audit event ingestion and causal links.".into(),
            sections: vec![
                AuditChainDocSection {
                    title: "Modes".into(),
                    content: "Supports enforcing (fail-closed), permissive (log warnings), and disabled modes.".into(),
                },
                AuditChainDocSection {
                    title: "Rules".into(),
                    content: "Prohibits anonymous actors, mandates signatures on kernel/sec tools, and bounds causal fanout.".into(),
                },
            ],
            tags: vec!["policy".into(), "security".into(), "enforcement".into(), "gating".into()],
            examples: vec!["aiosh audit policy".into()],
        });

        self.topics.push(AuditChainDocTopic {
            id: "audit-observability".into(),
            title: "Observability & Telemetry Snapshots".into(),
            category: AuditChainDocCategory::Observability,
            summary: "Real-time metrics, outcome distributions, and database health reports.".into(),
            sections: vec![
                AuditChainDocSection {
                    title: "Metrics Collected".into(),
                    content: "Tracks total rows, extended rows, distinct actors, distinct tools, outcome histogram, and db size.".into(),
                },
                AuditChainDocSection {
                    title: "Sanitization".into(),
                    content: "Output strings are sanitized against control characters and capped in memory.".into(),
                },
            ],
            tags: vec!["observability".into(), "telemetry".into(), "metrics".into(), "stats".into()],
            examples: vec!["aiosh audit stats --json".into()],
        });

        self.topics.push(AuditChainDocTopic {
            id: "audit-recovery".into(),
            title: "Audit Chain Recovery & Validation".into(),
            category: AuditChainDocCategory::Recovery,
            summary: "Integrity checking, corrupted hash repair, and verification mechanisms.".into(),
            sections: vec![
                AuditChainDocSection {
                    title: "Integrity Verification".into(),
                    content: "Validates sequential hashes against calculated payloads to detect tampering or torn writes.".into(),
                },
                AuditChainDocSection {
                    title: "Forward Repair".into(),
                    content: "Re-anchors corrupted chain tips with repair provenance records.".into(),
                },
            ],
            tags: vec!["recovery".into(), "repair".into(), "validation".into(), "integrity".into()],
            examples: vec!["aiosh audit verify".into()],
        });

        self.topics.push(AuditChainDocTopic {
            id: "audit-reference".into(),
            title: "CLI & MCP Tool Reference".into(),
            category: AuditChainDocCategory::Reference,
            summary: "Complete command-line interface and Model Context Protocol tool specifications.".into(),
            sections: vec![
                AuditChainDocSection {
                    title: "CLI Subcommands".into(),
                    content: "aiosh audit log, aiosh audit verify, aiosh audit stats, aiosh audit policy, aiosh audit doc.".into(),
                },
                AuditChainDocSection {
                    title: "MCP Tools".into(),
                    content: "aios.audit.log, aios.audit.verify, aios.audit.stats, aios.audit.policy, aios.audit.doc.".into(),
                },
            ],
            tags: vec!["reference".into(), "cli".into(), "mcp".into(), "tools".into()],
            examples: vec!["aios.audit.stats".into(), "aios.audit.doc".into()],
        });
    }

    /// Filters documentation topics by category.
    pub fn topics_by_category(&self, category: AuditChainDocCategory) -> Vec<&AuditChainDocTopic> {
        self.topics.iter().filter(|t| t.category == category).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_list_topics() {
        let index = AuditChainDocIndex::new();
        let topics = index.list_topics();
        assert_eq!(topics.len(), 7);
    }

    #[test]
    fn test_doc_get_topic() {
        let index = AuditChainDocIndex::new();
        assert!(index.get_topic("audit-arch").is_some());
        assert!(index.get_topic("audit-lineage").is_some());
        assert!(index.get_topic("nonexistent-topic").is_none());
    }

    #[test]
    fn test_doc_search_keywords() {
        let index = AuditChainDocIndex::new();
        let results = index.search("ed25519 signature").expect("search should succeed");
        assert!(!results.is_empty());
        assert_eq!(results[0].topic_id, "audit-crypto");
    }

    #[test]
    fn test_doc_search_bounds() {
        let index = AuditChainDocIndex::new();
        assert!(index.search("").is_err());
        assert!(index.search("   ").is_err());
        let oversized = "a".repeat(MAX_AUDIT_DOC_QUERY_LEN + 1);
        assert!(index.search(&oversized).is_err());
    }

    #[test]
    fn test_doc_category_filtering() {
        let index = AuditChainDocIndex::new();
        let arch_topics = index.topics_by_category(AuditChainDocCategory::Architecture);
        assert_eq!(arch_topics.len(), 1);
        assert_eq!(arch_topics[0].id, "audit-arch");
    }
}

