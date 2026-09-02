# T-00428 — Documentation Index Control / core service: Hardening

## 1. Hardening Scope
This task verifies and documents hardening constraints for the Documentation Index Control core service in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs`.

## 2. Hardening Measures
1. **Document Read Size Cap (`MAX_DOC_BYTES = 16 MiB`)**:
   - `build_doc_index_from_paths` enforces a 16 MiB read ceiling via `file.by_ref().take(MAX_DOC_BYTES)`, matching the security ceiling of `tools/check_task_docs.py`.
2. **Defensive Path Normalization**:
   - `normalize_path` collapses redundant separators, `./`, and `../` components, ensuring that relative link traversal outside repository boundaries is caught deterministically across platforms (Windows, Linux, macOS).
3. **Structured Diagnostics & Error Propagation**:
   - File errors, missing documents, and broken link reasons are never swallowed; they bubble cleanly through `Result<DocIndexManifest, String>` and `DocLinkValidationReport`.
4. **Zero Resource Leaks**:
   - All file handles close immediately on drop with no leaked descriptors or lingering handles.

## 3. Acceptance Verification
- All failure modes produce explicit, auditable reports.
- Memory and file descriptor consumption are bounded.
