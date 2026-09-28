//! Sandbox Enforcement Documentation Subsystem (SANDBOXDOC1..SANDBOXDOC6).
//!
//! Provides an offline, self-contained documentation repository for the AIOS
//! Sandbox Enforcement subsystem, covering architecture, profiles,
//! containment primitives, security policy, observability, and CLI/MCP references.

use serde::{Deserialize, Serialize};

/// Error code: Requested topic not found.
pub const SANDBOXDOC_ERR_NOT_FOUND: &str = "SANDBOXDOC_ERR_NOT_FOUND";

/// Error code: Invalid or empty query provided.
pub const SANDBOXDOC_ERR_EMPTY_QUERY: &str = "SANDBOXDOC_ERR_EMPTY_QUERY";

/// Maximum length for queries or topic IDs.
pub const MAX_DOC_QUERY_LEN: usize = 128;

/// Categories for Sandbox documentation topics (SANDBOXDOC1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxDocCategory {
    Architecture,
    Profiles,
    Isolation,
    Policy,
    Observability,
    Reference,
}

impl SandboxDocCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SandboxDocCategory::Architecture => "architecture",
            SandboxDocCategory::Profiles => "profiles",
            SandboxDocCategory::Isolation => "isolation",
            SandboxDocCategory::Policy => "policy",
            SandboxDocCategory::Observability => "observability",
            SandboxDocCategory::Reference => "reference",
        }
    }
}

/// A structured section within a Sandbox documentation topic (SANDBOXDOC2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxDocSection {
    pub title: String,
    pub content: String,
}

/// A complete documentation topic for Sandbox Enforcement (SANDBOXDOC2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxDocTopic {
    pub id: String,
    pub title: String,
    pub category: SandboxDocCategory,
    pub summary: String,
    pub sections: Vec<SandboxDocSection>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
}

/// Summary representation for listing topics (SANDBOXDOC3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxDocTopicSummary {
    pub id: String,
    pub title: String,
    pub category: SandboxDocCategory,
    pub summary: String,
    pub tags: Vec<String>,
}

/// Scored search result returned from `SandboxDocIndex::search` (SANDBOXDOC4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
}

/// In-memory documentation repository and search index for Sandbox Enforcement.
#[derive(Debug, Clone)]
pub struct SandboxDocIndex {
    topics: Vec<SandboxDocTopic>,
}

impl Default for SandboxDocIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl SandboxDocIndex {
    /// Creates a new `SandboxDocIndex` populated with canonical documentation topics.
    pub fn new() -> Self {
        let mut index = Self {
            topics: Vec::new(),
        };
        index.populate_canonical_topics();
        index
    }

    fn populate_canonical_topics(&mut self) {
        self.topics = vec![
            SandboxDocTopic {
                id: "overview".into(),
                title: "Sandbox Enforcement Architecture & Overview".into(),
                category: SandboxDocCategory::Architecture,
                summary: "Architectural overview of AIOS sandbox isolation, containment layers, and PEP fabric gating.".into(),
                sections: vec![
                    SandboxDocSection {
                        title: "System Architecture".into(),
                        content: "AIOS Sandbox Enforcement coordinates bounded subprocess execution with strict filesystem, network, and resource constraints under the control of the Policy Enforcement Point (PEP) Fabric.".into(),
                    },
                    SandboxDocSection {
                        title: "Fail-Closed Security Guarantee".into(),
                        content: "All unauthenticated, unrecognized, or policy-violating execution requests are rejected fail-closed before any OS process or subshell is spawned.".into(),
                    },
                ],
                tags: vec!["architecture".into(), "pep".into(), "containment".into(), "fail-closed".into()],
                examples: vec!["aiosh sandbox run --profile standard -- /bin/echo 'hello'".into()],
            },
            SandboxDocTopic {
                id: "profiles".into(),
                title: "Execution Profiles & Confinement Baselines".into(),
                category: SandboxDocCategory::Profiles,
                summary: "Predefined factory profiles (standard, strict, permissive) and custom profile construction.".into(),
                sections: vec![
                    SandboxDocSection {
                        title: "Standard Profile".into(),
                        content: "Default profile for general tasks. Enforces isolated workspace filesystem, blocked outbound network access, and bounded memory (256MB) and timeout limits (30s).".into(),
                    },
                    SandboxDocSection {
                        title: "Strict Profile".into(),
                        content: "High-security isolation. Complete filesystem virtualization or read-only binds, zero network capabilities, strict seccomp syscall filtering, and tight resource caps.".into(),
                    },
                    SandboxDocSection {
                        title: "Permissive Profile".into(),
                        content: "Relaxed confinement intended for trusted build tasks. Requires PEP capability authorization when policy mandates explicit gating.".into(),
                    },
                ],
                tags: vec!["profiles".into(), "standard".into(), "strict".into(), "permissive".into(), "quotas".into()],
                examples: vec!["aiosh sandbox profiles".into(), "aiosh sandbox run --profile strict -- cargo test".into()],
            },
            SandboxDocTopic {
                id: "isolation".into(),
                title: "Operating System Isolation Primitives".into(),
                category: SandboxDocCategory::Isolation,
                summary: "Host containment primitives including Linux Landlock LSM, Seccomp-BPF, and Windows Job Objects.".into(),
                sections: vec![
                    SandboxDocSection {
                        title: "Linux Landlock LSM".into(),
                        content: "Unprivileged filesystem access control enforced via Linux kernel Landlock LSM rulesets to restrict path traversals and write accesses.".into(),
                    },
                    SandboxDocSection {
                        title: "Seccomp-BPF Syscall Filtering".into(),
                        content: "Berkley Packet Filter rules deployed to trap or terminate processes issuing hazardous syscalls (e.g. ptrace, mount, reboot).".into(),
                    },
                    SandboxDocSection {
                        title: "Windows Job Objects & Restricted Tokens".into(),
                        content: "On Windows platforms, processes are assigned to restricted job objects with active memory and CPU rate limits.".into(),
                    },
                ],
                tags: vec!["isolation".into(), "landlock".into(), "seccomp".into(), "windows".into(), "job-objects".into()],
                examples: vec!["aiosh sandbox stats".into()],
            },
            SandboxDocTopic {
                id: "policy".into(),
                title: "Sandbox Security Policy Subsystem".into(),
                category: SandboxDocCategory::Policy,
                summary: "Declarative security policies, prohibited commands, forbidden environment variables, and PEP gating.".into(),
                sections: vec![
                    SandboxDocSection {
                        title: "Prohibited Destructive Commands".into(),
                        content: "Immutable denylist of dangerous tools (rm, dd, mkfs, format, fdisk, shutdown, reboot, poweroff) matched against executable basenames.".into(),
                    },
                    SandboxDocSection {
                        title: "Environment Variable Confinement".into(),
                        content: "Prevents dynamic linker hijacking and loader injection by scrubbing variables such as LD_PRELOAD, DYLD_INSERT_LIBRARIES, and PYTHONPATH.".into(),
                    },
                    SandboxDocSection {
                        title: "Resource Ceilings".into(),
                        content: "Global policy ceilings cap maximum permissible execution wall-time (default 300,000 ms) and memory limits (default 8 GiB).".into(),
                    },
                ],
                tags: vec!["policy".into(), "security".into(), "denylist".into(), "pep-grant".into(), "environment".into()],
                examples: vec!["aiosh sandbox policy".into(), "aiosh sandbox policy --json".into()],
            },
            SandboxDocTopic {
                id: "observability".into(),
                title: "Observability, Telemetry & Health Monitoring".into(),
                category: SandboxDocCategory::Observability,
                summary: "Runtime telemetry reporting, bounded outcome distributions, and host isolation capability probing.".into(),
                sections: vec![
                    SandboxDocSection {
                        title: "Telemetry Invariants".into(),
                        content: "Observability aggregates historical executions, outcome breakdowns, profile frequencies, and active containment features.".into(),
                    },
                    SandboxDocSection {
                        title: "Hardening & Bounds".into(),
                        content: "Outcome distributions are strictly bounded to MAX_OUTCOME_DISTRIBUTION_ENTRIES (128) and text strings are sanitized to eliminate ANSI control injection.".into(),
                    },
                ],
                tags: vec!["observability".into(), "telemetry".into(), "health".into(), "metrics".into(), "stats".into()],
                examples: vec!["aiosh sandbox stats".into(), "aiosh sandbox stats --json".into()],
            },
            SandboxDocTopic {
                id: "reference".into(),
                title: "CLI & MCP Tool API Reference".into(),
                category: SandboxDocCategory::Reference,
                summary: "Comprehensive command-line interface commands and Model Context Protocol (MCP) tool signatures.".into(),
                sections: vec![
                    SandboxDocSection {
                        title: "CLI Invocations".into(),
                        content: "aiosh sandbox run [--profile <name>] [--json] -- <command...>\naiosh sandbox profiles [--json]\naiosh sandbox policy [--path <path>] [--json]\naiosh sandbox stats [--json]\naiosh sandbox doc [<topic>] [--json]".into(),
                    },
                    SandboxDocSection {
                        title: "MCP Tools".into(),
                        content: "aios.sandbox.run, aios.sandbox.profiles, aios.sandbox.policy, aios.sandbox.stats, aios.sandbox.doc".into(),
                    },
                ],
                tags: vec!["reference".into(), "cli".into(), "mcp".into(), "api".into()],
                examples: vec!["aiosh sandbox doc overview".into()],
            },
        ];
    }

    /// Lists all topics in the index as lightweight summaries.
    pub fn list_topics(&self) -> Vec<SandboxDocTopicSummary> {
        self.topics
            .iter()
            .map(|t| SandboxDocTopicSummary {
                id: t.id.clone(),
                title: t.title.clone(),
                category: t.category,
                summary: t.summary.clone(),
                tags: t.tags.clone(),
            })
            .collect()
    }

    /// Looks up a documentation topic by its identifier (case-insensitive).
    pub fn get_topic(&self, id: &str) -> Option<&SandboxDocTopic> {
        let needle = id.trim().to_lowercase();
        self.topics.iter().find(|t| t.id.to_lowercase() == needle)
    }

    /// Performs lexical search across topic titles, summaries, tags, and section content.
    pub fn search(&self, query: &str) -> Vec<SandboxDocSearchResult> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        for topic in &self.topics {
            let mut score = 0;
            if topic.id.to_lowercase() == q {
                score += 100;
            } else if topic.id.to_lowercase().contains(&q) {
                score += 50;
            }
            if topic.title.to_lowercase().contains(&q) {
                score += 40;
            }
            for tag in &topic.tags {
                if tag.to_lowercase() == q {
                    score += 30;
                } else if tag.to_lowercase().contains(&q) {
                    score += 15;
                }
            }
            if topic.summary.to_lowercase().contains(&q) {
                score += 20;
            }
            let mut matched_snippet = String::new();
            for sec in &topic.sections {
                if sec.title.to_lowercase().contains(&q) {
                    score += 10;
                }
                if let Some(idx) = sec.content.to_lowercase().find(&q) {
                    score += 5;
                    if matched_snippet.is_empty() {
                        let start = idx.saturating_sub(20);
                        let end = (idx + q.len() + 60).min(sec.content.len());
                        matched_snippet = sec.content[start..end].to_string();
                    }
                }
            }

            if score > 0 {
                results.push(SandboxDocSearchResult {
                    topic_id: topic.id.clone(),
                    title: topic.title.clone(),
                    score,
                    snippet: if matched_snippet.is_empty() {
                        topic.summary.clone()
                    } else {
                        matched_snippet
                    },
                });
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results
    }
}
