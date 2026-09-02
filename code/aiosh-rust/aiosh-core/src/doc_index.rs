//! Documentation Index Control data model (T-00414).
//!
//! Contract: `docs/tasks/evidence/T-00412-data-model-specification.md`.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocIndexEntry {
    pub path: String,
    pub title: String,
    pub section: String,
    pub task_range: Option<String>,
    pub links: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocIndexManifest {
    pub version: String,
    pub entries: Vec<DocIndexEntry>,
}

impl Default for DocIndexManifest {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            entries: Vec::new(),
        }
    }
}

impl DocIndexManifest {
    /// Deserializes and validates a DocIndexManifest from a JSON string (T-00414).
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let manifest: DocIndexManifest = serde_json::from_str(json_str)
            .map_err(|e| format!("Malformed doc index JSON: {e}"))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validates and serializes a DocIndexManifest into a pretty-printed JSON string (T-00414).
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize doc index manifest: {e}"))
    }

    /// Finds a document entry by repository-relative path (T-00414).
    pub fn find_entry_by_path(&self, path: &str) -> Option<&DocIndexEntry> {
        self.entries.iter().find(|e| e.path == path)
    }

    /// Finds all document entries under a given section (T-00414).
    pub fn find_entries_by_section(&self, section: &str) -> Vec<&DocIndexEntry> {
        self.entries.iter().filter(|e| e.section == section).collect()
    }

    /// Validates the structural integrity, mandatory fields, and uniqueness of entries (T-00414 / T-00418).
    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() {
            return Err("DocIndexManifest version cannot be empty".into());
        }
        if self.entries.len() > 10_000 {
            return Err(format!(
                "DocIndexManifest exceeds maximum allowed entries (limit 10000, got {})",
                self.entries.len()
            ));
        }
        let mut seen_paths = HashSet::new();
        for entry in &self.entries {
            if entry.path.trim().is_empty() {
                return Err("DocIndexEntry path cannot be empty".into());
            }
            if entry.title.trim().is_empty() {
                return Err("DocIndexEntry title cannot be empty".into());
            }
            if entry.section.trim().is_empty() {
                return Err("DocIndexEntry section cannot be empty".into());
            }
            if entry.links.len() > 1_000 {
                return Err(format!(
                    "DocIndexEntry '{}' exceeds maximum allowed links (limit 1000, got {})",
                    entry.path,
                    entry.links.len()
                ));
            }
            if !seen_paths.insert(&entry.path) {
                return Err(format!("Duplicate doc index path: {}", entry.path));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_index_manifest_roundtrip_happy() {
        let manifest = DocIndexManifest {
            version: "1.0.0".into(),
            entries: vec![
                DocIndexEntry {
                    path: "docs/README.md".into(),
                    title: "Main Documentation".into(),
                    section: "Overview".into(),
                    task_range: Some("T-00001..T-00500".into()),
                    links: vec!["docs/tasks/GOALS.md".into()],
                },
                DocIndexEntry {
                    path: "docs/tasks/GOALS.md".into(),
                    title: "Goals & Laws".into(),
                    section: "Governance".into(),
                    task_range: None,
                    links: vec![],
                },
            ],
        };

        let json_str = manifest.to_json().unwrap();
        let loaded = DocIndexManifest::from_json(&json_str).unwrap();
        assert_eq!(manifest, loaded);
        assert_eq!(loaded.entries.len(), 2);
    }

    #[test]
    fn test_doc_index_manifest_empty_version_fails() {
        let manifest = DocIndexManifest {
            version: "".into(),
            entries: vec![],
        };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_doc_index_manifest_empty_path_fails() {
        let manifest = DocIndexManifest {
            version: "1.0.0".into(),
            entries: vec![DocIndexEntry {
                path: "".into(),
                title: "Title".into(),
                section: "Sec".into(),
                task_range: None,
                links: vec![],
            }],
        };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_doc_index_manifest_duplicate_path_fails() {
        let manifest = DocIndexManifest {
            version: "1.0.0".into(),
            entries: vec![
                DocIndexEntry {
                    path: "docs/README.md".into(),
                    title: "Doc 1".into(),
                    section: "Sec 1".into(),
                    task_range: None,
                    links: vec![],
                },
                DocIndexEntry {
                    path: "docs/README.md".into(),
                    title: "Doc 2".into(),
                    section: "Sec 2".into(),
                    task_range: None,
                    links: vec![],
                },
            ],
        };
        let err = manifest.validate().unwrap_err();
        assert!(err.contains("Duplicate doc index path"));
    }

    #[test]
    fn test_doc_index_manifest_query_helpers() {
        let manifest = DocIndexManifest {
            version: "1.0.0".into(),
            entries: vec![
                DocIndexEntry {
                    path: "docs/README.md".into(),
                    title: "Doc 1".into(),
                    section: "Phase 0".into(),
                    task_range: None,
                    links: vec![],
                },
                DocIndexEntry {
                    path: "docs/SPEC.md".into(),
                    title: "Doc 2".into(),
                    section: "Phase 0".into(),
                    task_range: None,
                    links: vec![],
                },
                DocIndexEntry {
                    path: "docs/OTHER.md".into(),
                    title: "Doc 3".into(),
                    section: "Phase 1".into(),
                    task_range: None,
                    links: vec![],
                },
            ],
        };

        let found = manifest.find_entry_by_path("docs/SPEC.md");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Doc 2");

        let p0_entries = manifest.find_entries_by_section("Phase 0");
        assert_eq!(p0_entries.len(), 2);

        let missing = manifest.find_entry_by_path("docs/NONEXISTENT.md");
        assert!(missing.is_none());
    }

    #[test]
    fn test_doc_index_manifest_empty_title_fails() {
        let manifest = DocIndexManifest {
            version: "1.0.0".into(),
            entries: vec![DocIndexEntry {
                path: "docs/README.md".into(),
                title: "   ".into(),
                section: "Sec".into(),
                task_range: None,
                links: vec![],
            }],
        };
        let err = manifest.validate().unwrap_err();
        assert!(err.contains("DocIndexEntry title cannot be empty"));
    }

    #[test]
    fn test_doc_index_manifest_empty_section_fails() {
        let manifest = DocIndexManifest {
            version: "1.0.0".into(),
            entries: vec![DocIndexEntry {
                path: "docs/README.md".into(),
                title: "Doc".into(),
                section: "".into(),
                task_range: None,
                links: vec![],
            }],
        };
        let err = manifest.validate().unwrap_err();
        assert!(err.contains("DocIndexEntry section cannot be empty"));
    }

    #[test]
    fn test_doc_index_manifest_malformed_json_fails() {
        let res = DocIndexManifest::from_json("{ not valid json");
        assert!(res.is_err());
    }

    #[test]
    fn test_doc_index_manifest_default_is_valid() {
        let manifest = DocIndexManifest::default();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_doc_index_manifest_links_limit_fails() {
        let excessive_links = (0..1_001).map(|i| format!("link_{i}.md")).collect();
        let manifest = DocIndexManifest {
            version: "1.0.0".into(),
            entries: vec![DocIndexEntry {
                path: "docs/OVERFLOW.md".into(),
                title: "Doc".into(),
                section: "Sec".into(),
                task_range: None,
                links: excessive_links,
            }],
        };
        let err = manifest.validate().unwrap_err();
        assert!(err.contains("exceeds maximum allowed links"));
    }
}
