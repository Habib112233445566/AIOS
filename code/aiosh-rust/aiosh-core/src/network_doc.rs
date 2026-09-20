//! Network Bootstrap Documentation Subsystem (NDOC1..NDOC6).
//!
//! Provides an offline, self-contained documentation repository for Linux
//! networking architecture, interface discovery, security policies, and telemetry,
//! as well as dynamic Markdown and ASCII topology report generators.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::network::NetworkState;

/// Categories for network documentation topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkDocCategory {
    Architecture,
    Discovery,
    Security,
    Observability,
    Configuration,
    Troubleshooting,
}

impl NetworkDocCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            NetworkDocCategory::Architecture => "architecture",
            NetworkDocCategory::Discovery => "discovery",
            NetworkDocCategory::Security => "security",
            NetworkDocCategory::Observability => "observability",
            NetworkDocCategory::Configuration => "configuration",
            NetworkDocCategory::Troubleshooting => "troubleshooting",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.len() > 32 {
            return None;
        }
        match trimmed.to_ascii_lowercase().as_str() {
            "architecture" | "arch" => Some(NetworkDocCategory::Architecture),
            "discovery" | "probe" | "scan" | "interfaces" => Some(NetworkDocCategory::Discovery),
            "security" | "sec" | "policy" => Some(NetworkDocCategory::Security),
            "observability" | "obs" | "metrics" | "telemetry" => Some(NetworkDocCategory::Observability),
            "configuration" | "config" | "cfg" => Some(NetworkDocCategory::Configuration),
            "troubleshooting" | "debug" | "triage" => Some(NetworkDocCategory::Troubleshooting),
            _ => None,
        }
    }
}

/// A structured section within a documentation topic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkDocSection {
    pub title: String,
    pub content: String,
}

/// A complete documentation topic with metadata, sections, examples, and references.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkDocTopic {
    pub id: String,
    pub title: String,
    pub category: NetworkDocCategory,
    pub summary: String,
    pub sections: Vec<NetworkDocSection>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub examples: Vec<String>,
}

/// Scored search result returned from `NetworkDocIndex::search`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
    pub matched_tags: Vec<String>,
}

pub const MAX_DOC_QUERY_LEN: usize = 256;
pub const MAX_DOC_SEARCH_RESULTS: usize = 50;
pub const MAX_DOC_FILE_BYTES: u64 = 1_048_576;

pub const NDOC_IO_ERROR: &str = "NDOC_IO_ERROR";
pub const NDOC_PATH_ERROR: &str = "NDOC_PATH_ERROR";
pub const NDOC_VALIDATION_ERROR: &str = "NDOC_VALIDATION_ERROR";

/// Validates documentation file path hygiene (no traversal, no control chars, max length 1024).
pub fn validate_doc_path(path: &Path) -> Result<(), String> {
    let path_str = path.to_str().ok_or_else(|| format!("{}: path must be valid UTF-8", NDOC_PATH_ERROR))?;
    if path_str.trim().is_empty() {
        return Err(format!("{}: path cannot be empty", NDOC_PATH_ERROR));
    }
    if path_str.len() > 1024 {
        return Err(format!("{}: path exceeds maximum length of 1024 characters", NDOC_PATH_ERROR));
    }
    if path_str.chars().any(|c| c.is_control() || c == '\0') {
        return Err(format!("{}: path cannot contain control characters", NDOC_PATH_ERROR));
    }
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(format!("{}: path traversal ('..') is not permitted", NDOC_PATH_ERROR));
    }
    Ok(())
}

/// Sanitizes text for safe inclusion inside Markdown table cells, escaping pipes and stripping control characters.
pub fn sanitize_table_cell(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() && *c != '\0')
        .collect::<String>()
        .replace('|', "\\|")
        .trim()
        .to_string()
}

/// Offline in-memory repository and index for Network Bootstrap documentation.
#[derive(Debug, Clone)]
pub struct NetworkDocIndex {
    pub topics: Vec<NetworkDocTopic>,
}

impl Default for NetworkDocIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkDocIndex {
    /// Creates a new index pre-populated with canonical topics (NDOC1).
    pub fn new() -> Self {
        let mut idx = NetworkDocIndex {
            topics: Vec::new(),
        };
        idx.populate_canonical_topics();
        idx
    }

    /// Populates repository with canonical reference topics.
    fn populate_canonical_topics(&mut self) {
        self.topics.push(NetworkDocTopic {
            id: "net-arch-overview".into(),
            title: "Network Bootstrap Architecture & Subsystems".into(),
            category: NetworkDocCategory::Architecture,
            summary: "Overview of AIOS Linux userspace network bootstrap architecture, kernel interfaces, and safety invariants.".into(),
            sections: vec![
                NetworkDocSection {
                    title: "Architecture Principles".into(),
                    content: "AIOS network bootstrap operates strictly unprivileged via sysfs, procfs, and resolvconf, enforcing invariants NET1..NET6.".into(),
                },
                NetworkDocSection {
                    title: "Subsystem Organization".into(),
                    content: "Organized into core data model, discovery service, CLI surface, MCP surface, configuration, automated tests, security policy, observability, and documentation.".into(),
                },
            ],
            tags: vec!["architecture".into(), "network".into(), "bootstrap".into(), "invariants".into()],
            references: vec!["RFC 1123".into(), "RFC 2863".into()],
            examples: vec!["aiosh network state --json".into()],
        });

        self.topics.push(NetworkDocTopic {
            id: "net-discovery-sysfs".into(),
            title: "Interface Discovery & sysfs Hierarchy".into(),
            category: NetworkDocCategory::Discovery,
            summary: "Details of Linux sysfs /sys/class/net parsing, MAC addresses, operstate, flags, and MTU extraction.".into(),
            sections: vec![
                NetworkDocSection {
                    title: "sysfs Attributes".into(),
                    content: "Reads operstate, mtu, address, flags, and type from /sys/class/net/<iface>/ with bounded buffer allocations.".into(),
                },
            ],
            tags: vec!["sysfs".into(), "discovery".into(), "interfaces".into(), "mac".into(), "mtu".into()],
            references: vec!["Linux Documentation/ABI/testing/sysfs-class-net".into()],
            examples: vec!["cat /sys/class/net/eth0/operstate".into()],
        });

        self.topics.push(NetworkDocTopic {
            id: "net-security-policy".into(),
            title: "Network Security Policy Engine & Invariants".into(),
            category: NetworkDocCategory::Security,
            summary: "Validation rules, promiscuous mode detection, allowlists, address masking, and enforcement modes.".into(),
            sections: vec![
                NetworkDocSection {
                    title: "Policy Invariants NPOL1..NPOL6".into(),
                    content: "Enforces interface gatekeeping, orphan route rejection, DNS whitelisting, capacity quotas, address redaction, and atomic persistence.".into(),
                },
            ],
            tags: vec!["security".into(), "policy".into(), "promiscuous".into(), "redaction".into(), "quotas".into()],
            references: vec!["AIOS Security Architecture".into()],
            examples: vec!["aios.network.policy.evaluate".into()],
        });

        self.topics.push(NetworkDocTopic {
            id: "net-observability-telemetry".into(),
            title: "Network Observability & Health Diagnostics".into(),
            category: NetworkDocCategory::Observability,
            summary: "Kernel counter parsing from /proc/net/dev, link carrier tracking, and composite health assessment.".into(),
            sections: vec![
                NetworkDocSection {
                    title: "Health Diagnostic Verdicts".into(),
                    content: "Evaluates host connectivity into Healthy, Degraded, or Critical states with bounded time-series ring buffering.".into(),
                },
            ],
            tags: vec!["observability".into(), "telemetry".into(), "metrics".into(), "health".into(), "carrier".into()],
            references: vec!["man 5 proc".into()],
            examples: vec!["aios.network.metrics".into()],
        });

        self.topics.push(NetworkDocTopic {
            id: "net-configuration-env".into(),
            title: "Network Configuration & Environment Overrides".into(),
            category: NetworkDocCategory::Configuration,
            summary: "Environment variables AIOS_NETWORK_*, capacity limits, timeout controls, and JSON persistence contracts.".into(),
            sections: vec![
                NetworkDocSection {
                    title: "Environment Variables".into(),
                    content: "Configures default store paths, sysfs/procfs overrides, timeouts, and fallback DNS servers.".into(),
                },
            ],
            tags: vec!["configuration".into(), "environment".into(), "timeouts".into(), "bounds".into()],
            references: vec!["docs/network_bootstrap.md".into()],
            examples: vec!["export AIOS_NETWORK_SCAN_TIMEOUT=15".into()],
        });

        self.topics.push(NetworkDocTopic {
            id: "net-troubleshooting-triage".into(),
            title: "Network Bootstrap Triage & Troubleshooting".into(),
            category: NetworkDocCategory::Troubleshooting,
            summary: "Resolving common network issues: missing carrier, route conflicts, DNS failures, and interface mismatches.".into(),
            sections: vec![
                NetworkDocSection {
                    title: "Diagnostic Workflow".into(),
                    content: "Check carrier flag, verify default route points to existing interface, validate DNS resolver IPs, and check drop rates.".into(),
                },
            ],
            tags: vec!["troubleshooting".into(), "triage".into(), "carrier".into(), "dns".into(), "routing".into()],
            references: vec!["AIOS Runbook".into()],
            examples: vec!["aiosh network metrics".into()],
        });
    }

    /// Retrieves topic by exact identifier.
    pub fn get_topic(&self, id: &str) -> Option<&NetworkDocTopic> {
        let trimmed = id.trim();
        self.topics.iter().find(|t| t.id == trimmed)
    }

    /// Lists topics, optionally filtered by category.
    pub fn list_topics(&self, category: Option<NetworkDocCategory>) -> Vec<&NetworkDocTopic> {
        match category {
            Some(cat) => self.topics.iter().filter(|t| t.category == cat).collect(),
            None => self.topics.iter().collect(),
        }
    }

    /// Searches topics with ranked scoring (NDOC3).
    /// Searches topics with ranked scoring (NDOC3).
    pub fn search(&self, query: &str) -> Vec<NetworkDocSearchResult> {
        let trimmed_query = query.trim();
        if trimmed_query.is_empty() || trimmed_query.len() > MAX_DOC_QUERY_LEN {
            return Vec::new();
        }

        let query_lower = trimmed_query.to_ascii_lowercase();
        let mut query_terms: Vec<&str> = query_lower.split_whitespace().collect();
        if query_terms.len() > 16 {
            query_terms.truncate(16);
        }
        let mut results = Vec::new();

        for topic in &self.topics {
            let mut score = 0;
            let mut matched_tags = Vec::new();

            let id_lower = topic.id.to_ascii_lowercase();
            let title_lower = topic.title.to_ascii_lowercase();
            let summary_lower = topic.summary.to_ascii_lowercase();

            for term in &query_terms {
                if id_lower.contains(term) {
                    score += 100;
                }
                if title_lower.contains(term) {
                    score += 50;
                }
                if summary_lower.contains(term) {
                    score += 20;
                }
                for tag in &topic.tags {
                    if tag.to_ascii_lowercase().contains(term) {
                        score += 25;
                        if !matched_tags.contains(tag) {
                            matched_tags.push(tag.clone());
                        }
                    }
                }
                for section in &topic.sections {
                    if section.title.to_ascii_lowercase().contains(term) {
                        score += 15;
                    }
                    if section.content.to_ascii_lowercase().contains(term) {
                        score += 5;
                    }
                }
            }

            if score > 0 {
                let snippet = if topic.summary.chars().count() > 120 {
                    let mut s: String = topic.summary.chars().take(117).collect();
                    s.push_str("...");
                    s
                } else {
                    topic.summary.clone()
                };

                results.push(NetworkDocSearchResult {
                    topic_id: topic.id.clone(),
                    title: topic.title.clone(),
                    score,
                    snippet,
                    matched_tags,
                });
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.topic_id.cmp(&b.topic_id)));
        if results.len() > MAX_DOC_SEARCH_RESULTS {
            results.truncate(MAX_DOC_SEARCH_RESULTS);
        }
        results
    }

    /// Renders a single topic as Markdown (NDOC4).
    pub fn render_topic_markdown(&self, id: &str) -> Option<String> {
        let topic = self.get_topic(id)?;
        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", topic.title));
        md.push_str(&format!("**Category:** `{}` | **ID:** `{}`\n\n", topic.category.as_str(), topic.id));
        md.push_str(&format!("{}\n\n", topic.summary));

        for section in &topic.sections {
            md.push_str(&format!("## {}\n\n", section.title));
            md.push_str(&format!("{}\n\n", section.content));
        }

        if !topic.examples.is_empty() {
            md.push_str("### Examples\n\n");
            for ex in &topic.examples {
                md.push_str(&format!("```bash\n{}\n```\n\n", ex));
            }
        }

        if !topic.references.is_empty() {
            md.push_str("### References\n\n");
            for rf in &topic.references {
                md.push_str(&format!("- {}\n", rf));
            }
            md.push('\n');
        }

        Some(md)
    }

    /// Generates dynamic Markdown documentation from live NetworkState (NDOC5).
    pub fn render_state_markdown(&self, state: &NetworkState) -> String {
        let mut md = String::new();
        md.push_str(&format!("# AIOS Host Network State Report: {}\n\n", state.hostname));
        md.push_str(&format!("**Timestamp:** `{}`\n\n", state.timestamp));

        // Interfaces table
        md.push_str("## Network Interfaces\n\n");
        md.push_str("| Interface | Type | OperState | MAC Address | MTU | IP Addresses |\n");
        md.push_str("|:---|:---|:---|:---|:---|:---|\n");
        for iface in &state.interfaces {
            let mac = iface.mac_address.as_deref().unwrap_or("none");
            let ips = if iface.ip_addresses.is_empty() {
                "none".into()
            } else {
                iface.ip_addresses
                    .iter()
                    .map(|ip| format!("{}/{}", ip.address, ip.prefix_len))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | {} | {} |\n",
                sanitize_table_cell(&iface.name),
                sanitize_table_cell(&format!("{:?}", iface.iftype)),
                sanitize_table_cell(&format!("{:?}", iface.operstate)),
                sanitize_table_cell(mac),
                iface.mtu,
                sanitize_table_cell(&ips)
            ));
        }
        md.push('\n');

        // Routes table
        md.push_str("## Routing Table\n\n");
        md.push_str("| Destination | Gateway | Interface | Metric |\n");
        md.push_str("|:---|:---|:---|:---|\n");
        for route in &state.routes {
            let gw = route.gateway.as_deref().unwrap_or("direct");
            let iface = route.interface.as_deref().unwrap_or("none");
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} |\n",
                sanitize_table_cell(&route.destination),
                sanitize_table_cell(gw),
                sanitize_table_cell(iface),
                route.metric
            ));
        }
        md.push('\n');

        // DNS section
        md.push_str("## DNS Resolver Configuration\n\n");
        if state.dns.nameservers.is_empty() {
            md.push_str("*No nameservers configured.*\n\n");
        } else {
            md.push_str("**Nameservers:**\n");
            for ns in &state.dns.nameservers {
                md.push_str(&format!("- `{}`\n", ns));
            }
            md.push('\n');
        }

        // ASCII Topology
        md.push_str("## Network Topology Diagram\n\n```\n");
        md.push_str(&self.render_ascii_topology(state));
        md.push_str("```\n");

        md
    }

    /// Renders ASCII representation of host network topology (NDOC5).
    pub fn render_ascii_topology(&self, state: &NetworkState) -> String {
        let mut ascii = String::new();
        ascii.push_str(&format!("[Host: {}]\n", state.hostname));
        ascii.push_str("  |\n");

        for (i, iface) in state.interfaces.iter().enumerate() {
            let is_last = i == state.interfaces.len() - 1;
            let branch = if is_last { "  \\--" } else { "  +--" };
            let state_str = format!("{:?}", iface.operstate);
            let mac_str = iface.mac_address.as_deref().unwrap_or("--");
            let ip_str = iface.ip_addresses.first().map(|ip| format!("{}/{}", ip.address, ip.prefix_len)).unwrap_or_else(|| "no-ip".into());

            ascii.push_str(&format!(
                "{} [{}] ({:?}) oper:{} mac:{} ip:{}\n",
                branch, iface.name, iface.iftype, state_str, mac_str, ip_str
            ));
        }

        ascii.push_str("  |\n  +-- [Routes]\n");
        for route in &state.routes {
            let target = route.interface.as_deref().unwrap_or("any");
            let gw = route.gateway.as_deref().unwrap_or("link");
            ascii.push_str(&format!("  |     dst: {} -> gw: {} dev: {}\n", route.destination, gw, target));
        }

        ascii.push_str("  |\n  \\-- [DNS Resolvers]\n");
        for ns in &state.dns.nameservers {
            ascii.push_str(&format!("        nameserver: {}\n", ns));
        }

        ascii
    }

    /// Saves rendered documentation atomically to disk (NDOC6).
    pub fn save_to_path(&self, content: &str, path: &Path) -> Result<(), String> {
        validate_doc_path(path)?;
        if content.len() as u64 > MAX_DOC_FILE_BYTES {
            return Err(format!(
                "{}: document size {} exceeds limit of {} bytes",
                NDOC_VALIDATION_ERROR,
                content.len(),
                MAX_DOC_FILE_BYTES
            ));
        }

        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("{}: failed to create directory {}: {}", NDOC_IO_ERROR, parent.display(), e))?;
        }

        let tmp_file_name = format!(
            ".{}.tmp.{}",
            path.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "doc".into()),
            std::process::id()
        );
        let tmp_path = if parent.as_os_str().is_empty() {
            PathBuf::from(tmp_file_name)
        } else {
            parent.join(tmp_file_name)
        };

        struct TempFileGuard {
            path: PathBuf,
            active: bool,
        }
        impl Drop for TempFileGuard {
            fn drop(&mut self) {
                if self.active && self.path.exists() {
                    let _ = fs::remove_file(&self.path);
                }
            }
        }

        let mut guard = TempFileGuard {
            path: tmp_path.clone(),
            active: true,
        };

        if let Err(e) = fs::write(&tmp_path, content) {
            return Err(format!("{}: failed to write temp file {}: {}", NDOC_IO_ERROR, tmp_path.display(), e));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
        }

        if let Err(e) = fs::rename(&tmp_path, path) {
            return Err(format!("{}: failed to atomically rename {} to {}: {}", NDOC_IO_ERROR, tmp_path.display(), path.display(), e));
        }

        guard.active = false;
        Ok(())
    }

    /// Reads persisted documentation from disk (NDOC6).
    pub fn load_from_path(path: &Path) -> Result<String, String> {
        validate_doc_path(path)?;
        if !path.exists() {
            return Err(format!("{}: file {} not found", NDOC_IO_ERROR, path.display()));
        }
        let metadata = fs::metadata(path)
            .map_err(|e| format!("{}: failed to read metadata for {}: {}", NDOC_IO_ERROR, path.display(), e))?;
        if metadata.len() > MAX_DOC_FILE_BYTES {
            return Err(format!(
                "{}: document file {} size {} exceeds maximum allowed ({} bytes)",
                NDOC_VALIDATION_ERROR,
                path.display(),
                metadata.len(),
                MAX_DOC_FILE_BYTES
            ));
        }
        fs::read_to_string(path)
            .map_err(|e| format!("{}: failed to read document from {}: {}", NDOC_IO_ERROR, path.display(), e))
    }
}

