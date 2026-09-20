# Task Evidence: T-01887 - Network Bootstrap / documentation: Security Review

## 1. Overview
- **Task ID**: `T-01887`
- **Sub-Epic**: 9 (Network Bootstrap Documentation Subsystem)
- **Goal**: Perform comprehensive security review and threat modeling of `network_doc.rs` in `aiosh-core`.

---

## 2. Threat Modeling & Risk Assessment

| Threat ID | Category | Description | Severity | Mitigation Status |
|---|---|---|---|---|
| `THREAT-NDOC-01` | Path Traversal / Injection | Untrusted destination paths containing `..`, control characters, or null bytes used in `save_to_path` or `load_from_path`. | High | Defended via `validate_doc_path`: rejects `ParentDir`, control characters, $> 1024$ chars. |
| `THREAT-NDOC-02` | UTF-8 Character Slicing Panic | Direct byte slicing `&topic.summary[..117]` in snippet generation can land on a non-character boundary, causing process panic on multi-byte UTF-8 characters. | High | Needs Hardening: Replace direct byte slice with `char_indices` / `is_char_boundary`. |
| `THREAT-NDOC-03` | CPU Denial of Service | Unbounded query terms or search strings causing excessive matching iterations across topic sections. | Medium | Bounded by `MAX_DOC_QUERY_LEN = 256`, need term count cap ($\le 16$). |
| `THREAT-NDOC-04` | Terminal / Markdown Injection | Malicious or untrusted strings in `NetworkState` (e.g. injected interface names, hostnames, routes) breaking Markdown tables or injecting ANSI escape sequences. | Medium | Needs Hardening: Sanitize state attributes before rendering in Markdown and ASCII topology. |
| `THREAT-NDOC-05` | File Size & Memory Bloat | Reading or writing oversized documentation files exhausting available system memory. | Medium | Defended: `MAX_DOC_FILE_BYTES = 1,048,576` (1 MB) enforced via `metadata` before reading. |
| `THREAT-NDOC-06` | Temporary File Leaks | Incomplete writes or interrupted rename operations leaving abandoned temporary sibling files. | Low | Defended: RAII `TempFileGuard` ensures deletion on drop unless explicitly disarmed. |

---

## 3. Actionable Hardening Plan for T-01888
1. **UTF-8 Slicing Guard**: Replace `&topic.summary[..117]` with safe UTF-8 character boundary scanning (`is_char_boundary` / char iterator) ensuring zero panic vectors on UTF-8 glyphs, emojis, or non-ASCII text.
2. **Search Term Bounds**: Cap `query_terms` to a maximum of 16 whitespace-separated tokens.
3. **Markdown Text Sanitization**: Ensure pipe characters (`|`) and newline/control characters in interface names and hostnames cannot escape Markdown table formatting.
4. **Unit Test Verification**: Add specific test cases for multi-byte UTF-8 string slicing in search snippets and query term limits.

Status: Security review complete. Ready for hardening in `T-01888`.
