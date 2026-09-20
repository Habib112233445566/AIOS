//! System Update Documentation Subsystem (UDOC1..UDOC6).
//!
//! Provides an offline, pre-populated reference repository, scored full-text search,
//! and dynamic Markdown report rendering for the AIOS System Update Mechanism.

use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::system_update_service::SystemUpdateService;
use crate::system_update_observability::SystemUpdateObservabilityReport;

/// Categorical domains for system update documentation topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemUpdateDocCategory {
    Architecture,
    ABPartitioning,
    Security,
    Observability,
    Configuration,
    Troubleshooting,
}

impl SystemUpdateDocCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Architecture => "architecture",
            Self::ABPartitioning => "ab_partitioning",
            Self::Security => "security",
            Self::Observability => "observability",
            Self::Configuration => "configuration",
            Self::Troubleshooting => "troubleshooting",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "architecture" | "arch" => Some(Self::Architecture),
            "ab_partitioning" | "ab" | "partitioning" | "slots" => Some(Self::ABPartitioning),
            "security" | "sec" | "policy" => Some(Self::Security),
            "observability" | "obs" | "telemetry" | "metrics" => Some(Self::Observability),
            "configuration" | "config" | "conf" => Some(Self::Configuration),
            "troubleshooting" | "trouble" | "debug" | "repair" => Some(Self::Troubleshooting),
            _ => None,
        }
    }
}

/// A technical documentation topic in the system update repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemUpdateDocTopic {
    pub id: String,
    pub title: String,
    pub category: SystemUpdateDocCategory,
    pub tags: Vec<String>,
    pub content: String,
    pub see_also: Vec<String>,
}

impl SystemUpdateDocTopic {
    pub fn to_markdown(&self) -> String {
        let mut md = format!("# {}\n\n", self.title);
        md.push_str(&format!("- **ID**: `{}`\n", self.id));
        md.push_str(&format!("- **Category**: `{}`\n", self.category.as_str()));
        md.push_str(&format!("- **Tags**: {}\n\n", self.tags.join(", ")));
        md.push_str(&self.content);
        md.push_str("\n\n");
        if !self.see_also.is_empty() {
            md.push_str("### See Also\n");
            for link in &self.see_also {
                md.push_str(&format!("- `{}`\n", link));
            }
            md.push('\n');
        }
        md
    }
}

/// Search result with calculated relevance score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemUpdateDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub category: SystemUpdateDocCategory,
    pub score: u32,
    pub snippet: String,
}

/// In-memory repository of system update documentation topics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemUpdateDocIndex {
    pub topics: Vec<SystemUpdateDocTopic>,
}

impl Default for SystemUpdateDocIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemUpdateDocIndex {
    /// Initializes documentation index pre-populated with canonical technical topics (UDOC1).
    pub fn new() -> Self {
        let topics = vec![
            SystemUpdateDocTopic {
                id: "arch-overview".to_string(),
                title: "AIOS System Update Architecture Overview".to_string(),
                category: SystemUpdateDocCategory::Architecture,
                tags: vec!["architecture".to_string(), "dual-slot".to_string(), "lifecycle".to_string()],
                content: "AIOS implements an atomic A/B dual-partition update architecture ensuring zero-downtime and resilient rollbacks.".to_string(),
                see_also: vec!["ab-slots".to_string(), "security-policy".to_string()],
            },
            SystemUpdateDocTopic {
                id: "ab-slots".to_string(),
                title: "A/B Dual-Slot Partition Layout & Switching".to_string(),
                category: SystemUpdateDocCategory::ABPartitioning,
                tags: vec!["ab".to_string(), "slots".to_string(), "bootloader".to_string(), "partitions".to_string()],
                content: "The system alternates between Slot A and Slot B partitions for rootfs, kernel, and initramfs artifacts.".to_string(),
                see_also: vec!["arch-overview".to_string(), "troubleshooting-rollback".to_string()],
            },
            SystemUpdateDocTopic {
                id: "security-policy".to_string(),
                title: "System Update Cryptographic Security Policy".to_string(),
                category: SystemUpdateDocCategory::Security,
                tags: vec!["security".to_string(), "policy".to_string(), "signature".to_string(), "anti-rollback".to_string()],
                content: "Enforces channel authorization, Ed25519 signature verification, semver downgrade prevention, and revocation denylists.".to_string(),
                see_also: vec!["config-schema".to_string()],
            },
            SystemUpdateDocTopic {
                id: "observability-telemetry".to_string(),
                title: "System Update Observability & Telemetry".to_string(),
                category: SystemUpdateDocCategory::Observability,
                tags: vec!["observability".to_string(), "telemetry".to_string(), "health".to_string(), "metrics".to_string()],
                content: "Provides unified reporting of slot health, staging progress, policy evaluation, and sanitized telemetry data.".to_string(),
                see_also: vec!["arch-overview".to_string()],
            },
            SystemUpdateDocTopic {
                id: "config-schema".to_string(),
                title: "System Update Configuration Schema & Options".to_string(),
                category: SystemUpdateDocCategory::Configuration,
                tags: vec!["configuration".to_string(), "options".to_string(), "limits".to_string(), "env".to_string()],
                content: "Configures update directories, polling intervals, quota limits, and trusted key digests.".to_string(),
                see_also: vec!["security-policy".to_string()],
            },
            SystemUpdateDocTopic {
                id: "troubleshooting-rollback".to_string(),
                title: "System Update Troubleshooting & Rollback Procedures".to_string(),
                category: SystemUpdateDocCategory::Troubleshooting,
                tags: vec!["troubleshooting".to_string(), "rollback".to_string(), "recovery".to_string(), "boot-failure".to_string()],
                content: "Details procedures for automated and manual rollback to the previous functional partition upon boot failure.".to_string(),
                see_also: vec!["ab-slots".to_string()],
            },
        ];

        Self { topics }
    }

    /// Finds a topic by its exact unique ID (UDOC1).
    pub fn get_by_id(&self, id: &str) -> Option<&SystemUpdateDocTopic> {
        self.topics.iter().find(|t| t.id == id)
    }

    /// Lists topics by category (UDOC2).
    pub fn list_by_category(&self, category: SystemUpdateDocCategory) -> Vec<&SystemUpdateDocTopic> {
        self.topics.iter().filter(|t| t.category == category).collect()
    }

    /// Searches documentation index using tokenized keyword scoring (UDOC3).
    pub fn search(&self, query: &str) -> Vec<SystemUpdateDocSearchResult> {
        let q_clean: String = query.chars().take(256).collect::<String>().trim().to_ascii_lowercase();
        if q_clean.is_empty() {
            return Vec::new();
        }

        let tokens: Vec<&str> = q_clean.split_whitespace().collect();
        let mut results = Vec::new();

        for topic in &self.topics {
            let mut score = 0u32;
            let id_lower = topic.id.to_ascii_lowercase();
            let title_lower = topic.title.to_ascii_lowercase();
            let content_lower = topic.content.to_ascii_lowercase();

            if id_lower == q_clean {
                score += 100;
            }

            for token in &tokens {
                if id_lower.contains(token) {
                    score += 50;
                }
                if title_lower.contains(token) {
                    score += 40;
                }
                for tag in &topic.tags {
                    if tag.to_ascii_lowercase().contains(token) {
                        score += 20;
                    }
                }
                if content_lower.contains(token) {
                    score += 5;
                }
            }

            if score > 0 {
                let snippet = if topic.content.len() > 120 {
                    format!("{}...", &topic.content[..120])
                } else {
                    topic.content.clone()
                };

                results.push(SystemUpdateDocSearchResult {
                    topic_id: topic.id.clone(),
                    title: topic.title.clone(),
                    category: topic.category,
                    score,
                    snippet,
                });
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(50);
        results
    }

    /// Renders the complete documentation index into formatted Markdown (UDOC4).
    pub fn render_full_index_markdown(&self) -> String {
        let mut md = String::from("# AIOS System Update Documentation Index\n\n");
        md.push_str("Comprehensive reference documentation for the AIOS system update engine.\n\n");

        for category in &[
            SystemUpdateDocCategory::Architecture,
            SystemUpdateDocCategory::ABPartitioning,
            SystemUpdateDocCategory::Security,
            SystemUpdateDocCategory::Observability,
            SystemUpdateDocCategory::Configuration,
            SystemUpdateDocCategory::Troubleshooting,
        ] {
            md.push_str(&format!("## Category: `{}`\n\n", category.as_str()));
            let cat_topics = self.list_by_category(*category);
            for topic in cat_topics {
                md.push_str(&topic.to_markdown());
                md.push_str("---\n\n");
            }
        }

        md
    }

    /// Generates dynamic Markdown report of live system update status with ASCII slot diagram (UDOC5).
    pub fn render_status_markdown(service: &SystemUpdateService) -> String {
        let slot_status = &service.slot_status;
        let update_status = &service.update_status;

        let mut md = String::from("# Live System Update Status Report\n\n");
        md.push_str("### Partition Slots Status\n\n");
        md.push_str(&format!("- **Current Active Slot**: `{:?}`\n", slot_status.current_slot));
        md.push_str(&format!("- **Target Update Slot**: `{:?}`\n", slot_status.target_slot));
        md.push_str(&format!("- **Rollback Slot**: `{:?}`\n", slot_status.rollback_slot));
        md.push_str(&format!("- **Slot A Version**: `{}` (successful: `{}`)\n", slot_status.slot_a_version, slot_status.slot_a_successful));
        md.push_str(&format!("- **Slot B Version**: `{}` (successful: `{}`)\n\n", slot_status.slot_b_version, slot_status.slot_b_successful));

        md.push_str("### A/B Partition Visual Diagram\n```text\n");
        let a_active = if slot_status.current_slot == crate::system_update::UpdateSlot::SlotA { "[ACTIVE]" } else { "[INACTIVE]" };
        let b_active = if slot_status.current_slot == crate::system_update::UpdateSlot::SlotB { "[ACTIVE]" } else { "[INACTIVE]" };
        md.push_str(&format!("+-----------------------+   +-----------------------+\n"));
        md.push_str(&format!("| Slot A: {:<13} |   | Slot B: {:<13} |\n", a_active, b_active));
        md.push_str(&format!("| Version: {:<12} |   | Version: {:<12} |\n", slot_status.slot_a_version, slot_status.slot_b_version));
        md.push_str(&format!("+-----------------------+   +-----------------------+\n```\n\n"));

        md.push_str("### Update Lifecycle State\n\n");
        md.push_str(&format!("- **State**: `{:?}`\n", update_status.state));
        md.push_str(&format!("- **Progress**: `{}%`\n", update_status.progress_percent));
        md.push_str(&format!("- **Current Running Version**: `{}`\n", update_status.current_version));
        if let Some(ref target) = update_status.target_version {
            md.push_str(&format!("- **Target Version**: `{}`\n", target));
        }
        if let Some(ref err) = update_status.last_error {
            md.push_str(&format!("- **Last Error**: `{}`\n", err));
        }

        md
    }

    /// Generates dynamic Markdown report of live observability report (UDOC5).
    pub fn render_observability_markdown(report: &SystemUpdateObservabilityReport) -> String {
        let mut md = String::from("# System Update Observability Telemetry Summary\n\n");
        md.push_str(&format!("- **Generated At**: `{}`\n", report.generated_at));
        md.push_str(&format!("- **Health Status**: `{}`\n", if report.is_healthy { "HEALTHY" } else { "DEGRADED" }));
        md.push_str(&format!("- **Current Slot**: `{:?}`\n", report.current_slot));
        md.push_str(&format!("- **Lifecycle State**: `{:?}` ({}%)\n", report.state, report.progress_percent));
        md.push_str(&format!("- **Staged Artifacts**: {} ({} bytes)\n", report.staged_artifacts_count, report.staged_payload_bytes));
        if let Some(ref verdict) = report.policy_verdict {
            md.push_str(&format!("- **Policy Verdict**: `{}` (violations: {})\n", verdict, report.policy_violations_count));
        }
        md
    }

    /// Exports full documentation to file atomically with symlink defense and size limit (UDOC6).
    pub fn export_to_file(&self, path: &Path) -> Result<(), String> {
        if path.to_string_lossy().len() > 1024 {
            return Err("export path exceeds maximum length of 1024 characters".to_string());
        }

        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(format!("path {:?} contains parent directory traversal ('..') which is disallowed", path));
        }

        let content = self.render_full_index_markdown();
        if content.len() > 1024 * 1024 {
            return Err("documentation export exceeds maximum size (1 MB)".to_string());
        }

        if let Ok(meta) = std::fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() {
                return Err(format!("destination path {:?} is a symlink (symlink attack rejected)", path));
            }
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create parent directories for {:?}: {}", path, e))?;
        }

        let tmp_path = format!("{}.tmp.{}", path.to_string_lossy(), std::process::id());
        let tmp_path = std::path::PathBuf::from(tmp_path);

        if let Err(e) = std::fs::write(&tmp_path, content.as_bytes()) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("failed to write temporary documentation file {:?}: {}", tmp_path, e));
        }

        if let Err(e) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("failed to rename temporary documentation file to {:?}: {}", path, e));
        }

        Ok(())
    }
}
