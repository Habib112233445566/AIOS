//! PEP Grant Lifecycle Documentation Subsystem (T-02281..T-02290).
//!
//! Provides an offline, self-contained reference repository and search index for PEP
//! capability grants, delegation depth attenuation, cascade revocation, security policies,
//! and MCP JSON-RPC tool surfaces.

use serde::{Deserialize, Serialize};

/// Maximum query length for grant documentation search.
pub const MAX_GRANT_DOC_QUERY_LEN: usize = 128;

/// Maximum number of search results returned.
pub const MAX_GRANT_DOC_SEARCH_RESULTS: usize = 10;

/// Maximum length of a snippet in search results.
pub const MAX_GRANT_DOC_SNIPPET_LEN: usize = 200;

/// Error code: Topic ID not found in documentation index.
pub const GRANTDOC_ERR_NOT_FOUND: &str = "GRANTDOC_ERR_NOT_FOUND";

/// Error code: Search query is empty or exceeds length limits.
pub const GRANTDOC_ERR_QUERY_BOUNDS: &str = "GRANTDOC_ERR_QUERY_BOUNDS";

/// Categories for grant documentation topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepGrantDocCategory {
    Architecture,
    Lifecycle,
    Attenuation,
    Revocation,
    Policy,
    Observability,
    Reference,
}

impl PepGrantDocCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PepGrantDocCategory::Architecture => "architecture",
            PepGrantDocCategory::Lifecycle => "lifecycle",
            PepGrantDocCategory::Attenuation => "attenuation",
            PepGrantDocCategory::Revocation => "revocation",
            PepGrantDocCategory::Policy => "policy",
            PepGrantDocCategory::Observability => "observability",
            PepGrantDocCategory::Reference => "reference",
        }
    }
}

/// A structured section within a grant documentation topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantDocSection {
    pub title: String,
    pub content: String,
}

/// A complete grant documentation topic with metadata, sections, and examples.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantDocTopic {
    pub id: String,
    pub title: String,
    pub category: PepGrantDocCategory,
    pub summary: String,
    pub sections: Vec<PepGrantDocSection>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
}

/// Scored search result from `PepGrantDocIndex::search`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
}

/// Repository and lexical search index for grant documentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantDocIndex {
    pub topics: Vec<PepGrantDocTopic>,
}

impl Default for PepGrantDocIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl PepGrantDocIndex {
    /// Creates a new index populated with canonical grant documentation topics.
    pub fn new() -> Self {
        let mut index = Self { topics: Vec::new() };
        index.populate_canonical_topics();
        index
    }

    /// Lists all documentation topics.
    pub fn list_topics(&self) -> &[PepGrantDocTopic] {
        &self.topics
    }

    /// Looks up a documentation topic by its unique ID.
    pub fn get_topic(&self, id: &str) -> Option<&PepGrantDocTopic> {
        self.topics.iter().find(|t| t.id == id)
    }

    /// Searches documentation topics with relevance scoring.
    pub fn search(&self, query: &str) -> Result<Vec<PepGrantDocSearchResult>, String> {
        let q = query.trim();
        if q.is_empty() {
            return Err(format!("{}: Search query must not be empty", GRANTDOC_ERR_QUERY_BOUNDS));
        }
        if q.len() > MAX_GRANT_DOC_QUERY_LEN {
            return Err(format!(
                "{}: Search query length {} exceeds max {}",
                GRANTDOC_ERR_QUERY_BOUNDS,
                q.len(),
                MAX_GRANT_DOC_QUERY_LEN
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
                            matched_snippet = sec.content.chars().take(MAX_GRANT_DOC_SNIPPET_LEN).collect();
                        }
                    }
                }
            }

            if score > 0 {
                if matched_snippet.is_empty() {
                    matched_snippet = topic.summary.chars().take(MAX_GRANT_DOC_SNIPPET_LEN).collect();
                }
                results.push(PepGrantDocSearchResult {
                    topic_id: topic.id.clone(),
                    title: topic.title.clone(),
                    score,
                    snippet: matched_snippet,
                });
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(MAX_GRANT_DOC_SEARCH_RESULTS);
        Ok(results)
    }

    /// Formats a documentation topic as a clean Markdown string.
    pub fn render_markdown(&self, topic_id: &str) -> Result<String, String> {
        let topic = self
            .get_topic(topic_id)
            .ok_or_else(|| format!("{}: topic '{}' not found", GRANTDOC_ERR_NOT_FOUND, topic_id))?;

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
        self.topics.push(PepGrantDocTopic {
            id: "grant-arch".into(),
            title: "Grant Architecture & Data Model".into(),
            category: PepGrantDocCategory::Architecture,
            summary: "Core structural definition of PepGrant, constraints, scopes, and identifiers in AIOS.".into(),
            sections: vec![
                PepGrantDocSection {
                    title: "Identifiers & Scopes".into(),
                    content: "Grants are uniquely identified by IDs up to 128 chars. Scopes include Filesystem, Network, Ipc, System, Tool, and Process.".into(),
                },
                PepGrantDocSection {
                    title: "Constraints".into(),
                    content: "Grants carry temporal constraints (not_before, expires_at), quota limits (max_bytes, max_invocations), and delegation depth.".into(),
                },
            ],
            tags: vec!["architecture".into(), "model".into(), "scope".into(), "id".into()],
            examples: vec!["aiosh pep-grant inspect --id g-root-1".into()],
        });

        self.topics.push(PepGrantDocTopic {
            id: "grant-lifecycle".into(),
            title: "Grant Lifecycle State Machine".into(),
            category: PepGrantDocCategory::Lifecycle,
            summary: "Five lifecycle states of PepGrant: Requested, Active, Suspended, Revoked, Expired.".into(),
            sections: vec![
                PepGrantDocSection {
                    title: "Terminal States".into(),
                    content: "Revoked and Expired are terminal states. Grants in terminal states can never return to Active.".into(),
                },
                PepGrantDocSection {
                    title: "Validation".into(),
                    content: "Authorization checks pass only when grant state is strictly Active.".into(),
                },
            ],
            tags: vec!["lifecycle".into(), "states".into(), "transitions".into(), "active".into()],
            examples: vec!["aiosh pep-grant validate --id g-1 --right read".into()],
        });

        self.topics.push(PepGrantDocTopic {
            id: "grant-attenuation".into(),
            title: "Grant Attenuation & Monotonicity".into(),
            category: PepGrantDocCategory::Attenuation,
            summary: "Deriving sub-grants with strictly attenuated rights and decremented delegation depth.".into(),
            sections: vec![
                PepGrantDocSection {
                    title: "Monotonicity Invariant".into(),
                    content: "A child grant's rights must be a strict subset of the parent grant's rights. Rights expansion is rejected.".into(),
                },
                PepGrantDocSection {
                    title: "Depth Decrement".into(),
                    content: "Child max_delegation_depth must be strictly less than parent. When depth reaches 0, delegation ceases.".into(),
                },
            ],
            tags: vec!["attenuation".into(), "delegation".into(), "monotonicity".into(), "depth".into()],
            examples: vec!["aios.pep.grant.attenuate(parent_id=\"g-1\", child_id=\"g-2\", rights=[\"read\"])".into()],
        });

        self.topics.push(PepGrantDocTopic {
            id: "grant-revocation".into(),
            title: "Grant Revocation & Cascade".into(),
            category: PepGrantDocCategory::Revocation,
            summary: "Direct revocation, recursive cascading revocation across child trees, and expiration sweeps.".into(),
            sections: vec![
                PepGrantDocSection {
                    title: "Cascade Revocation".into(),
                    content: "Revoking a parent grant with cascade=true recursively revokes all descendant child grants in the delegation forest.".into(),
                },
                PepGrantDocSection {
                    title: "Expiration Sweeps".into(),
                    content: "Periodic sweep transitions all grants past expires_at timestamp to Expired state.".into(),
                },
            ],
            tags: vec!["revocation".into(), "cascade".into(), "sweep".into(), "expired".into()],
            examples: vec!["aios.pep.grant.revoke(grant_id_param=\"g-1\", cascade=true)".into()],
        });

        self.topics.push(PepGrantDocTopic {
            id: "grant-policy".into(),
            title: "Grant Security Policy Governance".into(),
            category: PepGrantDocCategory::Policy,
            summary: "PepGrantSecurityPolicy bounds on grant lifetimes, disallowed delegation rights, and prohibited subjects.".into(),
            sections: vec![
                PepGrantDocSection {
                    title: "Enforcement Modes".into(),
                    content: "Enforcing (fail-closed), Permissive (audit-only), and Disabled (bypass).".into(),
                },
                PepGrantDocSection {
                    title: "Restricted Rights".into(),
                    content: "High-privilege rights such as Admin can be globally barred from delegation.".into(),
                },
            ],
            tags: vec!["policy".into(), "security".into(), "enforcement".into(), "admin".into()],
            examples: vec!["PepGrantSecurityPolicy::load_from_path(\"policy.json\")".into()],
        });

        self.topics.push(PepGrantDocTopic {
            id: "grant-observability".into(),
            title: "Grant Observability & Metrics".into(),
            category: PepGrantDocCategory::Observability,
            summary: "Point-in-time telemetry reports, capacity saturation monitoring, and health thresholds.".into(),
            sections: vec![
                PepGrantDocSection {
                    title: "SRE Health Rule".into(),
                    content: "Health flips to degraded when registry capacity utilization reaches or exceeds 90%.".into(),
                },
                PepGrantDocSection {
                    title: "Telemetry Sanitization".into(),
                    content: "Subject and issuer strings are stripped of control characters and capped at 256 chars.".into(),
                },
            ],
            tags: vec!["observability".into(), "telemetry".into(), "health".into(), "metrics".into()],
            examples: vec!["aios.pep.grant.report()".into()],
        });

        self.topics.push(PepGrantDocTopic {
            id: "grant-mcp".into(),
            title: "Grant MCP Tool Reference".into(),
            category: PepGrantDocCategory::Reference,
            summary: "Complete reference for all 8 aios.pep.grant.* JSON-RPC tools exposed over stdio.".into(),
            sections: vec![
                PepGrantDocSection {
                    title: "Tool Inventory".into(),
                    content: "aios.pep.grant.issue, attenuate, list, inspect, validate, revoke, sweep, report.".into(),
                },
                PepGrantDocSection {
                    title: "Standard Envelope".into(),
                    content: "All tools return standard JSON-RPC 2.0 result with { ok: true, tool: ..., ... }.".into(),
                },
            ],
            tags: vec!["mcp".into(), "tools".into(), "jsonrpc".into(), "api".into()],
            examples: vec!["tools/call with name: \"aios.pep.grant.list\"".into()],
        });
    }
}
