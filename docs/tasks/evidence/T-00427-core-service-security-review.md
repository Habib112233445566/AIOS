# T-00427 — Documentation Index Control / core service: Security Review

## 1. Review Scope
This security review assesses the core service functions (`parse_markdown_title`, `parse_markdown_links`, `validate_doc_links`, `build_doc_index_from_paths`) in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs`.

## 2. Threat Scenarios & Mitigations

### 1. Directory Traversal & Root Escape in Link Validation
- **Threat**: A documentation file contains malicious links attempting to probe sensitive host paths outside the repository checkout (e.g., `../../../../etc/passwd` or `C:\Windows\System32`).
- **Mitigation**: `validate_doc_links` performs component-level normalization (`normalize_path`) and canonicalization checks (`canonicalize().starts_with(&root_canon)`). Escaping paths are intercepted, flagged in `broken_links`, and prevented from causing information disclosure.

### 2. Denial of Service via Resource Exhaustion
- **Threat**: Ingesting enormous or streaming files to cause out-of-memory crashes.
- **Mitigation**: `build_doc_index_from_paths` enforces a strict 16 MiB read cap (`take(MAX_DOC_BYTES)`) on every file opened.

### 3. Untrusted Protocol Ingestion
- **Threat**: Markdown containing `javascript:`, `file://`, or malicious URI schemes executing in downstream UI/viewers.
- **Mitigation**: Link parser ignores non-relative schemes (`http://`, `https://`, `mailto:`, `ftp://`) and extracts purely clean filesystem targets.

## 3. Conclusion
No known security bypass remains open. Core service functions are read-only, strictly bounded, and tamper-resistant.
