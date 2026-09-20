//! Hardware Detection Documentation and Reference Index (HDOC1..HDOC6).
//!
//! Provides an offline, self-contained documentation repository for Linux
//! hardware discovery, sysfs topologies, security policies, and observability.

use serde::{Deserialize, Serialize};

/// Categories for hardware documentation topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareDocCategory {
    Architecture,
    Discovery,
    Security,
    Observability,
    Configuration,
    Troubleshooting,
}

impl HardwareDocCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            HardwareDocCategory::Architecture => "architecture",
            HardwareDocCategory::Discovery => "discovery",
            HardwareDocCategory::Security => "security",
            HardwareDocCategory::Observability => "observability",
            HardwareDocCategory::Configuration => "configuration",
            HardwareDocCategory::Troubleshooting => "troubleshooting",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.len() > 32 {
            return None;
        }
        match trimmed.to_ascii_lowercase().as_str() {
            "architecture" | "arch" => Some(HardwareDocCategory::Architecture),
            "discovery" | "probe" | "scan" => Some(HardwareDocCategory::Discovery),
            "security" | "sec" | "policy" => Some(HardwareDocCategory::Security),
            "observability" | "obs" | "telemetry" => Some(HardwareDocCategory::Observability),
            "configuration" | "config" | "cfg" => Some(HardwareDocCategory::Configuration),
            "troubleshooting" | "debug" | "triage" => Some(HardwareDocCategory::Troubleshooting),
            _ => None,
        }
    }
}

/// A structured section within a documentation topic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareDocSection {
    pub title: String,
    pub content: String,
}

/// A complete documentation topic with metadata, sections, examples, and references.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareDocTopic {
    pub id: String,
    pub title: String,
    pub category: HardwareDocCategory,
    pub summary: String,
    pub sections: Vec<HardwareDocSection>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub examples: Vec<String>,
}

/// Scored search result returned from `HardwareDocIndex::search`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
    pub matched_tags: Vec<String>,
}

pub const MAX_DOC_QUERY_LEN: usize = 256;
pub const MAX_DOC_SEARCH_RESULTS: usize = 50;
pub const MAX_TOPIC_ID_LEN: usize = 64;

/// Offline in-memory index of hardware detection documentation.
#[derive(Debug, Clone)]
pub struct HardwareDocIndex {
    pub topics: Vec<HardwareDocTopic>,
}

impl Default for HardwareDocIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareDocIndex {
    /// Creates a new documentation index pre-populated with canonical topics (HDOC1).
    pub fn new() -> Self {
        let mut idx = HardwareDocIndex {
            topics: Vec::new(),
        };
        idx.register_canonical_topics();
        idx
    }

    /// Looks up a topic by ID (case-insensitive) with defensive bounds (HDOC2).
    pub fn get_topic(&self, id: &str) -> Option<&HardwareDocTopic> {
        let id_clean = id.trim();
        if id_clean.is_empty()
            || id_clean.len() > MAX_TOPIC_ID_LEN
            || !id_clean.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return None;
        }
        self.topics.iter().find(|t| t.id.eq_ignore_ascii_case(id_clean))
    }

    /// Lists topics, optionally filtered by category (HDOC4).
    pub fn list_topics(&self, category: Option<HardwareDocCategory>) -> Vec<&HardwareDocTopic> {
        match category {
            Some(cat) => self.topics.iter().filter(|t| t.category == cat).collect(),
            None => self.topics.iter().collect(),
        }
    }

    /// Searches documentation topics with scored ranking and defensive bounds (HDOC3).
    pub fn search(&self, query: &str, category: Option<HardwareDocCategory>) -> Vec<HardwareDocSearchResult> {
        let trimmed = query.trim();
        if trimmed.is_empty() || trimmed.len() > MAX_DOC_QUERY_LEN || trimmed.chars().any(|c| c.is_control()) {
            return Vec::new();
        }
        let query_clean = trimmed.to_ascii_lowercase();

        let mut results = Vec::new();
        for topic in &self.topics {
            if let Some(cat) = category {
                if topic.category != cat {
                    continue;
                }
            }

            let mut score = 0;
            let mut matched_tags = Vec::new();
            let mut snippet = String::new();

            // 1. Exact ID match: +100
            if topic.id.to_ascii_lowercase() == query_clean {
                score += 100;
            } else if topic.id.to_ascii_lowercase().contains(&query_clean) {
                score += 40;
            }

            // 2. Tag match: +30 per matched tag
            for tag in &topic.tags {
                if tag.to_ascii_lowercase() == query_clean {
                    score += 50;
                    matched_tags.push(tag.clone());
                } else if tag.to_ascii_lowercase().contains(&query_clean) {
                    score += 20;
                    matched_tags.push(tag.clone());
                }
            }

            // 3. Title match: +35
            if topic.title.to_ascii_lowercase().contains(&query_clean) {
                score += 35;
            }

            // 4. Summary match: +15
            if topic.summary.to_ascii_lowercase().contains(&query_clean) {
                score += 15;
                if snippet.is_empty() {
                    snippet = topic.summary.clone();
                }
            }

            // 5. Section content match: +10
            for sec in &topic.sections {
                if sec.title.to_ascii_lowercase().contains(&query_clean) {
                    score += 10;
                }
                if let Some(idx) = sec.content.to_ascii_lowercase().find(&query_clean) {
                    score += 10;
                    if snippet.is_empty() {
                        let mut start = idx.saturating_sub(40);
                        while start > 0 && !sec.content.is_char_boundary(start) {
                            start -= 1;
                        }
                        let mut end = (idx + query_clean.len() + 60).min(sec.content.len());
                        while end < sec.content.len() && !sec.content.is_char_boundary(end) {
                            end += 1;
                        }
                        snippet = format!("...{}...", &sec.content[start..end]);
                    }
                }
            }

            if score > 0 {
                if snippet.is_empty() {
                    snippet = topic.summary.clone();
                }
                results.push(HardwareDocSearchResult {
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

    /// Formats a topic as readable Markdown (HDOC5).
    pub fn format_topic_markdown(topic: &HardwareDocTopic) -> String {
        let mut out = String::new();
        out.push_str(&format!("# {}\n\n", topic.title));
        out.push_str(&format!("**ID:** `{}` | **Category:** `{}`\n\n", topic.id, topic.category.as_str()));
        out.push_str(&format!("{}\n\n", topic.summary));

        for section in &topic.sections {
            out.push_str(&format!("## {}\n\n", section.title));
            out.push_str(&format!("{}\n\n", section.content.trim()));
        }

        if !topic.examples.is_empty() {
            out.push_str("## Examples\n\n```bash\n");
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

    fn register_canonical_topics(&mut self) {
        self.topics.push(HardwareDocTopic {
            id: "hw-sysfs-topology".into(),
            title: "Linux Sysfs Hardware Topology and Probing".into(),
            category: HardwareDocCategory::Discovery,
            summary: "Comprehensive reference on Linux /sys/bus and /sys/class virtual filesystems utilized for PCI, USB, block, network, CPU, and DMI device discovery.".into(),
            sections: vec![
                HardwareDocSection {
                    title: "PCI Subsystem".into(),
                    content: "PCI devices reside under /sys/bus/pci/devices. Each entry exposes vendor, device, class, and driver symlinks.".into(),
                },
                HardwareDocSection {
                    title: "USB Subsystem".into(),
                    content: "USB devices reside under /sys/bus/usb/devices, exposing idVendor, idProduct, manufacturer, and product strings.".into(),
                },
                HardwareDocSection {
                    title: "Storage & Network Subsystems".into(),
                    content: "Block storage resides under /sys/class/block (size, queue/rotational). Network interfaces reside under /sys/class/net (address, speed, operstate).".into(),
                },
            ],
            tags: vec!["sysfs".into(), "pci".into(), "usb".into(), "block".into(), "network".into(), "cpu".into(), "dmi".into()],
            references: vec![
                "Documentation/filesystems/sysfs.txt".into(),
                "Documentation/ABI/testing/sysfs-bus-pci".into(),
            ],
            examples: vec![
                "ls -la /sys/bus/pci/devices".into(),
                "cat /sys/class/net/eth0/address".into(),
            ],
        });

        self.topics.push(HardwareDocTopic {
            id: "hw-security-policy".into(),
            title: "Hardware Detection Security Policy and Gatekeeping".into(),
            category: HardwareDocCategory::Security,
            summary: "Declarative security policies for hardware inventories enforcing device allowlists, denylists, class/bus gatekeeping, and sensitive attribute redaction.".into(),
            sections: vec![
                HardwareDocSection {
                    title: "Policy Modes".into(),
                    content: "Enforcing mode filters out violating devices and yields 'deny'. Audit mode logs violations without modifying inventories. Permissive mode permits all devices.".into(),
                },
                HardwareDocSection {
                    title: "Attribute Redaction".into(),
                    content: "Sensitive keys (address, mac, serial, uuid, wwid) are automatically masked to '<REDACTED>' when redact_sensitive_attributes is enabled.".into(),
                },
            ],
            tags: vec!["security".into(), "policy".into(), "redaction".into(), "allowlist".into(), "denylist".into(), "gatekeeping".into()],
            references: vec![
                "ADR-0035 Security Architecture".into(),
                "CIS Linux Benchmark: Peripheral Devices".into(),
            ],
            examples: vec![
                "aiosh hardware scan --policy /etc/aios/hardware_policy.json".into(),
            ],
        });

        self.topics.push(HardwareDocTopic {
            id: "hw-observability-telemetry".into(),
            title: "Hardware Observability and Fleet Telemetry".into(),
            category: HardwareDocCategory::Observability,
            summary: "Structured telemetry reports detailing device class distributions, bus breakdowns, driver binding ratios, and compliance summaries.".into(),
            sections: vec![
                HardwareDocSection {
                    title: "Driver Binding Metrics".into(),
                    content: "Tracks driver_binding_count, unbound_device_count, and driver_binding_rate (ratio of bound devices to total devices).".into(),
                },
                HardwareDocSection {
                    title: "Telemetry Invariants".into(),
                    content: "Total device count must equal the sum of class_breakdown values (HO1) and bus_breakdown values (HO2).".into(),
                },
            ],
            tags: vec!["observability".into(), "telemetry".into(), "metrics".into(), "driver_binding".into(), "fleet".into()],
            references: vec![
                "OpenTelemetry Hardware Metrics Conventions".into(),
            ],
            examples: vec![
                "aiosh hardware observability --json".into(),
            ],
        });

        self.topics.push(HardwareDocTopic {
            id: "hw-config-options".into(),
            title: "Hardware Detection Configuration and Environment Overrides".into(),
            category: HardwareDocCategory::Configuration,
            summary: "Persistent configuration settings, resource limits, and environment variable overrides controlling hardware discovery.".into(),
            sections: vec![
                HardwareDocSection {
                    title: "HardwareConfig Schema".into(),
                    content: "Controls default_store_path, sysfs_path, procfs_path, enabled_classes, include_attributes, max_devices, and scan_timeout_secs.".into(),
                },
                HardwareDocSection {
                    title: "Environment Overrides".into(),
                    content: "AIOSH_HARDWARE_SYSFS, AIOSH_HARDWARE_PROCFS, AIOSH_HARDWARE_STORE, and AIOSH_HARDWARE_TIMEOUT_SECS override defaults.".into(),
                },
            ],
            tags: vec!["configuration".into(), "config".into(), "env".into(), "limits".into(), "timeouts".into()],
            references: vec![
                "docs/hardware_detection.md Section 11".into(),
            ],
            examples: vec![
                "export AIOSH_HARDWARE_TIMEOUT_SECS=60".into(),
            ],
        });

        self.topics.push(HardwareDocTopic {
            id: "hw-mcp-tools".into(),
            title: "Hardware Detection MCP Tool Catalog".into(),
            category: HardwareDocCategory::Architecture,
            summary: "Model Context Protocol (MCP) tool interfaces for hardware introspection (aios.hardware.scan, list, get, summary, verify).".into(),
            sections: vec![
                HardwareDocSection {
                    title: "Tool Catalog".into(),
                    content: "aios.hardware.scan performs comprehensive discovery; aios.hardware.list lists devices; aios.hardware.get inspects a specific device; aios.hardware.summary returns counts; aios.hardware.verify audits integrity.".into(),
                },
            ],
            tags: vec!["mcp".into(), "tools".into(), "scan".into(), "list".into(), "get".into(), "summary".into(), "verify".into()],
            references: vec![
                "docs/hardware_detection.md Section 10".into(),
            ],
            examples: vec![
                "call('aios.hardware.scan', {'include_attributes': true})".into(),
            ],
        });

        self.topics.push(HardwareDocTopic {
            id: "hw-troubleshooting".into(),
            title: "Hardware Detection Troubleshooting and Diagnostics".into(),
            category: HardwareDocCategory::Troubleshooting,
            summary: "Diagnostic procedures and resolutions for common hardware detection errors, missing devices, and hermetic mock testing.".into(),
            sections: vec![
                HardwareDocSection {
                    title: "Missing Devices in Sysfs".into(),
                    content: "Verify kernel module driver is loaded (lsmod). Ensure /sys is mounted properly. Check dmesg for PCI enumeration errors.".into(),
                },
                HardwareDocSection {
                    title: "MockSysfsBuilder Testing".into(),
                    content: "Use MockSysfsBuilder to simulate PCI, USB, block, network, CPU, and DMI hierarchies without requiring root access.".into(),
                },
            ],
            tags: vec!["troubleshooting".into(), "debug".into(), "diagnostics".into(), "mock".into(), "sysfs_builder".into()],
            references: vec![
                "docs/hardware_detection.md Section 12".into(),
            ],
            examples: vec![
                "dmesg | grep -i pci".into(),
            ],
        });
    }
}
