//! PEP Decision Engine Documentation and Reference Subsystem (PEPDOC1..PEPDOC6).
//!
//! Provides an offline, self-contained documentation repository for the AIOS
//! Policy Enforcement Point (PEP) Decision Engine, covering architecture,
//! evaluation algorithms, obligations, security policies, observability, and MCP tools.

use serde::{Deserialize, Serialize};

/// Categories for PEP documentation topics (PEPDOC1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepDocCategory {
    Architecture,
    Evaluation,
    Policy,
    Observability,
    Security,
    Reference,
}

impl PepDocCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PepDocCategory::Architecture => "architecture",
            PepDocCategory::Evaluation => "evaluation",
            PepDocCategory::Policy => "policy",
            PepDocCategory::Observability => "observability",
            PepDocCategory::Security => "security",
            PepDocCategory::Reference => "reference",
        }
    }
}

/// A structured section within a PEP documentation topic (PEPDOC2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepDocSection {
    pub title: String,
    pub content: String,
}

/// A complete PEP documentation topic with metadata, sections, examples, and references (PEPDOC2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepDocTopic {
    pub id: String,
    pub title: String,
    pub category: PepDocCategory,
    pub summary: String,
    pub sections: Vec<PepDocSection>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub examples: Vec<String>,
}

/// Scored search result returned from `PepDocIndex::search` (PEPDOC4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
    pub matched_tags: Vec<String>,
}

/// Maximum allowed length for search queries (256 characters) (PEPDOC4).
pub const MAX_DOC_QUERY_LEN: usize = 256;

/// Maximum number of search results returned (50) (PEPDOC4).
pub const MAX_DOC_SEARCH_RESULTS: usize = 50;

/// Maximum length for topic identifiers (64 characters) (PEPDOC2).
pub const MAX_TOPIC_ID_LEN: usize = 64;

/// Maximum length for extracted contextual snippets (160 characters) (PEPDOC4).
pub const MAX_SNIPPET_LEN: usize = 160;

/// Safely extracts a snippet around byte position `byte_idx` in `content`, respecting UTF-8 char boundaries (PEPDOC4).
pub fn extract_utf8_snippet(content: &str, byte_idx: usize, query_char_len: usize) -> String {
    let char_indices: Vec<(usize, char)> = content.char_indices().collect();
    if char_indices.is_empty() {
        return String::new();
    }

    let char_pos = char_indices
        .iter()
        .position(|&(idx, _)| idx >= byte_idx)
        .unwrap_or(char_indices.len() - 1);

    let half = (MAX_SNIPPET_LEN.saturating_sub(query_char_len)) / 2;
    let start_char = char_pos.saturating_sub(half);
    let end_char = (char_pos + query_char_len + half).min(char_indices.len());

    let start_byte = char_indices[start_char].0;
    let end_byte = if end_char < char_indices.len() {
        char_indices[end_char].0
    } else {
        content.len()
    };

    let slice = &content[start_byte..end_byte];
    let mut snippet = slice.replace('\n', " ").trim().to_string();

    if start_char > 0 {
        snippet = format!("...{}", snippet);
    }
    if end_char < char_indices.len() {
        snippet = format!("{}...", snippet);
    }

    snippet
}

/// Self-contained offline documentation repository for PEP Decision Engine (PEPDOC3).
#[derive(Debug, Clone)]
pub struct PepDocIndex {
    topics: Vec<PepDocTopic>,
}

impl Default for PepDocIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl PepDocIndex {
    /// Creates and initializes the documentation index with pre-populated topics (PEPDOC3).
    pub fn new() -> Self {
        let mut idx = Self { topics: Vec::new() };
        idx.seed_canonical_topics();
        idx
    }

    /// Seeds canonical documentation topics into index (PEPDOC3).
    fn seed_canonical_topics(&mut self) {
        self.topics.push(PepDocTopic {
            id: "pep-arch".to_string(),
            title: "PEP Decision Engine Architecture & Flow".to_string(),
            category: PepDocCategory::Architecture,
            summary: "Core architecture of the AIOS Policy Enforcement Point (PEP) and Policy Decision Point (PDP).".to_string(),
            sections: vec![
                PepDocSection {
                    title: "Overview".to_string(),
                    content: "The AIOS PEP Decision Engine mediates every consequential access request, resolving authorization through strict default-deny semantics, registered policy rules, combining algorithms, and post-decision obligations.".to_string(),
                },
                PepDocSection {
                    title: "Component Separation".to_string(),
                    content: "Follows RFC 2904 and XACML 3.0: Policy Enforcement Point intercepts commands, Policy Decision Point calculates Permit/Deny, Policy Administration Point manages rules, and Policy Information Point provides contextual attributes.".to_string(),
                },
            ],
            tags: vec!["architecture".to_string(), "pdp".to_string(), "pep".to_string(), "xacml".to_string()],
            references: vec!["RFC 2904".to_string(), "XACML 3.0".to_string(), "ADR-0035 §D-2".to_string()],
            examples: vec!["aiosh pep evaluate --subject agent:worker --resource fs:/etc/hosts --action read".to_string()],
        });

        self.topics.push(PepDocTopic {
            id: "pep-algorithms".to_string(),
            title: "Rule Combining Algorithms".to_string(),
            category: PepDocCategory::Evaluation,
            summary: "Resolution strategies when multiple rules match a single authorization request.".to_string(),
            sections: vec![
                PepDocSection {
                    title: "Deny Overrides".to_string(),
                    content: "DenyOverrides is the default, fail-closed combining algorithm. If any applicable rule produces Deny, the final decision is Deny regardless of Permit rules.".to_string(),
                },
                PepDocSection {
                    title: "Permit Overrides & First Applicable".to_string(),
                    content: "PermitOverrides grants access if any matching rule permits. FirstApplicable evaluates candidate rules sorted deterministically by rule ID and adopts the first match.".to_string(),
                },
            ],
            tags: vec!["algorithms".to_string(), "combining".to_string(), "deny-overrides".to_string(), "evaluation".to_string()],
            references: vec!["XACML 3.0 §C.1".to_string()],
            examples: vec!["aiosh pep evaluate --algorithm permit_overrides --subject agent:admin --resource sys:kernel --action load".to_string()],
        });

        self.topics.push(PepDocTopic {
            id: "pep-obligations".to_string(),
            title: "Post-Decision Obligations".to_string(),
            category: PepDocCategory::Policy,
            summary: "Structured side-effect operations required upon policy enforcement.".to_string(),
            sections: vec![
                PepDocSection {
                    title: "Obligation Types".to_string(),
                    content: "Supports AuditLog (levels: info, warn, error), RateLimit (key and cost), RedactFields (field list), and Custom (arbitrary JSON payload).".to_string(),
                },
                PepDocSection {
                    title: "Strict vs Best Effort Criticality".to_string(),
                    content: "Strict criticality revokes permit decisions to Deny if any obligation cannot be fulfilled. BestEffort logs a warning obligation and preserves the access decision.".to_string(),
                },
            ],
            tags: vec!["obligations".to_string(), "audit".to_string(), "rate-limit".to_string(), "redaction".to_string()],
            references: vec!["XACML 3.0 §7.18".to_string()],
            examples: vec!["aiosh pep rule-add --id r_audit --subject agent:* --resource sec:* --action * --effect permit".to_string()],
        });

        self.topics.push(PepDocTopic {
            id: "pep-secpolicy".to_string(),
            title: "Security Policy Governance & Boundaries".to_string(),
            category: PepDocCategory::Security,
            summary: "Administrative privilege boundaries, restricted resources, and temporal validity windows.".to_string(),
            sections: vec![
                PepDocSection {
                    title: "Privilege Governance".to_string(),
                    content: "Unprivileged callers cannot add Permit rules targeting restricted resource prefixes (sys:*, sec:*, kernel:*). Restricted permit rules require explicit privileged authorization.".to_string(),
                },
                PepDocSection {
                    title: "Temporal Validity".to_string(),
                    content: "Policies support valid_from_epoch_secs and valid_until_epoch_secs timestamps. Expired policies immediately transition to default-deny.".to_string(),
                },
            ],
            tags: vec!["security".to_string(), "governance".to_string(), "privilege".to_string(), "restricted".to_string()],
            references: vec!["ADR-0035 §F-2".to_string()],
            examples: vec!["aiosh pep rule-add --id r_sec --subject agent:admin --resource sys:kmod --action load --effect permit --privileged".to_string()],
        });

        self.topics.push(PepDocTopic {
            id: "pep-observability".to_string(),
            title: "PEP Observability & Health Reporting".to_string(),
            category: PepDocCategory::Observability,
            summary: "Metrics aggregation, rule effect distribution, capacity limits, and health threshold monitoring.".to_string(),
            sections: vec![
                PepDocSection {
                    title: "Capacity Limits & Health Degradation".to_string(),
                    content: "The policy service supports up to 5,000 rules (MAX_RULES_IN_SERVICE). If utilization reaches or exceeds 90% (4,500 rules), health status (is_healthy) flags false.".to_string(),
                },
                PepDocSection {
                    title: "Observability Metrics".to_string(),
                    content: "Reports include total rules, breakdown by effect (permit/deny/indeterminate/not_applicable), rules with obligations, obligation breakdown, and unique entity counts.".to_string(),
                },
            ],
            tags: vec!["observability".to_string(), "metrics".to_string(), "health".to_string(), "telemetry".to_string()],
            references: vec!["PEPOBS1..PEPOBS6".to_string()],
            examples: vec!["aiosh pep report --json".to_string()],
        });

        self.topics.push(PepDocTopic {
            id: "pep-cli-mcp".to_string(),
            title: "CLI & MCP Production Tool Reference".to_string(),
            category: PepDocCategory::Reference,
            summary: "Command line interface and Model Context Protocol tool calling reference.".to_string(),
            sections: vec![
                PepDocSection {
                    title: "CLI Commands".to_string(),
                    content: "Supported subcommands: evaluate, rule-add, rule-list, rule-remove, status, report. All commands support --json for machine-readable JSON envelopes.".to_string(),
                },
                PepDocSection {
                    title: "MCP Tools".to_string(),
                    content: "Registered MCP tools: aios.pep.evaluate, aios.pep.rule_add, aios.pep.rule_list, aios.pep.rule_remove, aios.pep.status, aios.pep.report.".to_string(),
                },
            ],
            tags: vec!["cli".to_string(), "mcp".to_string(), "reference".to_string(), "tools".to_string()],
            references: vec!["ADR-0035 §D-2".to_string()],
            examples: vec!["aiosh pep status".to_string(), "aiosh pep report".to_string()],
        });
    }

    /// Looks up a topic by identifier (PEPDOC5).
    pub fn get_topic(&self, id: &str) -> Option<&PepDocTopic> {
        self.topics.iter().find(|t| t.id.eq_ignore_ascii_case(id))
    }

    /// Returns a list of all registered topics, sorted lexicographically by ID (PEPDOC5).
    pub fn list_topics(&self) -> Vec<&PepDocTopic> {
        let mut list: Vec<&PepDocTopic> = self.topics.iter().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    /// Returns topics belonging to a specific category (PEPDOC5).
    pub fn list_by_category(&self, cat: PepDocCategory) -> Vec<&PepDocTopic> {
        let mut list: Vec<&PepDocTopic> = self.topics.iter().filter(|t| t.category == cat).collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    /// Searches documentation topics with ranked scoring and contextual snippets (PEPDOC4).
    pub fn search(&self, query: &str) -> Vec<PepDocSearchResult> {
        let cleaned = query
            .chars()
            .filter(|c| !c.is_control())
            .take(MAX_DOC_QUERY_LEN)
            .collect::<String>()
            .trim()
            .to_lowercase();

        if cleaned.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        for topic in &self.topics {
            let mut score = 0;
            let mut matched_tags = Vec::new();
            let mut best_snippet = String::new();

            // Match in title (+10)
            if topic.title.to_lowercase().contains(&cleaned) {
                score += 10;
            }

            // Match in tags (+5 per match)
            for tag in &topic.tags {
                if tag.to_lowercase().contains(&cleaned) {
                    score += 5;
                    matched_tags.push(tag.clone());
                }
            }

            // Match in summary (+3)
            let lower_summary = topic.summary.to_lowercase();
            if let Some(idx) = lower_summary.find(&cleaned) {
                score += 3;
                if best_snippet.is_empty() {
                    best_snippet = extract_utf8_snippet(&topic.summary, idx, cleaned.len());
                }
            }

            // Match in sections (+1 per section occurrence)
            for sec in &topic.sections {
                let lower_sec = sec.content.to_lowercase();
                if let Some(idx) = lower_sec.find(&cleaned) {
                    score += 1;
                    if best_snippet.is_empty() {
                        best_snippet = extract_utf8_snippet(&sec.content, idx, cleaned.len());
                    }
                }
            }

            if score > 0 {
                if best_snippet.is_empty() {
                    best_snippet = extract_utf8_snippet(&topic.summary, 0, 0);
                }
                results.push(PepDocSearchResult {
                    topic_id: topic.id.clone(),
                    title: topic.title.clone(),
                    score,
                    snippet: best_snippet,
                    matched_tags,
                });
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.topic_id.cmp(&b.topic_id)));
        results.truncate(MAX_DOC_SEARCH_RESULTS);
        results
    }

    /// Formats a topic as readable Markdown.
    pub fn format_topic_markdown(topic: &PepDocTopic) -> String {
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
            out.push_str("## Examples\n\n");
            for ex in &topic.examples {
                out.push_str(&format!("- `{}`\n", ex));
            }
            out.push('\n');
        }

        if !topic.references.is_empty() {
            out.push_str("## References\n\n");
            for r in &topic.references {
                out.push_str(&format!("- {}\n", r));
            }
            out.push('\n');
        }

        out
    }
}
