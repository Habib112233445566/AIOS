//! Kernel Module Documentation and Reference Index (KD1..KD6).
//!
//! Provides an offline, self-contained documentation repository for Linux
//! kernel module management, modprobe directives, CIS benchmark baselines,
//! operational lifecycles, and security policies.

use serde::{Deserialize, Serialize};

/// Categories for kernel module documentation topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocCategory {
    Directive,
    Lifecycle,
    Security,
    Observability,
    Baseline,
}

impl DocCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocCategory::Directive => "directive",
            DocCategory::Lifecycle => "lifecycle",
            DocCategory::Security => "security",
            DocCategory::Observability => "observability",
            DocCategory::Baseline => "baseline",
        }
    }
}

/// A structured section within a documentation topic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocSection {
    pub title: String,
    pub content: String,
}

/// A complete documentation topic with metadata, sections, examples, and references.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocTopic {
    pub id: String,
    pub title: String,
    pub category: DocCategory,
    pub summary: String,
    pub sections: Vec<DocSection>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub examples: Vec<String>,
}

/// Scored search result returned from `KernelModuleDocIndex::search`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
    pub matched_tags: Vec<String>,
}

/// Maximum allowed length for search queries (256 characters, hardened T-01688).
pub const MAX_DOC_QUERY_LEN: usize = 256;

/// Maximum number of search results returned (50, hardened T-01688).
pub const MAX_DOC_SEARCH_RESULTS: usize = 50;

/// Maximum length for topic identifiers (64 characters, hardened T-01688).
pub const MAX_TOPIC_ID_LEN: usize = 64;

/// Offline in-memory index of kernel module documentation.
#[derive(Debug, Clone)]
pub struct KernelModuleDocIndex {
    pub topics: Vec<DocTopic>,
}

impl Default for KernelModuleDocIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelModuleDocIndex {
    /// Creates a new documentation index pre-populated with canonical topics (KD1).
    pub fn new() -> Self {
        let mut idx = KernelModuleDocIndex {
            topics: Vec::new(),
        };
        idx.register_canonical_topics();
        idx
    }

    /// Looks up a topic by ID (case-insensitive) with defensive bounds (KD2, hardened T-01688).
    pub fn get_topic(&self, id: &str) -> Option<&DocTopic> {
        let id_clean = id.trim();
        if id_clean.is_empty() || id_clean.len() > MAX_TOPIC_ID_LEN || id_clean.chars().any(|c| c.is_control()) {
            return None;
        }
        self.topics.iter().find(|t| t.id.eq_ignore_ascii_case(id_clean))
    }

    /// Lists all topics in the index (KD1).
    pub fn list_topics(&self) -> Vec<&DocTopic> {
        self.topics.iter().collect()
    }

    /// Lists topics belonging to a specific category.
    pub fn list_by_category(&self, category: DocCategory) -> Vec<&DocTopic> {
        self.topics.iter().filter(|t| t.category == category).collect()
    }

    /// Searches documentation topics with scored ranking and defensive bounds (KD3, hardened T-01688).
    pub fn search(&self, query: &str) -> Vec<DocSearchResult> {
        let trimmed = query.trim();
        if trimmed.is_empty() || trimmed.len() > MAX_DOC_QUERY_LEN || trimmed.chars().any(|c| c.is_control()) {
            return Vec::new();
        }
        let query_clean = trimmed.to_ascii_lowercase();

        let mut results = Vec::new();
        for topic in &self.topics {
            let mut score = 0;
            let mut matched_tags = Vec::new();
            let mut snippet = String::new();

            // 1. Exact ID match: +100
            if topic.id.to_ascii_lowercase() == query_clean {
                score += 100;
            } else if topic.id.to_ascii_lowercase().contains(&query_clean) {
                score += 40;
            }

            // 2. Tag match: +50 per matched tag
            for tag in &topic.tags {
                if tag.to_ascii_lowercase() == query_clean {
                    score += 50;
                    matched_tags.push(tag.clone());
                } else if tag.to_ascii_lowercase().contains(&query_clean) {
                    score += 20;
                    matched_tags.push(tag.clone());
                }
            }

            // 3. Title match: +25
            if topic.title.to_ascii_lowercase().contains(&query_clean) {
                score += 25;
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
                        let start = idx.saturating_sub(40);
                        let end = (idx + query_clean.len() + 60).min(sec.content.len());
                        snippet = format!("...{}...", &sec.content[start..end]);
                    }
                }
            }

            if score > 0 {
                if snippet.is_empty() {
                    snippet = topic.summary.clone();
                }
                results.push(DocSearchResult {
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

    /// Formats a topic as readable Markdown (KD4).
    pub fn format_topic_markdown(topic: &DocTopic) -> String {
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

        if !topic.tags.is_empty() {
            out.push_str(&format!("**Tags:** {}\n", topic.tags.join(", ")));
        }

        out
    }

    /// Populates canonical built-in topics into the index (KD1).
    fn register_canonical_topics(&mut self) {
        self.topics.push(DocTopic {
            id: "modprobe-directives".into(),
            title: "Linux modprobe.d Configuration Directives".into(),
            category: DocCategory::Directive,
            summary: "Comprehensive specification of modprobe directives including alias, blacklist, options, install, remove, and softdep.".into(),
            sections: vec![
                DocSection {
                    title: "Overview of modprobe.d".into(),
                    content: "The /etc/modprobe.d directory contains configuration files for modprobe(8). All files ending in .conf are processed in ASCII sort order.".into(),
                },
                DocSection {
                    title: "Supported Directives".into(),
                    content: "- `alias <wildcard> <modulename>`: Maps alternative identifiers or hardware IDs to target modules.\n- `blacklist <modulename>`: Ignores internal module aliases so that modprobe will not load it automatically on hardware discovery.\n- `options <modulename> <option...>`: Supplies static parameters whenever the module is loaded.\n- `install <modulename> <command...>`: Executes custom command instead of the standard in-kernel init_module.\n- `remove <modulename> <command...>`: Executes custom command when the module is removed via modprobe -r.\n- `softdep <modulename> pre: <mods...> post: <mods...>`: Defines optional soft dependencies loaded before or after the primary module.".into(),
                },
            ],
            tags: vec!["modprobe".into(), "modprobe.d".into(), "blacklist".into(), "options".into(), "install".into(), "alias".into(), "softdep".into(), "remove".into()],
            references: vec!["modprobe.d(5)".into(), "modprobe(8)".into(), "Documentation/kbuild/kconfig.rst".into()],
            examples: vec![
                "blacklist cramfs".into(),
                "install tipc /bin/true".into(),
                "options overlay metacopy=on".into(),
                "alias net-pf-10 ipv6".into(),
            ],
        });

        self.topics.push(DocTopic {
            id: "cis-benchmark-hardening".into(),
            title: "CIS Linux Benchmark Kernel Module Hardening".into(),
            category: DocCategory::Security,
            summary: "Hardening baseline disabling legacy filesystems and vulnerable protocols according to CIS distribution benchmarks.".into(),
            sections: vec![
                DocSection {
                    title: "Legacy Filesystem Disabling".into(),
                    content: "CIS Section 1.1.1 mandates disabling legacy, rare, and unmaintained filesystems to reduce kernel attack surface: cramfs, freevxfs, jffs2, hfs, hfsplus, and udf.".into(),
                },
                DocSection {
                    title: "Vulnerable Protocol Disabling".into(),
                    content: "CIS Section 3.4 mandates disabling obsolete or rare network protocols: dccp, sctp, rds, and tipc.".into(),
                },
                DocSection {
                    title: "Double-Barreled Disabling Pattern".into(),
                    content: "Standard disabling requires two coordinated directives:\n1. `install <module> /bin/true`: Replaces the module loading routine with a no-op command that succeeds.\n2. `blacklist <module>`: Prevents autoloading triggered by device discovery or protocol socket creation.".into(),
                },
            ],
            tags: vec!["cis".into(), "benchmark".into(), "hardening".into(), "filesystems".into(), "protocols".into(), "security".into()],
            references: vec!["CIS Distribution Benchmark v2.0 §1.1.1".into(), "CIS Distribution Benchmark v2.0 §3.4".into()],
            examples: vec![
                "install cramfs /bin/true".into(),
                "blacklist cramfs".into(),
                "aiosh mod apply-preset cis_hardened_baseline".into(),
            ],
        });

        self.topics.push(DocTopic {
            id: "lifecycle-workflows".into(),
            title: "Kernel Module Lifecycle & Dependency Workflows".into(),
            category: DocCategory::Lifecycle,
            summary: "Operational guide covering module loading, unloading, runtime state transitions, refcounts, and dependency chains.".into(),
            sections: vec![
                DocSection {
                    title: "Runtime States".into(),
                    content: "Modules exist in one of four states: Live (active in kernel memory), Loading (in-flight initialization), Unloading (cleanup routines running), or Unloaded (absent from memory).".into(),
                },
                DocSection {
                    title: "Dependency Management".into(),
                    content: "Kernel module dependencies are resolved via modules.dep generated by depmod. When loading a module, all prerequisites are loaded in topological order.".into(),
                },
                DocSection {
                    title: "Safe Removal & Refcounts".into(),
                    content: "A module can only be unloaded when its reference count is zero. Unloading modules with non-zero dependents or active users will result in EBUSY or system instability.".into(),
                },
            ],
            tags: vec!["lifecycle".into(), "modprobe".into(), "rmmod".into(), "insmod".into(), "lsmod".into(), "refcount".into(), "state".into()],
            references: vec!["lsmod(8)".into(), "rmmod(8)".into(), "insmod(8)".into(), "modules.dep(5)".into()],
            examples: vec![
                "aiosh mod list".into(),
                "aiosh mod inspect overlay".into(),
                "modprobe -v overlay".into(),
            ],
        });

        self.topics.push(DocTopic {
            id: "observability-and-procfs".into(),
            title: "Kernel Module Telemetry & Procfs Inspection".into(),
            category: DocCategory::Observability,
            summary: "Guide to kernel module observability via /proc/modules, sysfs, memory tracking, and telemetry aggregation.".into(),
            sections: vec![
                DocSection {
                    title: "/proc/modules Schema".into(),
                    content: "Each line in /proc/modules contains 6 whitespace-delimited fields: Name, Size in bytes, Instance count (refcount), Sub-modules referring to it, Module state (Live/Loading/Unloading), and Kernel load address.".into(),
                },
                DocSection {
                    title: "Sysfs Hierarchy".into(),
                    content: "Kernel modules expose attributes and configurable runtime parameters under /sys/module/<name>/. Parameters can be inspected and updated dynamically via sysfs nodes.".into(),
                },
                DocSection {
                    title: "KASLR Address Stripping".into(),
                    content: "To prevent Kernel Address Space Layout Randomization (KASLR) defeat, AIOS observability intentionally strips raw memory load addresses from emitted reports.".into(),
                },
            ],
            tags: vec!["observability".into(), "telemetry".into(), "procfs".into(), "sysfs".into(), "proc/modules".into(), "kaslr".into(), "memory".into()],
            references: vec!["proc(5)".into(), "sysfs(5)".into(), "Documentation/filesystems/proc.rst".into()],
            examples: vec![
                "cat /proc/modules".into(),
                "aiosh mod observability".into(),
                "aiosh mod status --json".into(),
            ],
        });

        self.topics.push(DocTopic {
            id: "security-policy-and-pep".into(),
            title: "Kernel Module Security Policy & PEP Capability Grants".into(),
            category: DocCategory::Security,
            summary: "Security policy enforcement rules (SP-KM1..SP-KM6), PEP capability gates, prohibited modules, and protected modules.".into(),
            sections: vec![
                DocSection {
                    title: "Policy Rules & Checks".into(),
                    content: "- SP-KM1: Module name and configuration bounds.\n- SP-KM2: Prohibited module blacklisting and autoload prevention.\n- SP-KM3: Protected critical modules guard against uninstallation.\n- SP-KM4: Install command sanitization (/bin/true or /bin/false only).\n- SP-KM5: Dangerous parameter rejection (e.g. panic, init overrides).\n- SP-KM6: Tri-state modes (Enforcing, Permissive, Audit) and size caps.".into(),
                },
                DocSection {
                    title: "PEP Capability Gating".into(),
                    content: "Mutating kernel configurations requires explicit PEP grants. Read-only observability and documentation queries do not require elevated capabilities.".into(),
                },
            ],
            tags: vec!["policy".into(), "security".into(), "pep".into(), "prohibited".into(), "protected".into(), "audit".into(), "enforcing".into()],
            references: vec!["AIOS Security Policy Specification §10".into(), "ADR-0034 PEP Architecture".into()],
            examples: vec![
                "aiosh mod policy --json".into(),
                "aiosh mod policy cramfs".into(),
                "aiosh mod policy --evaluate-store".into(),
            ],
        });

        self.topics.push(DocTopic {
            id: "container-isolation".into(),
            title: "Container & Sandbox Namespace Module Configuration".into(),
            category: DocCategory::Baseline,
            summary: "Configuration guidelines for container virtualization, overlayfs drivers, network bridging, and Landlock/seccomp.".into(),
            sections: vec![
                DocSection {
                    title: "Overlay Filesystem Optimization".into(),
                    content: "OverlayFS enables lightweight container root filesystems. Optimizations include `metacopy=on` for fast metadata copy-up and `redirect_dir=on` for rename operations.".into(),
                },
                DocSection {
                    title: "Networking Virtualization".into(),
                    content: "Bridge netfilter (br_netfilter) and TUN/TAP devices provide packet routing between host network namespaces and guest sandboxes.".into(),
                },
            ],
            tags: vec!["container".into(), "overlay".into(), "tun".into(), "tap".into(), "br_netfilter".into(), "namespace".into(), "docker".into(), "podman".into()],
            references: vec!["Documentation/filesystems/overlayfs.rst".into(), "ip-link(8)".into()],
            examples: vec![
                "options overlay metacopy=on".into(),
                "aiosh mod apply-preset container_isolation_baseline".into(),
            ],
        });

        self.topics.push(DocTopic {
            id: "wireless-pentest".into(),
            title: "Penetration Testing Wireless Driver Baseline".into(),
            category: DocCategory::Baseline,
            summary: "Hardware driver baseline and configuration for wireless penetration testing, packet injection, and monitor mode.".into(),
            sections: vec![
                DocSection {
                    title: "Atheros & Realtek Hardware Drivers".into(),
                    content: "Standard penetration testing dongles rely on ath9k_htc (Atheros AR9271) and rtl8812au (Realtek 802.11ac). These drivers support monitor mode and packet injection.".into(),
                },
                DocSection {
                    title: "Hardware Cryptography Disablement".into(),
                    content: "Disabling hardware crypto offload via `nohwcrypt=1` on ath9k_htc allows userspace tools (aircrack-ng suite) to intercept raw encrypted 802.11 frames without NIC dropping.".into(),
                },
            ],
            tags: vec!["pentest".into(), "wireless".into(), "injection".into(), "monitor-mode".into(), "ath9k".into(), "rtl8812au".into(), "cfg80211".into(), "mac80211".into()],
            references: vec!["wireless(7)".into(), "iw(8)".into(), "airmon-ng(8)".into()],
            examples: vec![
                "options ath9k_htc nohwcrypt=1".into(),
                "options rtl8812au rtw_vht_enable=1".into(),
                "aiosh mod apply-preset pentest_wireless_baseline".into(),
            ],
        });
    }
}
