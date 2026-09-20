# Task Evidence: T-01984 - System Update / documentation: Implementation (Sub-Epic 9)

## 1. Overview
- **Task ID**: `T-01984`
- **Sub-Epic**: 9 (System Update Documentation Subsystem)
- **Goal**: Implement the complete working behavior for `system_update_doc.rs`, supporting offline reference topic lookup, scored full-text search, dynamic status and observability Markdown rendering, and secure file export.

---

## 2. Implementation Details
- **Module**: `code/aiosh-rust/aiosh-core/src/system_update_doc.rs`.
- **Key Capabilities**:
  1. **Canonical Offline Repository (`new()`)**:
     - Pre-populated with 6 core topics across all categories (`arch-overview`, `ab-slots`, `security-policy`, `observability-telemetry`, `config-schema`, `troubleshooting-rollback`).
  2. **Topic Navigation (`get_by_id()`, `list_by_category()`)**:
     - Constant-time and filtered lookup by topic ID and category.
  3. **Ranked Keyword Search (`search()`)**:
     - Tokenized multi-keyword evaluation with weighted scoring:
       - Exact ID match: +100
       - ID substring: +50
       - Title substring: +40
       - Tag match: +20
       - Content match: +5
     - Generates 120-character preview snippets.
  4. **Dynamic Markdown Rendering**:
     - `render_full_index_markdown()`: Complete catalog generation.
     - `render_status_markdown()`: Live slot status and ASCII slot diagram from `SystemUpdateService`.
     - `render_observability_markdown()`: Executive telemetry summary from `SystemUpdateObservabilityReport`.
  5. **Defensive File Export (`export_to_file()`)**:
     - Symlink defense via `symlink_metadata()`.
     - Max file size bounded at 1 MB.
     - Atomic persistence using `.tmp.<pid>` pattern with cleanup on failure.
