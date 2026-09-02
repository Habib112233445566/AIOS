# T-00412 — Documentation Index Control / data model: Specification

## 1. Specification Overview
This document specifies the data model contract and schema for Documentation Index Control in `aiosh-core`.

## 2. Data Types and Schema

### A. `DocIndexEntry`
Represents an individual indexed document or section within the AIOS documentation hierarchy:
- `path`: `String` — Repository-relative path to the markdown document (e.g., `"docs/README.md"`). Cannot be empty.
- `title`: `String` — Human-readable document or section title (e.g., `"Dependency & Toolchain Pinning"`). Cannot be empty.
- `section`: `String` — Structural grouping or Diátaxis category (e.g., `"Phase 0"`, `"Governance"`, `"Architecture"`).
- `task_range`: `Option<String>` — Associated task ledger range if applicable (e.g., `Some("T-00311..T-00410".into())`).
- `links`: `Vec<String>` — In-tree repository-relative paths linked from this document.

### B. `DocIndexManifest`
Represents the complete documentation catalog:
- `version`: `String` — Schema version (e.g. `"1.0.0"`).
- `entries`: `Vec<DocIndexEntry>` — Ordered list of documentation index entries.

## 3. Operations & Methods
1. **`from_json(json_str: &str) -> Result<DocIndexManifest, String>`**:
   - Parses a JSON string into `DocIndexManifest`.
   - Rejects malformed JSON, empty version strings, and entries with empty paths or titles.
2. **`to_json(&self) -> Result<String, String>`**:
   - Serializes the manifest into a formatted JSON string.
3. **`find_entry_by_path(&self, path: &str) -> Option<&DocIndexEntry>`**:
   - Returns the index entry matching the exact relative path.
4. **`find_entries_by_section(&self, section: &str) -> Vec<&DocIndexEntry>`**:
   - Returns all entries belonging to a given section.
5. **`validate(&self) -> Result<(), String>`**:
   - Checks that all entries have valid non-empty fields and there are no duplicate path keys.

## 4. Invariants & Failures
- **Empty Fields**: Returns `Err("DocIndexEntry path cannot be empty")` or `Err("DocIndexEntry title cannot be empty")`.
- **Duplicate Paths**: Returns `Err("Duplicate doc index path: ...")`.
- **Malformed JSON**: Returns `Err("Malformed doc index JSON: ...")`.
