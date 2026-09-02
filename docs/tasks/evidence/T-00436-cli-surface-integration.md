# T-00436 — Documentation Index Control / CLI surface: Integration

## 1. Integration Scope
This task verifies the integration of the `aiosh doc` command with the Rust CLI binary (`aiosh`), the core library (`aiosh-core`), and the repository documentation topology.

## 2. Integration Pathways
- **Binary Dispatch**:
  - `main.rs` dispatches `aiosh doc <show|check|search>` through `cmd_doc`.
  - Supports flags `--json` and `--repo <path>`.
- **Core Library Integration**:
  - Calls `aiosh_core::doc_index_service::build_doc_index_from_paths` and `aiosh_core::doc_index_service::validate_doc_links`.
- **Audit Logging**:
  - Emits audit records (`doc.show`, `doc.check`, `doc.search`) to the local audit ring.

## 3. Verification Results
- `cargo test -p aiosh-cli test_cmd_doc_show_check_and_search` -> PASS
- `python code/aiosh-cli/tests/test_doc_cli_smoke.py` -> PASS (9/9 tests)
