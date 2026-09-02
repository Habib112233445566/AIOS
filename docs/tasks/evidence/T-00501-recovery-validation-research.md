# T-00501 — Documentation Index Control / recovery & validation: Research

## 1. Goal
Establish facts, failure/drift scenarios, automated validation mechanisms, and recovery strategies for Documentation Index Control in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Current Implementation):
1. **Drift & Failure Modes**:
   - Missing or deleted documentation files referenced in default lists or configuration anchors.
   - Broken markdown relative links pointing to non-existent markdown files or invalid anchors.
   - Directory traversal attempts where links escape the repository root (`../..`).
   - Corrupted or unparseable configuration files (`docs/doc_index_config.json`).
   - Empty or oversized documents exceeding ingestion limits (16 MiB).
2. **Validation Capabilities**:
   - Schema validation: `DocIndexManifest::from_json` and `DocIndexConfig::from_path` validate structural integrity and parameter bounds.
   - Link integrity validation: `validate_doc_links` parses in-tree links, resolves relative paths, checks file existence, and detects directory traversal.
3. **Recovery Capabilities**:
   - Default configuration fallback: When configuration files are missing or unreadable, `DocIndexConfig::from_env` falls back to in-memory `DocIndexConfig::default()`.
   - Idempotent catalog reconstruction: `build_doc_index_from_paths` re-indexes raw files directly from disk without persisting corrupted state.

### Assumptions:
1. Automated link diagnostics (`BrokenDocLink`) should provide exact source line, target link, and remediation suggestions to assist autonomous agents in self-healing broken documentation.
2. Fallback to default index configurations should never crash daemon services or block CLI startup.

## 3. Prior Art & Authoritative Standards
- **Markdown Link Verification Tools (Lychee / markdown-link-check)**: Standards for parsing Markdown reference links and classifying broken targets.
- **Twelve-Factor App §IX (Disposability & Fast Startup)**: Resilient fallback to compile-time configuration defaults upon storage corruption.
- **OpenSSF Scorecard (Documentation Integrity Checks)**: Continuous validation of security references and repository guides.

## 4. Decisions Needed
1. **Recovery Helper Standardization**: Provide a dedicated `recover_default_doc_index_config()` helper and `validate_doc_index_catalog()` verification endpoint in `aiosh-core::doc_index_service`.
2. **Actionable Remediation Formatting**: Ensure `BrokenDocLink` reports include clear diagnostic reasons (`"Document not found"`, `"Target link escapes repository root"`).

## 5. Next Steps
Advance to Specification (T-00502) to formalize the recovery APIs and validation contracts.
