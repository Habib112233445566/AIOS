# T-00426 — Documentation Index Control / core service: Integration

## 1. Integration Scope
This task integrates the Documentation Index Control core service (`doc_index_service.rs`) with the file system and real repository documentation tree.

## 2. Integration Details
- **Module Exposure**: Exported `pub mod doc_index_service;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- **Real Repository Verification**:
  - Validated that `build_doc_index_from_paths` can scan real repository files (`docs/README.md`, `docs/SPEC-TASK-LEDGER.md`, `docs/tasks/GOALS.md`) and validate in-tree link graph resolution without crashing.
- **Cross-Substrate Parity**:
  - Report types (`BrokenDocLink`, `DocLinkValidationReport`) serialize to canonical JSON for MCP/CLI compatibility.

## 3. Verification
- `cargo test -p aiosh-core doc_index_service::tests` -> PASS (6/6 tests)
- `cargo check --workspace` -> PASS (Exit 0)
