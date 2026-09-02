# T-00506 — Documentation Index Control / recovery & validation: Integration

## 1. Integration Scope
This task integrates `reconcile_doc_index` into `aiosh doc check` and MCP `aios.doc.check`, providing unified multi-document loading, link verification, and telemetry generation across CLI and server surfaces.

## 2. Integrated Components
- `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - `aiosh doc check` utilizes `reconcile_doc_index(repo_root, default_docs)` to parse manifests, test links, and collect telemetry in a unified invocation.
- `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - `aios.doc.check` uses `reconcile_doc_index(repo, default_docs)` to return full validation and telemetry structures in the JSON-RPC response.

## 3. Verification
- `cargo test --workspace` -> 130 unit tests in `aiosh_core`, 2 in `aiosh_mcp`.
- `python code/aiosh-cli/tests/test_doc_cli_smoke.py` -> PASS.
- `python code/aiosh-mcp/tests/test_doc_mcp_smoke.py` -> PASS.
