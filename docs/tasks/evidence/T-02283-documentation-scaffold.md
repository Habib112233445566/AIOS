# T-02283 Scaffold: Grant Lifecycle Documentation

**Task:** Create the module skeleton and interfaces for the documentation of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Documentation  

---

## 1. What Was Created

1. **Source File:** `code/aiosh-rust/aiosh-core/src/pep_grant_doc.rs`
   - Defines `PepGrantDocCategory` (`Architecture`, `Lifecycle`, `Attenuation`, `Revocation`, `Policy`, `Observability`, `Reference`).
   - Defines `PepGrantDocSection`, `PepGrantDocTopic`, and `PepGrantDocSearchResult`.
   - Defines `PepGrantDocIndex` with:
     - `list_topics(&self) -> &[PepGrantDocTopic]`
     - `get_topic(&self, id: &str) -> Option<&PepGrantDocTopic>`
     - `search(&self, query: &str) -> Result<Vec<PepGrantDocSearchResult>, String>`
     - `render_markdown(&self, topic_id: &str) -> Result<String, String>`
   - Populated with 7 canonical grant topics: `grant-arch`, `grant-lifecycle`, `grant-attenuation`, `grant-revocation`, `grant-policy`, `grant-observability`, `grant-mcp`.
   - Constants: `MAX_GRANT_DOC_QUERY_LEN` (128), `MAX_GRANT_DOC_SEARCH_RESULTS` (10), `MAX_GRANT_DOC_SNIPPET_LEN` (200), `GRANTDOC_ERR_*`.

2. **Module Integration:**
   - Registered `pub mod pep_grant_doc;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
   - Re-exported core structs and constants in `aiosh_core`.

---

## 2. Acceptance Verification
- ✅ Module skeleton and interfaces created.
- ✅ Re-exported and registered in `lib.rs`.
- ✅ Compiles cleanly with zero errors under `cargo check -p aiosh-core`.
