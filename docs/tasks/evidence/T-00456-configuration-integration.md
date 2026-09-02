# T-00456 — Documentation Index Control / configuration: Integration

## 1. Integration Scope
This task integrates `DocIndexConfig` across both the CLI (`aiosh doc --config <path>`) and MCP API surfaces (`aios.doc.*` `config_path` argument).

## 2. Integration Pathways
- **CLI Integration**:
  - `aiosh doc` supports `--config <path>` override with environment variable fallback (`AIOS_DOC_INDEX_CONFIG`).
  - Graceful reporting of non-existent or corrupted configs with exit code 1.
- **MCP Tool Integration**:
  - `aios.doc.index.get`, `aios.doc.check`, and `aios.doc.search` support `config_path` parameter.
- **Cross-Substrate Validation**:
  - Validated by end-to-end smoke suites across CLI and MCP interfaces.

## 3. Verification Results
- `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core doc_index_config` -> PASS (5/5 tests)
- `python code/aiosh-cli/tests/test_doc_cli_smoke.py` -> PASS (11/11 tests)
- `python code/aiosh-mcp/tests/test_doc_mcp_smoke.py` -> PASS (5/5 tests)
