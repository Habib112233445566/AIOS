//! Documentation Index Control core service (T-00424).
//!
//! Contract: `docs/tasks/evidence/T-00422-core-service-specification.md`.

use crate::doc_index::{DocIndexEntry, DocIndexManifest};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const MAX_DOC_BYTES: u64 = 16 * 1024 * 1024; // 16 MiB

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrokenDocLink {
    pub source_path: String,
    pub target_link: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocLinkValidationReport {
    pub total_links_checked: usize,
    pub broken_links: Vec<BrokenDocLink>,
    pub is_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocIndexTelemetry {
    pub total_docs_indexed: usize,
    pub total_links_checked: usize,
    pub broken_links_count: usize,
    pub is_healthy: bool,
}

/// Collects telemetry aggregate metrics for Documentation Index Control (T-00484).
pub fn collect_doc_index_telemetry(
    manifest: &DocIndexManifest,
    report: Option<&DocLinkValidationReport>,
) -> DocIndexTelemetry {
    let total_docs_indexed = manifest.entries.len();
    let (total_links_checked, broken_links_count, is_healthy) = match report {
        Some(r) => (r.total_links_checked, r.broken_links.len(), r.is_valid),
        None => {
            let total_outbound: usize = manifest.entries.iter().map(|e| e.links.len()).sum();
            (total_outbound, 0, true)
        }
    };

    DocIndexTelemetry {
        total_docs_indexed,
        total_links_checked,
        broken_links_count,
        is_healthy,
    }
}

/// Formats a human-readable text summary of a DocIndexManifest (T-00494).
pub fn format_doc_index_summary(manifest: &DocIndexManifest) -> String {
    let mut out = format!("AIOS Documentation Index (v{}):\n", manifest.version);
    if manifest.entries.is_empty() {
        out.push_str("  (no documents indexed)");
        return out;
    }
    for (i, entry) in manifest.entries.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!("  [{}] {} ({})", entry.section, entry.title, entry.path));
    }
    out
}

/// Parses the first top-level Markdown title (# Title) from content (T-00424).
pub fn parse_markdown_title(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            let title = trimmed[2..].trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}

/// Parses all in-tree relative markdown links from content (T-00424).
pub fn parse_markdown_links(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut seen = HashSet::new();

    let mut rest = content;
    while let Some(open_bracket) = rest.find('[') {
        rest = &rest[open_bracket + 1..];
        if let Some(close_bracket) = rest.find(']') {
            let after_bracket = &rest[close_bracket + 1..];
            if after_bracket.starts_with('(') {
                if let Some(close_paren) = after_bracket.find(')') {
                    let raw_link = &after_bracket[1..close_paren].trim();
                    // Strip optional title in link e.g. (path "title")
                    let target = if let Some(space_pos) = raw_link.find(' ') {
                        &raw_link[..space_pos]
                    } else {
                        raw_link
                    };
                    let clean_target = target.trim_matches(|c| c == '<' || c == '>');
                    // Strip anchor #section
                    let path_part = if let Some(hash_pos) = clean_target.find('#') {
                        &clean_target[..hash_pos]
                    } else {
                        clean_target
                    };

                    if !path_part.is_empty()
                        && !path_part.starts_with("http://")
                        && !path_part.starts_with("https://")
                        && !path_part.starts_with("mailto:")
                        && !path_part.starts_with("ftp://")
                    {
                        if seen.insert(path_part.to_string()) {
                            links.push(path_part.to_string());
                        }
                    }
                    rest = &after_bracket[close_paren + 1..];
                    continue;
                }
            }
            rest = &rest[close_bracket + 1..];
        }
    }
    links
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Prefix(p) => components.push(p.as_os_str().to_string_lossy().to_string()),
            std::path::Component::RootDir => components.push(std::path::MAIN_SEPARATOR.to_string()),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::Normal(c) => components.push(c.to_string_lossy().to_string()),
        }
    }
    let mut buf = PathBuf::new();
    for (i, c) in components.iter().enumerate() {
        if i == 0 && (c == "/" || c == "\\") {
            buf.push(std::path::MAIN_SEPARATOR.to_string());
        } else {
            buf.push(c);
        }
    }
    buf
}

/// Validates that all links in a manifest exist and remain inside repo_root (T-00424).
pub fn validate_doc_links(repo_root: &Path, manifest: &DocIndexManifest) -> DocLinkValidationReport {
    let mut total_links_checked = 0;
    let mut broken_links = Vec::new();

    let root_canon = repo_root.canonicalize().unwrap_or_else(|_| repo_root.to_path_buf());

    for entry in &manifest.entries {
        let source_file = repo_root.join(&entry.path);
        let parent_dir = source_file.parent().unwrap_or(repo_root);

        for link in &entry.links {
            total_links_checked += 1;
            let target_path = parent_dir.join(link);
            
            // Check normalized path for escapes
            let norm = normalize_path(&target_path);
            let norm_root = normalize_path(repo_root);
            if !norm.starts_with(&norm_root) {
                broken_links.push(BrokenDocLink {
                    source_path: entry.path.clone(),
                    target_link: link.clone(),
                    reason: "Link escapes repository root".into(),
                });
                continue;
            }

            if let Ok(canon) = target_path.canonicalize() {
                if !canon.starts_with(&root_canon) {
                    broken_links.push(BrokenDocLink {
                        source_path: entry.path.clone(),
                        target_link: link.clone(),
                        reason: "Link escapes repository root".into(),
                    });
                }
            } else if !target_path.exists() {
                broken_links.push(BrokenDocLink {
                    source_path: entry.path.clone(),
                    target_link: link.clone(),
                    reason: "Target file does not exist on disk".into(),
                });
            }
        }
    }

    let is_valid = broken_links.is_empty();
    DocLinkValidationReport {
        total_links_checked,
        broken_links,
        is_valid,
    }
}

/// Builds a DocIndexManifest from a list of relative doc paths (T-00424).
pub fn build_doc_index_from_paths(repo_root: &Path, doc_paths: &[&str]) -> Result<DocIndexManifest, String> {
    let mut entries = Vec::new();

    for rel_path in doc_paths {
        let full_path = repo_root.join(rel_path);
        if !full_path.exists() {
            return Err(format!("Document not found at {}", rel_path));
        }

        let mut file = File::open(&full_path)
            .map_err(|e| format!("Failed to open document {}: {}", rel_path, e))?;
        let mut content = String::new();
        file.by_ref()
            .take(MAX_DOC_BYTES)
            .read_to_string(&mut content)
            .map_err(|e| format!("Failed to read document {}: {}", rel_path, e))?;

        let title = parse_markdown_title(&content)
            .unwrap_or_else(|| {
                Path::new(rel_path)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| rel_path.to_string())
            });

        let links = parse_markdown_links(&content);

        let section = if rel_path.starts_with("docs/tasks/") {
            "Task Ledger".to_string()
        } else if rel_path.starts_with("docs/") {
            "Documentation".to_string()
        } else {
            "General".to_string()
        };

        entries.push(DocIndexEntry {
            path: rel_path.to_string(),
            title,
            section,
            task_range: None,
            links,
        });
    }

    let manifest = DocIndexManifest {
        version: "1.0.0".into(),
        entries,
    };
    manifest.validate()?;
    Ok(manifest)
}

/// Checks security policy for documentation index actions (T-00474).
pub fn check_doc_index_policy(grant: Option<&str>, tool_name: &str) -> Result<(), String> {
    if crate::pep::is_irreversible(tool_name) {
        match grant {
            Some(g) if !g.trim().is_empty() => Ok(()),
            _ => Err(format!(
                "Security policy violation: '{}' is an irreversible mutating action and requires an active PEP grant",
                tool_name
            )),
        }
    } else {
        Ok(())
    }
}

/// Recovers canonical default configuration for Documentation Index Control (T-00504).
pub fn recover_default_doc_index_config() -> crate::doc_index_config::DocIndexConfig {
    crate::doc_index_config::DocIndexConfig::default()
}

/// Validates all links in a document index manifest, returning aggregate telemetry on success (T-00504).
pub fn validate_doc_index_catalog(
    repo_root: &Path,
    manifest: &DocIndexManifest,
) -> Result<DocIndexTelemetry, String> {
    let report = validate_doc_links(repo_root, manifest);
    let telemetry = collect_doc_index_telemetry(manifest, Some(&report));
    if report.is_valid {
        Ok(telemetry)
    } else {
        Err(format!(
            "Documentation link validation failed: {} broken link(s) detected",
            report.broken_links.len()
        ))
    }
}

/// Reconciles documentation index manifest, validation report, and telemetry (T-00504).
pub fn reconcile_doc_index(
    repo_root: &Path,
    doc_paths: &[&str],
) -> Result<(DocIndexManifest, DocLinkValidationReport, DocIndexTelemetry), String> {
    let manifest = build_doc_index_from_paths(repo_root, doc_paths)?;
    let report = validate_doc_links(repo_root, &manifest);
    let telemetry = collect_doc_index_telemetry(&manifest, Some(&report));
    Ok((manifest, report, telemetry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recover_default_doc_index_config_happy() {
        let config = recover_default_doc_index_config();
        assert_eq!(config.root_dirs, vec!["docs".to_string()]);
        assert!(config.enforce_strict_links);
    }

    #[test]
    fn test_validate_and_reconcile_doc_index_happy() {
        let temp_dir = std::env::temp_dir().join("aios_test_reconcile_doc");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("docs")).unwrap();

        std::fs::write(
            temp_dir.join("docs/README.md"),
            "# Root\n\nLink to [Child](child.md)\n",
        )
        .unwrap();
        std::fs::write(
            temp_dir.join("docs/child.md"),
            "# Child\n\nProse content.\n",
        )
        .unwrap();

        let res = reconcile_doc_index(&temp_dir, &["docs/README.md", "docs/child.md"]);
        assert!(res.is_ok());
        let (manifest, report, telemetry) = res.unwrap();
        assert_eq!(manifest.entries.len(), 2);
        assert!(report.is_valid);
        assert_eq!(telemetry.broken_links_count, 0);
        assert!(telemetry.is_healthy);

        let val_res = validate_doc_index_catalog(&temp_dir, &manifest);
        assert!(val_res.is_ok());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_validate_doc_index_catalog_broken_link_error() {
        let temp_dir = std::env::temp_dir().join("aios_test_validate_broken");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("docs")).unwrap();

        std::fs::write(
            temp_dir.join("docs/README.md"),
            "# Root\n\nLink to [Missing](missing.md)\n",
        )
        .unwrap();

        let manifest = build_doc_index_from_paths(&temp_dir, &["docs/README.md"]).unwrap();
        let val_res = validate_doc_index_catalog(&temp_dir, &manifest);
        assert!(val_res.is_err());
        assert!(val_res.unwrap_err().contains("1 broken link(s) detected"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_reconcile_doc_index_missing_file_error() {
        let temp_dir = std::env::temp_dir().join("aios_test_reconcile_missing");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let res = reconcile_doc_index(&temp_dir, &["docs/non_existent.md"]);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Document not found"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_format_doc_index_summary_happy() {
        let manifest = DocIndexManifest {
            version: "1.0.0".into(),
            entries: vec![
                DocIndexEntry {
                    path: "docs/README.md".into(),
                    title: "Overview".into(),
                    section: "General".into(),
                    task_range: None,
                    links: vec![],
                },
            ],
        };
        let summary = format_doc_index_summary(&manifest);
        assert!(summary.contains("AIOS Documentation Index (v1.0.0):"));
        assert!(summary.contains("[General] Overview (docs/README.md)"));
    }

    #[test]
    fn test_format_doc_index_summary_empty() {
        let manifest = DocIndexManifest::default();
        let summary = format_doc_index_summary(&manifest);
        assert!(summary.contains("AIOS Documentation Index (v1.0.0):"));
        assert!(summary.contains("(no documents indexed)"));
    }

    #[test]
    fn test_format_doc_index_summary_multiple() {
        let manifest = DocIndexManifest {
            version: "2.0.0".into(),
            entries: vec![
                DocIndexEntry {
                    path: "docs/README.md".into(),
                    title: "Overview".into(),
                    section: "General".into(),
                    task_range: None,
                    links: vec![],
                },
                DocIndexEntry {
                    path: "docs/GOALS.md".into(),
                    title: "Goals".into(),
                    section: "Strategy".into(),
                    task_range: None,
                    links: vec![],
                },
            ],
        };
        let summary = format_doc_index_summary(&manifest);
        assert!(summary.contains("AIOS Documentation Index (v2.0.0):"));
        assert!(summary.contains("[General] Overview (docs/README.md)"));
        assert!(summary.contains("[Strategy] Goals (docs/GOALS.md)"));
    }

    #[test]
    fn test_collect_doc_index_telemetry_happy() {
        let manifest = DocIndexManifest {
            version: "1.0.0".into(),
            entries: vec![
                DocIndexEntry {
                    path: "docs/README.md".into(),
                    title: "README".into(),
                    section: "Docs".into(),
                    task_range: None,
                    links: vec!["docs/GOALS.md".into()],
                },
                DocIndexEntry {
                    path: "docs/GOALS.md".into(),
                    title: "GOALS".into(),
                    section: "Docs".into(),
                    task_range: None,
                    links: vec![],
                },
            ],
        };
        let report = DocLinkValidationReport {
            total_links_checked: 1,
            broken_links: vec![],
            is_valid: true,
        };
        let telemetry = collect_doc_index_telemetry(&manifest, Some(&report));
        assert_eq!(telemetry.total_docs_indexed, 2);
        assert_eq!(telemetry.total_links_checked, 1);
        assert_eq!(telemetry.broken_links_count, 0);
        assert!(telemetry.is_healthy);
    }

    #[test]
    fn test_collect_doc_index_telemetry_with_broken_links() {
        let manifest = DocIndexManifest::default();
        let report = DocLinkValidationReport {
            total_links_checked: 2,
            broken_links: vec![BrokenDocLink {
                source_path: "docs/README.md".into(),
                target_link: "non_existent.md".into(),
                reason: "Document not found".into(),
            }],
            is_valid: false,
        };
        let telemetry = collect_doc_index_telemetry(&manifest, Some(&report));
        assert_eq!(telemetry.broken_links_count, 1);
        assert!(!telemetry.is_healthy);
    }

    #[test]
    fn test_collect_doc_index_telemetry_none_report() {
        let manifest = DocIndexManifest {
            version: "1.0.0".into(),
            entries: vec![
                DocIndexEntry {
                    path: "docs/README.md".into(),
                    title: "README".into(),
                    section: "Docs".into(),
                    task_range: None,
                    links: vec!["docs/GOALS.md".into(), "docs/SPEC.md".into()],
                },
            ],
        };
        let telemetry = collect_doc_index_telemetry(&manifest, None);
        assert_eq!(telemetry.total_docs_indexed, 1);
        assert_eq!(telemetry.total_links_checked, 2);
        assert_eq!(telemetry.broken_links_count, 0);
        assert!(telemetry.is_healthy);
    }

    #[test]
    fn test_check_doc_index_policy_enforcement() {
        // Read-only actions pass without a grant
        assert!(check_doc_index_policy(None, "aios.doc.index.get").is_ok());
        assert!(check_doc_index_policy(None, "aios.doc.check").is_ok());
        assert!(check_doc_index_policy(None, "aios.doc.search").is_ok());
        assert!(check_doc_index_policy(None, "doc.show").is_ok());
        assert!(check_doc_index_policy(None, "doc.check").is_ok());
        assert!(check_doc_index_policy(None, "doc.search").is_ok());

        // Irreversible actions fail without a grant
        let err = check_doc_index_policy(None, "aios.doc.set");
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("requires an active PEP grant"));

        let err_empty = check_doc_index_policy(Some(""), "aios.doc.set");
        assert!(err_empty.is_err());

        let err_whitespace = check_doc_index_policy(Some("   \t\n"), "aios.doc.set");
        assert!(err_whitespace.is_err());

        // Irreversible actions pass with a valid grant token
        assert!(check_doc_index_policy(Some("gr_12345678"), "aios.doc.set").is_ok());
        assert!(check_doc_index_policy(Some("gr_12345678"), "doc.set").is_ok());
    }

    #[test]
    fn test_parse_markdown_title_happy() {
        let md = "# Main Header\n\nSome introductory prose.\n## Subheader";
        assert_eq!(parse_markdown_title(md), Some("Main Header".into()));

        let no_h1 = "## Only H2\n### And H3";
        assert_eq!(parse_markdown_title(no_h1), None);
    }

    #[test]
    fn test_parse_markdown_links_happy() {
        let md = r#"
# Title
Here is a [local link](other.md) and [another](sub/doc.md#heading).
External links like [Google](https://google.com) or [Email](mailto:a@b.com) should be ignored.
Also check [anchor only](#anchor) and [bracketed](<special/path.md>).
        "#;
        let links = parse_markdown_links(md);
        assert_eq!(links, vec!["other.md", "sub/doc.md", "special/path.md"]);
    }

    #[test]
    fn test_validate_doc_links_and_build_index() {
        let temp_dir = std::env::temp_dir().join("aios_test_doc_index");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let doc1 = temp_dir.join("README.md");
        std::fs::write(&doc1, "# Readme\n\nSee [Guide](guide.md) and [Missing](missing.md).").unwrap();

        let doc2 = temp_dir.join("guide.md");
        std::fs::write(&doc2, "# Guide\n\nBack to [Readme](README.md).").unwrap();

        let manifest = build_doc_index_from_paths(&temp_dir, &["README.md", "guide.md"]).unwrap();
        assert_eq!(manifest.entries.len(), 2);
        assert_eq!(manifest.entries[0].title, "Readme");
        assert_eq!(manifest.entries[1].title, "Guide");

        let report = validate_doc_links(&temp_dir, &manifest);
        assert_eq!(report.total_links_checked, 3);
        assert!(!report.is_valid);
        assert_eq!(report.broken_links.len(), 1);
        assert_eq!(report.broken_links[0].target_link, "missing.md");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_validate_doc_links_escape_detected() {
        let temp_dir = std::env::temp_dir().join("aios_test_escape");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let doc1 = temp_dir.join("doc.md");
        std::fs::write(&doc1, "# Escape\n\n[Bad](../../../etc/passwd)").unwrap();

        let manifest = build_doc_index_from_paths(&temp_dir, &["doc.md"]).unwrap();
        let report = validate_doc_links(&temp_dir, &manifest);
        assert!(!report.is_valid);
        assert_eq!(report.broken_links.len(), 1);
        assert!(report.broken_links[0].reason.contains("escapes repository root"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_build_doc_index_missing_file_error() {
        let temp_dir = std::env::temp_dir().join("aios_test_missing_doc");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let res = build_doc_index_from_paths(&temp_dir, &["non_existent.md"]);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Document not found"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_real_repo_docs_index_and_validation() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        
        let doc_paths = &["docs/README.md", "docs/SPEC-TASK-LEDGER.md", "docs/tasks/GOALS.md"];
        let manifest = build_doc_index_from_paths(repo_root, doc_paths);
        assert!(manifest.is_ok(), "Manifest load error: {:?}", manifest.err());
        let manifest = manifest.unwrap();
        assert_eq!(manifest.entries.len(), 3);

        let report = validate_doc_links(repo_root, &manifest);
        // All in-tree links in core docs should be checked
        assert!(report.total_links_checked > 0);
    }
}
