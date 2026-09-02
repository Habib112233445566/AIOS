# T-00422 — Documentation Index Control / core service: Specification

## 1. Specification Overview
This document specifies the exact contract, API signatures, error conditions, and report types for the Documentation Index Control core service in `aiosh-core`.

## 2. Core Types

### A. `BrokenDocLink`
Represents an unresolved or invalid documentation link reference:
- `source_path`: `String` — Repository-relative path of the markdown document containing the link.
- `target_link`: `String` — Raw link target string (e.g., `"../nonexistent.md"`).
- `reason`: `String` — Failure explanation (e.g., `"File does not exist on disk"`, `"Link escapes repository root"`).

### B. `DocLinkValidationReport`
Summary report generated when validating document links:
- `total_links_checked`: `usize` — Total count of internal links inspected.
- `broken_links`: `Vec<BrokenDocLink>` — List of broken or out-of-bounds link references.
- `is_valid`: `bool` — `true` if `broken_links` is empty, `false` otherwise.

## 3. Core Service Operations

### 1. `parse_markdown_title(content: &str) -> Option<String>`
- Scans raw markdown text for the first top-level `# Title` heading.
- Returns `Some(String)` with trimmed title prose, or `None` if no top-level header exists.

### 2. `parse_markdown_links(content: &str) -> Vec<String>`
- Scans raw markdown text for CommonMark inline link patterns `[text](target)`.
- Ignores external schemes (`http://`, `https://`, `mailto:`) and standalone anchor hashes (`#section`).
- Returns a deduplicated vector of in-tree target paths.

### 3. `validate_doc_links(repo_root: &Path, manifest: &DocIndexManifest) -> DocLinkValidationReport`
- Iterates over all `entries` in `manifest` and resolves each entry's `links` relative to `entry.path`.
- Verifies physical file existence on disk within `repo_root`.
- Detects path escapes (e.g. `../../etc/passwd`).
- Emits a structured `DocLinkValidationReport`.

### 4. `build_doc_index_from_paths(repo_root: &Path, doc_paths: &[&str]) -> Result<DocIndexManifest, String>`
- Reads each specified markdown file bounded by `MAX_DOC_BYTES` (16MB).
- Extracts title, section categorization, and links.
- Returns a validated `DocIndexManifest`.

## 4. PEP & Audit Policy
- Core service inspection and validation routines are read-only diagnostics and safe for concurrent execution by operators and MCP agents.
